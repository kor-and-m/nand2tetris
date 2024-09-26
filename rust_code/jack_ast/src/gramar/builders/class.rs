use std::mem;

use file_context::FileContext;

use crate::{
    gramar::{
        ast::JackClass,
        units::{JackVariableName, JackVariableNameStyle},
    },
    tokens::{JackKeyword, JackSymbol, JackToken},
};

use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    declaration::JackDeclarationBuilder,
    errors::JackAnalizerError,
    subroutine::JackSubroutineBuilder,
};

#[derive(PartialEq, Default, Debug)]
enum JackClassStage {
    #[default]
    AwaitClass,
    AwaitClassName,
    AwaitBracket,
    AwaitVars,
    AwaitSubroutins,
    Ready,
}

#[derive(Default)]
pub struct JackClassBuilder {
    stage: JackClassStage,
    class: JackClass,
    var: Option<JackDeclarationBuilder>,
    subroutine: Option<JackSubroutineBuilder>,
}

impl JackClassBuilder {
    pub fn build(self) -> JackClass {
        self.class
    }

    pub fn save_old_var(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.var);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.class.vars.push(new_var);
            } else {
                unreachable!()
            }
        }
    }

    pub fn save_old_subroutine(&mut self) {
        let mut old_var = None;
        mem::swap(&mut old_var, &mut self.subroutine);

        if let Some(var) = old_var {
            if var.is_ready() {
                let new_var = var.build();
                self.class.subroutines.push(new_var);
            } else {
                unreachable!()
            }
        }
    }
}

impl JackAstBuilder for JackClassBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError> {
        match (&self.stage, &mut token.payload) {
            (JackClassStage::AwaitClass, JackToken::Keyword(JackKeyword::Class)) => {
                self.stage = JackClassStage::AwaitClassName;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackClassStage::AwaitClassName, token_payload) => {
                let style = JackVariableNameStyle::PascalCase;
                if let Some(var_name) = JackVariableName::from_token(token_payload, style) {
                    self.class.name = var_name;
                    self.stage = JackClassStage::AwaitBracket;
                    Ok(JackAstBuilderResponse::Continue)
                } else {
                    Err(JackAnalizerError::UnknownVar(FileContext::from_old(token)))
                }
            }
            (JackClassStage::AwaitBracket, JackToken::Symbol(JackSymbol::OpenCurlyBracket)) => {
                self.stage = JackClassStage::AwaitVars;
                Ok(JackAstBuilderResponse::Continue)
            }
            (JackClassStage::AwaitVars, JackToken::Keyword(keyword)) if keyword.is_var_declar() => {
                self.save_old_var();
                let mut var = JackDeclarationBuilder::default();
                var.global = true;
                self.var = Some(var);
                Ok(JackAstBuilderResponse::Move(
                    self.var.as_mut().unwrap() as *mut dyn JackAstBuilder
                ))
            }
            (JackClassStage::AwaitVars, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                self.stage = JackClassStage::Ready;
                self.save_old_var();
                Ok(JackAstBuilderResponse::Ready)
            }
            (JackClassStage::AwaitVars, _token_payload) => {
                self.save_old_var();
                self.stage = JackClassStage::AwaitSubroutins;
                self.feed(token)
            }
            (JackClassStage::AwaitSubroutins, JackToken::Keyword(keyword))
                if keyword.is_function() =>
            {
                self.save_old_subroutine();
                let var = JackSubroutineBuilder::default();
                self.subroutine = Some(var);
                Ok(JackAstBuilderResponse::Move(
                    self.subroutine.as_mut().unwrap() as *mut dyn JackAstBuilder,
                ))
            }
            (JackClassStage::AwaitSubroutins, JackToken::Symbol(JackSymbol::CloseCurlyBracket)) => {
                self.stage = JackClassStage::Ready;
                self.save_old_subroutine();
                Ok(JackAstBuilderResponse::Ready)
            }
            _ => unimplemented!(),
        }
    }

    fn is_ready(&self) -> bool {
        self.stage == JackClassStage::Ready
    }
}
