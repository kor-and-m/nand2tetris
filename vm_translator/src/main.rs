use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;

use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

mod lexer;
mod parser;
mod tokens;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = Path::new(&args[1]);
    let mut lexer = lexer::VMLexer::new(&args[1]).await?;
    // let default_out = file_path
    //     .with_extension("asm")
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    // let write_path = if args.len() > 2 {
    //     &args[2]
    // } else {
    //     &default_out
    // };

    // let mut parser = hack_parser::VMHackParser::new(lexer, write_path).await;
    // parser.run().await;

    while let Some(i) = lexer.next_token().await {
        println!("{}", i);
    }

    if file_path.extension().expect("File extension undefined") != "vm" {
        return Err(Error::new(ErrorKind::Other, "File ext should be vm!"));
    }
    let src_file_name = file_path.file_stem().expect("Wrong stem");

    let f = File::open(&file_path).await?;
    let mut f_write = if args.len() > 2 {
        File::create(&args[2]).await?
    } else {
        File::create(file_path.with_extension("asm")).await?
    };

    let reader = BufReader::new(f);
    let mut lines = reader.lines();

    let file_str_name = src_file_name.to_str().unwrap();
    let mut counter = 0;

    while let Some(line) = lines.next_line().await? {
        let r = parser::assemble(&line, file_str_name, counter);
        f_write.write(r.as_bytes()).await?;
        counter += 1;
    }

    Ok(())
}
