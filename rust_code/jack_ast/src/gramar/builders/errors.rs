use file_context::FileContext;

use crate::tokens::JackToken;

#[derive(Debug)]
pub enum JackAnalizerError {
    UnknownType(FileContext<JackToken>),
    UnknownSegment(FileContext<JackToken>),
    UnknownVar(FileContext<JackToken>),
    UnknownSubroutineKeyword(FileContext<JackToken>),
}
