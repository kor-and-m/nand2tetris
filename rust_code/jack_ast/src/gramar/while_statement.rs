use std::mem;

use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression_in_breackets::{ExpressionInBreackets, ExpressionInBreacketsData},
    statements::{Statements, StatementsData},
};

#[derive(Default, Clone, Copy)]
enum WhileStatementStage {
    #[default]
    AwaitKeyword,
    AwaitCondition,
    AwaitBody,
}

#[derive(Default)]
pub struct WhileStatementData {
    stage: WhileStatementStage,
    expression_in_breackets: Option<JackAstElem<ExpressionInBreackets, ExpressionInBreacketsData>>,
    statements: Option<JackAstElem<Statements, StatementsData>>,
}
pub struct WhileStatement;

impl JackAstElem<WhileStatement, WhileStatementData> {
    pub fn feed(&mut self, token: JackToken) {
        match (self.data.stage, &token) {
            (WhileStatementStage::AwaitKeyword, JackToken::Keyword(JackKeyword::While)) => {
                self.push_token(token);
                self.data.stage = WhileStatementStage::AwaitCondition;
            }
            (
                WhileStatementStage::AwaitCondition,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                let mut expression = None;
                self.data.expression_in_breackets.as_mut().unwrap();
                mem::swap(&mut expression, &mut self.data.expression_in_breackets);
                unsafe { self.push_ast(expression.unwrap()) };
                self.push_token(token);
                self.data.stage = WhileStatementStage::AwaitBody;
            }
            (WhileStatementStage::AwaitCondition, _) => {
                if self.data.expression_in_breackets.is_none() {
                    let expression = JackAstElem::default();
                    self.data.expression_in_breackets = Some(expression);
                }
                self.data
                    .expression_in_breackets
                    .as_mut()
                    .unwrap()
                    .feed(token);
            }
            (WhileStatementStage::AwaitBody, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                let mut statements = None;
                self.data.statements.as_mut().unwrap().terminate();
                mem::swap(&mut statements, &mut self.data.statements);
                self.is_ready = statements.as_ref().unwrap().is_ready;
                unsafe { self.push_ast(statements.unwrap()) };
                self.push_token(token);
            }
            (WhileStatementStage::AwaitBody, _) => {
                if self.data.statements.is_none() {
                    let statements = JackAstElem::default();
                    self.data.statements = Some(statements);
                }
                self.data.statements.as_mut().unwrap().feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for WhileStatement {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::While
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackInt},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_while_statement() {
        let list_tokens = vec![vec![
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
        ]];

        {
            let mut file = File::create("priv/test/gen/while_statement.xml")
                .await
                .unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<WhileStatement, WhileStatementData> =
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

        let mut generated_file = File::open("priv/test/gen/while_statement.xml")
            .await
            .unwrap();
        let mut cmp_file = File::open("priv/test/cmp/while_statement.xml")
            .await
            .unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
