#pragma once

// Lua sources for the seven workloads, one chunk each. Every chunk evaluates to
// a table of handlers so the same bytes can be loaded once and shared (parser /
// no-parser variants) or re-loaded per instance (cached-dispatch variant).
//
// Numeric notes:
//  * lua_Number in this psxlua fork is `long` (32-bit, integral on MIPS o32),
//    so these chunks operate on raw Q12 integers directly. There is no
//    FixedPoint table allocated per scalar.
//  * Every expression here is deliberately bounded so that a 32-bit wrapping
//    Lua multiply and the native saturating int64 contract agree bit for bit.
//    That is a property of these workloads, not of Lua: see the README.

namespace harness {

inline constexpr const char* LUA_SOURCES[] = {
    // absent: no handler at all.
    "return {}\n",

    // empty: handler present, empty body.
    "return { update = function(self, dt) end }\n",

    // arith: Q12 multiply/add plus an integer branch.
    "return { update = function(self, dt)\n"
    "  local v = self.v\n"
    "  v = (v * 4090) / 4096 + 16\n"
    "  local n = self.n + dt\n"
    "  if v > 10000 then n = n + 2 else n = n + 1 end\n"
    "  self.v = v\n"
    "  self.n = n\n"
    "end }\n",

    // native_call: one call into a native scalar method.
    "return { update = function(self, dt)\n"
    "  self.v = native_scale(self.v, 4090) + 16\n"
    "  self.n = self.n + dt\n"
    "end }\n",

    // position: get a 3-component position, modify it, set it back.
    "return { update = function(self, dt)\n"
    "  local p = get_position(self)\n"
    "  p.x = p.x + dt\n"
    "  p.y = p.y - dt\n"
    "  p.z = p.z + 1\n"
    "  set_position(self, p)\n"
    "  self.n = self.n + 1\n"
    "end }\n",

    // alloc: allocation-heavy per-step state.
    "return { update = function(self, dt)\n"
    "  local t = { a = self.v, b = self.n, c = self.v + dt, d = {1, 2, 3} }\n"
    "  local s = t.a + t.c + t.d[1] + t.d[2] + t.d[3]\n"
    "  self.n = self.n + (s - t.a - t.c)\n"
    "  self.v = t.c\n"
    "end }\n",

    // event_only: no update handler; a discrete event every EVENT_PERIOD steps.
    "return { event = function(self, dt)\n"
    "  self.n = self.n + 1\n"
    "end }\n",
};

inline constexpr const char* LUA_CHUNK_NAMES[] = {
    "=absent", "=empty", "=arith", "=native_call", "=position", "=alloc", "=event_only",
};

}  // namespace harness
