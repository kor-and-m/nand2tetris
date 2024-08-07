use std::{env, ffi::OsStr};

use jack_ast::{gramar::write_to_xml, tokens::JackTokenizer};
use tokio::{fs::File, io::Result};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let src_dir = &args[1];
    let out_dir = &args[1];

    let mut files_in_dir = tokio::fs::read_dir(src_dir).await.unwrap();
    let mut files = Vec::new();

    while let Some(child) = files_in_dir.next_entry().await.unwrap() {
        let metadata = child.metadata().await.unwrap();
        if metadata.is_file() {
            let p = child.path();
            if let Some("jack") = p.extension().and_then(OsStr::to_str) {
                files.push(p)
            };
        }
    }

    for f in files {
        let n = f.file_stem().and_then(OsStr::to_str).unwrap().to_string();
        let out_file_path = format!("{out_dir}/{n}.xml");
        let file = File::open(f).await.unwrap();
        let mut tokenizer = JackTokenizer::from_file(file, true);
        write_to_xml(&mut tokenizer, &out_file_path).await;
    }

    Ok(())
}
