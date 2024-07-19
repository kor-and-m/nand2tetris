use hack_macro::{BinaryInstruction, SymbolicElem};
use symbolic::SymbolicElem;

#[derive(SymbolicElem, BinaryInstruction, Debug)]
pub enum CInstructionExpression {
    #[hack(binary = b"0101010")]
    #[hack(symbol = b"0")]
    Zero,
    #[hack(binary = b"0111111")]
    #[hack(symbol = b"1")]
    One,
    #[hack(binary = b"0111010")]
    #[hack(symbol = b"-1")]
    MinusOne,
    #[hack(binary = b"0001100")]
    #[hack(symbol = b"D")]
    D,
    #[hack(binary = b"0110000")]
    #[hack(symbol = b"A")]
    A,
    #[hack(binary = b"1110000")]
    #[hack(symbol = b"M")]
    M,
    #[hack(binary = b"0001101")]
    #[hack(symbol = b"!D")]
    NotD,
    #[hack(binary = b"0110001")]
    #[hack(symbol = b"!A")]
    NotA,
    #[hack(binary = b"1110001")]
    #[hack(symbol = b"!M")]
    NotM,
    #[hack(binary = b"0001111")]
    #[hack(symbol = b"-D")]
    MinusD,
    #[hack(binary = b"0110011")]
    #[hack(symbol = b"-A")]
    MinusA,
    #[hack(binary = b"1110011")]
    #[hack(symbol = b"-M")]
    MinusM,
    #[hack(binary = b"0011111")]
    #[hack(symbol = b"D+1")]
    IncrementD,
    #[hack(binary = b"0110111")]
    #[hack(symbol = b"A+1")]
    IncrementA,
    #[hack(binary = b"1110111")]
    #[hack(symbol = b"M+1")]
    IncrementM,
    #[hack(binary = b"0110010")]
    #[hack(symbol = b"A-1")]
    DecrementA,
    #[hack(binary = b"1110010")]
    #[hack(symbol = b"M-1")]
    DecrementM,
    #[hack(binary = b"0001110")]
    #[hack(symbol = b"D-1")]
    DecrementD,
    #[hack(binary = b"0000010")]
    #[hack(symbol = b"D+A")]
    DPlusA,
    #[hack(binary = b"1000010")]
    #[hack(symbol = b"D+M")]
    DPlusM,
    #[hack(binary = b"0010011")]
    #[hack(symbol = b"D-A")]
    DMinusA,
    #[hack(binary = b"1010011")]
    #[hack(symbol = b"D-M")]
    DMinusM,
    #[hack(binary = b"0000111")]
    #[hack(symbol = b"A-D")]
    AMinusD,
    #[hack(binary = b"1000111")]
    #[hack(symbol = b"M-D")]
    MMinusD,
    #[hack(binary = b"0000000")]
    #[hack(symbol = b"D&A")]
    DAndA,
    #[hack(binary = b"1000000")]
    #[hack(symbol = b"D&M")]
    DAndM,
    #[hack(binary = b"0010101")]
    #[hack(symbol = b"D|A")]
    DOrA,
    #[hack(binary = b"1010101")]
    #[hack(symbol = b"D|M")]
    DOrM,
}

#[derive(SymbolicElem, BinaryInstruction, Debug)]
#[hack(suffix = b"=")]
pub enum CInstructionDest {
    #[hack(binary = b"111")]
    ADM,
    #[hack(binary = b"110")]
    AD,
    #[hack(binary = b"101")]
    AM,
    #[hack(binary = b"100")]
    A,
    #[hack(binary = b"011")]
    DM,
    #[hack(binary = b"010")]
    D,
    #[hack(binary = b"001")]
    M,
    #[hack(binary = b"000")]
    #[hack(symbol = b"")]
    NODEST,
}

#[derive(SymbolicElem, BinaryInstruction, Debug)]
#[hack(prefix = b";")]
pub enum CInstructionJump {
    #[hack(binary = b"001")]
    JGT,
    #[hack(binary = b"010")]
    JEQ,
    #[hack(binary = b"011")]
    JGE,
    #[hack(binary = b"100")]
    JLT,
    #[hack(binary = b"101")]
    JNE,
    #[hack(binary = b"110")]
    JLE,
    #[hack(binary = b"111")]
    JMP,
    #[hack(binary = b"000")]
    #[hack(symbol = b"")]
    NOJMP,
}

#[derive(Debug)]
pub struct CInstruction {
    pub dest: CInstructionDest,
    pub expression: CInstructionExpression,
    pub jump: CInstructionJump,
}

impl<'a> SymbolicElem<'a> for CInstruction {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        let mut result = self.dest.write_symbols(buff);
        result += self.expression.write_symbols(&mut buff[result..]);
        result + self.jump.write_symbols(&mut buff[result..])
    }
}

impl CInstruction {
    pub fn write_bytes(&self, buff: &mut [u8]) {
        buff[..3].copy_from_slice(b"111");
        buff[3..6].copy_from_slice(self.dest.as_bytes_const());
        buff[6..13].copy_from_slice(self.expression.as_bytes_const());
        buff[13..16].copy_from_slice(self.jump.as_bytes_const());
    }
}
