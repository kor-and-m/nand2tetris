use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum WhileTagStage {
    #[default]
    AwaitWhile,
    AwaitCondition,
    AwaitBody,
    Terminate,
}

#[derive(Default)]
pub struct WhileTag {
    stage: WhileTagStage,
}

impl JackAstTag for WhileTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::While
    }

    fn is_consistent(&self) -> bool {
        self.stage == WhileTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, WhileTagStage::Terminate) => JackAstTagResult::Finish,
            (_, WhileTagStage::AwaitBody) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::Curly));
                self.stage = WhileTagStage::Terminate;
                JackAstTagResult::Move(expression)
            }
            (_, WhileTagStage::AwaitCondition) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundExpression));
                self.stage = WhileTagStage::AwaitBody;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Keyword(JackKeyword::While), WhileTagStage::AwaitWhile) => {
                self.stage = WhileTagStage::AwaitCondition;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
