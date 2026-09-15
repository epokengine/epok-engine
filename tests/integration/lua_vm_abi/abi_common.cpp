#include "abi.hpp"

extern "C" {
volatile int32_t abi_probe[64] = {};
unsigned char abi_bytecode[16384] = {};
volatile int32_t abi_bytecode_size = 0;
__attribute__((noinline, used)) void abi_done() { asm volatile("nop\nnop\nnop" ::: "memory"); }
}
