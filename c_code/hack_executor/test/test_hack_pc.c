#include <stddef.h>

#include "unity.h"
#include "hack_pc.h"

#define PC 3
#define EXAMPLE_A 1212

#define POSITIVE_RESULT 1
#define NEGOTIVE_RESULT -1
#define ZERO_RESULT 0

void test_pc_unconditional_incr() {
	short i = (short) 0b1110101010111000;
	TEST_ASSERT_EQUAL(PC + 1, perform_jump(PC, POSITIVE_RESULT, EXAMPLE_A, i));
}

void test_pc_unconditional_jmp() {
	short i = (short) 0b1110101010111111;
	TEST_ASSERT_EQUAL(EXAMPLE_A, perform_jump(PC, POSITIVE_RESULT, EXAMPLE_A, i));
}

void test_pc_a_incr() {
	short i = (short) 0b0110101010111111;
	TEST_ASSERT_EQUAL(PC + 1, perform_jump(PC, POSITIVE_RESULT, EXAMPLE_A, i));
}

void test_eq_jmp() {
	short i = (short) 0b1110101010111010;
	TEST_ASSERT_EQUAL(EXAMPLE_A, perform_jump(PC, ZERO_RESULT, EXAMPLE_A, i));
    TEST_ASSERT_EQUAL(PC + 1, perform_jump(PC, POSITIVE_RESULT, EXAMPLE_A, i));
    TEST_ASSERT_EQUAL(PC + 1, perform_jump(PC, NEGOTIVE_RESULT, EXAMPLE_A, i));
}

void test_le_jmp() {
	short i = (short) 0b1110101010111110;
	TEST_ASSERT_EQUAL(EXAMPLE_A, perform_jump(PC, ZERO_RESULT, EXAMPLE_A, i));
    TEST_ASSERT_EQUAL(PC + 1, perform_jump(PC, POSITIVE_RESULT, EXAMPLE_A, i));
    TEST_ASSERT_EQUAL(EXAMPLE_A, perform_jump(PC, NEGOTIVE_RESULT, EXAMPLE_A, i));
}

void setUp() {}

void tearDown() {}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_pc_unconditional_incr);
    RUN_TEST(test_pc_unconditional_jmp);
    RUN_TEST(test_pc_a_incr);
    RUN_TEST(test_eq_jmp);
    RUN_TEST(test_le_jmp);
	UNITY_END();

	return 0;
}