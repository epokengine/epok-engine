#pragma once
#include <stdint.h>
#include <stddef.h>

#include "q12.hpp"

// Shared scaffolding for the four feasibility variants. Everything here is
// identical native C++ in every variant so that the only difference measured
// is how one simulation step reaches (or fails to reach) script code.

extern "C" {

// Observable probe. Read out of guest RAM by the PCSX-Redux driver script.
// Layout is mirrored in tests/integration/verify_lua_feasibility.py (PROBE_*).
extern volatile int32_t harness_probe[512];
extern volatile int32_t harness_mark_id;

// Breakpoint markers. Never inlined, never optimised away.
void harness_mark_begin();
void harness_mark_end();
void harness_done();

// Bytecode staging buffer (only written by the parser variant).
extern uint8_t harness_bytecode[24576];
extern volatile int32_t harness_bytecode_size;
}

#define HARNESS_PROBE_MAGIC 0
#define HARNESS_PROBE_VARIANT 1
#define HARNESS_PROBE_CASES 2
#define HARNESS_PROBE_DONE 3
#define HARNESS_PROBE_HEADER_SIZE 4
#define HARNESS_PROBE_HEADER 5      /* 18 bytes, one per slot */
#define HARNESS_PROBE_BLOB_SIZE 23
#define HARNESS_PROBE_CHUNK_SIZE 24 /* 8 slots */
#define HARNESS_PROBE_CHUNK_OFF 32  /* 8 slots */
#define HARNESS_PROBE_CHUNKS 40
#define HARNESS_PROBE_HEAP_RETAINED 41
#define HARNESS_PROBE_HEAP_PEAK 42
#define HARNESS_PROBE_ALLOC_COUNT 43
#define HARNESS_PROBE_ALLOC_BYTES 44
#define HARNESS_PROBE_FREE_COUNT 45
#define HARNESS_PROBE_STACK_HIGH 46
#define HARNESS_PROBE_LOAD_OK 47
#define HARNESS_PROBE_REALLOC_COUNT 48
#define HARNESS_PROBE_CHECKSUM 64   /* one per case */
#define HARNESS_PROBE_CALLS 128
#define HARNESS_PROBE_CASE_RETAINED 192
#define HARNESS_PROBE_CASE_PEAK 256
#define HARNESS_PROBE_CASE_ALLOCS 320
#define HARNESS_PROBE_CASE_ALLOC_BYTES 384

#define HARNESS_MAGIC 0x4C554146 /* 'LUAF' */
#define HARNESS_DONE 0x0000D09E

// Marker ids. Steady-state samples use the bare case index so the driver can
// bucket them; setup/teardown phases are offset into disjoint ranges.
#define HARNESS_ID_STEADY(case_index) (case_index)
#define HARNESS_ID_SETUP(case_index) (1000 + (case_index))
#define HARNESS_ID_TEARDOWN(case_index) (2000 + (case_index))
#define HARNESS_ID_LOAD(chunk) (3000 + (chunk))
#define HARNESS_ID_VM_OPEN 3900
#define HARNESS_ID_DUMP_DONE 3901

namespace harness {

enum Workload : int {
    ABSENT = 0,
    EMPTY,
    ARITH,
    NATIVE_CALL,
    POSITION,
    ALLOC,
    EVENT_ONLY,
    WORKLOAD_COUNT
};

inline constexpr const char* WORKLOAD_NAMES[WORKLOAD_COUNT] = {
    "absent", "empty", "arith", "native_call", "position", "alloc", "event_only"};

inline constexpr int INSTANCE_COUNTS[4] = {1, 16, 32, 64};
inline constexpr int INSTANCE_COUNT_SLOTS = 4;
inline constexpr int MAX_INSTANCES = 64;
inline constexpr int CASE_COUNT = WORKLOAD_COUNT * INSTANCE_COUNT_SLOTS;

inline constexpr int WARMUP_STEPS = 6;
inline constexpr int SAMPLE_STEPS = 60;
inline constexpr int TOTAL_STEPS = WARMUP_STEPS + SAMPLE_STEPS;

// The event workload fires its event every EVENT_PERIOD steps.
inline constexpr int EVENT_PERIOD = 16;

inline constexpr int case_index(int workload, int count_slot) {
    return workload * INSTANCE_COUNT_SLOTS + count_slot;
}

// Epok's 60 Hz fixed step. Mirrors runtime/time.hpp Time::begin_tick():
//   delta_raw = 68; delta_remainder += 16; if (delta_remainder >= 60) { ++delta_raw; delta_remainder -= 60; }
// Q12 seconds, 68/69 alternating without cumulative drift.
struct StepClock {
    int32_t delta_raw = 68;
    uint32_t delta_remainder = 0;
    uint32_t ticks = 0;
    void reset() {
        delta_raw = 68;
        delta_remainder = 0;
        ticks = 0;
    }
    int32_t begin_tick() {
        ++ticks;
        delta_raw = 68;
        delta_remainder += 16;
        if (delta_remainder >= 60) {
            ++delta_raw;
            delta_remainder -= 60;
        }
        return delta_raw;
    }
};

// Identical fold in every variant. Anything a workload can observe must end up
// here or the compiler/VM is free to drop the work.
struct Checksum {
    uint32_t value = 2166136261u;
    void mix(int32_t x) {
        value ^= uint32_t(x);
        value *= 16777619u;
    }
};

// Per-instance gameplay state. The native variant stores this directly; the VM
// variants keep the same five fields inside a Lua table.
struct State {
    int32_t v = 0, n = 0, px = 0, py = 0, pz = 0;
};

inline void reset_state(State& s, int index) {
    s.v = q12::from_int(1);          // 4096 == 1.0
    s.n = 0;
    s.px = index * 4096;
    s.py = 0;
    s.pz = 0;
}

inline void mark(int id) {
    harness_mark_id = id;
    harness_mark_end();
}

// Allocation accounting. The VM variants install this as the lua_Alloc, which
// gives exact old/new sizes instead of guessing from allocator headers. The
// native variant never allocates, which is itself the measurement.
struct AllocStats {
    uint32_t live = 0, peak = 0, allocations = 0, frees = 0, reallocs = 0, bytes = 0;
    void reset_window() {
        peak = live;
        allocations = frees = reallocs = bytes = 0;
    }
};

}  // namespace harness
