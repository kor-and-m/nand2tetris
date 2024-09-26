use std::mem;

use file_context::FileContext;

use crate::{
    gramar::{
        ast::{JackStatements, JackSubroutine},
        units::{JackSubroutineType, JackType, JackVariableName, JackVariableNameStyle},
    },
    tokens::{JackSymbol, JackToken},
};

use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    declaration::JackDeclarationBuilder,
    errors::JackAnalizerError,
    params::JackParamsBuilder,
    statements::JackStatementsBuilder,
};

#[derive(PartialEq, Default, Debug)]
enum JackSubroutineStage {
    #[default]
    AwaitSubroutineKey,
    AwaitSubroutineType,
    AwaitSubroutineName,
    AwaitSubroutineParams,
    AwaitOpenBracket,
    AwaitSubroutineVars,
    AwaitStatements,
    Ready,
}

#[derive(Default)]
pub struct JackSubroutineBuilder {
    stage: JackSubroutineStage,
    subroutine: JackSubroutine,
    var: Option<JackDeclarationBuilder>,
    statements: Option<JackStatementsBuilder>,
    params: Option<JackParamsBuilder>,
}

impl JackSubroutineBuilder {
    pub fn build(self) -> JackSubroutine {
        self.subroutine
    }

    pub fn save_old_var(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.var);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.subroutine.vars.push(new_var);
            } else {
                unreachable!()
            }
        }
    }

    pub fn save_params(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.params);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.subroutine.vars = new_var;
            } else {
                unreachable!()
            }
        }
    }

    pub fn save_old_statements(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.statements);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.subroutine.statements = JackStatements(new_var);
            } else {
                unreachable!()
            }
        }
    }
}

impl JackAstBuilder for JackSubroutineBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackSubroutineStage::AwaitSubroutineKey, token_payload) => {
                if let Some(key) = JackSubroutineType::from_token(token_payload) {
                    self.stage = JackSubroutineStage::AwaitSubroutineType;
                    self.subroutine.key = key;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownSubroutineKeyword(
                        FileContext::from_old(token),
                    ))
                }
            }
            (JackSubroutineStage::AwaitSubroutineType, token_payload) => {
                if let Some(kind) = JackType::from_token(token_payload) {
                    self.subroutine.kind = kind;
                    self.stage = JackSubroutineStage::AwaitSubroutineName;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownType(FileContext::from_old(token)))
                }
            }
            (JackSubroutineStage::AwaitSubroutineName, token_payload) => {
                let style = JackVariableNameStyle::CamelCase;
                if let Some(var_name) = JackVariableName::from_token(token_payload, style) {
                    self.subroutine.name = var_name;
                    self.stage = JackSubroutineStage::AwaitSubroutineParams;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownVar(FileContext::from_old(token)))
                }
            }
            (JackSubroutineStage::AwaitSubroutineParams, _) => {
                self.params = Some(JackParamsBuilder::default());
                self.stage = JackSubroutineStage::AwaitOpenBracket;
                Ok(JackAstBuilderResponse::Move(
                    self.params.as_mut().unwrap() as *mut dyn JackAstBuilder
                ))
            }
            (
                JackSubroutineStage::AwaitOpenBracket,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.save_params();
                self.stage = JackSubroutineStage::AwaitSubroutineVars;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackSubroutineStage::AwaitSubroutineVars, JackToken::Keyword(keyword))
                if keyword.is_var_declar() =>
            {
                self.save_old_var();
                let mut var = JackDeclarationBuilder::default();
                var.global = false;
                self.var = Some(var);
                Ok(JackAstBuilderResponse::Move(
                    self.var.as_mut().unwrap() as *mut dyn JackAstBuilder
                ))
            }
            (JackSubroutineStage::AwaitSubroutineVars, _token_payload) => {
                self.save_old_var();
                self.stage = JackSubroutineStage::AwaitStatements;
                self.feed(token)
            }
            (
                JackSubroutineStage::AwaitStatements,
                JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            ) => {
                self.save_old_statements();
                self.stage = JackSubroutineStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackSubroutineStage::AwaitStatements, token_payload) => {
                if self.statements.is_some() {
                    panic!("Double statements initialization {:?}", token_payload)
                }

                self.statements = Some(Default::default());
                let link = self.statements.as_mut().unwrap();
                Ok(JackAstBuilderResponse::Move(link))
            }
            _ => unimplemented!(),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackSubroutineStage::Ready
    }
}
