# P7 — Editor 3D / 2D / UI (2026-09-13)

Implements design.md section 8 over the P4 document: the Scene window's boolean 2D switch
becomes a three-way authoring mode, the Hierarchy filters by domain and shows document
actors, the map root opens a Map Settings window, the creation menu can place actors, and
the editor gains a general scene undo.

New module `src/scene_view_mode.rs` (`SceneViewMode` and the pure `History<T>` snapshot
stack). Everything else lands in `editor.rs`, `gui.rs`, `mcp_tools.rs`, `settings_ui.rs` and
`workspace.rs`.

## What changed in the UI

### Scene window

The **2D** checkbox is replaced by three radio buttons: **3D**, **2D**, **UI**.

| Mode | Viewport | Hierarchy |
| --- | --- | --- |
| `ThreeD` (default) | the shaded 3D preview, unchanged | World3D entities and actors |
| `TwoD` | a placeholder panel: "2D world view - Camera2D authoring arrives with P8 editor work" | World2D actors only |
| `UI` | the Canvas / HUD editor, unchanged | UI entities and actors |

Switching modes never touches the document. The 3D simulation, camera and selection are
untouched by a visit to 2D or UI; only leaving `UI` stops the HUD preview, which is the
behaviour the old checkbox already had.

### Hierarchy

- A virtual **map root** line named after the scene. It is not an instance: it is not
  selectable, not renamable and not a reparent drop target. Clicking it opens Map Settings;
  its context menu keeps the creation commands and adds "Map Settings...". Unparenting by
  drag still works on the empty area below the tree, which was already a drop target.
- Domain filtering (rules below).
- `scene.actors` are drawn under the entities with their own icon (codicon `symbol-class`),
  labelled from `ActorInstance.name` and nested by `logical_parent`. Selection uses the new
  `Editor.selected_actor: Option<Uuid>`; selecting an actor clears `selected` (and the asset
  selection) and selecting an entity clears `selected_actor`.
- An **Actor** submenu in every creation menu, listing `Model::placeable()` grouped as
  3D / 2D / UI / Logic. It is empty with an explanation when the project's reflection data
  does not resolve into a `Model`.

### Inspector

A selected actor gets an Actor section instead of the entity Inspector: class `cpp_name` and
domain (or the raw name in amber with a "class unresolved" note), identity, logical parent,
an editable name, an editable **Active** checkbox routed through `Editor::changed()`, the
component list with class names, a green `root` marker and a `(class default)` marker for
inherited components, and a read-only property table marking overrides. **Add Component** is
present but disabled, with the tooltip "arrives with P9".

### Map Settings (new window)

Scene name, document version, entity and actor counts; the Scene Blueprint section; and the
**Current Scene HUD Budget** controls, moved here from Project Settings. Project Settings
keeps the section heading with a one-line pointer and an "Open Map Settings" button, so no
author loses the control they knew.

The Scene Blueprint section shows the existing parent and Blueprint name when one exists.
Otherwise it offers a combo over `Model::scene_script_parents()` — seeded from the project
default — and a **Create Scene Blueprint** button that builds
`SceneScript { parent, blueprint: BlueprintAsset::new("{scene}_SceneScript", parent_id) }`
through `Editor::changed()`, validating with `Scene::validate_with_model` and rolling back
into `last_error` on failure. **Open Scene Blueprint** is disabled with the tooltip
"Editing embedded scene Blueprints arrives with P6".

### Project Settings

`workspace::Manifest` gains `#[serde(default)] default_scene_script_parent: Option<String>`,
edited under Project / Description > Scripting. It is a suggestion for Map Settings only:
changing it never modifies an existing scene.

## MCP compatibility mapping

`editor_view` accepts both spellings; `editor_state` and every `editor_view` reply report
both. `view_mode` wins when both are sent, so a client that always sends `scene_2d` and a
client that always sends `view_mode` can share a session.

| Request | Resulting mode | `scene_2d` reported | `view_mode` reported |
| --- | --- | --- | --- |
| (nothing) | unchanged, `3d` at startup | `false` | `"3d"` |
| `{"scene_2d": true}` | `UI` | `true` | `"ui"` |
| `{"scene_2d": false}` | `ThreeD` | `false` | `"3d"` |
| `{"view_mode": "2d"}` | `TwoD` | `false` | `"2d"` |
| `{"view_mode": "ui"}` | `UI` | `true` | `"ui"` |
| `{"scene_2d": true, "view_mode": "3d"}` | `ThreeD` | `false` | `"3d"` |
| `{"view_mode": "hud"}` | refused, mode unchanged | — | — |
| `{"view_mode": true}` / `{"scene_2d": "ui"}` | refused (type error) | — | — |

`scene_2d` is therefore lossy on the way out only for `TwoD`, which reports `false`: a
2D world is not the Canvas editor, and that is exactly what the legacy boolean meant.
`Editor::scene_2d()` / `set_scene_2d()` keep the same meaning for the render, screenshot and
HUD-preview paths (`platform.rs` 551/588/643, `main.rs` `--screenshot-hud` and
`--screenshot-native-hud`, `lighting_editor.rs`, the canvas/rect creation sites in `gui.rs`).

## Hierarchy filtering rules

| Item | Domain |
| --- | --- |
| Legacy entity with `canvas` or `rect` | `UI` |
| Legacy entity with 3D geometry, camera, sprite, light, collider, particle or shadow data | `World3D` |
| Legacy `Empty` with only shared data such as scripts, audio, timelines or resource material | `None` |
| `scene.actors` entry | the domain of its class, resolved through `ClassReference::resolve` against `Registry::model()` |
| `scene.actors` entry whose class does not resolve | `World3D`, with a "class unresolved" tooltip on the node and an amber class name in the Inspector |

- An item is drawn normally when its domain equals the mode's domain
  (`ThreeD → World3D`, `TwoD → World2D`, `UI → UI`).
- A `None` item is shared logic or a resource holder and is drawn normally in every mode.
- An item of another domain that has a descendant in the current one is drawn as a
  **presentation root**: the name in `text_disabled` grey, no selection, no rename, no drag
  source and no drop target, with a tooltip explaining why it is there. Its children are
  still walked.
- Everything else is skipped.
- Filtering is pure display. No index is renumbered, no `logical_parent` is rewritten, and
  nothing is mutated — `index` keeps its meaning for every existing caller, including MCP.
- Selection survives a mode switch. `e.selected` / `e.selected_actor` are left alone when
  the item is not in the visible domain; the editor does not force-reveal it and does not
  clear it. Returning to its mode shows it selected again.
- The search filter composes with the domain filter: an item must match both.

## Undo scope

`Editor` keeps a bounded `History<Scene>` of 32 steps. `Editor::changed()` records one step,
`undo_scene()` / `redo_scene()` move through it, and `accept_scene` resets it so a newly
opened map never replays the previous one's edits.

| In scope | Out of scope |
| --- | --- |
| Any edit that reaches `Editor::changed()` on the open map | Project Settings and Editor Preferences |
| Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z while Hierarchy, Scene or Inspector (or a child window) has focus | The Blueprint, mesh and timeline editors, which keep their own stacks and their own shortcuts |
| Edit > Undo/Redo Scene Edit menu items | Anything while `playing` — both are refused with a message |

The existing `script_undo` / `script_redo` attachment stack keeps priority:
`undo_scene()` tries `undo_attachment()` first and only falls back to the snapshot stack
when that stack is empty or its intervening-edit guard refuses. Restoring a snapshot goes
through `restore_scene`, which resets the Blueprint instance baseline, drops in-flight rename
and drag state, clamps `selected` and `selected_actor` to what still exists, and does not
record a new history step.

`History<T>` is a pure generic type with its own unit tests, so the bound, the redo-branch
drop and `reset` are covered without an `Editor` or a GPU.

## Deferred

| Work | Phase |
| --- | --- |
| Component add / remove UI and actor property editing (the disabled "Add Component" button) | P9 |
| Editing an embedded scene Blueprint (the disabled "Open Scene Blueprint" button) | P6 |
| A real 2D viewport, Camera2D authoring and 2D gizmos | P8 |
| Actor drag-and-drop reparenting, actor rename from the Hierarchy, actor duplicate/delete | P9 |
| The conversion action for hybrid 3D+UI entities that `actor_view` diagnoses | P9 |
| Writing derived actors into the document (a real migration rather than a view) | later, behind an explicit author action |

Creating an actor from a *child* context menu still creates a root actor; the menu says so.
Actor parenting has no UI yet.

## Validation run on this machine

| Step | Result |
| --- | --- |
| `cargo build --locked --bins` | succeeds |
| `cargo test --locked scene_view_mode` | 3 passed |
| `cargo test --locked mcp` | 8 passed, 0 failed |
| `cargo test --locked gui` | 1 passed, 1 failed — the pre-existing shared-ImGui `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events` |
| `cargo test --locked editor` | 68 passed, 2 failed — both on the P0 baseline list |
| `cargo test --locked` | **426 passed, 7 failed, 33 ignored** — the 7 are exactly the P0 baseline list, no new failures (P4 was 421 passed) |
| `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, exactly the pre-existing baseline count; none added |
| `cargo fmt` | run, then every file this phase did not intentionally change was reverted (the repo is not fmt-clean and a plain run reformats 32 unrelated files) |
| `cargo run -- --project examples/sample-game --screenshot` | runs; the 3D / 2D / UI selector renders above the Scene viewport with 3D active, and the Hierarchy shows the map root line |

MIPS builds, standalone exports and emulator runs were not executed (no SDK on this machine)
and nothing about them is claimed. The sample project has no resolvable reflection model
here (libclang is missing), so the Actor submenu and the scene Blueprint combo were exercised
by unit tests and by their empty-state paths rather than on screen.

## Tests added

`src/scene_view_mode.rs` (3):

- `view_modes_round_trip_their_wire_values_and_the_legacy_boolean` — every mode's `"3d"` /
  `"2d"` / `"ui"` key, its serde form, the default, an unknown key, and both directions of
  the `scene_2d` compatibility mapping.
- `history_is_bounded_and_redo_survives_until_the_next_edit` — the 32-step bound (tested at
  3), undo/redo walking, and a new edit dropping the redo branch.
- `reset_forgets_the_history_of_the_previous_document`.

`src/mcp_tests.rs` (1): `editor_view_translates_the_legacy_scene_2d_flag_and_the_three_way_view_mode`
— the whole table above, including the two refusals, the type errors and the HUD preview
stopping when the mode leaves `UI`.

`src/editor.rs` (1): `scene_undo_restores_the_previous_document_and_stops_at_the_oldest_step`
— undo to the original document and no further, redo back, refusal while playing, and that
changing the view mode never edits the scene.

## Files

`src/scene_view_mode.rs` (new), `src/editor.rs`, `src/gui.rs`, `src/mcp_tools.rs`,
`src/mcp_tests.rs`, `src/platform.rs`, `src/main.rs`, `src/lighting_editor.rs`,
`src/settings_ui.rs`, `src/workspace.rs`, `src/staging_files.rs` (one field in a test
manifest literal), `docs/editor.md`, `docs/hud.md`, `docs/mcp.md`, `docs/settings.md`.

## Open questions for later phases

- `Editor::changed()` records one history step per call, so a drag that reports every frame
  fills the stack with intermediate values. P9 should coalesce by gesture if authors find the
  32 steps shallow in practice.
- `TwoD` reports `scene_2d: false` over MCP. That is the honest legacy answer, but a client
  that polls `scene_2d` to decide whether to take a HUD screenshot will see `false` in a mode
  that has no 3D viewport either. `viewer_screenshot` targets are unchanged in this phase.
- Actors have no reparent, rename, duplicate or delete UI yet, so an actor created by mistake
  can currently only be removed through MCP or by editing the document.
