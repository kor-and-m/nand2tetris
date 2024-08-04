use std::mem;

use crate::tokens::JackToken;

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    statement::{Statement, StatementData},
};

#[derive(Default)]
pub struct StatementsData {
    statement: Option<JackAstElem<Statement, StatementData>>,
}

pub struct Statements;

impl JackAstElem<Statements, StatementsData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        if self.data.statement.is_none() {
            let mut statement: JackAstElem<Statement, StatementData> = JackAstElem::default();
            statement.feed(token);
            self.is_ready = statement.is_ready;
            self.data.statement = Some(statement);
            return;
        }

        if !self.data.statement.as_ref().unwrap().feed_token(&token) {
            self.data.statement.as_mut().unwrap().terminate();
            let mut statement: Option<JackAstElem<Statement, StatementData>> =
                Some(JackAstElem::default());
            statement.as_mut().unwrap().feed(token);
            mem::swap(&mut statement, &mut self.data.statement);
            unsafe { self.push_ast(statement.unwrap()) };
        } else {
            self.data.statement.as_mut().unwrap().feed(token);
        }

        self.is_ready = self.data.statement.as_ref().unwrap().is_ready;
    }

    pub fn terminate(&mut self) {
        if self.is_error || self.data.statement.is_none() {
            return;
        }

        let mut e: Option<JackAstElem<Statement, StatementData>> = None;
        mem::swap(&mut e, &mut self.data.statement);
        let mut statement = e.unwrap();
        statement.terminate();

        if statement.is_error || !statement.is_ready {
            return;
        }

        unsafe { self.push_ast(statement) };
        self.is_ready = true;
    }
}

impl IntoJackAstKind for Statements {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Statements
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackInt, JackKeyword, JackString, JackSymbol},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_statements() {
        let list_tokens = vec![vec![
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Keyboard".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"readInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"HOW MANY NUMBERS? ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Array".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"new".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
        ]];

        {
            let mut file = File::create("priv/test/gen/statements.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<Statements, StatementsData> = JackAstElem::default();
                for token in tokens {
                    new_elem.feed(token);
                    assert!(!new_elem.is_error);
                }

                new_elem.terminate();
                assert!(new_elem.is_ready);

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/statements.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/statements.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
