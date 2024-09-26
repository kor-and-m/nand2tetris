use crate::gramar::units::{JackSegment, JackType, JackVariableName};

#[derive(Debug, PartialEq, Default)]
pub struct JackDeclaration {
    pub names: Vec<JackVariableName>,
    pub kind: JackType,
    pub segment: JackSegment,
}
