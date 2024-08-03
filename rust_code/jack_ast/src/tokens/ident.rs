use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::xml::IntoXML;

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

impl IntoXML for JackIdent {
    async fn write_xml<T: AsyncWrite + Unpin>(&self, write: &mut T) -> Result<usize> {
        let mut n = write.write(b"<identifier> ").await?;
        n += write.write(&self.0).await?;
        n += write.write(b" </identifier>").await?;
        Ok(n)
    }
}
