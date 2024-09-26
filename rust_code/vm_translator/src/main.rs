use std::collections::HashMap;
use std::ffi::OsStr;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::{env, mem};

use context::WriteFileContext;
use hack_instructions::{Instruction, VariableFactory};
use tokio::fs::{read_dir, File, OpenOptions};
use tokio::io::{self, AsyncSeekExt, AsyncWriteExt};
use translator::{TranslateOpts, Translator};

mod context;
mod translator;

const PATH_TO_BIFS: &'static str = "../static/bifs";

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

    let mut file_context = WriteFileContext::new();

    if file_path.is_dir() {
        let mut translator = Translator::new_with_opts(opts);
        let mut factory = VariableFactory::new(b"initial_call");
        translator.init_translator(&mut factory);
        let l = if binary_target {
            translator.instructions_to_bytes(
                &mut buff,
                100,
                &mut pointer,
                &mut static_map,
                &mut file_context,
            )
        } else {
            translator.instructions_to_symbols(&mut buff, 100)
        };
        file_context.set_new_pointer(f_write.write(&mut buff[..l]).await.unwrap());

        let mut paths = read_dir(PATH_TO_BIFS).await.unwrap();

        while let Some(path) = paths.next_entry().await? {
            let path_type = path.path();
            let vm_extension = Some(OsStr::new("vm"));
            if vm_extension == path_type.extension() {
                translate_file(
                    path_type.as_path(),
                    &mut f_write,
                    &mut buff,
                    opts,
                    binary_target,
                    &mut pointer,
                    &mut static_map,
                    &mut file_context,
                )
                .await?
            }
        }

        paths = read_dir(file_path).await.unwrap();

        while let Some(path) = paths.next_entry().await? {
            let path_type = path.path();
            let vm_extension = Some(OsStr::new("vm"));
            if vm_extension == path_type.extension() {
                translate_file(
                    path_type.as_path(),
                    &mut f_write,
                    &mut buff,
                    opts,
                    binary_target,
                    &mut pointer,
                    &mut static_map,
                    &mut file_context,
                )
                .await?
            }
        }
    } else {
        translate_file(
            &file_path,
            &mut f_write,
            &mut buff,
            opts,
            binary_target,
            &mut pointer,
            &mut static_map,
            &mut file_context,
        )
        .await?
    };

    drop(f_write);

    let mut f2_write = OpenOptions::new().write(true).open(write_file_path).await?;

    for (label, idxs) in file_context.pointer_map.iter() {
        for idx in idxs {
            f2_write.seek(io::SeekFrom::Start(*idx as u64)).await?;
            let value = static_map.get(label).unwrap();
            f2_write.write(value.as_bytes()).await?;
        }
    }

    Ok(())
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
    file_pointer: &mut WriteFileContext,
) -> io::Result<()> {
    let mut parser = vm_parser::VMParser::new(file_path.to_str().unwrap()).await?;
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
            if let Some(token) = parser.next_instruction().await {
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
                    file_pointer,
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
            file_pointer,
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
    file_pointer: &mut WriteFileContext,
) -> io::Result<()> {
    let mut sum = 0;
    loop {
        let l = if binary_target {
            translator.instructions_to_bytes(buff, 100, static_pointer, static_map, file_pointer)
        } else {
            translator.instructions_to_symbols(buff, 300)
        };

        if l == 0 {
            break;
        }

        sum += f_write.write(&mut buff[..l]).await.unwrap();
    }
    file_pointer.set_new_pointer(sum);
    Ok(())
}
