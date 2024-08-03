use tokio::io::AsyncWrite;
use tokio::io::Result;

pub trait IntoXML {
    const XML_PADDING: usize = 2;

    async fn write_xml<T: AsyncWrite + Unpin + Send>(&self, write: &mut T) -> Result<usize>;
}
