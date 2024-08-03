#[derive(Debug)]
pub enum JackAstElemKind {
    Var,
    Let,
    Term,
    Expression,
    ExpressionList,
    FunctionCall,
    ArrayIndex,
    ReturnStatement,
    Statement,
    Do,
    Statements,
    ExpressionInBreackets,
    If,
    While,
    SubroutineBody,
    ParameterList,
    SubroutineDeclaration,
    ClassVar,
    ClassDeclaration,
}

impl JackAstElemKind {
    pub fn get_tag_name(&self) -> Option<&'static [u8]> {
        match self {
            Self::Var => Some(b"varDec"),
            Self::Term => Some(b"term"),
            Self::Expression => Some(b"expression"),
            Self::ExpressionList => Some(b"expressionList"),
            Self::FunctionCall => None,
            Self::ArrayIndex => None,
            Self::ReturnStatement => Some(b"returnStatement"),
            Self::Let => Some(b"letStatement"),
            Self::Do => Some(b"doStatement"),
            Self::Statement => None,
            Self::Statements => Some(b"statements"),
            Self::ExpressionInBreackets => None,
            Self::If => Some(b"ifStatement"),
            Self::While => Some(b"whileStatement"),
            Self::SubroutineBody => Some(b"subroutineBody"),
            Self::ParameterList => Some(b"parameterList"),
            Self::SubroutineDeclaration => Some(b"subroutineDec"),
            Self::ClassVar => Some(b"classVarDec"),
            Self::ClassDeclaration => Some(b"class"),
        }
    }
}

pub trait IntoJackAstKind {
    fn kind() -> JackAstElemKind;
}
