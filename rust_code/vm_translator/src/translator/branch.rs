use std::mem;

use hack_instructions::*;
use hack_macro::instruction;
use vm_parser::{AsmBranchInstruction, AsmBranchInstructionKind};

use super::{constants::POP_INSTRUCTIONS, Translator};

pub fn translate_branch_token<'a, 'b>(
    translator: &'a mut Translator<'b>,
    token: &'a mut AsmBranchInstruction,
    _factory: &'a mut VariableFactory<'b>,
) {
    match token.kind {
        AsmBranchInstructionKind::Label => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_instruction(Instruction::new_raw_label(t));
        }
        AsmBranchInstructionKind::Goto => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_instruction(Instruction::new_raw_var_label(t));
            translator.save_instruction(instruction!(b"0;JMP"));
        }
        AsmBranchInstructionKind::IfGoto => {
            let mut t = Vec::new();
            mem::swap(&mut token.name, &mut t);
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_instruction(Instruction::new_raw_var_label(t));
            translator.save_instruction(instruction!(b"D;JNE"));
        }
    }
}
