#include <stdlib.h>
#include "hack_memory.h"

short * new_hack_memory() {
    short * memory = malloc((KBD_ADDRESS + 1) * 2);
    return memory;
}

void set_hack_memory(short * memory, short addr, short val) {
    memory[addr] = val;
}

short get_hack_memory(short * memory, short addr) {
    return memory[addr];
}
