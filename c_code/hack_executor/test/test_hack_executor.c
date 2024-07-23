#include <stddef.h>

#include "unity.h"
#include "hack_executor.h"

void test_simple_add_program() {
    short instructions[] = {
        0b0000000000000010,
        0b1110110000010000,
        0b0000000000000011,
        0b1110000010010000,
        0b0000000000000000,
        0b1110001100001000
    };
    HackExecutor * executor = init_hack_executor(instructions);
    run_executor(executor, 6);
    short res = read_memory(executor, 0);
    free_hack_executor(executor);

	
	TEST_ASSERT_EQUAL(5, res);
}

void setUp() {}

void tearDown() {}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_simple_add_program);
	UNITY_END();

	return 0;
}