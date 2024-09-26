use std::mem;

use crate::gramar::ast::JackStatement;
use crate::tokens::{JackKeyword, JackToken};
use file_context::FileContext;

use super::assign::JackLetBuilder;
use super::call::JackDoBuilder;
use super::condition::JackIfBuilder;
use super::cycle::JackWhileBuilder;
use super::ret::JackReturnBuilder;
use super::statement::JackAstBuilderStatements;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(Default)]
pub struct JackStatementsBuilder {
    prev: Option<Box<dyn JackAstBuilderStatements>>,
    acc: Vec<Box<JackStatement>>,
}

impl JackStatementsBuilder {
    fn save_statement(&mut self) {
        let mut maybe_statement_builder = None;
        mem::swap(&mut maybe_statement_builder, &mut self.prev);
        if let Some(mut statement_builder) = maybe_statement_builder {
            let statement = statement_builder.as_mut().new_statement();
            self.acc.push(statement)
        }
    }

    pub fn build(self) -> Vec<Box<JackStatement>> {
        self.acc
    }
}

impl JackAstBuilder for JackStatementsBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match token.payload {
            JackToken::Keyword(JackKeyword::Let) => {
                self.save_statement();
                let mut prev = Box::new(JackLetBuilder::default());
                let link = prev.as_mut() as *mut dyn JackAstBuilder;
                self.prev = Some(prev);
                Ok(JackAstBuilderResponse::Move(link))
            }
            JackToken::Keyword(JackKeyword::Return) => {
                self.save_statement();
                let mut prev = Box::new(JackReturnBuilder::default());
                let link = prev.as_mut() as *mut dyn JackAstBuilder;
                self.prev = Some(prev);
                Ok(JackAstBuilderResponse::Move(link))
            }
            JackToken::Keyword(JackKeyword::Do) => {
                self.save_statement();
                let mut prev = Box::new(JackDoBuilder::default());
                let link = prev.as_mut() as *mut dyn JackAstBuilder;
                self.prev = Some(prev);
                Ok(JackAstBuilderResponse::Move(link))
            }
            JackToken::Keyword(JackKeyword::If) => {
                self.save_statement();
                let mut prev = Box::new(JackIfBuilder::default());
                let link = prev.as_mut() as *mut dyn JackAstBuilder;
                self.prev = Some(prev);
                Ok(JackAstBuilderResponse::Move(link))
            }
            JackToken::Keyword(JackKeyword::While) => {
                self.save_statement();
                let mut prev = Box::new(JackWhileBuilder::default());
                let link = prev.as_mut() as *mut dyn JackAstBuilder;
                self.prev = Some(prev);
                Ok(JackAstBuilderResponse::Move(link))
            }
            _ => {
                self.save_statement();
                Ok(JackAstBuilderResponse::MoveParent)
            }
        }
    }

    fn is_ready(&self) -> bool {
        true
    }
}
