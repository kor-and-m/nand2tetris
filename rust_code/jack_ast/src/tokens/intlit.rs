use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::xml::IntoXML;

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

    pub fn is_int_char(c: u8) -> bool {
        c > 47 && c < 58
    }
}

impl IntoXML for JackInt {
    async fn write_xml<T: AsyncWrite + Unpin>(&self, write: &mut T) -> Result<usize> {
        let mut n = write.write(b"<integerConstant> ").await?;
        n += write.write(&self.0).await?;
        n += write.write(b" </integerConstant>").await?;
        Ok(n)
    }
}
