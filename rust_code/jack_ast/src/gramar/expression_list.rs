use std::mem;

use crate::tokens::{JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression::{Expression, ExpressionData},
};

#[derive(Default)]
pub struct ExpressionListData {
    expression: Option<JackAstElem<Expression, ExpressionData>>,
}

pub struct ExpressionList;

impl JackAstElem<ExpressionList, ExpressionListData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        if self.data.expression.is_none() {
            let mut expression: JackAstElem<Expression, ExpressionData> = JackAstElem::default();
            expression.feed(token);
            self.data.expression = Some(expression);
            self.is_ready = false;
            return;
        }

        let is_expression_ready = self.data.expression.as_ref().unwrap().is_ready;

        match (&token, is_expression_ready) {
            (JackToken::Symbol(JackSymbol::Comma), true) => {
                let mut expression: Option<JackAstElem<Expression, ExpressionData>> =
                    Some(JackAstElem::default());
                mem::swap(&mut expression, &mut self.data.expression);
                expression.as_mut().unwrap().terminate();
                unsafe { self.push_ast(expression.unwrap()) };
                self.push_token(token);
            }
            _ => {
                self.data.expression.as_mut().unwrap().feed(token);
            }
        }

        self.is_ready = self.data.expression.as_ref().unwrap().is_ready;
    }

    pub fn terminate(&mut self) {
        if self.is_error || self.data.expression.is_none() {
            return;
        }

        let mut e: Option<JackAstElem<Expression, ExpressionData>> = None;
        mem::swap(&mut e, &mut self.data.expression);
        let mut expression = e.unwrap();

        if expression.is_error || !expression.is_ready {
            return;
        }

        expression.terminate();
        unsafe { self.push_ast(expression) };
        self.is_ready = true;
    }
}

impl IntoJackAstKind for ExpressionList {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ExpressionList
    }
}
