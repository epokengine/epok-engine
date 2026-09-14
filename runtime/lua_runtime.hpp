#pragma once
// Lua VM integration for the two VM execution modes (contract §10.4).
//
// Compiled only when `EPOK_LUA_MODE != 0`. `libpsyqo-lua.a` is deliberately NOT
// linked: its constructor loads a *source* bootstrap chunk and opens the full
// standard library, neither of which a cooked Epok build wants. This header
// owns the state, the allocator and the libc shims the fork expects, and
// registers only the `__epok_*` helpers — there is no `load`, `require`,
// `dofile` or standard library in the resulting VM.
//
// Numeric parity with the AOT backend is not approximated: every arithmetic
// helper below forwards to the same `epok::bp::*` function the native mode
// compiles to, so the fork's own `long` arithmetic never sees a user value.
#include "lua-config.hh"

#if defined(EPOK_LUA_MODE) && EPOK_LUA_MODE != 0

#include "blueprint_runtime.hpp"
#include "object_model.hpp"

#include "common/syscalls/syscalls.h"
#include "psyqo/kernel.hh"
#include "psyqo/xprintf.h"

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

extern "C" {
#include "lauxlib.h"
#include "lua.h"
}

// Static VM budget. The runtime has no heap, so this is the whole Lua world:
// states, chunks, strings and call frames all live here. Override from
// `sources.mk` when a project's class set outgrows it; exhaustion aborts.
#ifndef EPOK_LUA_ARENA_BYTES
#define EPOK_LUA_ARENA_BYTES (96 * 1024)
#endif
// Bound on the generated binding table. The bitmask below is 64 bits wide, so
// a class may carry at most 64 Lua-bodied methods.
#ifndef EPOK_LUA_MAX_CLASSES
#define EPOK_LUA_MAX_CLASSES 64
#endif
#define EPOK_LUA_MAX_SLOTS 64
#define EPOK_LUA_MAX_ARGS 8
#define EPOK_LUA_MAX_DEPTH 16

namespace epok::lua {

struct ClassBinding {
    uint64_t class_id;
    const char* name;
    const unsigned char* chunk;
    size_t chunk_size;
    const char* const* methods;
    uint32_t method_count;
    int32_t (*get_field)(Object&, uint32_t slot);
    void (*set_field)(Object&, uint32_t slot, int32_t value);
    int32_t (*self_call)(Object&, uint32_t slot, const int32_t* args, uint32_t argc);
    int32_t (*super_call)(Object&, uint32_t slot, const int32_t* args, uint32_t argc);
};
// Defined by the generated `scripts/generated/lua/lua_bindings.cpp`.
extern const ClassBinding class_bindings[];
extern const uint32_t class_binding_count;

// Reported in the build report; `peak` is what an arena budget must cover.
struct Stats {
    uint32_t live = 0, peak = 0, allocations = 0, frees = 0, reallocs = 0, failures = 0;
    uint32_t arena_bytes = EPOK_LUA_ARENA_BYTES;
};
inline Stats stats;

namespace detail {

// ---------------------------------------------------------------------------
// Arena allocator
// ---------------------------------------------------------------------------
// A first-fit free list over one static buffer, with forward coalescing during
// the scan. Lua hands the allocator both the old and the new size, but the
// block header is kept anyway so `realloc` can grow in place when the
// following block is free — the common case for a growing string buffer.

struct Block {
    uint32_t size;  // payload bytes, always a multiple of 8
    uint32_t free;
};
constexpr uint32_t kAlign = 8;
constexpr uint32_t kHeader = uint32_t(sizeof(Block));
static_assert(kHeader % kAlign == 0, "The block header must preserve payload alignment");

alignas(8) inline unsigned char arena[EPOK_LUA_ARENA_BYTES];
inline bool arena_ready = false;

inline unsigned char* arena_end() { return arena + EPOK_LUA_ARENA_BYTES; }
inline Block* block_of(void* payload) {
    return reinterpret_cast<Block*>(static_cast<unsigned char*>(payload) - kHeader);
}
inline unsigned char* payload_of(Block* block) {
    return reinterpret_cast<unsigned char*>(block) + kHeader;
}
inline Block* next_block(Block* block) {
    unsigned char* next = payload_of(block) + block->size;
    return next < arena_end() ? reinterpret_cast<Block*>(next) : nullptr;
}
inline uint32_t round_up(size_t bytes) {
    return uint32_t((bytes + kAlign - 1) & ~size_t(kAlign - 1));
}

inline void arena_initialize() {
    auto* first = reinterpret_cast<Block*>(arena);
    first->size = EPOK_LUA_ARENA_BYTES - kHeader;
    first->free = 1;
    arena_ready = true;
}

[[noreturn]] inline void exhausted(size_t request) {
    static char message[128];
    sprintf(message,
            "Lua arena exhausted: %u bytes requested, %u of %u live. Raise EPOK_LUA_ARENA_BYTES.",
            unsigned(request), unsigned(stats.live), unsigned(EPOK_LUA_ARENA_BYTES));
    psyqo::Kernel::abort(message);
}

inline void split(Block* block, uint32_t want) {
    if (block->size < want + kHeader + kAlign) return;
    auto* rest = reinterpret_cast<Block*>(payload_of(block) + want);
    rest->size = block->size - want - kHeader;
    rest->free = 1;
    block->size = want;
}

inline void* arena_alloc(size_t bytes) {
    if (!arena_ready) arena_initialize();
    const uint32_t want = round_up(bytes ? bytes : 1);
    for (auto* block = reinterpret_cast<Block*>(arena); block; block = next_block(block)) {
        if (!block->free) continue;
        // Coalesce forward before testing the fit, so a run of freed blocks is
        // usable again without a separate compaction pass.
        for (auto* after = next_block(block); after && after->free; after = next_block(block))
            block->size += kHeader + after->size;
        if (block->size < want) continue;
        split(block, want);
        block->free = 0;
        stats.live += block->size;
        ++stats.allocations;
        if (stats.live > stats.peak) stats.peak = stats.live;
        return payload_of(block);
    }
    ++stats.failures;
    return nullptr;
}

inline void arena_free(void* payload) {
    if (!payload) return;
    auto* block = block_of(payload);
    if (block->free) return;
    block->free = 1;
    stats.live -= block->size;
    ++stats.frees;
}

inline void* arena_realloc(void* payload, size_t bytes) {
    if (!payload) return arena_alloc(bytes);
    if (bytes == 0) {
        arena_free(payload);
        return nullptr;
    }
    auto* block = block_of(payload);
    const uint32_t want = round_up(bytes);
    const uint32_t before = block->size;
    if (want <= before) {
        split(block, want);
        stats.live -= before - block->size;
        ++stats.reallocs;
        return payload;
    }
    // Grow in place across the following free blocks. A growing string buffer
    // is the common case and this keeps it from fragmenting the arena.
    for (auto* after = next_block(block); after && after->free && block->size < want;
         after = next_block(block))
        block->size += kHeader + after->size;
    stats.live += block->size - before;
    if (stats.live > stats.peak) stats.peak = stats.live;
    if (block->size >= want) {
        const uint32_t grown = block->size;
        split(block, want);
        stats.live -= grown - block->size;
        ++stats.reallocs;
        return payload;
    }
    // Still short: the absorbed blocks stay with this allocation until it is
    // released, and the copy below is bounded by the (smaller) current size.
    void* fresh = arena_alloc(bytes);
    if (!fresh) return nullptr;
    __builtin_memcpy(fresh, payload, block->size);
    arena_free(payload);
    return fresh;
}

// Lua's allocator contract. `osize` is a type tag when `ptr` is null.
inline void* lua_allocator(void*, void* ptr, size_t, size_t nsize) {
    if (nsize == 0) {
        arena_free(ptr);
        return nullptr;
    }
    void* result = ptr ? arena_realloc(ptr, nsize) : arena_alloc(nsize);
    if (!result) exhausted(nsize);
    return result;
}

}  // namespace detail

}  // namespace epok::lua

// ---------------------------------------------------------------------------
// libc shims the fork's PSX target expects. `psyqo-lua.mk` binds these through
// --defsym against the psyqo heap; this runtime owns its arena instead, so it
// defines them directly and links no psyqo-lua object at all.
// ---------------------------------------------------------------------------
// `used` forces emission: these are referenced only from the archive's objects,
// so an ordinary inline definition would be discarded before the link sees it.
extern "C" {
[[gnu::used]] inline int luaI_sprintf(char* buf, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    const int written = vsprintf(buf, fmt, ap);
    va_end(ap);
    return written;
}
[[gnu::used]] inline void* luaI_realloc(void* ptr, size_t size) {
    return ptr ? epok::lua::detail::arena_realloc(ptr, size) : epok::lua::detail::arena_alloc(size);
}
[[gnu::used]] inline void luaI_free(void* ptr) { epok::lua::detail::arena_free(ptr); }
}

namespace epok::lua {

namespace detail {

inline lua_State* state = nullptr;
inline uint64_t bound_masks[EPOK_LUA_MAX_CLASSES] = {};
// One report per (class, slot): a failing handler must not flood the link.
inline uint64_t reported[EPOK_LUA_MAX_CLASSES] = {};

struct FrameState {
    Object* object;
    uint32_t class_index;
};
inline FrameState frames[EPOK_LUA_MAX_DEPTH] = {};
inline uint32_t depth = 0;

// Registry keys start past the reserved indices (`LUA_RIDX_LAST`) and past
// `luaL_ref`'s free-list slot 0, which nothing here uses but which the fork
// still reserves.
constexpr int kRegistryBase = 16;
inline int registry_key(uint32_t class_index, uint32_t slot) {
    return kRegistryBase + int(class_index) * EPOK_LUA_MAX_SLOTS + int(slot);
}

[[noreturn]] inline int panic(lua_State* L) {
    static char message[192];
    const char* text = lua_tostring(L, -1);
    sprintf(message, "Lua panic: %s", text ? text : "(no message)");
    psyqo::Kernel::abort(message);
}

inline int32_t argument(lua_State* L, int index) { return int32_t(lua_tonumber(L, index)); }
inline void push_int(lua_State* L, int32_t value) { lua_pushnumber(L, lua_Number(value)); }

// ---- self resolution ------------------------------------------------------

// `self` is a light userdata for the duration of one call. It is only ever the
// object the innermost frame entered with; anything else is a Lua error, not a
// native dereference of an unchecked pointer.
inline Object* current_self(lua_State* L, int index) {
    if (!lua_islightuserdata(L, index)) {
        luaL_error(L, "epok: self is not an Epok object");
        return nullptr;
    }
    auto* object = static_cast<Object*>(lua_touserdata(L, index));
    if (depth == 0 || frames[depth - 1].object != object) {
        luaL_error(L, "epok: self does not belong to the active call");
        return nullptr;
    }
    if (active_object_registry && active_object_registry->get(object->id()) != object) {
        luaL_error(L, "epok: self was destroyed during the call");
        return nullptr;
    }
    return object;
}

// The binding for an object's class. A Blueprint or C++ class deriving from a
// Lua class has its own class id, so the lookup walks the ancestry and picks
// the most derived binding the object actually is.
inline const ClassBinding* binding_for(Object* object) {
    const uint64_t id = object->class_id();
    if (depth > 0) {
        const uint32_t index = frames[depth - 1].class_index;
        if (index < class_binding_count && object_class_is_a(id, class_bindings[index].class_id))
            return &class_bindings[index];
    }
    const ClassBinding* best = nullptr;
    for (uint32_t i = 0; i < class_binding_count; ++i) {
        if (!object_class_is_a(id, class_bindings[i].class_id)) continue;
        if (!best || object_class_is_a(class_bindings[i].class_id, best->class_id))
            best = &class_bindings[i];
    }
    return best;
}

// ---- registered helpers ---------------------------------------------------

#define EPOK_LUA_BINARY(name, expression)                                   \
    inline int name(lua_State* L) {                                         \
        const int32_t a = argument(L, 1), b = argument(L, 2);               \
        push_int(L, int32_t(expression));                                   \
        return 1;                                                           \
    }
EPOK_LUA_BINARY(h_iadd, bp::iadd(a, b))
EPOK_LUA_BINARY(h_isub, bp::isub(a, b))
EPOK_LUA_BINARY(h_imul, bp::imul(a, b))
EPOK_LUA_BINARY(h_idiv, bp::idiv(a, b))
EPOK_LUA_BINARY(h_imod, bp::imod(a, b))
EPOK_LUA_BINARY(h_uadd, bp::uadd(uint32_t(a), uint32_t(b)))
EPOK_LUA_BINARY(h_usub, bp::usub(uint32_t(a), uint32_t(b)))
EPOK_LUA_BINARY(h_umul, bp::umul(uint32_t(a), uint32_t(b)))
EPOK_LUA_BINARY(h_udiv, bp::udiv(uint32_t(a), uint32_t(b)))
EPOK_LUA_BINARY(h_umod, bp::umod(uint32_t(a), uint32_t(b)))
EPOK_LUA_BINARY(h_fadd, bp::add(Fixed(a, Fixed::RAW), Fixed(b, Fixed::RAW)).raw())
EPOK_LUA_BINARY(h_fsub, bp::sub(Fixed(a, Fixed::RAW), Fixed(b, Fixed::RAW)).raw())
EPOK_LUA_BINARY(h_fmul, bp::mul(Fixed(a, Fixed::RAW), Fixed(b, Fixed::RAW)).raw())
EPOK_LUA_BINARY(h_fdiv, bp::div(Fixed(a, Fixed::RAW), Fixed(b, Fixed::RAW)).raw())
#undef EPOK_LUA_BINARY

inline int h_ineg(lua_State* L) {
    push_int(L, bp::ineg(argument(L, 1)));
    return 1;
}
inline int h_fneg(lua_State* L) {
    push_int(L, bp::neg(Fixed(argument(L, 1), Fixed::RAW)).raw());
    return 1;
}
inline int h_from_int(lua_State* L) {
    push_int(L, bp::from_int(argument(L, 1)).raw());
    return 1;
}
inline int h_to_int(lua_State* L) {
    push_int(L, bp::to_int(Fixed(argument(L, 1), Fixed::RAW)));
    return 1;
}
// UInt32 values travel as their bit pattern in a signed 32-bit number, so
// ordering has to be explicit rather than Lua's signed comparison.
inline int h_ult(lua_State* L) {
    lua_pushboolean(L, uint32_t(argument(L, 1)) < uint32_t(argument(L, 2)));
    return 1;
}
inline int h_ule(lua_State* L) {
    lua_pushboolean(L, uint32_t(argument(L, 1)) <= uint32_t(argument(L, 2)));
    return 1;
}

inline int h_getf(lua_State* L) {
    Object* object = current_self(L, 1);
    const ClassBinding* binding = binding_for(object);
    const uint32_t slot = uint32_t(argument(L, 2));
    if (!binding || !binding->get_field) return luaL_error(L, "epok: no field binding");
    push_int(L, binding->get_field(*object, slot));
    return 1;
}
inline int h_setf(lua_State* L) {
    Object* object = current_self(L, 1);
    const ClassBinding* binding = binding_for(object);
    const uint32_t slot = uint32_t(argument(L, 2));
    if (!binding || !binding->set_field) return luaL_error(L, "epok: no field binding");
    binding->set_field(*object, slot, argument(L, 3));
    return 0;
}

inline int dispatch(lua_State* L, bool super) {
    Object* object = current_self(L, 1);
    const ClassBinding* binding = binding_for(object);
    if (!binding) return luaL_error(L, "epok: no class binding");
    const uint32_t slot = uint32_t(argument(L, 2));
    const int count = lua_gettop(L) - 2;
    if (count < 0 || count > EPOK_LUA_MAX_ARGS)
        return luaL_error(L, "epok: call arity %d is outside the profile", count);
    int32_t args[EPOK_LUA_MAX_ARGS];
    for (int i = 0; i < count; ++i) args[i] = argument(L, 3 + i);
    auto* entry = super ? binding->super_call : binding->self_call;
    if (!entry) return luaL_error(L, "epok: no dispatch binding");
    push_int(L, entry(*object, slot, args, uint32_t(count)));
    return 1;
}
inline int h_call(lua_State* L) { return dispatch(L, false); }
inline int h_super(lua_State* L) { return dispatch(L, true); }

inline void register_helper(lua_State* L, const char* name, lua_CFunction fn) {
    lua_pushcfunction(L, fn);
    lua_setglobal(L, name);
}

// ---- chunk loading --------------------------------------------------------

struct Reader {
    const unsigned char* bytes;
    size_t size;
};
inline const char* read_chunk(lua_State*, void* data, size_t* size) {
    auto* reader = static_cast<Reader*>(data);
    if (reader->size == 0) {
        *size = 0;
        return nullptr;
    }
    *size = reader->size;
    reader->size = 0;
    return reinterpret_cast<const char*>(reader->bytes);
}

}  // namespace detail

/// Opens the VM, registers the helpers and loads every class chunk once.
/// Runs in `GameScene::start` after `install_actor_service_hooks()` and before
/// `epok::initialize_scripts()`. Any failure aborts with the real cause: a VM
/// build that started with an unloadable chunk would fail later, in a place
/// that says nothing about why.
inline void initialize() {
    using namespace detail;
    if (state) return;
    if (class_binding_count > EPOK_LUA_MAX_CLASSES)
        psyqo::Kernel::abort("Lua binding table exceeds EPOK_LUA_MAX_CLASSES");
    arena_initialize();
    state = lua_newstate(lua_allocator, nullptr);
    if (!state) psyqo::Kernel::abort("Lua state could not be created inside the arena");
    lua_atpanic(state, panic);

    lua_State* L = state;
    register_helper(L, "__epok_iadd", h_iadd);
    register_helper(L, "__epok_isub", h_isub);
    register_helper(L, "__epok_imul", h_imul);
    register_helper(L, "__epok_idiv", h_idiv);
    register_helper(L, "__epok_imod", h_imod);
    register_helper(L, "__epok_ineg", h_ineg);
    register_helper(L, "__epok_uadd", h_uadd);
    register_helper(L, "__epok_usub", h_usub);
    register_helper(L, "__epok_umul", h_umul);
    register_helper(L, "__epok_udiv", h_udiv);
    register_helper(L, "__epok_umod", h_umod);
    register_helper(L, "__epok_fadd", h_fadd);
    register_helper(L, "__epok_fsub", h_fsub);
    register_helper(L, "__epok_fmul", h_fmul);
    register_helper(L, "__epok_fdiv", h_fdiv);
    register_helper(L, "__epok_fneg", h_fneg);
    register_helper(L, "__epok_from_int", h_from_int);
    register_helper(L, "__epok_to_int", h_to_int);
    register_helper(L, "__epok_ult", h_ult);
    register_helper(L, "__epok_ule", h_ule);
    register_helper(L, "__epok_getf", h_getf);
    register_helper(L, "__epok_setf", h_setf);
    register_helper(L, "__epok_call", h_call);
    register_helper(L, "__epok_super", h_super);

    // Mode 1 loads bytecode only; the no-parser core rejects text with its own
    // message, which is propagated rather than replaced.
#if EPOK_LUA_MODE == 1
    const char* const mode = "b";
#else
    const char* const mode = "t";
#endif
    static char message[192];
    for (uint32_t index = 0; index < class_binding_count; ++index) {
        const ClassBinding& binding = class_bindings[index];
        if (binding.method_count > EPOK_LUA_MAX_SLOTS)
            psyqo::Kernel::abort("A Lua class exceeds the 64-method slot budget");
        Reader reader{binding.chunk, binding.chunk_size};
        if (lua_load(L, read_chunk, &reader, binding.name, mode) != 0) {
            sprintf(message, "Lua chunk %s was rejected: %s", binding.name, lua_tostring(L, -1));
            psyqo::Kernel::abort(message);
        }
        if (lua_pcall(L, 0, 1, 0) != 0) {
            sprintf(message, "Lua chunk %s failed to run: %s", binding.name, lua_tostring(L, -1));
            psyqo::Kernel::abort(message);
        }
        if (!lua_istable(L, -1)) {
            sprintf(message, "Lua chunk %s did not return its class table", binding.name);
            psyqo::Kernel::abort(message);
        }
        uint64_t mask = 0;
        for (uint32_t slot = 0; slot < binding.method_count; ++slot) {
            lua_getfield(L, -1, binding.methods[slot]);
            if (lua_isfunction(L, -1)) {
                // Cached under an integer registry key: dispatch is one
                // `lua_rawgeti`, never a name lookup in a table.
                lua_rawseti(L, LUA_REGISTRYINDEX, registry_key(index, slot));
                mask |= uint64_t(1) << slot;
            } else {
                lua_pop(L, 1);
            }
        }
        bound_masks[index] = mask;
        lua_pop(L, 1);  // the class table itself is not retained
    }
    ramsyscall_printf("EPOK: Lua VM ready, %u classes, %u/%u arena bytes live\n",
                      unsigned(class_binding_count), unsigned(stats.live),
                      unsigned(EPOK_LUA_ARENA_BYTES));
}

/// One call into Lua. Absent handlers never enter the VM: `bound()` is a
/// bitmask test against the table built by `initialize()`.
class Frame {
  public:
    Frame(Object& self, uint32_t class_index, uint32_t slot)
        : m_class(class_index), m_slot(slot) {
        if (!detail::state || class_index >= class_binding_count ||
            slot >= EPOK_LUA_MAX_SLOTS ||
            !(detail::bound_masks[class_index] & (uint64_t(1) << slot)))
            return;
        if (detail::depth >= EPOK_LUA_MAX_DEPTH)
            psyqo::Kernel::abort("Lua call depth exceeded EPOK_LUA_MAX_DEPTH");
        m_bound = true;
        // Storage of an object destroyed inside this call stays quarantined
        // until the scope unwinds, exactly as a native dispatch would.
        if (active_object_registry) {
            m_registry = active_object_registry;
            ++m_registry->dispatch;
        }
        detail::frames[detail::depth++] = {&self, class_index};
        lua_State* L = detail::state;
        m_top = lua_gettop(L);
        lua_rawgeti(L, LUA_REGISTRYINDEX, detail::registry_key(class_index, slot));
        lua_pushlightuserdata(L, &self);
        m_args = 1;
    }
    ~Frame() {
        if (!m_bound) return;
        lua_settop(detail::state, m_top);
        if (detail::depth) --detail::depth;
        if (m_registry) {
            if (m_registry->dispatch) --m_registry->dispatch;
            m_registry->finish_release();
        }
    }
    Frame(const Frame&) = delete;
    Frame& operator=(const Frame&) = delete;

    bool bound() const { return m_bound; }
    void arg(int32_t value) {
        if (!m_bound) return;
        detail::push_int(detail::state, value);
        ++m_args;
    }
    bool call(unsigned results) {
        if (!m_bound) return false;
        lua_State* L = detail::state;
        if (lua_pcall(L, m_args, int(results), 0) != 0) {
            const uint64_t bit = uint64_t(1) << m_slot;
            if (!(detail::reported[m_class] & bit)) {
                detail::reported[m_class] |= bit;
                ramsyscall_printf("EPOK: Lua error in %s.%s: %s\n", class_bindings[m_class].name,
                                  class_bindings[m_class].methods[m_slot], lua_tostring(L, -1));
            }
            lua_settop(L, m_top);
            m_args = 0;
            m_results = 0;
            return false;
        }
        m_args = 0;
        m_results = int(results);
        return true;
    }
    /// The single result of the call. Booleans fold to 0/1 so the trampolines
    /// read `Bool` and the numeric types through one path.
    int32_t ret() const {
        if (!m_bound || m_results <= 0) return 0;
        lua_State* L = detail::state;
        if (lua_isboolean(L, -1)) return lua_toboolean(L, -1) ? 1 : 0;
        return int32_t(lua_tonumber(L, -1));
    }

  private:
    uint32_t m_class = 0, m_slot = 0;
    ObjectRegistry* m_registry = nullptr;
    int m_top = 0, m_args = 0, m_results = 0;
    bool m_bound = false;
};

}  // namespace epok::lua

#endif  // EPOK_LUA_MODE
