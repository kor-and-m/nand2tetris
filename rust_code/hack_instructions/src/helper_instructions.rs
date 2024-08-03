use std::collections::HashMap;

use symbolic::SymbolicElem;

#[derive(Debug, Clone)]
pub struct LabelInstruction<'a> {
    pub prefix: &'a [u8],
    pub name: &'a [u8],
    pub prefix_len: usize,
    pub name_len: usize,
    pub idx: i16,
}

impl<'a> SymbolicElem<'a> for LabelInstruction<'a> {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        buff[..self.prefix_len].copy_from_slice(self.prefix);
        buff[self.prefix_len] = b'_';
        buff[(self.prefix_len + 1)..(self.prefix_len + self.name_len + 1)]
            .copy_from_slice(self.name);

        buff[self.prefix_len + self.name_len + 1] = b'_';

        let l = write_i16_to_buff(self.idx, &mut buff[(self.prefix_len + self.name_len + 2)..]);

        self.prefix_len + self.name_len + 2 + l
    }
}

#[derive(Debug, Clone)]
pub enum HelperInstruction<'a> {
    RawLabel(Vec<u8>),
    RawVarLabel(Vec<u8>),
    Label(LabelInstruction<'a>),
    LabelVariable(LabelInstruction<'a>),
    Comment(&'a [u8]),
}

impl HelperInstruction<'_> {
    pub fn write_bytes(
        &self,
        buff: &mut [u8],
        instruction_number: usize,
        m: &mut HashMap<Vec<u8>, String>,
    ) -> (usize, Option<Vec<u8>>) {
        match self {
            Self::Comment(_c) => (0, None),
            Self::RawLabel(v_init) => {
                let v = v_init.clone();
                let s = format!("{:016b}", instruction_number);
                m.insert(v, s);

                (0, None)
            }
            Self::Label(label) => {
                let mut v = Vec::with_capacity(label.name_len + label.prefix_len + 4);
                unsafe { v.set_len(label.name_len + label.prefix_len + 4) };
                let l2 = label.write_symbols(&mut v);
                unsafe { v.set_len(l2) };

                let s = format!("{:016b}", instruction_number);
                m.insert(v, s);

                (0, None)
            }
            Self::LabelVariable(label) => {
                let mut v = Vec::with_capacity(label.name_len + label.prefix_len + 4);
                unsafe { v.set_len(label.name_len + label.prefix_len + 4) };
                let l2 = label.write_symbols(&mut v);
                unsafe { v.set_len(l2) };

                if let Some(value) = m.get(&v[1..]) {
                    buff[..16].copy_from_slice(value.as_bytes());
                    (16, None)
                } else {
                    buff[..16].copy_from_slice(b"1000000000000000");
                    (16, Some(v))
                }
            }
            Self::RawVarLabel(v_init) => {
                let v = v_init.clone();

                if let Some(value) = m.get(&v) {
                    buff[..16].copy_from_slice(value.as_bytes());
                    (16, None)
                } else {
                    buff[..16].copy_from_slice(b"1000000000000000");
                    (16, Some(v))
                }
            }
        }
    }
}

impl<'a> SymbolicElem<'a> for HelperInstruction<'a> {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        match self {
            HelperInstruction::RawLabel(data) => {
                buff[0] = b'(';

                let l = data.len();
                buff[1..(l + 1)].copy_from_slice(data);
                buff[l + 1] = b')';
                l + 2
            }
            HelperInstruction::RawVarLabel(data) => {
                buff[0] = b'@';

                let l = data.len();
                buff[1..(l + 1)].copy_from_slice(data);
                l + 1
            }
            HelperInstruction::Label(label) => {
                buff[0] = b'(';
                let l = label.write_symbols(&mut buff[1..]);
                buff[l + 1] = b')';
                l + 2
            }
            HelperInstruction::LabelVariable(label) => {
                buff[0] = b'@';
                let l = label.write_symbols(&mut buff[1..]);
                l + 1
            }
            HelperInstruction::Comment(text) => {
                let l = text.len();
                if l == 0 {
                    return 0;
                }

                buff[0] = b'/';
                buff[1] = b'/';
                buff[2] = b' ';
                buff[3..(l + 3)].copy_from_slice(text);
                l + 3
            }
        }
    }
}

fn write_i16_to_buff(n: i16, buff: &mut [u8]) -> usize {
    let mut idx = 0;
    for i in n.to_string().chars() {
        buff[idx] = i as u8;
        idx += 1;
    }
    idx
}
