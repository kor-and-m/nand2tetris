use phf::phf_map;
use std::fmt;

use hack_macro::SymbolicElem;
use symbolic::SymbolicElem;

pub static SEGMENTS: phf::Map<&'static [u8], Segment> = phf_map! {
    b"argument" => Segment::Arg,
    b"constant" => Segment::Const,
    b"pointer" => Segment::Pointer,
    b"static" => Segment::Static,
    b"local" => Segment::Local,
    b"temp" => Segment::Temp,
    b"that" => Segment::That,
    b"this" => Segment::This
};

#[derive(Hash, Debug, Clone, Copy, SymbolicElem)]
pub enum Segment {
    #[hack(symbol = b"argument")]
    Arg,
    #[hack(symbol = b"constant")]
    Const,
    #[hack(symbol = b"pointer")]
    Pointer,
    #[hack(symbol = b"static")]
    Static,
    #[hack(symbol = b"local")]
    Local,
    #[hack(symbol = b"temp")]
    Temp,
    #[hack(symbol = b"that")]
    That,
    #[hack(symbol = b"this")]
    This,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Segment::Arg => "argument",
            Segment::Const => "constant",
            Segment::Pointer => "pointer",
            Segment::Static => "static",
            Segment::Local => "local",
            Segment::Temp => "temp",
            Segment::That => "that",
            Segment::This => "this",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug, PartialEq)]
pub enum MemoryTokenKind {
    Pop,
    Push,
}

impl fmt::Display for MemoryTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            MemoryTokenKind::Pop => "pop",
            MemoryTokenKind::Push => "push",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug)]
pub struct MemoryToken {
    pub segment: Segment,
    pub kind: MemoryTokenKind,
    pub val: i16,
}

impl fmt::Display for MemoryToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.kind, self.segment, self.val)
    }
}

#[derive(Debug, SymbolicElem)]
pub enum ArithmeticToken {
    #[hack(symbol = b"add")]
    Add,
    #[hack(symbol = b"sub")]
    Sub,
    #[hack(symbol = b"neg")]
    Neg,
    #[hack(symbol = b"eq")]
    Eq,
    #[hack(symbol = b"gt")]
    Gt,
    #[hack(symbol = b"lt")]
    Lt,
    #[hack(symbol = b"and")]
    And,
    #[hack(symbol = b"or")]
    Or,
    #[hack(symbol = b"not")]
    Not,
}

impl fmt::Display for ArithmeticToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ArithmeticToken::Add => "add",
            ArithmeticToken::Sub => "sub",
            ArithmeticToken::Neg => "neg",
            ArithmeticToken::Eq => "eq",
            ArithmeticToken::Gt => "gt",
            ArithmeticToken::Lt => "lt",
            ArithmeticToken::And => "and",
            ArithmeticToken::Or => "or",
            ArithmeticToken::Not => "not",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug)]
pub enum TokenPayload {
    Memory(MemoryToken),
    Arithmetic(ArithmeticToken),
}

#[derive(Debug)]
pub struct Token {
    pub payload: TokenPayload,
    pub instruction: usize,
    pub src: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.payload {
            TokenPayload::Memory(memory_token) => memory_token.fmt(f),
            TokenPayload::Arithmetic(arithmetic_token) => arithmetic_token.fmt(f),
        }
    }
}
