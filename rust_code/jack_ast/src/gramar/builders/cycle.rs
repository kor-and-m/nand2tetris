use std::mem;

use file_context::FileContext;

use crate::gramar::ast::{JackStatement, JackStatements, JackTerm, JackTermPayload, JackWhile};
use crate::tokens::JackSymbol;
use crate::tokens::{JackKeyword, JackToken};

use super::statement::JackAstBuilderStatements;
use super::statements::JackStatementsBuilder;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackWhileStage {
    #[default]
    AwaitWhile,
    AwaitOpenBracket,
    AwaitStatements,
    Ready,
}

pub struct JackWhileBuilder {
    stage: JackWhileStage,
    condition: Option<Box<JackStatement>>,
    statements: Option<JackStatementsBuilder>,
    acc: Vec<JackToken>,
}

impl JackWhileBuilder {
    fn unwrap_condition(&mut self) -> &mut JackWhile {
        let link = self.condition.as_mut().unwrap().as_mut();
        match link {
            JackStatement::While(l) => l,
            _ => unreachable!(),
        }
    }

    pub fn save_old_statements(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.statements);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.unwrap_condition().statements = Box::new(JackStatements(new_var));
            } else {
                unreachable!()
            }
        }
    }
}

impl Default for JackWhileBuilder {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            condition: Some(Box::new(JackStatement::While(Default::default()))),
            statements: Default::default(),
            acc: Default::default(),
        }
    }
}

impl JackAstBuilder for JackWhileBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackWhileStage::AwaitWhile, JackToken::Keyword(JackKeyword::While)) => {
                self.stage = JackWhileStage::AwaitOpenBracket;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackWhileStage::AwaitOpenBracket, JackToken::Symbol(JackSymbol::OpenCurlyBracket)) => {
                let s = self.acc.len();
                let term = JackTerm::new(&mut self.acc, s);

                match term.payload {
                    JackTermPayload::Expression(_) => (),
                    _ => panic!("wrong condition term"),
                }

                self.unwrap_condition().condition = term;
                unsafe { self.acc.set_len(0) };
                self.stage = JackWhileStage::AwaitStatements;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackWhileStage::AwaitOpenBracket, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackWhileStage::AwaitStatements, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                self.save_old_statements();
                self.stage = JackWhileStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackWhileStage::AwaitStatements, token_payload) => {
                if self.statements.is_some() {
                    panic!("Double statements initialization {:?}", token_payload)
                }

                self.statements = Some(Default::default());
                let link = self.statements.as_mut().unwrap();
                Ok(JackAstBuilderResponse::Move(link))
            }
            _ => panic!("while error {:?}", token),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackWhileStage::Ready
    }
}

impl JackAstBuilderStatements for JackWhileBuilder {
    fn new_statement(&mut self) -> Box<JackStatement> {
        let mut res = None;
        mem::swap(&mut res, &mut self.condition);
        res.unwrap()
    }
}
