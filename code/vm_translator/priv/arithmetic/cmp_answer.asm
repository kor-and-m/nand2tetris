// Decriment SP
@SP
M=M-1
// Extract value from SP to D
@SP
A=M
D=M
@SP
A=M-1
D=M-D
@AnyFile_TRUE_0
D;JEQ
@SP
A=M-1
M=0
@AnyFile_FALSE_0
0;JMP
(AnyFile_TRUE_0)
@SP
A=M-1
M=-1
(AnyFile_FALSE_0)
