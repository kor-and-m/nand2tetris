use super::{assign::JackLet, call::JackDo, condition::JackIf, cycle::JackWhile, ret::JackReturn};

#[derive(Debug, PartialEq)]
pub enum JackStatement {
    Do(JackDo),
    Let(JackLet),
    Return(JackReturn),
    If(JackIf),
    While(JackWhile),
}

#[derive(Debug, PartialEq, Default)]
pub struct JackStatements(pub Vec<Box<JackStatement>>);
