use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::expression::ExpressionTag;

#[derive(Default, PartialEq)]
pub enum ReturnTagStage {
    #[default]
    AwaitReturn,
    AwaitExpr,
    AwaitSemicolon,
    Terminate,
}

#[derive(Default)]
pub struct ReturnTag {
    stage: ReturnTagStage,
}

impl JackAstTag for ReturnTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Return
    }

    fn is_consistent(&self) -> bool {
        self.stage == ReturnTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, ReturnTagStage::Terminate) => JackAstTagResult::Finish,
            (JackToken::Symbol(JackSymbol::Semicolon), ReturnTagStage::AwaitSemicolon) => {
                self.stage = ReturnTagStage::Terminate;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::Semicolon), ReturnTagStage::AwaitExpr) => {
                self.stage = ReturnTagStage::Terminate;
                JackAstTagResult::Push
            }
            (_, ReturnTagStage::AwaitExpr) => {
                let expression = Box::new(ExpressionTag::default());
                self.stage = ReturnTagStage::AwaitSemicolon;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Keyword(JackKeyword::Return), ReturnTagStage::AwaitReturn) => {
                self.stage = ReturnTagStage::AwaitExpr;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
