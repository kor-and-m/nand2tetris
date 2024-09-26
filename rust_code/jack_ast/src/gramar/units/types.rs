use std::mem;

use crate::tokens::{JackKeyword, JackToken};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JackBasicType {
    Int,
    Char,
    Bool,
    String,
    Arr,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JackType {
    Basic(JackBasicType),
    Class(Vec<u8>),
}

impl Default for JackType {
    fn default() -> Self {
        JackType::Basic(JackBasicType::Arr)
    }
}

impl JackType {
    pub fn is_array(&self) -> bool {
        self == &Self::Basic(JackBasicType::Arr)
    }

    pub fn is_void(&self) -> bool {
        self == &Self::Basic(JackBasicType::Void)
    }

    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Class(class) => class,
            Self::Basic(JackBasicType::Arr) => b"Array",
            Self::Basic(JackBasicType::String) => b"String",
            _ => unimplemented!(),
        }
    }

    pub fn take(&mut self) -> Self {
        match self {
            Self::Basic(basic) => Self::Basic(*basic),
            Self::Class(class) => {
                let mut tmp = Vec::new();
                mem::swap(class, &mut tmp);
                Self::Class(tmp)
            }
        }
    }

    pub fn from_token(token: &mut JackToken) -> Option<Self> {
        match token {
            JackToken::Keyword(JackKeyword::Char) => Some(JackType::Basic(JackBasicType::Char)),
            JackToken::Keyword(JackKeyword::Int) => Some(JackType::Basic(JackBasicType::Int)),
            JackToken::Keyword(JackKeyword::Boolean) => Some(JackType::Basic(JackBasicType::Bool)),
            JackToken::Keyword(JackKeyword::Void) => Some(JackType::Basic(JackBasicType::Void)),
            JackToken::Ident(ident) if ident.0 == b"Array" => {
                Some(JackType::Basic(JackBasicType::Arr))
            }
            JackToken::Ident(ident) if ident.0 == b"String" => {
                Some(JackType::Basic(JackBasicType::String))
            }
            JackToken::Ident(ident) => {
                let mut v = Vec::new();
                mem::swap(&mut v, &mut ident.0);
                Some(JackType::Class(v))
            }
            _ => None,
        }
    }
}
