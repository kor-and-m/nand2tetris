use crate::gramar::units::{JackSubroutineType, JackType, JackVariableName};

use super::{declaration::JackDeclaration, statements::JackStatements};

#[derive(Debug, PartialEq, Default)]
pub struct JackSubroutine {
    pub name: JackVariableName,
    pub kind: JackType,
    pub key: JackSubroutineType,
    pub vars: Vec<JackDeclaration>,
    pub statements: JackStatements,
}
