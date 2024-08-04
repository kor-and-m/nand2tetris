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
    AwaitOpenBracket,
    AwaitBody,
    AwaitElse,
    Terminate,
}

#[derive(Default)]
pub struct IfStatementData {
    is_else: bool,
    brackets: usize,
    stage: IfStatementStage,
    expression_in_breackets: Option<JackAstElem<ExpressionInBreackets, ExpressionInBreacketsData>>,
    statements: Option<JackAstElem<Statements, StatementsData>>,
}
pub struct IfStatement;

impl JackAstElem<IfStatement, IfStatementData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.data.stage, &token) {
            (IfStatementStage::AwaitKeyword, JackToken::Keyword(JackKeyword::If)) => {
                self.push_token(token);
                self.data.stage = IfStatementStage::AwaitCondition;
            }
            (
                IfStatementStage::AwaitOpenBracket,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.push_token(token);
                self.data.stage = IfStatementStage::AwaitBody;
                let mut statements = JackAstElem::default();
                statements.is_ready = true;
                self.data.statements = Some(statements);
                self.data.brackets = 1;
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

                if self.data.expression_in_breackets.as_ref().unwrap().is_ready {
                    self.data.is_else = false;
                    let expression =
                        JackAstElem::from_option(&mut self.data.expression_in_breackets);
                    unsafe { self.push_ast(expression.unwrap()) };
                    self.data.stage = IfStatementStage::AwaitOpenBracket;
                }
            }
            (IfStatementStage::AwaitBody, JackToken::Symbol(JackSymbol::CloseCurlyBracket))
                if self.data.brackets == 1 =>
            {
                let mut statements = JackAstElem::from_option(&mut self.data.statements);
                statements.as_mut().unwrap().terminate();
                self.is_ready = statements.as_ref().unwrap().is_ready;
                unsafe { self.push_ast(statements.unwrap()) };
                self.push_token(token);
                if self.data.is_else {
                    self.data.stage = IfStatementStage::Terminate;
                } else {
                    self.data.stage = IfStatementStage::AwaitElse;
                }
            }
            (IfStatementStage::AwaitBody, t) => {
                match t {
                    JackToken::Symbol(JackSymbol::OpenCurlyBracket) => self.data.brackets += 1,
                    JackToken::Symbol(JackSymbol::CloseCurlyBracket) => self.data.brackets -= 1,
                    _ => (),
                };
                self.data.statements.as_mut().unwrap().feed(token);
            }
            (IfStatementStage::AwaitElse, JackToken::Keyword(JackKeyword::Else)) => {
                self.is_ready = false;
                self.push_token(token);
                self.data.stage = IfStatementStage::AwaitOpenBracket;
                self.data.is_else = true;
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
