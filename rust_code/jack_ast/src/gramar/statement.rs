use std::mem;

use crate::tokens::{JackKeyword, JackToken};

use super::{
    ast_elem::{JackAstElem, UnknownAstElem},
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    do_statement::{DoDeclaration, DoDeclarationtData},
    if_statement::{IfStatement, IfStatementData},
    let_statement::{Let, LettData},
    return_statement::{ReturnStatement, ReturnStatementData},
    while_statement::{WhileStatement, WhileStatementData},
};

enum StatementKind {
    Let(JackAstElem<Let, LettData>),
    Return(JackAstElem<ReturnStatement, ReturnStatementData>),
    Do(JackAstElem<DoDeclaration, DoDeclarationtData>),
    If(JackAstElem<IfStatement, IfStatementData>),
    While(JackAstElem<WhileStatement, WhileStatementData>),
}

impl StatementKind {
    pub fn feed(&mut self, token: JackToken) -> bool {
        match self {
            Self::Let(l) => {
                l.feed(token);
                l.is_ready
            }
            Self::Return(l) => {
                l.feed(token);
                l.is_ready
            }
            Self::Do(l) => {
                l.feed(token);
                l.is_ready
            }
            Self::If(l) => {
                l.feed(token);
                l.is_ready
            }
            Self::While(l) => {
                l.feed(token);
                l.is_ready
            }
        }
    }

    pub unsafe fn extract_ast_elem(self) -> JackAstElem<UnknownAstElem> {
        match self {
            Self::Let(l) => l.cast_to_unknown(),
            Self::Return(l) => l.cast_to_unknown(),
            Self::Do(l) => l.cast_to_unknown(),
            Self::If(l) => l.cast_to_unknown(),
            Self::While(l) => l.cast_to_unknown(),
        }
    }
}

#[derive(Default)]
pub struct StatementData {
    statement: Option<StatementKind>,
}
pub struct Statement;

impl JackAstElem<Statement, StatementData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (&token, self.data.statement.as_mut()) {
            (JackToken::Keyword(JackKeyword::Let), None) => {
                self.data.statement = Some(StatementKind::Let(JackAstElem::default()))
            }
            (JackToken::Keyword(JackKeyword::Return), None) => {
                self.data.statement = Some(StatementKind::Return(JackAstElem::default()))
            }
            (JackToken::Keyword(JackKeyword::Do), None) => {
                self.data.statement = Some(StatementKind::Do(JackAstElem::default()))
            }
            (JackToken::Keyword(JackKeyword::If), None) => {
                self.data.statement = Some(StatementKind::If(JackAstElem::default()))
            }
            (JackToken::Keyword(JackKeyword::While), None) => {
                self.data.statement = Some(StatementKind::While(JackAstElem::default()))
            }
            (_, Some(StatementKind::If(s))) => {
                s.feed(token);
                self.is_ready = s.is_ready;
                return;
            }
            (_, Some(statement)) => {
                self.is_ready = statement.feed(token);
                self.statement_to_ast();
                return;
            }
            _ => {
                self.is_error = true;
            }
        }

        if self.is_error {
            return;
        }

        self.data.statement.as_mut().unwrap().feed(token);
    }

    pub fn feed_token(&self, token: &JackToken) -> bool {
        if !self.is_ready {
            return true;
        }

        match (&self.data.statement, token) {
            (Some(StatementKind::If(_)), JackToken::Keyword(JackKeyword::Else)) => true,
            _ => false,
        }
    }

    pub fn terminate(&mut self) {
        self.statement_to_ast()
    }

    fn statement_to_ast(&mut self) {
        if self.is_ready && self.data.statement.is_some() {
            let mut statement_new: Option<StatementKind> = None;
            mem::swap(&mut statement_new, &mut self.data.statement);
            let target_statemenet = unsafe { statement_new.unwrap().extract_ast_elem() };
            unsafe { self.push_ast(target_statemenet) }
        }
    }
}

impl IntoJackAstKind for Statement {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::Statement
    }
}
