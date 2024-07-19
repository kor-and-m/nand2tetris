use std::mem::{self, MaybeUninit};

use hack_ast::*;

use crate::tokens::{Token, TokenPayload};
use symbolic::SymbolicElem;

use super::{arithmetic::translate_arithmetic_token, memory::translate_memory_token};

enum InstructionOrLink<'a> {
    I(Instruction<'a>),
    L(&'a [Instruction<'a>]),
}

const TRANSLATOR_INSTRUCTIONS_CAPACITY: usize = 512;
const TRANSLATOR_TOKEN_CAPACITY: usize = 32;

pub struct Translator<'a> {
    instructions: [InstructionOrLink<'a>; TRANSLATOR_INSTRUCTIONS_CAPACITY],
    tokens: [Token; TRANSLATOR_TOKEN_CAPACITY],
    tokens_cursor_up: usize,
    tokens_cursor_down: usize,
    cursor: usize,
}

impl<'a> Translator<'a> {
    pub fn new() -> Self {
        let instructions = {
            let x: [MaybeUninit<InstructionOrLink<'_>>; TRANSLATOR_INSTRUCTIONS_CAPACITY] =
                unsafe { MaybeUninit::uninit().assume_init() };
            unsafe {
                mem::transmute::<_, [InstructionOrLink<'_>; TRANSLATOR_INSTRUCTIONS_CAPACITY]>(x)
            }
        };

        let tokens = {
            let x: [MaybeUninit<Token>; TRANSLATOR_TOKEN_CAPACITY] =
                unsafe { MaybeUninit::uninit().assume_init() };
            unsafe { mem::transmute::<_, [Token; TRANSLATOR_TOKEN_CAPACITY]>(x) }
        };

        Self {
            instructions,
            tokens,
            cursor: 0,
            tokens_cursor_up: 0,
            tokens_cursor_down: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn instructions_to_symbols(&self, buff: &mut [u8]) -> usize {
        let mut cursor = 0;
        for il in self.instructions[..self.cursor].iter() {
            cursor += match il {
                InstructionOrLink::I(i) => i.write_symbols(&mut buff[cursor..]),
                InstructionOrLink::L(l) => {
                    let mut cursor2 = 0;
                    for i2 in l.iter() {
                        cursor2 += i2.write_symbols(&mut buff[(cursor + cursor2)..]);
                        buff[cursor + cursor2] = b'\n';
                        cursor2 += 1;
                    }
                    cursor2 - 1
                }
            };
            buff[cursor] = b'\n';
            cursor += 1;
        }
        cursor
    }

    pub fn translate<'b, 'c>(&'b mut self, factory: &'c mut VariableFactory<'a>) {
        while self.tokens_cursor_down % TRANSLATOR_TOKEN_CAPACITY != self.tokens_cursor_up % TRANSLATOR_TOKEN_CAPACITY {
            let raw_token =
            &self.tokens[self.tokens_cursor_down % TRANSLATOR_TOKEN_CAPACITY] as *const Token;
            let token = unsafe { &*raw_token };
            self.tokens_cursor_down += 1;
            match &token.payload {
                TokenPayload::Memory(memory) => translate_memory_token(self, memory, factory),
                TokenPayload::Arithmetic(arithmetic) => translate_arithmetic_token(self, arithmetic, factory)
            }   
        }
    }

    pub fn check_free_space(&self) -> usize {
        TRANSLATOR_TOKEN_CAPACITY + self.tokens_cursor_down - self.tokens_cursor_up
    }

    pub fn save_token(&mut self, token: Token) {
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

    pub fn save_instruction(&mut self, i: Instruction<'a>) -> bool {
        if self.cursor == TRANSLATOR_INSTRUCTIONS_CAPACITY {
            false
        } else {
            self.instructions[self.cursor] = InstructionOrLink::I(i);
            self.cursor += 1;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use hack_macro::instruction;

    use std::fs::File;
    use std::io::prelude::*;

    use crate::tokens::{ArithmeticToken, MemoryToken, MemoryTokenKind, Segment};
    use crate::translator::constants::PUSH_INSTRUCTIONS;

    use super::*;

    #[test]
    fn save_instruction_test() {
        let mut t = Translator::new();
        assert!(t.save_instruction(instruction!(b"@SP")));
        let mut buff = [0; 100];
        let l = t.instructions_to_symbols(&mut buff);
        assert!(buff[..l] == *b"@SP\n")
    }

    #[test]
    fn save_link_test() {
        let mut t = Translator::new();
        assert!(t.save_link(&PUSH_INSTRUCTIONS));
        let mut buff = [0; 100];
        let l = t.instructions_to_symbols(&mut buff);
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
        let l = t.instructions_to_symbols(&mut buff);
        let answer = b"@THIS\n// Write value to SP from D\n@SP\nA=M\nM=D\n// Incriment sp\n@SP\nM=M+1\nM=D\n";
        assert!(l == answer.len());
        assert!(buff[..l] == *answer)
    }

    #[test]
    fn translate_memory_test() {
        let mut file = File::open("./priv/memory/arg_0_pop_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = Token {
            src: 0,
            instruction: 0,
            payload: TokenPayload::Memory(MemoryToken {
                kind: MemoryTokenKind::Pop,
                segment: Segment::Arg,
                val: 0,
            }),
        };

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_cmp_test() {
        let mut file = File::open("./priv/arithmetic/cmp_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = Token {
            src: 0,
            instruction: 0,
            payload: TokenPayload::Arithmetic(ArithmeticToken::Eq)
        };

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_add_test() {
        let mut file = File::open("./priv/arithmetic/add_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = Token {
            src: 0,
            instruction: 0,
            payload: TokenPayload::Arithmetic(ArithmeticToken::Add)
        };

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_sub_test() {
        let mut file = File::open("./priv/arithmetic/sub_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = Token {
            src: 0,
            instruction: 0,
            payload: TokenPayload::Arithmetic(ArithmeticToken::Sub)
        };

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }

    #[test]
    fn translate_arithmetic_not_test() {
        let mut file = File::open("./priv/arithmetic/not_answer.asm").unwrap();
        let mut factory = VariableFactory::new(b"AnyFile");

        let token = Token {
            src: 0,
            instruction: 0,
            payload: TokenPayload::Arithmetic(ArithmeticToken::Not)
        };

        let mut buff = [0u8; 1024];
        let mut file_buff = [0u8; 1024];

        let mut t = Translator::new();

        t.save_token(token);
        t.translate(&mut factory);

        let l = t.instructions_to_symbols(&mut buff);
        let l2 = file.read(&mut file_buff).unwrap();

        assert!(buff[..l] == file_buff[..l2]);
    }
}
