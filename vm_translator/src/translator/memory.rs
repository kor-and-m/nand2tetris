use hack_ast::*;
use hack_macro::instruction;

use crate::tokens::{MemoryToken, MemoryTokenKind, Segment};

use super::constants::{POP_INSTRUCTIONS, PUSH_INSTRUCTIONS};
use super::model::Translator;

const POP_TERMINATOR: Instruction<'static> = instruction!(b"M=D");
const PUSH_TERMINATOR: Instruction<'static> = instruction!(b"D=M");
const TMP_REGISTER: Instruction<'static> = instruction!(b"@R5");

const SAVE_D_TO_TMP: [Instruction<'static>; 2] = [TMP_REGISTER, instruction!(b"M=D")];

const RESTORE_FROM_TMP_TO_A: [Instruction<'static>; 2] = [TMP_REGISTER, instruction!(b"A=M")];

pub fn translate_memory_token<'links, 'structs>(
    translator: &'links mut Translator<'structs>,
    token: &'links MemoryToken,
    factory: &'links mut VariableFactory<'structs>,
) {
    if token.kind == MemoryTokenKind::Pop && token.segment == Segment::Const {
        panic!("Pop const commands are restricted");
    }

    if token.segment == Segment::Const {
        // TODO check if successfully saved
        translator.save_instruction(instruction!(b"// Set const value to D"));
        translator.save_instruction(Instruction::new_number(token.val));
        translator.save_instruction(instruction!(b"D=A"));
        translator.save_link(&PUSH_INSTRUCTIONS);
    } else {
        prepare_a_reading(translator, token);

        if token.kind == MemoryTokenKind::Pop {
            translator.save_link(&POP_INSTRUCTIONS);
        }

        match token.segment {
            Segment::Const => unreachable!(),
            Segment::Temp => translator.save_instruction(Instruction::new_number(token.val + 5)),
            Segment::Static => {
                translator.save_instruction(factory.new_variable_with_idx(token.val))
            }
            Segment::Pointer => match token.val {
                0 => translator.save_instruction(instruction!(b"@THIS")),
                1 => translator.save_instruction(instruction!(b"@THAT")),
                _ => unreachable!(),
            },
            s => {
                if should_be_prepared(token) {
                    translator.save_link(&RESTORE_FROM_TMP_TO_A)
                } else {
                    translator.save_instruction(s.as_instruction());
                    match token.val {
                        0 => translator.save_instruction(instruction!(b"A=M")),
                        1 => {
                            translator.save_instruction(instruction!(b"A=M"));
                            translator.save_instruction(instruction!(b"A=A+1"))
                        }
                        v => match token.kind {
                            MemoryTokenKind::Pop => {
                                let mut t = translator.save_instruction(instruction!(b"A=M"));
                                for _i in 0..v {
                                    t = translator.save_instruction(instruction!(b"A=A+1"));
                                }
                                t
                            }
                            MemoryTokenKind::Push => {
                                translator.save_instruction(instruction!(b"D=M"));
                                translator.save_instruction(Instruction::new_number(v));
                                translator.save_instruction(instruction!(b"A=D+A"))
                            }
                        },
                    }
                }
            }
        };

        let terminator = if token.kind == MemoryTokenKind::Push {
            PUSH_TERMINATOR
        } else {
            POP_TERMINATOR
        };

        translator.save_instruction(terminator);

        if token.kind == MemoryTokenKind::Push {
            translator.save_link(&PUSH_INSTRUCTIONS);
        }
    };
}

fn prepare_a_reading(translator: &mut Translator, token: &MemoryToken) {
    if should_be_prepared(token) {
        translator.save_instruction(instruction!(b"// Start Prepare"));
        translator.save_instruction(token.segment.as_instruction());
        translator.save_instruction(instruction!(b"D=M"));
        translator.save_instruction(Instruction::new_number(token.val));
        translator.save_instruction(instruction!(b"D=D+A"));
        translator.save_link(&SAVE_D_TO_TMP);
        translator.save_instruction(instruction!(b"// End Prepare"));
    }
}

fn should_be_prepared(token: &MemoryToken) -> bool {
    match token.segment {
        Segment::Const => false,
        Segment::Temp => false,
        Segment::Static => false,
        Segment::Pointer => false,
        _ => token.kind == MemoryTokenKind::Pop && token.val > 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::prelude::*;

    use crate::tokens::{MemoryToken, MemoryTokenKind, Segment};

    #[test]
    fn translate_const_44_push_answer_test() {
        let mut file = File::open("./priv/memory/const_44_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");
        let token = new_memory_token(MemoryTokenKind::Push, Segment::Const, 44);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);

        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_pointer_1_pop_test() {
        let mut file = File::open("./priv/memory/pointer_1_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Pointer, 1);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_pointer_0_push_test() {
        let mut file = File::open("./priv/memory/pointer_0_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Pointer, 0);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_static_44_push_test() {
        let mut file = File::open("./priv/memory/static_44_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Static, 44);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_static_20_pop_test() {
        let mut file = File::open("./priv/memory/static_20_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Static, 20);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_tmp_2_push_test() {
        let mut file = File::open("./priv/memory/tmp_2_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Temp, 2);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_tmp_0_pop_test() {
        let mut file = File::open("./priv/memory/tmp_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Temp, 0);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_0_pop_test() {
        let mut file = File::open("./priv/memory/arg_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Arg, 0);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_0_push_test() {
        let mut file = File::open("./priv/memory/arg_0_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Arg, 0);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_1_pop_test() {
        let mut file = File::open("./priv/memory/arg_1_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Arg, 1);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_1_push_test() {
        let mut file = File::open("./priv/memory/arg_1_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Arg, 1);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_7_pop_test() {
        let mut file = File::open("./priv/memory/arg_7_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Arg, 7);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_7_push_test() {
        let mut file = File::open("./priv/memory/arg_7_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::Arg, 7);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arg_3_pop_test() {
        let mut file = File::open("./priv/memory/arg_3_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Arg, 3);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_that_3_pop_test() {
        let mut file = File::open("./priv/memory/that_3_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::That, 3);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_this_5_push_test() {
        let mut file = File::open("./priv/memory/this_5_push_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Push, Segment::This, 5);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_local_10_pop_test() {
        let mut file = File::open("./priv/memory/local_10_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_memory_token(MemoryTokenKind::Pop, Segment::Local, 10);

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        translate_memory_token(&mut t, &token, &mut factory);
        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    fn new_memory_token(kind: MemoryTokenKind, segment: Segment, val: i16) -> MemoryToken {
        MemoryToken { kind, segment, val }
    }
}
