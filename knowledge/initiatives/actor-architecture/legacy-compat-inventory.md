# Legacy compatibility inventory (P11)

Every adapter, alias, derived view, placeholder and "not wired yet" item the
actor-architecture initiative introduced or leans on, with the condition that lets it
be deleted. Nothing here is user documentation; the user-facing behaviour is in
`docs/actors.md` and `docs/migration-actors.md`.

The phase reports that introduced each item are linked; when a report and this file
disagree, the report is the record of what was done and this file is the record of
what still has to happen.

---

## 1. `LegacyEntityStorage` and the `Entity` alias

| | |
| --- | --- |
| Where | `runtime/epok.hpp` — `struct LegacyEntityStorage` (~line 137) with `using Entity = LegacyEntityStorage`. Generated code and the runtime headers refer to the slot record as `Entity`: `src/scene_bank.rs`, `src/project.rs`, `src/blueprint_spawn.rs`, `runtime/lifecycle.hpp`, `runtime/actor_tables.hpp`. |
| Why it exists | P0 renamed the old `epok::Object` slot record so that `epok::Object` could become the root of the new class system. The alias keeps every generated table, every host test and every game script compiling unchanged. |
| Fixtures that need it | All of them. Every cooked bank still emits `std::array<Entity,N> objects;`, and `SceneComponent3D::bind_slot` / `RectTransformComponent::bind_slot` point a component's transform at a slot so collision, rendering and motion interpolation keep reading one memory location. `tests/runtime/object_model.cpp::legacy_adapter`, `tests/runtime/test_actor_tables.cpp::load_and_begin_play`. |
| Removal condition | The cook stops emitting a legacy slot table, i.e. every renderable, collidable and audible thing in a cooked bank is an actor component with its own storage. That is well past the component adapters of item 9. Until then the alias is permanent, not debt. |
| Reports | [p0-baseline.md](p0-baseline.md), [p2-p3-runtime.md](p2-p3-runtime.md) §5 |

## 2. `epok::LegacyBehaviourComponent`

| | |
| --- | --- |
| Where | `runtime/object_model.hpp` (~line 544), id `430cb0ca-21c3-420c-96a3-da2f7b99781a`. Derived onto actors by `actor_document::actor_view` (role `behaviour`) in `src/actor_document.rs`. |
| Why it exists | A `Behaviour` binds to one entity. On a migrated actor the same script must keep receiving `start`, `update`, `frame_update`, `on_enable`, `on_disable`, `on_destroy` and `on_trigger` exactly once each. The component forwards them with the entity's `Transform&`. |
| Fixtures that need it | Every project with a Behaviour — all sample and example projects. `tests/runtime/object_model.cpp::legacy_adapter`, `tests/runtime/actor_services.cpp::legacy_frame_update_runs_while_paused` and `::legacy_trigger_forwarding`, `src/actor_document.rs::legacy_entities_map_to_actors_deterministically_without_touching_the_scene`. |
| Invariant to preserve | A bound entity must **not** also appear in the scene bank's `Binding` table. When it does, the Behaviour would receive every event twice. The cook is what keeps the two sets disjoint. |
| Removal condition | The `Behaviour` family itself is retired, or the migration that rewrites Behaviours as Actor/Component classes exists and every shipped project has run it. Neither is planned. |
| Reports | [p2-p3-runtime.md](p2-p3-runtime.md) §5, [p9-services.md](p9-services.md) §5 |

## 3. `actor_document::actor_view` — the derived actor view

| | |
| --- | --- |
| Where | `src/actor_document.rs` (`actor_view`, `derived_id`, salt `epok actor view identity v1`). Consumers: the MCP `scene_actors` tool (`src/mcp_tools.rs`), and the editor wherever an actor list is needed for a map that has not been migrated. |
| Why it exists | P4 decided that loading a legacy map must never rewrite it. The actor half is therefore *derived in memory* from the legacy entities, deterministically (SHA-256 over a fixed salt, the entity UUID and a role string), and skipped for any entity that already has an authored actor. |
| Fixtures that need it | Every legacy map: `examples/sample-game`, `examples/rpg-2-5d-demo`, `examples/timeline-spell`, and the scene fixtures in `src/actor_document.rs` and `src/mcp_tests.rs`. |
| Removal condition | A real migration action exists (an explicit author command that writes the derived actors into the document) **and** every fixture in the repository has been migrated. Note that the view must survive even then, for a user's own unmigrated project, until the scene document drops the `entities` array entirely. |
| Open question carried from P4 | `actor_view` derives one actor per legacy entity, including UI entities that the cook still treats as pure HUD data. P10 cooked whatever the document contains and did not collapse canvas subtrees; whether the cooked form should keep the 1:1 relationship is still undecided. |
| Reports | [p4-documents.md](p4-documents.md), [p10-cook-runtime.md](p10-cook-runtime.md) §10 |

## 4. Hybrid 3D + UI entity diagnostic

| | |
| --- | --- |
| Where | `src/actor_document.rs`, diagnostic code `hybrid-entity-pending-conversion` (~line 918). |
| Why it exists | An entity carrying both 3D data and UI data cannot be derived into one actor of one domain. Splitting it into two actors would break every persisted reference to it, so the view keeps a single `epok::Actor3D` with the 3D components and reports the conversion as pending. |
| Fixtures that need it | The hybrid fixture in `src/actor_document.rs::legacy_entities_map_to_actors_deterministically_without_touching_the_scene`; the fixture scene used by `the_derived_view_satisfies_the_model_and_reuses_authored_actors` filters this code out explicitly, so any *new* diagnostic there fails the test. |
| Removal condition | The editor gains the conversion action P7 deferred to P9 and P9 did not land: the author is asked what the UI half should become, the document is rewritten into two actors, and the references are repaired. Until that action exists the diagnostic is the whole feature. |
| Reports | [p4-documents.md](p4-documents.md), [p7-editor.md](p7-editor.md) *Deferred* |

## 5. `ClassRef` redirects — derived, not applied

| | |
| --- | --- |
| Where | `src/blueprint_refs.rs`: `redirects(scene) -> Vec<Redirect>` and `resolve_class_id(id, &table)`. |
| Why it exists | P4 owed the table design.md §7 reserves for "old Blueprint classes used by Spawn/IsA/ClassRef". Two reasons exist: `BehaviourHostedByLegacyComponent` (a `ScriptBinding::class_id` → `epok::LegacyBehaviourComponent`) and `BlueprintInstanceIsActorClass` (a Blueprint instance class → **itself**; the identity is unchanged but the family is not). |
| Current state | The table is **derived and unit-tested, and no production caller applies it.** P4 explicitly handed the decision of which reason applies to which reference kind to P5, and P5 did not wire it: a `ClassRef` on a Spawn node and a `ClassRef` used for an `IsA` test do not want the same answer. Applying the whole table blindly is a no-op for the second reason by construction. |
| Fixtures that need it | Only `src/blueprint_refs.rs::legacy_class_redirects_are_derived_from_the_scene_and_resolve_transitively`. |
| Removal condition | Either (a) the Blueprint compiler starts calling `resolve_class_id` at the specific reference sites that need it, at which point the `#[allow(dead_code)]` comes off and this entry becomes a real feature; or (b) a decision is recorded that legacy class references never need redirection, and the module and its test are deleted. **Leaving it derived-but-unused indefinitely is the worst of the three.** |
| Reports | [p4-documents.md](p4-documents.md) *Legacy class redirects*, [p5-blueprints.md](p5-blueprints.md) |

## 6. `scene_2d` compatibility accessors

| | |
| --- | --- |
| Where | `Editor::scene_2d()` / `Editor::set_scene_2d()` (`src/editor.rs` ~866), over `Editor::scene_view_mode: SceneViewMode` (`src/scene_view_mode.rs`). Consumers that still speak the boolean: `src/platform.rs` (look capture, render branch, screenshot), `src/main.rs` (`--screenshot-hud`, `--screenshot-native-hud`), `src/lighting_editor.rs`, the canvas/rect creation sites in `src/gui.rs`, and the MCP surface in `src/mcp_tools.rs`. |
| Why it exists | `scene_2d` was the editor's public three-state-as-a-boolean: `true` meant the Canvas/HUD editor. MCP clients written against it must keep working, and the render/screenshot paths genuinely only care about "is this the HUD view". |
| Known lossiness | `SceneViewMode::TwoD` reports `scene_2d: false`. That is the honest legacy answer — a 2D world is not the Canvas editor — but a client polling `scene_2d` to decide whether to take a HUD screenshot sees `false` in a mode with no 3D viewport either. Documented in `docs/mcp.md` and `docs/migration-actors.md`. |
| Fixtures that need it | `src/mcp_tests.rs::editor_view_translates_the_legacy_scene_2d_flag_and_the_three_way_view_mode` pins the whole table including both refusals. |
| Removal condition | Two separate steps. (a) The internal consumers stop using the boolean: `platform.rs` and `main.rs` should branch on `SceneViewMode` once the 2D viewport exists, because "not 3D" will no longer imply "HUD". (b) The MCP `scene_2d` argument and reply field are removed only after a deprecation period long enough for third-party clients; there is no schedule for that and the field costs nothing. |
| Reports | [p7-editor.md](p7-editor.md) |

## 7. Empty `object_classes[]` placeholder

| | |
| --- | --- |
| Where | `src/blueprint_spawn.rs` (`object_class_table`, the empty-catalog branch at ~line 287), emitted unconditionally from `scene_bank::header_with_templates`. The text is `inline const ClassDescriptor object_classes[] = {\n{},\n};` with `object_class_count=0`. |
| Why it exists | `runtime/object_model.hpp` declares `extern const ClassDescriptor object_classes[]` and `object_class_count`, so the symbols must exist in every image or `main.cpp` does not link. A project with no object-model class — every existing project today — would otherwise fail to build. A single default-constructed row is emitted because a zero-length array is not valid C++. |
| Consequence | `find_object_class` resolves nothing and the `Level` stays empty, which is exactly right for a legacy-only project. It also means a project whose class graph is broken but which declares no `Family`/`Domain` anywhere compiles exactly as before rather than failing on someone else's diagnostic. |
| Fixtures that need it | `examples/sample-game` and every other legacy-only project; `src/scene_bank.rs::a_scene_without_actors_emits_the_empty_table_and_unchanged_legacy_text`, `tests/runtime/test_actor_tables.cpp::a_bank_without_actors_still_has_one_scene_script`. |
| Removal condition | Never, as long as the runtime declares the tables `extern`. The alternative — `#ifdef`-ing the declaration — would move the same conditional into the runtime header. Revisit only if `object_classes[]` becomes non-optional for every project. |
| Reports | [p5-blueprints.md](p5-blueprints.md) §5, [p10-cook-runtime.md](p10-cook-runtime.md) §1 |

## 8. `TimelineRequires` mapping gaps

| | |
| --- | --- |
| Where | `object_model::timeline_requirement_component(req) -> Option<&'static str>` (`src/object_model.rs` ~line 130); used by `Model::validate_timeline_requirement` and `timeline_compile::verify_actor_requirements`. |
| The gap | `TimelineComponentRequirement::PaletteAnimator` and `::ParticleEmitter` return `None`: design.md §2 reserves no component UUID for them. The function returns `Option` rather than a name precisely so that an unmapped requirement is **not checked on actors** instead of manufacturing a false diagnostic for every actor. `Camera3DComponent` is mapped but is not reflected by any registry yet, which is the second deliberate non-failure. |
| Consequence | A timeline adapter declaring `TimelineRequires=PaletteAnimator` or `=ParticleEmitter` is fully validated on a legacy entity (`timeline::validate_components` checks the entity member) and silently unvalidated on an actor. |
| Fixtures that need it | `src/object_model.rs::timeline_requirements_map_onto_the_reserved_component_classes` asserts both `None` values, so introducing the classes will fail that test until it is updated — which is the intended tripwire. `src/timeline_compile.rs::timeline_requires_is_checked_against_an_actor_component_set`. |
| Removal condition | `epok::PaletteAnimatorComponent` and `epok::ParticleEmitterComponent` are introduced with reserved UUIDs added to design.md §2 and to the `pub const` block in `src/object_model.rs`; then the two `None` arms become names and the test is updated. Nothing else in the table changes. |
| Reports | [p9-services.md](p9-services.md) §3 |

## 9. Reserved component classes that do not exist yet

| | |
| --- | --- |
| Where | design.md §2 "Reserved for P3/P8 component adapters" and the matching `pub const` identifiers in `src/object_model.rs`. |
| The list | `Mesh3DComponent`, `Sprite3DComponent`, `Camera3DComponent`, `Light3DComponent`, `Collider3DComponent`, `Sprite2DComponent`, `Camera2DComponent`, `Collider2DComponent`, `CanvasComponent`, `ImageComponent`, `TextComponent`, `ProgressBarComponent`. Twelve UUIDs, no C++ class. |
| Where they are nevertheless used | `actor_document::actor_view` names them when it derives a legacy entity, so the derived view already talks about classes the runtime does not implement. This is consistent — the derived view is a *description* of a legacy entity, not something the cook instantiates — but it means the names are load-bearing before the classes exist. |
| Consequence | An authored actor cannot be given a mesh, camera, light, collider, sprite or UI component: the editor's **Add Component** is disabled and the cook has nothing to emit. An authored 3D actor today is a transform, optional audio and whatever a Blueprint graph does. |
| Fixtures that need it | `src/actor_document.rs::legacy_entities_map_to_actors_deterministically_without_touching_the_scene` pins the derived component classes by name. |
| Removal condition | Each class is introduced in `runtime/object_model.hpp` (or a companion header) with its reserved UUID, its `Owners`/`Root`/`Cardinality` contract and its adapter over the existing legacy member, and the reserved-id entry in design.md §2 moves into the main identity table. The derived view then names a real class. `Collider2DComponent` additionally has to bind `runtime/world2d.hpp`'s `Collider2D` and synchronize `CollisionWorld2D` from the `Level`. |
| Reports | [p2-p3-runtime.md](p2-p3-runtime.md) §8, [p8-world2d-runtime.md](p8-world2d-runtime.md) §9, [p9-services.md](p9-services.md) §6 |

## 10. `runtime/main.cpp` is syntax-reviewed only

| | |
| --- | --- |
| Where | `runtime/main.cpp` — three additions: `epok::level.tick(dt)` in the fixed-step loop, `epok::level.frame_update(epok::time.frame_microseconds)` once per rendered frame, and `epok::level.refresh_stats()`. Plus the generated preamble `#define EPOK_OBJECT_REGISTRY_CAPACITY` / `#include "actor_tables.hpp"` and its interaction with the real include order. |
| Why it is unverified | `main.cpp` needs PsyQo, the MIPS toolchain and a real generated `scene.hh`; none is available on the machine every phase ran on. The symbols and signatures those three lines use **are** exercised on the host by `tests/runtime/test_actor_tables.cpp`, which links the real `actor_tables.hpp`, `lifecycle.hpp` and `scene_service.hpp` — but the translation unit itself has never been compiled here. |
| Removal condition | A MIPS build on an SDK machine: `cargo run --locked -- --project examples/sample-game --build-psx`. See the exact command list in [p11-delivery.md](p11-delivery.md) §"Not validated on this machine". |
| Reports | [p10-cook-runtime.md](p10-cook-runtime.md) §8 |

---

## Also pending, not adapters

These are not compatibility shims but they belong on the same removal ledger.

| Item | Where | Removal condition |
| --- | --- | --- |
| Overrides applied *after* an actor's own `begin_play` | `SceneLevel::load_bank` in `runtime/actor_tables.hpp`; deviation recorded in [p10-cook-runtime.md](p10-cook-runtime.md) §3 | A `Level::spawn_batch` overload in `runtime/object_model.hpp` taking a `void (*prepare)(Level&, Actor&, size_t)` callback invoked between defaults and `begin_play`. Until then a Blueprint actor's `begin_play` graph reads class defaults, not instance overrides — documented in `docs/actors.md`. |
| `audio_source_retained` hook not installed | `runtime/object_model.hpp` declares `inline bool (*audio_source_retained)(const AudioSource*) = nullptr`; nothing in `runtime/main.cpp` or `runtime/scene_service.hpp` installs it from `music_active`/`music_requested`, and nothing calls `collect_quarantined()` per frame | Level/World wiring phase. With no hook installed nothing is quarantined and release is immediate, which is correct for a build without the XA music service but wrong for one with it. |
| `dispatch_trigger` never called | `runtime/object_model.hpp` defines it; `runtime/collision.hpp` and `runtime/world2d.hpp` do not call it | Level/World wiring phase. Until then an actor component's `on_trigger` only fires for a Behaviour hosted by `LegacyBehaviourComponent` through the legacy path. |
| Map-scoped references not bound at load | `Compilation::scene_references` is produced by `blueprint_compile` and carries `#[allow(dead_code)]`; the generated constructor writes the null identity | The `Level` loader binds them in step 3 of the lifecycle. Authored values are preserved in the document meanwhile, so nothing is lost — the reference is simply null at run time. |
| Blueprint debugger hooks absent on Actor/Component classes | `blueprint_class_id`, `epok_debug` and `epok::bp::trace` are `epok::Behaviour` virtuals; actor classes carry `#line` mapping but no hooks | Binding the trace protocol to the object model. Node-level stack traces work; per-instance value inspection does not. |
| No `actor_select` over MCP | `entity_select` is entity-indexed (`src/mcp_tools.rs`) | An actor selection tool, once the editor has actor selection over MCP. |
| Spatial `attach` not authorable over MCP | `scene_add_actor` / `scene_set_actor` set the logical parent only | An `attach` argument, or an editor-only decision recorded. |
