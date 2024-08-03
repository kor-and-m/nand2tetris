#[derive(Debug, PartialEq)]
pub struct JackComment(pub Vec<u8>);

impl JackComment {
    pub fn parse_comment(buff: &[u8]) -> (Option<Self>, usize, usize, bool) {
        if let Some(b'/') = buff.get(0) {
        } else {
            return (None, 0, 0, true);
        };

        let is_multiline = match (buff.get(1), buff.get(2)) {
            (Some(b'/'), _) => false,
            (Some(b'*'), Some(b'*')) => true,
            _ => return (None, 0, 0, true),
        };

        let mut l = if is_multiline { 3 } else { 2 };
        let mut v = Vec::new();
        let mut lines = 0;

        loop {
            match (buff.get(l), is_multiline) {
                (Some(b'\n'), false) => return (Some(JackComment(v)), l + 1, 0, true),
                (Some(b'\r'), false) => return (Some(JackComment(v)), l + 1, 0, true),
                (Some(b'*'), true) => {
                    if let Some(b'/') = buff.get(l + 1) {
                        return (Some(JackComment(v)), l + 2, lines, true);
                    } else {
                        v.push(b'*');
                    }
                }
                (None, _) => return (Some(JackComment(v)), l + 1, lines, false),
                (Some(c), _) => {
                    if *c == b'\n' {
                        lines += 1;
                    }
                    v.push(*c);
                }
            }

            l += 1;
        }
    }
}
