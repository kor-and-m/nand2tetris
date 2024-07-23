#include <stdlib.h>
#include <stdio.h>

#include "hack_memory.h"
#include "hack_alu.h"
#include "hack_pc.h"
#include "hack_executor.h"

HackExecutor * init_hack_executor(short instructions[]) {
    short * memory = new_hack_memory();
    HackExecutor * executor = malloc(sizeof(HackExecutor));
    executor->a = 0;
    executor->d=0;
    executor->pc=0;
    executor->memory=memory;
    executor->program=instructions;
    return executor;
}

void free_hack_executor(HackExecutor * executor) {
    free(executor->memory);
    free(executor);
}

void run_executor(HackExecutor * executor, short iterations_count) {
    int a;

    for(a = 0; a < iterations_count; a++) {
        short instruction = get_hack_memory(executor->program, executor->pc);
        short result = hack_alu_perform(executor->memory, executor->a, executor->d, instruction);
        executor->pc = perform_jump(executor->pc, result, executor->a, instruction);
        save_result(executor, result, instruction);
    }
}

short read_stack_value(HackExecutor * executor) {
    short v = read_memory(executor, SP);
    return read_memory(executor, v - 1);
}

short read_memory(HackExecutor * executor, short pointer) {
    return get_hack_memory(executor->memory, pointer);
}

void save_result(HackExecutor * executor, short result, short i) {
    if (i >= 0) {
        executor->a = result;
        return;
    }

    short result_code = (short) ((i >> 3) & 0b111);
    if (result_code & 0b1) {
        set_hack_memory(executor->memory, executor->a, result);
    }
    if (result_code > 3) {
        executor->a = result;
    }
    if ((result_code & 0b11) > 1) {
        executor->d = result;
    }
}
