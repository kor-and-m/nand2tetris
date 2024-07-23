#include <stdlib.h>
#include "hack_memory.h"
#include "hack_alu.h"

short hack_alu_perform(short * memory, short a, short d, short i) {
    if (i >= 0) {
        return i;
    } else {
        short op_code = (short) ((i >> 6) & 0b1111111);
        switch (op_code) {
        case 0b0101010:
            return 0;
            break;
        case 0b0111111:
            return 1;
            break;
        case 0b0111010:
            return -1;
            break;
        case 0b0001100:
            return d;
            break;
        case 0b0110000:
            return a;
            break;
        case 0b1110000:
            return get_hack_memory(memory, a);
            break;
        case 0b0001101:
            return ~d;
            break;
        case 0b0110001:
            return ~a;
            break;
        case 0b1110001:
            return ~get_hack_memory(memory, a);
            break;
        case 0b0001111:
            return -d;
            break;
        case 0b0110011:
            return -a;
            break;
        case 0b1110011:
            return -get_hack_memory(memory, a);
            break;
        case 0b0011111:
            return d + 1;
            break;
        case 0b0110111:
            return a + 1;
            break;
        case 0b1110111:
            return get_hack_memory(memory, a) + 1;
            break;
        case 0b0001110:
            return d - 1;
            break;
        case 0b0110010:
            return a - 1;
            break;
        case 0b1110010:
            return get_hack_memory(memory, a) - 1;
            break;
        case 0b0000010:
            return d + a;
            break;
        case 0b1000010:
            return d + get_hack_memory(memory, a);
            break;
        case 0b0010011:
            return d - a;
            break;
        case 0b1010011:
            return d - get_hack_memory(memory, a);
            break;
        case 0b0000111:
            return a - d;
            break;
        case 0b1000111:
            return get_hack_memory(memory, a) - d;
            break;
        case 0b0000000:
            return d & a;
            break;
        case 0b1000000:
            return d & get_hack_memory(memory, a);
            break;
        case 0b0010101:
            return d | a;
            break;
        case 0b1010101:
            return d | get_hack_memory(memory, a);
            break;
        default:
            return -1;
            break;
        }
    }
}