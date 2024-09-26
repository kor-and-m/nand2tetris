use super::JackInt;

#[derive(Debug, PartialEq)]
pub struct JackIdent(pub Vec<u8>);

impl JackIdent {
    pub fn parse_ident(buff: &[u8]) -> (Self, usize, bool) {
        let mut l = 0;
        let mut v = Vec::new();

        loop {
            match buff.get(l) {
                None => return (JackIdent(v), l, false),
                Some(c) => {
                    if !Self::is_ident_char(*c) && !JackInt::is_int_char(*c) {
                        return (JackIdent(v), l, true);
                    }
                    v.push(*c);
                }
            }

            l += 1;
        }
    }

    pub fn is_ident_char(c: u8) -> bool {
        if c > 96 && c < 123 {
            return true;
        }

        if c > 64 && c < 91 {
            return true;
        }

        c == b'_'
    }
}
