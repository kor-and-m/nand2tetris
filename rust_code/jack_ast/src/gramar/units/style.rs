use std::env;
use std::str::from_utf8;

use inflections::case::{is_camel_case, is_constant_case, is_pascal_case};

#[derive(Default, Debug)]
pub enum JackVariableNameStyle {
    #[default]
    Utf8,
    ConstantCase,
    CamelCase,
    PascalCase,
}

impl JackVariableNameStyle {
    pub fn check(&self, data: &[u8]) -> bool {
        let is_strict = if let Ok(v) = env::var("STRICT_MODE") {
            v == "1"
        } else {
            false
        };

        if !is_strict {
            return true;
        }

        if let Ok(v) = from_utf8(data) {
            match self {
                Self::Utf8 => true,
                Self::CamelCase => is_camel_case(v),
                Self::ConstantCase => is_constant_case(v),
                Self::PascalCase => is_pascal_case(v),
            }
        } else {
            false
        }
    }
}
