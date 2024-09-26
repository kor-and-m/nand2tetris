use crate::tokens::{JackKeyword, JackToken};

#[derive(Debug, PartialEq, Default)]
pub enum JackSubroutineType {
    #[default]
    Method,
    Function,
    Constructor,
}

impl JackSubroutineType {
    pub fn from_token(token: &mut JackToken) -> Option<Self> {
        match token {
            JackToken::Keyword(JackKeyword::Method) => Some(Self::Method),
            JackToken::Keyword(JackKeyword::Function) => Some(Self::Function),
            JackToken::Keyword(JackKeyword::Constructor) => Some(Self::Constructor),
            _ => None,
        }
    }
}
