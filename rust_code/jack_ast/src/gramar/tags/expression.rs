use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::JackToken;

use super::term::TermTag;

#[derive(Default, PartialEq)]
pub enum ExpressionTagStage {
    #[default]
    AwaitTerm,
    AwaitOp,
}

#[derive(Default)]
pub struct ExpressionTag {
    stage: ExpressionTagStage,
}

impl JackAstTag for ExpressionTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Expression
    }

    fn is_consistent(&self) -> bool {
        self.stage == ExpressionTagStage::AwaitOp
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (JackToken::Symbol(symbol), ExpressionTagStage::AwaitOp) if symbol.is_op() => {
                self.stage = ExpressionTagStage::AwaitTerm;
                JackAstTagResult::Push
            }
            (_, ExpressionTagStage::AwaitTerm) => {
                self.stage = ExpressionTagStage::AwaitOp;
                let term = Box::new(TermTag::default());
                JackAstTagResult::Move(term)
            }
            (_, ExpressionTagStage::AwaitOp) => JackAstTagResult::Finish,
        }
    }
}
