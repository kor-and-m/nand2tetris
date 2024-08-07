use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::do_statement::DoTag;
use super::if_statement::IfTag;
use super::let_statement::LetTag;
use super::return_statement::ReturnTag;
use super::while_statement::WhileTag;

#[derive(Default)]
pub struct StatementsTag {}

impl JackAstTag for StatementsTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Statements
    }

    fn is_consistent(&self) -> bool {
        true
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match token {
            JackToken::Keyword(keyword) => {
                let value_to_move: Option<Box<dyn JackAstTag>> = match keyword {
                    JackKeyword::Let => Some(Box::new(LetTag::default())),
                    JackKeyword::Do => Some(Box::new(DoTag::default())),
                    JackKeyword::Return => Some(Box::new(ReturnTag::default())),
                    JackKeyword::If => Some(Box::new(IfTag::default())),
                    JackKeyword::While => Some(Box::new(WhileTag::default())),
                    _ => None,
                };

                if let Some(v) = value_to_move {
                    JackAstTagResult::Move(v)
                } else {
                    JackAstTagResult::Error
                }
            }
            _ => JackAstTagResult::Error,
        }
    }
}
