// Emulator conformance smoke for `runtime/lua_runtime.hpp`.
//
// Links the real runtime header against a handwritten `ClassBinding` for one
// fake object and runs the SAME normalized chunk in both packagings: as source
// (EPOK_LUA_MODE=2, parser archive) and as host-cooked bytecode
// (EPOK_LUA_MODE=1, no-parser archive). Every probe below must read identically
// in the two builds, which is what makes the two modes interchangeable.
//
// This is not a game and not a benchmark: it links no Epok runtime beyond the
// object model and the Blueprint numerics that `lua_runtime.hpp` itself needs.
#include "abi.hpp"

#include "lua_runtime.hpp"

#include "common/syscalls/syscalls.h"

#if EPOK_LUA_MODE == 1
#include "abi_bytecode.h"
#else
#include "abi_chunk.h"
#endif

namespace {

constexpr uint64_t kProbeClassId = epok::detail::compact_class_id("6f1a7c20-5f77-4a41-9e0b-3f4f2c8a1d55");

// One fake Lua-backed object: two reflected fields (slot 0 Int32, slot 1 Bool)
// and the eight method slots the chunk declares.
class Probe : public epok::Object {
  public:
    static constexpr uint64_t static_class_id = kProbeClassId;
    uint64_t class_id() const override { return static_class_id; }

    int32_t value = 0;
    int32_t flag = 0;
    int32_t entries = 0;  // how many times a Lua body was actually entered

    // The trampolines lua_aot.rs generates for VM modes, written by hand.
    int32_t enter(uint32_t slot, int32_t argument) {
        epok::lua::Frame frame(*this, 0, slot);
        if (!frame.bound()) return 0;
        ++entries;
        frame.arg(argument);
        if (!frame.call(1)) return 0;
        return frame.ret();
    }
    bool bound(uint32_t slot) {
        epok::lua::Frame frame(*this, 0, slot);
        return frame.bound();
    }
};

Probe g_probe;
epok::ObjectRegistryStorage<4> g_registry;

int32_t get_field(epok::Object& object, uint32_t slot) {
    auto& probe = static_cast<Probe&>(object);
    return slot == 0 ? probe.value : probe.flag;
}
void set_field(epok::Object& object, uint32_t slot, int32_t value) {
    auto& probe = static_cast<Probe&>(object);
    (slot == 0 ? probe.value : probe.flag) = value;
}
// `__epok_call` goes through the C++ virtual, which re-enters Lua. That is the
// nesting path (Lua -> C++ -> Lua) the frame stack has to survive.
int32_t self_call(epok::Object& object, uint32_t slot, const int32_t* args, uint32_t argc) {
    return static_cast<Probe&>(object).enter(slot, argc ? args[0] : 0);
}
int32_t super_call(epok::Object&, uint32_t, const int32_t*, uint32_t) { return 0; }

const char* const kMethods[] = {"probe_iadd", "probe_idiv",    "probe_fmul",   "probe_ineg",
                                "probe_ult",  "probe_bool",    "probe_call",   "probe_absent"};

epok::Object* probe_create(void* storage) { return new (storage) Probe(); }
void probe_destroy(epok::Object* instance) {
    if (instance) instance->~Object();
}

}  // namespace

namespace epok {
// The cook emits this table next to the scene banks; the smoke program supplies
// its own so `object_class_is_a` can resolve the fake class.
const ClassDescriptor object_classes[] = {
    {kProbeClassId, Object::static_class_id, ObjectFamily::Object, ObjectDomain::None, 0, 0,
     probe_create, probe_destroy, sizeof(Probe), alignof(Probe), nullptr, nullptr, 0, nullptr},
};
const size_t object_class_count = 1;

namespace lua {
const ClassBinding class_bindings[] = {{
    kProbeClassId,
    "@abi_chunk.lua",
#if EPOK_LUA_MODE == 1
    ABI_BYTECODE,
    sizeof(ABI_BYTECODE),
#else
    reinterpret_cast<const unsigned char*>(ABI_CHUNK),
    sizeof(ABI_CHUNK) - 1,
#endif
    kMethods,
    8,
    get_field,
    set_field,
    nullptr,
    nullptr,
    self_call,
    super_call,
    nullptr,
    nullptr,
    nullptr,
    nullptr,
}};
const uint32_t class_binding_count = 1;
}  // namespace lua
}  // namespace epok

int main() {
    abi_probe[ABI_PROBE_MAGIC] = ABI_MAGIC;
    abi_probe[ABI_PROBE_MODE] = EPOK_LUA_MODE;
    abi_probe[ABI_PROBE_LOAD_OK] = 1;

    epok::active_object_registry = &g_registry;
    g_registry.adopt(g_probe, epok::object_classes[0]);

    epok::lua::initialize();

    // Numeric edge cases, all of them routed through the registered helpers and
    // therefore through the same epok::bp functions the AOT backend compiles to.
    abi_probe[ABI_PROBE_IADD] = g_probe.enter(0, 0x7fffffff);   // saturates
    abi_probe[ABI_PROBE_IDIV] = g_probe.enter(1, 7);            // division by zero is 0
    abi_probe[ABI_PROBE_FMUL] = g_probe.enter(2, 4097);         // truncates toward zero
    abi_probe[ABI_PROBE_INEG] = g_probe.enter(3, -2147483647 - 1);
    abi_probe[ABI_PROBE_ULT] = g_probe.enter(4, 0);             // 0x80000000 > 1 unsigned

    // Bool round trip: the field is 0/1 natively and a Lua boolean inside.
    g_probe.flag = 0;
    abi_probe[ABI_PROBE_BOOL_BEFORE] = g_probe.flag;
    abi_probe[ABI_PROBE_BOOL_AFTER] = g_probe.enter(5, 0);
    abi_probe[ABI_PROBE_BOOL_FIELD] = g_probe.flag;

    // Lua -> __epok_call -> C++ virtual -> Lua.
    abi_probe[ABI_PROBE_NESTED] = g_probe.enter(6, 41);

    // An absent handler must never enter the VM.
    const int32_t before = g_probe.entries;
    abi_probe[ABI_PROBE_ABSENT_BOUND] = g_probe.bound(7) ? 1 : 0;
    g_probe.enter(7, 1);
    abi_probe[ABI_PROBE_ABSENT_ENTRIES] = g_probe.entries - before;

    abi_probe[ABI_PROBE_ARENA_PEAK] = int32_t(epok::lua::stats.peak);
    abi_probe[ABI_PROBE_ARENA_LIVE] = int32_t(epok::lua::stats.live);
    abi_probe[ABI_PROBE_ARENA_ALLOCS] = int32_t(epok::lua::stats.allocations);
    abi_probe[ABI_PROBE_ERRORS] = int32_t(epok::lua::stats.failures);
    abi_probe[ABI_PROBE_DONE] = ABI_DONE;
    ramsyscall_printf("abi: smoke mode %d done\n", int(EPOK_LUA_MODE));
    abi_done();
    while (true) asm volatile("");
    return 0;
}
