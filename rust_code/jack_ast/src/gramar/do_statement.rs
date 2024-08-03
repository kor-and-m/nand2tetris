use std::mem;

use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    function_call::{FunctionCall, FunctionCallData},
};

#[derive(Default)]
pub struct DoDeclarationtData {
    function_call: Option<JackAstElem<FunctionCall, FunctionCallData>>,
}
pub struct DoDeclaration;

impl JackAstElem<DoDeclaration, DoDeclarationtData> {
    pub fn feed(&mut self, token: JackToken) {
        match (self.children_count(), &token) {
            (0, JackToken::Keyword(JackKeyword::Do)) => self.push_token(token),
            (x, JackToken::Symbol(JackSymbol::Semicolon)) if x != 0 => {
                if self.data.function_call.is_some() {
                    let mut expression = None;
                    self.data.function_call.as_mut().unwrap();
                    mem::swap(&mut expression, &mut self.data.function_call);
                    self.is_ready = expression.as_ref().unwrap().is_ready;
                    unsafe { self.push_ast(expression.unwrap()) };
                } else {
                    self.is_ready = true;
                }
                self.push_token(token);
            }
            (x, _) if x != 0 => {
                if self.data.function_call.is_none() {
                    let expression = JackAstElem::default();
                    self.data.function_call = Some(expression);
                }
                self.data.function_call.as_mut().unwrap().feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for DoDeclaration {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Do
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackString},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_let_statement() {
        let list_tokens = vec![
            vec![
                JackToken::Keyword(JackKeyword::Do),
                JackToken::Ident(JackIdent(b"Output".to_vec())),
                JackToken::Symbol(JackSymbol::Period),
                JackToken::Ident(JackIdent(b"printString".to_vec())),
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
                JackToken::StringLiteral(JackString(b"THE AVERAGE IS: ".to_vec())),
                JackToken::Symbol(JackSymbol::CloseRoundBracket),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
            vec![
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
            ],
            vec![
                JackToken::Keyword(JackKeyword::Do),
                JackToken::Ident(JackIdent(b"Output".to_vec())),
                JackToken::Symbol(JackSymbol::Period),
                JackToken::Ident(JackIdent(b"println".to_vec())),
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
                JackToken::Symbol(JackSymbol::CloseRoundBracket),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
        ];

        {
            let mut file = File::create("priv/test/gen/do.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<DoDeclaration, DoDeclarationtData> =
                    JackAstElem::default();
                for token in tokens {
                    new_elem.feed(token);
                    assert!(!new_elem.is_error);
                }

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/do.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/do.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
