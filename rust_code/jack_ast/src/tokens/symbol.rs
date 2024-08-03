use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::xml::IntoXML;

#[derive(Debug, PartialEq)]
pub enum JackSymbol {
    OpenCurlyBracket,
    CloseCurlyBracket,
    OpenSquareBracket,
    CloseSquareBracket,
    OpenRoundBracket,
    CloseRoundBracket,
    Period,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Less,
    Greater,
    Eq,
    Not,
}

impl JackSymbol {
    pub fn is_op(&self) -> bool {
        match self {
            JackSymbol::And => true,
            JackSymbol::Or => true,
            JackSymbol::Eq => true,
            JackSymbol::Greater => true,
            JackSymbol::Less => true,
            JackSymbol::Plus => true,
            JackSymbol::Minus => true,
            JackSymbol::Multiply => true,
            JackSymbol::Divide => true,
            _ => false,
        }
    }

    pub fn is_unary_op(&self) -> bool {
        match self {
            JackSymbol::Not => true,
            _ => false,
        }
    }

    pub fn char_to_symbol(c: u8) -> Option<Self> {
        match c {
            b'(' => Some(Self::OpenRoundBracket),
            b')' => Some(Self::CloseRoundBracket),
            b'[' => Some(Self::OpenSquareBracket),
            b']' => Some(Self::CloseSquareBracket),
            b'{' => Some(Self::OpenCurlyBracket),
            b'}' => Some(Self::CloseCurlyBracket),
            b'.' => Some(Self::Period),
            b',' => Some(Self::Comma),
            b';' => Some(Self::Semicolon),
            b'+' => Some(Self::Plus),
            b'-' => Some(Self::Minus),
            b'*' => Some(Self::Multiply),
            b'/' => Some(Self::Divide),
            b'&' => Some(Self::And),
            b'|' => Some(Self::Or),
            b'<' => Some(Self::Less),
            b'>' => Some(Self::Greater),
            b'=' => Some(Self::Eq),
            b'~' => Some(Self::Not),
            _ => None,
        }
    }
}

impl IntoXML for JackSymbol {
    async fn write_xml<T: AsyncWrite + Unpin>(&self, write: &mut T) -> Result<usize> {
        let mut n = write.write(b"<symbol> ").await?;

        n += match self {
            Self::OpenRoundBracket => write.write(b"(").await?,
            Self::CloseRoundBracket => write.write(b")").await?,
            Self::OpenCurlyBracket => write.write(b"{").await?,
            Self::CloseCurlyBracket => write.write(b"}").await?,
            Self::OpenSquareBracket => write.write(b"[").await?,
            Self::CloseSquareBracket => write.write(b"]").await?,
            Self::Period => write.write(b".").await?,
            Self::Comma => write.write(b",").await?,
            Self::Semicolon => write.write(b";").await?,
            Self::Plus => write.write(b"+").await?,
            Self::Minus => write.write(b"-").await?,
            Self::Multiply => write.write(b"*").await?,
            Self::Divide => write.write(b"/").await?,
            Self::And => write.write(b"&").await?,
            Self::Or => write.write(b"|").await?,
            Self::Less => write.write(b"&lt;").await?,
            Self::Greater => write.write(b"&gt;").await?,
            Self::Eq => write.write(b"=").await?,
            Self::Not => write.write(b"!").await?,
        };

        n += write.write(b" </symbol>").await?;
        Ok(n)
    }
}
