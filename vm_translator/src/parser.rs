use phf::phf_map;
use std::borrow::Cow;

const POP_FROM_SP: &str = "// Decriment sp\n@SP\nM=M-1\n// Extract value from sp\n@SP\nA=M\nD=M\n";
const PUSH_TO_SP: &str = "// Write value to sp\n@SP\nA=M\nM=D\n// Incriment sp\n@SP\nM=M+1\n";

#[derive(Hash, Debug)]
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

fn load_value_by_pointer_label(pointer_label: &str, mut val: i16) -> Cow<'static, str> {
    let mut s = format!("@{}\nA=M\n", pointer_label);
    while val > 0 {
        s.push_str("A=A+1\n");
        val -= 1;
    }
    s.push_str("D=M\n");
    Cow::Owned(s)
}

fn save_value_to_pointer_label(pointer_label: &str, mut val: i16) -> Cow<'static, str> {
    let mut s = format!("@{}\nA=M\n", pointer_label);
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
            Self::Const => Cow::Owned(format!("@{val}\nD=A\n")),
            Self::Temp => Cow::Owned(format!("@{}\nD=M\n", val + 5)),
            Self::Static => Cow::Owned(format!("@{}.{}\nD=M\n", file_name, val)),
            Self::Pointer => match val {
                0 => Cow::Owned(format!("@THIS\nD=M\n")),
                1 => Cow::Owned(format!("@THAT\nD=M\n")),
                _ => unreachable!(),
            },
        }
    }

    pub fn save_data(&self, val: i16, file_name: &str) -> Cow<'static, str> {
        match self {
            Self::Arg => save_value_to_pointer_label("ARG", val),
            Self::This => save_value_to_pointer_label("THIS", val),
            Self::That => save_value_to_pointer_label("THAT", val),
            Self::Local => save_value_to_pointer_label("LCL", val),
            Self::Const => unreachable!(),
            Self::Temp => Cow::Owned(format!("@{}\nM=D\n", val + 5)),
            Self::Static => Cow::Owned(format!("@{}.{}\nM=D\n", file_name, val)),
            Self::Pointer => match val {
                0 => Cow::Owned(format!("@THIS\nM=D\n")),
                1 => Cow::Owned(format!("@THAT\nM=D\n")),
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug)]
enum MemoryTokenKind {
    Pop,
    Push,
}

#[derive(Debug)]
struct MemoryToken<'a> {
    segment: &'a Segment,
    kind: MemoryTokenKind,
    val: i16,
}

impl<'a> MemoryToken<'a> {
    pub fn new(kind: MemoryTokenKind, segment: &'a Segment, val: i16) -> Self {
        Self { segment, val, kind }
    }
}

#[derive(Debug)]
enum BranchTokenKind {
    Label,
    Goto,
    IfGoto,
}

#[derive(Debug)]
struct BranchToken<'a> {
    kind: BranchTokenKind,
    name: &'a str,
}

#[derive(Debug)]
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

static SEGMENTS: phf::Map<&'static [u8], Segment> = phf_map! {
    b"argument" => Segment::Arg,
    b"constant" => Segment::Const,
    b"pointer" => Segment::Pointer,
    b"static" => Segment::Static,
    b"local" => Segment::Local,
    b"temp" => Segment::Temp,
    b"that" => Segment::That,
    b"this" => Segment::This
};

trait Assemble {
    fn assemble(&self, file_name: &str, idx: usize) -> Cow<'static, str>;
}

impl Assemble for MemoryToken<'_> {
    fn assemble(&self, file_name: &str, _idx: usize) -> Cow<'static, str> {
        let command = format!("// {:?} {:?} {}\n", self.kind, self.segment, self.val);
        match &self.kind {
            MemoryTokenKind::Pop => Cow::Owned(format!(
                "\n{}{}{}\n",
                command,
                POP_FROM_SP,
                self.segment.save_data(self.val, file_name)
            )),
            MemoryTokenKind::Push => Cow::Owned(format!(
                "\n{}{}{}\n",
                command,
                self.segment.load_data(self.val, file_name),
                PUSH_TO_SP
            )),
        }
    }
}

fn two_elements_operation(name: &str, operand: &str) -> Cow<'static, str> {
    Cow::Owned(format!(
        "// {}\n{}// Update in place sp\n@SP\nA=M-1\nM=M{}D\n",
        name, POP_FROM_SP, operand
    ))
}

fn cmp_operation(name: &str, jump: &str, idx: usize) -> Cow<'static, str> {
    let to_bool = format!("@TRUE{idx}\nD;{jump}\n@SP\nA=M-1\nM=0\n@FALSE{idx}\n0;JMP\n(TRUE{idx})\n@SP\nA=M-1\nM=-1\n(FALSE{idx})\n");
    Cow::Owned(format!(
        "// {}\n{}// Write cmp result sp\n@SP\nA=M-1\nD=M-D\n{}",
        name, POP_FROM_SP, to_bool
    ))
}

impl Assemble for BranchToken<'_> {
    fn assemble(&self, _file_name: &str, _idx: usize) -> Cow<'static, str> {
        match self.kind {
            BranchTokenKind::Label => Cow::Owned(format!("({})", self.name)),
            BranchTokenKind::Goto => Cow::Owned(format!("// goto\n@{}\n0;JMP\n", self.name)),
            BranchTokenKind::IfGoto => Cow::Owned(format!(
                "// if goto\n{}\n@{}\nD;JNE\n",
                POP_FROM_SP, self.name
            )),
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
            ' ' => (),
            '\t' => (),
            '/' => {
                if '/' == b[pointer + 1] as char {
                    return Cow::Borrowed("");
                }
                unreachable!();
            }
            _ => {
                new_pointer = pointer + cupture_word(&b[pointer..]);
                let token: Box<dyn Assemble> = match &b[pointer..new_pointer] {
                    b"push" => {
                        Box::new(build_memory_command(MemoryTokenKind::Push, b, new_pointer))
                    }
                    b"pop" => Box::new(build_memory_command(MemoryTokenKind::Pop, b, new_pointer)),
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
                        Box::new(build_branch_command(BranchTokenKind::Label, b, new_pointer))
                    }
                    b"goto" => {
                        Box::new(build_branch_command(BranchTokenKind::Goto, b, new_pointer))
                    }
                    b"if-goto" => Box::new(build_branch_command(
                        BranchTokenKind::IfGoto,
                        b,
                        new_pointer,
                    )),
                    v => panic!("Unexpected command {}", std::str::from_utf8(v).unwrap()),
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
