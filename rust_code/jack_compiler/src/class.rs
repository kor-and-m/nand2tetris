use jack_ast::gramar::{JackClass, JackVariableName};

use crate::vars::JackTableNames;

pub struct JackClassCompilerContext {
    class_name: JackVariableName,
    pub vars: JackTableNames,
}

impl JackClassCompilerContext {
    pub fn init(class: &mut JackClass) -> Self {
        let mut global = JackTableNames::default();

        for i in class.vars.iter_mut() {
            global.migrate(i)
        }

        Self {
            class_name: class.name.take(),
            vars: global,
        }
    }

    pub fn class(&self) -> &JackVariableName {
        &self.class_name
    }
}
