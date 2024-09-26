use std::mem;

use file_context::FileContext;

use crate::gramar::ast::{JackExpression, JackLet, JackStatement, JackTerm, JackTermPayload};
use crate::tokens::JackSymbol;
use crate::tokens::{JackKeyword, JackToken};

use super::statement::JackAstBuilderStatements;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackLetStage {
    #[default]
    AwaitLet,
    AwaitEq,
    AwaitSemicolon,
    Ready,
}

#[derive(PartialEq, Debug)]
pub struct JackLetBuilder {
    stage: JackLetStage,
    assign: Option<Box<JackStatement>>,
    acc: Vec<JackToken>,
}

impl JackLetBuilder {
    fn unwrap_assign(&mut self) -> &mut JackLet {
        let link = self.assign.as_mut().unwrap().as_mut();
        match link {
            JackStatement::Let(l) => l,
            _ => unreachable!(),
        }
    }
}

impl Default for JackLetBuilder {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            assign: Some(Box::new(JackStatement::Let(Default::default()))),
            acc: Default::default(),
        }
    }
}

impl JackAstBuilder for JackLetBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackLetStage::AwaitLet, JackToken::Keyword(JackKeyword::Let)) => {
                self.stage = JackLetStage::AwaitEq;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackLetStage::AwaitEq, JackToken::Symbol(JackSymbol::Eq)) => {
                let s = self.acc.len();
                let term = JackTerm::new(&mut self.acc, s);

                match term.payload {
                    JackTermPayload::Ident(_) => (),
                    JackTermPayload::ArrayElem(_, _) => (),
                    _ => panic!("wrong assign term"),
                }

                self.unwrap_assign().variable = term;
                unsafe { self.acc.set_len(0) };
                self.stage = JackLetStage::AwaitSemicolon;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackLetStage::AwaitEq, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackLetStage::AwaitSemicolon, JackToken::Symbol(JackSymbol::Semicolon)) => {
                let s = self.acc.len();
                self.unwrap_assign().expression = JackExpression::new(&mut self.acc, s);
                unsafe { self.acc.set_len(0) };
                self.stage = JackLetStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackLetStage::AwaitSemicolon, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            _ => panic!("let error {:?}", token),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackLetStage::Ready
    }
}

impl JackAstBuilderStatements for JackLetBuilder {
    fn new_statement(&mut self) -> Box<JackStatement> {
        let mut res = None;
        mem::swap(&mut res, &mut self.assign);
        res.unwrap()
    }
}
