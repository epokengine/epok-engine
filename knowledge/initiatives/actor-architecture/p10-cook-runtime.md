# P10 — Cook, runtime wiring, tools and performance (2026-09-13)

Turns the Object/Actor/Component model from an inert header into something a cooked game
actually runs. The Rust cook emits one actor table per scene bank, `main.cpp` owns a
`Level` for the whole image, the scene transition tears it down in the design order, and
the MCP/CLI surface can author actors. Everything a legacy project generates is unchanged
except for the new (possibly empty) tables, which always exist so `main.cpp` links.

New file: `runtime/actor_tables.hpp` (registered in `project::runtime_sources()`) and
`tests/runtime/test_actor_tables.cpp` (registered in `verify_blueprint_runtime.py`).

## 1. Cooked table format

`project::actor_table_body(scene, catalog)` appends this to each bank's generated body,
after the legacy `bindings`/`initialize_scripts` text, so the existing byte shape of the
generated header (`std::array<Entity,36> objects;`, `transform_order`, `Binding`) is
untouched. `scene_bank::header_with_templates` puts the bank body in `epok::scene_{i}`, so
the symbols below are per bank.

```cpp
namespace epok::scene_0 {
inline void actor_apply_0(ObjectRegistry& registry,Actor& actor,const ObjectId* components){
(void)registry;(void)actor;(void)components;
if(auto* self=registry.resolve<BP_Hero>(actor.id())){
self->speed = Fixed(10240, Fixed::RAW);
}
}
inline constexpr ActorComponentRecord actor_components_0[]={
  {UINT64_C(<SceneComponent3D>),"Root",true,-1,0},
  {UINT64_C(<AudioComponent>),"Footsteps",false,-1,-1}};
inline constexpr ActorRecord actor_records[]={
  {UINT64_C(<BP_Hero>),"Hero",true,-1,-1,-1,actor_components_0,2,&actor_apply_0},
  {UINT64_C(<Actor3D>),"Marker",false,0,0,0,actor_components_1,1,nullptr}};
inline constexpr ActorTable actor_table={actor_records,2,UINT64_C(<SceneScriptActor>)};
inline constexpr uint64_t scene_script_class=UINT64_C(<SceneScriptActor>);
inline constexpr size_t actor_registry_slots=7;
}
namespace epok { inline constexpr uint64_t scene_script_class_0=scene_0::scene_script_class; }
```

| Field | Meaning |
| --- | --- |
| `ActorRecord.class_id` | `blueprint_refs::compact_id` of the resolved `ClassModel`, the same SHA-256 prefix `object_model.hpp` computes at compile time. |
| `ActorRecord.name` | The authored name, JSON-escaped; a string literal in rodata, so the pointer outlives a deferred spawn. |
| `ActorRecord.active` | `ActorInstance::active`. |
| `ActorRecord.logical_parent` | Index into this table, or -1. **Parents always precede children**: the loader spawns one batch per generation, because `Level::spawn_batch` can only bind a logical parent whose `ObjectId` already exists. |
| `ActorRecord.attach_actor` / `attach_component` | `ActorInstance::attach`, as a table index plus a component index inside that actor (-1 = its root). Spatial attachment, not the logical hierarchy. |
| `ActorComponentRecord.root` | The declarative root the actor embeds. The cook does not create it; it names it and binds it. |
| `ActorComponentRecord.attach_parent` | Component index inside the same actor. |
| `ActorComponentRecord.legacy_slot` | Entity index of `ActorInstance::legacy_entity`, emitted only on a root record. The root then uses `bind_slot`, so the canonical `LegacyEntityStorage` transform stays the single source that collision, rendering and motion interpolation read. -1 means component-owned storage. |
| `ActorRecord.apply` | Generated setters for the reflected property overrides of the actor and of its components, through the existing `blueprint_refs::assignment` generator. `nullptr` when nothing is overridden. |

Rules:

- Overrides are taken from `overrides` when it is non-empty, otherwise from every key in
  `properties` (which is what `ActorInstance::Deserialize` already implies). A key the
  class chain does not declare is a **cook diagnostic naming the key**
  (`Actors: actor Hero: class BP_Hero declares no property stamina`), never a silent drop.
- Nothing new is persisted. Every index is derived from the order of `scene.actors`;
  identities come from the resolved `object_model::Model`.
- Rows are emitted in document order (stable after the parents-first sort), so re-cooking
  the same document is byte identical. Asserted by
  `actor_banks_emit_records_overrides_scene_script_and_registry_capacity`.
- A bank with no actors and no scene script emits
  `inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};` and
  `actor_registry_slots=0`. The loader then still creates one `SceneScriptActor`.

### `object_classes[]`

`blueprint_spawn::object_class_table` is now emitted **unconditionally** from
`scene_bank::header_with_templates`: the runtime declares `object_classes[]` and
`object_class_count` `extern`, so the symbols must exist in every image. A project with no
object-model class gets the placeholder
`inline const ClassDescriptor object_classes[] = {\n{},\n};` with `object_class_count=0`;
`find_object_class` then resolves nothing and the level stays empty. A project whose class
graph is broken but which declares no `family`/`domain` anywhere still compiles exactly as
before rather than failing on someone else's diagnostic.

## 2. Capacity policy

`EPOK_OBJECT_REGISTRY_CAPACITY` is emitted by `scene_bank::header_with_templates` before
`#include "actor_tables.hpp"`:

```
max over banks of (actors + components) + 32 dynamic slots, minimum 32
```

The 32 covers runtime `SpawnActor`, the `Level` itself (adopted into a slot by `bind`),
the scene script and its root. Exhausting the table increments `ObjectStats::rejected` and
returns a null `ObjectId`; it never overwrites memory. The other bounds are unchanged
runtime constants: `level_actor_capacity = 64`, `actor_component_capacity = 8`,
`level_pending_capacity = 16`, `object_hierarchy_depth = 16`, and the per-class
`ObjectPool<T,4>` with the 64 KiB static-assert from P5.

`runtime/actor_tables.hpp` owns the state so there is exactly one owner in the image:

```cpp
inline ObjectRegistryStorage<EPOK_OBJECT_REGISTRY_CAPACITY> object_registry;
inline SceneLevel level;
```

`SceneLevel` is a `Level` subclass declared in the same header; it adds the cooked-table
loader and a class-id-addressed `add_component_by_class`. `runtime/object_model.hpp` was
not touched.

## 3. Load, tick and teardown order as wired

**Bank load** (generated `load_bank_{i}`, before `activate_texture_bank` and the legacy
`scene_{i}::initialize_scripts()`):

```
load_actor_bank(scene_{i}::actor_table, objects.data(), object_count);
```

`SceneLevel::load_bank` then runs:

1. `ensure_bound(object_registry)` — binds the slot table once and publishes
   `active_object_registry`; a transition reuses the same registry and Level identity.
2. `Level::spawn_batch`, one batch per logical-parent generation: reserve, defaults
   (level id, name, active flag, logical parent, declarative root), register, and the
   actors' `begin_play` (components before their owning actor, in batch order).
3. Cooked configuration per actor: authored components by class id, `bind_slot` for a root
   backed by a legacy entity, component and actor attachment, then `apply` for the
   property overrides.
4. `begin_play` (and `on_enable` when the actor is active) for the components added in
   step 3. `begin_play_component` is idempotent, so the declarative root is not dispatched
   twice.
5. `create_bank_scene_script` — the cooked class when the map authored one, otherwise the
   base `epok::SceneScriptActor`; exactly one per bank — then `begin_play_scene_script()`.

So the scene script always observes fully configured actors.

**Deviation from design.md section 6, recorded deliberately.** The design wants
defaults/overrides applied *before* an actor's own `begin_play` (step 2 of the spawn
order). `Level::spawn_batch` runs `begin_play` itself and exposes no defaults hook, and
`Actor`'s identity fields (`m_level`, `m_logical_parent`, `m_active`, name) are private to
`Level`, so a subclass cannot reproduce the defaults step. This phase therefore applies the
cooked configuration between the actors' `begin_play` and the scene script's. The visible
consequence: a Blueprint actor class with an authored `begin_play` graph reads its class
defaults, not its per-instance overrides, inside that one callback. Everything after
(`on_enable`, `tick`, the scene script, other actors) sees the overrides. The fix belongs
in `runtime/object_model.hpp` — a `spawn_batch` overload taking a
`void (*prepare)(Level&, Actor&, size_t)` callback invoked between step 2 and step 5 — and
is owned by another agent this phase. Tracked below under open issues.

**Simulation step** (`runtime/main.cpp`, inside the fixed-step loop):

```
legacy bindings blueprint_tick/update  (unchanged)
epok::level.tick(dt)                   // actors+components in table order, script last
refresh_collisions(); ...
```

**Frame update** (once per rendered frame, even while paused, for legacy Behaviour
parity): the legacy `bindings` / `bp::visit` loops first, then
`epok::level.frame_update(epok::time.frame_microseconds)`.

**Scene transition** (`runtime/scene_service.hpp::scene_tick`), guarded by
`#ifdef EPOK_ACTOR_TABLES` so the pre-existing host tests that include this header without
a level are unaffected:

```
lifecycle_tearing_down = true;
unload_actor_bank();          // level.end_play_all(LevelUnloaded):
                              //   scene script end_play first, actors still alive,
                              //   then each actor in reverse table order
<legacy slots marked dead, generations bumped>
<bp::retire, legacy binding blueprint_cancel/on_disable/on_destroy>
reset_runtime_services();
lifecycle_tearing_down = false;
scene_banks[next].load();     // load_bank_N -> load_actor_bank -> ...
```

The level teardown runs while both the actors and their legacy slots are still alive, and
before the legacy binding teardown, exactly as the brief requires. No heap is used anywhere
on this path; every array is a fixed-size local or static.

## 4. Staging inputs and staleness

`scene_dependencies::signature` serializes the whole `Scene` minus the derived lighting
bake, so `actors` and `scene_script` are already part of `scene-file:` / `scene-editor:`
provenance. This phase pins that with tests rather than adding a parallel node:

- `staging_files::tests::actor_overrides_and_the_scene_script_are_staged_build_inputs` —
  a byte-identical re-save certifies; adding an actor property override makes
  `BuildTicket::complete` fail and marks `scene-file:…` stale on `stage:.epok/build`; the
  scene script and the overrides both change the signature; the lighting bake still does
  not.
- `scene_bank::tests::an_override_or_scene_script_change_changes_the_cooked_text_and_nothing_else_does`
  — the same at the cook level: changing an override or the scene script parent changes the
  emitted text, and an unrelated legacy edit (an entity material) leaves the actor half
  byte identical.

`project::runtime_sources()` now carries `actor_tables.hpp`, so the header is staged into
the build and into the native HUD preview, and a change to it is an ordinary build input.

**Build independence.** The generated tables reference only staged runtime headers:
`#include "actor_tables.hpp"` with no directory part, next to `epok.hpp`.
`scene_bank::tests::generated_actor_tables_reference_no_absolute_worktree_path` asserts
that the generated text contains no `CARGO_MANIFEST_DIR` and that no `#include` line
carries a drive letter or a `..` segment.

## 5. MCP and CLI surface

New tools (documented in `docs/mcp.md`, including the legacy command translation table):

| Tool | Arguments | Notes |
| --- | --- | --- |
| `scene_actors` | — | Read-only, allowed while the editor is busy. Authored actors + the derived view of unmigrated legacy entities + derivation diagnostic codes + the placeable class list. |
| `scene_add_actor` | `revision`, `class`, `name?`, `parent?` | `class` is gated by `object_model::Model::placeable()`; the root component for the class domain is added; the scene is then checked with `Scene::validate_with_model`. |
| `scene_remove_actor` | `revision`, `id` | Clears, rather than dangles, references from the survivors. |
| `scene_set_actor` | `revision`, `id`, `active?`, `name?`, `properties?` | A `null` property value clears the override. |

Every mutation goes through the same transaction the entity tools use (`install` →
`Editor::changed()`) and is pushed onto the shared MCP history, so P7's undo/redo reverses
actor edits together with entity edits. Only *authored* actors can be edited: a derived
actor is the in-memory view of a legacy entity and migrating it is an explicit editor
action.

CLI parity: `--project <folder> --add-actor <class>:<name> [--scene assets/scenes/X.epokmap]`
applies the same `placeable()` gate, the same root component and the same
`validate_with_model` before saving, defaulting to the project's startup scene.

## 6. Performance counters

`runtime/actor_tables.hpp` publishes `epok::actor_stats` (`ActorStats`), a plain `uint32`
struct in the same shape as `spawn_stats`/`particle_stats`:

`alive peak rejected spawned deferred actors components scene_scripts banks_loaded`

`tools/profile_runtime.py` is table-driven, so it gained one `PLAYBACK_STATS` row
(`"actor": ("actor_stats", …)`); gauges are `alive`, `peak`, `actors`, `components`. It is
refreshed once per frame after `audio_tick()` and on every bank load/unload. A build
without the symbol reports `available: false`. `docs/performance.md` documents the fields
and the capacity constants.

## 7. Host test

`tests/runtime/test_actor_tables.cpp` (in `verify_blueprint_runtime.py`'s list) links the
real `actor_tables.hpp`, `lifecycle.hpp` and `scene_service.hpp` against a hand-written
cooked table **in the exact shape the cook emits** (records, component arrays, the
`actor_apply_0` signature and body), plus a hand-written `object_classes[]`. It asserts:

| Case | Covers |
| --- | --- |
| `load_and_begin_play` | one `SceneScriptActor` exists and is the cooked class; components begin before their actor and the script begins last; the script sees `speed`, the bound legacy slot and the configured component; logical parent and cross-actor attachment resolved from table indices; legacy bindings start after the actor half, once; `actor_stats` counts. |
| `tick_order_runs_actors_then_script` | components tick before their actor, actors before the scene script, legacy binding updates once. |
| `transition_ends_the_script_first_and_keeps_legacy_teardown` | `end_play(LevelUnloaded)`: scene script, then components, then actors; the legacy binding `on_destroy` follows, exactly once; the next bank comes up with its own script. |
| `a_bank_without_actors_still_has_one_scene_script` | the empty table still produces exactly one base `SceneScriptActor` and ticks safely. |

## 8. Validation on this machine

| Step | Command | Result |
| --- | --- | --- |
| Host C++ | `tests/runtime/verify_blueprint_runtime.py` | pass, including the new `test_actor_tables` (MIPS syntax step skipped: no SDK) |
| Host C++ | `tests/runtime/verify_spatial.py` | pass |
| Targeted | `cargo test --locked scene_bank` | 9 passed, 0 failed, 1 ignored |
| Targeted | `cargo test --locked staging` | 11 passed, 0 failed, 1 ignored |
| Targeted | `cargo test --locked mcp` | 9 passed, 0 failed |
| Full suite | `cargo test --locked` | **442 passed, 7 failed, 33 ignored** — baseline at `1c4a506` was 435/7/33; the same 7 pre-existing failures (p0-baseline.md), +7 tests, no new failure |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` | 23 file-level errors, byte-identical to the pre-existing baseline; none in a file or region this phase added |
| Format | `cargo fmt` | new code is rustfmt-clean; the repository is not fmt-clean at baseline, so rustfmt's reformatting of pre-existing code was reverted in every touched file |
| Build | `cargo build --locked --bins` | success |
| MIPS build / export / emulator | — | **not run** |

### Compile coverage of the runtime wiring

`runtime/actor_tables.hpp`, the cooked table shape, `scene_service.hpp`'s teardown and the
whole `SceneLevel` loader are **compiled and executed** on the host by
`test_actor_tables.cpp` (clang++, `-std=c++20 -Wall -Wextra -fno-rtti -fno-exceptions`).

`runtime/main.cpp` cannot be compiled here: it needs PsyQo, the MIPS toolchain and a real
generated `scene.hh`. Its three additions (`epok::level.tick(dt)`,
`epok::level.frame_update(...)`, `epok::level.refresh_stats()`) are **syntax-reviewed
only** — the symbols and signatures they use are exercised by the host test, but the
translation unit itself has not been compiled on this machine. The same applies to the
interaction between the generated `#define EPOK_OBJECT_REGISTRY_CAPACITY` / `#include
"actor_tables.hpp"` preamble and the real include order of `main.cpp`.

## 9. What remains unverified

The first machine with the PSX SDK must run, in this order:

```
cargo run --locked -- --project examples/sample-game --build-psx
python tests/integration/verify_selected_scene_build.py
python tools/profile_runtime.py
```

and confirm:

1. `main.cpp` compiles and links against the generated tables for MIPS, with the object
   pools and the registry inside the image budget (the P5 64 KiB static-assert and the
   linker's real data footprint).
2. The Clang extractor reports the native bases with the identities the cook emits, so
   `object_classes[]` is populated in a real project (every test here builds its catalog by
   hand — libclang needs the MIPS include paths, p0-baseline.md).
3. `epok::actor_stats` appears in `.epok/build/epok.map` as `_ZN4epok11actor_statsE` and
   the profiler's `playback.actor` report is `available: true` with plausible values.
4. A scene transition in the emulator keeps the frame budget with a level loaded, and the
   PSX RAM cost of `ObjectRegistryStorage<N>` + `Level` is measured rather than estimated
   (the host sizes in p2-p3-runtime.md section 6 are upper bounds for 64-bit pointers).

## 10. Open issues for later phases

- **Overrides before `begin_play`** (section 3). Needs a defaults hook in
  `Level::spawn_batch` (`runtime/object_model.hpp`). Until then a Blueprint actor's
  `begin_play` graph must not read its own overridden properties.
- **UI actors and the HUD cook.** P4 left open whether a canvas subtree stays 1:1 with its
  actors. This phase cooks whatever the document contains and binds a `UIActor` root to its
  legacy slot with `RectTransformComponent::bind_slot`; the HUD component data is still
  emitted by the legacy path. Collapsing the two is not done.
- **Component property overrides on `inherited` components.** `ComponentInstance::inherited`
  is cooked like any other component; Blueprint component templates (P8) may want a
  different rule.
- **Spatial attachment over MCP.** `scene_add_actor`/`scene_set_actor` set the logical
  parent only; `attach` is authored in the editor.
- **Selection.** `entity_select` is still entity-indexed; there is no `actor_select`.
