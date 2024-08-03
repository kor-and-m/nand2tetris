use phf::phf_map;
use std::fmt;

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

#[derive(Debug)]
pub enum AsmFunctionInstruction {
    Definition(FunctionMetadata),
    Call(FunctionMetadata),
    Return,
}

#[derive(Debug)]
pub enum AsmBranchInstructionKind {
    Label,
    Goto,
    IfGoto,
}

#[derive(Debug)]
pub struct AsmBranchInstruction {
    pub kind: AsmBranchInstructionKind,
    pub name: Vec<u8>,
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
            AsmMemoryInstructionSegment::Arg => "argument",
            AsmMemoryInstructionSegment::Const => "constant",
            AsmMemoryInstructionSegment::Pointer => "pointer",
            AsmMemoryInstructionSegment::Static => "static",
            AsmMemoryInstructionSegment::Local => "local",
            AsmMemoryInstructionSegment::Temp => "temp",
            AsmMemoryInstructionSegment::That => "that",
            AsmMemoryInstructionSegment::This => "this",
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
            AsmMemoryInstructionKind::Pop => "pop",
            AsmMemoryInstructionKind::Push => "push",
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
            AsmArithmeticInstruction::Add => "add",
            AsmArithmeticInstruction::Sub => "sub",
            AsmArithmeticInstruction::Neg => "neg",
            AsmArithmeticInstruction::Eq => "eq",
            AsmArithmeticInstruction::Gt => "gt",
            AsmArithmeticInstruction::Lt => "lt",
            AsmArithmeticInstruction::And => "and",
            AsmArithmeticInstruction::Or => "or",
            AsmArithmeticInstruction::Not => "not",
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

impl fmt::Display for AsmInstructionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            AsmInstructionPayload::Function(_) => Ok(()),
            AsmInstructionPayload::Branch(_) => Ok(()),
            AsmInstructionPayload::Memory(memory_instruction) => memory_instruction.fmt(f),
            AsmInstructionPayload::Arithmetic(arithmetic_instruction) => {
                arithmetic_instruction.fmt(f)
            }
        }
    }
}
