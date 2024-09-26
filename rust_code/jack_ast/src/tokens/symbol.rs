#[derive(Debug, PartialEq, Clone, Copy)]
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
            JackSymbol::Minus => true,
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
