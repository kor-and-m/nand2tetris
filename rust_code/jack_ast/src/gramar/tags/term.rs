use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackSymbol, JackToken};

use super::bracket::{BracketTag, BracketTagKind};

#[derive(Default, PartialEq)]
pub enum TermTagKind {
    #[default]
    Unknown,
    UnaryOpTerm,
    IdentType,
    AwaitIdent,
    TerminateAfterMove,
    Terminate,
}

#[derive(Default)]
pub struct TermTag {
    kind: TermTagKind,
}

impl JackAstTag for TermTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Term
    }

    fn is_consistent(&self) -> bool {
        match self.kind {
            TermTagKind::Unknown => false,
            TermTagKind::AwaitIdent => false,
            TermTagKind::UnaryOpTerm => false,
            _ => true,
        }
    }

    fn feed_token(&mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.kind) {
            (JackToken::Symbol(s), TermTagKind::Unknown) if s.is_unary_op() => {
                self.kind = TermTagKind::UnaryOpTerm;
                JackAstTagResult::Push
            }
            (_, TermTagKind::UnaryOpTerm) => {
                self.kind = TermTagKind::TerminateAfterMove;
                let expression = Box::new(Self::default());
                JackAstTagResult::Move(expression)
            }
            (JackToken::Symbol(JackSymbol::OpenRoundBracket), TermTagKind::Unknown) => {
                self.kind = TermTagKind::TerminateAfterMove;
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundExpression));
                JackAstTagResult::Move(expression)
            }
            (JackToken::Symbol(JackSymbol::OpenRoundBracket), TermTagKind::IdentType) => {
                self.kind = TermTagKind::TerminateAfterMove;
                let expression = Box::new(BracketTag::new(BracketTagKind::RoundExpressions));
                JackAstTagResult::Move(expression)
            }
            (JackToken::StringLiteral(_), TermTagKind::Unknown) => {
                self.kind = TermTagKind::TerminateAfterMove;
                JackAstTagResult::Push
            }
            (JackToken::IntLiteral(_), TermTagKind::Unknown) => {
                self.kind = TermTagKind::TerminateAfterMove;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), TermTagKind::Unknown) => {
                self.kind = TermTagKind::IdentType;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(keyword), TermTagKind::Unknown) if keyword.is_value() => {
                self.kind = TermTagKind::IdentType;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::Period), TermTagKind::IdentType) => {
                self.kind = TermTagKind::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), TermTagKind::AwaitIdent) => {
                self.kind = TermTagKind::IdentType;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::OpenSquareBracket), TermTagKind::IdentType) => {
                self.kind = TermTagKind::TerminateAfterMove;
                let expression = Box::new(BracketTag::new(BracketTagKind::Square));
                JackAstTagResult::Move(expression)
            }
            (_, TermTagKind::TerminateAfterMove) => {
                self.kind = TermTagKind::Terminate;
                JackAstTagResult::Finish
            }
            (_, TermTagKind::IdentType) => JackAstTagResult::Finish,
            _ => JackAstTagResult::Error,
        }
    }
}
