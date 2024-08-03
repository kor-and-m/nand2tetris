use crate::tokens::{JackKeyword, JackSymbol, JackToken};

use super::{
    ast_elem::JackAstElem,
    ast_kind::{IntoJackAstKind, JackAstElemKind},
};

pub struct ClassVarDeclaration;

impl JackAstElem<ClassVarDeclaration> {
    pub fn feed(&mut self, token: JackToken) {
        match (self.children_count(), &token) {
            (0, JackToken::Keyword(JackKeyword::Static)) => self.push_token(token),
            (0, JackToken::Keyword(JackKeyword::Field)) => self.push_token(token),
            (1, JackToken::Keyword(keyword))
                if keyword == &JackKeyword::Char
                    || keyword == &JackKeyword::Int
                    || keyword == &JackKeyword::Boolean =>
            {
                self.push_token(token)
            }
            (1, JackToken::Ident(_)) => self.push_token(token),
            (n, JackToken::Ident(_)) if n != 0 && n % 2 == 0 => self.push_token(token),
            (n, JackToken::Symbol(JackSymbol::Comma)) if n != 1 && n % 2 == 1 => {
                self.push_token(token)
            }
            (n, JackToken::Symbol(JackSymbol::Semicolon)) if n != 1 && n % 2 == 1 => {
                self.is_ready = true;
                self.push_token(token);
            }
            _ => {
                self.is_error = true;
            }
        }
    }
}

impl IntoJackAstKind for ClassVarDeclaration {
    fn kind() -> JackAstElemKind {
        JackAstElemKind::ClassVar
    }
}

#[cfg(test)]
mod tests {
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{tokens::JackIdent, xml::IntoXML};

    use super::*;

    #[tokio::test]
    async fn test_class_var_statement() {
        let list_tokens = vec![
            vec![
                JackToken::Keyword(JackKeyword::Field),
                JackToken::Ident(JackIdent(b"Array".to_vec())),
                JackToken::Ident(JackIdent(b"a".to_vec())),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
            vec![
                JackToken::Keyword(JackKeyword::Field),
                JackToken::Keyword(JackKeyword::Int),
                JackToken::Ident(JackIdent(b"length".to_vec())),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
            vec![
                JackToken::Keyword(JackKeyword::Static),
                JackToken::Keyword(JackKeyword::Boolean),
                JackToken::Ident(JackIdent(b"i".to_vec())),
                JackToken::Symbol(JackSymbol::Comma),
                JackToken::Ident(JackIdent(b"sum".to_vec())),
                JackToken::Symbol(JackSymbol::Semicolon),
            ],
        ];

        {
            let mut file = File::create("priv/test/gen/class_vars.xml").await.unwrap();

            for tokens in list_tokens {
                let mut new_elem: JackAstElem<ClassVarDeclaration> = JackAstElem::default();
                let mut l = tokens.len();
                for token in tokens {
                    new_elem.feed(token);
                    l -= 1;
                    assert!(!new_elem.is_error);

                    if l == 0 {
                        assert!(new_elem.is_ready);
                    } else {
                        assert!(!new_elem.is_ready);
                    }
                }

                new_elem.write_xml(&mut file).await.unwrap();
            }
        }

        let mut generated_buff = Vec::new();
        let mut cmp_buff = Vec::new();

        let mut generated_file = File::open("priv/test/gen/class_vars.xml").await.unwrap();
        let mut cmp_file = File::open("priv/test/cmp/class_vars.xml").await.unwrap();

        let generated_size = generated_file
            .read_to_end(&mut generated_buff)
            .await
            .unwrap();
        let cmp_size = cmp_file.read_to_end(&mut cmp_buff).await.unwrap();

        assert_eq!(generated_size, cmp_size);
        assert_eq!(generated_buff, cmp_buff);
    }
}
