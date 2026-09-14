# Lua scripting — acceptance validation log, 2026-09-14

Maintainer record of the acceptance milestone for the Lua scripting initiative.
It holds the numbers, the environment they came from, the bugs the milestone
found, and what was *not* validated. The user-facing guides
([`docs/lua-scripting.md`](../../../docs/lua-scripting.md),
[`docs/lua-vm-runtime.md`](../../../docs/lua-vm-runtime.md)) state behaviour and
quote only the figures a reader needs; the evidence lives here.

Contract: [`contract.md`](contract.md). Baseline: `develop` @ `32fc510`.

Every figure below is labelled:

- **[measured]** — produced by a run recorded in this revision.
- **[estimated]** — derived or extrapolated, not directly observed.
- **[pending]** — not done.

## 1. Environment

| | |
| --- | --- |
| Host | macOS arm64 (Darwin 25.4) |
| Toolchain | `mipsel-none-elf-gcc` 16.2.0 |
| Nugget | pinned `third_party/nugget` @ `6186b131` |
| psxlua | nested `third_party/psxlua` @ `abed030e` |
| Emulator | PCSX-Redux under `.tools/`, headless, with `openbios.bin` |
| Runtime optimization | `EPOK_RUNTIME_OPT=-Os`, pinned by the driver so the three modes compare like for like |

Nothing in this file was validated on physical PlayStation hardware. PCSX-Redux
is used for reproducible development comparison only.

## 2. Baseline

**[measured]** On a clean `develop` @ `32fc510` with the local toolchain,
`cargo test --locked` reports **565 passing** and **2 failing** tests. Both
failures are in the audio subsystem and are **pre-existing on that commit** —
they are unrelated to Lua, reproduce without any Lua change applied, and are not
regressions introduced by this initiative. They are the baseline against which
the initiative's own test additions were judged, not an accepted defect of it.

## 3. Feasibility benchmark (pre-implementation)

Source: [`tests/integration/lua_feasibility/README.md`](../../../tests/integration/lua_feasibility/README.md),
driver `verify_lua_feasibility.py`, report `artifacts/lua_feasibility/report.json`.

This harness links **no Epok engine code**. It compares four variants — `native`
(per-instance handler pointer, no heap), `vm-parser` (`liblua.a`, source chunk),
`vm-noparser` (bytecode chunk), and `vm-cached-dispatch` (bytecode plus an event
bitmask and numeric registry ids, with per-instance chunk reload) — across seven
workloads at 1, 16, 32 and 64 instances, 6 warm-up plus 60 measured steps.

**[measured]** Correctness gate: all four variants produced the **same checksum
and the same handler-call count** for all 28 cases. `passed: true`, no failures.

**[measured]** Median guest cycles per step, 64 instances:

| Workload | `native` | `vm-parser` | `vm-noparser` | `vm-cached-dispatch` |
| --- | --- | --- | --- | --- |
| `arith` | 14 353 | 274 795 | 273 062 | 212 133 |
| `native_call` | 11 025 | 309 355 | 307 622 | 246 693 |
| `position` | 19 857 | 1 194 733 | 1 145 067 | 1 643 657 |
| `alloc` | 23 441 | 1 091 913 | 1 005 366 | 1 499 954 |
| `event_only` | 1 297 | 76 075 | 75 494 | 1 445 |

**[measured]** Linked sections of the harness executables (`size -A`), resident
bytes: `native` 7 964; `vm-noparser` 98 344; `vm-cached-dispatch` 100 392;
`vm-parser` 153 644 — the parser alone is about **29 KiB of `.text`** plus a far
larger `.bss` (26 720 vs. 2 140).

What the benchmark decided, and why the shipped design is what it is:

1. An interpreted body costs roughly **one to two orders of magnitude** more
   guest cycles than the native equivalent on arithmetic and native-call
   workloads. That is why Native C++ is the default mode and the VM modes are an
   explicit opt-in, not a transparent runtime choice.
2. Removing the parser is **nearly free at steady state** (`vm-parser` vs.
   `vm-noparser` differ by well under 1%) but saves ~29 KiB of `.text` and ~24 KiB
   of `.bss`. That is why the bytecode mode exists as a separate mode rather than
   as an optimization of the source mode.
3. The `event_only` row is the interesting one: gating entry into the VM on an
   event bitmask brings a no-handler step back to near-native
   (1 445 vs. 1 297 cycles, against 75 494 for unconditional dispatch). The
   shipped design takes the same idea as the `bound_mask`, which reports
   `bound() == false` for an absent slot without entering the VM.
4. Allocating workloads (`position`, `alloc`) are where the VM is worst, and
   `vm-cached-dispatch` is *worse* than plain `vm-noparser` there because of its
   per-instance chunk reload. The shipped design allocates no table per value
   access: whole-vector values are rejected by the profile and components move as
   scalars.

**Caveat carried forward, and resolved by the shipped design:** in this harness
Lua arithmetic *wraps* at 32 bits while `epok::bp` *saturates*, and the workloads
were hand-bounded so the two agree. That is a property of those workloads, not an
equivalence. The shipped runtime removes the caveat by routing every arithmetic
operation through a registered `__epok_*` helper that calls the same
`epok::bp::*` function the native mode compiles to.

## 4. Bytecode ABI verification

Driver: `verify_lua_vm_abi.py`.

**[measured]** One normalized chunk cooked by the shipping host cooker
(`build.rs` + `native/lua/epok_ldump32.c`) against the same chunk compiled on
target and dumped by the fork's own `luaU_dump`, read back out of guest RAM:

- Host: **1805 bytes**. Target: **1805 bytes**. **Byte-for-byte identical.**
- 18-byte header `1b4c7561520001040404040119930d0a1a0a`, reported by
  `luaU_header()` on the target build and equal to `lua_bytecode::HEADER` on the
  host.

**[measured]** Debug-information cost on that chunk (seven small methods, 1154
source bytes): 1805 bytes unstripped vs. 992 stripped — **813 bytes, ~45%**.
`KEEP_DEBUG_INFO` is **true**, so a runtime Lua error names the authored `.lua`
file and line. **[estimated]** That ratio is a property of a chunk that is nearly
all control flow and short bodies; larger bodies carry proportionally less.

A desktop `luac` is not trusted and is not used: the pinned psxlua `make host32`
does not even link on macOS arm64, which is recorded in the feasibility report
under `bytecode_abi.host_luac`. The 32-bit-ABI dumper exists precisely so the two
quantities that differ between host and console — `size_t` lengths and psxlua's
`lua_Number`, a 64-bit `long` here and 32-bit on target — are written at the
console's width, and a numeric constant outside `int32` is an **error**, never a
silent truncation.

## 5. Cross-mode conformance

Driver: `tests/integration/verify_lua_modes.py`. Report:
`artifacts/lua_modes/report.json`. Fixture: `tests/integration/lua_conformance/`
— a reflected C++ `EnemyBase` plus two Lua classes (`Guard`, `Patrol`), built as
one real project through the production CLI.

**[measured]** Result: **21 of 21 checks passed** (`"passed": true`).

### 5.1 Linked sizes per mode

| Mode | `.text` | `.rodata` | `.data` | `.bss` | resident | `epok.ps-exe` |
| --- | --- | --- | --- | --- | --- | --- |
| `native_cpp` | 141 632 | 25 016 | 25 792 | 500 136 | 692 576 | **194 560** |
| `vm_bytecode` | 194 272 | 38 616 | 27 136 | 600 112 | 860 136 | **262 144** |
| `vm_source` | 221 840 | 36 888 | 25 872 | 600 112 | 884 712 | **286 720** |

Reading these: the bytecode mode adds ~52.6 KiB of `.text` (the interpreter core)
over native; the source mode adds a further ~27.6 KiB (parser, lexer, code
generator) — the same ~27–29 KiB the feasibility harness isolated, reproduced
inside a real project. The two VM modes share an identical `.bss` because the
arena dominates it, and `native_cpp` links **0** `lua_*` symbols while both VM
modes link **54**.

These are this fixture's sizes. They are not a game's, not RAM consumption during
play, and not a per-class cost that can be multiplied out.

### 5.2 Tick cost

**[measured]** Guest cycles between the `epok_lua_probe_begin` and
`epok_lua_probe_end` markers inside `Guard:tick`, sampled in PCSX-Redux, 240
samples per mode:

| Mode | min | median | p95 | max |
| --- | --- | --- | --- | --- |
| `native_cpp` | 438 | **438** | **438** | 438 |
| `vm_bytecode` | 11 074 | **12 171** | **13 269** | 13 269 |
| `vm_source` | 11 101 | **12 198** | **13 296** | 13 296 |

`native_cpp` is exactly constant across all 240 samples, which is what a lowered
native body should look like. The two VM modes are within **0.2%** of each other
at both median and p95 — once a chunk is loaded, source and bytecode execute the
same instructions, and the difference is link size and initialization, not
steady-state speed.

These are **not** host time, **not** frame time and **not** a frame rate. The
window includes the probe writes and any interrupt taken inside it.

### 5.3 Arena peaks

**[measured]** Peak live bytes inside `EPOK_LUA_ARENA_BYTES`, same fixture:

| Mode | Peak live | Retained after init | Allocations | Budget | Headroom |
| --- | --- | --- | --- | --- | --- |
| `vm_bytecode` | **14 264** | 14 032 | 195 | 98 304 (96 KiB) | 6.9× |
| `vm_source` | **96 592** | 17 560 | 342 | 196 608 (192 KiB) | 2.0× |

This is the entire justification for the per-mode budget in
`settings::LuaExecution::arena_bytes`. The source mode's transient parse peak is
~6.8× its own retained footprint and **exceeds the bytecode mode's whole budget**;
after initialization the two differ by only ~3.5 KiB. A single shared budget
would have had to be the larger one, so the bytecode mode would have carried
96 KiB of `.bss` it never touches.

**[estimated]** Scaling is with the number and size of chunks, not with instance
count. A project with many classes must re-measure; these figures must not be
multiplied out.

### 5.4 The 21 checks

| # | Check |
| --- | --- |
| 1 | `native_cpp` links no interpreter at all (0 `lua_*` symbols) |
| 2–3 | `vm_bytecode` / `vm_source` each link the PsyQo Lua core (54 `lua_*` symbols) |
| 4–6 | Each mode's every probe slot equals the value computed from the numeric contract |
| 7 | The probe array is identical across all three modes |
| 8 | The staged scene documents (`scene.epokmap`, `scene.hh`) are byte-identical in all three modes |
| 9 | `Scripts.epokmanifest` lists the same `.lua` dependencies in all three modes |
| 10–12 | Three unsupported constructs (`while`, a table constructor, string concatenation) fail with **one identical diagnostic** in all three modes |
| 13 | Switching to `vm_bytecode` emits `lua_bindings.cpp` + `lua_chunks.cpp` and leaves no lowered `epok::bp::` in a generated header |
| 14 | Switching back to `native_cpp` removes `lua_bindings.cpp` and reproduces the first native build's header bytes **exactly** |
| 15 | The play-cache receipt is not reused across modes; the three `epok.ps-exe` hashes differ |
| 16, 18, 20 | Each mode's standalone export builds with `make BUILD=Release NUGGET_DIR=<nugget>` **with no editor** |
| 17 | The `native_cpp` export contains no `lua_bindings.cpp` and no chunk payload |
| 19, 21 | Each VM export carries `lua.mk`, `lua_runtime.hpp`, the chunk payload and a `Scripts.epokmanifest` listing both `.lua` sources |

Checks 4–6 are the load-bearing ones: the expected probe values are **computed in
Python from the contract**, not captured from a previous run, so a bug that
changed all three modes identically would still fail.

## 6. Example validation

**[measured]** `examples/lua-scripting/` (`EnemyBase.hpp`, `Guard.lua`,
`Patrol.lua`) built through the production CLI in all three modes on this
revision. Per-mode `epok.ps-exe` sizes are recorded in
[`examples/lua-scripting/README.md`](../../../examples/lua-scripting/README.md).
The example was **built, not executed**; on-target equivalence is established by
§5 on its own fixture.

## 7. Bugs found and fixed during acceptance

Acceptance was not a rubber stamp. Five defects were found and fixed:

1. **`and` / `or` did not short-circuit side effects.** The right operand was
   lowered eagerly, so a call on the right ran even when the left operand already
   decided the result — a semantic divergence from Lua that a script could
   observe. Fixed in `lua_frontend`: when the right operand requires a statement,
   it is lowered into its own block and that block is emitted under a guard
   instead of unconditionally. Covered by
   `lua_frontend_guards_short_circuit_operands_that_need_a_statement` and by the
   fixture's `bump()` probe, which counts the calls that must **not** happen.
2. **Inherited reflected callables had no VM dispatch slot.** The VM binding
   published slots only for methods the Lua class itself declared or overrode, so
   a VM build rejected `self:apply_damage(...)` where `apply_damage` came from
   the native parent — a real behavioural difference between modes, and the one
   the mode-independence promise cannot tolerate. Fixed in
   `lua_vm::method_slots` (own methods first, then every function from
   `registry.ancestry`) and `lua_aot::emit_bindings` (the inherited entries join
   the same `self_call` / `super_call` switches). Verified on the shipped example:
   the generated `lua_bindings.cpp` now carries an `apply_damage` slot on both
   `Guard` and `Patrol`.
3. **Spurious `super_call` cases.** `method_index` mapped a function id to a
   dense slot, but a slot's *overridden* parent ids were not folded into the same
   entry, so some parent calls emitted a case that could never be reached and
   others resolved to the wrong slot. `method_index` now inserts each slot under
   its own id **and** under every id in `function.overrides`, so `Expr::CallParent`
   — which carries the parent's id — lands on the same dense slot as `self_call`.
4. **Non-deterministic `ar` output blocked certification.** Archive members
   carried ambient metadata, so two builds of the same sources produced different
   archive bytes. Made deterministic; without it the byte-identity checks (8, 9,
   14) could not be asserted at all, and a reproducible-build claim would have
   been unsupportable.
5. **Whole-vector narrowing was a backend rule, not a profile rule.** Vector
   values were accepted by the frontend and rejected later by the VM backends
   with a VM-specific message, so the same source was valid or invalid depending
   on a project setting — exactly the mode-observability the contract forbids.
   Moved into the frontend as `profile::VECTOR_VALUE`, so all three modes reject
   it identically and the diagnostic names no mode.

Items 1, 2, 3 and 5 were each a mode-observable or semantics-observable
difference. They are the reason the acceptance milestone existed.

## 8. Not validated

- **[pending] Physical hardware.** Nothing here ran on a real PlayStation. Every
  number is from PCSX-Redux, for reproducible development comparison only.
- **[pending] Many classes.** The conformance fixture is two Lua classes and one
  C++ base. Arena behaviour with many chunks, scene cycling, and arena exhaustion
  under load are not covered.
- **[pending] Long-run and steady-state memory.** Peaks are initialization peaks
  over a short run. Fragmentation of the first-fit free list over a long session
  is not characterized.
- **[pending] Lua → Blueprint parents.** Deferred, not merely unimplemented: it
  is a declaration-order question (Lua compiles before Blueprints) and is out of
  scope for profile v1.
- **[pending] Blueprint-with-Lua-parent from the CLI.** Supported in the editor;
  `--new-blueprint --parent` resolves from the reflected C++ registry and
  compiled Blueprints only, so the case is covered by the Rust test
  `lua_compile::lua_classes_are_blueprint_parents_but_blueprint_classes_are_not_lua_parents`
  rather than by the integration driver.
- **[pending] Source-level debugging.** VM builds keep chunk debug information, so
  a runtime error names file and line, but there is no stepping or breakpoints.
- **[pending] Hot reload.** Not attempted; a `.lua` change rebuilds and relinks.
- The **arena** figures cover Lua allocations inside `EPOK_LUA_ARENA_BYTES` only
  — not the native stack, static data, or anything else the runtime allocates.
- The **linked sizes** are text + rodata + data + bss of one fixture project, not
  peak allocator, VRAM or SPU usage.

## 9. Reproducing

```sh
python3 -m pip install -r tools/requirements.txt
python3 tests/integration/verify_lua_modes.py          # 21 checks, ~10+ min
python3 tests/integration/verify_lua_vm_abi.py --output abi.json
python3 tests/integration/verify_lua_feasibility.py --emulator --output feas.json
```

Prerequisites and flags are in
[`knowledge/maintainers/testing.md`](../../maintainers/testing.md#target-checks).
Serialize them with the other emulator checks.
