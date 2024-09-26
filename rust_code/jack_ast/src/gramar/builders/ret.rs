use std::mem;

use file_context::FileContext;

use crate::gramar::ast::{JackExpression, JackReturn, JackStatement};
use crate::tokens::JackSymbol;
use crate::tokens::{JackKeyword, JackToken};

use super::statement::JackAstBuilderStatements;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackReturnStage {
    #[default]
    AwaitReturn,
    AwaitSemicolon,
    Ready,
}

#[derive(PartialEq, Debug)]
pub struct JackReturnBuilder {
    stage: JackReturnStage,
    assign: Option<Box<JackStatement>>,
    acc: Vec<JackToken>,
}

impl JackReturnBuilder {
    fn unwrap_ret(&mut self) -> &mut JackReturn {
        let link = self.assign.as_mut().unwrap().as_mut();
        match link {
            JackStatement::Return(l) => l,
            _ => unreachable!(),
        }
    }
}

impl Default for JackReturnBuilder {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            assign: Some(Box::new(JackStatement::Return(Default::default()))),
            acc: Default::default(),
        }
    }
}

impl JackAstBuilder for JackReturnBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackReturnStage::AwaitReturn, JackToken::Keyword(JackKeyword::Return)) => {
                self.stage = JackReturnStage::AwaitSemicolon;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackReturnStage::AwaitSemicolon, JackToken::Symbol(JackSymbol::Semicolon)) => {
                let s = self.acc.len();
                if s != 0 {
                    self.unwrap_ret().expression = Some(JackExpression::new(&mut self.acc, s));
                    unsafe { self.acc.set_len(0) };
                }
                self.stage = JackReturnStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackReturnStage::AwaitSemicolon, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            _ => panic!("let error {:?}", token),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackReturnStage::Ready
    }
}

impl JackAstBuilderStatements for JackReturnBuilder {
    fn new_statement(&mut self) -> Box<JackStatement> {
        let mut res = None;
        mem::swap(&mut res, &mut self.assign);
        res.unwrap()
    }
}
