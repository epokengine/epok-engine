# Blueprint completion work

The user expanded the request on 2026-09-08 from the first foundation to the full
Blueprint feature described in `research.md`. Implementation and visual acceptance
are complete for the bounded contracts below. The foundation report remains a
historical snapshot; current acceptance is recorded in `validation-status.md`.

## Completion gates

- [x] Versioned data-only assets, native/visual inheritance, defaults, member IDs,
  reparenting and recoverable rename/reference migration.
- [x] Typed graph model/IR/compiler, functions/events/Call Parent, explicit
  evaluation order, structured bounded flow, Q12 policies and node diagnostics.
- [x] Usable node canvas and catalog, typed links, movement/navigation, selection,
  comments/reroutes, copy/paste, undo/redo, defaults and function editing.
- [x] Nonblocking bounded continuations, instance isolation, pause/deactivation,
  teardown cancellation, timelines and bounded traces.
- [x] Stable entity identities and checked entity/asset/class references.
- [x] Entity/component class templates, placement, bounded dynamic class factories
  and capacity/handle/lifecycle acceptance.
- [x] Construction logic with an explicit safe host backend; visual debugging with
  real node/instance mappings, pause/step and inspectable state.
- [x] Build/export/cache integration, safe edits during Play via rebuild/restart,
  standalone MIPS rebuild, native emulator behavior and measured prototype costs.
- [x] Native editor interactions, regression checks, examples and current user docs.
- [x] Final native screenshot and full-editor visual QA against both supplied references.

Checked gates refer to the bounded contracts and reproducible evidence in
`validation-status.md`, not arbitrary external compatibility or unsupported native
types. The final native feature fixture includes typed cross-instance calls,
Make Vector/component extraction and dynamic ClassRef spawning. The inheritance
fixture was rerun after integration and passed MIPS/export/relocation/emulator
acceptance. The live debugger has separate real-emulator evidence.

The latest default Rust suite passed 154 tests with seven explicitly ignored.
The extended ImGui test passed clipping, search, component-selection preservation
and Add/Undo interaction again; the real-extractor stale-resource regression
passed with explicit overrides retained. Final native visual QA passed alongside
both original references, with the capture at `docs/images/blueprint-editor.png`.
Compile, checkbox edit/Undo clearing dirty state, Components selection, the real
Compute Strength function and Class Defaults passed in the native editor.
Publication and preservation of incoming upstream changes are handled separately
as release work, not outstanding feature acceptance.

## Execution and ownership

Parallel agents own the asset/IR/compiler, visual-editor module and bounded runtime
header/tests. The primary agent reviews their code and owns integration, scene
identity, provider/build behavior, remaining class features and final acceptance.
Native emulator runs remain serialized.

Use the existing ImGui draw-list API for the first integrated canvas, avoiding a
second native ImGui dependency/context. This is an implementation choice, not a
claim that a third-party node library was validated.

Lua remains a separately reserved provider; the post-BP VFX/sequencing editor is a
separate initiative. Native code/layout patching during Play is not implied: the
documented safe iteration contract is rebuild/restart with explicit state policy.
Physical hardware checks require hardware and must be reported honestly.
