use hack_instructions::*;
use hack_macro::instruction;

use vm_parser::{AsmMemoryInstruction, AsmMemoryInstructionKind, AsmMemoryInstructionSegment};

use super::constants::{POP_INSTRUCTIONS, PUSH_INSTRUCTIONS};
use super::model::Translator;

const POP_TERMINATOR: Instruction<'static> = instruction!(b"M=D");
const PUSH_TERMINATOR: Instruction<'static> = instruction!(b"D=M");
const TMP_REGISTER: Instruction<'static> = instruction!(b"@R5");

const SAVE_D_TO_TMP: [Instruction<'static>; 2] = [TMP_REGISTER, instruction!(b"M=D")];

const RESTORE_FROM_TMP_TO_A: [Instruction<'static>; 2] = [TMP_REGISTER, instruction!(b"A=M")];

pub fn translate_memory_token<'links, 'structs>(
    translator: &'links mut Translator<'structs>,
    token: &'links AsmMemoryInstruction,
    factory: &'links mut VariableFactory<'structs>,
) {
    if token.kind == AsmMemoryInstructionKind::Pop
        && token.segment == AsmMemoryInstructionSegment::Const
    {
        panic!("Pop const commands are restricted");
    }

    if token.segment == AsmMemoryInstructionSegment::Const {
        // TODO check if successfully saved
        translator.save_instruction(instruction!(b"// Set const value to D"));
        translator.save_instruction(Instruction::new_number(token.val));
        translator.save_instruction(instruction!(b"D=A"));
        translator.save_link(&PUSH_INSTRUCTIONS);
    } else {
        prepare_a_reading(translator, token);

        if token.kind == AsmMemoryInstructionKind::Pop {
            translator.save_link(&POP_INSTRUCTIONS);
        }

        match token.segment {
            AsmMemoryInstructionSegment::Const => unreachable!(),
            AsmMemoryInstructionSegment::Temp => {
                translator.save_instruction(Instruction::new_number(token.val + 5))
            }
            AsmMemoryInstructionSegment::Static => {
                translator.save_instruction(factory.new_variable_with_idx(token.val))
            }
            AsmMemoryInstructionSegment::Pointer => match token.val {
                0 => translator.save_instruction(instruction!(b"@THIS")),
                1 => translator.save_instruction(instruction!(b"@THAT")),
                _ => unreachable!(),
            },
            s => {
                if should_be_prepared(token) {
                    translator.save_link(&RESTORE_FROM_TMP_TO_A)
                } else {
                    translator.save_instruction(segment_as_instruction(s));
                    match token.val {
                        0 => translator.save_instruction(instruction!(b"A=M")),
                        1 => {
                            translator.save_instruction(instruction!(b"A=M"));
                            translator.save_instruction(instruction!(b"A=A+1"))
                        }
                        v => match token.kind {
                            AsmMemoryInstructionKind::Pop => {
                                let mut t = translator.save_instruction(instruction!(b"A=M"));
                                for _i in 0..v {
                                    t = translator.save_instruction(instruction!(b"A=A+1"));
                                }
                                t
                            }
                            AsmMemoryInstructionKind::Push => {
                                translator.save_instruction(instruction!(b"D=M"));
                                translator.save_instruction(Instruction::new_number(v));
                                translator.save_instruction(instruction!(b"A=D+A"))
                            }
                        },
                    }
                }
            }
        };

        let terminator = if token.kind == AsmMemoryInstructionKind::Push {
            PUSH_TERMINATOR
        } else {
            POP_TERMINATOR
        };

        translator.save_instruction(terminator);

        if token.kind == AsmMemoryInstructionKind::Push {
            translator.save_link(&PUSH_INSTRUCTIONS);
        }
    };
}

fn prepare_a_reading(translator: &mut Translator, token: &AsmMemoryInstruction) {
    if should_be_prepared(token) {
        translator.save_instruction(instruction!(b"// Start Prepare"));
        translator.save_instruction(segment_as_instruction(token.segment));
        translator.save_instruction(instruction!(b"D=M"));
        translator.save_instruction(Instruction::new_number(token.val));
        translator.save_instruction(instruction!(b"D=D+A"));
        translator.save_link(&SAVE_D_TO_TMP);
        translator.save_instruction(instruction!(b"// End Prepare"));
    }
}

fn should_be_prepared(token: &AsmMemoryInstruction) -> bool {
    match token.segment {
        AsmMemoryInstructionSegment::Const => false,
        AsmMemoryInstructionSegment::Temp => false,
        AsmMemoryInstructionSegment::Static => false,
        AsmMemoryInstructionSegment::Pointer => false,
        _ => token.kind == AsmMemoryInstructionKind::Pop && token.val > 6,
    }
}

pub fn segment_as_instruction(segment: AsmMemoryInstructionSegment) -> Instruction<'static> {
    match segment {
        AsmMemoryInstructionSegment::Arg => instruction!(b"@ARG"),
        AsmMemoryInstructionSegment::Local => instruction!(b"@LCL"),
        AsmMemoryInstructionSegment::This => instruction!(b"@THIS"),
        AsmMemoryInstructionSegment::That => instruction!(b"@THAT"),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::prelude::*;

    use vm_parser::{AsmMemoryInstruction, AsmMemoryInstructionKind, AsmMemoryInstructionSegment};

    #[test]
    fn translate_const_44_push_answer_test() {
        let mut file = File::open("./priv/memory/const_44_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");
        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Const,
            44,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);

        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_pointer_1_pop_test() {
        let mut file = File::open("./priv/memory/pointer_1_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Pointer,
            1,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_pointer_0_push_test() {
        let mut file = File::open("./priv/memory/pointer_0_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Pointer,
            0,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_static_44_push_test() {
        let mut file = File::open("./priv/memory/static_44_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Static,
            44,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_static_20_pop_test() {
        let mut file = File::open("./priv/memory/static_20_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Static,
            20,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_tmp_2_push_test() {
        let mut file = File::open("./priv/memory/tmp_2_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Temp,
            2,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_tmp_0_pop_test() {
        let mut file = File::open("./priv/memory/tmp_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Temp,
            0,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_0_pop_test() {
        let mut file = File::open("./priv/memory/arg_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Arg,
            0,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_0_push_test() {
        let mut file = File::open("./priv/memory/arg_0_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Arg,
            0,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_1_pop_test() {
        let mut file = File::open("./priv/memory/arg_1_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Arg,
            1,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_1_push_test() {
        let mut file = File::open("./priv/memory/arg_1_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Arg,
            1,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_7_pop_test() {
        let mut file = File::open("./priv/memory/arg_7_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Arg,
            7,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_7_push_test() {
        let mut file = File::open("./priv/memory/arg_7_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::Arg,
            7,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_3_pop_test() {
        let mut file = File::open("./priv/memory/arg_3_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Arg,
            3,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_that_3_pop_test() {
        let mut file = File::open("./priv/memory/that_3_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::That,
            3,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_this_5_push_test() {
        let mut file = File::open("./priv/memory/this_5_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Push,
            AsmMemoryInstructionSegment::This,
            5,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_local_10_pop_test() {
        let mut file = File::open("./priv/memory/local_10_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(
            AsmMemoryInstructionKind::Pop,
            AsmMemoryInstructionSegment::Local,
            10,
        );

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    fn new_memory_token(
        kind: AsmMemoryInstructionKind,
        segment: AsmMemoryInstructionSegment,
        val: i16,
    ) -> AsmMemoryInstruction {
        AsmMemoryInstruction { kind, segment, val }
    }
}
