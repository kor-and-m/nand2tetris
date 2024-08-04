use ast_elem::JackAstElem;
use class::{ClassDeclaration, ClassDeclarationData};
use file_context::FileContext;
use futures::{Stream, StreamExt};
use tokio::fs::File;

use crate::{tokens::JackToken, xml::IntoXML};

mod array_index;
mod ast_elem;
mod ast_kind;
mod class;
mod class_var;
mod do_statement;
mod expression;
mod expression_in_breackets;
mod expression_list;
mod function_call;
mod if_statement;
mod let_statement;
mod params_list;
mod return_statement;
mod statement;
mod statements;
mod subroutine_body;
mod subroutine_dec;
mod term;
mod var;
mod while_statement;

pub async fn write_to_xml<S: Stream<Item = FileContext<JackToken>> + Unpin>(
    stream: &mut S,
    file_path: &str,
) {
    let mut new_elem: JackAstElem<ClassDeclaration, ClassDeclarationData> = JackAstElem::default();
    while let Some(t) = stream.next().await {
        new_elem.feed(t.payload);
    }

    let mut file = File::create(file_path).await.unwrap();

    new_elem.write_xml(&mut file).await.unwrap();
}

mod tests {
    #![allow(unused_imports)]
    use std::{ffi::OsStr, path::PathBuf};

    use tokio::{
        fs::File,
        io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    };

    use crate::{
        tokens::{JackIdent, JackTokenizer},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_array_file() {
        let file = File::open("./priv/ArrayTest.jack").await.unwrap();

        let mut tokenizer = JackTokenizer::from_file(file, true);

        write_to_xml(&mut tokenizer, "priv/test/gen/ArrayTest.xml").await;
        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/ArrayTest.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/ArrayTest.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }

    #[tokio::test]
    async fn test_expression_less_square_project() {
        let src_dir = "./priv/ExpressionLessSquare/";
        let cmp_dir = "./priv/ExpressionLessSquare/";
        let out_dir = "./priv/test/gen/ExpressionLessSquare/";

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
            let out_file_path = format!("{out_dir}{n}.xml");
            let cmp_file_path = format!("{cmp_dir}{n}.xml");
            let file = File::open(f).await.unwrap();
            let mut tokenizer = JackTokenizer::from_file(file, true);
            write_to_xml(&mut tokenizer, &out_file_path).await;

            let generated_file = File::open(&out_file_path).await.unwrap();
            let cmp_file = File::open(&cmp_file_path).await.unwrap();

            let cmp_file_reader = BufReader::new(cmp_file);
            let generated_file_reader = BufReader::new(generated_file);
            let mut cmp_file_reader_lines = cmp_file_reader.lines();
            let mut generated_file_reader_lines = generated_file_reader.lines();

            let mut idx = 0;

            while let Some(line) = cmp_file_reader_lines
                .next_line()
                .await
                .expect("Failed to read file")
            {
                let generated = generated_file_reader_lines
                    .next_line()
                    .await
                    .expect("Failed to read file")
                    .expect("Fail read line");
                assert_eq!(line, generated, "Line: {} File {}.jack", idx, n);
                idx += 1;
            }
        }
    }
}
