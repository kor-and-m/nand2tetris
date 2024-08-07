use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::declar::DeclarTag;
use super::function::FunctionTag;

#[derive(Default)]
pub struct FunctionsDeclarTag {
    declared: bool,
}

impl JackAstTag for FunctionsDeclarTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::FunctionsDeclar
    }

    fn is_consistent(&self) -> bool {
        true
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, self.declared) {
            (JackToken::Keyword(JackKeyword::Static), false) => {
                let expression = Box::new(DeclarTag::new_class());
                JackAstTagResult::Move(expression)
            }
            (JackToken::Keyword(JackKeyword::Field), false) => {
                let expression = Box::new(DeclarTag::new_class());
                JackAstTagResult::Move(expression)
            }
            _ => {
                self.declared = true;
                let expressions = Box::new(FunctionTag::default());
                JackAstTagResult::Move(expressions)
            }
        }
    }
}
