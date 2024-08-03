use file_context::{FileContext, FileSpan};
use std::io::{Error, ErrorKind};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

mod instructions;

pub use instructions::*;

const PARSER_BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub struct VMParser {
    file: File,
    buffer: [u8; PARSER_BUFFER_SIZE],
    cursor: usize,
    end_word_cursor: usize,
    end_page_cursor: usize,
    is_eof: bool,
    src_line: usize,
    instruction_number: usize,
    is_in_progress: bool,
}

impl VMParser {
    pub async fn new(path: &str) -> io::Result<Self> {
        let file_path = Path::new(&path);
        if file_path.extension().expect("File extension undefined") != "vm" {
            return Err(Error::new(ErrorKind::Other, "File ext should be vm!"));
        }

        let file = File::open(&file_path).await?;

        let mut self_state = Self {
            file,
            buffer: [0; PARSER_BUFFER_SIZE],
            cursor: PARSER_BUFFER_SIZE,
            end_word_cursor: PARSER_BUFFER_SIZE,
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
        let to_copy = PARSER_BUFFER_SIZE - self.cursor;

        // becouse instruction size can't be bigger than PARSER_BUFFER_SIZE / 2
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
                        panic!("Comment parsing error")
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
                (false, true, v) if v == b' ' || v == b'\t' || v == b'\r' || v == b'\n' => {
                    if v == b'\n' {
                        self.src_line += 1;
                    }
                    return Some(&self.buffer[self.cursor..self.end_word_cursor]);
                }
                (false, true, _) => self.end_word_cursor += 1,
            }
        }
    }

    async fn build_memory_instruction(
        &mut self,
        kind: AsmMemoryInstructionKind,
    ) -> Option<AsmInstructionPayload> {
        let word = self.next_word().await.unwrap();
        let segment = SEGMENTS[word];
        let val: i16 = self
            .next_word()
            .await
            .unwrap()
            .iter()
            .fold(0, |v, x| v * 10 - 48 + *x as i16);

        Some(AsmInstructionPayload::Memory(AsmMemoryInstruction {
            kind,
            segment,
            val,
        }))
    }

    pub async fn next_instruction(&mut self) -> Option<FileContext<AsmInstructionPayload>> {
        self.is_in_progress = true;
        self.instruction_number += 1;

        let src_line = self.src_line.clone();
        let word = self.next_word().await?;

        let instruction_payload = match word {
            [b'p', b'u', b's', b'h', ..] => self
                .build_memory_instruction(AsmMemoryInstructionKind::Push)
                .await
                .expect("fail to build push instruction"),
            [b'p', b'o', b'p', ..] => self
                .build_memory_instruction(AsmMemoryInstructionKind::Pop)
                .await
                .expect("fail to build pop instruction"),
            [b'a', b'd', b'd', ..] => {
                self.build_arithmetic_instruction(AsmArithmeticInstruction::Add)
            }
            [b's', b'u', b'b', ..] => {
                self.build_arithmetic_instruction(AsmArithmeticInstruction::Sub)
            }
            [b'n', b'e', b'g', ..] => {
                self.build_arithmetic_instruction(AsmArithmeticInstruction::Neg)
            }
            [b'e', b'q', ..] => self.build_arithmetic_instruction(AsmArithmeticInstruction::Eq),
            [b'g', b't', ..] => self.build_arithmetic_instruction(AsmArithmeticInstruction::Gt),
            [b'l', b't', ..] => self.build_arithmetic_instruction(AsmArithmeticInstruction::Lt),
            [b'a', b'n', b'd', ..] => {
                self.build_arithmetic_instruction(AsmArithmeticInstruction::And)
            }
            [b'o', b'r', ..] => self.build_arithmetic_instruction(AsmArithmeticInstruction::Or),
            [b'n', b'o', b't', ..] => {
                self.build_arithmetic_instruction(AsmArithmeticInstruction::Not)
            }
            [b'r', b'e', b't', b'u', b'r', b'n', ..] => {
                AsmInstructionPayload::Function(AsmFunctionInstruction::Return)
            }
            [b'f', b'u', b'n', b'c', b't', b'i', b'o', b'n', ..] => {
                self.build_function_instruction(true).await
            }
            [b'c', b'a', b'l', b'l', ..] => self.build_function_instruction(false).await,
            [b'l', b'a', b'b', b'e', b'l', ..] => {
                self.build_branch_instruction(AsmBranchInstructionKind::Label)
                    .await
            }
            [b'g', b'o', b't', b'o', ..] => {
                self.build_branch_instruction(AsmBranchInstructionKind::Goto)
                    .await
            }
            [b'i', b'f', b'-', b'g', b'o', b't', b'o', ..] => {
                self.build_branch_instruction(AsmBranchInstructionKind::IfGoto)
                    .await
            }
            _ => {
                let i = std::str::from_utf8(word).unwrap();
                panic!("{}: Unexected instruction {}", src_line, i);
            }
        };
        Some(self.enrich_instruction_payload(instruction_payload))
    }

    async fn build_branch_instruction(
        &mut self,
        kind: AsmBranchInstructionKind,
    ) -> AsmInstructionPayload {
        let word = self.next_word().await.unwrap();
        AsmInstructionPayload::Branch(AsmBranchInstruction {
            kind,
            name: word.to_vec(),
        })
    }

    async fn build_function_instruction(&mut self, is_definition: bool) -> AsmInstructionPayload {
        let name = self.next_word().await.unwrap().to_vec();
        let args_count_bytes = self.next_word().await.unwrap();

        let args_count = match args_count_bytes.len() {
            0 => unreachable!(),
            1 => parse_an_int_val(args_count_bytes[0]),
            2 => parse_an_int_val(args_count_bytes[0]) * 10 + parse_an_int_val(args_count_bytes[1]),
            _ => panic!("Too much args"),
        };

        if is_definition {
            AsmInstructionPayload::Function(AsmFunctionInstruction::Definition(FunctionMetadata {
                name,
                args_count,
            }))
        } else {
            AsmInstructionPayload::Function(AsmFunctionInstruction::Call(FunctionMetadata {
                name,
                args_count,
            }))
        }
    }

    fn build_arithmetic_instruction(
        &mut self,
        kind: AsmArithmeticInstruction,
    ) -> AsmInstructionPayload {
        AsmInstructionPayload::Arithmetic(kind)
    }

    fn enrich_instruction_payload(
        &self,
        payload: AsmInstructionPayload,
    ) -> FileContext<AsmInstructionPayload> {
        let span = Some(FileSpan::new(self.src_line, 0));
        FileContext::new(payload, self.instruction_number, None, span)
    }
}

fn parse_an_int_val(i: u8) -> i16 {
    match i {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        _ => panic!("Count args must be an integer"),
    }
}
