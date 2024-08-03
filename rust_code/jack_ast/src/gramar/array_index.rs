use std::mem;

use crate::tokens::{JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression::{Expression, ExpressionData},
};

#[derive(Default)]
pub struct ArrayIndexData {
    breackets: usize,
    expression: Option<JackAstElem<Expression, ExpressionData>>,
}

pub struct ArrayIndex;

impl JackAstElem<ArrayIndex, ArrayIndexData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.children_count(), &token) {
            (0, JackToken::Symbol(JackSymbol::OpenSquareBracket)) => {
                self.push_token(token);
                self.data.breackets = 1;
                let expression = JackAstElem::default();
                self.data.expression = Some(expression);
            }
            (x, JackToken::Symbol(JackSymbol::OpenSquareBracket)) if x != 0 => {
                self.data.breackets += 1;
                self.data.expression.as_mut().unwrap().feed(token);
            }
            (x, JackToken::Symbol(JackSymbol::CloseSquareBracket)) if x != 0 => {
                if self.data.breackets > 1 {
                    self.data.breackets -= 1;
                    self.data.expression.as_mut().unwrap().feed(token);
                } else {
                    self.data.breackets = 0;
                    let mut expression = None;
                    self.data.expression.as_mut().unwrap().terminate();
                    mem::swap(&mut expression, &mut self.data.expression);
                    self.is_ready = expression.as_ref().unwrap().is_ready;
                    unsafe { self.push_ast(expression.unwrap()) };
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

impl IntoJackAstKind for ArrayIndex {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ArrayIndex
    }
}
