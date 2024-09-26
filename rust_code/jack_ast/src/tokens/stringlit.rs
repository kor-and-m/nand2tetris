#[derive(Debug, PartialEq)]
pub struct JackString(pub Vec<u8>);

impl JackString {
    pub fn parse_string_literal(buff: &[u8]) -> (Option<Self>, usize) {
        let mut l = 0;
        let mut v = Vec::new();

        loop {
            match buff.get(l) {
                Some(b'"') => return (Some(JackString(v)), l + 1),
                None => return (None, l),
                Some(c) => {
                    v.push(*c);
                }
            }

            l += 1;
        }
    }
}
