/* Host-side cooker entry points shared by `epok_luac.c` and `epok_ldump32.c`. */
#ifndef EPOK_LUAC_H
#define EPOK_LUAC_H

#include <stddef.h>

#define EPOK_LUAC_HEADER_SIZE 18

/* Returns 0 to accept the block. Deliberately free of `lua_State` so the Rust
 * side never needs a Lua type. */
typedef int (*epok_luac_writer)(void *ctx, const void *bytes, size_t size);

/* Dumps an already compiled `Proto` at the target ABI. */
int epok_dump32(const void *proto, int strip, void *ctx, epok_luac_writer writer, char *err,
                size_t errcap);

/* Compiles `src` under the pinned parser and dumps it at the target ABI.
 * Returns 0 on success; on failure `err` carries the real cause. */
int epok_luac_cook(const char *name, const char *src, size_t len, int strip, void *ctx,
                   epok_luac_writer writer, char *err, size_t errcap);

#endif
