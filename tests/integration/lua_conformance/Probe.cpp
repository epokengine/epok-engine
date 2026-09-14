// Fixture for tests/integration/verify_lua_modes.py. See EnemyBase.hpp.
#include "EnemyBase.hpp"
#include "blueprint_runtime.hpp"
#include "lua-config.hh"
#if EPOK_LUA_MODE != EPOK_LUA_MODE_NATIVE
#include "lua_runtime.hpp"
#endif

extern "C" {
volatile int32_t epok_lua_probe[64] = {};
__attribute__((noinline, used)) void epok_lua_probe_begin() { asm volatile("nop" ::: "memory"); }
__attribute__((noinline, used)) void epok_lua_probe_end() { asm volatile("nop\nnop" ::: "memory"); }
__attribute__((noinline, used)) void epok_lua_probe_done() { asm volatile("nop\nnop\nnop" ::: "memory"); }
}

// --- probe layout (mirrored by the Python driver) --------------------------
// 0..15   Guard1 (slot 0): lifecycle and inherited state
// 16..31  Guard1 only: the numeric edge battery
// 32..47  Patrol1 (slot 32): the same lifecycle probes through Lua->Lua
// 48..63  native and global observations
enum : int32_t {
    P_MAGIC = 48, P_NATIVE_BEGIN = 49, P_NATIVE_DAMAGE = 50, P_BUMPS = 51,
    P_EVAL_ORDER = 52, P_FOR_SUM = 53, P_PAUSE_DELTA = 54, P_DEACTIVATE_DELTA = 55,
    P_DESTROY_DELTA = 56, P_DIRECTOR_TICKS = 57, P_ARENA_PEAK = 58, P_ARENA_LIVE = 59,
    P_ARENA_ALLOCS = 60, P_ARENA_FAILURES = 61, P_LUA_MODE = 62, P_DONE = 63,
};
static constexpr int32_t MAGIC = 0x4C4D4F31;   // 'LMO1'
static constexpr int32_t DONE = 0x0000D09E;

namespace {
EnemyBase* g_enemies[4] = {};
uint32_t g_enemy_count = 0;
int32_t g_next_id = 0;
}

void EnemyBase::damage(epok::Fixed amount) {
    health = epok::bp::sub(health, amount);
    hits = epok::bp::iadd(hits, 1);
    epok_lua_probe[P_NATIVE_DAMAGE] = epok_lua_probe[P_NATIVE_DAMAGE] + 1;
}

void EnemyBase::probe(int32_t index, int32_t value) {
    if (index >= 0 && index < 64) epok_lua_probe[index] = value;
}
void EnemyBase::probe_u(int32_t index, uint32_t value) {
    if (index >= 0 && index < 64) epok_lua_probe[index] = int32_t(value);
}
void EnemyBase::probe_f(int32_t index, epok::Fixed value) {
    if (index >= 0 && index < 64) epok_lua_probe[index] = value.raw();
}
void EnemyBase::probe_b(int32_t index, bool value) {
    if (index >= 0 && index < 64) epok_lua_probe[index] = value ? 1 : 0;
}
bool EnemyBase::bump() {
    epok_lua_probe[P_BUMPS] = epok_lua_probe[P_BUMPS] + 1;
    return true;
}
int32_t EnemyBase::next_id() { return ++g_next_id; }
void EnemyBase::mark_begin() { epok_lua_probe_begin(); }
void EnemyBase::mark_end() { epok_lua_probe_end(); }

void EnemyBase::begin_play() {
    epok_lua_probe[P_MAGIC] = MAGIC;
    epok_lua_probe[P_NATIVE_BEGIN] = epok_lua_probe[P_NATIVE_BEGIN] + 1;
    epok_lua_probe[P_LUA_MODE] = EPOK_LUA_MODE;
    if (g_enemy_count < 4) g_enemies[g_enemy_count++] = this;
    // Both calls go through the BASE type: whatever a subclass overrides in Lua
    // has to be reached by an ordinary C++ virtual call in every mode.
    EnemyBase* base = this;
    base->damage(epok::Fixed(2048, epok::Fixed::RAW));
    base->on_alert();
}

// --- director ---------------------------------------------------------------
// Every phase boundary is keyed on an exact tick index, never on a frame count,
// so a slower build cannot reach a different state: `steps` per rendered frame
// varies with the mode, tick indices do not. Only the pause release lives in
// `frame_update`, because no actor ticks at all while `epok::time` is paused.
namespace {
constexpr uint32_t PAUSE_AT_TICK = 131;     // 130 measured ticks first
constexpr uint32_t PAUSE_FRAMES = 30;
constexpr uint32_t DEACTIVATE_AT_TICK = 141;
constexpr uint32_t DESTROY_AT_TICK = 151;
constexpr uint32_t FINAL_TICK = 161;

epok::Level* level_of(epok::ActorComponent& component) {
    if (!epok::active_object_registry) return nullptr;
    auto* owner = component.get_owner();
    if (!owner) return nullptr;
    return epok::active_object_registry->resolve<epok::Level>(owner->level_id());
}
uint32_t g_ticks = 0;
uint32_t g_pause_frames = 0;
bool g_paused = false;
int32_t g_before_pause = 0, g_before_deactivate = 0, g_before_destroy = 0;
int32_t guard_ticks() { return epok_lua_probe[7]; }
}

void Director::tick(epok::Fixed) {
    ++g_ticks;
    auto* level = level_of(*this);
    if (g_ticks == PAUSE_AT_TICK) {
        // The enemies tick after this component inside the same step, so the
        // reference count is taken on the first paused frame, not here.
        g_before_pause = -1;
        g_pause_frames = 0;
        g_paused = true;
        epok::time.set_paused(true);
    } else if (g_ticks == DEACTIVATE_AT_TICK) {
        g_before_deactivate = guard_ticks();
        if (level && g_enemy_count > 0) level->set_active(g_enemies[0]->id(), false);
    } else if (g_ticks == DESTROY_AT_TICK) {
        epok_lua_probe[P_DEACTIVATE_DELTA] = guard_ticks() - g_before_deactivate;
        g_before_destroy = guard_ticks();
        if (level && g_enemy_count > 0) level->destroy_actor(g_enemies[0]->id());
    } else if (g_ticks == FINAL_TICK) {
        epok_lua_probe[P_DESTROY_DELTA] = guard_ticks() - g_before_destroy;
        epok_lua_probe[P_DIRECTOR_TICKS] = int32_t(g_ticks);
#if EPOK_LUA_MODE != EPOK_LUA_MODE_NATIVE
        epok_lua_probe[P_ARENA_PEAK] = int32_t(epok::lua::stats.peak);
        epok_lua_probe[P_ARENA_LIVE] = int32_t(epok::lua::stats.live);
        epok_lua_probe[P_ARENA_ALLOCS] = int32_t(epok::lua::stats.allocations);
        epok_lua_probe[P_ARENA_FAILURES] = int32_t(epok::lua::stats.failures);
#endif
        epok_lua_probe[P_DONE] = DONE;
        epok::time.set_paused(true);
        epok_lua_probe_done();
    }
}

void Director::frame_update(uint32_t) {
    ++m_frame;
    if (!g_paused) return;
    if (g_before_pause < 0) g_before_pause = guard_ticks();
    if (++g_pause_frames < PAUSE_FRAMES) return;
    epok_lua_probe[P_PAUSE_DELTA] = guard_ticks() - g_before_pause;
    g_paused = false;
    epok::time.set_paused(false);
}
