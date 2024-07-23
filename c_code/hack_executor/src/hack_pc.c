#include <stdio.h>

#include "hack_pc.h"

short perform_jump(short pc, short result, short a, short i) {
    if (i >= 0) {
        return pc + 1;
    }

    short jump_code1 = (short) (i & 0b111);
    short jump_code2 = (result < 0) * 4 + (result == 0) * 2 + (result > 0);
    short jmp = jump_code1 & jump_code2;

    if (jmp) {
        return a;
    } else {
        return pc + 1;
    }
}
