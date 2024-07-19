use symbolic::SymbolicElem;

#[derive(Debug)]
pub struct LabelInstruction<'a> {
    pub prefix: &'a [u8],
    pub name: &'a [u8],
    pub prefix_len: usize,
    pub name_len: usize,
    pub idx: i16,
}

#[derive(Debug)]
pub enum HelperInstruction<'a> {
    Label(LabelInstruction<'a>),
    LabelVariable(LabelInstruction<'a>),
    Comment(&'a [u8]),
}

impl<'a> SymbolicElem<'a> for HelperInstruction<'a> {
    fn write_symbols(&self, buff: &mut [u8]) -> usize {
        match self {
            HelperInstruction::Label(label) => {
                buff[0] = b'(';

                buff[1..(label.prefix_len + 1)].copy_from_slice(label.prefix);
                buff[label.prefix_len + 1] = b'_';
                buff[(label.prefix_len + 2)..(label.prefix_len + label.name_len + 2)]
                    .copy_from_slice(label.name);

                buff[label.prefix_len + label.name_len + 2] = b'_';

                let l = write_i16_to_buff(
                    label.idx,
                    &mut buff[(label.prefix_len + label.name_len + 3)..],
                );

                buff[label.prefix_len + label.name_len + l + 3] = b')';
                label.prefix_len + label.name_len + l + 4
            }
            HelperInstruction::LabelVariable(label) => {
                buff[0] = b'@';

                buff[1..(label.prefix_len + 1)].copy_from_slice(label.prefix);
                buff[label.prefix_len + 1] = b'_';
                buff[(label.prefix_len + 2)..(label.prefix_len + label.name_len + 2)]
                    .copy_from_slice(label.name);

                buff[label.prefix_len + label.name_len + 2] = b'_';

                let l = write_i16_to_buff(
                    label.idx,
                    &mut buff[(label.prefix_len + label.name_len + 3)..],
                );

                label.prefix_len + label.name_len + l + 3
            }
            HelperInstruction::Comment(text) => {
                let l = text.len();
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
