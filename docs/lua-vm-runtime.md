# The Lua VM runtime

Two of the three Lua execution modes run scripts through an interpreter on the
console. This file describes what those builds actually contain, what has been
measured, and what has not.

The product contract and the mode selection live in
`knowledge/initiatives/lua-scripting/contract.md`; this is the runtime side of
§7, §8 and §10.4.

## What is linked, per mode

| Mode | `EPOK_LUA_MODE` | Interpreter archive | Chunk payload |
| --- | --- | --- | --- |
| Native C++ | 0 | none | none — bodies are compiled MIPS |
| Lua VM — bytecode | 1 | `lua/liblua-epok-noparser.a` | cooked bytecode |
| Lua VM — source | 2 | `lua/liblua-epok-parser.a` | normalized Lua text |

`src/settings.rs` writes those lines into the generated `sources.mk`; the
runtime `Makefile` includes `lua.mk` only when `EPOK_LUA_VM` is set. The archive
is built into the *build* directory, from
`$(NUGGET_DIR)/third_party/psxlua/src`, with per-variant object directories —
upstream's own Makefile writes both archives from one set of objects, so the two
variants could otherwise contaminate each other across a mode change.

The archive carries psxlua's core plus `lauxlib`. The standard libraries
(`lbaselib`, `lbitlib`, `lcorolib`, `ldblib`, `lstrlib`, `ltablib`, `linit`) are
deliberately absent, and so is `libpsyqo-lua.a`: its constructor loads a
*source* bootstrap chunk and opens every library, which a cooked build neither
needs nor wants. The only globals that exist in a shipped VM are the registered
`__epok_*` helpers — there is no `load`, `require`, `dofile` or `print`.

`runtime/lua_runtime.hpp` therefore supplies the three libc hooks the fork
expects on this target itself (`luaI_sprintf`, `luaI_realloc`, `luaI_free`).
There are no `--defsym` bindings in the link.

## Numeric parity

Every arithmetic operation in a normalized chunk is a call to a registered C
function that forwards to the same `epok::bp::*` function the native mode
compiles to. The fork's own `long` arithmetic never sees a user value. That is
what makes the three modes interchangeable rather than approximately equal:
saturation, division by zero, truncation toward zero and `ineg(INT32_MIN)` are
one implementation, not three.

Unsigned ordering is explicit (`__epok_ult` / `__epok_ule`) because a `UInt32`
travels as its bit pattern inside a signed 32-bit `lua_Number`. Signed ordering
uses Lua's own `<`, which already matches C++ for `Int32`, `Fixed` and `Enum`.

## Arena budget

The runtime has no heap, so the VM's whole world — state, chunks, strings, call
frames — lives in one static buffer. **The size is per mode, chosen by the
editor, not by the project:**

| Mode | `EPOK_LUA_ARENA_BYTES` | Why |
| --- | --- | --- |
| Native C++ | *(none — no arena is linked)* | There is no VM. |
| Lua VM — bytecode | **98 304** (96 KiB) | `luaU_undump` allocates the chunk's structures and little else. |
| Lua VM — source | **196 608** (192 KiB) | Parsing on target allocates transiently during initialization, far above steady state. |

`src/settings.rs` (`LuaExecution::arena_bytes`) picks the value and writes it
into the generated `lua-config.hh` in the build directory, wrapped in
`#ifndef EPOK_LUA_ARENA_BYTES`, which `runtime/lua_runtime.hpp` includes first.
The header's own `#ifndef` fallback (96 KiB) is reached only when no generated
config exists, i.e. in the standalone harnesses.

Because the generated define is guarded, a project can raise the budget for one
build without touching the editor by appending
`CPPFLAGS += -DEPOK_LUA_ARENA_BYTES=262144` to the export's or build
directory's `sources.mk` (the editor regenerates `sources.mk`, so in an editor
build the durable route is `LuaExecution::arena_bytes`). Exhaustion aborts with
`Lua arena exhausted: ... Raise EPOK_LUA_ARENA_BYTES.`; it never degrades
silently.

Sizing this is not guesswork — measured peak live bytes on the two-class
conformance fixture (`tests/integration/lua_conformance/`, `Guard.lua` plus
`Patrol.lua`, read out of guest RAM by `verify_lua_modes.py`):

| Mode | Peak live | Retained after init | Allocations | Headroom against the budget |
| --- | --- | --- | --- | --- |
| Lua VM — bytecode | **14 264 B** | 14 032 B | 195 | 6.9× |
| Lua VM — source | **96 592 B** | 17 560 B | 342 | 2.0× |

The source figure is the whole reason the source mode reserves 192 KiB: the
parser's transient peak is roughly **6.8×** its own retained footprint, and it
alone exceeds the bytecode mode's entire 96 KiB budget. After initialization the
two modes differ by only about 3.5 KiB of retained memory — the 192 KiB buys
initialization, not steady state.

These are one fixture's numbers. Scale with the number and size of chunks, not
with instance count; a project with many classes must re-measure rather than
multiply.

The allocator is a first-fit free list with forward coalescing and in-place
`realloc` growth. `epok::lua::stats` tracks live bytes, peak bytes and allocation counts.
Exhaustion calls `psyqo::Kernel::abort` with the request size and the live
total. It is never silent and never degrades: a build that cannot fit its
scripts says so on the first frame that needs the memory.

Related fixed budgets, all overridable or asserted at startup:
`EPOK_LUA_MAX_CLASSES` (64 binding entries), 64 method slots per class (the
`bound_mask` is a `uint64_t`), 8 arguments per call, and a call depth of 16 —
Lua may call back into C++ which re-enters Lua, and that nesting is bounded.

## Bytecode ABI, and how it is verified

The bytecode mode cooks chunks on the host. `build.rs` compiles the pinned
`psxlua` parser for this machine and links it against
`native/lua/epok_ldump32.c`, a dumper that writes the two quantities whose width
differs between host and console at the console's width: `size_t` lengths and
psxlua's `lua_Number`, which is `long` — 64-bit here, 32-bit on the
PlayStation. A numeric constant outside `int32` is an error, never a truncation.
Without the submodule, `cargo build` still succeeds and the bytecode mode
reports the missing sources as its real cause; it never falls back to shipping
source.

`tests/integration/verify_lua_vm_abi.py` is the proof, not the assumption:

1. It cooks one normalized chunk with the shipping host cooker.
2. It builds a PSX executable that compiles the **same** chunk on target and
   dumps it with the fork's own `luaU_dump`, runs it under PCSX-Redux, and reads
   the bytes back out of guest RAM.
3. It compares the two byte for byte, and checks the 18-byte header the target
   build reports through `luaU_header()` against `lua_bytecode::HEADER`.

Measured on macOS arm64 with `mipsel-none-elf-gcc` 16.2.0, Nugget
`6186b131`, psxlua `abed030e`: **1805 host bytes, 1805 target bytes, identical**;
header `1b4c7561520001040404040119930d0a1a0a` on both sides.

## Debug information

`lua_bytecode::KEEP_DEBUG_INFO` is **true**: chunks are cooked unstripped, so a
Lua error at runtime names the authored `.lua` file and the line. Stripping is a
choice the contract permits, not a requirement.

The cost, measured on the verifier's chunk (seven small methods, 1154 source
bytes): 1805 bytes unstripped versus 992 stripped — **813 bytes, about 45%**.
That ratio is a property of this chunk, which is nearly all control flow and
short bodies; a chunk with larger bodies carries proportionally less debug
information.

## Conformance smoke

The same verifier links `lua_runtime.hpp` into a minimal PsyQo program twice —
once with the chunk as source against the parser archive, once with the
host-cooked bytecode against the no-parser archive — and requires identical
results from both for: `iadd(INT32_MAX, 1)` saturating, `idiv(x, 0) == 0`,
`fmul` truncating toward zero on a negative product, `ineg(INT32_MIN) ==
INT32_MAX`, unsigned `0x80000000 > 1`, a `Bool` round trip through the generated
field accessors, a nested Lua → `__epok_call` → C++ virtual → Lua dispatch, and
an absent slot whose `Frame` reports `bound() == false` and never enters the VM.

Last run: all probes matched in both modes. Arena peak 9088 bytes (source) and
6296 bytes (bytecode) for that one class, from 238 and 142 allocations.

Linked section sizes **of that smoke program** — not of a game:

| Build | `.text` | `.rodata` | `.data` | `.bss` | resident |
| --- | --- | --- | --- | --- | --- |
| smoke, source (parser archive) | 84 256 | 9 448 | 2 496 | 103 196 | 199 396 |
| smoke, bytecode (no-parser archive) | 56 672 | 8 184 | 2 672 | 103 196 | 170 724 |

`.bss` is dominated by the arena and is identical in both: this smoke program
includes `lua_runtime.hpp` without a generated `lua-config.hh`, so both builds
take the header's own 96 KiB `#ifndef` fallback rather than the per-mode budgets
above. The 27 584-byte
`.text` difference is the parser, lexer and code generator that the bytecode
mode does not link.

## Standalone exports

An exported project in a VM mode needs both the pinned Nugget checkout **and**
its nested `third_party/psxlua` submodule; `lua.mk` builds the archive from the
latter during `make`. The three setup scripts initialize and verify psxlua
alongside Nugget. `make` needs nothing else: no host Lua, no `luac`, and no
network access — the chunk payloads are already inside the generated
`scripts/generated/lua/lua_chunks.cpp`.

## What is not validated

- Nothing here ran on physical PlayStation hardware. PCSX-Redux is used for
  reproducible development comparison only.
- The sizes above are the smoke program's. They are not a game's, not RAM
  consumption during play, and not a per-class cost that can be multiplied out.
- The arena figures cover Lua allocations inside `EPOK_LUA_ARENA_BYTES` only —
  not the native side's stack, static data, or anything the rest of the runtime
  allocates.
- The smoke exercises one class with one instance. Many classes, scene cycling
  and arena exhaustion under load are not covered by it.
- Steady-state cost per call is not measured here. The separate feasibility
  harness (`tests/integration/verify_lua_feasibility.py`) measured guest cycles
  for the technology comparison that chose this design.

## Where the whole-project numbers live

This file covers the runtime in isolation. The cross-mode acceptance run —
per-mode linked sizes and ps-exe sizes for one real two-class project, tick
cycles in all three modes, the arena peaks quoted above and the 21 equivalence
checks behind them — is recorded in
[`knowledge/initiatives/lua-scripting/validation-2026-09-14.md`](../knowledge/initiatives/lua-scripting/validation-2026-09-14.md),
produced by `tests/integration/verify_lua_modes.py`.
