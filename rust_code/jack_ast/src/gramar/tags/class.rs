use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum ClassTagStage {
    #[default]
    AwaitClass,
    AwaitIdent,
    AwaitBody,
    Terminate,
}

#[derive(Default)]
pub struct ClassTag {
    stage: ClassTagStage,
}

impl JackAstTag for ClassTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Class
    }

    fn is_consistent(&self) -> bool {
        self.stage == ClassTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, ClassTagStage::Terminate) => JackAstTagResult::Finish,
            (_, ClassTagStage::AwaitBody) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::CurlyClass));
                self.stage = ClassTagStage::Terminate;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Ident(_), ClassTagStage::AwaitIdent) => {
                self.stage = ClassTagStage::AwaitBody;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Class), ClassTagStage::AwaitClass) => {
                self.stage = ClassTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
