# P0 — Isolation and baseline (2026-09-13)

## Reference

| Item | Value |
| --- | --- |
| Base commit | `b5c56161b8c58da0bbd637567bfc39e077b4f63f` (develop) |
| Working branch | `feature/actor-architecture`, created from that commit |
| Working tree at start | clean except the untracked plan file `knowledge/initiatives/bp-scene-optimization.mdy` |
| Other worktrees | none (`git worktree list` showed only this checkout) |
| Audio task status | `26cdef3 Add portable audio assets and PSX sequenced playback` is already committed on develop; no uncommitted audio changes existed |

## Isolation decision

The plan proposed a second worktree because another agent was editing audio files in the
original checkout. At the start of this implementation the tree was clean and no other
task was active, so the work proceeds on a dedicated branch in this checkout. If a
concurrent task appears, move to a worktree before touching shared files
(`runtime/epok.hpp`, `runtime/lifecycle.hpp`, scene generation, reflection, Inspector).

## Host tooling available on this machine

| Tool | Status |
| --- | --- |
| Rust 1.97.1 (pinned), Cargo | available |
| Host C++ (clang++/g++, C++20) | available |
| Nugget submodule | initialized during P0 (`third_party/nugget` @ 6186b131) |
| MIPS toolchain (`mipsel-none-elf-g++`) | not installed |
| PCSX-Redux | not installed |
| libclang / `epok-header-tool` extraction | binary builds; extraction needs the MIPS compiler include paths, so Clang extraction cannot run here |

Consequence: validation on this machine covers Rust unit tests, Clippy, the editor build and
host C++ contract tests (`python3 tests/runtime/verify_spatial.py` compiles the runtime
headers with clang++). MIPS builds, standalone exports and emulator runs are not executed
here and are never claimed as verified in the phase reports.

## Schema and document versions at the start

| Contract | Version | Location |
| --- | --- | --- |
| Reflection wire schema | 7 | `src/reflection_schema.rs` `SCHEMA_VERSION` |
| Scene document (`.epokmap`) | accepted 1–4, saved as 3 or 4 | `src/scene.rs` |
| Blueprint asset (`.epokbp`) | 3 | `src/blueprint_asset.rs` `VERSION` |
| Blueprint runtime class catalog | ≤ 64 classes, `TypedPool<T,4>` per concrete class | `src/blueprint_spawn.rs` |
| Runtime object slots | `scene entities + 32`, editor limit 512 objects per scene | `src/scene_bank.rs`, `src/scene.rs` |

Reserved for this initiative (see the design contract): reflection schema 8, scene
document version 5, Blueprint asset version 4.

## Baseline test run

`cargo build --locked --bins` succeeds. `cargo test --locked` before any change:

| Result | Count |
| --- | --- |
| passed | 387 |
| failed | 7 |
| ignored | 33 |

Pre-existing failures (not regressions of this initiative):

- `audio_contract_tests::audio_authoring_valid_target_error_and_tool_identity`
- `audio_contract_tests::audio_legacy_golden_outputs`
- `console::autoscroll_preserves_read_only_selection_and_copy` (ImGui context)
- `editor::tests::attachment_undo_restores_overrides_and_rejects_intervening_edits`
- `editor::tests::invalid_source_marks_outputs_stale_and_keeps_running_snapshot`
- `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`
- `file_watch::tests::native_events_detect_create_rename_delete_and_preserved_timestamp`

These depend on the pinned SDK tools, a shared ImGui context or platform file watching.
Each later phase compares against this list.

## Inventory of legacy contracts that the new model must preserve

- `epok::Object` (renamed `LegacyEntityStorage`, alias `Entity`) is the single runtime slot
  record with 3D, HUD, sprite, audio and lighting components inline; `EntityHandle` is
  index + generation over `objects[]`.
- `Behaviour` binds to one entity; events `start/update/frame_update/on_enable/on_disable/
  on_destroy/on_trigger`; Blueprint continuations hook `blueprint_tick/cancel/observe`.
- `Binding { behaviour, entity, class_id }` tables per scene bank; `bp::reserve/spawn/
  start_batch/retire` implement batch spawn with quarantine and generation checks.
- Scene documents: `Entity { id, kind: Mesh|Camera|Empty, parent index, components as
  Option<...>, script: ScriptBinding, blueprint_instance }`.
- Reflection: `Class { id, provider, backend, cpp_name, parent, abstract, final,
  blueprintable, timeline_component, properties, functions }`.
- Editor: `Editor.scene_2d: bool` switches the Scene window to the Canvas/HUD editor; MCP
  exposes `scene_2d` in `editor_state` and `set_view`.

## PSX budget baseline

No emulator is available here, so the numeric baseline (EXE size, RAM, pool usage, frame
scanlines) could not be captured in P0. The plan's rule stands: no overhead percentage is
asserted without a measurement. The first machine with the SDK must run
`tools/profile_runtime.py` on `examples/rpg-2-5d-demo` before and after the runtime
changes and attach the comparison to the P10 report.

## Changes made during P0

- `runtime/epok.hpp`: the slot record `struct Object` is now `struct LegacyEntityStorage`;
  `using Entity=LegacyEntityStorage` is unchanged. Generated code (`src/scene_bank.rs`,
  `src/project.rs`, `src/blueprint_spawn.rs`) and the runtime headers/tests that named the
  slot type now use `Entity`. This frees `epok::Object` for the new class root.
- `tests/runtime/verify_spatial.py`: host binaries are named `<test>-test` so the `utility`
  executable no longer shadows `<utility>` on the include path (macOS/Linux failure).
- `tests/runtime/test_blueprint_playback.cpp`: the global `wait` became `playback_wait`; it
  collided with POSIX `wait()` on macOS.
- Host harness results after these changes: `verify_spatial.py`, `verify_sprites_particles.py`
  and `verify_blueprint_runtime.py` pass (MIPS syntax step skipped, no SDK).

## Pre-existing failures fixed after P11

Five tests failed in the default `cargo test --locked` run on a macOS host without the
MIPS/Clang SDK. All five were real defects, not environment noise; the run is now
474 passed / 2 failed (only the two `audio_contract_tests` that need `psxavenc`).

- `console::autoscroll_preserves_read_only_selection_and_copy` — the ImGui context is
  process-global, and four non-`#[ignore]`d tests created one concurrently, so
  `igCreateContext`'s "another one already exists" assert fired at random. Every test now
  takes its context from `crate::gui::tests::imgui_context()`, a guard that holds a global
  `Mutex` and destroys the context before releasing it (poison-tolerant, so a panicking
  test cannot leak it).
- The same test's selection assertion (`Mouse selection crosses lines: ""`) came from Dear
  ImGui defaulting `ConfigMacOSXBehaviors` to true under `__APPLE__`: copy/select-all/line
  navigation move from Ctrl to Super, so the simulated Ctrl+C copied nothing. The shared
  guard pins `config_mac_os_behaviors = false` so simulated input matches CI hosts.
- `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events` —
  three separate defects, all previously masked by the panic above:
  - The Console/Project assertions still assumed the two panels shared a tab bar. Commit
    737f496 moved Console into its own dock node, so `DockTabIsVisible` is true for both.
    The assertions now compare the focused panel (`NavWindow`) and additionally require it
    to be the selected tab, which is strictly stronger.
  - `settings_ui::windows` reserved 270px for the settings footer, but the Dependencies
    page adds an "Install / Repair N missing" button to the same row. Apply was laid out
    past the window's right edge and could not be clicked at all — a real UI bug. The
    footer now reserves the wider row on that page.
  - `project_browser::verify_interactions` builds a blueprint deriving from
    `epok::Behaviour`, which only the pinned Clang/MIPS extractor can supply. That block is
    now conditional on the reflected runtime bases being present and prints a skip reason;
    hosts with the SDK still run it, and the rest of the browser coverage runs everywhere.
- `editor::tests::attachment_undo_restores_overrides_and_rejects_intervening_edits` — a real
  product bug in `Editor::apply_script_catalog`. Its recovery branch re-read the native
  catalog but discarded it unless `blueprint::native_registry` also succeeded, and that call
  runs the very reflection pass that had just failed. A project with legacy `.epokscript`
  metadata therefore lost its whole catalog and nothing could be attached. The branch now
  falls back to `blueprint::legacy_registry`.
- `editor::tests::invalid_source_marks_outputs_stale_and_keeps_running_snapshot` — a missing
  reflection registry is a genuine registry error on this host, and it marks every timeline
  cache stale. The two assertions that assumed an SDK now compare against the registry state
  itself: the broken timeline must not *change* the registry error, and the unrelated
  "Untouched" timeline must keep its compiled payload with `stale` iff a registry error
  exists. Both are equivalent to the originals wherever the SDK is present.
- `file_watch::tests::native_events_detect_create_rename_delete_and_preserved_timestamp` — not
  a timing problem. macOS FSEvents reports paths under `/private`, and both `/tmp` and the
  per-user temp folder are symlinks, so the watch root (`/var/folders/...`) never matched the
  delivered paths and `Scope::accepts` rejected every event. `normalized` now resolves
  symlinked ancestors first, keeping the leaf name so renamed and deleted files still match.
  Verified over 10 consecutive runs.
