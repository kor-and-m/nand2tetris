use std::mem;

use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression::{Expression, ExpressionData},
};

#[derive(Default)]
pub struct ReturnStatementData {
    expression: Option<JackAstElem<Expression, ExpressionData>>,
}

pub struct ReturnStatement;

impl JackAstElem<ReturnStatement, ReturnStatementData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.children_count(), &token) {
            (0, JackToken::Keyword(JackKeyword::Return)) => {
                self.push_token(token);
            }
            (x, JackToken::Symbol(JackSymbol::Semicolon)) if x > 0 => {
                if self.data.expression.is_some() {
                    let mut expression = None;
                    self.data.expression.as_mut().unwrap().terminate();
                    mem::swap(&mut expression, &mut self.data.expression);
                    self.is_ready = expression.as_ref().unwrap().is_ready;
                    unsafe { self.push_ast(expression.unwrap()) };
                } else {
                    self.is_ready = true;
                }
                self.push_token(token);
            }
            (x, _) if x > 0 => {
                if self.data.expression.is_none() {
                    let expression = JackAstElem::default();
                    self.data.expression = Some(expression);
                }
                self.data.expression.as_mut().unwrap().feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for ReturnStatement {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ReturnStatement
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{tokens::JackSymbol, xml::IntoXML};

    use super::*;

    #[tokio::test]
    async fn test_var_statement() {
        let list_tokens = vec![
            vec![
                JackToken::Keyword(JackKeyword::Return),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
            vec![
                JackToken::Keyword(JackKeyword::Return),
                JackToken::Keyword(JackKeyword::This),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
        ];

        {
            let mut file = File::create("priv/test/gen/return.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<ReturnStatement, ReturnStatementData> =
                    JackAstElem::default();
                for token in tokens {
                    // println!("{:?}", &token);
                    new_elem.feed(token);
                    assert!(!new_elem.is_error);
                }

                assert!(new_elem.is_ready);

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/return.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/return.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
