typedef struct HackExecutor {
    short a;
    short d;
    short pc;
    short * memory;
    short * program;
} HackExecutor;

#define SP 0

HackExecutor * init_hack_executor(short instructions[]);
void free_hack_executor(HackExecutor * executor);

void run_executor(HackExecutor * executor, short iterations);
short read_memory(HackExecutor * executor, short pointer);
short read_stack_value(HackExecutor * executor);

void save_result(HackExecutor * executor, short result, short i);
