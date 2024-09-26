use std::{env, ffi::OsStr, path::Path};

use jack_ast::gramar::*;
use jack_ast::tokens::JackTokenizer;
use subroutine::JackSubroutineCompilerContext;
use tokio::{
    fs::{read_dir, File},
    io::{AsyncWriteExt, Result},
    task::JoinSet,
};

use class::JackClassCompilerContext;
use vm_parser::AsmInstructionPayload;

mod class;
mod subroutine;
mod vars;

#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let src_file_or_dir = Path::new(&args[1]);

    if src_file_or_dir.is_dir() {
        let mut paths = read_dir(src_file_or_dir).await.unwrap();
        while let Some(path) = paths.next_entry().await? {
            let path_type = path.path();
            let jack_extension = Some(OsStr::new("jack"));
            if jack_extension == path_type.extension() {
                let dest = path_type.with_extension("vm");
                compile_file(&path_type.as_path(), dest.as_path()).await?;
            }
        }
        Ok(())
    } else {
        let dest = src_file_or_dir.with_extension("vm");
        compile_file(src_file_or_dir, dest.as_path()).await?;
        Ok(())
    }
}

async fn compile_file(src: &Path, dest: &Path) -> Result<()> {
    let file = File::open(src).await?;
    let mut file_write = File::create(dest).await?;
    let mut tokenizer = JackTokenizer::from_file(file, true);
    let ast_builder = JackASTBuilderEngine::new(&mut tokenizer);
    let mut ast = ast_builder.build_class().await;
    let class_context = JackClassCompilerContext::init(&mut ast);
    execute_tasks(class_context, ast, &mut file_write).await?;

    Ok(())
}

async fn execute_tasks(
    class_context: JackClassCompilerContext,
    mut ast: JackClass,
    file_write: &mut File,
) -> Result<()> {
    let mut tasks = JoinSet::new();
    let link = unsafe { &*(&class_context as *const JackClassCompilerContext) };
    for subroutine in ast.subroutines.iter_mut() {
        new_task(&mut tasks, link, unsafe {
            &mut *(subroutine as *mut JackSubroutine)
        })
    }

    while let Some(result) = tasks.join_next().await {
        for i in result.unwrap() {
            let s = format!("{}\n", i);
            file_write.write(s.as_bytes()).await?;
        }
    }

    Ok(())
}

fn new_task(
    set: &mut JoinSet<Vec<AsmInstructionPayload>>,
    class_context: &'static JackClassCompilerContext,
    subroutine: &'static mut JackSubroutine,
) {
    set.spawn(async move {
        let r = JackSubroutineCompilerContext::init(&class_context, subroutine, true);

        r.collect::<Vec<_>>()
    });
}
