use symbolic::SymbolicElem;

mod a_instructions;
mod c_instructions;
mod helper_instructions;

pub use a_instructions::*;
pub use c_instructions::*;
pub use helper_instructions::*;

#[macro_export]
macro_rules! write_instruction_set_symbols {
    ( $buff:expr, $( $x:expr ),* ) => {
        {
            let mut cursor = 0;
            $(
                for i in $x.iter() {
                    cursor += i.write_symbols(&mut $buff[cursor..]);
                    $buff[cursor] = b'\n';
                    cursor += 1;
                }
            )*
            cursor
        }
    };
}

#[macro_export]
macro_rules! write_instruction_set_bin {
    ( $buff:expr, $variable_pointer:expr, $( $x:expr ),* ) => {
        {
            let mut cursor = 0;
            let mut variable_pointer = $variable_pointer;
            $(
                for i in $x.iter() {
                    let cursor_incr = i.write_bytes(&mut $buff[cursor..], &mut variable_pointer);
                    cursor += cursor_incr;

                    if cursor_incr > 0 {
                        $buff[cursor] = b'\n';
                        cursor += 1;
                    }
                }
            )*
            cursor
        }
    };
}

#[derive(Debug)]
pub enum Instruction<'a> {
    A(AInstruction<'a>),
    C(CInstruction),
    Helper(HelperInstruction<'a>),
}

impl Instruction<'_> {
    pub fn new_number(v: i16) -> Self {
        Instruction::A(AInstruction::Number(v))
    }

    pub fn new_raw_label<'b>(b: Vec<u8>) -> Instruction<'b> {
        Instruction::Helper(HelperInstruction::RawLabel(b))
    }

    pub fn new_raw_var_label<'b>(b: Vec<u8>) -> Instruction<'b> {
        Instruction::Helper(HelperInstruction::RawVarLabel(b))
    }

    pub fn write_bytes(&self, buff: &mut [u8], variable_pointer: &mut i16) -> usize {
        match self {
            Self::A(a) => {
                a.write_bytes(buff, variable_pointer);
                16
            }
            Self::C(c) => {
                c.write_bytes(buff);
                16
            }
            Self::Helper(_) => 0,
        }
    }
}

pub struct VariableFactory<'a> {
    idx: i16,
    prefix: &'a [u8],
    l: usize,
}

impl<'a> VariableFactory<'a> {
    pub fn new<'b>(prefix: &'b [u8]) -> VariableFactory<'b> {
        VariableFactory {
            prefix,
            idx: 0,
            l: prefix.len(),
        }
    }

    pub fn new_bool_variables(
        &mut self,
    ) -> (
        Instruction<'a>,
        Instruction<'a>,
        Instruction<'a>,
        Instruction<'a>,
    ) {
        self.idx += 1;
        (
            Instruction::Helper(HelperInstruction::Label(LabelInstruction {
                prefix: self.prefix,
                prefix_len: self.l,
                name: b"TRUE",
                name_len: 4,
                idx: self.idx - 1,
            })),
            Instruction::Helper(HelperInstruction::LabelVariable(LabelInstruction {
                prefix: self.prefix,
                prefix_len: self.l,
                name: b"TRUE",
                name_len: 4,
                idx: self.idx - 1,
            })),
            Instruction::Helper(HelperInstruction::Label(LabelInstruction {
                prefix: self.prefix,
                prefix_len: self.l,
                name: b"FALSE",
                name_len: 5,
                idx: self.idx - 1,
            })),
            Instruction::Helper(HelperInstruction::LabelVariable(LabelInstruction {
                prefix: self.prefix,
                prefix_len: self.l,
                name: b"FALSE",
                name_len: 5,
                idx: self.idx - 1,
            })),
        )
    }

    pub fn new_variable_with_idx(&self, idx: i16) -> Instruction<'a> {
        Instruction::A(AInstruction::Variable((self.prefix, self.l, idx)))
    }

    pub fn new_variable(&mut self) -> Instruction<'_> {
        let i = Instruction::A(AInstruction::Variable((self.prefix, self.l, self.idx)));
        self.idx += 1;
        i
    }
}

impl<'a> SymbolicElem<'a> for Instruction<'a> {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        match self {
            Instruction::A(ainstruction) => ainstruction.write_symbols(buff),
            Instruction::C(cinstruction) => cinstruction.write_symbols(buff),
            Instruction::Helper(helper_instruction) => helper_instruction.write_symbols(buff),
        }
    }
}

#[cfg(test)]
mod tests {
    use hack_macro::instruction;

    use super::*;

    const PUSH_INSTRUCTION_SET: [Instruction<'static>; 6] = [
        instruction!(b"// push value from D"),
        instruction!(b"@SP"),
        instruction!(b"A=M"),
        instruction!(b"M=D"),
        instruction!(b"@SP"),
        instruction!(b"M=M+1"),
    ];

    #[test]
    fn const_instruction1_format() {
        let instruction = instruction!(b"@SP");
        let mut buff = *b"XXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@SPX");
        assert_eq!(buff[..l], *b"@SP");
    }

    #[test]
    fn const_instruction2_format() {
        let instruction = instruction!(b"@THIS");
        let mut buff = *b"XXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@THISXX");
        assert_eq!(buff[..l], *b"@THIS");
    }

    #[test]
    fn number_instruction1_format() {
        let instruction = instruction!(b"@5");
        let mut buff = *b"XXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@5XXXXX");
        assert_eq!(buff[..l], *b"@5");
    }

    #[test]
    fn number_instruction2_format() {
        let instruction = instruction!(b"@133");
        let mut buff = *b"XXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@133XXX");
        assert_eq!(buff[..l], *b"@133");
    }

    #[test]
    fn variable_instruction1_format() {
        let mut factory = VariableFactory::new(b"ExampleFile");
        let instruction = factory.new_variable();
        let mut buff = *b"XXXXXXXXXXXXXXXXXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@ExampleFile.0XXXXXXXX");
        assert_eq!(buff[..l], *b"@ExampleFile.0");
    }

    #[test]
    fn variable_instruction2_format() {
        let mut factory = VariableFactory::new(b"Another");
        for _i in 0..11 {
            factory.new_variable();
        }

        let instruction = factory.new_variable();
        let mut buff = *b"XXXXXXXXXXXXXXXXXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff, *b"@Another.11XXXXXXXXXXX");
        assert_eq!(buff[..l], *b"@Another.11");
    }

    #[test]
    fn c_instruction1_format() {
        let instruction = instruction!(b"D=D+A");
        let mut buff = *b"XXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"D=D+A");
    }

    #[test]
    fn c_instruction2_format() {
        let instruction = instruction!(b"0;JMP");
        let mut buff = *b"XXXXXXX";
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"0;JMP");
    }

    #[test]
    fn helper_instruction1_format() {
        let instruction = instruction!(b"(MYFILE_TRUE_111)");
        let mut buff = [0u8; 100];
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"(MYFILE_TRUE_111)");
    }

    #[test]
    fn helper_instruction2_format() {
        let instruction = instruction!(b"// Test comment");
        let mut buff = [0u8; 100];
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"// Test comment");
    }

    #[test]
    fn helper_instruction3_format() {
        let instruction = instruction!(b"/@MYFILE_TRUE_111");
        let mut buff = [0u8; 100];
        let l = instruction.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"@MYFILE_TRUE_111");
    }

    #[test]
    fn helper_instruction4_format() {
        let mut factory = VariableFactory::new(b"ExampleFile");
        let (instruction1, instruction2, instruction3, instruction4) = factory.new_bool_variables();
        let mut buff = [0u8; 100];
        let mut l = instruction1.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"(ExampleFile_TRUE_0)");
        l = instruction2.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"@ExampleFile_TRUE_0");

        l = instruction3.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"(ExampleFile_FALSE_0)");
        l = instruction4.write_symbols(&mut buff);
        assert_eq!(buff[..l], *b"@ExampleFile_FALSE_0");
    }

    #[test]
    fn write_instruction_set_symbols1_test() {
        let mut buff = [0u8; 100];

        let l = write_instruction_set_symbols!(&mut buff, &PUSH_INSTRUCTION_SET);
        assert_eq!(
            buff[..l],
            *b"// push value from D\n@SP\nA=M\nM=D\n@SP\nM=M+1\n"
        );
    }

    #[test]
    fn write_instruction_set_symbols2_test() {
        let mut buff = [0u8; 100];

        let mut factory = VariableFactory::new(b"Another");
        let mut p = Vec::new();
        let i = instruction!(b"@11");
        p.push(i);

        let l = write_instruction_set_symbols!(
            &mut buff,
            &p,
            &PUSH_INSTRUCTION_SET,
            &[instruction!(b"@5"), factory.new_variable()]
        );

        assert_eq!(
            buff[..l],
            *b"@11\n// push value from D\n@SP\nA=M\nM=D\n@SP\nM=M+1\n@5\n@Another.0\n"
        );
    }

    #[test]
    fn sp_instruction_to_bin() {
        let sp = instruction!(b"@SP");
        let mut buff = [0u8; 500];
        sp.write_bytes(&mut buff, &mut 400);
        assert_eq!(buff[..16], *b"0000000000000000")
    }

    #[test]
    fn c_instruction_to_bin() {
        let mut buff = [0u8; 500];

        let a_eq_m = instruction!(b"A=M");
        a_eq_m.write_bytes(&mut buff, &mut 400);
        assert_eq!(buff[..16], *b"1111001110000000");

        let m_eq_d = instruction!(b"M=D");
        m_eq_d.write_bytes(&mut buff, &mut 400);
        assert_eq!(buff[..16], *b"1110010001100000");

        let incr_m = instruction!(b"M=M+1");
        incr_m.write_bytes(&mut buff, &mut 400);
        assert_eq!(buff[..16], *b"1110011110111000");
    }

    #[test]
    fn write_instruction_set_bin1_test() {
        let mut buff = [0u8; 500];

        let l = write_instruction_set_bin!(&mut buff, 100, &PUSH_INSTRUCTION_SET);
        assert_eq!(
            buff[..l],
            *b"0000000000000000\n1111001110000000\n1110010001100000\n0000000000000000\n1110011110111000\n"
        );
    }

    #[test]
    fn write_instruction_set_bin2_test() {
        let mut buff = [0u8; 500];

        let mut factory = VariableFactory::new(b"Another");

        let l = write_instruction_set_bin!(
            &mut buff,
            100,
            &[instruction!(b"@11")],
            &PUSH_INSTRUCTION_SET,
            &[instruction!(b"@5"), factory.new_variable()]
        );

        assert_eq!(
            buff[..l],
            *b"0000000000001011\n0000000000000000\n1111001110000000\n1110010001100000\n0000000000000000\n1110011110111000\n0000000000000101\n0000000001100100\n"
        );
    }
}
