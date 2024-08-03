use std::mem;

use crate::tokens::{JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression::{Expression, ExpressionData},
};

#[derive(Default)]
pub struct ExpressionInBreacketsData {
    breackets: usize,
    expression: Option<JackAstElem<Expression, ExpressionData>>,
}

pub struct ExpressionInBreackets;

impl JackAstElem<ExpressionInBreackets, ExpressionInBreacketsData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.children_count(), &token) {
            (0, JackToken::Symbol(JackSymbol::OpenRoundBracket)) => {
                self.push_token(token);
                self.data.breackets = 1;
                let expressions = JackAstElem::default();
                self.data.expression = Some(expressions);
            }
            (x, JackToken::Symbol(JackSymbol::OpenRoundBracket)) if x != 0 => {
                self.data.breackets += 1;
                self.data.expression.as_mut().unwrap().feed(token);
            }
            (x, JackToken::Symbol(JackSymbol::CloseRoundBracket)) if x != 0 => {
                if self.data.breackets > 1 {
                    self.data.breackets -= 1;
                    self.data.expression.as_mut().unwrap().feed(token);
                } else {
                    self.data.breackets = 0;
                    let mut expressions = None;
                    self.data.expression.as_mut().unwrap().terminate();
                    mem::swap(&mut expressions, &mut self.data.expression);
                    self.is_ready = expressions.as_ref().unwrap().is_ready;
                    unsafe { self.push_ast(expressions.unwrap()) };
                    self.push_token(token);
                }
            }
            (x, _) if x != 0 => {
                self.data.expression.as_mut().unwrap().feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for ExpressionInBreackets {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ExpressionInBreackets
    }
}
