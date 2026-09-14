// Variant 1: `native`.
//
// Handwritten C++ matching Epok's proposed AOT lowering for the seven
// workloads: typed per-instance fields in a fixed pool, direct field access,
// no allocation, no dynamic dispatch beyond a per-class handler pointer, and
// the Q12 contract mirrored from runtime/blueprint_runtime.hpp (see q12.hpp).

#include "harness.hpp"

#include "common/syscalls/syscalls.h"

namespace harness {
void paint_stack();
int32_t stack_high_water();
}  // namespace harness

using namespace harness;

namespace {

// The native scalar method the `native_call` workload reaches. Kept out of line
// so the call is a real call in both the native and the VM variants.
__attribute__((noinline)) int32_t native_scale(int32_t value, int32_t factor) {
    return q12::mul(value, factor);
}

__attribute__((noinline)) q12::Vector3 native_get_position(const State& s) {
    return {s.px, s.py, s.pz};
}

__attribute__((noinline)) void native_set_position(State& s, const q12::Vector3& p) {
    s.px = p.x;
    s.py = p.y;
    s.pz = p.z;
}

using Handler = void (*)(State&, int32_t);

void update_empty(State&, int32_t) {}

void update_arith(State& s, int32_t dt) {
    int32_t v = s.v;
    v = q12::add(q12::mul(v, 4090), 16);
    int32_t n = q12::iadd(s.n, dt);
    if (v > 10000)
        n = q12::iadd(n, 2);
    else
        n = q12::iadd(n, 1);
    s.v = v;
    s.n = n;
}

void update_native_call(State& s, int32_t dt) {
    s.v = q12::iadd(native_scale(s.v, 4090), 16);
    s.n = q12::iadd(s.n, dt);
}

void update_position(State& s, int32_t dt) {
    q12::Vector3 p = native_get_position(s);
    p.x = q12::iadd(p.x, dt);
    p.y = q12::isub(p.y, dt);
    p.z = q12::iadd(p.z, 1);
    native_set_position(s, p);
    s.n = q12::iadd(s.n, 1);
}

void update_alloc(State& s, int32_t dt) {
    // The AOT lowering of the allocating Lua chunk: the transient record lives
    // in the frame, not on a heap. That absence is the measurement.
    struct Record {
        int32_t a, b, c;
        int32_t d[3];
    } t = {s.v, s.n, q12::iadd(s.v, dt), {1, 2, 3}};
    const int32_t total = q12::iadd(q12::iadd(q12::iadd(q12::iadd(t.a, t.c), t.d[0]), t.d[1]), t.d[2]);
    s.n = q12::iadd(s.n, q12::isub(q12::isub(total, t.a), t.c));
    s.v = t.c;
}

void event_tick(State& s, int32_t) { s.n = q12::iadd(s.n, 1); }

struct Instance {
    State state;
    Handler update = nullptr;
    Handler event = nullptr;
};

Instance g_instances[MAX_INSTANCES];
StepClock g_clock;
uint32_t g_calls = 0;

void register_instances(int workload, int count) {
    for (int i = 0; i < count; ++i) {
        Instance& inst = g_instances[i];
        reset_state(inst.state, i);
        inst.update = nullptr;
        inst.event = nullptr;
        switch (workload) {
            case ABSENT: break;
            case EMPTY: inst.update = update_empty; break;
            case ARITH: inst.update = update_arith; break;
            case NATIVE_CALL: inst.update = update_native_call; break;
            case POSITION: inst.update = update_position; break;
            case ALLOC: inst.update = update_alloc; break;
            case EVENT_ONLY: inst.event = event_tick; break;
            default: break;
        }
    }
}

void teardown_instances(int count) {
    for (int i = 0; i < count; ++i) {
        g_instances[i].update = nullptr;
        g_instances[i].event = nullptr;
    }
}

__attribute__((noinline)) void step(int count, int32_t dt, uint32_t tick) {
    const bool fire = (tick % EVENT_PERIOD) == 0;
    for (int i = 0; i < count; ++i) {
        Instance& inst = g_instances[i];
        if (inst.update) {
            inst.update(inst.state, dt);
            ++g_calls;
        }
        if (fire && inst.event) {
            inst.event(inst.state, dt);
            ++g_calls;
        }
    }
}

void run_case(int workload, int slot) {
    const int index = case_index(workload, slot);
    const int count = INSTANCE_COUNTS[slot];

    harness_mark_begin();
    register_instances(workload, count);
    mark(HARNESS_ID_SETUP(index));

    g_clock.reset();
    g_calls = 0;
    for (int i = 0; i < WARMUP_STEPS; ++i) {
        const int32_t dt = g_clock.begin_tick();
        step(count, dt, g_clock.ticks);
    }
    for (int i = 0; i < SAMPLE_STEPS; ++i) {
        const int32_t dt = g_clock.begin_tick();
        harness_mark_begin();
        step(count, dt, g_clock.ticks);
        mark(HARNESS_ID_STEADY(index));
    }

    Checksum sum;
    for (int i = 0; i < count; ++i) {
        const State& s = g_instances[i].state;
        sum.mix(s.v);
        sum.mix(s.n);
        sum.mix(s.px);
        sum.mix(s.py);
        sum.mix(s.pz);
    }
    harness_probe[HARNESS_PROBE_CHECKSUM + index] = int32_t(sum.value);
    harness_probe[HARNESS_PROBE_CALLS + index] = int32_t(g_calls);
    harness_probe[HARNESS_PROBE_CASE_RETAINED + index] = 0;
    harness_probe[HARNESS_PROBE_CASE_PEAK + index] = 0;
    harness_probe[HARNESS_PROBE_CASE_ALLOCS + index] = 0;
    harness_probe[HARNESS_PROBE_CASE_ALLOC_BYTES + index] = 0;

    harness_mark_begin();
    teardown_instances(count);
    mark(HARNESS_ID_TEARDOWN(index));
}

}  // namespace

int main() {
    harness::paint_stack();
    harness_probe[HARNESS_PROBE_MAGIC] = HARNESS_MAGIC;
    harness_probe[HARNESS_PROBE_VARIANT] = 0;
    harness_probe[HARNESS_PROBE_CASES] = CASE_COUNT;
    harness_probe[HARNESS_PROBE_HEADER_SIZE] = 0;
    harness_probe[HARNESS_PROBE_BLOB_SIZE] = 0;
    harness_probe[HARNESS_PROBE_CHUNKS] = 0;
    harness_probe[HARNESS_PROBE_LOAD_OK] = 1;

    harness_mark_begin();
    asm volatile("" ::: "memory");  // no VM to open
    mark(HARNESS_ID_VM_OPEN);

    for (int workload = 0; workload < WORKLOAD_COUNT; ++workload) {
        harness_mark_begin();
        asm volatile("" ::: "memory");  // no script to load
        mark(HARNESS_ID_LOAD(workload));
        for (int slot = 0; slot < INSTANCE_COUNT_SLOTS; ++slot) {
            run_case(workload, slot);
            ramsyscall_printf("native %s x%d done\n", WORKLOAD_NAMES[workload], INSTANCE_COUNTS[slot]);
        }
    }

    harness_probe[HARNESS_PROBE_HEAP_RETAINED] = 0;
    harness_probe[HARNESS_PROBE_HEAP_PEAK] = 0;
    harness_probe[HARNESS_PROBE_ALLOC_COUNT] = 0;
    harness_probe[HARNESS_PROBE_ALLOC_BYTES] = 0;
    harness_probe[HARNESS_PROBE_FREE_COUNT] = 0;
    harness_probe[HARNESS_PROBE_REALLOC_COUNT] = 0;
    harness_probe[HARNESS_PROBE_STACK_HIGH] = harness::stack_high_water();
    harness_probe[HARNESS_PROBE_DONE] = HARNESS_DONE;
    harness_done();
    while (true) asm volatile("");
    return 0;
}
