use std::mem;
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

use file_context::FileContext;
use futures::{FutureExt, Stream, StreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::tokens::JackToken;
use crate::xml::IntoXML;

use super::tags::ClassTag;

#[derive(Debug, Clone, Copy)]
pub enum JackAstTagLabel {
    Class,
    Function,
    Var,
    StaticOrField,
    Let,
    While,
    If,
    Return,
    Do,
    Term,
    Expression,
    Statements,
    Params,
    Expressions,
    RoundExpressionBracket,
    RoundExpressionsBracket,
    RoundParamsBracket,
    SquareBracket,
    CurlyBracket,
    CurlyFunctionBracket,
    CurlyClassBracket,
    FunctionsDeclar,
    StatementsDeclar,
}

impl JackAstTagLabel {
    pub fn get_tag_name(&self) -> Option<&'static [u8]> {
        match self {
            Self::Var => Some(b"varDec"),
            Self::Term => Some(b"term"),
            Self::Expression => Some(b"expression"),
            Self::Expressions => Some(b"expressionList"),
            Self::Return => Some(b"returnStatement"),
            Self::Let => Some(b"letStatement"),
            Self::Do => Some(b"doStatement"),
            Self::Statements => Some(b"statements"),
            Self::If => Some(b"ifStatement"),
            Self::While => Some(b"whileStatement"),
            Self::CurlyFunctionBracket => Some(b"subroutineBody"),
            Self::Params => Some(b"parameterList"),
            Self::Function => Some(b"subroutineDec"),
            Self::StaticOrField => Some(b"classVarDec"),
            Self::Class => Some(b"class"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JackAstTagStruct {
    pub label: JackAstTagLabel,
    pub is_open: bool,
    pub idx: usize,
}

impl JackAstTagStruct {
    pub fn new(label: JackAstTagLabel, idx: usize, is_open: bool) -> Self {
        Self {
            label,
            is_open,
            idx,
        }
    }
}

impl IntoXML for JackAstTagStruct {
    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize> {
        if let Some(body) = self.label.get_tag_name() {
            let mut n = 0;
            n += write.write(b"<").await?;

            if !self.is_open {
                n += write.write(b"/").await?;
            }

            n += write.write(&body).await?;
            n += write.write(b">").await?;

            Ok(n)
        } else {
            Ok(0)
        }
    }
}

#[derive(Debug)]
pub enum JackAstElement {
    Tag(JackAstTagStruct),
    Token(FileContext<JackToken>),
    Error(Box<JackAstElement>),
}

impl IntoXML for JackAstElement {
    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize> {
        match self {
            Self::Tag(t) => t.write_xml(write).await,
            Self::Token(t) => t.payload.write_xml(write).await,
            Self::Error(_) => unimplemented!(),
        }
    }
}

impl JackAstElement {
    pub fn raise_if_error(&self, src_file: &str) {
        match self {
            Self::Error(err) => match err.as_ref() {
                Self::Tag(_) => {
                    unimplemented!()
                }
                Self::Token(t) => {
                    let span = t.span.as_ref().expect("Compilation error");
                    let line = span.line + 1;
                    let symbol = span.symbol + 1;
                    panic!("Compilation error {src_file}:{line}:{symbol}")
                }
                Self::Error(_) => unreachable!(),
            },
            _ => (),
        }
    }

    pub fn is_displyable(&self) -> bool {
        match self {
            JackAstElement::Tag(t) => t.label.get_tag_name().is_some(),
            _ => true,
        }
    }

    pub fn is_open_tag(&self) -> bool {
        match self {
            JackAstElement::Tag(t) => t.is_open && self.is_displyable(),
            _ => false,
        }
    }

    pub fn is_close_tag(&self) -> bool {
        match self {
            JackAstElement::Tag(t) => !t.is_open && self.is_displyable(),
            _ => false,
        }
    }
}

pub enum JackAstTagResult {
    Move(Box<dyn JackAstTag + 'static>),
    Push,
    Finish,
    Error,
}

pub trait JackAstTag {
    fn feed_token(&mut self, token: &JackToken) -> JackAstTagResult;
    fn get_label(&self) -> JackAstTagLabel;
    fn is_consistent(&self) -> bool;

    fn intercept(&mut self, _token: &JackToken) -> bool {
        false
    }
}

pub struct JackAstEngine<'a, S: Stream<Item = FileContext<JackToken>> + Unpin> {
    stream: &'a mut S,
    tag_idx: usize,
    intercept_idx: Option<usize>,
    next_token: Option<FileContext<JackToken>>,
    scopes: Vec<(Box<dyn JackAstTag + 'a>, JackAstTagStruct)>,
}

impl<'a, S> JackAstEngine<'a, S>
where
    S: Stream<Item = FileContext<JackToken>> + Unpin,
{
    pub fn new(stream: &'a mut S) -> Self {
        let mut scopes: Vec<(Box<dyn JackAstTag + 'a>, JackAstTagStruct)> = Vec::new();
        scopes.push((
            Box::new(ClassTag::default()),
            JackAstTagStruct::new(ClassTag::default().get_label(), 0, true),
        ));
        Self {
            stream,
            scopes,
            tag_idx: 0,
            intercept_idx: None,
            next_token: None,
        }
    }

    pub fn scopes_to_vec(&self) -> Vec<JackAstTagLabel> {
        let mut v = Vec::new();
        for i in self.scopes.iter() {
            v.push(i.0.get_label());
        }
        v
    }

    fn pop_last_scope(&mut self) -> Option<JackAstElement> {
        self.scopes.pop().map(|(scope, mut scope_tag)| {
            scope_tag.is_open = false;
            if !scope.is_consistent() {
                JackAstElement::Error(Box::new(JackAstElement::Tag(scope_tag)))
            } else {
                JackAstElement::Tag(scope_tag)
            }
        })
    }

    fn update_intercept_idx(&mut self, token: &JackToken) {
        for i in 0..(self.scopes.len() - 1) {
            self.intercept_one(i, token)
        }
    }

    fn intercept_one(&mut self, i: usize, token: &JackToken) {
        let (s, _) = &mut self.scopes[i];
        let intercepted = s.as_mut().intercept(token);
        if intercepted {
            self.intercept_idx = Some(i);
        }
    }

    async fn next_token<'b>(&mut self) -> Option<JackAstElement> {
        if self.tag_idx == 0 {
            self.tag_idx = 1;
            return Some(JackAstElement::Tag(self.scopes[0].1));
        }

        if let Some(idx) = self.intercept_idx {
            if idx + 1 < self.scopes.len() {
                return self.pop_last_scope();
            } else {
                self.intercept_idx = None;
            }
        }

        let mut maybe_token: Option<FileContext<JackToken>> = None;
        if self.next_token.is_none() {
            maybe_token = self.stream.next().await;
            if maybe_token.is_none() {
                return self.pop_last_scope();
            }
            let token = maybe_token.as_ref().unwrap();

            self.update_intercept_idx(&token.payload);
        } else {
            mem::swap(&mut maybe_token, &mut self.next_token);
        }

        if let Some(token) = maybe_token {
            if self.intercept_idx.is_some() {
                self.next_token = Some(token);
                return self.pop_last_scope();
            }

            loop {
                let (mut scope, mut scope_tag) = self.scopes.pop().expect("Undefined scope");
                let result = scope.feed_token(&token.payload);
                match result {
                    JackAstTagResult::Push => {
                        self.scopes.push((scope, scope_tag));
                        return Some(JackAstElement::Token(token));
                    }
                    JackAstTagResult::Finish => {
                        self.next_token = Some(token);
                        scope_tag.is_open = false;
                        return Some(JackAstElement::Tag(scope_tag));
                    }
                    JackAstTagResult::Move(new_scope) => {
                        self.scopes.push((scope, scope_tag));
                        self.intercept_one(self.scopes.len() - 1, &token.payload);
                        self.next_token = Some(token);
                        let label = new_scope.get_label();
                        self.tag_idx += 1;
                        let tag = JackAstTagStruct::new(label, self.tag_idx - 1, true);
                        self.scopes.push((new_scope, tag));
                        return Some(JackAstElement::Tag(tag));
                    }
                    JackAstTagResult::Error => {
                        return Some(JackAstElement::Error(Box::new(JackAstElement::Token(
                            token,
                        ))));
                    }
                };
            }
        } else {
            unreachable!()
        }
    }
}

impl<S> Stream for JackAstEngine<'_, S>
where
    S: Stream<Item = FileContext<JackToken>> + Unpin,
{
    type Item = JackAstElement;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut next_future = pin!(self.get_mut().next_token());
        next_future.poll_unpin(cx)
    }
}

mod tests {
    #![allow(unused_imports, dead_code)]
    use std::{ffi::OsStr, path::PathBuf};

    use tokio::{
        fs::File,
        io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    };

    use crate::{
        tokens::{JackIdent, JackTokenizer},
        xml::{IntoXML, XML_PADDING},
    };

    use super::*;

    #[tokio::test]
    async fn test_array_file() {
        cmp_files(
            "./priv/ArrayTest.jack",
            "priv/test/gen/ArrayTest.xml",
            "priv/test/cmp/ArrayTest.xml",
        )
        .await;
    }

    #[tokio::test]
    async fn test_expression_less_square_project() {
        cmp_dirs(
            "./priv/test/cmp/ExpressionLessSquare/",
            "./priv/test/cmp/ExpressionLessSquare/",
            "./priv/test/gen/ExpressionLessSquare/",
        )
        .await;
    }

    #[tokio::test]
    async fn test_square_project() {
        cmp_dirs(
            "./priv/test/cmp/Square/",
            "./priv/test/cmp/Square/",
            "./priv/test/gen/Square/",
        )
        .await;
    }

    async fn cmp_dirs(src_dir: &str, cmp_dir: &str, out_dir: &str) {
        let mut files_in_dir = tokio::fs::read_dir(src_dir).await.unwrap();
        let mut files = Vec::new();

        while let Some(child) = files_in_dir.next_entry().await.unwrap() {
            let metadata = child.metadata().await.unwrap();
            if metadata.is_file() {
                let p = child.path();
                if let Some("jack") = p.extension().and_then(OsStr::to_str) {
                    files.push(p)
                };
            }
        }

        for f in files {
            let n = f.file_stem().and_then(OsStr::to_str).unwrap().to_string();
            let out_file_path = format!("{out_dir}{n}.xml");
            let cmp_file_path = format!("{cmp_dir}{n}.xml");
            cmp_files(f.to_str().unwrap(), &out_file_path, &cmp_file_path).await
        }
    }

    async fn cmp_files(src_code: &str, gen_file_path: &str, cmp_file_path: &str) {
        let file = File::open(src_code).await.unwrap();

        let mut tokenizer = JackTokenizer::from_file(file, true);
        let mut engine = JackAstEngine::new(&mut tokenizer);
        let mut xml_file = File::create(gen_file_path).await.unwrap();

        let mut padding = 0;

        while let Some(v) = engine.next().await {
            println!("{:?}", v);
            v.raise_if_error(src_code);

            if v.is_close_tag() {
                padding -= XML_PADDING;
            }

            if v.is_displyable() {
                xml_file.write(&vec![b' '; padding]).await.unwrap();
            }

            if v.is_open_tag() {
                padding += XML_PADDING;
            }

            let t = v.write_xml(&mut xml_file).await.unwrap();

            if t != 0 {
                xml_file.write(b"\n").await.unwrap();
            }
        }

        let generated_file = File::open(gen_file_path).await.unwrap();
        let cmp_file = File::open(cmp_file_path).await.unwrap();

        let cmp_file_reader = BufReader::new(cmp_file);
        let generated_file_reader = BufReader::new(generated_file);

        let mut cmp_file_reader_lines = cmp_file_reader.lines();
        let mut generated_file_reader_lines = generated_file_reader.lines();

        let mut idx = 0;

        while let Some(line) = cmp_file_reader_lines
            .next_line()
            .await
            .expect("Failed to read file")
        {
            let generated = generated_file_reader_lines
                .next_line()
                .await
                .expect("Failed to read file")
                .expect("Fail read line");
            idx += 1;
            assert_eq!(
                line, generated,
                "Not equal jack_ast/{gen_file_path}:{idx}:0 jack_ast/{cmp_file_path}:{idx}:0"
            );
        }
    }
}
