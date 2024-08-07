use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum IfTagStage {
    #[default]
    AwaitIf,
    AwaitCondition,
    AwaitBody,
    AwaitMaybeElse,
    AwaitElseBody,
    Terminate,
}

#[derive(Default)]
pub struct IfTag {
    stage: IfTagStage,
}

impl JackAstTag for IfTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::If
    }

    fn is_consistent(&self) -> bool {
        self.stage == IfTagStage::Terminate || self.stage == IfTagStage::AwaitMaybeElse
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, IfTagStage::Terminate) => JackAstTagResult::Finish,
            (JackToken::Keyword(JackKeyword::Else), IfTagStage::AwaitMaybeElse) => {
                self.stage = IfTagStage::AwaitElseBody;
                JackAstTagResult::Push
            }
            (_, IfTagStage::AwaitMaybeElse) => {
                self.stage = IfTagStage::Terminate;
                JackAstTagResult::Finish
            }
            (_, IfTagStage::AwaitElseBody) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::Curly));
                self.stage = IfTagStage::Terminate;
                JackAstTagResult::Move(expression)
            }
            (_, IfTagStage::AwaitBody) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::Curly));
                self.stage = IfTagStage::AwaitMaybeElse;
                JackAstTagResult::Move(expression)
            }
            (_, IfTagStage::AwaitCondition) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundExpression));
                self.stage = IfTagStage::AwaitBody;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Keyword(JackKeyword::If), IfTagStage::AwaitIf) => {
                self.stage = IfTagStage::AwaitCondition;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
