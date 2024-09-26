use file_context::FileContext;

use super::errors::JackAnalizerError;
use crate::tokens::JackToken;

pub enum JackAstBuilderResponse {
    Ready,
    MoveParent,
    Continue,
    Move(*mut dyn JackAstBuilder),
}

pub trait JackAstBuilder {
    fn feed(
        &mut self,
        token: &mut FileContext<JackToken>,
    ) -> Result<JackAstBuilderResponse, JackAnalizerError>;
    fn is_ready(&self) -> bool;
}
