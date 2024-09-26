use std::mem;

use file_context::FileContext;

use crate::gramar::ast::JackDeclaration;
use crate::tokens::JackToken;
use crate::{gramar::units::*, tokens::JackSymbol};

use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackParamsStage {
    #[default]
    AwaitOpenBracket,
    AwaitTypeInit,
    AwaitType,
    AwaitName,
    AwaitComma,
    Ready,
}

#[derive(PartialEq, Default, Debug)]
pub struct JackParamsBuilder {
    global: bool,
    stage: JackParamsStage,
    declarations: Vec<JackDeclaration>,
    var: Option<JackDeclaration>,
}

impl JackParamsBuilder {
    pub fn build(self) -> Vec<JackDeclaration> {
        self.declarations
    }

    pub fn save_old_var(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.var);

        if let Some(var) = old_var {
            self.declarations.push(var);
        }
    }
}

impl JackAstBuilder for JackParamsBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (
                JackParamsStage::AwaitOpenBracket,
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
            ) => {
                self.stage = JackParamsStage::AwaitTypeInit;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackParamsStage::AwaitTypeInit, JackToken::Symbol(JackSymbol::CloseRoundBracket)) => {
                self.stage = JackParamsStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackParamsStage::AwaitComma, JackToken::Symbol(JackSymbol::CloseRoundBracket)) => {
                self.stage = JackParamsStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackParamsStage::AwaitTypeInit, _) => {
                self.stage = JackParamsStage::AwaitType;
                self.feed(token)
            }
            (JackParamsStage::AwaitType, token_payload) => {
                if let Some(kind) = JackType::from_token(token_payload) {
                    let mut var = JackDeclaration::default();

                    var.kind = kind;
                    self.stage = JackParamsStage::AwaitName;
                    self.var = Some(var);
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownType(FileContext::from_old(token)))
                }
            }
            (JackParamsStage::AwaitName, token_payload) => {
                let style = JackVariableNameStyle::CamelCase;
                if let Some(var_name) = JackVariableName::from_token(token_payload, style) {
                    self.var.as_mut().unwrap().names.push(var_name);
                    self.stage = JackParamsStage::AwaitComma;
                    self.save_old_var();
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownVar(FileContext::from_old(token)))
                }
            }
            (JackParamsStage::AwaitComma, JackToken::Symbol(JackSymbol::Comma)) => {
                self.stage = JackParamsStage::AwaitType;
                Ok(JackAstBuilderResponse::Continue)
            }
            _ => unimplemented!(),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackParamsStage::Ready
    }
}
