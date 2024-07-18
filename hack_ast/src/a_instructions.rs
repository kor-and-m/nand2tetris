use hack_macro::{BinaryInstruction, SymbolicElem};
use symbolic::SymbolicElem;
pub enum AInstruction<'a> {
    Number(i16),
    Variable((&'a [u8], usize, i16)),
    Const(AConst),
}

impl AInstruction<'_> {
    pub fn write_bytes(&self, buff: &mut [u8], variable_pointer: &mut i16) {
        match self {
            Self::Const(c) => buff[..16].copy_from_slice(c.as_bytes_const()),
            Self::Number(n) => buff[..16].copy_from_slice(format!("{:016b}", n).as_bytes()),
            Self::Variable(_) => {
                buff[..16].copy_from_slice(format!("{:016b}", variable_pointer).as_bytes());
                *variable_pointer += 1;
            }
        }
    }
}

impl<'a> SymbolicElem<'a> for AInstruction<'a> {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        buff[0] = b'@';
        match self {
            Self::Const(const_instruction) => const_instruction.write_symbols(&mut buff[1..]) + 1,
            Self::Variable((prefix, l, n)) => {
                buff[1..(l + 1)].copy_from_slice(prefix);
                buff[l + 1] = b'.';
                write_i16_to_buff(*n, &mut buff[(l + 2)..]) + l + 2
            }
            Self::Number(n) => write_i16_to_buff(*n, &mut buff[1..]) + 1,
        }
    }
}

#[derive(SymbolicElem, BinaryInstruction)]
pub enum AConst {
    #[hack(int = b"0")]
    SP,
    #[hack(int = b"1")]
    LCL,
    #[hack(int = b"2")]
    ARG,
    #[hack(int = b"3")]
    THIS,
    #[hack(int = b"4")]
    THAT,
    #[hack(int = b"0")]
    R0,
    #[hack(int = b"1")]
    R1,
    #[hack(int = b"2")]
    R2,
    #[hack(int = b"3")]
    R3,
    #[hack(int = b"4")]
    R4,
    #[hack(int = b"5")]
    R5,
    #[hack(int = b"6")]
    R6,
    #[hack(int = b"7")]
    R7,
    #[hack(int = b"8")]
    R8,
    #[hack(int = b"9")]
    R9,
    #[hack(int = b"10")]
    R10,
    #[hack(int = b"11")]
    R11,
    #[hack(int = b"12")]
    R12,
    #[hack(int = b"13")]
    R13,
    #[hack(int = b"14")]
    R14,
    #[hack(int = b"15")]
    R15,
    #[hack(int = b"16384")]
    SCREEN,
    #[hack(int = b"24576")]
    KBD,
}

fn write_i16_to_buff(n: i16, buff: &mut [u8]) -> usize {
    let mut idx = 0;
    for i in n.to_string().chars() {
        buff[idx] = i as u8;
        idx += 1;
    }
    idx
}
