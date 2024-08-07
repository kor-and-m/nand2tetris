use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum FunctionTagStage {
    #[default]
    AwaitFunctionKeyword,
    AwaitType,
    AwaitIdent,
    AwaitParams,
    AwaitBody,
    Terminate,
}

#[derive(Default)]
pub struct FunctionTag {
    stage: FunctionTagStage,
}

impl JackAstTag for FunctionTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Function
    }

    fn is_consistent(&self) -> bool {
        self.stage == FunctionTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, FunctionTagStage::Terminate) => JackAstTagResult::Finish,
            (_, FunctionTagStage::AwaitBody) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::CurlyFunction));
                self.stage = FunctionTagStage::Terminate;
                JackAstTagResult::Move(expression)
            }
            (_, FunctionTagStage::AwaitParams) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundParams));
                self.stage = FunctionTagStage::AwaitBody;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Ident(_), FunctionTagStage::AwaitIdent) => {
                self.stage = FunctionTagStage::AwaitParams;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), FunctionTagStage::AwaitType) => {
                self.stage = FunctionTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Void), FunctionTagStage::AwaitType) => {
                self.stage = FunctionTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(keyword), FunctionTagStage::AwaitType) if keyword.is_type() => {
                self.stage = FunctionTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Function), FunctionTagStage::AwaitFunctionKeyword) => {
                self.stage = FunctionTagStage::AwaitType;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Method), FunctionTagStage::AwaitFunctionKeyword) => {
                self.stage = FunctionTagStage::AwaitType;
                JackAstTagResult::Push
            }
            (
                JackToken::Keyword(JackKeyword::Constructor),
                FunctionTagStage::AwaitFunctionKeyword,
            ) => {
                self.stage = FunctionTagStage::AwaitType;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
