use file_context::FileContext;
use futures::{Stream, StreamExt};

use crate::{gramar::ast::JackClass, tokens::JackToken};

use super::{
    behaviour::{JackAstBuilder, JackAstBuilderResponse},
    class::JackClassBuilder,
};

pub struct JackASTBuilderEngine<'a, S: Stream<Item = FileContext<JackToken>> + Unpin> {
    stream: &'a mut S,
    scopes: Vec<*mut dyn JackAstBuilder>,
    class: Box<JackClassBuilder>,
}

impl<'a, S> JackASTBuilderEngine<'a, S>
where
    S: Stream<Item = FileContext<JackToken>> + Unpin,
{
    pub fn new(stream: &'a mut S) -> Self {
        let scopes = Vec::new();
        let class = Box::new(JackClassBuilder::default());

        let mut state = Self {
            stream,
            scopes,
            class,
        };

        state.set_pointer();

        state
    }

    fn set_pointer(&mut self) {
        let class_link = self.class.as_mut() as *mut dyn JackAstBuilder;

        self.scopes.push(class_link);
    }

    pub fn get_class(self) -> JackClass {
        self.class.build()
    }

    pub async fn step(&mut self) {
        if let Some(token) = self.stream.next().await {
            if let Some(scope) = self.scopes.pop() {
                self.feed_scope(token, scope)
            } else {
                panic!("Out of scopes")
            }
        }
    }

    pub async fn build_class(mut self) -> JackClass {
        while !self.class.is_ready() {
            self.step().await;
        }

        self.get_class()
    }

    fn feed_scope(
        &mut self,
        mut token: FileContext<JackToken>,
        unsafe_scope: *mut dyn JackAstBuilder,
    ) {
        let scope = unsafe { &mut *unsafe_scope };
        match scope.feed(&mut token) {
            Err(err) => {
                println!("{:?}", err);
                unimplemented!()
            }
            Ok(JackAstBuilderResponse::Continue) => self.scopes.push(scope),
            Ok(JackAstBuilderResponse::Ready) => (),
            Ok(JackAstBuilderResponse::MoveParent) => {
                if let Some(scope) = self.scopes.pop() {
                    self.feed_scope(token, scope)
                } else {
                    panic!("Parent not found")
                }
            }
            Ok(JackAstBuilderResponse::Move(new_scope)) => {
                self.scopes.push(scope);
                self.feed_scope(token, new_scope)
            }
        }
    }
}

mod tests {
    #![allow(unused_imports, dead_code)]
    use futures::StreamExt;

    use crate::gramar::ast::{
        JackDeclaration, JackExpression, JackLet, JackReturn, JackStatement, JackStatements,
        JackSubroutine, JackTerm, JackTermPayload,
    };
    use crate::gramar::units::*;

    use crate::tokens::{JackIdent, JackInt, JackTokenizer};

    use super::*;

    #[tokio::test]
    async fn simple_class_test() {
        let mut tokenizer = JackTokenizer::from_slice(
            b"class Main {
                field Array o, p;
                field String ll;
                static int CONST_VARIABLE;
                
                function void gg(int tt, String n) {
                    var Helper helper;

                    let k[10] = 12;
                    return;
                }
            }",
            true,
        );
        let ast_engine = JackASTBuilderEngine::new(&mut tokenizer);
        let class = ast_engine.build_class().await;

        let var1 = JackDeclaration {
            names: vec![
                JackVariableName(b"o".to_vec()),
                JackVariableName(b"p".to_vec()),
            ],
            kind: JackType::Basic(JackBasicType::Arr),
            segment: JackSegment::Field,
        };
        let var2 = JackDeclaration {
            names: vec![JackVariableName(b"ll".to_vec())],
            kind: JackType::Basic(JackBasicType::String),
            segment: JackSegment::Field,
        };
        let var3 = JackDeclaration {
            names: vec![JackVariableName(b"CONST_VARIABLE".to_vec())],
            kind: JackType::Basic(JackBasicType::Int),
            segment: JackSegment::Static,
        };

        let statement_variable = JackTerm {
            size: 4,
            payload: JackTermPayload::ArrayElem(
                JackVariableName(b"k".to_vec()),
                Box::new(JackExpression {
                    term: JackTerm::new_int(JackInt(b"10".to_vec())),
                    size: 1,
                    extra: vec![],
                }),
            ),
        };

        let statement_expression = JackExpression {
            term: JackTerm::new_int(JackInt(b"12".to_vec())),
            size: 1,
            extra: vec![],
        };

        let statement = JackLet {
            variable: statement_variable,
            expression: statement_expression,
        };

        let statements = JackStatements(vec![
            Box::new(JackStatement::Let(statement)),
            Box::new(JackStatement::Return(JackReturn::default())),
        ]);
        let vars = vec![var1, var2, var3];

        let local_var = JackDeclaration {
            names: vec![JackVariableName(b"helper".to_vec())],
            kind: JackType::Class(b"Helper".to_vec()),
            segment: JackSegment::Lcl,
        };

        let arg1 = JackDeclaration {
            names: vec![JackVariableName(b"tt".to_vec())],
            kind: JackType::Basic(JackBasicType::Int),
            segment: JackSegment::Arg,
        };

        let arg2 = JackDeclaration {
            names: vec![JackVariableName(b"n".to_vec())],
            kind: JackType::Basic(JackBasicType::String),
            segment: JackSegment::Arg,
        };

        let subroutine = JackSubroutine {
            name: JackVariableName(b"gg".to_vec()),
            kind: JackType::Basic(JackBasicType::Void),
            key: JackSubroutineType::Function,
            vars: vec![arg1, arg2, local_var],
            statements,
        };

        let expected_class = JackClass {
            name: JackVariableName(b"Main".to_vec()),
            vars,
            subroutines: vec![subroutine],
        };

        assert_eq!(expected_class, class)
    }
}
