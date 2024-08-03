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
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackTokenizer},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_var_statement() {
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
}
