# Blueprint implementation and validation

Date: 2026-09-08. Current expanded-scope record; the foundation report is historical.
Implementation and visual acceptance are complete for the bounded contracts
below. The final native editor screenshot was compared alongside both supplied
references. Publication is tracked separately as a release process.

## Implemented contracts

Original versioned Blueprint assets compile through typed IR into native C++.
Native/visual ancestry, explicit inherited defaults, event overrides/Call Parent,
visual functions, checked cross-instance calls, semantic caching and standalone
exports share the reflected class registry. Authoring UUIDs are not runtime slots.
Scene v1/v2 data migrates in memory to v3 without silently rewriting files.

The editable native canvas has typed pins, execution wires, searchable nodes,
comments/reroutes, selection/drag/navigation, copy/paste, undo/redo, diagnostics,
class defaults and graph signatures. Components/My Blueprint navigation on the
left and Details on the right are implemented against the latest reference.

Continuations, timelines, typed factories, linked entity/component templates,
transactional spawning and the closed construction backend have explicit
capacity and cancellation rules. Debug builds include node/source/instance
mapping and typed snapshots with real interpreter breakpoints, stepping and
continue; normal release builds omit the debugger hook. Editing during Play
uses stop/save/rebuild/restart, not code/layout patching or runtime-state retention.

## Reproducible evidence

- `tests/integration/verify_blueprints.py --keep --emulator`: native to visual to
  visual inheritance, independent defaults/overrides, Call Parent, Delay,
  destruction/bank-reset behavior, layout-only cache stability, relocation and
  editor-independent exported MIPS rebuild.
- `tests/integration/verify_blueprint_features.py --keep --emulator`: real MIPS
  release/instrumented builds; timeline interpolation/Updated/Finished/stop,
  cancellation and fresh-instance state; transactional root/child template and
  internal handles before Start; construction, four-slot class exhaustion/reuse,
  resource-only dependencies, checked casts and calls between visual classes.
  The latest run additionally validates Make Vector 2/3, component extraction,
  transform assignment and Spawn Class through a variable/empty ClassRef.
  Its release executable is 258,048 bytes; instrumented is 266,240 bytes
  (8,192-byte difference). The release has no debugger hook symbol.
- `live_blueprint_breakpoint_snapshot_step_resume_and_cleanup`: actual owned
  PCSX-Redux pause, typed health snapshot, distinct next-node step, continue,
  Pause Next, trace transport and cleanup. Host protocol tests separately cover
  instance filtering; the real one-shot Start fixture does not prove a subsequent
  per-instance breakpoint re-hit.
- `tests/runtime/verify_blueprint_runtime.py`: host assertions with documented
  display/callback adaptations plus compilation against real pinned MIPS/PsyQo/
  EASTL headers; arithmetic, continuation/timeline isolation, cancellation,
  typed pool lifecycle, rollback and reentrant destruction quarantine.
- `tests/runtime/verify_blueprint_bridge.py`: pinned LuaJIT protocol assertions
  with emulator services mocked, explicitly not a replacement for the live test.
- `design-qa.md`: final visual acceptance passed against both supplied references;
  the native capture is `docs/images/blueprint-editor.png`. Native Compile,
  checkbox editing followed by Undo clearing dirty state, Components selection,
  the actual Compute Strength function and Class Defaults passed. This is native
  interaction evidence, not a screenshot-only implementation.

Local machine outputs live under ignored `artifacts/blueprints/`. Test generators
can retain isolated example projects; see `docs/blueprints.md` for commands.

Consolidated Rust checks: 154 passed / seven explicitly ignored, including the
latest width regression. The extended ImGui interaction test passed again
explicitly, including canvas clipping, search, preserving component selection
and Add/Undo interaction. The real-extractor
linked-resource-order regression also passed explicitly: actual staging refreshes
stale inherited resources before resolving assets while retaining a position
override and authoring IDs. The latest inheritance/export, native feature and
live debugger acceptance runs passed. Clippy with warnings denied passed.

## Measured prototype cost

The equivalent workload is 16 Q12 arithmetic iterations per update, at 1/16/64
instances, with 10 warmup and 120 measured updates per instance. Native and
generated code run under the same final `-Os` flag and produce identical RAM
results. Each workload function is 100 bytes with a zero-byte own prologue frame.

| Instances | Native median cycles | BP median cycles | Extra PS-X EXE bytes | Extra linked resident bytes |
| --- | --- | --- | --- | --- |
| 1 | 611 | 611 | 2,048 | 2,184 |
| 16 | 611 | 611 | 4,096 | 4,648 |
| 64 | 611 | 611 | 6,144 | 8,040 |

Source: `artifacts/blueprints/performance-validation.json`, generated by
`tests/integration/verify_blueprint_performance.py --keep --emulator`.
Cycles are guest interpreter cycles between identical dispatch/probe markers,
including possible interrupts, not host time or FPS. Resident bytes are linked
text+data+BSS, not peak allocator, stack, VRAM or SPU usage. Whole-binary overhead
includes BP registration/runtime support, not only the equivalent function.

A prior run with production defaults used native `-Os` versus generated-main
`-O2`: 611 versus 1121 cycles (1.835x) for this workload. It is retained separately
as `performance-production-mixed-flags.json`, not presented as a same-flags
compiler comparison. The engine's production optimization policy was not changed
to improve benchmark results. Neither result establishes performance for an
arbitrary game, graph or handwritten native call.

## Deliberate limits and unverified environments

- This is Epok's bounded AOT Blueprint feature, not external asset/API compatibility
  or a pixel-identical port. Existing Codicons/native widgets replace external assets.
- Native reflection supports public single nonvirtual inheritance and supported
  declarative fields/signatures. Arbitrary records, nested/template classes,
  open-ended collections, native pointer serialization and handwritten classes
  deriving from generated BP headers are not supported.
- Loops: combined 4096 iterations; expressions: depth 128 / expansion 4096;
  continuations: eight graph frames; timeline: 16 strictly increasing keys;
  templates: 32 entities; dynamic slots: 32; per-class objects: four; cooked
  classes: 64; typed-pool ceiling: 64 KiB. Exhaustion is diagnosed or returns an
  invalid handle, not unbounded allocation.
- Debug snapshots expose the first 16 reflected members, with bounded trace and
  command buffers. The debugger is not arbitrary native memory inspection.
- Construction is a closed host operation list, not execution of MIPS/native
  constructors on the editor host. Lua and the post-BP VFX/sequencing initiative
  remain separate; there is no operational third-party provider loader.
- The incoming macOS editor support is preserved, but current reflection pins
  the Windows libclang distribution. macOS Blueprint/reflected-class authoring
  is not supported/provisioned or validated. This is not merely an unmeasured
  performance claim; it requires a platform library/setup extension.
- No physical-console validation, peak-stack measurement, fresh empty-cache SDK
  checkout test, or live Windows file-association installation is claimed.
  Existing path/descriptor limitations remain documented in foundation-status.
