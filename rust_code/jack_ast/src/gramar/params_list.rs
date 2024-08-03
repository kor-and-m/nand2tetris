use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
};

pub struct ParameterList;

impl JackAstElem<ParameterList> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.children_count(), &token) {
            (n, JackToken::Keyword(keyword))
                if n % 3 == 0
                    && (keyword == &JackKeyword::Char || keyword == &JackKeyword::Int) =>
            {
                self.push_token(token)
            }
            (n, JackToken::Ident(_)) if n % 3 == 0 => self.push_token(token),
            (n, JackToken::Ident(_)) if n % 3 == 1 => {
                self.push_token(token);
                self.is_ready = true;
            }
            (n, JackToken::Symbol(JackSymbol::Comma)) if n % 3 == 2 => {
                self.push_token(token);
                self.is_ready = false;
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for ParameterList {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ParameterList
    }
}
