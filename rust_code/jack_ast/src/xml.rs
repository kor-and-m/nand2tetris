use tokio::io::AsyncWrite;
use tokio::io::Result;

pub const XML_PADDING: usize = 2;

pub trait IntoXML {
    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize>;
}
