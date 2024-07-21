use phf::phf_map;
use std::fmt;

use hack_macro::SymbolicElem;
use symbolic::SymbolicElem;

pub static SEGMENTS: phf::Map<&'static [u8], MemoryTokenSegment> = phf_map! {
    b"argument" => MemoryTokenSegment::Arg,
    b"constant" => MemoryTokenSegment::Const,
    b"pointer" => MemoryTokenSegment::Pointer,
    b"static" => MemoryTokenSegment::Static,
    b"local" => MemoryTokenSegment::Local,
    b"temp" => MemoryTokenSegment::Temp,
    b"that" => MemoryTokenSegment::That,
    b"this" => MemoryTokenSegment::This
};

#[derive(Debug)]
pub struct FunctionMetadata {
    pub name: Vec<u8>,
    pub args_count: i16,
}

#[derive(Debug)]
pub enum FunctionToken {
    Definition(FunctionMetadata),
    Call(FunctionMetadata),
    Return,
}

#[derive(Debug)]
pub enum BranchTokenKind {
    Label,
    Goto,
    IfGoto,
}

#[derive(Debug)]
pub struct BranchToken {
    pub kind: BranchTokenKind,
    pub name: Vec<u8>,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, SymbolicElem)]
pub enum MemoryTokenSegment {
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

impl fmt::Display for MemoryTokenSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            MemoryTokenSegment::Arg => "argument",
            MemoryTokenSegment::Const => "constant",
            MemoryTokenSegment::Pointer => "pointer",
            MemoryTokenSegment::Static => "static",
            MemoryTokenSegment::Local => "local",
            MemoryTokenSegment::Temp => "temp",
            MemoryTokenSegment::That => "that",
            MemoryTokenSegment::This => "this",
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
    pub segment: MemoryTokenSegment,
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
    Branch(BranchToken),
    Function(FunctionToken),
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
            TokenPayload::Function(_) => Ok(()),
            TokenPayload::Branch(_) => Ok(()),
            TokenPayload::Memory(memory_token) => memory_token.fmt(f),
            TokenPayload::Arithmetic(arithmetic_token) => arithmetic_token.fmt(f),
        }
    }
}
