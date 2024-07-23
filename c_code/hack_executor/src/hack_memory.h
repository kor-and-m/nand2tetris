#define KBD_ADDRESS 24576
// #define SCREEN_ADDRESS 16384
// #define SP 256

short * new_hack_memory();
void set_hack_memory(short * memory, short addr, short val);
short get_hack_memory(short * memory, short addr);
