use hack_instructions::*;
use hack_macro::instruction;

pub const POP_INSTRUCTIONS: [Instruction<'static>; 7] = [
    instruction!(b"// Decriment SP"),
    instruction!(b"@SP"),
    instruction!(b"M=M-1"),
    instruction!(b"// Extract value from SP to D"),
    instruction!(b"@SP"),
    instruction!(b"A=M"),
    instruction!(b"D=M"),
];

pub const PUSH_INSTRUCTIONS: [Instruction<'static>; 7] = [
    instruction!(b"// Write value to SP from D"),
    instruction!(b"@SP"),
    instruction!(b"A=M"),
    instruction!(b"M=D"),
    instruction!(b"// Incriment sp"),
    instruction!(b"@SP"),
    instruction!(b"M=M+1"),
];
