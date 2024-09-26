use std::mem;

use file_context::FileContext;

use crate::gramar::ast::{JackIf, JackStatement, JackStatements, JackTerm, JackTermPayload};
use crate::tokens::JackSymbol;
use crate::tokens::{JackKeyword, JackToken};

use super::statement::JackAstBuilderStatements;
use super::statements::JackStatementsBuilder;
use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackIfStage {
    #[default]
    AwaitIf,
    AwaitOpenBracket,
    AwaitStatements,
    AwaitElse,
    AwaitElseOpenBracket,
    AwaitElseStatements,
    Ready,
}

pub struct JackIfBuilder {
    stage: JackIfStage,
    condition: Option<Box<JackStatement>>,
    statements: Option<JackStatementsBuilder>,
    acc: Vec<JackToken>,
}

impl JackIfBuilder {
    fn unwrap_condition(&mut self) -> &mut JackIf {
        let link = self.condition.as_mut().unwrap().as_mut();
        match link {
            JackStatement::If(l) => l,
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

    pub fn save_old_statements_else(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.statements);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.unwrap_condition().else_statements = Some(Box::new(JackStatements(new_var)));
            } else {
                unreachable!()
            }
        }
    }
}

impl Default for JackIfBuilder {
    fn default() -> Self {
        Self {
            stage: Default::default(),
            condition: Some(Box::new(JackStatement::If(Default::default()))),
            statements: Default::default(),
            acc: Default::default(),
        }
    }
}

impl JackAstBuilder for JackIfBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackIfStage::AwaitIf, JackToken::Keyword(JackKeyword::If)) => {
                self.stage = JackIfStage::AwaitOpenBracket;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackIfStage::AwaitElse, JackToken::Keyword(JackKeyword::Else)) => {
                self.stage = JackIfStage::AwaitElseOpenBracket;
                Ok(JackAstBuilderResponse::Continue)
            }
            (
                JackIfStage::AwaitElseOpenBracket,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.stage = JackIfStage::AwaitElseStatements;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackIfStage::AwaitOpenBracket, JackToken::Symbol(JackSymbol::OpenCurlyBracket)) => {
                let s = self.acc.len();
                let term = JackTerm::new(&mut self.acc, s);

                match term.payload {
                    JackTermPayload::Expression(_) => (),
                    _ => panic!("wrong condition term"),
                }

                self.unwrap_condition().condition = term;
                unsafe { self.acc.set_len(0) };
                self.stage = JackIfStage::AwaitStatements;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackIfStage::AwaitOpenBracket, token) => {
                let mut t = JackToken::default();
                mem::swap(&mut t, token);
                self.acc.push(t);
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackIfStage::AwaitStatements, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                self.save_old_statements();
                self.stage = JackIfStage::AwaitElse;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackIfStage::AwaitElse, _) => Ok(JackAstBuilderResponse::MoveParent),
            (JackIfStage::AwaitStatements, token_payload) => {
                if self.statements.is_some() {
                    panic!("Double statements initialization {:?}", token_payload)
                }

                self.statements = Some(Default::default());
                let link = self.statements.as_mut().unwrap();
                Ok(JackAstBuilderResponse::Move(link))
            }
            (
                JackIfStage::AwaitElseStatements,
                JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            ) => {
                self.save_old_statements_else();
                self.stage = JackIfStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackIfStage::AwaitElseStatements, token_payload) => {
                if self.statements.is_some() {
                    panic!("Double statements initialization {:?}", token_payload)
                }

                self.statements = Some(Default::default());
                let link = self.statements.as_mut().unwrap();
                Ok(JackAstBuilderResponse::Move(link))
            }
            _ => panic!("if error {:?}", token),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackIfStage::Ready
    }
}

impl JackAstBuilderStatements for JackIfBuilder {
    fn new_statement(&mut self) -> Box<JackStatement> {
        let mut res = None;
        mem::swap(&mut res, &mut self.condition);
        res.unwrap()
    }
}
