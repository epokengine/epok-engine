# P6 — Scene Blueprint (2026-09-13)

The per-map `SceneScriptActor` document becomes a real compiler input: it is
discovered, compiled, observed for staleness and editable, and the compilation
tells P10 which class belongs to which map. Nothing about standalone `.epokbp`
assets changes: an embedded document is a different *source* of the same
`BlueprintAsset`, not a different document shape.

## 1. Document shape

`Scene::scene_script` is the P4 shape, unchanged:

```rust
pub struct SceneScript { pub parent: ClassReference, pub blueprint: BlueprintAsset }
```

`parent` is the authored truth. The embedded `BlueprintAsset::parent` only carries
it, and `blueprint_asset::embedded` overwrites it from `parent.class_id` when it
builds the compiler input, so the two can never disagree in the compiler.

`blueprint_asset::AssetFile` gained one field:

```rust
pub struct AssetFile { pub path: PathBuf, pub asset: BlueprintAsset, pub source: AssetSource }
pub enum AssetSource { File, EmbeddedScene(MapScope) }        // Default: File
pub struct MapScope { actors, components, entities: BTreeSet<Uuid> }
```

`AssetSource::File` is the default and `AssetFile::file(path, asset)` is the
constructor every existing site now uses, so a `.epokbp` input is byte-for-byte
the input it was. For an embedded asset `path` is the `.epokmap` itself: the
document has no file of its own, so the map is what diagnostics point at, what the
artifact dependency graph records, and what Save writes.

`blueprint_asset::load_all(root)` now also walks `assets/scenes` for `.epokmap`
files. A map is only parsed when its bytes contain `scene_script`, so a project
full of classic maps pays one substring scan per map and no YAML parse.

| Property | Value |
| --- | --- |
| Class name | `{scene name}_SceneScript` |
| Class id | the embedded `BlueprintAsset::id`, a normal class UUID |
| Parent | `scene_script.parent.class_id`, which must resolve to a `SceneScriptActor` subclass |
| Generated file | `scripts/generated/{class id}.hpp`, like any other Blueprint |
| Family | Actor (P5 codegen), so it derives from the chain and is emitted into `object_classes[]` by the existing `blueprint_spawn::object_class_table` path |

Because the map is a compiler input in its own right, the class compiles even when
no entity in the document has a Behaviour — the plan's "compilar el script del mapa
incluso cuando ninguna entidad tenga Behaviour". Before this phase
`scripts::compile_blueprints` returned early when `assets.is_empty()`.

## 2. Default creation policy

`Scene::ensure_scene_script(default_parent: Option<&ClassReference>) -> bool`
creates the document in memory when the map has none:

- parent = `default_parent` when the caller resolved the project's
  `default_scene_script_parent` to a `SceneScriptActor` subclass, otherwise
  `epok::SceneScriptActor` (`b4c08aa0-fa85-4abf-8f45-7501e1c8a040`);
- name `{scene name}_SceneScript`, empty graph, fresh class id;
- it does **not** touch `Scene::version`. `upgrade_entity_ids` raises the document
  to 5 at save time, as it already did for actor content.

It is not an edit. `Editor::ensure_scene_script` calls it from `accept_scene` and
from `Editor::from_scene`, before the history baseline is taken and without
`changed()`, so:

- `Scene::load` / `load_unresolved` still never rewrite a document's bytes;
- `dirty` stays what the load produced; only a user edit sets it;
- an untouched default is still written by the next explicit save, which is what
  the plan asks for.

One deviation: `Editor::ensure_scene_script` does nothing when the project's model
has no `epok::SceneScriptActor` — a host that cannot extract the runtime headers,
for instance because the MIPS include paths are missing. Writing a parent the
project cannot compile would turn a missing toolchain into a broken document, and
the sample projects in the test suite are exactly that case. `Scene::ensure_scene_script`
itself is unconditional; the gate is the editor's policy, not the document's.

## 3. Parent rules

| Rule | Where |
| --- | --- |
| The Map Settings combo lists `Model::scene_script_parents()` and nothing else | `gui::scene_blueprint_settings` |
| A parent change is one transaction: `Model::validate_reparent`, then `Scene::validate_with_model`, then a speculative compile of the whole project with the unsaved draft | `Editor::set_scene_script_parent` |
| A refusal restores the previous `SceneScript` whole — parent and graph — and reports why in `last_error` | same |
| Undo/Redo is the map's `History<Scene>` from P7, because the document lives in the scene | same |
| The compiler refuses a parent outside the `SceneScriptActor` chain, pointing at the map | `blueprint_compile::compile` |

The compile-time check is the authority: Map Settings is one way to choose a
parent, and a hand-edited document must be refused the same way.

## 4. Editing

`blueprint_editor::open_embedded(&AssetFile)` opens the map's document. `path` is
the `.epokmap` and `disk` is `None`, so the external-edit baseline that protects a
`.epokbp` does not apply.

- `blueprint_workflow::draw` calls `Editor::sync_embedded_blueprint` every frame.
  An edit is taken out of the Blueprint editor (`take_embedded_edit`) and written
  into `editor.scene.scene_script.blueprint` through `changed()`, so the map's
  dirty flag, undo history and Save are the only ones. A change that came from the
  other direction — a scene Undo, a reopened map — is adopted by the Blueprint
  editor instead (`adopt_embedded`).
- `BlueprintEditor::dirty()` is therefore always false for an embedded document,
  and `Editor::has_unsaved_changes` / `save_all` skip the `.epokbp` save path for
  it entirely.
- The Blueprint editor's **Save** sets `save_to_map`; the workflow turns that into
  `Editor::save_all`, which writes the map. **Revert** is refused with a message
  pointing at Undo Scene Edit.
- Opening a different map discards an embedded document that belonged to the old
  one (`accept_scene`).

### Duplication

`Scene::duplicate_document(new_name)` wraps the P4 `duplicate_identities`: fresh
UUIDs for every entity, actor and component, a fresh class id for the embedded
Blueprint, every UUID literal inside it rewritten through the same table (P4's
`remap_scene_script` walks the asset as JSON, so pins, defaults and variables are
covered), and the class renamed to `{new name}_SceneScript`. Asset UUIDs are not
in the table and survive.

The Project browser's Duplicate now goes through it for `.epokmap`. A byte copy
would have given two maps the same class identity, which the compiler rejects as a
duplicate class UUID, and the same actor identities, which would make the copy
drive the original's actors.

## 5. Reference scoping

`MapScope` is the set of identities a map's Blueprint may name. The compiler
resolves every persisted reference against it in `blueprint_compile::scene_reference`:

| Pin type | Accepted identity |
| --- | --- |
| `ActorRef` | an actor of the owning map |
| `ComponentRef` | a component of an actor of the owning map |
| `EntityRef` | a legacy entity of the owning map |
| `ObjectRef` | any of the three |

- In a standalone Blueprint any of these is refused: *"a map-scoped `ActorRef<...>`
  is only available inside a map's scene Blueprint"*. Before this phase the same
  value failed with "No native value adapter for this type", which said nothing.
- An identity that is not in the owning map is refused naming the identity and the
  map, and the authored value is preserved for repair.
- Resolution happens at compile/cook. Nothing is looked up by name at Tick.

Two deliberate limits:

1. **Only persisted defaults carry a map-scoped reference.** A class property
   default or a Blueprint variable default is per-instance data the Level loader
   can bind at spawn (design.md section 6, step 3). A literal inside a graph is
   code with no per-instance slot, so it is refused with the node it came from and
   a note to promote it to a variable.
2. **The generated constructor still writes the null identity**, exactly as P5
   decided for every typed object reference: `epok::ObjectId{}` / `epok::EntityHandle{}`.
   The resolution is handed to P10 on `Compilation::scene_references` rather than
   emitted as text, because binding it needs the `Level` loader that P10 builds.
   No handle is ever written into generated code or into a document.

The nulling is for code generation only: `files` keeps the authored values, so
semantic hashes, footprints and the document itself are unchanged.

## 6. Dependencies and staleness

Nothing needed a special case:

- `blueprint_dependencies::observe_sources` iterates `files`, so an embedded asset
  publishes `blueprint:{id}` and `blueprint-audio:{id}` like a file asset. Editing
  the map's graph makes its generated class stale.
- `blueprint_dependencies::capture` walks `registry.ancestry`, so the map's
  footprint already contains `blueprint-class:` / `blueprint-layout:` for every
  transitive parent and `blueprint:{id}` for a Blueprint parent. A change to the
  C++ or Blueprint base invalidates the map's class.
- `scene_dependencies::signature` hashes the whole document minus the derived
  lighting bake, so the scene Blueprint is part of the map's provenance with no
  change at all. `the_scene_signature_covers_the_embedded_scene_blueprint` pins
  that, so a future narrowing of `signature` cannot silently drop it.
- The map file is added to `artifacts.dependencies` by the normal
  `artifacts.dependencies.insert(file.path)` in the emit loop.

## 7. What P10 must consume

```rust
pub struct Compilation {
    ...
    pub scene_scripts: Vec<(PathBuf, String)>,   // (map path, generated class id), sorted by map
    pub scene_references: Vec<SceneReference>,   // resolved map-scoped bindings, sorted
}
pub struct SceneReference { pub map: PathBuf, pub class_id: String, pub member: String,
                            pub kind: SceneRefKind, pub target: Uuid }
pub enum SceneRefKind { Actor, Component, Entity }
```

- `scene_scripts` is the map → class table. P10 emits exactly one instance of that
  class per cooked scene bank and gives the `Level` its `scene_script` ObjectId.
  The class is already in `object_classes[]`: it is an Actor-family Blueprint, so
  `blueprint_spawn::object_class_table` picks it up from the catalog with no new
  code.
- `scene_references` is what the loader must bind in step 3 of the lifecycle.
  `member` is `property:<name>`, the persisted member on the generated class.
- Both are `#[allow(dead_code)]` until P10 reads them, with the justification in
  place, exactly as P5 left `object_classes[]` inert.

## 8. Deferred

| Work | Phase |
| --- | --- |
| Emitting the scene-script instance per bank, the `Level`/`World` process lifetime and the transition that drops the previous Level's listeners, continuations and references | P10 |
| Binding `scene_references` into the cooked Level data (the generated constructor writes the null identity today) | P10 |
| Map-scoped references in graph literals (refused today; needs the loader hook) | P10 or later |
| A picker for map-scoped references in the Blueprint editor's variable details (values are typed in today) | P9 |
| Debugger hooks on Actor-family classes, the scene script included | P9 |
| Coalescing `changed()` per gesture, so a graph drag does not fill the 32-step scene history | P9 |
| MIPS build, standalone export and emulator measurement | first machine with the SDK |

## 9. Validation run on this machine

| Step | Result |
| --- | --- |
| `cargo build --locked --bins` | succeeds |
| `cargo test --locked blueprint` | 95 passed, 0 failed, 11 ignored |
| `cargo test --locked scene` | 70 passed, 1 failed - the pre-existing shared-ImGui `gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events` |
| `cargo test --locked actor_document` | 14 passed, 0 failed |
| `cargo test --locked` | **446 passed, 7 failed, 33 ignored** — the 7 are exactly the P0 baseline list, no new failures (P7 left 435 passed) |
| `cargo test --locked --bin epok-header-tool` | 9 passed, 0 failed |
| `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, exactly the pre-existing baseline count; none added |
| `cargo fmt` | run, then every hunk outside the lines this phase wrote was reverted (the repository is not fmt-clean: a plain run reformats 35 unrelated files) |
| MIPS build / standalone export / emulator | **not run**, no `mipsel-none-elf-g++` on this machine (p0-baseline.md); nothing about them is claimed |

## 10. Tests added (11)

`src/blueprint_scene_tests.rs`, included as `blueprint_compile::tests::scene_scripts`
(7). The registry is hand built from the design.md section 2 identities, like the P5
actor tests, and adds a C++ `GameMode : epok::SceneScriptActor`:

| Test | Covers |
| --- | --- |
| `a_map_blueprint_is_discovered_compiled_and_reported_for_the_level_loader` | `load_all` yields the embedded asset from a saved v5 map, the parent comes from `scene_script.parent`, it compiles with no Behaviour anywhere, `scene_scripts` reports it, and the footprint carries the whole transitive parent chain |
| `two_maps_sharing_a_cpp_scene_script_parent_both_compile` | two maps, one C++ base, two distinct classes |
| `a_scene_blueprint_that_does_not_derive_from_scene_script_actor_is_refused` | the parent rule, with the map as the diagnostic's asset |
| `map_scoped_references_resolve_by_uuid_and_scope` | an in-map `ActorRef` resolves into `scene_references`, the generated code carries the null identity and never the UUID, and an out-of-map identity is refused by name |
| `a_standalone_blueprint_refuses_map_scoped_references` | the same value in a `.epokbp` |
| `a_graph_literal_cannot_carry_a_map_identity` | graph literals are refused with their node |
| `observation_publishes_the_embedded_document_like_any_other_source` | `blueprint:{id}` is published for the map's class and changes when the graph changes |

`src/scene.rs` (2):

- `a_map_without_a_scene_blueprint_gets_the_default_in_memory_and_saves_it` — the
  default's parent, name and empty graph, that it is idempotent, that it leaves the
  version at 3 and the file byte-identical, that a chosen parent is used verbatim,
  and that an explicit save writes it as version 5.
- `duplicating_a_map_remaps_the_scene_blueprint_and_its_map_scoped_references` — a
  new class id and name, new actor/entity identities, the reference rewritten to the
  copy's actor, and an asset UUID left alone.

`src/scene_dependencies.rs` (1): `the_scene_signature_covers_the_embedded_scene_blueprint`.

`src/editor.rs` (1): `changing_the_scene_blueprint_parent_is_a_transaction_that_rolls_back`
— the combo lists only `SceneScriptActor` subclasses, a class outside that chain and
an unknown class are both refused with the document unchanged, and re-choosing the
current parent is not an edit.

## 11. Open questions for later phases

- `sync_embedded_blueprint` calls `changed()` once per frame in which the graph
  differs, so a node drag records a scene-history step per frame. P7 already noted
  the same for `changed()`; P9 should coalesce by gesture.
- The map is reparsed by `load_all` on every catalog refresh when it contains
  `scene_script`. With many maps this is one YAML parse each. If it shows up in a
  profile, the map's mtime is the obvious cache key.
- `Editor::ensure_scene_script` declines when the model has no `SceneScriptActor`.
  That is right for a host without extraction, but it also means a project opened
  on such a host never gains the default. P11 should say so in the docs if the
  situation is common.
- Map-scoped references are authored by typing a UUID into a variable default.
  A picker (P9) is what makes the feature usable; the compiler contract is what
  P6 owed.
