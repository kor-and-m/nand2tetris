use hack_ast::*;
use hack_macro::instruction;

use vm_parser::tokens::ArithmeticToken;

use super::{constants::POP_INSTRUCTIONS, model::Translator};

const POINT_STACK_VALUE: [Instruction<'static>; 2] = [instruction!(b"@SP"), instruction!(b"A=M-1")];

pub fn translate_arithmetic_token<'a, 'b>(
    translator: &'a mut Translator<'b>,
    token: &'a ArithmeticToken,
    factory: &'a mut VariableFactory<'b>,
) {
    match token {
        ArithmeticToken::Neg => {
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=-M"));
        }
        ArithmeticToken::Not => {
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=!M"));
        }
        ArithmeticToken::Add => {
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=D+M"));
        }
        ArithmeticToken::Sub => {
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=M-D"));
        }
        ArithmeticToken::Or => {
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=D|M"));
        }
        ArithmeticToken::And => {
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=D&M"));
        }
        t => {
            let (label_true, var_true, label_false, var_false) = factory.new_bool_variables();
            translator.save_link(&POP_INSTRUCTIONS);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"D=M-D"));
            translator.save_instruction(var_true);
            translator.save_instruction(jump_instruction_by_token(t));
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=0"));
            translator.save_instruction(var_false);
            translator.save_instruction(instruction!(b"0;JMP"));
            translator.save_instruction(label_true);
            translator.save_link(&POINT_STACK_VALUE);
            translator.save_instruction(instruction!(b"M=-1"));
            translator.save_instruction(label_false);
        }
    }
}

fn jump_instruction_by_token(token: &ArithmeticToken) -> Instruction<'static> {
    match token {
        ArithmeticToken::Eq => instruction!(b"D;JEQ"),
        ArithmeticToken::Gt => instruction!(b"D;JGT"),
        ArithmeticToken::Lt => instruction!(b"D;JLT"),
        _ => unreachable!(),
    }
}
