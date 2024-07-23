#include <stddef.h>

#include "unity.h"
#include "hack_alu.h"
#include "hack_memory.h"

#define EXAMPLE_D -1488
#define EXAMPLE_A 1212
#define EXAMPLE_M -1666

void test_alu_zero() {
	short * m = new_hack_memory();
	short i = (short) 0b1110101010111111;
	TEST_ASSERT_EQUAL(0, hack_alu_perform(m, EXAMPLE_A, EXAMPLE_D, i));
}

void test_alu_d() {
	short * m = new_hack_memory();
	short i = (short) 0b1110001100111111;
	TEST_ASSERT_EQUAL(EXAMPLE_D, hack_alu_perform(m, EXAMPLE_A, EXAMPLE_D, i));
}

void test_alu_m() {
	short * m = new_hack_memory();
	set_hack_memory(m, EXAMPLE_A, EXAMPLE_M);
	short i = (short) 0b1111110000111111;
	TEST_ASSERT_EQUAL(EXAMPLE_M, hack_alu_perform(m, EXAMPLE_A, EXAMPLE_D, i));
}

void test_not_alu_m() {
	short * m = new_hack_memory();
	set_hack_memory(m, EXAMPLE_A, EXAMPLE_M);
	short i = (short) 0b1111110001111111;
	TEST_ASSERT_EQUAL(~EXAMPLE_M, hack_alu_perform(m, EXAMPLE_A, EXAMPLE_D, i));
}

void test_add_alu_a_d() {
	short * m = new_hack_memory();
	short i = (short) 0b1110000010111111;
	TEST_ASSERT_EQUAL(EXAMPLE_A + EXAMPLE_D, hack_alu_perform(m, EXAMPLE_A, EXAMPLE_D, i));
}

void setUp() {}

void tearDown() {}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_alu_zero);
	RUN_TEST(test_alu_d);
	RUN_TEST(test_alu_m);
	RUN_TEST(test_not_alu_m);
	RUN_TEST(test_add_alu_a_d);
	UNITY_END();

	return 0;
}