use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression_in_breackets::{ExpressionInBreackets, ExpressionInBreacketsData},
    statements::{Statements, StatementsData},
};

#[derive(Default, Clone, Copy)]
enum IfStatementStage {
    #[default]
    AwaitKeyword,
    AwaitCondition,
    AwaitBody,
}

#[derive(Default)]
pub struct IfStatementData {
    stage: IfStatementStage,
    expression_in_breackets: Option<JackAstElem<ExpressionInBreackets, ExpressionInBreacketsData>>,
    statements: Option<JackAstElem<Statements, StatementsData>>,
}
pub struct IfStatement;

impl JackAstElem<IfStatement, IfStatementData> {
    pub fn feed(&mut self, token: JackToken) {
        match (self.data.stage, &token) {
            (IfStatementStage::AwaitKeyword, JackToken::Keyword(JackKeyword::If)) => {
                self.push_token(token);
                self.data.stage = IfStatementStage::AwaitCondition;
            }
            (IfStatementStage::AwaitCondition, JackToken::Symbol(JackSymbol::OpenCurlyBracket)) => {
                let expression = JackAstElem::from_option(&mut self.data.expression_in_breackets);
                unsafe { self.push_ast(expression.unwrap()) };
                self.push_token(token);
                self.data.stage = IfStatementStage::AwaitBody;
            }
            (IfStatementStage::AwaitCondition, _) => {
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
            (IfStatementStage::AwaitBody, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                let mut statements = JackAstElem::from_option(&mut self.data.statements);
                statements.as_mut().unwrap().terminate();
                self.is_ready = statements.as_ref().unwrap().is_ready;
                unsafe { self.push_ast(statements.unwrap()) };
                self.push_token(token);
            }
            (IfStatementStage::AwaitBody, _) => {
                if self.data.statements.is_none() {
                    let statements = JackAstElem::default();
                    self.data.statements = Some(statements);
                }
                self.data
                    .expression_in_breackets
                    .as_mut()
                    .unwrap()
                    .feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for IfStatement {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::If
    }
}
