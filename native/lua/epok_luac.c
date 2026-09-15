/* Compiles one normalized Epok chunk with the pinned psxlua parser and writes
 * it out at the PSX target's bytecode ABI (`epok_ldump32.c`).
 *
 * The parser, lexer and code generator are the submodule's own sources, built
 * here for the host. Only the dumper differs from an on-target `luaU_dump`, and
 * `tests/integration/verify_lua_vm_abi.py` byte-compares the two.
 */
#include <stddef.h>
#include <string.h>

#define LUA_CORE

#include "lua.h"

#include "lauxlib.h"
#include "lobject.h"
#include "lstate.h"

#include "epok_luac.h"

static void report(char *err, size_t errcap, const char *message) {
    size_t n;
    if (!err || !errcap) return;
    if (!message) message = "unknown error";
    n = strlen(message);
    if (n >= errcap) n = errcap - 1;
    memcpy(err, message, n);
    err[n] = '\0';
}

int epok_luac_cook(const char *name, const char *src, size_t len, int strip, void *ctx,
                   epok_luac_writer writer, char *err, size_t errcap) {
    lua_State *L;
    StkId top;
    int status;
    if (err && errcap) err[0] = '\0';
    L = luaL_newstate();
    if (L == NULL) {
        report(err, errcap, "could not create the host Lua state");
        return 1;
    }
    status = luaL_loadbuffer(L, src, len, name);
    if (status != 0) {
        report(err, errcap, lua_tostring(L, -1));
        lua_close(L);
        return 1;
    }
    top = L->top - 1;
    if (!ttisLclosure(top)) {
        report(err, errcap, "the compiled chunk is not a Lua closure");
        lua_close(L);
        return 1;
    }
    status = epok_dump32(getproto(top), strip, ctx, writer, err, errcap);
    lua_close(L);
    return status;
}
