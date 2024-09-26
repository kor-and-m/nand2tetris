use super::{statements::JackStatements, JackTerm};

#[derive(Debug, PartialEq, Default)]
pub struct JackIf {
    pub condition: JackTerm,
    pub statements: Box<JackStatements>,
    pub else_statements: Option<Box<JackStatements>>,
}
