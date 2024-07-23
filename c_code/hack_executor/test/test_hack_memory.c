#include <stddef.h>

#include "unity.h"
#include "hack_memory.h"

#define EXAMPLE_ADDRES 10
#define EXAMPLE_VALUE -12

void test_memory_save_and_restore() {
    short * i = new_hack_memory();
	set_hack_memory(i, EXAMPLE_ADDRES, EXAMPLE_VALUE);
	TEST_ASSERT_EQUAL(EXAMPLE_VALUE, get_hack_memory(i, EXAMPLE_ADDRES));
}

void setUp() {}

void tearDown() {}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_memory_save_and_restore);
	UNITY_END();

	return 0;
}