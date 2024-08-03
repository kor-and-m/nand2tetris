use std::mem;

use hack_instructions::*;
use hack_macro::instruction;
use vm_parser::{
    AsmBranchInstruction, AsmBranchInstructionKind, AsmFunctionInstruction, AsmMemoryInstruction,
    AsmMemoryInstructionKind, AsmMemoryInstructionSegment, FunctionMetadata,
};

use super::branch::translate_branch_token;
use super::constants::{POP_INSTRUCTIONS, PUSH_INSTRUCTIONS};
use super::memory::{segment_as_instruction, translate_memory_token};
use super::Translator;

const WRITE_ZERO_CONST_TO_D: [Instruction<'static>; 2] =
    [instruction!(b"@0"), instruction!(b"D=A")];

const SET_SP_TO_LCL: [Instruction<'static>; 5] = [
    instruction!(b"@SP"),
    instruction!(b"A=M"),
    instruction!(b"D=A"),
    instruction!(b"@LCL"),
    instruction!(b"M=D"),
];

const MOVE_SP_TO_COLLER: [Instruction<'static>; 4] = [
    instruction!(b"@LCL"),
    instruction!(b"D=M"),
    instruction!(b"@SP"),
    instruction!(b"M=D"),
];

const SAVE_PARENT_SP_TO_R13: [Instruction<'static>; 4] = [
    instruction!(b"@ARG"),
    instruction!(b"D=M+1"),
    instruction!(b"@R13"),
    instruction!(b"M=D"),
];

const RESTORE_PARENT_SP_FROM_R13: [Instruction<'static>; 4] = [
    instruction!(b"@R13"),
    instruction!(b"D=M"),
    instruction!(b"@SP"),
    instruction!(b"M=D"),
];

const SAVE_RETURN_TO_R14: [Instruction<'static>; 7] = [
    instruction!(b"@LCL"),
    instruction!(b"D=M"),
    instruction!(b"@5"),
    instruction!(b"A=D-A"),
    instruction!(b"D=M"),
    instruction!(b"@R14"),
    instruction!(b"M=D"),
];

const RETURN_FROM_R14: [Instruction<'static>; 3] = [
    instruction!(b"@R14"),
    instruction!(b"A=M"),
    instruction!(b"0;JMP"),
];

pub fn translate_function_token<'a, 'b>(
    translator: &'a mut Translator<'b>,
    token: &'a mut AsmFunctionInstruction,
    factory: &'a mut VariableFactory<'b>,
    instruction_id: usize,
) {
    match token {
        AsmFunctionInstruction::Call(meta) => {
            let b_tmp = instruction_id.to_string();
            let b = b_tmp.as_bytes();
            call(translator, b, meta, factory)
        }
        AsmFunctionInstruction::Definition(meta) => {
            let v = mem::replace(&mut meta.name, Vec::new());
            let mut label_token = AsmBranchInstruction {
                kind: AsmBranchInstructionKind::Label,
                name: v,
            };
            translator.save_instruction(Instruction::new_line());
            translator.save_instruction(Instruction::new_line());
            translate_branch_token(translator, &mut label_token, factory);
            translator.save_instruction(instruction!(b"// Push local variables"));
            for _i in 0..meta.args_count {
                translator.save_link(&WRITE_ZERO_CONST_TO_D);
                translator.save_link(&PUSH_INSTRUCTIONS);
            }
            translator.save_instruction(instruction!(b"// Execute body"));
        }
        AsmFunctionInstruction::Return => {
            translator.save_instruction(instruction!(
                b"// Save return address in case zero arguments"
            ));
            translator.save_link(&SAVE_RETURN_TO_R14);
            translator.save_instruction(instruction!(b"// Push value to zero ARG"));
            let memory_token = AsmMemoryInstruction {
                segment: AsmMemoryInstructionSegment::Arg,
                kind: AsmMemoryInstructionKind::Pop,
                val: 0,
            };
            translate_memory_token(translator, &memory_token, factory);
            translator.save_instruction(instruction!(b"// Move SP to restoring segments"));
            translator.save_link(&MOVE_SP_TO_COLLER);
            translator.save_link(&SAVE_PARENT_SP_TO_R13);
            translator.save_instruction(instruction!(b"// Restoring coller segemnts"));
            restore_context_elem(translator, AsmMemoryInstructionSegment::That);
            restore_context_elem(translator, AsmMemoryInstructionSegment::This);
            restore_context_elem(translator, AsmMemoryInstructionSegment::Arg);
            restore_context_elem(translator, AsmMemoryInstructionSegment::Local);
            translator.save_instruction(instruction!(b"// Move PC back"));
            translator.save_link(&RESTORE_PARENT_SP_FROM_R13);
            translator.save_link(&RETURN_FROM_R14);
            translator.save_instruction(instruction!(b"// Return back to caller function"));
        }
    }
}

fn call<'a, 'b>(
    translator: &'a mut Translator<'b>,
    call_id: &[u8],
    meta: &'a mut FunctionMetadata,
    factory: &'a mut VariableFactory<'b>,
) {
    let mut name = meta.name.clone();
    let function_name = name.clone();
    name.push(b'.');
    for i in factory.prefix {
        name.push(*i)
    }
    name.push(b'.');
    for i in call_id {
        name.push(*i)
    }

    let caller_name = name.clone();

    let mut label_token = AsmBranchInstruction {
        kind: AsmBranchInstructionKind::Label,
        name,
    };

    let mut goto_function_token = AsmBranchInstruction {
        kind: AsmBranchInstructionKind::Goto,
        name: function_name,
    };
    translator.save_instruction(instruction!(b"// Call function"));
    translator.save_instruction(instruction!(b"// Saving current segemnts"));
    push_label_elem(translator, caller_name);
    push_context_elem(translator, AsmMemoryInstructionSegment::Local);
    push_context_elem(translator, AsmMemoryInstructionSegment::Arg);
    push_context_elem(translator, AsmMemoryInstructionSegment::This);
    push_context_elem(translator, AsmMemoryInstructionSegment::That);
    translator.save_instruction(instruction!(b"// Saved current segemnts"));
    translator.save_instruction(instruction!(b"// Set ARG pointer"));
    set_arg_for_collee(translator, meta.args_count);
    translator.save_instruction(instruction!(b"// Set LCL = SP"));
    translator.save_link(&SET_SP_TO_LCL);
    translator.save_instruction(instruction!(b"// Goto function body"));
    translate_branch_token(translator, &mut goto_function_token, factory);
    translate_branch_token(translator, &mut label_token, factory);
    translator.save_instruction(instruction!(b"// Here we return after function execution"));
    translator.save_instruction(instruction!(b"// End function call"));
}

fn set_arg_for_collee<'a, 'b>(translator: &'a mut Translator<'b>, args_count: i16) {
    translator.save_instruction(instruction!(b"@SP"));
    translator.save_instruction(instruction!(b"D=M"));
    translator.save_instruction(instruction!(b"@5"));
    translator.save_instruction(instruction!(b"D=D-A"));
    translator.save_instruction(Instruction::new_number(args_count));
    translator.save_instruction(instruction!(b"D=D-A"));
    translator.save_instruction(instruction!(b"@ARG"));
    translator.save_instruction(instruction!(b"M=D"));
}

fn push_label_elem<'a, 'b>(translator: &'a mut Translator<'b>, function_name: Vec<u8>) {
    translator.save_instruction(Instruction::new_raw_var_label(function_name));
    translator.save_instruction(instruction!(b"D=A"));
    translator.save_link(&PUSH_INSTRUCTIONS);
}

fn push_context_elem<'a, 'b>(
    translator: &'a mut Translator<'b>,
    segement: AsmMemoryInstructionSegment,
) {
    translator.save_instruction(segment_as_instruction(segement));
    translator.save_instruction(instruction!(b"D=M"));
    translator.save_link(&PUSH_INSTRUCTIONS);
}

fn restore_context_elem<'a, 'b>(
    translator: &'a mut Translator<'b>,
    segement: AsmMemoryInstructionSegment,
) {
    translator.save_link(&POP_INSTRUCTIONS);
    translator.save_instruction(segment_as_instruction(segement));
    translator.save_instruction(instruction!(b"M=D"));
}
