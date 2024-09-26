use super::{expression::JackExpression, JackTerm};

#[derive(Debug, PartialEq, Default)]
pub struct JackLet {
    pub variable: JackTerm,
    pub expression: JackExpression,
}
