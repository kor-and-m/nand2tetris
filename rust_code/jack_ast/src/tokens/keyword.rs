#[derive(Debug, PartialEq, Default)]
pub enum JackKeyword {
    #[default]
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
    pub fn is_var_declar(&self) -> bool {
        match self {
            Self::Var => true,
            Self::Static => true,
            Self::Field => true,
            _ => false,
        }
    }

    pub fn is_value(&self) -> bool {
        match self {
            Self::True => true,
            Self::False => true,
            Self::This => true,
            Self::Null => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            Self::Function => true,
            Self::Method => true,
            Self::Constructor => true,
            _ => false,
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            Self::Char => true,
            Self::Boolean => true,
            Self::Int => true,
            Self::Void => true,
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
