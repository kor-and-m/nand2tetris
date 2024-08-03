use tokio::io::{AsyncWrite, AsyncWriteExt, Result};

use crate::xml::IntoXML;

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

impl IntoXML for JackString {
    async fn write_xml<T: AsyncWrite + Unpin>(&self, write: &mut T) -> Result<usize> {
        let mut n = write.write(b"<stringConstant> ").await?;
        n += write.write(&self.0).await?;
        n += write.write(b" </stringConstant>").await?;
        Ok(n)
    }
}
