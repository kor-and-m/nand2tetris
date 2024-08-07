use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum DoTagStage {
    #[default]
    AwaitDo,
    AwaitIdent,
    AwaitExpr,
    AwaitSemicolon,
    Terminate,
}

#[derive(Default)]
pub struct DoTag {
    stage: DoTagStage,
}

impl JackAstTag for DoTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Do
    }

    fn is_consistent(&self) -> bool {
        self.stage == DoTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, DoTagStage::Terminate) => JackAstTagResult::Finish,
            (JackToken::Symbol(JackSymbol::Semicolon), DoTagStage::AwaitSemicolon) => {
                self.stage = DoTagStage::Terminate;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::Period), DoTagStage::AwaitExpr) => {
                self.stage = DoTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (_, DoTagStage::AwaitExpr) => {
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundExpressions));
                self.stage = DoTagStage::AwaitSemicolon;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Ident(_), DoTagStage::AwaitIdent) => {
                self.stage = DoTagStage::AwaitExpr;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Do), DoTagStage::AwaitDo) => {
                self.stage = DoTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
