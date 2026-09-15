/* Lua 5.2's public lua_dump() has no strip parameter, so the target-side dump
 * calls the internal luaU_dump() directly. `strip` is passed in so it can match
 * `lua_bytecode::KEEP_DEBUG_INFO` exactly; the comparison would otherwise
 * differ for a reason that has nothing to do with the ABI. Nothing here is
 * Epok code: it uses the pinned psxlua internal headers. */
#define LUA_CORE

#include "lua.h"
#include "lobject.h"
#include "lstate.h"
#include "lundump.h"

int abi_dump_top(lua_State *L, lua_Writer w, void *data, int strip) {
    StkId o = L->top - 1;
    if (!ttisLclosure(o)) return -1;
    return luaU_dump(L, getproto(o), w, data, strip);
}
