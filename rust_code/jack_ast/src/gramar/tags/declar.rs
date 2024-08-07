use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackSymbol, JackToken};

#[derive(PartialEq)]
pub enum DeclarTagStage {
    AwaitKeyword,
    AwaitType,
    AwaitIdent,
    AwaitSemicolon,
    Terminate,
}

pub enum DeclarTagScope {
    Class,
    Function,
}

pub struct DeclarTag {
    stage: DeclarTagStage,
    scope: DeclarTagScope,
}

impl DeclarTag {
    pub fn new_class() -> Self {
        Self {
            stage: DeclarTagStage::AwaitKeyword,
            scope: DeclarTagScope::Class,
        }
    }

    pub fn new_function() -> Self {
        Self {
            stage: DeclarTagStage::AwaitKeyword,
            scope: DeclarTagScope::Function,
        }
    }
}

impl JackAstTag for DeclarTag {
    fn is_consistent(&self) -> bool {
        self.stage == DeclarTagStage::Terminate
    }

    fn get_label(&self) -> JackAstTagLabel {
        match self.scope {
            DeclarTagScope::Class => JackAstTagLabel::StaticOrField,
            DeclarTagScope::Function => JackAstTagLabel::Var,
        }
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.stage, &self.scope) {
            (_, DeclarTagStage::Terminate, _) => JackAstTagResult::Finish,
            (JackToken::Symbol(JackSymbol::Semicolon), DeclarTagStage::AwaitSemicolon, _) => {
                self.stage = DeclarTagStage::Terminate;
                JackAstTagResult::Push
            }
            (JackToken::Symbol(JackSymbol::Comma), DeclarTagStage::AwaitSemicolon, _) => {
                self.stage = DeclarTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), DeclarTagStage::AwaitIdent, _) => {
                self.stage = DeclarTagStage::AwaitSemicolon;
                JackAstTagResult::Push
            }
            (JackToken::Ident(_), DeclarTagStage::AwaitType, _) => {
                self.stage = DeclarTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (JackToken::Keyword(keyword), DeclarTagStage::AwaitType, _) if keyword.is_type() => {
                self.stage = DeclarTagStage::AwaitIdent;
                JackAstTagResult::Push
            }
            (
                JackToken::Keyword(JackKeyword::Static),
                DeclarTagStage::AwaitKeyword,
                DeclarTagScope::Class,
            ) => {
                self.stage = DeclarTagStage::AwaitType;
                JackAstTagResult::Push
            }
            (
                JackToken::Keyword(JackKeyword::Field),
                DeclarTagStage::AwaitKeyword,
                DeclarTagScope::Class,
            ) => {
                self.stage = DeclarTagStage::AwaitType;
                JackAstTagResult::Push
            }
            (
                JackToken::Keyword(JackKeyword::Var),
                DeclarTagStage::AwaitKeyword,
                DeclarTagScope::Function,
            ) => {
                self.stage = DeclarTagStage::AwaitType;
                JackAstTagResult::Push
            }
            _ => JackAstTagResult::Error,
        }
    }
}
