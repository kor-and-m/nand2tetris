use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{env, mem};

use hack_ast::{Instruction, VariableFactory};
use tokio::fs::{read_dir, File, OpenOptions};
use tokio::io::{self, AsyncWriteExt};
use translator::{TranslateOpts, Translator};

mod lexer;
mod translator;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let file_path = Path::new(&args[1]);

    let mut buff = {
        let x: [MaybeUninit<u8>; 4096] = unsafe { MaybeUninit::uninit().assume_init() };
        unsafe { mem::transmute::<_, [u8; 4096]>(x) }
    };

    let silent_comments = env::var("SILENT_COMMENTS").is_ok();
    let binary_target = env::var("TO_BINARY").is_ok();
    let ext = if binary_target { "hack" } else { "asm" };
    let mut opts = TranslateOpts::new();
    opts.set_comments(!silent_comments);

    let f = if file_path.is_dir() {
        let mut b = file_path.to_path_buf();
        let os_s = b.file_stem().unwrap();
        let s = format!("{}.{}", os_s.to_str().unwrap(), ext);
        b.push(&s);
        b
    } else {
        file_path.with_extension(ext)
    };

    let write_file_path = if args.len() > 2 {
        Path::new(&args[2])
    } else {
        f.as_path()
    };

    let mut f_write = open_write_file(write_file_path).await?;
    let mut pointer = 16;
    let mut static_map = HashMap::new();

    if file_path.is_dir() {
        let mut paths = read_dir(file_path).await.unwrap();
        let mut translator = Translator::new_with_opts(opts);
        let mut factory = VariableFactory::new(b"initial_call");
        translator.init_translator(&mut factory);
        let l = if binary_target {
            translator.instructions_to_bytes(&mut buff, 100, &mut pointer, &mut static_map)
        } else {
            translator.instructions_to_symbols(&mut buff, 100)
        };
        f_write.write(&mut buff[..l]).await.unwrap();

        while let Some(path) = paths.next_entry().await? {
            let path_type = path.path();
            let ext = path_type.extension().unwrap();
            if let Some("vm") = ext.to_str() {
                translate_file(
                    path_type.as_path(),
                    &mut f_write,
                    &mut buff,
                    opts,
                    binary_target,
                    &mut pointer,
                    &mut static_map,
                )
                .await?
            }
        }

        Ok(())
    } else {
        translate_file(
            &file_path,
            &mut f_write,
            &mut buff,
            opts,
            binary_target,
            &mut pointer,
            &mut static_map,
        )
        .await
    }
}

async fn open_write_file(file_path: &Path) -> io::Result<File> {
    let mut f_write_options = OpenOptions::new();
    f_write_options.append(true).write(true).create(true);
    let f_write = f_write_options.open(file_path).await?;
    f_write.set_len(0).await.unwrap();
    Ok(f_write)
}

async fn translate_file(
    file_path: &Path,
    f_write: &mut File,
    buff: &mut [u8],
    opts: TranslateOpts,
    binary_target: bool,
    static_pointer: &mut i16,
    static_map: &mut HashMap<Vec<u8>, String>,
) -> io::Result<()> {
    let mut lexer = lexer::VMLexer::new(file_path.to_str().unwrap()).await?;
    let src_file_name = file_path.file_stem().expect("Wrong stem");
    let file_parse_comment = format!("Start parsing {}", src_file_name.to_str().unwrap());
    let mut factory = VariableFactory::new(src_file_name.as_bytes());
    let mut translator = Translator::new_with_opts(opts);

    translator.save_instruction(Instruction::new_line());
    translator.save_instruction(Instruction::new_line());
    translator.save_instruction(Instruction::new_comment(file_parse_comment.as_bytes()));

    'outer: loop {
        let space = translator.check_free_space();

        for _i in 0..space {
            if let Some(token) = lexer.next_token().await {
                translator.save_token(token);
            } else {
                translator.translate(&mut factory);
                write_chunks(
                    &mut translator,
                    f_write,
                    buff,
                    binary_target,
                    static_pointer,
                    static_map,
                )
                .await?;
                break 'outer;
            }
        }

        translator.translate(&mut factory);
        write_chunks(
            &mut translator,
            f_write,
            buff,
            binary_target,
            static_pointer,
            static_map,
        )
        .await?;
        translator.reset();
    }

    Ok(())
}

async fn write_chunks(
    translator: &mut Translator<'_>,
    f_write: &mut File,
    buff: &mut [u8],
    binary_target: bool,
    static_pointer: &mut i16,
    static_map: &mut HashMap<Vec<u8>, String>,
) -> io::Result<()> {
    loop {
        let l = if binary_target {
            translator.instructions_to_bytes(buff, 100, static_pointer, static_map)
        } else {
            translator.instructions_to_symbols(buff, 100)
        };
        if l == 0 {
            break;
        }
        f_write.write(&mut buff[..l]).await.unwrap();
        translator.reset_buffer();
    }
    Ok(())
}
