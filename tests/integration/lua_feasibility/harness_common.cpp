#include "harness.hpp"

#include "psyqo/alloc.h"

extern "C" {

volatile int32_t harness_probe[512] = {};
volatile int32_t harness_mark_id = 0;
uint8_t harness_bytecode[24576] = {};
volatile int32_t harness_bytecode_size = 0;

// The three breakpoint markers. They must survive -Os and --gc-sections, must
// not be inlined into their callers, and must not be merged with each other,
// so each one has a distinct body.
__attribute__((noinline, used)) void harness_mark_begin() { asm volatile("nop" ::: "memory"); }
__attribute__((noinline, used)) void harness_mark_end() { asm volatile("nop\nnop" ::: "memory"); }
__attribute__((noinline, used)) void harness_done() { asm volatile("nop\nnop\nnop" ::: "memory"); }
}

namespace harness {

// Stack high-water measurement: paint a band of untouched stack below the
// current frame, then scan it afterwards. Skipped (reported as null) if the
// band would overlap the live heap.
static constexpr uint32_t STACK_BAND = 32768;
static constexpr uint32_t STACK_PATTERN = 0xA5A5A5A5u;
static uint32_t* s_band_low = nullptr;
static uint32_t* s_band_high = nullptr;

void paint_stack() {
    uintptr_t sp = reinterpret_cast<uintptr_t>(__builtin_frame_address(0));
    uintptr_t high = (sp - 256) & ~uintptr_t(3);
    uintptr_t low = high - STACK_BAND;
    // The heap is lazy and may not exist yet, so the overlap check happens at
    // the end of the run instead, when the heap has reached its final extent.
    s_band_low = reinterpret_cast<uint32_t*>(low);
    s_band_high = reinterpret_cast<uint32_t*>(high);
    for (uint32_t* p = s_band_low; p < s_band_high; ++p) *p = STACK_PATTERN;
}

// Returns bytes of the painted band that were overwritten, or -1 if the
// measurement was not taken, or -2 if the band was fully consumed (meaning the
// real high water is at least STACK_BAND and the number is a lower bound).
int32_t stack_high_water() {
    if (!s_band_low) return -1;
    // If the heap ever reached into the painted band, the pattern is not
    // evidence of stack use and the measurement is discarded.
    uintptr_t heap_end = reinterpret_cast<uintptr_t>(psyqo_heap_end());
    if (heap_end >= reinterpret_cast<uintptr_t>(s_band_low)) return -1;
    uint32_t* p = s_band_low;
    while (p < s_band_high && *p == STACK_PATTERN) ++p;
    if (p == s_band_low) return -2;
    return int32_t(reinterpret_cast<uintptr_t>(s_band_high) - reinterpret_cast<uintptr_t>(p));
}

}  // namespace harness
