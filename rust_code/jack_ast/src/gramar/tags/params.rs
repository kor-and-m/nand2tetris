use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackSymbol, JackToken};

#[derive(Default, PartialEq)]
pub enum ParamsTagStage {
    #[default]
    AwaitType,
    AwaitIdent,
    AwaitComma,
}

#[derive(Default)]
pub struct ParamsTag {
    stage: ParamsTagStage,
}

impl JackAstTag for ParamsTag {
    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::Params
    }

    fn is_consistent(&self) -> bool {
        self.stage != ParamsTagStage::AwaitIdent
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage) {
            (JackToken::Ident(_), ParamsTagStage::AwaitIdent) => {
                self.stage = ParamsTagStage::AwaitComma;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::Comma), ParamsTagStage::AwaitComma) => {
                self.stage = ParamsTagStage::AwaitType;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), ParamsTagStage::AwaitType) => {
                self.stage = ParamsTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(keyword), ParamsTagStage::AwaitType) if keyword.is_type() => {
                self.stage = ParamsTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (_, ParamsTagStage::AwaitType) => JackAstTagResult::Finish,
            _ => JackAstTagResult::Error,
        }
    }
}
