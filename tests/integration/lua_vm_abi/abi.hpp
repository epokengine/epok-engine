#pragma once
#include <stdint.h>

// Observable probe read out of guest RAM by tests/integration/verify_lua_vm_abi.py.
// Slot meanings are mirrored there as PROBE_*.
extern "C" {
extern volatile int32_t abi_probe[64];
extern unsigned char abi_bytecode[16384];
extern volatile int32_t abi_bytecode_size;
// Breakpoint marker: never inlined, never collected.
void abi_done();
}

#define ABI_PROBE_MAGIC 0
#define ABI_PROBE_MODE 1
#define ABI_PROBE_DONE 2
#define ABI_PROBE_LOAD_OK 3
#define ABI_PROBE_HEADER_SIZE 4
#define ABI_PROBE_HEADER 5 /* 18 slots, one byte each */
#define ABI_PROBE_BLOB_SIZE 23
#define ABI_PROBE_IADD 24
#define ABI_PROBE_IDIV 25
#define ABI_PROBE_FMUL 26
#define ABI_PROBE_INEG 27
#define ABI_PROBE_ULT 28
#define ABI_PROBE_BOOL_BEFORE 29
#define ABI_PROBE_BOOL_AFTER 30
#define ABI_PROBE_BOOL_FIELD 31
#define ABI_PROBE_NESTED 32
#define ABI_PROBE_ABSENT_BOUND 33
#define ABI_PROBE_ABSENT_ENTRIES 34
#define ABI_PROBE_ARENA_PEAK 35
#define ABI_PROBE_ARENA_LIVE 36
#define ABI_PROBE_ARENA_ALLOCS 37
#define ABI_PROBE_ERRORS 38

#define ABI_MAGIC 0x4C554142 /* 'LUAB' */
#define ABI_DONE 0x0000D09E
