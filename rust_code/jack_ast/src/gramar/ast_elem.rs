use futures::future::BoxFuture;
use futures::FutureExt;
use std::{marker::PhantomData, mem};

use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::{tokens::JackToken, xml::IntoXML};

use super::ast_kind::{IntoJackAstKind, JackAstElemKind};

pub struct UnknownAstElem;

pub enum TokenOrAst<T> {
    Token(JackToken),
    AST(JackAstElem<T>),
}

impl<T> TokenOrAst<T> {
    pub fn new_token(token: JackToken) -> Self {
        Self::Token(token)
    }

    pub fn new_ast(token: JackAstElem<T>) -> Self {
        Self::AST(token)
    }
}

pub struct JackAstElem<UnwrapedKind, BoxedData = ()> {
    kind_label: JackAstElemKind,
    _kind: PhantomData<UnwrapedKind>,
    pub data: Box<BoxedData>,
    pub is_ready: bool,
    pub is_error: bool,
    children: Vec<TokenOrAst<UnknownAstElem>>,
}

impl<UnwrapedKind: IntoJackAstKind, BoxedData: Default> Default
    for JackAstElem<UnwrapedKind, BoxedData>
{
    fn default() -> Self {
        let unwraped_kind = PhantomData;
        let kind_label = UnwrapedKind::kind();
        JackAstElem {
            kind_label,
            _kind: unwraped_kind,
            data: Box::new(BoxedData::default()),
            is_error: false,
            is_ready: false,
            children: Vec::new(),
        }
    }
}

impl<UnwrapedKind, BoxedData> JackAstElem<UnwrapedKind, BoxedData> {
    pub fn children_count(&self) -> usize {
        self.children.len()
    }

    pub fn push_token(&mut self, token: JackToken) {
        self.children.push(TokenOrAst::new_token(token));
    }

    pub fn from_option(opts: &mut Option<Self>) -> Option<Self> {
        let mut elem = None;
        mem::swap(&mut elem, opts);
        elem
    }

    pub unsafe fn push_ast<ElemType, ElemDataType>(
        &mut self,
        elem: JackAstElem<ElemType, ElemDataType>,
    ) {
        let unknown_elem = unsafe { elem.cast_to_unknown() };
        self.children.push(TokenOrAst::new_ast(unknown_elem));
    }

    pub unsafe fn cast_to_unknown(self) -> JackAstElem<UnknownAstElem> {
        mem::transmute::<_, JackAstElem<UnknownAstElem>>(self)
    }

    fn write_xml_recursive<'a, T: AsyncWrite + Unpin + Send>(
        &'a self,
        write: &'a mut T,
        padding: usize,
        deep: usize,
    ) -> BoxFuture<'a, Result<usize>> {
        let tag = self.kind_label.get_tag_name();
        write_xml_recursive(&self.children, write, padding, deep, tag).boxed()
    }
}

async fn write_xml_recursive<'a, T: AsyncWrite + Unpin + Send>(
    tokens: &Vec<TokenOrAst<UnknownAstElem>>,
    write: &'a mut T,
    padding: usize,
    deep: usize,
    maybe_tag_name: Option<&'static [u8]>,
) -> Result<usize> {
    let padding_abs = (deep + 1) * padding;
    let v = vec![b' '; padding_abs];
    let mut n = 0;
    if let Some(tag_name) = maybe_tag_name {
        n += write.write(&v[..(padding_abs - padding)]).await?;
        n += write.write(b"<").await?;
        n += write.write(tag_name).await?;
        n += write.write(b">\n").await?;
    }

    for child in tokens {
        n += match child {
            TokenOrAst::AST(ast) => {
                ast.write_xml_recursive(write, padding, deep + maybe_tag_name.is_some() as usize)
                    .await?
            }
            TokenOrAst::Token(token) => {
                let mut t = 0;

                t += write
                    .write(&v[..(padding_abs - padding * maybe_tag_name.is_none() as usize)])
                    .await?;
                t += token.write_xml(write).await?;
                t += write.write(b"\n").await?;
                t
            }
        };
    }

    if let Some(tag_name) = maybe_tag_name {
        n += write.write(&v[..(padding_abs - padding)]).await?;
        n += write.write(b"</").await?;
        n += write.write(tag_name).await?;
        n += write.write(b">\n").await?;
    }
    Ok(n)
}

impl<Kind, Data> IntoXML for JackAstElem<Kind, Data> {
    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize> {
        self.write_xml_recursive(write, Self::XML_PADDING, 0).await
    }
}
