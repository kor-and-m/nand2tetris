use std::{env, mem};
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use hack_ast::VariableFactory;
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncWriteExt};
use translator::Translator;

mod lexer;
mod tokens;
mod translator;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = Path::new(&args[1]);
    let mut lexer = lexer::VMLexer::new(&args[1]).await?;
    let src_file_name = file_path.file_stem().expect("Wrong stem");
    let mut factory = VariableFactory::new(src_file_name.as_bytes());
    let mut translator = Translator::new();

    let mut f_write_options = OpenOptions::new();

    f_write_options.append(true).write(true).create(true);

    let mut f_write = if args.len() > 2 {
        f_write_options.open(&args[2]).await?
    } else {
        f_write_options.open(file_path.with_extension("asm")).await?
    };

    let mut buff = {
        let x: [MaybeUninit<u8>; 4096] =
            unsafe { MaybeUninit::uninit().assume_init() };
        unsafe {
            mem::transmute::<_, [u8; 4096]>(x)
        }
    };

    'outer: loop {
        let space = translator.check_free_space();

        for _i in 0..space {
            if let Some(token) = lexer.next_token().await {
                translator.save_token(token);
            } else {
                translator.translate(&mut factory);
                let l = translator.instructions_to_symbols(&mut buff);
                f_write.write(&mut buff[..l]).await.unwrap();
                translator.reset();
                break 'outer;
            }
        }

        translator.translate(&mut factory);
        let l = translator.instructions_to_symbols(&mut buff);
        f_write.write_all(&mut buff[..l]).await.unwrap();
        translator.reset();
    }

    Ok(())
}
