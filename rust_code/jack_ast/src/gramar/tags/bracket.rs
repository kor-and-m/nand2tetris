use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackSymbol, JackToken};

use super::expression::ExpressionTag;
use super::expressions::ExpressionsTag;
use super::funvtions_declar::FunctionsDeclarTag;
use super::params::ParamsTag;
use super::statements::StatementsTag;
use super::statements_declar::StatementsDeclarTag;

#[derive(PartialEq)]
pub enum BracketTagKind {
    RoundExpression,
    RoundExpressions,
    RoundParams,
    Square,
    Curly,
    CurlyFunction,
    CurlyClass,
}

impl BracketTagKind {
    pub fn is_round(&self) -> bool {
        match self {
            Self::RoundExpression => true,
            Self::RoundExpressions => true,
            Self::RoundParams => true,
            _ => false,
        }
    }

    pub fn is_curly(&self) -> bool {
        match self {
            Self::Curly => true,
            Self::CurlyClass => true,
            Self::CurlyFunction => true,
            _ => false,
        }
    }

    pub fn is_expr(&self) -> bool {
        match self {
            Self::RoundExpression => true,
            Self::Square => true,
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub enum BracketTagStage {
    AwaitOpen,
    AwaitBody,
    AwaitClose,
    Closed,
}

pub struct BracketTag {
    kind: BracketTagKind,
    stage: BracketTagStage,
    inc_symbol: JackSymbol,
    dec_symbol: JackSymbol,
    counter: usize,
}

impl BracketTag {
    pub fn new(kind: BracketTagKind) -> Self {
        let (inc_symbol, dec_symbol) = match (kind.is_round(), kind.is_curly()) {
            (true, true) => unreachable!(),
            (false, true) => (JackSymbol::OpenCurlyBracket, JackSymbol::CloseCurlyBracket),
            (true, false) => (JackSymbol::OpenRoundBracket, JackSymbol::CloseRoundBracket),
            (false, false) => (
                JackSymbol::OpenSquareBracket,
                JackSymbol::CloseSquareBracket,
            ),
        };
        BracketTag {
            kind,
            inc_symbol,
            dec_symbol,
            stage: BracketTagStage::AwaitOpen,
            counter: 0,
        }
    }
}

impl JackAstTag for BracketTag {
    fn get_label(&self) -> JackAstTagLabel {
        match self.kind {
            BracketTagKind::Curly => JackAstTagLabel::CurlyBracket,
            BracketTagKind::CurlyClass => JackAstTagLabel::CurlyClassBracket,
            BracketTagKind::CurlyFunction => JackAstTagLabel::CurlyFunctionBracket,
            BracketTagKind::RoundExpression => JackAstTagLabel::RoundExpressionBracket,
            BracketTagKind::RoundParams => JackAstTagLabel::RoundParamsBracket,
            BracketTagKind::RoundExpressions => JackAstTagLabel::RoundExpressionsBracket,
            BracketTagKind::Square => JackAstTagLabel::SquareBracket,
        }
    }

    fn is_consistent(&self) -> bool {
        self.stage == BracketTagStage::Closed
    }

    fn intercept(&mut self, token: &JackToken) -> bool {
        match token {
            JackToken::Symbol(s) if *s == self.inc_symbol => {
                self.counter += 1;
                false
            }
            JackToken::Symbol(s) if *s == self.dec_symbol => {
                self.counter -= 1;
                if self.counter == 0 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn feed_token(&mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.kind, &self.stage) {
            (JackToken::Symbol(s), _, BracketTagStage::AwaitOpen) if *s == self.inc_symbol => {
                self.counter = 1;
                self.stage = BracketTagStage::AwaitBody;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(s), _, BracketTagStage::AwaitClose) if *s == self.dec_symbol => {
                self.stage = BracketTagStage::Closed;
                JackAstTagResult::Push
            }
            (_, kind, BracketTagStage::AwaitBody) if kind.is_expr() => {
                let expression = Box::new(ExpressionTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expression)
            }
            (_, BracketTagKind::Curly, BracketTagStage::AwaitBody) => {
                let expressions = Box::new(StatementsTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expressions)
            }
            (_, BracketTagKind::CurlyFunction, BracketTagStage::AwaitBody) => {
                let expressions = Box::new(StatementsDeclarTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expressions)
            }
            (_, BracketTagKind::CurlyClass, BracketTagStage::AwaitBody) => {
                let expressions = Box::new(FunctionsDeclarTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expressions)
            }
            (_, BracketTagKind::RoundExpressions, BracketTagStage::AwaitBody) => {
                let expressions = Box::new(ExpressionsTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expressions)
            }
            (_, BracketTagKind::RoundParams, BracketTagStage::AwaitBody) => {
                let expressions = Box::new(ParamsTag::default());
                self.stage = BracketTagStage::AwaitClose;
                JackAstTagResult::Move(expressions)
            }
            (_, _, BracketTagStage::Closed) => JackAstTagResult::Finish,
            _ => JackAstTagResult::Error,
        }
    }
}
