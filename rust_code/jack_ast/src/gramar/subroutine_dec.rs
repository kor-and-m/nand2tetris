use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
    params_list::ParameterList,
    subroutine_body::{SubroutineBody, SubroutineBodyData},
};

#[derive(Default, Clone, Copy)]
enum SubroutineDeclarationStage {
    #[default]
    AwaitSubroutineKind,
    AwaitSubroutineReturnType,
    AwaitSubroutineIdent,
    AwaitOpenRoundBracket,
    AwaitCloseRoundBracket,
    FillSubroutineBody,
}

#[derive(Default)]
pub struct SubroutineDeclarationData {
    stage: SubroutineDeclarationStage,
    params: Option<JackAstElem<ParameterList>>,
    body: Option<JackAstElem<SubroutineBody, SubroutineBodyData>>,
}
pub struct SubroutineDeclaration;

impl JackAstElem<SubroutineDeclaration, SubroutineDeclarationData> {
    pub fn feed(&mut self, token: JackToken) {
        if self.is_error {
            return;
        }

        match (self.data.stage, &token) {
            (
                SubroutineDeclarationStage::AwaitSubroutineKind,
                JackToken::Keyword(subroutine_type),
            ) => {
                self.is_error = match subroutine_type {
                    JackKeyword::Function => false,
                    JackKeyword::Constructor => false,
                    JackKeyword::Method => false,
                    _ => true,
                };
                self.push_token(token);
                self.data.stage = SubroutineDeclarationStage::AwaitSubroutineReturnType;
            }
            (
                SubroutineDeclarationStage::AwaitSubroutineReturnType,
                JackToken::Keyword(subroutine_type),
            ) => {
                self.is_error = match subroutine_type {
                    JackKeyword::Void => false,
                    JackKeyword::Int => false,
                    JackKeyword::Char => false,
                    JackKeyword::Boolean => false,
                    _ => true,
                };
                self.push_token(token);
                self.data.stage = SubroutineDeclarationStage::AwaitSubroutineIdent;
            }
            (SubroutineDeclarationStage::AwaitSubroutineReturnType, JackToken::Ident(_)) => {
                self.push_token(token);
                self.data.stage = SubroutineDeclarationStage::AwaitSubroutineIdent;
            }
            (SubroutineDeclarationStage::AwaitSubroutineIdent, JackToken::Ident(_)) => {
                self.push_token(token);
                self.data.stage = SubroutineDeclarationStage::AwaitOpenRoundBracket;
            }
            (
                SubroutineDeclarationStage::AwaitOpenRoundBracket,
                JackToken::Symbol(JackSymbol::OpenRoundBracket),
            ) => {
                self.data.params = Some(JackAstElem::default());
                self.data.params.as_mut().unwrap().is_ready = true;
                self.push_token(token);
                self.data.stage = SubroutineDeclarationStage::AwaitCloseRoundBracket;
            }
            (
                SubroutineDeclarationStage::AwaitCloseRoundBracket,
                JackToken::Symbol(JackSymbol::CloseRoundBracket),
            ) => {
                self.is_error = self.data.params.as_mut().unwrap().is_error
                    || !self.data.params.as_mut().unwrap().is_ready;
                let elem = JackAstElem::from_option(&mut self.data.params);
                unsafe { self.push_ast(elem.unwrap()) }
                self.push_token(token);
                self.data.body = Some(JackAstElem::default());
                self.data.stage = SubroutineDeclarationStage::FillSubroutineBody;
            }
            (SubroutineDeclarationStage::AwaitCloseRoundBracket, _) => {
                self.data.params.as_mut().unwrap().feed(token);
            }
            (SubroutineDeclarationStage::FillSubroutineBody, _) => {
                self.data.body.as_mut().unwrap().feed(token);
                self.is_ready = self.data.body.as_ref().unwrap().is_ready;
                if self.is_ready {
                    let elem = JackAstElem::from_option(&mut self.data.body);
                    unsafe { self.push_ast(elem.unwrap()) }
                }
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for SubroutineDeclaration {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::SubroutineDeclaration
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{
        tokens::{JackIdent, JackInt, JackString},
        xml::IntoXML,
    };

    use super::*;

    #[tokio::test]
    async fn test_function() {
        let list_tokens = vec![vec![
            JackToken::Keyword(JackKeyword::Function),
            JackToken::Keyword(JackKeyword::Void),
            JackToken::Ident(JackIdent(b"main".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Ident(JackIdent(b"Array".to_vec())),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Keyword(JackKeyword::Int),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Var),
            JackToken::Keyword(JackKeyword::Int),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Comma),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
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
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Array".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"new".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::While),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Less),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::OpenSquareBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::CloseSquareBracket),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"Keyboard".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"readInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"ENTER THE NEXT NUMBER: ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::IntLiteral(JackInt(b"1".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::IntLiteral(JackInt(b"0".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::While),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Less),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::OpenCurlyBracket),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::Ident(JackIdent(b"a".to_vec())),
            JackToken::Symbol(JackSymbol::OpenSquareBracket),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::CloseSquareBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Let),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Eq),
            JackToken::Ident(JackIdent(b"i".to_vec())),
            JackToken::Symbol(JackSymbol::Plus),
            JackToken::IntLiteral(JackInt(b"1".to_vec())),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"printString".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::StringLiteral(JackString(b"THE AVERAGE IS: ".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"printInt".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Ident(JackIdent(b"sum".to_vec())),
            JackToken::Symbol(JackSymbol::Divide),
            JackToken::Ident(JackIdent(b"length".to_vec())),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Do),
            JackToken::Ident(JackIdent(b"Output".to_vec())),
            JackToken::Symbol(JackSymbol::Period),
            JackToken::Ident(JackIdent(b"println".to_vec())),
            JackToken::Symbol(JackSymbol::OpenRoundBracket),
            JackToken::Symbol(JackSymbol::CloseRoundBracket),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Keyword(JackKeyword::Return),
            JackToken::Symbol(JackSymbol::Semicolon),
            JackToken::Symbol(JackSymbol::CloseCurlyBracket),
        ]];

        {
            let mut file = File::create("priv/test/gen/function.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<SubroutineDeclaration, SubroutineDeclarationData> =
                    JackAstElem::default();
                for token in tokens {
                    new_elem.feed(token);
                    assert!(!new_elem.is_error);
                }

                assert!(new_elem.is_ready);

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/function.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/function.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
