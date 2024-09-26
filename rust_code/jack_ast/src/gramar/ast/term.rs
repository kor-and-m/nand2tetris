use std::mem;

use crate::{
    gramar::units::{JackVariableName, JackVariableNameStyle},
    tokens::{JackInt, JackKeyword, JackString, JackSymbol, JackToken},
};

use super::{expression::JackExpression, expressions::JackExpressions};

#[derive(Debug, PartialEq, Default)]
pub struct JackTerm {
    pub payload: JackTermPayload,
    pub size: usize,
}

#[derive(Debug, PartialEq)]
pub enum JackConstantTerm {
    True,
    False,
    Null,
    This,
}

#[derive(Debug, PartialEq)]
pub enum JackTermPayload {
    Expression(Box<JackExpression>),
    Unary(JackSymbol, Box<JackTerm>),
    FunctionCall(JackVariableName, JackVariableName, JackExpressions),
    MethodCall(JackVariableName, JackExpressions),
    String(JackString),
    Int(JackInt),
    Ident(JackVariableName),
    Const(JackConstantTerm),
    ArrayElem(JackVariableName, Box<JackExpression>),
}

impl Default for JackTermPayload {
    fn default() -> Self {
        Self::Const(JackConstantTerm::Null)
    }
}

impl JackTerm {
    pub fn new_ident(token: JackVariableName) -> Self {
        Self {
            payload: JackTermPayload::Ident(token),
            size: 1,
        }
    }

    pub fn new_string(data: JackString) -> Self {
        Self {
            payload: JackTermPayload::String(data),
            size: 1,
        }
    }

    pub fn new_int(data: JackInt) -> Self {
        Self {
            payload: JackTermPayload::Int(data),
            size: 1,
        }
    }

    pub fn new(tokens: &mut [JackToken], tokens_count: usize) -> Self {
        let mut v = Vec::new();
        let head = &mut tokens[0] as *mut JackToken;

        match unsafe { &mut *head } {
            JackToken::Keyword(JackKeyword::True) => Self {
                payload: JackTermPayload::Const(JackConstantTerm::True),
                size: 1,
            },
            JackToken::Keyword(JackKeyword::False) => Self {
                payload: JackTermPayload::Const(JackConstantTerm::False),
                size: 1,
            },
            JackToken::Keyword(JackKeyword::Null) => Self {
                payload: JackTermPayload::Const(JackConstantTerm::Null),
                size: 1,
            },
            JackToken::Keyword(JackKeyword::This) => Self {
                payload: JackTermPayload::Const(JackConstantTerm::This),
                size: 1,
            },
            JackToken::StringLiteral(s) => {
                mem::swap(&mut s.0, &mut v);
                Self::new_string(JackString(v))
            }
            JackToken::IntLiteral(s) => {
                mem::swap(&mut s.0, &mut v);
                Self::new_int(JackInt(v))
            }
            JackToken::Symbol(JackSymbol::OpenRoundBracket) => {
                let expr = JackExpression::extract_from_round_bracket(&mut tokens[1..]);
                let size = expr.size + 2;
                Self {
                    payload: JackTermPayload::Expression(Box::new(expr)),
                    size,
                }
            }
            JackToken::Symbol(symbol) if symbol.is_unary_op() => {
                let term = JackTerm::new(&mut tokens[1..], tokens_count - 1);
                let size = term.size + 1;
                Self {
                    payload: JackTermPayload::Unary(*symbol, Box::new(term)),
                    size,
                }
            }
            s => {
                if let Some(new_ident) =
                    JackVariableName::from_token(s, JackVariableNameStyle::Utf8)
                {
                    match &tokens.get(1) {
                        Some(JackToken::Symbol(JackSymbol::OpenRoundBracket)) => {
                            let expr =
                                JackExpressions::extract_from_round_bracket(&mut tokens[2..]);
                            let size = expr.size + 3;
                            Self {
                                payload: JackTermPayload::MethodCall(new_ident, expr),
                                size,
                            }
                        }
                        Some(JackToken::Symbol(JackSymbol::OpenSquareBracket)) => {
                            let expr =
                                JackExpression::extract_from_square_bracket(&mut tokens[2..]);
                            let size = expr.size + 3;
                            Self {
                                payload: JackTermPayload::ArrayElem(new_ident, Box::new(expr)),
                                size,
                            }
                        }
                        Some(JackToken::Symbol(JackSymbol::Period)) => {
                            if tokens_count < 5 {
                                panic!("Unknown term");
                            }

                            let maybe_function_name = JackVariableName::from_token(
                                &mut tokens[2],
                                JackVariableNameStyle::CamelCase,
                            );

                            match (maybe_function_name, &tokens[3]) {
                                (
                                    Some(function_name),
                                    JackToken::Symbol(JackSymbol::OpenRoundBracket),
                                ) => {
                                    let expr = JackExpressions::extract_from_round_bracket(
                                        &mut tokens[4..],
                                    );
                                    let size = expr.size + 5;
                                    Self {
                                        payload: JackTermPayload::FunctionCall(
                                            new_ident,
                                            function_name,
                                            expr,
                                        ),
                                        size,
                                    }
                                }
                                _ => unimplemented!(),
                            }
                        }
                        _ => Self::new_ident(new_ident),
                    }
                } else {
                    panic!("Not valid term")
                }
            }
        }
    }
}

mod tests {
    #![allow(unused_imports, dead_code)]
    use futures::StreamExt;

    use crate::tokens::{JackIdent, JackInt, JackTokenizer};

    use super::*;

    #[tokio::test]
    async fn function_call_test() {
        let mut data = bytes_to_tokens(b"Main.calc(12 + (46 + c), \"hi\")").await;
        let l = data.len();
        let term = JackTerm::new(&mut data, l);

        assert_eq!(term.size, l);

        if let JackTermPayload::FunctionCall(class_name, function_name, _expressions) = term.payload
        {
            assert_eq!(class_name, JackVariableName(b"Main".to_vec()));
            assert_eq!(function_name, JackVariableName(b"calc".to_vec()));
        } else {
            panic!("wrong type")
        }
    }

    #[tokio::test]
    async fn function_call_zero_args_test() {
        let mut data = bytes_to_tokens(b"Main.calc()").await;
        let l = data.len();
        let term = JackTerm::new(&mut data, l);

        assert_eq!(term.size, l);

        if let JackTermPayload::FunctionCall(class_name, function_name, _expressions) = term.payload
        {
            assert_eq!(class_name, JackVariableName(b"Main".to_vec()));
            assert_eq!(function_name, JackVariableName(b"calc".to_vec()));
        } else {
            panic!("wrong type")
        }
    }

    #[tokio::test]
    async fn function_call_one_arg_test() {
        let mut data = bytes_to_tokens(b"Main.calc(5)").await;
        let l = data.len();
        let term = JackTerm::new(&mut data, l);

        assert_eq!(term.size, l);

        if let JackTermPayload::FunctionCall(class_name, function_name, _expressions) = term.payload
        {
            assert_eq!(class_name, JackVariableName(b"Main".to_vec()));
            assert_eq!(function_name, JackVariableName(b"calc".to_vec()));
        } else {
            panic!("wrong type")
        }
    }

    #[tokio::test]
    async fn method_call_test() {
        let mut data = bytes_to_tokens(b"calc(5, (1488))").await;
        let l = data.len();
        let term = JackTerm::new(&mut data, l);

        assert_eq!(term.size, l);

        if let JackTermPayload::MethodCall(function_name, _expressions) = term.payload {
            assert_eq!(function_name, JackVariableName(b"calc".to_vec()));
        } else {
            panic!("wrong type")
        }
    }

    #[tokio::test]
    async fn array_call_test() {
        let mut data = bytes_to_tokens(b"myArray[450 - (l / 9)]").await;
        let l = data.len();
        let term = JackTerm::new(&mut data, l);

        assert_eq!(term.size, l);

        if let JackTermPayload::ArrayElem(function_name, _expression) = term.payload {
            assert_eq!(function_name, JackVariableName(b"myArray".to_vec()));
        } else {
            panic!("wrong type")
        }
    }

    async fn bytes_to_tokens(expr: &'static [u8]) -> Vec<JackToken> {
        let tokenizer = JackTokenizer::from_slice(expr, true);
        tokenizer
            .map(|x| x.payload)
            .collect::<Vec<JackToken>>()
            .await
    }
}
