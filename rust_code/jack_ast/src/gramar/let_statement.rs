use std::mem;

use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    array_index::{ArrayIndex, ArrayIndexData},
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression::{Expression, ExpressionData},
};

#[derive(Default)]
enum LetStage {
    #[default]
    AwaitLet,
    AwaitIdent,
    AwaitIndex,
    AwaitEq,
    AwaitExpression,
}

#[derive(Default)]
pub struct LettData {
    stage: LetStage,
    expression: Option<JackAstElem<Expression, ExpressionData>>,
    array_index: Option<JackAstElem<ArrayIndex, ArrayIndexData>>,
}
pub struct Let;

impl JackAstElem<Let, LettData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (&self.data.stage, &token) {
            (LetStage::AwaitLet, JackToken::Keyword(JackKeyword::Let)) => {
                self.push_token(token);
                self.data.stage = LetStage::AwaitIdent;
            }
            (LetStage::AwaitIdent, JackToken::Ident(_)) => {
                self.push_token(token);
                self.data.stage = LetStage::AwaitEq;
            }
            (LetStage::AwaitEq, JackToken::Symbol(JackSymbol::OpenSquareBracket)) => {
                let mut array_index: JackAstElem<ArrayIndex, ArrayIndexData> =
                    JackAstElem::default();

                array_index.feed(token);
                self.data.array_index = Some(array_index);
                self.data.stage = LetStage::AwaitIndex;
            }
            (LetStage::AwaitIndex, _) => {
                self.data.array_index.as_mut().unwrap().feed(token);
                if self.data.array_index.as_ref().unwrap().is_ready {
                    let elem = JackAstElem::from_option(&mut self.data.array_index);
                    unsafe { self.push_ast(elem.unwrap()) };
                    self.data.stage = LetStage::AwaitEq;
                }
            }
            (LetStage::AwaitEq, JackToken::Symbol(JackSymbol::Eq)) => {
                self.push_token(token);
                self.data.stage = LetStage::AwaitExpression;
            }
            (LetStage::AwaitExpression, JackToken::Symbol(JackSymbol::Semicolon)) => {
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
            (LetStage::AwaitExpression, _) => {
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

impl IntoJackAstKind for Let {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Let
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackInt, JackString},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_let_statement() {
        let list_tokens = vec![
            vec![
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
            ],
            vec![
                JackToken::Keyword(JackKeyword::Let),
                JackToken::Ident(JackIdent(b"i".to_vec())),
                JackToken::Symbol(JackSymbol::Eq),
                JackToken::IntLiteral(JackInt(b"0".to_vec())),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
            vec![
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
            ],
        ];

        {
            let mut file = File::create("priv/test/gen/lets.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<Let, LettData> = JackAstElem::default();
                for token in tokens {
                    new_elem.feed(token);
                    assert!(!new_elem.is_error);
                }

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/lets.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/lets.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
