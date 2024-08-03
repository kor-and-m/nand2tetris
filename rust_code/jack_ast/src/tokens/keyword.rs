use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::xml::IntoXML;

#[derive(Debug, PartialEq)]
pub enum JackKeyword {
    Constructor,
    Function,
    Boolean,
    Method,
    Static,
    Return,
    Class,
    Field,
    False,
    While,
    Char,
    Void,
    True,
    Null,
    This,
    Else,
    Var,
    Int,
    Let,
    Do,
    If,
}

impl JackKeyword {
    pub fn is_value(&self) -> bool {
        match self {
            JackKeyword::True => true,
            JackKeyword::False => true,
            JackKeyword::This => true,
            JackKeyword::Null => true,
            _ => false,
        }
    }

    pub fn bytes_to_keyword(buff: &[u8]) -> Option<Self> {
        match buff {
            [b'c', b'l', b'a', b's', b's', ..] => Some(Self::Class),
            [b'c', b'o', b'n', b's', b't', b'r', b'u', b'c', b't', b'o', b'r', ..] => {
                Some(Self::Constructor)
            }
            [b'm', b'e', b't', b'h', b'o', b'd', ..] => Some(Self::Method),
            [b'f', b'u', b'n', b'c', b't', b'i', b'o', b'n', ..] => Some(Self::Function),
            [b'f', b'i', b'e', b'l', b'd', ..] => Some(Self::Field),
            [b's', b't', b'a', b't', b'i', b'c', ..] => Some(Self::Static),
            [b'v', b'a', b'r', ..] => Some(Self::Var),
            [b'i', b'n', b't', ..] => Some(Self::Int),
            [b'c', b'h', b'a', b'r', ..] => Some(Self::Char),
            [b'b', b'o', b'o', b'l', b'e', b'a', b'n', ..] => Some(Self::Boolean),
            [b'v', b'o', b'i', b'd', ..] => Some(Self::Void),
            [b't', b'r', b'u', b'e', ..] => Some(Self::True),
            [b'f', b'a', b'l', b's', b'e', ..] => Some(Self::False),
            [b'n', b'u', b'l', b'l', ..] => Some(Self::Null),
            [b't', b'h', b'i', b's', ..] => Some(Self::This),
            [b'l', b'e', b't', ..] => Some(Self::Let),
            [b'd', b'o', ..] => Some(Self::Do),
            [b'i', b'f', ..] => Some(Self::If),
            [b'e', b'l', b's', b'e', ..] => Some(Self::Else),
            [b'w', b'h', b'i', b'l', b'e', ..] => Some(Self::While),
            [b'r', b'e', b't', b'u', b'r', b'n', ..] => Some(Self::Return),
            _ => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Do => 2,
            Self::If => 2,
            Self::Let => 3,
            Self::Var => 3,
            Self::Int => 3,
            Self::Char => 4,
            Self::Void => 4,
            Self::True => 4,
            Self::Null => 4,
            Self::This => 4,
            Self::Else => 4,
            Self::Class => 5,
            Self::Field => 5,
            Self::False => 5,
            Self::While => 5,
            Self::Method => 6,
            Self::Static => 6,
            Self::Return => 6,
            Self::Boolean => 7,
            Self::Function => 8,
            Self::Constructor => 11,
        }
    }
}

impl IntoXML for JackKeyword {
    async fn write_xml<T: AsyncWrite + Unpin>(&self, write: &mut T) -> Result<usize> {
        let mut n = write.write(b"<keyword> ").await?;

        n += match self {
            Self::Constructor => write.write(b"constructor").await?,
            Self::Function => write.write(b"function").await?,
            Self::Boolean => write.write(b"boolean").await?,
            Self::Method => write.write(b"method").await?,
            Self::Static => write.write(b"static").await?,
            Self::Return => write.write(b"return").await?,
            Self::Class => write.write(b"class").await?,
            Self::Field => write.write(b"field").await?,
            Self::False => write.write(b"false").await?,
            Self::While => write.write(b"while").await?,
            Self::Char => write.write(b"char").await?,
            Self::Void => write.write(b"void").await?,
            Self::True => write.write(b"true").await?,
            Self::Null => write.write(b"null").await?,
            Self::This => write.write(b"this").await?,
            Self::Else => write.write(b"else").await?,
            Self::Var => write.write(b"var").await?,
            Self::Int => write.write(b"int").await?,
            Self::Let => write.write(b"let").await?,
            Self::If => write.write(b"if").await?,
            Self::Do => write.write(b"do").await?,
        };

        n += write.write(b" </keyword>").await?;
        Ok(n)
    }
}
