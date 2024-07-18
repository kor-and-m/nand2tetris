#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
mod ast {
    use crate::symbolic::SymbolicElem;
    use hack_macro::SymbolicElem;
    pub fn write_instruction_set(
        buff: &mut [u8],
        instructions: &[Instruction<'_>],
    ) -> usize {
        let mut cursor = 0;
        for i in instructions.iter() {
            cursor += i.write_symbols(&mut buff[cursor..]);
            buff[cursor] = b'\n';
            cursor += 1;
        }
        cursor
    }
    pub enum Instruction<'a> {
        A(AInstruction<'a>),
        C(CInstruction),
    }
    pub struct VariableFactory<'a> {
        idx: i16,
        prefix: &'a [u8],
        l: usize,
    }
    impl VariableFactory<'_> {
        pub fn new<'a>(prefix: &'a [u8]) -> VariableFactory<'a> {
            VariableFactory {
                prefix,
                idx: 0,
                l: prefix.len(),
            }
        }
        pub fn new_variable(&mut self) -> AInstruction<'_> {
            let i = AInstruction::Variable((self.prefix, self.l, self.idx));
            self.idx += 1;
            i
        }
    }
    impl<'a> SymbolicElem<'a> for Instruction<'a> {
        const MAX_SYMBOLIC_LEN: usize = if AInstruction::MAX_SYMBOLIC_LEN
            > CInstruction::MAX_SYMBOLIC_LEN
        {
            AInstruction::MAX_SYMBOLIC_LEN
        } else {
            CInstruction::MAX_SYMBOLIC_LEN
        };
        fn read_symbols(buff: &'a [u8]) -> Option<(Instruction<'a>, usize)> {
            if buff[0] == b'@' {
                AInstruction::read_symbols(buff).map(|res| (Self::A(res.0), res.1))
            } else {
                CInstruction::read_symbols(buff).map(|res| (Self::C(res.0), res.1))
            }
        }
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Instruction::A(ainstruction) => ainstruction.write_symbols(buff),
                Instruction::C(cinstruction) => cinstruction.write_symbols(buff),
            }
        }
    }
    impl Instruction<'_> {
        pub const fn new_const(c: AConst) -> Self {
            Instruction::A(AInstruction::Const(c))
        }
        pub fn new_number(n: i16) -> Self {
            Instruction::A(AInstruction::Number(n))
        }
    }
    pub enum AInstruction<'a> {
        Number(i16),
        Variable((&'a [u8], usize, i16)),
        Const(AConst),
    }
    impl<'a> SymbolicElem<'a> for AInstruction<'a> {
        const MAX_SYMBOLIC_LEN: usize = AConst::MAX_SYMBOLIC_LEN + 1;
        fn read_symbols(buff: &'a [u8]) -> Option<(AInstruction<'a>, usize)> {
            if buff[0] != b'@' {
                return None;
            }
            if let Some((constant, constant_size)) = AConst::read_symbols(&buff[1..]) {
                Some((AInstruction::Const(constant), constant_size + 1))
            } else {
                let str_buff = std::str::from_utf8(&buff[1..]).unwrap();
                if let Ok(v) = str_buff.parse::<i16>() {
                    Some((AInstruction::Number(v), buff.len()))
                } else {
                    let varibale: Vec<&str> = str_buff.split(".").collect();
                    let n = varibale[1].parse::<i16>();
                    if varibale.len() != 2 || n.is_err() {
                        None
                    } else {
                        let l = varibale[0].len();
                        let prefix = varibale[0].as_bytes();
                        Some((
                            AInstruction::Variable((prefix, l, n.unwrap())),
                            buff.len(),
                        ))
                    }
                }
            }
        }
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            buff[0] = b'@';
            match self {
                Self::Const(const_instruction) => {
                    const_instruction.write_symbols(&mut buff[1..]) + 1
                }
                Self::Variable((prefix, l, n)) => {
                    buff[1..(l + 1)].copy_from_slice(prefix);
                    buff[l + 1] = b'.';
                    write_i16_to_buff(*n, &mut buff[(l + 2)..]) + l + 2
                }
                Self::Number(n) => write_i16_to_buff(*n, &mut buff[1..]) + 1,
            }
        }
    }
    pub enum AConst {
        SP,
        LCL,
        ARG,
        THIS,
        THAT,
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6,
        R7,
        R8,
        R9,
        R10,
        R11,
        R12,
        R13,
        R14,
        R15,
        SCREEN,
        KBD,
    }
    impl<'a> SymbolicElem<'a> for AConst {
        const MAX_SYMBOLIC_LEN: usize = 6;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::SCREEN => {
                    buff[..6].copy_from_slice(b"SCREEN");
                    6
                }
                Self::THIS => {
                    buff[..4].copy_from_slice(b"THIS");
                    4
                }
                Self::THAT => {
                    buff[..4].copy_from_slice(b"THAT");
                    4
                }
                Self::LCL => {
                    buff[..3].copy_from_slice(b"LCL");
                    3
                }
                Self::ARG => {
                    buff[..3].copy_from_slice(b"ARG");
                    3
                }
                Self::R10 => {
                    buff[..3].copy_from_slice(b"R10");
                    3
                }
                Self::R11 => {
                    buff[..3].copy_from_slice(b"R11");
                    3
                }
                Self::R12 => {
                    buff[..3].copy_from_slice(b"R12");
                    3
                }
                Self::R13 => {
                    buff[..3].copy_from_slice(b"R13");
                    3
                }
                Self::R14 => {
                    buff[..3].copy_from_slice(b"R14");
                    3
                }
                Self::R15 => {
                    buff[..3].copy_from_slice(b"R15");
                    3
                }
                Self::KBD => {
                    buff[..3].copy_from_slice(b"KBD");
                    3
                }
                Self::SP => {
                    buff[..2].copy_from_slice(b"SP");
                    2
                }
                Self::R0 => {
                    buff[..2].copy_from_slice(b"R0");
                    2
                }
                Self::R1 => {
                    buff[..2].copy_from_slice(b"R1");
                    2
                }
                Self::R2 => {
                    buff[..2].copy_from_slice(b"R2");
                    2
                }
                Self::R3 => {
                    buff[..2].copy_from_slice(b"R3");
                    2
                }
                Self::R4 => {
                    buff[..2].copy_from_slice(b"R4");
                    2
                }
                Self::R5 => {
                    buff[..2].copy_from_slice(b"R5");
                    2
                }
                Self::R6 => {
                    buff[..2].copy_from_slice(b"R6");
                    2
                }
                Self::R7 => {
                    buff[..2].copy_from_slice(b"R7");
                    2
                }
                Self::R8 => {
                    buff[..2].copy_from_slice(b"R8");
                    2
                }
                Self::R9 => {
                    buff[..2].copy_from_slice(b"R9");
                    2
                }
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b'S', b'C', b'R', b'E', b'E', b'N', ..] => Some((Self::SCREEN, 6)),
                [b'T', b'H', b'I', b'S', ..] => Some((Self::THIS, 4)),
                [b'T', b'H', b'A', b'T', ..] => Some((Self::THAT, 4)),
                [b'L', b'C', b'L', ..] => Some((Self::LCL, 3)),
                [b'A', b'R', b'G', ..] => Some((Self::ARG, 3)),
                [b'R', b'1', b'0', ..] => Some((Self::R10, 3)),
                [b'R', b'1', b'1', ..] => Some((Self::R11, 3)),
                [b'R', b'1', b'2', ..] => Some((Self::R12, 3)),
                [b'R', b'1', b'3', ..] => Some((Self::R13, 3)),
                [b'R', b'1', b'4', ..] => Some((Self::R14, 3)),
                [b'R', b'1', b'5', ..] => Some((Self::R15, 3)),
                [b'K', b'B', b'D', ..] => Some((Self::KBD, 3)),
                [b'S', b'P', ..] => Some((Self::SP, 2)),
                [b'R', b'0', ..] => Some((Self::R0, 2)),
                [b'R', b'1', ..] => Some((Self::R1, 2)),
                [b'R', b'2', ..] => Some((Self::R2, 2)),
                [b'R', b'3', ..] => Some((Self::R3, 2)),
                [b'R', b'4', ..] => Some((Self::R4, 2)),
                [b'R', b'5', ..] => Some((Self::R5, 2)),
                [b'R', b'6', ..] => Some((Self::R6, 2)),
                [b'R', b'7', ..] => Some((Self::R7, 2)),
                [b'R', b'8', ..] => Some((Self::R8, 2)),
                [b'R', b'9', ..] => Some((Self::R9, 2)),
                _ => None,
            }
        }
    }
    pub enum CInstructionExpression {
        #[hack(symbol = b"0")]
        Zero,
        #[hack(symbol = b"1")]
        One,
        #[hack(symbol = b"-1")]
        MinusOne,
        #[hack(symbol = b"D")]
        D,
        #[hack(symbol = b"A")]
        A,
        #[hack(symbol = b"M")]
        M,
        #[hack(symbol = b"!D")]
        NotD,
        #[hack(symbol = b"!A")]
        NotA,
        #[hack(symbol = b"!M")]
        NotM,
        #[hack(symbol = b"-D")]
        MinusD,
        #[hack(symbol = b"-A")]
        MinusA,
        #[hack(symbol = b"-M")]
        MinusM,
        #[hack(symbol = b"D+1")]
        IncrementD,
        #[hack(symbol = b"A+1")]
        IncrementA,
        #[hack(symbol = b"M+1")]
        IncrementM,
        #[hack(symbol = b"A-1")]
        DecrementA,
        #[hack(symbol = b"M-1")]
        DecrementM,
        #[hack(symbol = b"D-1")]
        DecrementD,
        #[hack(symbol = b"D+A")]
        DPlusA,
        #[hack(symbol = b"D+M")]
        DPlusM,
        #[hack(symbol = b"D-A")]
        DMinusA,
        #[hack(symbol = b"D-M")]
        DMinusM,
        #[hack(symbol = b"A-D")]
        AMinusD,
        #[hack(symbol = b"M-D")]
        MMinusD,
        #[hack(symbol = b"D&A")]
        DAndA,
        #[hack(symbol = b"D&M")]
        DAndM,
        #[hack(symbol = b"D|A")]
        DOrA,
        #[hack(symbol = b"D|M")]
        DOrM,
    }
    impl<'a> SymbolicElem<'a> for CInstructionExpression {
        const MAX_SYMBOLIC_LEN: usize = 3;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::IncrementD => {
                    buff[..3].copy_from_slice(b"D+1");
                    3
                }
                Self::IncrementA => {
                    buff[..3].copy_from_slice(b"A+1");
                    3
                }
                Self::IncrementM => {
                    buff[..3].copy_from_slice(b"M+1");
                    3
                }
                Self::DecrementA => {
                    buff[..3].copy_from_slice(b"A-1");
                    3
                }
                Self::DecrementM => {
                    buff[..3].copy_from_slice(b"M-1");
                    3
                }
                Self::DecrementD => {
                    buff[..3].copy_from_slice(b"D-1");
                    3
                }
                Self::DPlusA => {
                    buff[..3].copy_from_slice(b"D+A");
                    3
                }
                Self::DPlusM => {
                    buff[..3].copy_from_slice(b"D+M");
                    3
                }
                Self::DMinusA => {
                    buff[..3].copy_from_slice(b"D-A");
                    3
                }
                Self::DMinusM => {
                    buff[..3].copy_from_slice(b"D-M");
                    3
                }
                Self::AMinusD => {
                    buff[..3].copy_from_slice(b"A-D");
                    3
                }
                Self::MMinusD => {
                    buff[..3].copy_from_slice(b"M-D");
                    3
                }
                Self::DAndA => {
                    buff[..3].copy_from_slice(b"D&A");
                    3
                }
                Self::DAndM => {
                    buff[..3].copy_from_slice(b"D&M");
                    3
                }
                Self::DOrA => {
                    buff[..3].copy_from_slice(b"D|A");
                    3
                }
                Self::DOrM => {
                    buff[..3].copy_from_slice(b"D|M");
                    3
                }
                Self::MinusOne => {
                    buff[..2].copy_from_slice(b"-1");
                    2
                }
                Self::NotD => {
                    buff[..2].copy_from_slice(b"!D");
                    2
                }
                Self::NotA => {
                    buff[..2].copy_from_slice(b"!A");
                    2
                }
                Self::NotM => {
                    buff[..2].copy_from_slice(b"!M");
                    2
                }
                Self::MinusD => {
                    buff[..2].copy_from_slice(b"-D");
                    2
                }
                Self::MinusA => {
                    buff[..2].copy_from_slice(b"-A");
                    2
                }
                Self::MinusM => {
                    buff[..2].copy_from_slice(b"-M");
                    2
                }
                Self::Zero => {
                    buff[..1].copy_from_slice(b"0");
                    1
                }
                Self::One => {
                    buff[..1].copy_from_slice(b"1");
                    1
                }
                Self::D => {
                    buff[..1].copy_from_slice(b"D");
                    1
                }
                Self::A => {
                    buff[..1].copy_from_slice(b"A");
                    1
                }
                Self::M => {
                    buff[..1].copy_from_slice(b"M");
                    1
                }
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b'D', b'+', b'1', ..] => Some((Self::IncrementD, 3)),
                [b'A', b'+', b'1', ..] => Some((Self::IncrementA, 3)),
                [b'M', b'+', b'1', ..] => Some((Self::IncrementM, 3)),
                [b'A', b'-', b'1', ..] => Some((Self::DecrementA, 3)),
                [b'M', b'-', b'1', ..] => Some((Self::DecrementM, 3)),
                [b'D', b'-', b'1', ..] => Some((Self::DecrementD, 3)),
                [b'D', b'+', b'A', ..] => Some((Self::DPlusA, 3)),
                [b'D', b'+', b'M', ..] => Some((Self::DPlusM, 3)),
                [b'D', b'-', b'A', ..] => Some((Self::DMinusA, 3)),
                [b'D', b'-', b'M', ..] => Some((Self::DMinusM, 3)),
                [b'A', b'-', b'D', ..] => Some((Self::AMinusD, 3)),
                [b'M', b'-', b'D', ..] => Some((Self::MMinusD, 3)),
                [b'D', b'&', b'A', ..] => Some((Self::DAndA, 3)),
                [b'D', b'&', b'M', ..] => Some((Self::DAndM, 3)),
                [b'D', b'|', b'A', ..] => Some((Self::DOrA, 3)),
                [b'D', b'|', b'M', ..] => Some((Self::DOrM, 3)),
                [b'-', b'1', ..] => Some((Self::MinusOne, 2)),
                [b'!', b'D', ..] => Some((Self::NotD, 2)),
                [b'!', b'A', ..] => Some((Self::NotA, 2)),
                [b'!', b'M', ..] => Some((Self::NotM, 2)),
                [b'-', b'D', ..] => Some((Self::MinusD, 2)),
                [b'-', b'A', ..] => Some((Self::MinusA, 2)),
                [b'-', b'M', ..] => Some((Self::MinusM, 2)),
                [b'0', ..] => Some((Self::Zero, 1)),
                [b'1', ..] => Some((Self::One, 1)),
                [b'D', ..] => Some((Self::D, 1)),
                [b'A', ..] => Some((Self::A, 1)),
                [b'M', ..] => Some((Self::M, 1)),
                _ => None,
            }
        }
    }
    #[hack(suffix = b"=")]
    pub enum CInstructionDest {
        ADM,
        AD,
        AM,
        A,
        DM,
        D,
        M,
        #[hack(symbol = b"")]
        NODEST,
    }
    impl<'a> SymbolicElem<'a> for CInstructionDest {
        const MAX_SYMBOLIC_LEN: usize = 4;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::ADM => {
                    buff[..4].copy_from_slice(b"ADM=");
                    4
                }
                Self::AD => {
                    buff[..3].copy_from_slice(b"AD=");
                    3
                }
                Self::AM => {
                    buff[..3].copy_from_slice(b"AM=");
                    3
                }
                Self::DM => {
                    buff[..3].copy_from_slice(b"DM=");
                    3
                }
                Self::A => {
                    buff[..2].copy_from_slice(b"A=");
                    2
                }
                Self::D => {
                    buff[..2].copy_from_slice(b"D=");
                    2
                }
                Self::M => {
                    buff[..2].copy_from_slice(b"M=");
                    2
                }
                Self::NODEST => 0,
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b'A', b'D', b'M', b'=', ..] => Some((Self::ADM, 4)),
                [b'A', b'D', b'=', ..] => Some((Self::AD, 3)),
                [b'A', b'M', b'=', ..] => Some((Self::AM, 3)),
                [b'D', b'M', b'=', ..] => Some((Self::DM, 3)),
                [b'A', b'=', ..] => Some((Self::A, 2)),
                [b'D', b'=', ..] => Some((Self::D, 2)),
                [b'M', b'=', ..] => Some((Self::M, 2)),
                _ => Some((Self::NODEST, 0)),
            }
        }
    }
    #[hack(prefix = b";")]
    pub enum CInstructionJump {
        JGT,
        JEQ,
        JGE,
        JLT,
        JNE,
        JLE,
        JMP,
        #[hack(symbol = b"")]
        NOJMP,
    }
    impl<'a> SymbolicElem<'a> for CInstructionJump {
        const MAX_SYMBOLIC_LEN: usize = 4;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::JGT => {
                    buff[..4].copy_from_slice(b";JGT");
                    4
                }
                Self::JEQ => {
                    buff[..4].copy_from_slice(b";JEQ");
                    4
                }
                Self::JGE => {
                    buff[..4].copy_from_slice(b";JGE");
                    4
                }
                Self::JLT => {
                    buff[..4].copy_from_slice(b";JLT");
                    4
                }
                Self::JNE => {
                    buff[..4].copy_from_slice(b";JNE");
                    4
                }
                Self::JLE => {
                    buff[..4].copy_from_slice(b";JLE");
                    4
                }
                Self::JMP => {
                    buff[..4].copy_from_slice(b";JMP");
                    4
                }
                Self::NOJMP => 0,
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b';', b'J', b'G', b'T', ..] => Some((Self::JGT, 4)),
                [b';', b'J', b'E', b'Q', ..] => Some((Self::JEQ, 4)),
                [b';', b'J', b'G', b'E', ..] => Some((Self::JGE, 4)),
                [b';', b'J', b'L', b'T', ..] => Some((Self::JLT, 4)),
                [b';', b'J', b'N', b'E', ..] => Some((Self::JNE, 4)),
                [b';', b'J', b'L', b'E', ..] => Some((Self::JLE, 4)),
                [b';', b'J', b'M', b'P', ..] => Some((Self::JMP, 4)),
                _ => Some((Self::NOJMP, 0)),
            }
        }
    }
    pub struct CInstruction {
        dest: CInstructionDest,
        expression: CInstructionExpression,
        jump: CInstructionJump,
    }
    impl<'a> SymbolicElem<'a> for CInstruction {
        const MAX_SYMBOLIC_LEN: usize = CInstructionDest::MAX_SYMBOLIC_LEN
            + CInstructionExpression::MAX_SYMBOLIC_LEN
            + CInstructionJump::MAX_SYMBOLIC_LEN;
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            let (dest, mut cursor) = CInstructionDest::read_symbols(buff).unwrap();
            let (expression, expr_size) = CInstructionExpression::read_symbols(
                &buff[cursor..],
            )?;
            cursor += expr_size;
            let (jump, jump_size) = CInstructionJump::read_symbols(&buff[cursor..])
                .unwrap();
            Some((Self { dest, expression, jump }, cursor + jump_size))
        }
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            let mut result = self.dest.write_symbols(buff);
            result += self.expression.write_symbols(&mut buff[result..]);
            result + self.jump.write_symbols(buff)
        }
    }
    fn write_i16_to_buff(n: i16, buff: &mut [u8]) -> usize {
        let mut idx = 0;
        for i in n.to_string().chars() {
            buff[idx] = i as u8;
            idx += 1;
        }
        idx
    }
}
mod lexer {
    use pin_utils::pin_mut;
    use std::fmt;
    use std::future::Future;
    use std::io::{Error, ErrorKind};
    use std::path::Path;
    use phf::phf_map;
    use tokio::fs::File;
    use tokio::io::{self, AsyncReadExt};
    use crate::tokens::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio_stream::Stream;
    const LEXER_BUFFER_SIZE: usize = 4096;
    pub struct VMLexer {
        file: File,
        buffer: [u8; LEXER_BUFFER_SIZE],
        cursor: usize,
        end_word_cursor: usize,
        end_page_cursor: usize,
        is_eof: bool,
        src_line: usize,
        instruction_number: usize,
        src_file_name: String,
        is_in_progress: bool,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for VMLexer {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "file",
                "buffer",
                "cursor",
                "end_word_cursor",
                "end_page_cursor",
                "is_eof",
                "src_line",
                "instruction_number",
                "src_file_name",
                "is_in_progress",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.file,
                &self.buffer,
                &self.cursor,
                &self.end_word_cursor,
                &self.end_page_cursor,
                &self.is_eof,
                &self.src_line,
                &self.instruction_number,
                &self.src_file_name,
                &&self.is_in_progress,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "VMLexer",
                names,
                values,
            )
        }
    }
    impl VMLexer {
        pub async fn new(path: &str) -> io::Result<Self> {
            let file_path = Path::new(&path);
            if file_path.extension().expect("File extension undefined") != "vm" {
                return Err(Error::new(ErrorKind::Other, "File ext should be vm!"));
            }
            let src_file_name = file_path
                .file_stem()
                .expect("Wrong stem")
                .to_str()
                .unwrap()
                .to_string();
            let file = File::open(&file_path).await?;
            let mut self_state = Self {
                file,
                src_file_name,
                buffer: [0; LEXER_BUFFER_SIZE],
                cursor: LEXER_BUFFER_SIZE,
                end_word_cursor: LEXER_BUFFER_SIZE,
                end_page_cursor: 0,
                is_eof: false,
                is_in_progress: false,
                instruction_number: 0,
                src_line: 1,
            };
            self_state.fill_buffer().await?;
            Ok(self_state)
        }
        async fn fill_buffer(&mut self) -> io::Result<()> {
            let to_copy = LEXER_BUFFER_SIZE - self.cursor;
            let p = self.buffer.as_mut_ptr();
            unsafe {
                std::ptr::copy_nonoverlapping(p.add(self.cursor), p, to_copy);
            }
            let n = self.file.read(&mut self.buffer[to_copy..]).await?;
            self.is_eof = self.cursor > n;
            self.end_word_cursor -= self.cursor;
            self.cursor = 0;
            self.end_page_cursor = n;
            Ok(())
        }
        async fn next_word(&mut self) -> Option<&[u8]> {
            let mut is_comment = false;
            let mut is_word = false;
            self.cursor = self.end_word_cursor;
            loop {
                if self.end_word_cursor >= self.end_page_cursor {
                    if self.is_eof {
                        return if is_word {
                            Some(&self.buffer[self.cursor..self.end_word_cursor])
                        } else {
                            None
                        };
                    } else {
                        self.fill_buffer().await.expect("fail to fill buffer");
                    }
                }
                match (is_comment, is_word, self.buffer[self.end_word_cursor]) {
                    (true, _, b'\n') => {
                        self.src_line += 1;
                        is_comment = false;
                        self.cursor += 1;
                        self.end_word_cursor += 1;
                    }
                    (true, _, _) => {
                        self.cursor += 1;
                        self.end_word_cursor += 1;
                    }
                    (false, _, b'/') => {
                        if self.buffer[self.end_word_cursor + 1] != b'/' {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!("Comment parsing error"),
                                );
                            }
                        }
                        self.cursor += 2;
                        self.end_word_cursor += 2;
                        is_comment = true;
                    }
                    (false, false, v) if v == b' ' || v == b'\t' || v == b'\r' => {
                        self.cursor += 1;
                        self.end_word_cursor += 1;
                    }
                    (false, false, b'\n') => {
                        self.src_line += 1;
                        self.cursor += 1;
                        self.end_word_cursor += 1;
                    }
                    (false, false, _) => is_word = true,
                    (
                        false,
                        true,
                        v,
                    ) if v == b' ' || v == b'\t' || v == b'\r' || v == b'\n' => {
                        if v == b'\n' {
                            self.src_line += 1;
                        }
                        return Some(&self.buffer[self.cursor..self.end_word_cursor]);
                    }
                    (false, true, _) => self.end_word_cursor += 1,
                }
            }
        }
        async fn build_memory_token(
            &mut self,
            kind: MemoryTokenKind,
        ) -> Option<TokenPayload> {
            let word = self.next_word().await.unwrap();
            let segment = SEGMENTS[word];
            let val: i16 = self
                .next_word()
                .await
                .unwrap()
                .iter()
                .fold(0, |v, x| v * 10 - 48 + *x as i16);
            Some(TokenPayload::Memory(MemoryToken { kind, segment, val }))
        }
        pub async fn next_token(&mut self) -> Option<Token> {
            self.is_in_progress = true;
            self.instruction_number += 1;
            let src_line = self.src_line.clone();
            let word = self.next_word().await?;
            let token_payload = match word {
                [b'p', b'u', b's', b'h', ..] => {
                    self.build_memory_token(MemoryTokenKind::Push)
                        .await
                        .expect("fail to build push token")
                }
                [b'p', b'o', b'p', ..] => {
                    self.build_memory_token(MemoryTokenKind::Pop)
                        .await
                        .expect("fail to build pop token")
                }
                [b'a', b'd', b'd', ..] => {
                    self.build_arithmetic_token(ArithmeticToken::Add)
                }
                [b's', b'u', b'b', ..] => {
                    self.build_arithmetic_token(ArithmeticToken::Sub)
                }
                [b'n', b'e', b'g', ..] => {
                    self.build_arithmetic_token(ArithmeticToken::Neg)
                }
                [b'e', b'q', ..] => self.build_arithmetic_token(ArithmeticToken::Eq),
                [b'g', b't', ..] => self.build_arithmetic_token(ArithmeticToken::Gt),
                [b'l', b't', ..] => self.build_arithmetic_token(ArithmeticToken::Lt),
                [b'a', b'n', b'd', ..] => {
                    self.build_arithmetic_token(ArithmeticToken::And)
                }
                [b'o', b'r', ..] => self.build_arithmetic_token(ArithmeticToken::Or),
                [b'n', b'o', b't', ..] => {
                    self.build_arithmetic_token(ArithmeticToken::Not)
                }
                _ => {
                    let i = std::str::from_utf8(word).unwrap();
                    {
                        ::core::panicking::panic_fmt(
                            format_args!("{0}: Unexected instruction {1}", src_line, i),
                        );
                    };
                }
            };
            Some(self.enrich_token_payload(token_payload))
        }
        fn build_arithmetic_token(&mut self, kind: ArithmeticToken) -> TokenPayload {
            TokenPayload::Arithmetic(kind)
        }
        fn enrich_token_payload(&self, payload: TokenPayload) -> Token {
            Token {
                payload,
                instruction: self.instruction_number,
                src: self.src_line,
            }
        }
    }
}
mod parser {
    use phf::phf_map;
    use std::borrow::Cow;
    const POP_FROM_SP: &str = "// Decriment sp\n@SP\nM=M-1\n// Extract value from sp\n@SP\nA=M\nD=M\n";
    const PUSH_TO_SP: &str = "// Write value to sp\n@SP\nA=M\nM=D\n// Incriment sp\n@SP\nM=M+1\n";
    enum Segment {
        Arg,
        Const,
        Pointer,
        Static,
        Local,
        Temp,
        That,
        This,
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Segment {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Segment {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Segment::Arg => "Arg",
                    Segment::Const => "Const",
                    Segment::Pointer => "Pointer",
                    Segment::Static => "Static",
                    Segment::Local => "Local",
                    Segment::Temp => "Temp",
                    Segment::That => "That",
                    Segment::This => "This",
                },
            )
        }
    }
    fn load_value_by_pointer_label(
        pointer_label: &str,
        mut val: i16,
    ) -> Cow<'static, str> {
        let mut s = {
            let res = ::alloc::fmt::format(format_args!("@{0}\nA=M\n", pointer_label));
            res
        };
        while val > 0 {
            s.push_str("A=A+1\n");
            val -= 1;
        }
        s.push_str("D=M\n");
        Cow::Owned(s)
    }
    fn save_value_to_pointer_label(
        pointer_label: &str,
        mut val: i16,
    ) -> Cow<'static, str> {
        let mut s = {
            let res = ::alloc::fmt::format(format_args!("@{0}\nA=M\n", pointer_label));
            res
        };
        while val > 0 {
            s.push_str("A=A+1\n");
            val -= 1;
        }
        s.push_str("M=D\n");
        Cow::Owned(s)
    }
    impl Segment {
        pub fn load_data(&self, val: i16, file_name: &str) -> Cow<'static, str> {
            match self {
                Self::Arg => load_value_by_pointer_label("ARG", val),
                Self::This => load_value_by_pointer_label("THIS", val),
                Self::That => load_value_by_pointer_label("THAT", val),
                Self::Local => load_value_by_pointer_label("LCL", val),
                Self::Const => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(format_args!("@{0}\nD=A\n", val));
                        res
                    })
                }
                Self::Temp => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!("@{0}\nD=M\n", val + 5),
                        );
                        res
                    })
                }
                Self::Static => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!("@{0}.{1}\nD=M\n", file_name, val),
                        );
                        res
                    })
                }
                Self::Pointer => {
                    match val {
                        0 => {
                            Cow::Owned({
                                let res = ::alloc::fmt::format(
                                    format_args!("@THIS\nD=M\n"),
                                );
                                res
                            })
                        }
                        1 => {
                            Cow::Owned({
                                let res = ::alloc::fmt::format(
                                    format_args!("@THAT\nD=M\n"),
                                );
                                res
                            })
                        }
                        _ => {
                            ::core::panicking::panic(
                                "internal error: entered unreachable code",
                            )
                        }
                    }
                }
            }
        }
        pub fn save_data(&self, val: i16, file_name: &str) -> Cow<'static, str> {
            match self {
                Self::Arg => save_value_to_pointer_label("ARG", val),
                Self::This => save_value_to_pointer_label("THIS", val),
                Self::That => save_value_to_pointer_label("THAT", val),
                Self::Local => save_value_to_pointer_label("LCL", val),
                Self::Const => {
                    ::core::panicking::panic("internal error: entered unreachable code")
                }
                Self::Temp => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!("@{0}\nM=D\n", val + 5),
                        );
                        res
                    })
                }
                Self::Static => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!("@{0}.{1}\nM=D\n", file_name, val),
                        );
                        res
                    })
                }
                Self::Pointer => {
                    match val {
                        0 => {
                            Cow::Owned({
                                let res = ::alloc::fmt::format(
                                    format_args!("@THIS\nM=D\n"),
                                );
                                res
                            })
                        }
                        1 => {
                            Cow::Owned({
                                let res = ::alloc::fmt::format(
                                    format_args!("@THAT\nM=D\n"),
                                );
                                res
                            })
                        }
                        _ => {
                            ::core::panicking::panic(
                                "internal error: entered unreachable code",
                            )
                        }
                    }
                }
            }
        }
    }
    enum MemoryTokenKind {
        Pop,
        Push,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryTokenKind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    MemoryTokenKind::Pop => "Pop",
                    MemoryTokenKind::Push => "Push",
                },
            )
        }
    }
    struct MemoryToken<'a> {
        segment: &'a Segment,
        kind: MemoryTokenKind,
        val: i16,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for MemoryToken<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "MemoryToken",
                "segment",
                &self.segment,
                "kind",
                &self.kind,
                "val",
                &&self.val,
            )
        }
    }
    impl<'a> MemoryToken<'a> {
        pub fn new(kind: MemoryTokenKind, segment: &'a Segment, val: i16) -> Self {
            Self { segment, val, kind }
        }
    }
    enum BranchTokenKind {
        Label,
        Goto,
        IfGoto,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BranchTokenKind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    BranchTokenKind::Label => "Label",
                    BranchTokenKind::Goto => "Goto",
                    BranchTokenKind::IfGoto => "IfGoto",
                },
            )
        }
    }
    struct BranchToken<'a> {
        kind: BranchTokenKind,
        name: &'a str,
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for BranchToken<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "BranchToken",
                "kind",
                &self.kind,
                "name",
                &&self.name,
            )
        }
    }
    enum ArithmeticTokens {
        Add,
        Sub,
        Neg,
        Eq,
        Gt,
        Lt,
        And,
        Or,
        Not,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ArithmeticTokens {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ArithmeticTokens::Add => "Add",
                    ArithmeticTokens::Sub => "Sub",
                    ArithmeticTokens::Neg => "Neg",
                    ArithmeticTokens::Eq => "Eq",
                    ArithmeticTokens::Gt => "Gt",
                    ArithmeticTokens::Lt => "Lt",
                    ArithmeticTokens::And => "And",
                    ArithmeticTokens::Or => "Or",
                    ArithmeticTokens::Not => "Not",
                },
            )
        }
    }
    static SEGMENTS: phf::Map<&'static [u8], Segment> = phf::Map {
        key: 10121458955350035957u64,
        disps: &[(1u32, 0u32), (0u32, 0u32)],
        entries: &[
            (b"argument", Segment::Arg),
            (b"that", Segment::That),
            (b"this", Segment::This),
            (b"temp", Segment::Temp),
            (b"pointer", Segment::Pointer),
            (b"local", Segment::Local),
            (b"constant", Segment::Const),
            (b"static", Segment::Static),
        ],
    };
    trait Assemble {
        fn assemble(&self, file_name: &str, idx: usize) -> Cow<'static, str>;
    }
    impl Assemble for MemoryToken<'_> {
        fn assemble(&self, file_name: &str, _idx: usize) -> Cow<'static, str> {
            let command = {
                let res = ::alloc::fmt::format(
                    format_args!(
                        "// {0:?} {1:?} {2}\n",
                        self.kind,
                        self.segment,
                        self.val,
                    ),
                );
                res
            };
            match &self.kind {
                MemoryTokenKind::Pop => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "\n{0}{1}{2}\n",
                                command,
                                POP_FROM_SP,
                                self.segment.save_data(self.val, file_name),
                            ),
                        );
                        res
                    })
                }
                MemoryTokenKind::Push => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "\n{0}{1}{2}\n",
                                command,
                                self.segment.load_data(self.val, file_name),
                                PUSH_TO_SP,
                            ),
                        );
                        res
                    })
                }
            }
        }
    }
    fn two_elements_operation(name: &str, operand: &str) -> Cow<'static, str> {
        Cow::Owned({
            let res = ::alloc::fmt::format(
                format_args!(
                    "// {0}\n{1}// Update in place sp\n@SP\nA=M-1\nM=M{2}D\n",
                    name,
                    POP_FROM_SP,
                    operand,
                ),
            );
            res
        })
    }
    fn cmp_operation(name: &str, jump: &str, idx: usize) -> Cow<'static, str> {
        let to_bool = {
            let res = ::alloc::fmt::format(
                format_args!(
                    "@TRUE{0}\nD;{1}\n@SP\nA=M-1\nM=0\n@FALSE{0}\n0;JMP\n(TRUE{0})\n@SP\nA=M-1\nM=-1\n(FALSE{0})\n",
                    idx,
                    jump,
                ),
            );
            res
        };
        Cow::Owned({
            let res = ::alloc::fmt::format(
                format_args!(
                    "// {0}\n{1}// Write cmp result sp\n@SP\nA=M-1\nD=M-D\n{2}",
                    name,
                    POP_FROM_SP,
                    to_bool,
                ),
            );
            res
        })
    }
    impl Assemble for BranchToken<'_> {
        fn assemble(&self, _file_name: &str, _idx: usize) -> Cow<'static, str> {
            match self.kind {
                BranchTokenKind::Label => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(format_args!("({0})", self.name));
                        res
                    })
                }
                BranchTokenKind::Goto => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!("// goto\n@{0}\n0;JMP\n", self.name),
                        );
                        res
                    })
                }
                BranchTokenKind::IfGoto => {
                    Cow::Owned({
                        let res = ::alloc::fmt::format(
                            format_args!(
                                "// if goto\n{0}\n@{1}\nD;JNE\n",
                                POP_FROM_SP,
                                self.name,
                            ),
                        );
                        res
                    })
                }
            }
        }
    }
    impl Assemble for ArithmeticTokens {
        fn assemble(&self, _file_name: &str, idx: usize) -> Cow<'static, str> {
            match self {
                ArithmeticTokens::Add => two_elements_operation("add", "+"),
                ArithmeticTokens::Sub => two_elements_operation("sub", "-"),
                ArithmeticTokens::Neg => {
                    Cow::Borrowed("// neg\n// Update in place sp\n@SP\nA=M-1\nM=-M\n")
                }
                ArithmeticTokens::Eq => cmp_operation("eq", "JEQ", idx),
                ArithmeticTokens::Gt => cmp_operation("gt", "JGT", idx),
                ArithmeticTokens::Lt => cmp_operation("lt", "JLT", idx),
                ArithmeticTokens::Not => {
                    Cow::Borrowed("// not\n// Update in place sp\n@SP\nA=M-1\nM=!M\n")
                }
                ArithmeticTokens::Or => two_elements_operation("or", "|"),
                ArithmeticTokens::And => two_elements_operation("and", "&"),
            }
        }
    }
    fn cupture_word(byte_list: &[u8]) -> usize {
        for (i, &item) in byte_list.iter().enumerate() {
            if item == b' ' {
                return i;
            }
        }
        byte_list.len()
    }
    pub fn assemble(s: &str, file_name: &str, idx: usize) -> Cow<'static, str> {
        let mut pointer = 0;
        let b = s.as_bytes();
        let l = b.len();
        let new_pointer;
        while pointer < l {
            match b[pointer] as char {
                ' ' => {}
                '\t' => {}
                '/' => {
                    if '/' == b[pointer + 1] as char {
                        return Cow::Borrowed("");
                    }
                    ::core::panicking::panic("internal error: entered unreachable code");
                }
                _ => {
                    new_pointer = pointer + cupture_word(&b[pointer..]);
                    let token: Box<dyn Assemble> = match &b[pointer..new_pointer] {
                        b"push" => {
                            Box::new(
                                build_memory_command(MemoryTokenKind::Push, b, new_pointer),
                            )
                        }
                        b"pop" => {
                            Box::new(
                                build_memory_command(MemoryTokenKind::Pop, b, new_pointer),
                            )
                        }
                        b"add" => Box::new(ArithmeticTokens::Add),
                        b"sub" => Box::new(ArithmeticTokens::Sub),
                        b"neg" => Box::new(ArithmeticTokens::Neg),
                        b"eq" => Box::new(ArithmeticTokens::Eq),
                        b"gt" => Box::new(ArithmeticTokens::Gt),
                        b"lt" => Box::new(ArithmeticTokens::Lt),
                        b"and" => Box::new(ArithmeticTokens::And),
                        b"or" => Box::new(ArithmeticTokens::Or),
                        b"not" => Box::new(ArithmeticTokens::Not),
                        b"label" => {
                            Box::new(
                                build_branch_command(BranchTokenKind::Label, b, new_pointer),
                            )
                        }
                        b"goto" => {
                            Box::new(
                                build_branch_command(BranchTokenKind::Goto, b, new_pointer),
                            )
                        }
                        b"if-goto" => {
                            Box::new(
                                build_branch_command(
                                    BranchTokenKind::IfGoto,
                                    b,
                                    new_pointer,
                                ),
                            )
                        }
                        v => {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "Unexpected command {0}",
                                    std::str::from_utf8(v).unwrap(),
                                ),
                            );
                        }
                    };
                    return token.assemble(file_name, idx);
                }
            }
            pointer += 1;
        }
        Cow::Borrowed("")
    }
    fn build_branch_command(
        command_kind: BranchTokenKind,
        b: &[u8],
        mut new_pointer: usize,
    ) -> BranchToken {
        let pointer = new_pointer + 1;
        new_pointer = pointer + cupture_word(&b[pointer..]);
        let name = std::str::from_utf8(&b[pointer..new_pointer]).unwrap();
        BranchToken {
            kind: command_kind,
            name,
        }
    }
    fn build_memory_command(
        command_kind: MemoryTokenKind,
        b: &[u8],
        mut new_pointer: usize,
    ) -> MemoryToken {
        let mut pointer = new_pointer + 1;
        new_pointer = pointer + cupture_word(&b[pointer..]);
        let segment = &SEGMENTS[&b[pointer..new_pointer]];
        pointer = new_pointer + 1;
        new_pointer = pointer + cupture_word(&b[pointer..]);
        let v: i16 = b[pointer..new_pointer]
            .iter()
            .fold(0, |v, x| v * 10 - 48 + *x as i16);
        MemoryToken::new(command_kind, segment, v)
    }
}
mod symbolic {
    pub trait SymbolicElem<'a>: Sized {
        const MAX_SYMBOLIC_LEN: usize;
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)>;
        fn write_symbols(&self, buff: &mut [u8]) -> usize;
    }
}
mod tokens {
    use phf::phf_map;
    use std::fmt;
    use crate::symbolic::SymbolicElem;
    use hack_macro::SymbolicElem;
    pub static SEGMENTS: phf::Map<&'static [u8], Segment> = phf::Map {
        key: 10121458955350035957u64,
        disps: &[(1u32, 0u32), (0u32, 0u32)],
        entries: &[
            (b"argument", Segment::Arg),
            (b"that", Segment::That),
            (b"this", Segment::This),
            (b"temp", Segment::Temp),
            (b"pointer", Segment::Pointer),
            (b"local", Segment::Local),
            (b"constant", Segment::Const),
            (b"static", Segment::Static),
        ],
    };
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
    #[automatically_derived]
    impl ::core::hash::Hash for Segment {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_tag, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Segment {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Segment::Arg => "Arg",
                    Segment::Const => "Const",
                    Segment::Pointer => "Pointer",
                    Segment::Static => "Static",
                    Segment::Local => "Local",
                    Segment::Temp => "Temp",
                    Segment::That => "That",
                    Segment::This => "This",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Segment {
        #[inline]
        fn clone(&self) -> Segment {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Segment {}
    impl<'a> SymbolicElem<'a> for Segment {
        const MAX_SYMBOLIC_LEN: usize = 8;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::Arg => {
                    buff[..8].copy_from_slice(b"argument");
                    8
                }
                Self::Const => {
                    buff[..8].copy_from_slice(b"constant");
                    8
                }
                Self::Pointer => {
                    buff[..7].copy_from_slice(b"pointer");
                    7
                }
                Self::Static => {
                    buff[..6].copy_from_slice(b"static");
                    6
                }
                Self::Local => {
                    buff[..5].copy_from_slice(b"local");
                    5
                }
                Self::Temp => {
                    buff[..4].copy_from_slice(b"temp");
                    4
                }
                Self::That => {
                    buff[..4].copy_from_slice(b"that");
                    4
                }
                Self::This => {
                    buff[..4].copy_from_slice(b"this");
                    4
                }
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b'a', b'r', b'g', b'u', b'm', b'e', b'n', b't', ..] => {
                    Some((Self::Arg, 8))
                }
                [b'c', b'o', b'n', b's', b't', b'a', b'n', b't', ..] => {
                    Some((Self::Const, 8))
                }
                [b'p', b'o', b'i', b'n', b't', b'e', b'r', ..] => {
                    Some((Self::Pointer, 7))
                }
                [b's', b't', b'a', b't', b'i', b'c', ..] => Some((Self::Static, 6)),
                [b'l', b'o', b'c', b'a', b'l', ..] => Some((Self::Local, 5)),
                [b't', b'e', b'm', b'p', ..] => Some((Self::Temp, 4)),
                [b't', b'h', b'a', b't', ..] => Some((Self::That, 4)),
                [b't', b'h', b'i', b's', ..] => Some((Self::This, 4)),
                _ => None,
            }
        }
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
            f.write_fmt(format_args!("{0}", label))
        }
    }
    pub enum MemoryTokenKind {
        Pop,
        Push,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryTokenKind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    MemoryTokenKind::Pop => "Pop",
                    MemoryTokenKind::Push => "Push",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for MemoryTokenKind {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for MemoryTokenKind {
        #[inline]
        fn eq(&self, other: &MemoryTokenKind) -> bool {
            let __self_tag = ::core::intrinsics::discriminant_value(self);
            let __arg1_tag = ::core::intrinsics::discriminant_value(other);
            __self_tag == __arg1_tag
        }
    }
    impl fmt::Display for MemoryTokenKind {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let label = match self {
                MemoryTokenKind::Pop => "pop",
                MemoryTokenKind::Push => "push",
            };
            f.write_fmt(format_args!("{0}", label))
        }
    }
    pub struct MemoryToken {
        pub segment: Segment,
        pub kind: MemoryTokenKind,
        pub val: i16,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MemoryToken {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "MemoryToken",
                "segment",
                &self.segment,
                "kind",
                &self.kind,
                "val",
                &&self.val,
            )
        }
    }
    impl fmt::Display for MemoryToken {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("{0} {1} {2}", self.kind, self.segment, self.val))
        }
    }
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
    #[automatically_derived]
    impl ::core::fmt::Debug for ArithmeticToken {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    ArithmeticToken::Add => "Add",
                    ArithmeticToken::Sub => "Sub",
                    ArithmeticToken::Neg => "Neg",
                    ArithmeticToken::Eq => "Eq",
                    ArithmeticToken::Gt => "Gt",
                    ArithmeticToken::Lt => "Lt",
                    ArithmeticToken::And => "And",
                    ArithmeticToken::Or => "Or",
                    ArithmeticToken::Not => "Not",
                },
            )
        }
    }
    impl<'a> SymbolicElem<'a> for ArithmeticToken {
        const MAX_SYMBOLIC_LEN: usize = 3;
        fn write_symbols(&self, buff: &mut [u8]) -> usize {
            match self {
                Self::Add => {
                    buff[..3].copy_from_slice(b"add");
                    3
                }
                Self::Sub => {
                    buff[..3].copy_from_slice(b"sub");
                    3
                }
                Self::Neg => {
                    buff[..3].copy_from_slice(b"neg");
                    3
                }
                Self::And => {
                    buff[..3].copy_from_slice(b"and");
                    3
                }
                Self::Not => {
                    buff[..3].copy_from_slice(b"not");
                    3
                }
                Self::Eq => {
                    buff[..2].copy_from_slice(b"eq");
                    2
                }
                Self::Gt => {
                    buff[..2].copy_from_slice(b"gt");
                    2
                }
                Self::Lt => {
                    buff[..2].copy_from_slice(b"lt");
                    2
                }
                Self::Or => {
                    buff[..2].copy_from_slice(b"or");
                    2
                }
            }
        }
        fn read_symbols(buff: &'a [u8]) -> Option<(Self, usize)> {
            match buff {
                [b'a', b'd', b'd', ..] => Some((Self::Add, 3)),
                [b's', b'u', b'b', ..] => Some((Self::Sub, 3)),
                [b'n', b'e', b'g', ..] => Some((Self::Neg, 3)),
                [b'a', b'n', b'd', ..] => Some((Self::And, 3)),
                [b'n', b'o', b't', ..] => Some((Self::Not, 3)),
                [b'e', b'q', ..] => Some((Self::Eq, 2)),
                [b'g', b't', ..] => Some((Self::Gt, 2)),
                [b'l', b't', ..] => Some((Self::Lt, 2)),
                [b'o', b'r', ..] => Some((Self::Or, 2)),
                _ => None,
            }
        }
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
            f.write_fmt(format_args!("{0}", label))
        }
    }
    pub enum TokenPayload {
        Memory(MemoryToken),
        Arithmetic(ArithmeticToken),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TokenPayload {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                TokenPayload::Memory(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Memory",
                        &__self_0,
                    )
                }
                TokenPayload::Arithmetic(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Arithmetic",
                        &__self_0,
                    )
                }
            }
        }
    }
    pub struct Token {
        pub payload: TokenPayload,
        pub instruction: usize,
        pub src: usize,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Token {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "Token",
                "payload",
                &self.payload,
                "instruction",
                &self.instruction,
                "src",
                &&self.src,
            )
        }
    }
    impl fmt::Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.payload {
                TokenPayload::Memory(memory_token) => memory_token.fmt(f),
                TokenPayload::Arithmetic(arithmetic_token) => arithmetic_token.fmt(f),
            }
        }
    }
}
fn main() -> io::Result<()> {
    let body = async {
        let args: Vec<String> = env::args().collect();
        let file_path = Path::new(&args[1]);
        let mut lexer = lexer::VMLexer::new(&args[1]).await?;
        while let Some(i) = lexer.next_token().await {
            {
                ::std::io::_print(format_args!("{0}\n", i));
            };
        }
        if file_path.extension().expect("File extension undefined") != "vm" {
            return Err(Error::new(ErrorKind::Other, "File ext should be vm!"));
        }
        let src_file_name = file_path.file_stem().expect("Wrong stem");
        let f = File::open(&file_path).await?;
        let mut f_write = if args.len() > 2 {
            File::create(&args[2]).await?
        } else {
            File::create(file_path.with_extension("asm")).await?
        };
        let reader = BufReader::new(f);
        let mut lines = reader.lines();
        let file_str_name = src_file_name.to_str().unwrap();
        let mut counter = 0;
        while let Some(line) = lines.next_line().await? {
            let r = parser::assemble(&line, file_str_name, counter);
            f_write.write(r.as_bytes()).await?;
            counter += 1;
        }
        Ok(())
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
