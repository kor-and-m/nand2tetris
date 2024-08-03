use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    class_var::ClassVarDeclaration,
    subroutine_dec::{SubroutineDeclaration, SubroutineDeclarationData},
};

#[derive(Default, Clone, Copy)]
enum ClassDeclarationStage {
    #[default]
    AwaitClassKeyword,
    AwaitClassIdent,
    AwaitOpenCurleyBraket,
    FillVarDeclaration,
    FillSubroutineDeclaration,
    AwaitCloseCurleyBraket,
}

#[derive(Default)]
pub struct ClassDeclarationData {
    stage: ClassDeclarationStage,
    var: Option<JackAstElem<ClassVarDeclaration>>,
    function: Option<JackAstElem<SubroutineDeclaration, SubroutineDeclarationData>>,
}
pub struct ClassDeclaration;

impl JackAstElem<ClassDeclaration, ClassDeclarationData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.data.stage, &token) {
            (ClassDeclarationStage::AwaitClassKeyword, JackToken::Keyword(JackKeyword::Class)) => {
                self.push_token(token);
                self.data.stage = ClassDeclarationStage::AwaitClassIdent;
            }
            (ClassDeclarationStage::AwaitClassIdent, JackToken::Ident(_)) => {
                self.push_token(token);
                self.data.stage = ClassDeclarationStage::AwaitOpenCurleyBraket;
            }
            (
                ClassDeclarationStage::AwaitOpenCurleyBraket,
                JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            ) => {
                self.push_token(token);
                self.data.stage = ClassDeclarationStage::AwaitCloseCurleyBraket;
            }
            (
                ClassDeclarationStage::AwaitCloseCurleyBraket,
                JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            ) => {
                self.push_token(token);
                self.is_ready = true;
                self.data.stage = ClassDeclarationStage::AwaitCloseCurleyBraket;
            }
            (ClassDeclarationStage::AwaitCloseCurleyBraket, JackToken::Keyword(keyword)) => {
                if keyword == &JackKeyword::Static || keyword == &JackKeyword::Field {
                    self.data.var = Some(JackAstElem::default());
                    self.data.var.as_mut().unwrap().feed(token);
                    self.is_error = self.data.var.as_ref().unwrap().is_error;
                    self.data.stage = ClassDeclarationStage::FillVarDeclaration;
                } else {
                    self.data.function = Some(JackAstElem::default());
                    self.data.function.as_mut().unwrap().feed(token);
                    self.is_error = self.data.function.as_ref().unwrap().is_error;
                    self.data.stage = ClassDeclarationStage::FillSubroutineDeclaration;
                }
            }
            (ClassDeclarationStage::FillVarDeclaration, _) => {
                self.data.var.as_mut().unwrap().feed(token);
                self.is_error = self.data.var.as_ref().unwrap().is_error;
                if self.data.var.as_ref().unwrap().is_ready {
                    let elem = JackAstElem::from_option(&mut self.data.var);
                    unsafe { self.push_ast(elem.unwrap()) };
                    self.data.stage = ClassDeclarationStage::AwaitCloseCurleyBraket;
                }
            }
            (ClassDeclarationStage::FillSubroutineDeclaration, _) => {
                self.data.function.as_mut().unwrap().feed(token);
                self.is_error = self.data.function.as_ref().unwrap().is_error;
                if self.data.function.as_ref().unwrap().is_ready {
                    let elem = JackAstElem::from_option(&mut self.data.function);
                    unsafe { self.push_ast(elem.unwrap()) };
                    self.data.stage = ClassDeclarationStage::AwaitCloseCurleyBraket;
                }
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for ClassDeclaration {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ClassDeclaration
    }
}
