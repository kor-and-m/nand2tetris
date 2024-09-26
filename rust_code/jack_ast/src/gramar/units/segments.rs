use crate::tokens::{JackKeyword, JackToken};

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum JackSegment {
    #[default]
    Arg,
    Lcl,
    Field,
    Static,
}

impl JackSegment {
    pub fn from_token(token: &mut JackToken, global: bool) -> Option<Self> {
        match (token, global) {
            (JackToken::Keyword(JackKeyword::Var), false) => Some(Self::Lcl),
            (JackToken::Keyword(JackKeyword::Field), true) => Some(Self::Field),
            (JackToken::Keyword(JackKeyword::Static), true) => Some(Self::Static),
            _ => None,
        }
    }
}
