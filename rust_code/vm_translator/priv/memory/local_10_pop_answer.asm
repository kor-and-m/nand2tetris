// Start Prepare
@LCL
D=M
@10
D=D+A
@R5
M=D
// End Prepare
// Decriment SP
@SP
M=M-1
// Extract value from SP to D
@SP
A=M
D=M
@R5
A=M
M=D
