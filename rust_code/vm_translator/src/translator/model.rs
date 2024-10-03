use std::{
    collections::HashMap,
    mem::{self, MaybeUninit},
};

use hack_instructions::*;

use file_context::FileContext;
use hack_macro::instruction;
use symbolic::SymbolicElem;
use vm_parser::{AsmFunctionInstruction, AsmInstructionPayload, FunctionMetadata};

use crate::context::WriteFileContext;

use super::{
    arithmetic::translate_arithmetic_token, branch::translate_branch_token,
    function::translate_function_token, memory::translate_memory_token,
};

const TRANSLATOR_INSTRUCTIONS_CAPACITY: usize = 2048;
const TRANSLATOR_TOKEN_CAPACITY: usize = 64;
const INIT_SP: [Instruction<'static>; 4] = [
    instruction!(b"@256"),
    instruction!(b"D=A"),
    instruction!(b"@SP"),
    instruction!(b"M=D"),
];

#[derive(Clone, Copy)]
pub struct TranslateOpts {
    comments: bool,
}

impl TranslateOpts {
    pub fn new() -> Self {
        Self { comments: true }
    }

    pub fn set_comments(&mut self, v: bool) -> &mut Self {
        self.comments = v;
        self
    }
}

#[derive(Debug)]
enum InstructionOrLink<'a> {
    I(Instruction<'a>),
    L(&'a [Instruction<'a>]),
}

pub struct Translator<'a> {
    instructions: [InstructionOrLink<'a>; TRANSLATOR_INSTRUCTIONS_CAPACITY],
    tokens: [FileContext<AsmInstructionPayload>; TRANSLATOR_TOKEN_CAPACITY],
    tokens_cursor_up: usize,
    tokens_cursor_down: usize,
    cursor: usize,
    cursor_link: usize,
    cursor_down: usize,
    translate_opts: TranslateOpts,
}

impl<'a> Translator<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::new_with_opts(TranslateOpts::new())
    }

    pub fn new_with_opts(opts: TranslateOpts) -> Self {
        let instructions = {
            let x: [MaybeUninit<InstructionOrLink<'_>>; TRANSLATOR_INSTRUCTIONS_CAPACITY] =
                unsafe { MaybeUninit::uninit().assume_init() };
            unsafe {
                mem::transmute::<_, [InstructionOrLink<'_>; TRANSLATOR_INSTRUCTIONS_CAPACITY]>(x)
            }
        };

        let tokens = {
            let x: [MaybeUninit<FileContext<AsmInstructionPayload>>; TRANSLATOR_TOKEN_CAPACITY] =
                unsafe { MaybeUninit::uninit().assume_init() };
            unsafe {
                mem::transmute::<_, [FileContext<AsmInstructionPayload>; TRANSLATOR_TOKEN_CAPACITY]>(
                    x,
                )
            }
        };

        Self {
            instructions,
            tokens,
            cursor: 0,
            tokens_cursor_up: 0,
            tokens_cursor_down: 0,
            cursor_link: 0,
            cursor_down: 0,
            translate_opts: opts,
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.cursor_down = 0;
    }

    fn next_instruction(&mut self) -> Option<&Instruction<'_>> {
        while self.cursor_down < self.cursor {
            let res = match &self.instructions[self.cursor_down] {
                InstructionOrLink::I(i) => {
                    self.cursor_down += 1;
                    i
                }
                InstructionOrLink::L(l) => {
                    let r = &l[self.cursor_link];
                    self.cursor_link += 1;

                    if self.cursor_link == l.len() {
                        self.cursor_down += 1;
                        self.cursor_link = 0;
                    }

                    r
                }
            };

            if self.instruction_is_needed(res) {
                return Some(res);
            }
        }

        None
    }

    pub fn instructions_to_symbols(&mut self, buff: &mut [u8], chunk: usize) -> usize {
        let mut res = 0;
        for _idx in 0..chunk {
            res += if let Some(i) = self.next_instruction() {
                i.write_symbols(&mut buff[res..])
            } else {
                break;
            };

            buff[res] = b'\n';
            res += 1;
        }
        res
    }

    pub fn instructions_to_bytes(
        &mut self,
        buff: &mut [u8],
        chunk: usize,
        static_pointer: &mut i16,
        static_map: &mut HashMap<Vec<u8>, String>,
        file_context: &mut WriteFileContext,
    ) -> usize {
        let mut res = 0;

        for _idx in 0..chunk {
            let n = file_context.global_instruction_number();
            res += if let Some(i) = self.next_instruction() {
                let (mut l, maybe_val_to_save) =
                    i.write_bytes(&mut buff[res..], static_pointer, n, static_map);

                if let Some(val_to_save) = maybe_val_to_save {
                    file_context.set_intruction(val_to_save)
                }

                if l != 0 {
                    file_context.incr();
                    buff[res + l] = b'\n';
                    l += 1;
                }

                l
            } else {
                break;
            };
        }

        res
    }

    pub fn translate<'b, 'c>(&'b mut self, factory: &'c mut VariableFactory<'a>) {
        while self.tokens_cursor_down != self.tokens_cursor_up {
            let raw_token = &mut self.tokens[self.tokens_cursor_down % TRANSLATOR_TOKEN_CAPACITY]
                as *mut FileContext<AsmInstructionPayload>;
            let token = unsafe { &mut *raw_token };
            self.tokens_cursor_down += 1;
            self.run_for_token(token, factory)
        }
    }

    fn run_for_token(
        &mut self,
        token: &mut FileContext<AsmInstructionPayload>,
        factory: &mut VariableFactory<'a>,
    ) {
        match &mut token.payload {
            AsmInstructionPayload::Function(function) => {
                translate_function_token(self, function, factory, token.idx)
            }
            AsmInstructionPayload::Branch(branch) => translate_branch_token(self, branch, factory),
            AsmInstructionPayload::Memory(memory) => translate_memory_token(self, memory, factory),
            AsmInstructionPayload::Arithmetic(arithmetic) => {
                translate_arithmetic_token(self, arithmetic, factory)
            }
        }
    }

    pub fn check_free_space(&self) -> usize {
        TRANSLATOR_TOKEN_CAPACITY + self.tokens_cursor_down - self.tokens_cursor_up
    }

    pub fn save_token(&mut self, token: FileContext<AsmInstructionPayload>) {
        self.tokens[self.tokens_cursor_up % TRANSLATOR_TOKEN_CAPACITY] = token;
        self.tokens_cursor_up += 1;
    }

    pub fn save_link(&mut self, l: &'a [Instruction<'a>]) -> bool {
        if self.cursor == TRANSLATOR_INSTRUCTIONS_CAPACITY {
            false
        } else {
            self.instructions[self.cursor] = InstructionOrLink::L(l);
            self.cursor += 1;
            true
        }
    }

    pub fn init_translator(&mut self, factory: &mut VariableFactory<'a>) {
        self.save_link(&INIT_SP);
        let init_function = b"Sys.init".to_vec();
        let payload =
            AsmInstructionPayload::Function(AsmFunctionInstruction::Call(FunctionMetadata {
                name: init_function,
                args_count: 0,
            }));
        let context = FileContext::new(payload, 0, None, None);
        self.save_token(context);
        self.translate(factory);
        self.save_instruction(instruction!(b"// Finish init"));
    }

    pub fn save_instruction(&mut self, i: Instruction<'a>) -> bool {
        if !self.instruction_is_needed(&i) {
            return true;
        }

        if self.cursor == TRANSLATOR_INSTRUCTIONS_CAPACITY {
            unreachable!();
        } else {
            self.instructions[self.cursor] = InstructionOrLink::I(i);
            self.cursor += 1;
            true
        }
    }

    fn instruction_is_needed(&self, i: &Instruction<'_>) -> bool {
        if self.translate_opts.comments {
            true
        } else {
            if let Instruction::Helper(HelperInstruction::Comment(_)) = i {
                false
            } else {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hack_macro::instruction;

    use std::fs::File;
    use std::io::prelude::*;

    use crate::translator::constants::PUSH_INSTRUCTIONS;
    use vm_parser::{
        AsmArithmeticInstruction, AsmMemoryInstruction, AsmMemoryInstructionKind,
        AsmMemoryInstructionSegment,
    };

    use super::*;

    #[test]
    fn save_instruction_test() {
        let mut t = Translator::new();
        assert!(t.save_instruction(instruction!(b"@SP")));
        let mut buff = [0; 100];
        let l = t.instructions_to_symbols(&mut buff, 100);
        assert!(buff[..l] == *b"@SP\n")
    }

    #[test]
    fn save_link_test() {
        let mut t = Translator::new();
        assert!(t.save_link(&PUSH_INSTRUCTIONS));
        let mut buff = [0; 100];
        let l = t.instructions_to_symbols(&mut buff, 100);
        let answer = b"// Write value to SP from D\n@SP\nA=M\nM=D\n// Incriment sp\n@SP\nM=M+1\n";
        assert!(l == answer.len());
        assert!(buff[..l] == *answer)
    }

    #[test]
    fn save_instruction_and_link_test() {
        let mut t = Translator::new();
        assert!(t.save_instruction(instruction!(b"@THIS")));
        assert!(t.save_link(&PUSH_INSTRUCTIONS));
        assert!(t.save_instruction(instruction!(b"M=D")));
        let mut buff = [0; 100];
        let l = t.instructions_to_symbols(&mut buff, 100);
        let answer = b"@THIS\n// Write value to SP from D\n@SP\nA=M\nM=D\n// Incriment sp\n@SP\nM=M+1\nM=D\n";
        assert!(l == answer.len());
        assert!(buff[..l] == *answer)
    }

    #[test]
    fn translate_memory_test() {
        let mut file = File::open("./priv/memory/arg_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_asm_instruction(AsmInstructionPayload::Memory(AsmMemoryInstruction {
            kind: AsmMemoryInstructionKind::Pop,
            segment: AsmMemoryInstructionSegment::Arg,
            val: 0,
        }));

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_cmp_test() {
        let mut file = File::open("./priv/arithmetic/cmp_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_asm_instruction(AsmInstructionPayload::Arithmetic(
            AsmArithmeticInstruction::Eq,
        ));

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_add_test() {
        let mut file = File::open("./priv/arithmetic/add_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_asm_instruction(AsmInstructionPayload::Arithmetic(
            AsmArithmeticInstruction::Add,
        ));

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_sub_test() {
        let mut file = File::open("./priv/arithmetic/sub_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_asm_instruction(AsmInstructionPayload::Arithmetic(
            AsmArithmeticInstruction::Sub,
        ));

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_not_test() {
        let mut file = File::open("./priv/arithmetic/not_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = new_asm_instruction(AsmInstructionPayload::Arithmetic(
            AsmArithmeticInstruction::Not,
        ));

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff, 100);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    fn new_asm_instruction(payload: AsmInstructionPayload) -> FileContext<AsmInstructionPayload> {
        FileContext::new(payload, 0, None, None)
    }
}
