// Target side of the bytecode ABI check.
//
// Compiles the normalized chunk with the pinned parser ON TARGET and dumps it
// with the fork's own `luaU_dump`. The driver reads the bytes back out of guest
// RAM and compares them with what `native/lua/epok_ldump32.c` produced on the
// host. This is the only way to establish that the host cooker writes the ABI
// the console actually loads, rather than assuming it.
//
// This program deliberately does NOT include `lua_runtime.hpp`: it must be able
// to dump even if the runtime is broken.
#include "abi.hpp"

#include "common/syscalls/syscalls.h"
#include "psyqo/alloc.h"
#include "psyqo/xprintf.h"

#include <stdarg.h>

extern "C" {
#include "lauxlib.h"
#include "lua.h"

void luaU_header(unsigned char* h);
int abi_dump_top(lua_State* L, lua_Writer w, void* data, int strip);

// The fork's PSX libc hooks. This build uses the psyqo heap rather than the
// runtime's arena, because it is measuring the dumper and not the runtime.
int luaI_sprintf(char* buf, const char* fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    const int written = vsprintf(buf, fmt, ap);
    va_end(ap);
    return written;
}
void luaI_free(void* ptr) { psyqo_free(ptr); }
void* luaI_realloc(void* ptr, size_t size) { return psyqo_realloc(ptr, size); }
}

#include "abi_chunk.h"

namespace {
void* allocator(void*, void* ptr, size_t, size_t nsize) {
    if (nsize == 0) {
        psyqo_free(ptr);
        return nullptr;
    }
    return ptr ? psyqo_realloc(ptr, nsize) : psyqo_malloc(nsize);
}

int writer(lua_State*, const void* bytes, size_t size, void* ud) {
    auto* cursor = static_cast<int32_t*>(ud);
    if (*cursor + int32_t(size) > int32_t(sizeof(abi_bytecode))) return 1;
    __builtin_memcpy(abi_bytecode + *cursor, bytes, size);
    *cursor += int32_t(size);
    return 0;
}
}  // namespace

int main() {
    abi_probe[ABI_PROBE_MAGIC] = ABI_MAGIC;
    abi_probe[ABI_PROBE_MODE] = 3;  // dumper, not a VM mode
    abi_probe[ABI_PROBE_LOAD_OK] = 1;

    // The header this build accepts, recorded byte by byte. It is the authority
    // on the ABI, so the driver decodes it rather than trusting a constant.
    unsigned char header[18] = {};
    luaU_header(header);
    abi_probe[ABI_PROBE_HEADER_SIZE] = int32_t(sizeof(header));
    for (int i = 0; i < int(sizeof(header)); ++i) abi_probe[ABI_PROBE_HEADER + i] = header[i];

    lua_State* L = lua_newstate(allocator, nullptr);
    if (!L) {
        abi_probe[ABI_PROBE_LOAD_OK] = 0;
        abi_probe[ABI_PROBE_DONE] = ABI_DONE;
        abi_done();
        while (true) asm volatile("");
    }
    if (luaL_loadbuffer(L, ABI_CHUNK, __builtin_strlen(ABI_CHUNK), "@abi_chunk.lua") != 0) {
        ramsyscall_printf("abi: chunk did not compile: %s\n", lua_tostring(L, -1));
        abi_probe[ABI_PROBE_LOAD_OK] = 0;
    } else {
        int32_t cursor = 0;
        if (abi_dump_top(L, writer, &cursor, ABI_DUMP_STRIP) != 0) {
            ramsyscall_printf("abi: dump failed\n");
            abi_probe[ABI_PROBE_LOAD_OK] = 0;
        }
        abi_bytecode_size = cursor;
        abi_probe[ABI_PROBE_BLOB_SIZE] = cursor;
    }
    abi_probe[ABI_PROBE_DONE] = ABI_DONE;
    abi_done();
    while (true) asm volatile("");
    return 0;
}
