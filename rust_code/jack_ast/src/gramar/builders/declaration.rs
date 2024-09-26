use file_context::FileContext;

use crate::gramar::ast::JackDeclaration;
use crate::tokens::JackToken;
use crate::{gramar::units::*, tokens::JackSymbol};

use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    errors::JackAnalizerError,
};

#[derive(PartialEq, Default, Debug)]
enum JackDeclarationStage {
    #[default]
    AwaitScope,
    AwaitType,
    AwaitName,
    AwaitSemicolon,
    Ready,
}

#[derive(PartialEq, Default, Debug)]
pub struct JackDeclarationBuilder {
    pub global: bool,
    stage: JackDeclarationStage,
    declaration: JackDeclaration,
}

impl JackDeclarationBuilder {
    pub fn build(self) -> JackDeclaration {
        self.declaration
    }
}

impl JackAstBuilder for JackDeclarationBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackDeclarationStage::AwaitScope, token_payload) => {
                if let Some(segment) = JackSegment::from_token(token_payload, self.global) {
                    self.declaration.segment = segment;
                    self.stage = JackDeclarationStage::AwaitType;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownSegment(FileContext::from_old(
                        token,
                    )))
                }
            }
            (JackDeclarationStage::AwaitType, token_payload) => {
                if let Some(kind) = JackType::from_token(token_payload) {
                    self.declaration.kind = kind;
                    self.stage = JackDeclarationStage::AwaitName;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownType(FileContext::from_old(token)))
                }
            }
            (JackDeclarationStage::AwaitName, token_payload) => {
                let style = if self.declaration.segment == JackSegment::Static {
                    JackVariableNameStyle::ConstantCase
                } else {
                    JackVariableNameStyle::CamelCase
                };
                if let Some(var_name) = JackVariableName::from_token(token_payload, style) {
                    self.declaration.names.push(var_name);
                    self.stage = JackDeclarationStage::AwaitSemicolon;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownVar(FileContext::from_old(token)))
                }
            }
            (JackDeclarationStage::AwaitSemicolon, JackToken::Symbol(JackSymbol::Semicolon)) => {
                self.stage = JackDeclarationStage::Ready;
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackDeclarationStage::AwaitSemicolon, JackToken::Symbol(JackSymbol::Comma)) => {
                self.stage = JackDeclarationStage::AwaitName;
                Ok(JackAstBuilderResponse::Continue)
            }
            _ => unimplemented!(),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackDeclarationStage::Ready
    }
}
