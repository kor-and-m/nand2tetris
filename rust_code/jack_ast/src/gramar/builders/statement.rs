use crate::gramar::ast::JackStatement;

use super::behaviour::JackAstBuilder;

pub trait JackAstBuilderStatements: JackAstBuilder {
    fn new_statement(&mut self) -> Box<JackStatement>;
}
