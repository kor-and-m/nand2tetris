use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    statements::{Statements, StatementsData},
    var::VarDeclaration,
};

#[derive(Default, Clone, Copy)]
enum SubroutineBodyStage {
    #[default]
    AwaitCurleyBracket,
    AwaitVar,
    FillVar,
    FillStatements,
}

#[derive(Default)]
pub struct SubroutineBodyData {
    stage: SubroutineBodyStage,
    brackets: usize,
    var: Option<JackAstElem<VarDeclaration>>,
    statements: Option<JackAstElem<Statements, StatementsData>>,
}
pub struct SubroutineBody;

impl JackAstElem<SubroutineBody, SubroutineBodyData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error || self.is_ready {
            return;
        }

        let brackets = self.data.brackets;

        match (self.data.stage, &token) {
            (
                SubroutineBodyStage::AwaitCurleyBracket,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.push_token(token);
                self.data.stage = SubroutineBodyStage::AwaitVar;
                self.data.brackets = 1;
            }
            (SubroutineBodyStage::AwaitCurleyBracket, _) => {
                self.is_error = true;
            }
            (
                SubroutineBodyStage::FillStatements,
                JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            ) if brackets == 1 => {
                let mut child = JackAstElem::from_option(&mut self.data.statements);

                if child.is_none() {
                    self.is_error = false;
                    return;
                }

                child.as_mut().unwrap().terminate();
                self.is_ready = child.as_ref().unwrap().is_ready;
                self.is_error = !self.is_ready;
                unsafe { self.push_ast(child.unwrap()) };
                self.push_token(token);
            }
            (
                SubroutineBodyStage::FillStatements,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.data.brackets += 1;
                self.data.statements.as_mut().unwrap().feed(token);
            }
            (
                SubroutineBodyStage::FillStatements,
                JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            ) => {
                self.data.brackets -= 1;
                self.data.statements.as_mut().unwrap().feed(token);
            }
            (SubroutineBodyStage::AwaitVar, JackToken::Keyword(JackKeyword::Var)) => {
                if self.data.var.is_some() {
                    self.is_error = true;
                } else {
                    self.data.var = Some(JackAstElem::default());
                    self.data.var.as_mut().unwrap().feed(token);
                    self.data.stage = SubroutineBodyStage::FillVar;
                }
            }
            (SubroutineBodyStage::AwaitVar, _) => {
                if self.data.statements.is_some() {
                    unreachable!()
                } else {
                    self.data.statements = Some(JackAstElem::default());
                    self.data.statements.as_mut().unwrap().feed(token);
                    self.data.stage = SubroutineBodyStage::FillStatements;
                }
            }
            (SubroutineBodyStage::FillVar, _) => {
                self.data.var.as_mut().unwrap().feed(token);
                self.is_error = self.data.var.as_ref().unwrap().is_error;
                if self.data.var.as_ref().unwrap().is_ready {
                    self.data.stage = SubroutineBodyStage::AwaitVar;
                    let child = JackAstElem::from_option(&mut self.data.var);
                    unsafe { self.push_ast(child.unwrap()) }
                }
            }
            (SubroutineBodyStage::FillStatements, _) => {
                self.data.statements.as_mut().unwrap().feed(token);
            }
        }
    }
}

impl IntoJackAstKind for SubroutineBody {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::SubroutineBody
    }
}
