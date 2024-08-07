use crate::gramar::tag::{JackAstTag, JackAstTagLabel, JackAstTagResult};
use crate::tokens::{JackKeyword, JackToken};

use super::declar::DeclarTag;
use super::statements::StatementsTag;

#[derive(Default, PartialEq)]
enum StatementsDeclarTagStage {
    #[default]
    AwaitVarOrStatements,
    Terminate,
}

#[derive(Default)]
pub struct StatementsDeclarTag {
    kind: StatementsDeclarTagStage,
}

impl JackAstTag for StatementsDeclarTag {
    fn is_consistent(&self) -> bool {
        self.kind == StatementsDeclarTagStage::Terminate
    }

    fn get_label(&self) -> JackAstTagLabel {
        JackAstTagLabel::StatementsDeclar
    }

    fn feed_token<'a>(&'a mut self, token: &JackToken) -> JackAstTagResult {
        match (token, &self.kind) {
            (
                JackToken::Keyword(JackKeyword::Var),
                StatementsDeclarTagStage::AwaitVarOrStatements,
            ) => {
                let expression = Box::new(DeclarTag::new_function());
                JackAstTagResult::Move(expression)
            }
            (_, StatementsDeclarTagStage::AwaitVarOrStatements) => {
                self.kind = StatementsDeclarTagStage::Terminate;
                let expressions = Box::new(StatementsTag::default());
                JackAstTagResult::Move(expressions)
            }
            _ => JackAstTagResult::Error,
        }
    }
}
