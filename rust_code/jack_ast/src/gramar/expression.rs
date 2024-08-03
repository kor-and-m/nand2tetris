use std::mem;

use crate::tokens::JackToken;

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    term::{Term, TermData},
};

#[derive(Default)]
pub struct ExpressionData {
    term: JackAstElem<Term, TermData>,
}

pub struct Expression;

impl JackAstElem<Expression, ExpressionData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        let t = &self.data.term;

        if t.is_error {
            self.is_error = true;
            return;
        }

        if !t.is_ready {
            self.data.term.feed(token);
        } else {
            match &token {
                JackToken::Symbol(symbol) if symbol.is_op() => {
                    let mut term: JackAstElem<Term, TermData> = JackAstElem::default();
                    mem::swap(&mut self.data.term, &mut term);
                    term.terminate();
                    unsafe { self.push_ast(term) };
                    self.push_token(token);
                }
                _ => self.data.term.feed(token),
            }
        };

        self.is_error = self.data.term.is_error;
        self.is_ready = self.data.term.is_ready;
    }

    pub fn terminate(&mut self) {
        if self.is_error {
            return;
        }

        let mut e: JackAstElem<Term, TermData> = JackAstElem::default();
        mem::swap(&mut e, &mut self.data.term);

        if e.is_error || !e.is_ready {
            return;
        }

        e.terminate();
        unsafe { self.push_ast(e) };
        self.is_ready = true;
    }
}

impl IntoJackAstKind for Expression {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Expression
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackString, JackSymbol},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_expression_statement() {
        let list_tokens = vec![
            vec![
                JackToken::Ident(JackIdent(b"sum".to_vec())),
                JackToken::Symbol(JackSymbol::Plus),
                JackToken::Ident(JackIdent(b"a".to_vec())),
                JackToken::Symbol(JackSymbol::OpenSquareBracket),
                JackToken::Ident(JackIdent(b"i".to_vec())),
                JackToken::Symbol(JackSymbol::CloseSquareBracket),
            ],
            vec![
                JackToken::Ident(JackIdent(b"Keyboard".to_vec())),
                JackToken::Symbol(JackSymbol::Period),
                JackToken::Ident(JackIdent(b"readInt".to_vec())),
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
                JackToken::StringLiteral(JackString(b"ENTER THE NEXT NUMBER: ".to_vec())),
                JackToken::Symbol(JackSymbol::CloseRoundBracket),
            ],
        ];

        {
            let mut file = File::create("priv/test/gen/expressions.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<Expression, ExpressionData> = JackAstElem::default();
                for token in tokens {
                    // println!("{:?}", &token);
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

        let mut generated_file = File::open("priv/test/gen/expressions.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/expressions.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
