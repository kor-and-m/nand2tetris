use std::mem;

use crate::tokens::{JackSymbol, JackToken};

use super::{
    array_index::{ArrayIndex, ArrayIndexData},
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    expression_in_breackets::{ExpressionInBreackets, ExpressionInBreacketsData},
    function_call::{FunctionCall, FunctionCallData},
};

#[derive(Default, Clone, Copy)]
enum TermType {
    #[default]
    Unknown,
    IdentType,
    SimpleType,
    ExpressionInBrackets,
    FunctionCall,
    ArrayElement,
}

#[derive(Default)]
pub struct TermData {
    kind: TermType,
    ident: Option<JackToken>,
    expression: Option<JackAstElem<ExpressionInBreackets, ExpressionInBreacketsData>>,
    function_call: Option<JackAstElem<FunctionCall, FunctionCallData>>,
    array_element: Option<JackAstElem<ArrayIndex, ArrayIndexData>>,
}

pub struct Term;

impl JackAstElem<Term, TermData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        let kind = self.data.kind;
        match (&token, kind) {
            (JackToken::Symbol(s), TermType::Unknown) if s.is_unary_op() => {
                self.push_token(token);
            }
            (JackToken::Ident(_), TermType::Unknown) => {
                self.data.ident = Some(token);
                self.data.kind = TermType::IdentType;
                self.is_ready = true;
            }
            (JackToken::IntLiteral(_), TermType::Unknown) => {
                self.push_token(token);
                self.data.kind = TermType::SimpleType;
                self.is_ready = true;
            }
            (JackToken::Keyword(keyword), TermType::Unknown) if keyword.is_value() => {
                self.push_token(token);
                self.data.kind = TermType::SimpleType;
                self.is_ready = true;
            }
            (JackToken::StringLiteral(_), TermType::Unknown) => {
                self.push_token(token);
                self.data.kind = TermType::SimpleType;
                self.is_ready = true;
            }
            (JackToken::Symbol(JackSymbol::OpenRoundBracket), TermType::Unknown) => {
                self.push_token(token);
                self.data.kind = TermType::ExpressionInBrackets;
                let expression = JackAstElem::default();
                self.data.expression = Some(expression);
            }
            (_, TermType::ExpressionInBrackets) => {
                self.data.expression.as_mut().unwrap().feed(token);
            }
            (JackToken::Symbol(JackSymbol::OpenSquareBracket), TermType::IdentType) => {
                let mut array_index: JackAstElem<ArrayIndex, ArrayIndexData> =
                    JackAstElem::default();
                let mut maybe_token = None;
                mem::swap(&mut maybe_token, &mut self.data.ident);
                self.push_token(maybe_token.unwrap());
                array_index.feed(token);
                self.data.kind = TermType::ArrayElement;
                self.data.array_element = Some(array_index);
            }
            (_, TermType::ArrayElement) => {
                self.data.array_element.as_mut().unwrap().feed(token);
                self.is_ready = self.data.array_element.as_ref().unwrap().is_ready;
                if self.is_ready {
                    let array_element = JackAstElem::from_option(&mut self.data.array_element);
                    unsafe { self.push_ast(array_element.unwrap()) };
                }
            }
            (JackToken::Symbol(JackSymbol::Period), TermType::IdentType) => {
                let mut function_call: JackAstElem<FunctionCall, FunctionCallData> =
                    JackAstElem::default();
                let mut maybe_token = None;
                mem::swap(&mut maybe_token, &mut self.data.ident);
                function_call.feed(maybe_token.unwrap());
                function_call.feed(token);
                self.data.kind = TermType::FunctionCall;
                self.data.function_call = Some(function_call);
            }
            (_, TermType::FunctionCall) => {
                self.data.function_call.as_mut().unwrap().feed(token);
                self.is_ready = self.data.function_call.as_ref().unwrap().is_ready;
                if self.is_ready {
                    let mut function_call = None;
                    mem::swap(&mut function_call, &mut self.data.function_call);
                    unsafe { self.push_ast(function_call.unwrap()) };
                }
            }
            _ => {
                self.is_error = true;
            }
        }
    }

    pub fn terminate(&mut self) {
        if self.is_error {
            return;
        }

        if self.data.ident.is_some() {
            let mut e: Option<JackToken> = None;
            mem::swap(&mut e, &mut self.data.ident);
            self.push_token(e.unwrap());
        }
    }
}

impl IntoJackAstKind for Term {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Term
    }
}
