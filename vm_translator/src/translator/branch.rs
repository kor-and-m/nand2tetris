use std::mem;

use hack_ast::*;
use hack_macro::instruction;
use vm_tokens::{BranchToken, BranchTokenKind};

use super::{constants::POP_INSTRUCTIONS, Translator};

pub fn translate_branch_token<'a, 'b>(
    translator: &'a mut Translator<'b>,
    token: &'a mut BranchToken,
    _factory: &'a mut VariableFactory<'b>,
) {
    match token.kind {
        BranchTokenKind::Label => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_instruction(Instruction::new_raw_label(t));
        }
        BranchTokenKind::Goto => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_instruction(Instruction::new_raw_var_label(t));
            translator.save_instruction(instruction!(b"0;JMP"));
        }
        BranchTokenKind::IfGoto => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_instruction(Instruction::new_raw_var_label(t));
            translator.save_instruction(instruction!(b"0;JNE"));
        }
    }
}
