use std::pin::{pin, Pin};

use std::task::{Context, Poll};

use file_context::{FileContext, FileDataLocation, FileSpan};
use futures::FutureExt;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, Result};
use tokio_stream::Stream;

mod comment;
mod ident;
mod intlit;
mod keyword;
mod stringlit;
mod symbol;

pub use comment::*;
pub use ident::*;
pub use intlit::*;
pub use keyword::*;
pub use stringlit::*;
pub use symbol::*;

use crate::xml::IntoXML;

type PinnedInputReader = Pin<Box<dyn AsyncRead>>;

pub struct JackTokenizer {
    reader: PinnedInputReader,
    buffer: [u8; 1024],
    cursor: usize,
    cursor_abs: usize,
    len: usize,
    eof: bool,
    skip_comments: bool,
    token_idx: usize,
    line: usize,
    symbol: usize,
}

impl JackTokenizer {
    pub fn new(reader: PinnedInputReader, skip_comments: bool) -> Self {
        let buffer = [0; 1024];
        Self {
            reader,
            buffer,
            cursor: 0,
            cursor_abs: 0,
            len: 0,
            token_idx: 0,
            line: 0,
            symbol: 0,
            eof: false,
            skip_comments,
        }
    }

    pub fn from_file(file: File, skip_comments: bool) -> Self {
        Self::new(Box::pin(file), skip_comments)
    }

    async fn fill_buff(&mut self) -> Result<()> {
        let to_copy = self.len - self.cursor;

        if to_copy != 0 {
            let p = self.buffer.as_mut_ptr();

            if self.cursor > to_copy {
                unsafe {
                    std::ptr::copy_nonoverlapping(p.add(self.cursor), p, to_copy);
                }
            } else {
                unsafe { std::ptr::copy(p.add(self.cursor), p, to_copy) }
            }
        }

        let n = self.reader.read(&mut self.buffer[to_copy..]).await?;
        self.cursor_abs += self.cursor;
        self.cursor = to_copy;
        self.len = n + to_copy;
        self.eof = n == 0;
        Ok(())
    }

    async fn next_token(&mut self) -> Option<FileContext<JackToken>> {
        loop {
            if self.eof {
                return None;
            };

            if self.cursor == self.len {
                self.fill_buff().await.expect("Fill buffer error");
            };

            let (trim_size, trim_lines, trim_symbols) =
                trim_start(&self.buffer[self.cursor..self.len], self.symbol);
            self.symbol = trim_symbols;
            self.line += trim_lines;
            self.cursor += trim_size;

            let (maybe_token_payload, token_size, lines, terminator) =
                JackToken::bytes_to_token(&self.buffer[self.cursor..self.len]);

            let token = maybe_token_payload.map(|token_payload| {
                let location = FileDataLocation::new(self.cursor_abs + self.cursor, token_size);
                let span = FileSpan::new(self.line, trim_symbols);
                FileContext::new(token_payload, self.token_idx, Some(location), Some(span))
            });

            self.symbol += token_size;

            self.line += lines;

            if terminator {
                self.cursor += token_size;

                if !self.skip_comments {
                    self.token_idx += 1;
                    return token;
                }

                if let Some(FileContext {
                    payload: JackToken::Comment(_),
                    ..
                }) = token
                {
                    continue;
                } else {
                    self.token_idx += 1;
                    return token;
                }
            } else {
                self.fill_buff().await.expect("Fill buffer error");
            }
        }
    }
}

impl Stream for JackTokenizer {
    type Item = FileContext<JackToken>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.eof {
            Poll::Ready(None)
        } else {
            let mut next_future = pin!(self.get_mut().next_token());
            next_future.poll_unpin(cx)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum JackToken {
    Keyword(JackKeyword),
    Symbol(JackSymbol),
    IntLiteral(JackInt),
    StringLiteral(JackString),
    Comment(JackComment),
    Ident(JackIdent),
}

impl IntoXML for JackToken {
    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize> {
        let res = match self {
            JackToken::Comment(_) => 0,
            JackToken::Ident(s) => s.write_xml(write).await?,
            JackToken::Symbol(s) => s.write_xml(write).await?,
            JackToken::Keyword(s) => s.write_xml(write).await?,
            JackToken::IntLiteral(s) => s.write_xml(write).await?,
            JackToken::StringLiteral(s) => s.write_xml(write).await?,
        };

        Ok(res)
    }
}

fn trim_start(buff: &[u8], s: usize) -> (usize, usize, usize) {
    let mut cursor = 0;
    let mut lines = 0;
    let mut symbols = s;

    loop {
        match buff.get(cursor) {
            Some(b' ') => symbols += 1,
            Some(b'\n') => {
                lines += 1;
                symbols = 0;
            }
            Some(b'\r') => (),
            Some(b'\t') => symbols += 1,
            _ => break,
        }
        cursor += 1;
    }

    (cursor, lines, symbols)
}

impl JackToken {
    pub fn bytes_to_token<'a>(buff: &'a [u8]) -> (Option<Self>, usize, usize, bool) {
        let cursor = 0;

        match JackComment::parse_comment(&buff[cursor..]) {
            (None, 0, _, true) => (),
            (comment, comment_len, lines, terminator) => {
                return (
                    comment.map(|x| Self::Comment(x)),
                    comment_len + cursor,
                    lines,
                    terminator,
                )
            }
        };

        let c: u8 = if let Some(c2) = buff.get(cursor) {
            *c2
        } else {
            return (None, cursor, 0, false);
        };

        if let Some(symbol) = JackSymbol::char_to_symbol(c) {
            return (Some(JackToken::Symbol(symbol)), cursor + 1, 0, true);
        }

        if c == b'"' {
            let (answer, literal_len) = JackString::parse_string_literal(&buff[(cursor + 1)..]);
            let with_terminator = answer.is_some();
            return (
                answer.map(|x| Self::StringLiteral(x)),
                literal_len + cursor + 1,
                0,
                with_terminator,
            );
        }

        if JackInt::is_int_char(c) {
            let (answer, literal_len, with_terminator) =
                JackInt::parse_int_literal(&buff[cursor..]);
            return (
                Some(Self::IntLiteral(answer)),
                literal_len + cursor,
                0,
                with_terminator,
            );
        }

        if let Some(keyword) = JackKeyword::bytes_to_keyword(&buff[cursor..]) {
            let keyword_size = keyword.size();
            match buff.get(cursor + keyword_size) {
                Some(x) if JackIdent::is_ident_char(*x) => (),
                Some(x) if JackInt::is_int_char(*x) => (),
                _ => {
                    return (
                        Some(JackToken::Keyword(keyword)),
                        cursor + keyword_size,
                        0,
                        true,
                    );
                }
            }
        }

        let (result, ident_size, with_terminator) = JackIdent::parse_ident(&buff[cursor..]);
        (
            Some(Self::Ident(result)),
            ident_size + cursor,
            0,
            with_terminator,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn test_let_statement_manually() {
        let buff = b"let pointX = 94;";
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(buff);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Keyword(JackKeyword::Let)));
        assert_eq!(token_size, 3);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[4..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Ident(JackIdent(b"pointX".to_vec()))));
        assert_eq!(token_size, 6);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[11..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Symbol(JackSymbol::Eq)));
        assert_eq!(token_size, 1);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[13..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::IntLiteral(JackInt(b"94".to_vec()))));
        assert_eq!(token_size, 2);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[15..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Symbol(JackSymbol::Semicolon)));
        assert_eq!(token_size, 1);
    }

    #[test]
    fn test_ident_like_keyword_manually() {
        let buff = b"let lett_x = 94;";
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(buff);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Keyword(JackKeyword::Let)));
        assert_eq!(token_size, 3);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[4..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Ident(JackIdent(b"lett_x".to_vec()))));
        assert_eq!(token_size, 6);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[11..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Symbol(JackSymbol::Eq)));
        assert_eq!(token_size, 1);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[13..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::IntLiteral(JackInt(b"94".to_vec()))));
        assert_eq!(token_size, 2);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[15..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Symbol(JackSymbol::Semicolon)));
        assert_eq!(token_size, 1);
    }

    #[test]
    fn test_ident_without_terminator() {
        let buff = b"lett_x";
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(buff);
        assert!(!with_terminator);
        assert_eq!(token, Some(JackToken::Ident(JackIdent(b"lett_x".to_vec()))));
        assert_eq!(token_size, 6);
    }

    #[test]
    fn test_int_without_terminator() {
        let buff = b"199";
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(buff);
        assert!(!with_terminator);
        assert_eq!(token, Some(JackToken::IntLiteral(JackInt(b"199".to_vec()))));
        assert_eq!(token_size, 3);
    }

    #[test]
    fn test_int_with_terminator() {
        let buff = b"199 ";
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(buff);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::IntLiteral(JackInt(b"199".to_vec()))));
        assert_eq!(token_size, 3);
    }

    #[test]
    fn test_symbol() {
        let buff = b"  \n+";
        let (trimed, _, _) = trim_start(buff, 0);
        let (token, token_size, _, with_terminator) = JackToken::bytes_to_token(&buff[trimed..]);
        assert!(with_terminator);
        assert_eq!(token, Some(JackToken::Symbol(JackSymbol::Plus)));
        assert_eq!(token_size, 1);
    }

    #[tokio::test]
    async fn test_arrays_from_file() {
        let file = File::open("./priv/ArrayTest.jack").await.unwrap();

        let mut tokenizer = JackTokenizer::from_file(file, true);

        let buff_tokens = vec![
            // JackToken::Comment(JackComment(b" This file is part of www.nand2tetris.org".to_vec())),
            // JackToken::Comment(JackComment(b" and the book \"The Elements of Computing Systems\"".to_vec())),
            // JackToken::Comment(JackComment(b" by Nisan and Schocken, MIT Press.".to_vec())),
            // JackToken::Comment(JackComment(b" File name: projects/10/ArrayTest/Main.jack".to_vec())),
            // JackToken::Comment(JackComment(b" (identical to projects/09/Average/Main.jack)".to_vec())),
            // JackToken::Comment(JackComment(b" Computes the average of a sequence of integers. ".to_vec())),
            JackToken::Keyword(JackKeyword::Class),
            JackToken::Ident(JackIdent(b"Main".to_vec())),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Function),
            JackToken::Keyword(JackKeyword::Void),
            JackToken::Ident(JackIdent(b"main".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Ident(JackIdent(b"Array".to_vec())),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Keyword(JackKeyword::Int),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Keyword(JackKeyword::Int),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Comma),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Keyboard".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"readInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"HOW MANY NUMBERS? ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Array".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"new".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::While),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Less),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::OpenSquareBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::CloseSquareBracket),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Keyboard".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"readInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"ENTER THE NEXT NUMBER: ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::IntLiteral(JackInt(b"1".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::While),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Less),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::OpenSquareBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::CloseSquareBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::IntLiteral(JackInt(b"1".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"printString".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"THE AVERAGE IS: ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"printInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Divide),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"println".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Return),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
        ];

        for (idx, token) in buff_tokens.into_iter().enumerate() {
            let parsed_token = tokenizer.next().await.unwrap();
            let s = parsed_token.span.unwrap();
            let line = s.line + 1;
            let symbol = s.symbol + 1;

            assert_eq!(parsed_token.idx, idx);
            assert_eq!(
                token, parsed_token.payload,
                "({line}:{symbol}) Error in instruction: {idx}"
            );
        }

        let last_token = tokenizer.next().await;

        assert!(last_token.is_none());
    }
}
