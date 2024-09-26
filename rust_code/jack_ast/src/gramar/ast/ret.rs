use super::expression::JackExpression;

#[derive(Debug, PartialEq, Default)]
pub struct JackReturn {
    pub expression: Option<JackExpression>,
}
