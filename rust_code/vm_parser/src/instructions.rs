use phf::phf_map;
use std::{fmt, str::from_utf8};

use hack_macro::SymbolicElem;
use symbolic::SymbolicElem;

pub static SEGMENTS: phf::Map<&'static [u8], AsmMemoryInstructionSegment> = phf_map! {
    b"argument" => AsmMemoryInstructionSegment::Arg,
    b"constant" => AsmMemoryInstructionSegment::Const,
    b"pointer" => AsmMemoryInstructionSegment::Pointer,
    b"static" => AsmMemoryInstructionSegment::Static,
    b"local" => AsmMemoryInstructionSegment::Local,
    b"temp" => AsmMemoryInstructionSegment::Temp,
    b"that" => AsmMemoryInstructionSegment::That,
    b"this" => AsmMemoryInstructionSegment::This
};

#[derive(Debug)]
pub struct FunctionMetadata {
    pub name: Vec<u8>,
    pub args_count: i16,
}

impl fmt::Display for FunctionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", from_utf8(&self.name).unwrap(), self.args_count)
    }
}

#[derive(Debug)]
pub enum AsmFunctionInstruction {
    Definition(FunctionMetadata),
    Call(FunctionMetadata),
    Return,
}

impl fmt::Display for AsmFunctionInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Call(meta) => {
                write!(f, "call ")?;
                meta.fmt(f)
            }
            Self::Definition(meta) => {
                write!(f, "function ")?;
                meta.fmt(f)
            }
            Self::Return => write!(f, "return"),
        }
    }
}

#[derive(Debug)]
pub enum AsmBranchInstructionKind {
    Label,
    Goto,
    IfGoto,
}

impl fmt::Display for AsmBranchInstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Label => write!(f, "label"),
            Self::Goto => write!(f, "goto"),
            Self::IfGoto => write!(f, "if-goto"),
        }
    }
}

#[derive(Debug)]
pub struct AsmBranchInstruction {
    pub kind: AsmBranchInstructionKind,
    pub name: Vec<u8>,
}

impl fmt::Display for AsmBranchInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        write!(f, " {}", from_utf8(&self.name).unwrap())
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, SymbolicElem)]
pub enum AsmMemoryInstructionSegment {
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

impl fmt::Display for AsmMemoryInstructionSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Arg => "argument",
            Self::Const => "constant",
            Self::Pointer => "pointer",
            Self::Static => "static",
            Self::Local => "local",
            Self::Temp => "temp",
            Self::That => "that",
            Self::This => "this",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug, PartialEq)]
pub enum AsmMemoryInstructionKind {
    Pop,
    Push,
}

impl fmt::Display for AsmMemoryInstructionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Pop => "pop",
            Self::Push => "push",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug)]
pub struct AsmMemoryInstruction {
    pub segment: AsmMemoryInstructionSegment,
    pub kind: AsmMemoryInstructionKind,
    pub val: i16,
}

impl fmt::Display for AsmMemoryInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.kind, self.segment, self.val)
    }
}

#[derive(Debug, SymbolicElem)]
pub enum AsmArithmeticInstruction {
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

impl fmt::Display for AsmArithmeticInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Neg => "neg",
            Self::Eq => "eq",
            Self::Gt => "gt",
            Self::Lt => "lt",
            Self::And => "and",
            Self::Or => "or",
            Self::Not => "not",
        };

        write!(f, "{}", label)
    }
}

#[derive(Debug)]
pub enum AsmInstructionPayload {
    Memory(AsmMemoryInstruction),
    Arithmetic(AsmArithmeticInstruction),
    Branch(AsmBranchInstruction),
    Function(AsmFunctionInstruction),
}

impl Default for AsmInstructionPayload {
    fn default() -> Self {
        Self::Arithmetic(AsmArithmeticInstruction::Add)
    }
}

impl fmt::Display for AsmInstructionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AsmInstructionPayload::Function(function_instruction) => function_instruction.fmt(f),
            AsmInstructionPayload::Branch(branch_instruction) => branch_instruction.fmt(f),
            AsmInstructionPayload::Memory(memory_instruction) => memory_instruction.fmt(f),
            AsmInstructionPayload::Arithmetic(arithmetic_instruction) => {
                arithmetic_instruction.fmt(f)
            }
        }
    }
}
