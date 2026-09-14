/* Bytecode cooker support, compiled into the vm-parser build only.
 *
 * Lua 5.2's public lua_dump() has no strip parameter and always keeps debug
 * information. An offline cooker can call the internal luaU_dump() with
 * strip=1, so this shim does the same in order to produce comparable bytecode.
 * It uses the pinned psxlua internal headers; nothing here is Epok code.
 */
#define LUA_CORE

#include "lua.h"
#include "lobject.h"
#include "lstate.h"
#include "lundump.h"

int harness_dump_stripped(lua_State *L, lua_Writer w, void *data) {
    StkId o = L->top - 1;
    if (!ttisLclosure(o)) return -1;
    return luaU_dump(L, getproto(o), w, data, 1);
}
