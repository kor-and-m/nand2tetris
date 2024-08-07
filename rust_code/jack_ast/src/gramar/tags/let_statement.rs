use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::bracket::{BracketTag, BracketTagKind};
use super::expression::ExpressionTag;

#[derive(Default, PartialEq)]
pub enum LetTagStage {
    #[default]
    AwaitLet,
    AwaitIdent,
    AwaitEq,
    AwaitExpr,
    AwaitSemicolon,
    Terminate,
}

#[derive(Default)]
pub struct LetTag {
    stage: LetTagStage,
}

impl JackAstTag for LetTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Let
    }

    fn is_consistent(&self) -> bool {
        self.stage == LetTagStage::Terminate
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (_, LetTagStage::Terminate) => JackAstTagResult::Finish,
            (JackToken::Symbol(JackSymbol::Semicolon), LetTagStage::AwaitSemicolon) => {
                self.stage = LetTagStage::Terminate;
                JackAstTagResult::Push
            }
            (_, LetTagStage::AwaitExpr) => {
                let expression = Box::new(ExpressionTag::default());
                self.stage = LetTagStage::AwaitSemicolon;
                JackAstTagResult::Move(expression)
            }
            (JackToken::Symbol(JackSymbol::Eq), LetTagStage::AwaitEq) => {
                self.stage = LetTagStage::AwaitExpr;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::OpenSquareBracket), LetTagStage::AwaitEq) => {
                self.stage = LetTagStage::AwaitEq;
                let expression = Box::new(BracketTag::new(BracketTagKind::Square));
                JackAstTagResult::Move(expression)
            }
            (JackToken::Ident(_), LetTagStage::AwaitIdent) => {
                self.stage = LetTagStage::AwaitEq;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(JackKeyword::Let), LetTagStage::AwaitLet) => {
                self.stage = LetTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
