use file_context::FileContext;
use futures::{Stream, StreamExt};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    tokens::JackToken,
    xml::{IntoXML, XML_PADDING},
};

mod tag;
mod tags;

pub use tag::JackAstEngine;


pub async fn write_to_xml<S: Stream<Item = FileContext<JackToken>> + Unpin>(
    stream: &mut S,
    file_path: &str,
) {
    let mut engine = JackAstEngine::new(stream);
    let mut xml_file = File::create(file_path).await.unwrap();

    let mut padding = 0;

    while let Some(v) = engine.next().await {
        v.raise_if_error(file_path);

        if v.is_close_tag() {
            padding -= XML_PADDING;
        }

        if v.is_displyable() {
            xml_file.write_all(&vec![b' '; padding]).await.unwrap();
        }

        if v.is_open_tag() {
            padding += XML_PADDING;
        }

        let t = v.write_xml(&mut xml_file).await.unwrap();

        if t != 0 {
            xml_file.write_all(b"\n").await.unwrap();
        }
    }
}
