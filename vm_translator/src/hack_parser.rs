use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::lexer::VMLexer;
use crate::tokens::*;

trait Translate {
    fn translate(&self, buff: &mut [u8], file_name: &[u8], file_name_size: usize, is_d_loaded: bool) -> usize;
    fn translate_size(&self, file_name_size: usize, is_d_loaded: bool) -> usize;
}

impl MemoryToken {
    const RESTORE_ADDRESS_COMPLEXITY: i16 = 6;
    const PUSH_D_TO_STACK: &[u8; 22] = b"@SP\nA=M\nM=D\n@SP\nM=M+1\n";
    const POP_D_FROM_STACK: &[u8; 22] = b"@SP\nM=M-1\n@SP\nA=M\nD=M\n";

    // const MAX_STORE_SEGMENT_ADDRESS_SIZE: usize = 29;
    const LOAD_TO_D_REGISTER_SIZE: usize = 44; // or file_name_size + 6
    const LOAD_FROM_D_REGISTER_SIZE: usize = 44; // or file_name_size + 6

    fn write_label(&self, buff: &mut [u8]) -> usize {
        match self.segment {
            Segment::Arg => {
                buff[..3].copy_from_slice(b"@ARG\n");
                5
            }
            Segment::This => {
                buff[..4].copy_from_slice(b"@THIS\n");
                6
            }
            Segment::That => {
                buff[..3].copy_from_slice(b"@THAT\n");
                6
            }
            Segment::Local => {
                buff[..3].copy_from_slice(b"@LCL\n");
                5
            }
            _ => unreachable!()
        }
    }

    fn determine_store_segment_address(&self, buff: &mut [u8]) -> usize {
        match (self.segment, self.val) {
            (Segment::Const, _) => 0,
            (Segment::Static, _) => 0,
            (Segment::Temp, _) => 0,
            (Segment::Pointer, _) => 0,
            (_, v) if v >= Self::RESTORE_ADDRESS_COMPLEXITY => {
                let mut buff_pointer = 0;

                buff_pointer += self.write_label(buff); // max 6

                write_constant(buff, &mut buff_pointer, b"D=A\n@");
                buff_pointer += write_as_bytes(&mut buff[buff_pointer..], v); // 3
                write_constant(buff, &mut buff_pointer, b"\nD=D+A\n@13\nM=D\n");

                buff_pointer
            }
            (_, _) => 0
        }
    }

    fn load_data_to_d_register(&self, buff: &mut [u8], file_name: &[u8], file_name_size: usize) -> usize {
        let mut buff_pointer = 0;
        match (self.segment, self.val) {
            (Segment::Const, v) => {
                buff[buff_pointer] = b'@'; // 1
                buff_pointer += 1;

                buff_pointer += write_as_bytes(&mut buff[buff_pointer..], v); // 3
                write_constant(buff, &mut buff_pointer, b"\nD=A\n"); // 5
                //9
            }
            (Segment::Static, _) =>  {
                buff[buff_pointer] = b'@'; // 1
                buff_pointer += 1;

                buff[buff_pointer..(buff_pointer + file_name_size + 1)].copy_from_slice(file_name);
                buff_pointer += file_name_size;
               
                write_constant(buff, &mut buff_pointer, b"\nD=A\n"); // 5
                // file_name_size + 6
            }
            (Segment::Temp, v) => {
                if v > 7 {
                    panic!("Max tmp value is 7")
                }

                buff[buff_pointer] = b'@'; // 1
                buff_pointer += 1;

                buff_pointer += write_as_bytes(&mut buff[buff_pointer..], v + 5); // 2

                write_constant(buff, &mut buff_pointer, b"\nD=A\n"); // 5
                //8
            }
            (Segment::Pointer, 0) => {
                write_constant(buff, &mut buff_pointer, b"@THIS\nD=M\n");
                //9
            }
            (Segment::Pointer, 1) => {
                write_constant(buff, &mut buff_pointer, b"@THAT\nD=M\n");
                //9
            }
            (Segment::Pointer, _) => panic!("pointer value should be or 0 or 1"),
            (_, v) if v >= Self::RESTORE_ADDRESS_COMPLEXITY => {
                write_constant(buff, &mut buff_pointer, b"@13\nD=M\n"); // 8
                //8
            }
            (_, v) => {
                buff_pointer += self.write_label(buff); // 6
                write_constant(buff, &mut buff_pointer, b"A=M\n"); // 4

                for _i in 0..v {
                    write_constant(buff, &mut buff_pointer, b"A=A+1\n"); // 6
                }

                write_constant(buff, &mut buff_pointer, b"D=M\n"); // 4

                // max v is RESTORE_ADDRESS_COMPLEXITY - 1
                // so 14 + 6 * (RESTORE_ADDRESS_COMPLEXITY - 1)
                // so 44
            }
        }
        buff_pointer
    }

    fn load_data_from_d_register(&self, buff: &mut [u8], file_name: &[u8], file_name_size: usize) -> usize {
        let mut buff_pointer = 0;
        match (self.segment, self.val) {
            (Segment::Const, _) => panic!("Can't pop const"),
            (Segment::Static, _) =>  {
                buff[buff_pointer] = b'@'; // 1
                buff_pointer += 1;

                buff[buff_pointer..(buff_pointer + file_name_size + 1)].copy_from_slice(file_name);
                buff_pointer += file_name_size;
               
                write_constant(buff, &mut buff_pointer, b"\nM=D\n"); // 5
                // file_name_size + 6
            }
            (Segment::Temp, v) => {
                if v > 7 {
                    panic!("Max tmp value is 7")
                }

                buff[buff_pointer] = b'@'; // 1
                buff_pointer += 1;

                buff_pointer += write_as_bytes(&mut buff[buff_pointer..], v + 5); // 2

                write_constant(buff, &mut buff_pointer, b"\nM=D\n"); // 5
                //8
            }
            (Segment::Pointer, 0) => {
                write_constant(buff, &mut buff_pointer, b"@THIS\nM=D\n");
                //9
            }
            (Segment::Pointer, 1) => {
                write_constant(buff, &mut buff_pointer, b"@THAT\nM=D\n");
                //9
            }
            (Segment::Pointer, _) => panic!("pointer value should be or 0 or 1"),
            (_, v) if v >= Self::RESTORE_ADDRESS_COMPLEXITY => {
                write_constant(buff, &mut buff_pointer, b"@13\nM=D\n"); // 8
                //8
            }
            (_, v) => {
                buff_pointer += self.write_label(buff); // 6
                write_constant(buff, &mut buff_pointer, b"A=M\n"); // 4

                for _i in 0..v {
                    write_constant(buff, &mut buff_pointer, b"A=A+1\n"); // 6
                }

                write_constant(buff, &mut buff_pointer, b"M=D\n"); // 4

                // max v is RESTORE_ADDRESS_COMPLEXITY - 1
                // so 14 + 6 * (RESTORE_ADDRESS_COMPLEXITY - 1)
                // so 44
            }
        }
        buff_pointer
    }
}

fn write_constant(buff: &mut [u8], buff_pointer: &mut usize, to_write: &[u8]) {
    let l = to_write.len();
    buff[*buff_pointer..(*buff_pointer + l + 1)].copy_from_slice(to_write);
    *buff_pointer += l;
}

fn write_as_bytes(buff: &mut [u8], v: i16) -> usize {
    let str_representation = v.to_string();
    let v_byte_representation = str_representation.as_bytes();
    let l = v_byte_representation.len(); // max 3

    if l > 3 {
        panic!("To big pointer")
    }

    buff[..(l + 1)].copy_from_slice(v_byte_representation);
    l
}

impl Translate for MemoryToken {
    fn translate(&self, buff: &mut [u8], file_name: &[u8], file_name_size: usize, is_d_loaded: bool) -> usize {
        let mut pointer = 0;
        match self.kind {
            MemoryTokenKind::Pop => {
                if !is_d_loaded {
                    pointer += 22;
                    buff[..23].copy_from_slice(Self::POP_D_FROM_STACK);
                }
                self.load_data_from_d_register(&mut buff[22..ß])

            },
            MemoryTokenKind::Push => {

            }
        }
    }

    fn translate_size(&self, file_name_size: usize, is_d_loaded: bool) -> usize {
        let mut size = match self.kind {
            MemoryTokenKind::Pop => Self::LOAD_FROM_D_REGISTER_SIZE,
            MemoryTokenKind::Push => Self::LOAD_TO_D_REGISTER_SIZE
        };

        size = std::cmp::max(size, file_name_size + 6);

        if is_d_loaded {
            if self.kind == MemoryTokenKind::Push {
                size + 22
            } else {
                panic!("pop value which wasn't pushed")
            }
        } else {
            if self.kind == MemoryTokenKind::Push {
                size
            }
            size + 22
        }
    }
}


pub struct VMHackParser {
    file: File,
    lexer: VMLexer,
    buffer: [u8; Self::PARSER_BUFFER_SIZE],
    memory_cursor: usize,
}

impl VMHackParser {
    const PARSER_BUFFER_SIZE: usize = 8192;

    pub async fn new(lexer: VMLexer, file_path: &str) -> Self {
        let file = File::create(file_path).await.expect("Create error");
        Self {
            file,
            lexer,
            buffer: [0; Self::PARSER_BUFFER_SIZE],
            memory_cursor: 0,
        }
    }

    pub async fn run(&mut self) {
        while let Some(token) = self.lexer.next_token().await {
            self.translate_token(token).await;
        }
    }

    async fn translate_token(&mut self, token: Token) {
        println!("{}", token);
    }

    async fn write_buffer(&mut self) {
        self.file
            .write_all(&self.buffer[..self.memory_cursor])
            .await
            .expect("Write error");
        self.memory_cursor = 0;
    }

    async fn enable_block_writing(&mut self, block_size: usize) {
        if self.memory_cursor + block_size >= Self::PARSER_BUFFER_SIZE {
            self.write_buffer();
        }
    }
}
