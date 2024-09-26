use super::{statements::JackStatements, JackTerm};

#[derive(Debug, PartialEq, Default)]
pub struct JackWhile {
    pub condition: JackTerm,
    pub statements: Box<JackStatements>,
}
