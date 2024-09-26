use std::mem;

use crate::tokens::JackToken;

use super::style::JackVariableNameStyle;

#[derive(Debug, PartialEq, Default, Eq, Hash)]
pub struct JackVariableName(pub Vec<u8>);

impl JackVariableName {
    pub fn take(&mut self) -> Self {
        let mut tmp = Vec::new();
        mem::swap(&mut tmp, &mut self.0);
        Self(tmp)
    }

    pub fn from_token(token: &mut JackToken, style: JackVariableNameStyle) -> Option<Self> {
        match token {
            JackToken::Ident(ident) => {
                if style.check(&ident.0) {
                    let mut v = Vec::new();
                    mem::swap(&mut v, &mut ident.0);
                    Some(Self(v))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
