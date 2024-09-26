use std::mem;

use file_context::FileContext;

use crate::gramar::ast::{JackDo, JackStatement, JackTerm, JackTermPayload};
use crate::tokens::JackSymbol;
use crate::tokens::{JackKeyword, JackToken};

use super::statement::JackAstBuilderStatements;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackDoStage {
    #[default]
    AwaitDo,
    AwaitSemicolon,
    Ready,
}

#[derive(PartialEq, Debug)]
pub struct JackDoBuilder {
    stage: JackDoStage,
    assign: Option<Box<JackStatement>>,
    acc: Vec<JackToken>,
}

impl JackDoBuilder {
    fn unwrap_assign(&mut self) -> &mut JackDo {
        let link = self.assign.as_mut().unwrap().as_mut();
        match link {
            JackStatement::Do(l) => l,
            _ => unreachable!(),
        }
    }
}

impl Default for JackDoBuilder {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            assign: Some(Box::new(JackStatement::Do(Default::default()))),
            acc: Default::default(),
        }
    }
}

impl JackAstBuilder for JackDoBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackDoStage::AwaitDo, JackToken::Keyword(JackKeyword::Do)) => {
                self.stage = JackDoStage::AwaitSemicolon;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackDoStage::AwaitSemicolon, JackToken::Symbol(JackSymbol::Semicolon)) => {
                let s = self.acc.len();
                let term = JackTerm::new(&mut self.acc, s);

                match term.payload {
                    JackTermPayload::MethodCall(_, _) => (),
                    JackTermPayload::FunctionCall(_, _, _) => (),
                    _ => panic!("wrong call term"),
                }

                self.unwrap_assign().call = term;
                self.stage = JackDoStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackDoStage::AwaitSemicolon, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            _ => panic!("let error {:?}", token),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackDoStage::Ready
    }
}

impl JackAstBuilderStatements for JackDoBuilder {
    fn new_statement(&mut self) -> Box<JackStatement> {
        let mut res = None;
        mem::swap(&mut res, &mut self.assign);
        res.unwrap()
    }
}
