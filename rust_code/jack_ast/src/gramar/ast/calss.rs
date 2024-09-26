use crate::gramar::units::JackVariableName;

use super::{declaration::JackDeclaration, subroutine::JackSubroutine};

#[derive(Debug, PartialEq, Default)]
pub struct JackClass {
    pub name: JackVariableName,
    pub vars: Vec<JackDeclaration>,
    pub subroutines: Vec<JackSubroutine>,
}
