/* Target-ABI bytecode writer for the pinned psxlua fork.
 *
 * Structurally a copy of psxlua's `ldump.c` (same field order, same traversal),
 * with the two quantities whose width differs between this host and the PSX
 * target written at the target's width instead of the host's:
 *   - `size_t` lengths: 4 bytes here, `sizeof(size_t)` in the upstream dumper.
 *   - `lua_Number` (`long` in this fork): 4 bytes here, range-checked to int32.
 * `int` and `Instruction` are 4 bytes on both, so they are copied verbatim.
 *
 * The 18-byte header is the fixed target header rather than `luaU_header()`,
 * which would report the host's widths. `tests/integration/verify_lua_vm_abi.py`
 * byte-compares this writer's output against `luaU_dump()` running on target.
 */
#include <stddef.h>
#include <string.h>

#define LUA_CORE

#include "lua.h"

#include "lobject.h"
#include "lstate.h"
#include "lundump.h"

#include "epok_luac.h"

/* Fixed target header: "\x1bLua", version 5.2, format 0, little endian,
 * sizeof(int)=sizeof(size_t)=sizeof(Instruction)=sizeof(lua_Number)=4,
 * integral numbers, LUAC_TAIL. Mirrored by `lua_bytecode::HEADER`. */
static const unsigned char EPOK_LUAC_HEADER[EPOK_LUAC_HEADER_SIZE] = {
    0x1B, 0x4C, 0x75, 0x61, 0x52, 0x00, 0x01, 0x04, 0x04,
    0x04, 0x04, 0x01, 0x19, 0x93, 0x0D, 0x0A, 0x1A, 0x0A};

typedef struct {
    epok_luac_writer writer;
    void *data;
    int strip;
    int status;
    char *err;
    size_t errcap;
} Dump32;

static void fail(Dump32 *D, const char *message) {
    if (D->status == 0) {
        D->status = 1;
        if (D->err && D->errcap) {
            size_t n = strlen(message);
            if (n >= D->errcap) n = D->errcap - 1;
            memcpy(D->err, message, n);
            D->err[n] = '\0';
        }
    }
}

static void DumpBlock(const void *b, size_t size, Dump32 *D) {
    if (D->status == 0 && size > 0 && (*D->writer)(D->data, b, size) != 0)
        fail(D, "bytecode writer rejected a block");
}

static void DumpU32(unsigned long value, Dump32 *D) {
    unsigned char raw[4];
    raw[0] = (unsigned char)(value & 0xff);
    raw[1] = (unsigned char)((value >> 8) & 0xff);
    raw[2] = (unsigned char)((value >> 16) & 0xff);
    raw[3] = (unsigned char)((value >> 24) & 0xff);
    DumpBlock(raw, 4, D);
}

static void DumpChar(int y, Dump32 *D) {
    char x = (char)y;
    DumpBlock(&x, 1, D);
}

/* `int` is 4 bytes on this host and on the target; the little-endian split
 * keeps the output independent of the host's byte order all the same. */
static void DumpInt(int x, Dump32 *D) { DumpU32((unsigned long)(unsigned int)x, D); }

/* psxlua's `lua_Number` is `long`: 64-bit on LP64 hosts and 32-bit on
 * Windows and the target. When the host is wider, reject values that cannot be
 * represented by the target rather than silently truncating them. */
static void DumpNumber(lua_Number x, Dump32 *D) {
    if (x > 2147483647L || x < (-2147483647L - 1L))
        fail(D, "numeric constant does not fit the target's 32-bit lua_Number");
    DumpU32((unsigned long)(unsigned int)(int)x, D);
}

static void DumpSize(size_t x, Dump32 *D) {
    if (x > 0xffffffffUL) fail(D, "object exceeds the target's 32-bit size_t");
    DumpU32((unsigned long)x, D);
}

static void DumpCodeVector(const Instruction *code, int n, Dump32 *D) {
    int i;
    DumpInt(n, D);
    for (i = 0; i < n; ++i) DumpU32((unsigned long)(unsigned int)code[i], D);
}

static void DumpLineVector(const int *lines, int n, Dump32 *D) {
    int i;
    DumpInt(n, D);
    for (i = 0; i < n; ++i) DumpInt(lines[i], D);
}

static void DumpString(const TString *s, Dump32 *D) {
    if (s == NULL) {
        DumpSize(0, D);
    } else {
        size_t size = s->tsv.len + 1; /* include the trailing '\0' */
        DumpSize(size, D);
        DumpBlock(getstr(s), size, D);
    }
}

static void DumpFunction(const Proto *f, Dump32 *D);

static void DumpConstants(const Proto *f, Dump32 *D) {
    int i, n = f->sizek;
    DumpInt(n, D);
    for (i = 0; i < n; i++) {
        const TValue *o = &f->k[i];
        DumpChar(ttypenv(o), D);
        switch (ttypenv(o)) {
            case LUA_TNIL: break;
            case LUA_TBOOLEAN: DumpChar(bvalue(o), D); break;
            case LUA_TNUMBER: DumpNumber(nvalue(o), D); break;
            case LUA_TSTRING: DumpString(rawtsvalue(o), D); break;
            default: fail(D, "unsupported constant type in the compiled chunk"); break;
        }
    }
    n = f->sizep;
    DumpInt(n, D);
    for (i = 0; i < n; i++) DumpFunction(f->p[i], D);
}

static void DumpUpvalues(const Proto *f, Dump32 *D) {
    int i, n = f->sizeupvalues;
    DumpInt(n, D);
    for (i = 0; i < n; i++) {
        DumpChar(f->upvalues[i].instack, D);
        DumpChar(f->upvalues[i].idx, D);
    }
}

static void DumpDebug(const Proto *f, Dump32 *D) {
    int i, n;
    DumpString(D->strip ? NULL : f->source, D);
    n = D->strip ? 0 : f->sizelineinfo;
    DumpLineVector(f->lineinfo, n, D);
    n = D->strip ? 0 : f->sizelocvars;
    DumpInt(n, D);
    for (i = 0; i < n; i++) {
        DumpString(f->locvars[i].varname, D);
        DumpInt(f->locvars[i].startpc, D);
        DumpInt(f->locvars[i].endpc, D);
    }
    n = D->strip ? 0 : f->sizeupvalues;
    DumpInt(n, D);
    for (i = 0; i < n; i++) DumpString(f->upvalues[i].name, D);
}

static void DumpFunction(const Proto *f, Dump32 *D) {
    DumpInt(f->linedefined, D);
    DumpInt(f->lastlinedefined, D);
    DumpChar(f->numparams, D);
    DumpChar(f->is_vararg, D);
    DumpChar(f->maxstacksize, D);
    DumpCodeVector(f->code, f->sizecode, D);
    DumpConstants(f, D);
    DumpUpvalues(f, D);
    DumpDebug(f, D);
}

int epok_dump32(const void *proto, int strip, void *data, epok_luac_writer writer, char *err,
                size_t errcap) {
    Dump32 D;
    D.writer = writer;
    D.data = data;
    D.strip = strip;
    D.status = 0;
    D.err = err;
    D.errcap = errcap;
    DumpBlock(EPOK_LUAC_HEADER, EPOK_LUAC_HEADER_SIZE, &D);
    DumpFunction((const Proto *)proto, &D);
    return D.status;
}
