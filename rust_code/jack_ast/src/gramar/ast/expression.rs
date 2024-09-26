use super::{term::JackTerm, JackConstantTerm, JackTermPayload};
use crate::tokens::{JackSymbol, JackToken};

#[derive(Debug, PartialEq, Default)]
pub struct JackExpression {
    pub term: JackTerm,
    pub extra: Vec<(JackSymbol, JackTerm)>,
    pub size: usize,
}

impl JackExpression {
    pub fn is_this(&self) -> bool {
        self.extra.len() == 0 && self.term.payload == JackTermPayload::Const(JackConstantTerm::This)
    }

    pub fn new(tokens: &mut [JackToken], tokens_count: usize) -> Self {
        let term = JackTerm::new(tokens, tokens_count);
        let mut size = term.size;
        let mut extra = Vec::new();

        while size != tokens_count {
            if let JackToken::Symbol(s) = tokens[size] {
                if !s.is_op() {
                    panic!("wrong op")
                }

                size += 1;
                let new_term = JackTerm::new(&mut tokens[size..], tokens_count - size);
                size += new_term.size;

                extra.push((s, new_term))
            } else {
                panic!("wrong op")
            };
        }

        Self { term, extra, size }
    }

    pub fn extract_from_round_bracket(tokens: &mut [JackToken]) -> JackExpression {
        let mut brakets = 1;
        let mut i = 0;

        while brakets != 0 {
            match tokens[i] {
                JackToken::Symbol(JackSymbol::OpenRoundBracket) => brakets += 1,
                JackToken::Symbol(JackSymbol::CloseRoundBracket) => brakets -= 1,
                _ => (),
            };

            i += 1;
        }

        Self::new(&mut tokens[..i], i - 1)
    }

    pub fn extract_from_square_bracket(tokens: &mut [JackToken]) -> JackExpression {
        let mut brakets = 1;
        let mut i = 0;

        while brakets != 0 {
            match tokens[i] {
                JackToken::Symbol(JackSymbol::OpenSquareBracket) => brakets += 1,
                JackToken::Symbol(JackSymbol::CloseSquareBracket) => brakets -= 1,
                _ => (),
            };

            i += 1;
        }

        Self::new(&mut tokens[..i], i - 1)
    }
}

mod tests {
    #![allow(unused_imports, dead_code)]
    use futures::StreamExt;

    use crate::{
        gramar::units::JackVariableName,
        tokens::{JackIdent, JackInt, JackTokenizer},
    };

    use super::*;

    #[tokio::test]
    async fn simple_expression_test() {
        let mut data = bytes_to_tokens(b"a + 7 * 4").await;
        let l = data.len();
        let expr = JackExpression::new(&mut data, l);

        assert_eq!(
            expr,
            JackExpression {
                size: l,
                term: JackTerm::new_ident(JackVariableName(b"a".to_vec())),
                extra: vec![
                    (JackSymbol::Plus, JackTerm::new_int(JackInt(b"7".to_vec()))),
                    (
                        JackSymbol::Multiply,
                        JackTerm::new_int(JackInt(b"4".to_vec()))
                    )
                ]
            }
        );
    }

    #[tokio::test]
    async fn expression_test() {
        let mut data = bytes_to_tokens(b"art + Main.calc(12 + 46, \"hi\") * 4 / a[24 - k]").await;
        let l = data.len();
        let expr = JackExpression::new(&mut data, l);

        assert_eq!(expr.size, l);
    }

    async fn bytes_to_tokens(expr: &'static [u8]) -> Vec<JackToken> {
        let tokenizer = JackTokenizer::from_slice(expr, true);
        tokenizer
            .map(|x| x.payload)
            .collect::<Vec<JackToken>>()
            .await
    }
}
