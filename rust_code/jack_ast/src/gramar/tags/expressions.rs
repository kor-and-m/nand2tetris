use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackSymbol, JackToken};

use super::expression::ExpressionTag;

#[derive(Default, PartialEq)]
pub enum ExpressionsTagStage {
    #[default]
    InitExpression,
    AwaitExpression,
    AwaitComma,
}

#[derive(Default)]
pub struct ExpressionsTag {
    stage: ExpressionsTagStage,
}

impl JackAstTag for ExpressionsTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Expressions
    }

    fn is_consistent(&self) -> bool {
        self.stage == ExpressionsTagStage::AwaitComma
            || self.stage == ExpressionsTagStage::InitExpression
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        if self.stage == ExpressionsTagStage::InitExpression {
            self.stage = ExpressionsTagStage::AwaitExpression;
        }

        match (token, &self.stage) {
            (JackToken::Symbol(JackSymbol::Comma), ExpressionsTagStage::AwaitComma) => {
                self.stage = ExpressionsTagStage::AwaitExpression;
                JackAstTagResult::Push
            }
            (_, ExpressionsTagStage::AwaitExpression) => {
                self.stage = ExpressionsTagStage::AwaitComma;
                let term = Box::new(ExpressionTag::default());
                JackAstTagResult::Move(term)
            }
            (_, ExpressionsTagStage::AwaitComma) => JackAstTagResult::Finish,
            (_, ExpressionsTagStage::InitExpression) => unreachable!(),
        }
    }
}
