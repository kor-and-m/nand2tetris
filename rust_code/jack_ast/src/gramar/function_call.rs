use std::mem;

use crate::tokens::{JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression_list::{ExpressionList, ExpressionListData},
};

#[derive(Default)]
enum FunctionCallStage {
    #[default]
    AwaitIdent,
    AwaitOpenBreacket,
    AwaitCloseBreacket,
}

#[derive(Default)]
pub struct FunctionCallData {
    satge: FunctionCallStage,
    breackets: usize,
    expressions: Option<JackAstElem<ExpressionList, ExpressionListData>>,
}

pub struct FunctionCall;

impl JackAstElem<FunctionCall, FunctionCallData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (&self.data.satge, &token) {
            (FunctionCallStage::AwaitIdent, JackToken::Ident(_)) => {
                self.push_token(token);
                self.data.satge = FunctionCallStage::AwaitOpenBreacket;
            }
            (FunctionCallStage::AwaitOpenBreacket, JackToken::Symbol(JackSymbol::Period)) => {
                self.push_token(token);
                self.data.satge = FunctionCallStage::AwaitIdent;
            }
            (
                FunctionCallStage::AwaitOpenBreacket,
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
            ) => {
                self.push_token(token);
                self.data.breackets = 1;
                self.data.expressions = Some(JackAstElem::default());
                self.data.expressions.as_mut().unwrap().is_ready = true;
                self.data.satge = FunctionCallStage::AwaitCloseBreacket;
            }
            (
                FunctionCallStage::AwaitCloseBreacket,
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
            ) => {
                self.data.breackets += 1;
                self.data.expressions.as_mut().unwrap().feed(token);
            }
            (
                FunctionCallStage::AwaitCloseBreacket,
                JackToken::Symbol(JackSymbol::CloseRoundBracket),
            ) => {
                if self.data.breackets > 1 {
                    self.data.breackets -= 1;
                    self.data.expressions.as_mut().unwrap().feed(token);
                } else {
                    self.data.breackets = 0;
                    let mut expressions = None;
                    self.data.expressions.as_mut().unwrap().terminate();
                    mem::swap(&mut expressions, &mut self.data.expressions);
                    self.is_ready = expressions.as_ref().unwrap().is_ready;
                    unsafe { self.push_ast(expressions.unwrap()) };
                    self.push_token(token);
                }
            }
            (FunctionCallStage::AwaitCloseBreacket, _) => {
                self.data.expressions.as_mut().unwrap().feed(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for FunctionCall {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::FunctionCall
    }
}
