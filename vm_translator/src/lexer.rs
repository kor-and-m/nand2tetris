use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

use crate::tokens::*;

const LEXER_BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct VMLexer {
    file: File,
    buffer: [u8; LEXER_BUFFER_SIZE],
    cursor: usize,
    end_word_cursor: usize,
    end_page_cursor: usize,
    is_eof: bool,
    src_line: usize,
    instruction_number: usize,
    is_in_progress: bool,
}

impl VMLexer {
    pub async fn new(path: &str) -> io::Result<Self> {
        let file_path = Path::new(&path);
        if file_path.extension().expect("File extension undefined") != "vm" {
            return Err(Error::new(ErrorKind::Other, "File ext should be vm!"));
        }

        let file = File::open(&file_path).await?;

        let mut self_state = Self {
            file,
            buffer: [0; LEXER_BUFFER_SIZE],
            cursor: LEXER_BUFFER_SIZE,
            end_word_cursor: LEXER_BUFFER_SIZE,
            end_page_cursor: 0,
            is_eof: false,
            is_in_progress: false,
            instruction_number: 0,
            src_line: 1,
        };
        self_state.fill_buffer().await?;
        Ok(self_state)
    }

    async fn fill_buffer(&mut self) -> io::Result<()> {
        let to_copy = LEXER_BUFFER_SIZE - self.cursor;

        // becouse token size can't be bigger than LEXER_BUFFER_SIZE / 2
        let p = self.buffer.as_mut_ptr();
        unsafe {
            std::ptr::copy_nonoverlapping(p.add(self.cursor), p, to_copy);
        }

        let n = self.file.read(&mut self.buffer[to_copy..]).await?;
        self.is_eof = self.cursor > n;
        self.end_word_cursor -= self.cursor;
        self.cursor = 0;
        self.end_page_cursor = n;
        Ok(())
    }

    async fn next_word(&mut self) -> Option<&[u8]> {
        let mut is_comment = false;
        let mut is_word = false;
        self.cursor = self.end_word_cursor;

        loop {
            if self.end_word_cursor >= self.end_page_cursor {
                if self.is_eof {
                    return if is_word {
                        Some(&self.buffer[self.cursor..self.end_word_cursor])
                    } else {
                        None
                    };
                } else {
                    self.fill_buffer().await.expect("fail to fill buffer");
                }
            }

            match (is_comment, is_word, self.buffer[self.end_word_cursor]) {
                (true, _, b'\n') => {
                    self.src_line += 1;
                    is_comment = false;
                    self.cursor += 1;
                    self.end_word_cursor += 1;
                }
                (true, _, _) => {
                    self.cursor += 1;
                    self.end_word_cursor += 1;
                }
                (false, _, b'/') => {
                    if self.buffer[self.end_word_cursor + 1] != b'/' {
                        panic!("Comment parsing error")
                    }
                    self.cursor += 2;
                    self.end_word_cursor += 2;
                    is_comment = true;
                }
                (false, false, v) if v == b' ' || v == b'\t' || v == b'\r' => {
                    self.cursor += 1;
                    self.end_word_cursor += 1;
                }
                (false, false, b'\n') => {
                    self.src_line += 1;
                    self.cursor += 1;
                    self.end_word_cursor += 1;
                }
                (false, false, _) => is_word = true,
                (false, true, v) if v == b' ' || v == b'\t' || v == b'\r' || v == b'\n' => {
                    if v == b'\n' {
                        self.src_line += 1;
                    }
                    return Some(&self.buffer[self.cursor..self.end_word_cursor]);
                }
                (false, true, _) => self.end_word_cursor += 1,
            }
        }
    }

    async fn build_memory_token(&mut self, kind: MemoryTokenKind) -> Option<TokenPayload> {
        let word = self.next_word().await.unwrap();
        let segment = SEGMENTS[word];
        let val: i16 = self
            .next_word()
            .await
            .unwrap()
            .iter()
            .fold(0, |v, x| v * 10 - 48 + *x as i16);

        Some(TokenPayload::Memory(MemoryToken { kind, segment, val }))
    }

    pub async fn next_token(&mut self) -> Option<Token> {
        self.is_in_progress = true;
        self.instruction_number += 1;

        let src_line = self.src_line.clone();
        let word = self.next_word().await?;

        let token_payload = match word {
            [b'p', b'u', b's', b'h', ..] => self
                .build_memory_token(MemoryTokenKind::Push)
                .await
                .expect("fail to build push token"),
            [b'p', b'o', b'p', ..] => self
                .build_memory_token(MemoryTokenKind::Pop)
                .await
                .expect("fail to build pop token"),
            [b'a', b'd', b'd', ..] => self.build_arithmetic_token(ArithmeticToken::Add),
            [b's', b'u', b'b', ..] => self.build_arithmetic_token(ArithmeticToken::Sub),
            [b'n', b'e', b'g', ..] => self.build_arithmetic_token(ArithmeticToken::Neg),
            [b'e', b'q', ..] => self.build_arithmetic_token(ArithmeticToken::Eq),
            [b'g', b't', ..] => self.build_arithmetic_token(ArithmeticToken::Gt),
            [b'l', b't', ..] => self.build_arithmetic_token(ArithmeticToken::Lt),
            [b'a', b'n', b'd', ..] => self.build_arithmetic_token(ArithmeticToken::And),
            [b'o', b'r', ..] => self.build_arithmetic_token(ArithmeticToken::Or),
            [b'n', b'o', b't', ..] => self.build_arithmetic_token(ArithmeticToken::Not),
            _ => {
                let i = std::str::from_utf8(word).unwrap();
                panic!("{}: Unexected instruction {}", src_line, i);
            }
        };
        Some(self.enrich_token_payload(token_payload))
    }

    fn build_arithmetic_token(&mut self, kind: ArithmeticToken) -> TokenPayload {
        TokenPayload::Arithmetic(kind)
    }

    fn enrich_token_payload(&self, payload: TokenPayload) -> Token {
        Token {
            payload,
            instruction: self.instruction_number,
            src: self.src_line,
        }
    }
}
