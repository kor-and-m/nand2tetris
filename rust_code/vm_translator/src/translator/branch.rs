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

// ...
// // (MyFun.2)
// (MyFunLCL)
// define lcl
// execute body
// pop current val to (MyFunArgs)
// // returm
// ...
// ...
// (MyFunArgs)
// push MyFunArg 1
// push MyFunArg 2
// // MyFun(MyFunArg1, MyFunArg2)
// PUSH current frame to stack
// // ARG = MyFunReturnArgs
// // goto (MyFun.2)
// (SimpleFunction.test.2.return)
// @SP
// A=M
// D=M
// @ARG
// M=D
// @SP
// M=M-1

// @SP
// A=M
// D=M
// @LCL
// M=D
// @SP
// M=M-1

// @SP
// A=M
// D=M
// @THIS
// M=D
// @SP
// M=M-1

// @SP
// A=M
// D=M
// @THAT
// M=D
// @SP
// M=M-1

// (SimpleFunction.test.2)
// @0
// D=A
// // pop
// @SP
// A=M
// D=M
// @SP
// M=M+1
// @0
// D=A
// // pop
// @SP
// A=M
// M=D
// @SP
// M=M+1
// Body execute
// Pop ARG 0
// @SimpleFunction.test.2.return
// 0;JMP

// @ARG
// A=M
// D=M
// @2
// D=D+A
// @LCL
// M=D

// @ARG
// A=M
// D=M
// @3
// D=D+A
// @THIS
// M=D

// @ARG
// A=M
// D=M
// @4
// D=D+A
// @THAT
// M=D
