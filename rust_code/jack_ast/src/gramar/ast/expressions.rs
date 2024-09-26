use crate::tokens::{JackSymbol, JackToken};

use super::expression::JackExpression;

#[derive(Debug, PartialEq)]
pub struct JackExpressions {
    pub size: usize,
    pub data: Vec<JackExpression>,
}

impl JackExpressions {
    pub fn extract_from_round_bracket(tokens: &mut [JackToken]) -> Self {
        let mut brakets = 1;
        let mut i = 0;
        let mut expressions = Vec::new();
        let mut expression_cursor = i;

        while brakets != 0 {
            match tokens[i] {
                JackToken::Symbol(JackSymbol::OpenRoundBracket) => brakets += 1,
                JackToken::Symbol(JackSymbol::CloseRoundBracket) => brakets -= 1,
                JackToken::Symbol(JackSymbol::Comma) => {
                    let expr = JackExpression::new(
                        &mut tokens[expression_cursor..i],
                        i - expression_cursor,
                    );

                    expression_cursor = i + 1;
                    expressions.push(expr)
                }
                _ => (),
            };

            i += 1;
        }

        if expression_cursor + 1 < i {
            let expr =
                JackExpression::new(&mut tokens[expression_cursor..i], i - expression_cursor - 1);
            expressions.push(expr)
        }

        JackExpressions {
            data: expressions,
            size: i - 1,
        }
    }
}

mod tests {
    #![allow(unused_imports, dead_code)]
    use futures::StreamExt;

    use crate::tokens::{JackIdent, JackInt, JackTokenizer};

    use super::*;

    #[tokio::test]
    async fn simple_expressions_test() {
        let mut data = bytes_to_tokens(b"12 + c, \"hi\")").await;
        let l = data.len();
        let expressions = JackExpressions::extract_from_round_bracket(&mut data);

        assert_eq!(expressions.size, l - 1);
        assert_eq!(expressions.data.len(), 2);
    }

    #[tokio::test]
    async fn nasted_expressions_test() {
        let mut data = bytes_to_tokens(b"12 + (c * 10), \"hi\")").await;
        let l = data.len();
        let expressions = JackExpressions::extract_from_round_bracket(&mut data);

        assert_eq!(expressions.size, l - 1);
        assert_eq!(expressions.data.len(), 2);
    }

    async fn bytes_to_tokens(expr: &'static [u8]) -> Vec<JackToken> {
        let tokenizer = JackTokenizer::from_slice(expr, true);
        tokenizer
            .map(|x| x.payload)
            .collect::<Vec<JackToken>>()
            .await
    }
}
