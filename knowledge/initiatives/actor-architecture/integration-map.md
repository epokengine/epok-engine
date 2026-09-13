# Integration map for P4–P7 (surveyed 2026-09-13 at commit f1ec7aa)

Line numbers drift as phases land; use the identifiers.

## A. Scene document (`src/scene.rs`)

- Only `impl Default for Scene` (`scene.rs` ~232) and the test literal near ~763 build `Scene`
  without `..Default::default()`; every other site spreads defaults. `Entity::cube` (~162) is the
  single `Entity` literal.
- Version handling is fully inside `scene.rs`: `validate()` accepts `[1,2,3,4]`; `upgrade_entity_ids`
  (~546–575) decides 3 vs 4 and is the place to raise to 5 when actors/scene_script exist. Entry
  points: `load`, `load_unresolved`, `save`.
- Editor has no general undo stack for scene edits; only `script_undo/script_redo`
  (`editor.rs` ~167, `undo_attachment` ~997). `Editor::changed()` (~791) records Blueprint
  instance overrides via `blueprint_templates::record_overrides`, then sets `dirty`/`view_dirty`.
- UI element rule everywhere: `entity.canvas.is_some() || entity.rect.is_some()`; `hud::validate`
  (~184) enforces canvas root/Empty kind, rect parent chain, graphics need rect, budgets.
- Cook: `project::scene_header_body_with_layout` (~146–577) emits `objects` literals (~255–306),
  HUD component data (~410–480), and `Binding` records for `entity.script` (~512–570) via
  `script_backend::resolve` + `blueprint_refs::compact_id`. Blueprint instances are materialized
  earlier by `project::refresh_linked_scene` → `blueprint_templates::refresh_instances`.

## B. Blueprint assets

- `blueprint_asset.rs`: `VERSION=3`; `BlueprintAsset` derives Clone/Debug/Serialize/Deserialize
  only (no Default/PartialEq); `load` accepts `[1,2,VERSION]`, migrates in memory; `create` is
  create_new; `load_all` walks `assets/` for `*.epokbp`, rejects symlinks, sorts by path.
- `blueprint_compile.rs`: `compile(root, native, files)` → `Compilation { scripts, registry,
  artifacts, footprints }`; `declaration_registry(native, files)` for observation. Generated
  class: `class {a.name} : public {registry.classes[&a.parent].cpp_name}` (~1384), file
  `scripts/generated/{a.id}.hpp` + `blueprints/{a.id}.epokdebug` (~1601). Parent include chosen
  ~1362–1380 (`epok.hpp` when parent is `epok::Behaviour`).
- Behaviour assumptions in codegen: `this->entity()` at ~584, 590, 648, 844, 898, 906, 921, 925,
  992, 1511; `blueprint_tick(Transform&,Fixed)`/`blueprint_cancel`/`blueprint_observe` overrides
  ~1593–1598; root filter `cpp_name != "epok::Behaviour"` ~86; `project.rs` preview binds
  `b.behaviour->bind(objects[b.entity])` (~559, 576). Spawn API `epok::bp::api::spawn/spawn_class`.
- Parent selection: `blueprint_workflow::draw` uses `Registry::blueprint_parents()`; reparent UI is
  in `blueprint_editor.rs` ~1275–1329 (speculative `asset.parent = new; compile; rollback`).
- Staleness: `blueprint_dependencies::observe_sources` publishes `blueprint:{id}` =
  `semantic_hash()` and `blueprint-audio:{id}`; `scene_dependencies::signature(scene)` hashes the
  whole scene minus `bake`, nodes `scene-file:{relative}`.

## C. Editor view mode (`scene_2d`)

Reads/writes: `editor.rs` 206 (field), 613 (init false), ~1359 (true after UI create);
`gui.rs` ~1503/1510 (true after canvas/rect add), ~1580 checkbox, ~1585 branch to
`hud_editor::view`; `mcp_tools.rs` ~142 (`editor_state` JSON), ~328 (`editor_view` flag loop),
~901 (schema `{"view","grid","wire","scene_2d"}`); `lighting_editor.rs` 77 (false after light);
`platform.rs` 551 (look capture), 588 (render branch), 643 (screenshot); `main.rs` 900/910
(`--screenshot-hud`, `--screenshot-native-hud`). No `mcp_tests.rs` assertion names `scene_2d`.

Hierarchy: `gui::hierarchy` (~1051) builds `children`, `roots`, `visible` (search filter);
`hierarchy_node` (~918). `creation_menu` (~873) returns action strings (`empty`, `add`,
`light-*`, `ui-canvas|panel|image|text|progress`). Inspector `gui::inspector` (~1195): Transform
section gated by `rect.is_none() && canvas.is_none()`; component editors self-guard on
`Option`s; Mesh sections gated by `kind == "Mesh"`.

Scene-level settings: `lighting_editor::window` (~171) edits `scene.environment`/`fog`;
`settings_ui::windows` has a "Current Scene HUD Budget" section (~316). No Map Settings window
exists yet.

## D. Project settings

`workspace::Manifest` (`workspace.rs` ~17–37) is the `.epokproject` YAML: add
`#[serde(default)] pub default_scene_script_parent: Option<String>` next to
`default_sound_bank`; validate in `read_manifest_at`; UI analog `settings_ui.rs` ~257.

## E. Documentation to update in P11

`docs/formats.md` (scene/BP versions), `docs/blueprints.md` (parents, templates, scene BP),
`docs/blueprints-tutorial.md`, `docs/hud.md` (2D/UI rules), `docs/mcp.md` (`editor_view`),
`docs/settings.md` (manifest row), `docs/editor.md` (hierarchy/inspector/view mode),
`docs/features.md`, `docs/projects.md`; regenerate `docs/api/*` via `make api-docs`.
