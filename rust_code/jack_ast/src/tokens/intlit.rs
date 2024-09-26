#[derive(Debug, PartialEq)]
pub struct JackInt(pub Vec<u8>);

impl JackInt {
    pub fn parse_int_literal(buff: &[u8]) -> (Self, usize, bool) {
        let mut l = 0;
        let mut v = Vec::new();

        loop {
            match buff.get(l) {
                None => return (JackInt(v), l, false),
                Some(c) => {
                    if !Self::is_int_char(*c) {
                        return (JackInt(v), l, true);
                    }
                    v.push(*c);
                }
            }

            l += 1;
        }
    }

    pub fn to_int(&self) -> i16 {
        let l = self.0.len();

        if l > 5 {
            panic!("Too big int token");
        }

        let mut sum = 0;
        for i in 0..l {
            sum += (self.0[l - 1 - i] as i16 - 48) * 10_i32.pow(i as u32) as i16;
        }
        sum
    }

    pub fn is_int_char(c: u8) -> bool {
        c > 47 && c < 58
    }
}
