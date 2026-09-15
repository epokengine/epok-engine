// Variants 2, 3 and 4: `vm-parser`, `vm-noparser`, `vm-cached-dispatch`.
//
//   HARNESS_VARIANT 1  vm-parser            liblua.a,           chunks loaded from source
//   HARNESS_VARIANT 2  vm-noparser          liblua-noparser.a,  chunks loaded from bytecode
//   HARNESS_VARIANT 3  vm-cached-dispatch   liblua-noparser.a,  same bytecode, plus two integration choices
//                                           (numeric-id handler
//                                           cache in the registry + event bitmask, and per
//                                           instance chunk re-load for state isolation)
//
// Variants 2 and 3 share every line of VM setup, binding registration and
// numeric behaviour with variant 1; only the loader and, for variant 3, the
// dispatch strategy differ.

#include "harness.hpp"
#include "workloads_lua.hpp"

#include "common/syscalls/syscalls.h"
#include "psyqo/alloc.h"
#include "psyqo/xprintf.h"

extern "C" {
#include "lauxlib.h"
#include "lua.h"
#include "lualib.h"

// Internal, but exported by lundump.o in both the parser and the no-parser
// archives. This is the authority on what bytecode header THIS build accepts.
void luaU_header(unsigned char* h);

// harness_dump.c: luaU_dump() with strip=1, stripping debug information like an offline cooker.
// Compiled into the parser build only.
#if HARNESS_VARIANT == 1
int harness_dump_stripped(lua_State* L, lua_Writer w, void* data);
#endif
}

#if HARNESS_VARIANT >= 2
#include "bytecode_blob.h"
#endif

#define LUAC_HEADER_BYTES 18

namespace harness {
void paint_stack();
int32_t stack_high_water();
}  // namespace harness

using namespace harness;

// ---------------------------------------------------------------------------
// libc shims psxlua expects on the PSX target. psyqo-lua supplies these through
// --defsym; the harness defines them directly so it can also own the allocator.
// ---------------------------------------------------------------------------
extern "C" {
int luaI_sprintf(char* buf, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int ret = vsprintf(buf, fmt, ap);
    va_end(ap);
    return ret;
}
void luaI_free(void* ptr) { psyqo_free(ptr); }
void* luaI_realloc(void* ptr, size_t size) { return psyqo_realloc(ptr, size); }
}

namespace {

AllocStats g_alloc;
uint32_t g_global_peak = 0;
// Lifetime totals: AllocStats::reset_window() clears the per-case counters.
uint32_t g_total_allocs = 0, g_total_frees = 0, g_total_reallocs = 0, g_total_bytes = 0;

// Exact accounting: Lua hands us both the old and the new size, so this is a
// measurement rather than an allocator-header guess.
void* harness_alloc(void*, void* ptr, size_t osize, size_t nsize) {
    if (ptr == nullptr) osize = 0;  // osize is a type tag on fresh allocations
    if (nsize == 0) {
        if (ptr) {
            g_alloc.live -= uint32_t(osize);
            ++g_alloc.frees;
            ++g_total_frees;
            psyqo_free(ptr);
        }
        return nullptr;
    }
    void* result;
    if (ptr == nullptr) {
        ++g_alloc.allocations;
        ++g_total_allocs;
        g_alloc.bytes += uint32_t(nsize);
        g_total_bytes += uint32_t(nsize);
        result = psyqo_malloc(nsize);
        if (result) g_alloc.live += uint32_t(nsize);
    } else {
        ++g_alloc.reallocs;
        ++g_total_reallocs;
        if (nsize > osize) {
            g_alloc.bytes += uint32_t(nsize - osize);
            g_total_bytes += uint32_t(nsize - osize);
        }
        result = psyqo_realloc(ptr, nsize);
        if (result) g_alloc.live = g_alloc.live - uint32_t(osize) + uint32_t(nsize);
    }
    if (g_alloc.live > g_alloc.peak) g_alloc.peak = g_alloc.live;
    if (g_alloc.live > g_global_peak) g_global_peak = g_alloc.live;
    return result;
}

lua_State* L = nullptr;

// ---------------------------------------------------------------------------
// Native bindings. Identical in all three VM variants.
// ---------------------------------------------------------------------------
__attribute__((noinline)) int32_t native_scale(int32_t value, int32_t factor) {
    return q12::mul(value, factor);
}

int lua_native_scale(lua_State* S) {
    const int32_t value = int32_t(lua_tonumber(S, 1));
    const int32_t factor = int32_t(lua_tonumber(S, 2));
    lua_pushnumber(S, native_scale(value, factor));
    return 1;
}

// Builds a fresh three-component table, mirroring a table-based position getter
// shape. Components are raw Q12 numbers, not per-scalar FixedPoint tables.
int lua_get_position(lua_State* S) {
    lua_createtable(S, 0, 3);
    lua_getfield(S, 1, "px");
    lua_setfield(S, -2, "x");
    lua_getfield(S, 1, "py");
    lua_setfield(S, -2, "y");
    lua_getfield(S, 1, "pz");
    lua_setfield(S, -2, "z");
    return 1;
}

int lua_set_position(lua_State* S) {
    lua_getfield(S, 2, "x");
    lua_setfield(S, 1, "px");
    lua_getfield(S, 2, "y");
    lua_setfield(S, 1, "py");
    lua_getfield(S, 2, "z");
    lua_setfield(S, 1, "pz");
    return 0;
}

// ---------------------------------------------------------------------------
// Bytecode production (parser build only) and loading.
// ---------------------------------------------------------------------------
#if HARNESS_VARIANT == 1
int dump_writer(lua_State*, const void* p, size_t size, void* ud) {
    int32_t* cursor = static_cast<int32_t*>(ud);
    if (*cursor + int32_t(size) > int32_t(sizeof(harness_bytecode))) return 1;
    __builtin_memcpy(harness_bytecode + *cursor, p, size);
    *cursor += int32_t(size);
    return 0;
}

// Produces bytecode with the target's own dumper, so there is no host/target
// ABI question to begin with; the header is recorded regardless.
void dump_all_chunks() {
    int32_t cursor = 0;
    for (int i = 0; i < WORKLOAD_COUNT; ++i) {
        const char* src = LUA_SOURCES[i];
        const int status = luaL_loadbuffer(L, src, __builtin_strlen(src), LUA_CHUNK_NAMES[i]);
        if (status != 0) {
            ramsyscall_printf("dump load failed %s: %s\n", LUA_CHUNK_NAMES[i], lua_tostring(L, -1));
            harness_probe[HARNESS_PROBE_LOAD_OK] = 0;
            lua_pop(L, 1);
            continue;
        }
        const int32_t start = cursor;
        // strip=1 drops debug information, as an offline cooker would.
        if (harness_dump_stripped(L, dump_writer, &cursor) != 0) {
            ramsyscall_printf("dump failed %s\n", LUA_CHUNK_NAMES[i]);
            harness_probe[HARNESS_PROBE_LOAD_OK] = 0;
        }
        lua_pop(L, 1);
        harness_probe[HARNESS_PROBE_CHUNK_OFF + i] = start;
        harness_probe[HARNESS_PROBE_CHUNK_SIZE + i] = cursor - start;
    }
    harness_bytecode_size = cursor;
    harness_probe[HARNESS_PROBE_BLOB_SIZE] = cursor;
    harness_probe[HARNESS_PROBE_CHUNKS] = WORKLOAD_COUNT;
}
#endif

// Pushes the chunk for `workload` onto the stack. Returns 0 on success.
int load_chunk(int workload) {
#if HARNESS_VARIANT == 1
    const char* src = LUA_SOURCES[workload];
    return luaL_loadbuffer(L, src, __builtin_strlen(src), LUA_CHUNK_NAMES[workload]);
#else
    return luaL_loadbuffer(L, reinterpret_cast<const char*>(BYTECODE_BLOB) + BYTECODE_OFFSET[workload],
                           size_t(BYTECODE_SIZE[workload]), LUA_CHUNK_NAMES[workload]);
#endif
}

// ---------------------------------------------------------------------------
// Instances.
// ---------------------------------------------------------------------------
constexpr uint32_t EVENT_UPDATE = 1u << 0;
constexpr uint32_t EVENT_TICK = 1u << 1;

struct Instance {
    int self_ref = LUA_NOREF;
#if HARNESS_VARIANT == 3
    int update_ref = LUA_NOREF;
    int event_ref = LUA_NOREF;
    uint32_t events = 0;
#endif
};

Instance g_instances[MAX_INSTANCES];
StepClock g_clock;
uint32_t g_calls = 0;

// Shared handler refs for variants 1 and 2: the chunk is executed once and the
// resulting closures are shared by every instance.
#if HARNESS_VARIANT != 3
int g_shared_table = LUA_NOREF;
#endif

void push_new_state_table(int index) {
    State s;
    reset_state(s, index);
    lua_createtable(L, 0, 7);
    lua_pushnumber(L, s.v);
    lua_setfield(L, -2, "v");
    lua_pushnumber(L, s.n);
    lua_setfield(L, -2, "n");
    lua_pushnumber(L, s.px);
    lua_setfield(L, -2, "px");
    lua_pushnumber(L, s.py);
    lua_setfield(L, -2, "py");
    lua_pushnumber(L, s.pz);
    lua_setfield(L, -2, "pz");
}

bool g_case_failed = false;

void report(const char* what) {
    ramsyscall_printf("ERROR %s: %s\n", what, lua_isstring(L, -1) ? lua_tostring(L, -1) : "?");
    harness_probe[HARNESS_PROBE_LOAD_OK] = 0;
    g_case_failed = true;
}

// Runs a chunk and leaves its handler table on the stack. Returns false on error.
bool run_chunk(int workload) {
    if (load_chunk(workload) != 0) {
        report("chunk load");
        lua_pop(L, 1);
        return false;
    }
    if (lua_pcall(L, 0, 1, 0) != 0) {
        report("chunk call");
        lua_pop(L, 1);
        return false;
    }
    if (!lua_istable(L, -1)) {
        ramsyscall_printf("ERROR chunk did not return a table\n");
        harness_probe[HARNESS_PROBE_LOAD_OK] = 0;
        lua_pop(L, 1);
        return false;
    }
    return true;
}

void register_instances(int workload, int count) {
#if HARNESS_VARIANT != 3
    // Variants 1 and 2: one shared handler table, per-instance state tables,
    // handlers copied into each instance table (naive integration).
    for (int i = 0; i < count; ++i) {
        push_new_state_table(i);
        lua_rawgeti(L, LUA_REGISTRYINDEX, g_shared_table);  // handlers
        lua_getfield(L, -1, "update");
        if (lua_isnil(L, -1))
            lua_pop(L, 1);
        else
            lua_setfield(L, -3, "update");
        lua_getfield(L, -1, "event");
        if (lua_isnil(L, -1))
            lua_pop(L, 1);
        else
            lua_setfield(L, -3, "event");
        lua_pop(L, 1);  // handlers
        g_instances[i].self_ref = luaL_ref(L, LUA_REGISTRYINDEX);
    }
#else
    // Variant 3, cached-dispatch: re-load and re-execute the chunk once per
    // instance so each instance gets its own closures/file-local state, then
    // cache each handler under a numeric registry id and record an event
    // bitmask so an absent handler never reaches Lua at all.
    for (int i = 0; i < count; ++i) {
        Instance& inst = g_instances[i];
        inst.update_ref = inst.event_ref = LUA_NOREF;
        inst.events = 0;
        if (!run_chunk(workload)) {
            push_new_state_table(i);
            inst.self_ref = luaL_ref(L, LUA_REGISTRYINDEX);
            continue;
        }
        lua_getfield(L, -1, "update");
        if (lua_isnil(L, -1)) {
            lua_pop(L, 1);
        } else {
            inst.update_ref = luaL_ref(L, LUA_REGISTRYINDEX);
            inst.events |= EVENT_UPDATE;
        }
        lua_getfield(L, -1, "event");
        if (lua_isnil(L, -1)) {
            lua_pop(L, 1);
        } else {
            inst.event_ref = luaL_ref(L, LUA_REGISTRYINDEX);
            inst.events |= EVENT_TICK;
        }
        lua_pop(L, 1);  // handler table
        push_new_state_table(i);
        inst.self_ref = luaL_ref(L, LUA_REGISTRYINDEX);
    }
#endif
}

void teardown_instances(int count) {
    for (int i = 0; i < count; ++i) {
        Instance& inst = g_instances[i];
        if (inst.self_ref != LUA_NOREF) luaL_unref(L, LUA_REGISTRYINDEX, inst.self_ref);
        inst.self_ref = LUA_NOREF;
#if HARNESS_VARIANT == 3
        if (inst.update_ref != LUA_NOREF) luaL_unref(L, LUA_REGISTRYINDEX, inst.update_ref);
        if (inst.event_ref != LUA_NOREF) luaL_unref(L, LUA_REGISTRYINDEX, inst.event_ref);
        inst.update_ref = inst.event_ref = LUA_NOREF;
        inst.events = 0;
#endif
    }
}

__attribute__((noinline)) void step(int count, int32_t dt, uint32_t tick) {
    const bool fire = (tick % EVENT_PERIOD) == 0;
    for (int i = 0; i < count; ++i) {
        Instance& inst = g_instances[i];
#if HARNESS_VARIANT != 3
        lua_rawgeti(L, LUA_REGISTRYINDEX, inst.self_ref);
        lua_getfield(L, -1, "update");
        if (lua_isnil(L, -1)) {
            lua_pop(L, 2);
        } else {
            lua_insert(L, -2);
            lua_pushnumber(L, dt);
            if (lua_pcall(L, 2, 0, 0) != 0) {
                report("update");
                lua_pop(L, 1);
            }
            ++g_calls;
        }
        if (fire) {
            lua_rawgeti(L, LUA_REGISTRYINDEX, inst.self_ref);
            lua_getfield(L, -1, "event");
            if (lua_isnil(L, -1)) {
                lua_pop(L, 2);
            } else {
                lua_insert(L, -2);
                lua_pushnumber(L, dt);
                if (lua_pcall(L, 2, 0, 0) != 0) {
                    report("event");
                    lua_pop(L, 1);
                }
                ++g_calls;
            }
        }
#else
        if (inst.events & EVENT_UPDATE) {
            lua_rawgeti(L, LUA_REGISTRYINDEX, inst.update_ref);
            lua_rawgeti(L, LUA_REGISTRYINDEX, inst.self_ref);
            lua_pushnumber(L, dt);
            if (lua_pcall(L, 2, 0, 0) != 0) {
                report("update");
                lua_pop(L, 1);
            }
            ++g_calls;
        }
        if (fire && (inst.events & EVENT_TICK)) {
            lua_rawgeti(L, LUA_REGISTRYINDEX, inst.event_ref);
            lua_rawgeti(L, LUA_REGISTRYINDEX, inst.self_ref);
            lua_pushnumber(L, dt);
            if (lua_pcall(L, 2, 0, 0) != 0) {
                report("event");
                lua_pop(L, 1);
            }
            ++g_calls;
        }
#endif
    }
}

int32_t read_field(int ref, const char* key) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, ref);
    lua_getfield(L, -1, key);
    const int32_t value = int32_t(lua_tonumber(L, -1));
    lua_pop(L, 2);
    return value;
}

void run_case(int workload, int slot) {
    const int index = case_index(workload, slot);
    const int count = INSTANCE_COUNTS[slot];

    g_alloc.reset_window();
    harness_mark_begin();
    register_instances(workload, count);
    mark(HARNESS_ID_SETUP(index));

    g_clock.reset();
    g_calls = 0;
    for (int i = 0; i < WARMUP_STEPS; ++i) {
        const int32_t dt = g_clock.begin_tick();
        step(count, dt, g_clock.ticks);
    }
    // Steady state only: the window above absorbs first-touch string interning,
    // table rehashes and the first GC cycles.
    const uint32_t allocations_before = g_alloc.allocations;
    const uint32_t bytes_before = g_alloc.bytes;
    g_alloc.peak = g_alloc.live;
    for (int i = 0; i < SAMPLE_STEPS; ++i) {
        const int32_t dt = g_clock.begin_tick();
        harness_mark_begin();
        step(count, dt, g_clock.ticks);
        mark(HARNESS_ID_STEADY(index));
    }

    Checksum sum;
    for (int i = 0; i < count; ++i) {
        sum.mix(read_field(g_instances[i].self_ref, "v"));
        sum.mix(read_field(g_instances[i].self_ref, "n"));
        sum.mix(read_field(g_instances[i].self_ref, "px"));
        sum.mix(read_field(g_instances[i].self_ref, "py"));
        sum.mix(read_field(g_instances[i].self_ref, "pz"));
    }
    harness_probe[HARNESS_PROBE_CHECKSUM + index] = g_case_failed ? 0 : int32_t(sum.value);
    harness_probe[HARNESS_PROBE_CALLS + index] = int32_t(g_calls);
    harness_probe[HARNESS_PROBE_CASE_RETAINED + index] = int32_t(g_alloc.live);
    harness_probe[HARNESS_PROBE_CASE_PEAK + index] = int32_t(g_alloc.peak);
    harness_probe[HARNESS_PROBE_CASE_ALLOCS + index] = int32_t(g_alloc.allocations - allocations_before);
    harness_probe[HARNESS_PROBE_CASE_ALLOC_BYTES + index] = int32_t(g_alloc.bytes - bytes_before);

    harness_mark_begin();
    teardown_instances(count);
    mark(HARNESS_ID_TEARDOWN(index));
    lua_gc(L, LUA_GCCOLLECT, 0);
    lua_settop(L, 0);
}

}  // namespace

int main() {
    harness::paint_stack();
    harness_probe[HARNESS_PROBE_MAGIC] = HARNESS_MAGIC;
    harness_probe[HARNESS_PROBE_VARIANT] = HARNESS_VARIANT;
    harness_probe[HARNESS_PROBE_CASES] = CASE_COUNT;
    harness_probe[HARNESS_PROBE_LOAD_OK] = 1;

    // Record the bytecode header THIS build accepts, straight from the linked
    // undumper. The driver compares it against the produced bytecode.
    unsigned char header[LUAC_HEADER_BYTES];
    luaU_header(header);
    harness_probe[HARNESS_PROBE_HEADER_SIZE] = LUAC_HEADER_BYTES;
    for (int i = 0; i < LUAC_HEADER_BYTES; ++i) harness_probe[HARNESS_PROBE_HEADER + i] = header[i];

    harness_mark_begin();
    L = lua_newstate(harness_alloc, nullptr);
    if (L) {
        luaL_openlibs(L);
        lua_pushcfunction(L, lua_native_scale);
        lua_setglobal(L, "native_scale");
        lua_pushcfunction(L, lua_get_position);
        lua_setglobal(L, "get_position");
        lua_pushcfunction(L, lua_set_position);
        lua_setglobal(L, "set_position");
    }
    mark(HARNESS_ID_VM_OPEN);
    if (!L) {
        harness_probe[HARNESS_PROBE_LOAD_OK] = 0;
        harness_probe[HARNESS_PROBE_DONE] = HARNESS_DONE;
        harness_done();
        while (true) asm volatile("");
    }

#if HARNESS_VARIANT == 1
    // Dedicated marker so a bytecode-production run can stop here; it also
    // times the on-target compile+dump of all seven chunks.
    harness_mark_begin();
    dump_all_chunks();
    mark(HARNESS_ID_DUMP_DONE);
    lua_gc(L, LUA_GCCOLLECT, 0);
#else
    harness_probe[HARNESS_PROBE_BLOB_SIZE] = int32_t(sizeof(BYTECODE_BLOB));
    harness_probe[HARNESS_PROBE_CHUNKS] = WORKLOAD_COUNT;
    for (int i = 0; i < WORKLOAD_COUNT; ++i) {
        harness_probe[HARNESS_PROBE_CHUNK_OFF + i] = BYTECODE_OFFSET[i];
        harness_probe[HARNESS_PROBE_CHUNK_SIZE + i] = BYTECODE_SIZE[i];
    }
#endif

    for (int workload = 0; workload < WORKLOAD_COUNT; ++workload) {
        g_case_failed = false;
        // Loading/compiling is timed separately from steady state.
        harness_mark_begin();
#if HARNESS_VARIANT != 3
        const bool ok = run_chunk(workload);
        if (ok)
            g_shared_table = luaL_ref(L, LUA_REGISTRYINDEX);
        else
            g_shared_table = LUA_NOREF;
#else
        // Variant 3 loads per instance during registration; this phase only
        // validates that the chunk is loadable at all.
        if (run_chunk(workload)) lua_pop(L, 1);
#endif
        mark(HARNESS_ID_LOAD(workload));

        for (int slot = 0; slot < INSTANCE_COUNT_SLOTS; ++slot) {
            run_case(workload, slot);
            ramsyscall_printf("vm%d %s x%d calls=%d\n", HARNESS_VARIANT, WORKLOAD_NAMES[workload],
                              INSTANCE_COUNTS[slot], int(harness_probe[HARNESS_PROBE_CALLS + case_index(workload, slot)]));
        }
#if HARNESS_VARIANT != 3
        if (g_shared_table != LUA_NOREF) luaL_unref(L, LUA_REGISTRYINDEX, g_shared_table);
        g_shared_table = LUA_NOREF;
#endif
        lua_gc(L, LUA_GCCOLLECT, 0);
    }

    harness_probe[HARNESS_PROBE_HEAP_RETAINED] = int32_t(g_alloc.live);
    harness_probe[HARNESS_PROBE_HEAP_PEAK] = int32_t(g_global_peak);
    harness_probe[HARNESS_PROBE_ALLOC_COUNT] = int32_t(g_total_allocs);
    harness_probe[HARNESS_PROBE_ALLOC_BYTES] = int32_t(g_total_bytes);
    harness_probe[HARNESS_PROBE_FREE_COUNT] = int32_t(g_total_frees);
    harness_probe[HARNESS_PROBE_REALLOC_COUNT] = int32_t(g_total_reallocs);
    harness_probe[HARNESS_PROBE_STACK_HIGH] = harness::stack_high_water();
    harness_probe[HARNESS_PROBE_DONE] = HARNESS_DONE;
    harness_done();
    while (true) asm volatile("");
    return 0;
}
