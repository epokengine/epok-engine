# P11 — Delivery matrix (2026-09-13)

What the actor-architecture initiative delivered, what it did not, what is covered by
a test and what has never been run on this machine. Written from the P0–P10 reports
and the code at the merge commit; where a claim is not backed by a phase report it is
marked as such rather than asserted.

This file is the closing record. [legacy-compat-inventory.md](legacy-compat-inventory.md)
is its companion: every compatibility adapter and pending item with a removal
condition.

## 1. Phases

| Phase | Scope | Status | Report |
| --- | --- | --- | --- |
| P0 | Isolation, baseline, version reservations, `epok::Object` → `LegacyEntityStorage` rename | delivered | [p0-baseline.md](p0-baseline.md) |
| P1 | Reflection schema 8, annotation grammar, host `Model` | delivered | [p1-registry.md](p1-registry.md) |
| P2/P3 | `runtime/object_model.hpp`: identity, pools, registry, `Level`, lifecycle, components, legacy bridge | delivered | [p2-p3-runtime.md](p2-p3-runtime.md) |
| P3 (component adapters) | `Mesh3DComponent`, `Sprite3DComponent`, `Camera3DComponent`, `Light3DComponent`, `Collider3DComponent` | **deferred** — reserved UUIDs only | [p2-p3-runtime.md](p2-p3-runtime.md) §8 |
| P4 | Scene document version 5, `actor_document`, legacy → actor mapping, class redirects | delivered | [p4-documents.md](p4-documents.md) |
| P5 | Actor/Component Blueprints, asset version 4, typed reference ABI, `SpawnActor`, `object_classes[]` | delivered | [p5-blueprints.md](p5-blueprints.md) |
| P6 | Scene Blueprint: discovery, compilation, staleness, editing, map scoping | delivered | [p6-scene-blueprint.md](p6-scene-blueprint.md) |
| P7 | Editor 3D/2D/UI modes, filtered Hierarchy, Actor creation, Map Settings, scene undo | **partial** — component add/remove, actor rename/duplicate/delete/reparent and the hybrid-entity conversion action are not implemented | [p7-editor.md](p7-editor.md) |
| P8 (runtime half) | `runtime/world2d.hpp`: units, projection, hierarchy, draw order, colliders, movement, triggers, picking | delivered | [p8-world2d-runtime.md](p8-world2d-runtime.md) |
| P8 (editor half) | 2D viewport, Camera2D/Sprite2D authoring, 2D gizmos, 2D component cooking, Blueprint component templates | **deferred** | [p8-world2d-runtime.md](p8-world2d-runtime.md) §9 |
| P9 | `AudioComponent` contract and quarantine, target capabilities, `TimelineRequires` on actors, preview phase boundary and protocol v2 | delivered | [p9-services.md](p9-services.md) |
| P9 (deferred) | Installing `audio_source_retained`, calling `dispatch_trigger` from the collision services, positional audio, remaining component adapters, debugger hooks on actor classes | **deferred** | [p9-services.md](p9-services.md) §6 |
| P10 | Cooked actor tables, registry capacity, `main.cpp` wiring, transition teardown, MCP/CLI actor tools, `actor_stats` | delivered, with one recorded deviation (§5) | [p10-cook-runtime.md](p10-cook-runtime.md) |
| P11 (docs) | `docs/actors.md`, `docs/migration-actors.md`, formats/blueprints/editor/hud/mcp/features/projects/README updates, `docs/api` regeneration, this file and the compat inventory | delivered — this report | — |
| P11 (runtime integration) | delivered | [p11-runtime-integration.md](p11-runtime-integration.md) — prepare hook before Begin Play, audio quarantine and trigger hooks installed, map-scoped reference table, capacity diagnostics | `runtime/main.cpp` syntax-reviewed only |
| P11 (editor) | delivered | [p11-editor.md](p11-editor.md) — component add/remove and property overrides, actor rename/duplicate/delete/reparent, coalesced undo, orthographic 2D view, MCP parity | class/domain conversion, Camera2D authoring deferred |

## 2. Test matrix

Rows are the acceptance areas of the plan's section 8. Test names are the real
`#[test]` functions in `src/*.rs` and the real case functions in
`tests/runtime/*.cpp`. Everything listed passes on this host at the time of each
phase report; nothing here is a claim about a MIPS build.

### Herencia — inheritance and families

| Where | Tests |
| --- | --- |
| `src/object_model.rs` | `native_bases_resolve_to_the_documented_contract`, `cpp_and_blueprint_chains_resolve_and_blueprint_never_parents_native`, `legacy_behaviour_chains_stay_in_the_behaviour_family`, `redeclaring_a_different_family_or_domain_is_a_diagnostic`, `missing_and_cyclic_parents_are_reported`, `default_components_accumulate_down_the_chain`, `eligible_parents_placeable_and_scene_script_parents`, `registry_exposes_the_model` |
| `src/blueprint_actor_tests.rs` | `actor_blueprints_derive_across_levels_and_override_each_event_once`, `the_family_hint_never_overrides_the_resolved_parent_chain`, `component_blueprints_resolve_their_owner_through_a_typed_actor_reference` |
| `tests/runtime/object_model.cpp` | `identity_and_storage` (`is_a` across three levels, no slicing) |

### Identidad — stable identities

| Where | Tests |
| --- | --- |
| `src/object_model.rs` | `native_identities_match_the_runtime_header` (parses `runtime/object_model.hpp` and checks every design.md §2 UUID) |
| `tests/runtime/object_model.cpp` | `compact_identities` (all 15 `static_class_id` values against independently computed SHA-256 constants) |
| `src/blueprint_spawn.rs` | `object_class_table_emits_descriptors_pools_and_a_bounded_budget` |
| `src/actor_document.rs` | `duplicate_uuids_across_entities_actors_and_components_are_rejected`, `duplicating_a_scene_remaps_actor_and_component_identities_but_not_assets` |
| `src/scene.rs` | `duplicating_a_map_remaps_the_scene_blueprint_and_its_map_scoped_references` |

### Componentes — contracts, roots, domains

| Where | Tests |
| --- | --- |
| `src/object_model.rs` | `component_owners_may_only_narrow`, `component_contracts_and_placement_stay_inside_their_family`, `component_domains_gate_attachment`, `a_spatial_actor_needs_exactly_one_matching_root`, `requires_excludes_cardinality_and_capabilities`, `target_capabilities_come_from_the_target_not_a_hard_coded_list` |
| `src/header_extract.rs` | `class_options_parse_the_documented_grammar`, `class_options_reject_unknown_or_contradictory_tokens`, `component_field_options_parse_root_name_and_attachment`, `the_runtime_object_model_header_parses_with_this_grammar` |
| `src/actor_document.rs` | `a_ui_component_on_a_three_d_actor_is_rejected_by_the_model` |
| `tests/runtime/object_model.cpp` | `component_rules`, `attachment_rules` |
| `tests/runtime/actor_services.cpp` | `audio_is_domain_agnostic` |

### Valores / templates — defaults and overrides

| Where | Tests |
| --- | --- |
| `src/actor_document.rs` | `version_five_scene_round_trips_actors_components_attachments_and_scene_script`, `two_save_cycles_are_idempotent_for_legacy_and_version_five_documents` |
| `src/scene_bank.rs` | `actor_banks_emit_records_overrides_scene_script_and_registry_capacity`, `an_override_or_scene_script_change_changes_the_cooked_text_and_nothing_else_does` |
| `src/staging_files.rs` | `actor_overrides_and_the_scene_script_are_staged_build_inputs` |
| `tests/runtime/test_actor_tables.cpp` | `load_and_begin_play` (the scene script observes an applied override) |

**Gap.** Blueprint *component templates* and per-actor component defaults/overrides
(P5 deferred them to P8, and the P8 runtime half did not include them) have **no
coverage**, because the feature does not exist. `ComponentInstance::inherited` is
cooked like any other component; whether a component template wants a different rule
is an open question in [p10-cook-runtime.md](p10-cook-runtime.md) §10.

**Gap.** The deviation in §5 below — overrides applied after an actor's own
`begin_play` — is documented but **not pinned by a test asserting the desired
order**. A test that fails until the `prepare` hook exists would be the right tripwire.

### Reparent

| Where | Tests |
| --- | --- |
| `src/object_model.rs` | `reparent_checks_family_domain_derivability_and_cycles` (family crossing, domain, final, non-blueprintable, cycle, self, unknown) |
| `src/blueprint_actor_tests.rs` | `reparenting_an_actor_blueprint_onto_a_component_is_rejected_by_the_model` |
| `src/editor.rs` | `changing_the_scene_blueprint_parent_is_a_transaction_that_rolls_back` |

**Gap.** Reparenting a placed *actor* in the Hierarchy has **no coverage** because
there is no UI for it (P7 deferred). `ActorInstance::logical_parent` is only
reachable through MCP `scene_add_actor` / `scene_set_actor`.

### Lifecycle

| Where | Tests |
| --- | --- |
| `tests/runtime/object_model.cpp` | `lifecycle_order`, `end_play_once`, `deferred_mutations`, `capacity_and_rollback`, `activation_propagation` |
| `tests/runtime/test_actor_tables.cpp` | `load_and_begin_play`, `tick_order_runs_actors_then_script`, `transition_ends_the_script_first_and_keeps_legacy_teardown`, `a_bank_without_actors_still_has_one_scene_script` |
| `tests/runtime/actor_services.cpp` | `slot_backed_audio_never_double_plays`, `owned_audio_plays_once`, `owned_audio_stops_on_disable_and_end_play`, `owned_audio_quarantine_blocks_reuse`, `legacy_frame_update_runs_while_paused`, `legacy_trigger_forwarding` |
| `src/blueprint_actor_tests.rs` | `actor_blueprint_latent_graphs_pump_tasks_from_tick_and_cancel_in_end_play`, `authored_tick_graph_keeps_its_body_behind_the_synthesized_pump` |

### Script de mapa — scene Blueprint

| Where | Tests |
| --- | --- |
| `src/blueprint_scene_tests.rs` | `a_map_blueprint_is_discovered_compiled_and_reported_for_the_level_loader`, `two_maps_sharing_a_cpp_scene_script_parent_both_compile`, `a_scene_blueprint_that_does_not_derive_from_scene_script_actor_is_refused`, `map_scoped_references_resolve_by_uuid_and_scope`, `a_standalone_blueprint_refuses_map_scoped_references`, `a_graph_literal_cannot_carry_a_map_identity`, `observation_publishes_the_embedded_document_like_any_other_source` |
| `src/scene.rs` | `a_map_without_a_scene_blueprint_gets_the_default_in_memory_and_saves_it`, `duplicating_a_map_remaps_the_scene_blueprint_and_its_map_scoped_references` |
| `src/scene_dependencies.rs` | `the_scene_signature_covers_the_embedded_scene_blueprint` |
| `src/editor.rs` | `changing_the_scene_blueprint_parent_is_a_transaction_that_rolls_back` |
| `src/actor_document.rs` | `a_scene_script_actor_may_not_be_placed_and_must_parent_the_scene_blueprint` |
| `tests/runtime/test_actor_tables.cpp` | all four cases (one script per bank, begins last, ends first) |

**Gap.** Binding `Compilation::scene_references` into the cooked level is not done, so
a map-scoped reference is null at run time. No test asserts the *bound* value,
because there is nothing to bind it with yet.

### Vistas — 3D / 2D / UI authoring modes

| Where | Tests |
| --- | --- |
| `src/scene_view_mode.rs` | `view_modes_round_trip_their_wire_values_and_the_legacy_boolean`, `history_is_bounded_and_redo_survives_until_the_next_edit`, `reset_forgets_the_history_of_the_previous_document` |
| `src/mcp_tests.rs` | `editor_view_translates_the_legacy_scene_2d_flag_and_the_three_way_view_mode` |
| `src/editor.rs` | `scene_undo_restores_the_previous_document_and_stops_at_the_oldest_step` |

**Gap.** The Hierarchy's **domain filtering and presentation roots** have no automated
coverage: the rules live in `gui.rs` drawing code and the only ImGui interaction test
in that module (`gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`)
is on the P0 pre-existing-failure list for an unrelated shared-context reason. P7
verified the selector and the map-root line by running
`cargo run -- --project examples/sample-game --screenshot`, which is an observation,
not a regression test.

### 2D

| Where | Tests |
| --- | --- |
| `tests/runtime/world2d.cpp` | `units_and_validation`, `camera_projection`, `hierarchy`, `draw_order`, `overlaps`, `rays`, `movement`, `triggers`, `picking`, `size_report` |

**Gap.** Everything above the header: no 2D viewport, no Camera2D/Sprite2D authoring,
no `Sprite2DComponent` / `Camera2DComponent` / `Collider2DComponent`, no cooking of 2D
content and therefore no 2D editor or cook test. `epok::Actor2D` can be placed and
saved today, but nothing renders or simulates it.

### Legacy

| Where | Tests |
| --- | --- |
| `src/actor_document.rs` | `legacy_entities_map_to_actors_deterministically_without_touching_the_scene`, `the_derived_view_satisfies_the_model_and_reuses_authored_actors`, `duplicate_and_delete_branch_carry_and_drop_the_actors_of_the_branch`, `cyclic_logical_parents_and_dangling_references_are_rejected`, `attachment_across_domains_is_rejected_only_when_a_model_is_available`, `unknown_class_and_invalid_class_names_are_rejected`, `version_six_is_rejected_as_written_by_a_newer_editor`, `saving_actor_content_raises_the_version_and_classic_scenes_stay_at_three` |
| `src/scene.rs` | `every_supported_scene_version_loads_and_a_newer_document_is_refused`, `actor_content_survives_a_document_round_trip_and_raises_the_version` |
| `src/blueprint_refs.rs` | `legacy_class_redirects_are_derived_from_the_scene_and_resolve_transitively` |
| `src/reflection_schema.rs` | `version_seven_classes_deserialize_without_actor_metadata` |
| `src/blueprint_asset.rs` | `version_three_assets_keep_their_semantic_hash_under_the_family_hint` |
| `src/blueprint_spawn.rs` | `native_only_games_do_not_emit_blueprint_factories` |
| `src/scene_bank.rs` | `a_scene_without_actors_emits_the_empty_table_and_unchanged_legacy_text` |
| `tests/runtime/object_model.cpp` | `legacy_adapter` |
| `tests/runtime/actor_services.cpp` | `legacy_frame_update_runs_while_paused`, `legacy_trigger_forwarding` |

**Gap.** The redirect table is derived and tested but **applied by nobody**; see
[legacy-compat-inventory.md](legacy-compat-inventory.md) §5.

### Export

| Where | Tests |
| --- | --- |
| `src/scene_bank.rs` | `generated_actor_tables_reference_no_absolute_worktree_path` (no `CARGO_MANIFEST_DIR`, no drive letter, no `..` in an `#include`) |
| `src/staging_files.rs` | `actor_overrides_and_the_scene_script_are_staged_build_inputs` |

**Gap.** No standalone export of a project containing actor content has been produced
or rebuilt. `--export-psx` is in the not-run list below. The export path is believed
to be covered because `project::runtime_sources()` carries `object_model.hpp`,
`world2d.hpp`, `actor_blueprint.hpp` and `actor_tables.hpp`, but that is an argument,
not a test.

### Recursos — capacity and budgets

| Where | Tests |
| --- | --- |
| `src/blueprint_spawn.rs` | `object_class_table_emits_descriptors_pools_and_a_bounded_budget` (the 64 KiB static-assert text) |
| `src/scene_bank.rs` | `actor_banks_emit_records_overrides_scene_script_and_registry_capacity` |
| `tests/runtime/object_model.cpp` | `capacity_and_rollback`, `size_report` |
| `tests/runtime/world2d.cpp` | `size_report` |

**Gap.** Every number is a **host** measurement. No PSX RAM, EXE-size or frame-budget
figure exists for actor content; see §3 and §4.

### MCP, CLI and preview (not a plan section 8 row, recorded for completeness)

| Where | Tests |
| --- | --- |
| `src/mcp_tests.rs` | `actor_tools_place_validate_edit_and_undo_through_the_class_model` |
| `src/hud_simulation.rs` | `protocol_version_mismatch_is_rejected`, `phase_and_blueprint_support_come_from_the_child`, `blueprint_driven_scene_reports_its_own_diagnostic` |
| `src/timeline_compile.rs` | `timeline_requires_is_checked_against_an_actor_component_set` |
| `src/object_model.rs` | `timeline_requirements_map_onto_the_reserved_component_classes`, `timeline_requirement_diagnoses_an_actor_without_the_component` |

## 3. Not validated on this machine

`mipsel-none-elf-g++`, PCSX-Redux and a working libclang extraction are all absent
here; every phase report says so and never claims otherwise
([p0-baseline.md](p0-baseline.md)). The first machine with the PlayStation SDK must
run, in this order:

```
cargo run --locked -- --project examples/sample-game --build-psx
python tests/integration/verify_selected_scene_build.py
python tools/profile_runtime.py --output artifacts/actor-architecture/profile.json
```

(those three lines are [p10-cook-runtime.md](p10-cook-runtime.md) §9 verbatim, with
`--output` added because `tools/profile_runtime.py` requires it.)

Additionally, for the items P0 and the export gap identify:

```
cargo run --locked -- --project examples/rpg-2-5d-demo --build-psx
python tools/profile_runtime.py --project examples/rpg-2-5d-demo \
    --output artifacts/actor-architecture/rpg-after.json      # and the same before, per p0-baseline.md
python tools/compare_runtime.py --baseline artifacts/actor-architecture/rpg-before.json \
    --candidate artifacts/actor-architecture/rpg-after.json
cargo run --locked -- --project examples/sample-game --export-psx
cargo run --locked -- --project examples/sample-game --reflect
```

What each must confirm:

| # | Claim that is currently unverified | Evidence needed |
| --- | --- | --- |
| 1 | `runtime/main.cpp` compiles and links for MIPS against the generated tables, and the generated `#define EPOK_OBJECT_REGISTRY_CAPACITY` / `#include "actor_tables.hpp"` preamble agrees with its real include order | a successful `--build-psx` |
| 2 | The object pools and the registry fit the image budget | the P5 64 KiB static-assert holding **and** the linker's real data footprint in `.epok/build/epok.map` |
| 3 | The Clang extractor reports the native bases with the identities the cook emits, so `object_classes[]` is populated in a real project | `--reflect` on a project deriving from an `epok::` actor base; every test in this initiative builds its catalog by hand |
| 4 | `EPOK_CLASS(... Owners=A\|B ...)` survives `#__VA_ARGS__` stringification with the pipes intact, and an `EPOK_COMPONENT` field of class type does not trip the defaulted-constructor check | the same `--reflect` run |
| 5 | The extractor reports `epok::Actor`'s lifecycle events with the ids the generated overrides expect | the same `--reflect` run, then `--compile-blueprints` on an Actor Blueprint |
| 6 | `epok::actor_stats` appears in `.epok/build/epok.map` as `_ZN4epok11actor_statsE` and the profiler's `playback.actor` report is `available: true` with plausible values | `tools/profile_runtime.py` |
| 7 | A scene transition in the emulator keeps the frame budget with a level loaded | a profiled emulator route, compared against the same route before the initiative |
| 8 | The PSX RAM cost of `ObjectRegistryStorage<N>` + `Level` + `CollisionWorld2D` | measured, not estimated; the host sizes in §4 are 64-bit upper bounds |
| 9 | A standalone export containing actor content rebuilds without the editor | `--export-psx`, then the export's own `build.sh` / `build.ps1` |

No overhead percentage, EXE size delta or FPS figure is asserted anywhere in this
initiative. P0 could not capture a numeric PSX baseline either, so the first SDK
machine has to produce both sides of the comparison.

## 4. Measured host sizes

Host measurements only: clang++ x86-64 with 64-bit pointers and vtable pointers. The
MIPS image uses 32-bit pointers, so the object-model figures are **upper bounds**; the
`world2d.hpp` types contain no pointer or vtable, so those should carry over except
for alignment padding.

From [p2-p3-runtime.md](p2-p3-runtime.md) §6 (`tests/runtime/object_model.cpp::size_report`):

| Type | bytes | Type | bytes |
| --- | --- | --- | --- |
| `Object` | 16 | `ActorComponent` | 56 |
| `Actor` | 96 | `SceneComponent3D` | 112 |
| `Actor3D` | 208 | `SceneComponent2D` | 88 |
| `Actor2D` | 184 | `UIComponent` | 56 |
| `UIActor` | 216 | `RectTransformComponent` | 120 |
| `SceneScriptActor` | 96 | `AudioComponent` | 88 |
| `Level` | 952 | `LegacyBehaviourComponent` | 80 |
| `World` | 32 | `ObjectId` | 4 |
| `ObjectSlot` | 24 | `ClassDescriptor` | 72 |
| `ObjectRegistryStorage<32>` | 800 | | |

Pool storage: `ObjectPool<Actor3D,4>` 868 B, `ObjectPool<SceneComponent3D,4>` 484 B,
`ObjectPool<AudioComponent,4>` 388 B.

From [p8-world2d-runtime.md](p8-world2d-runtime.md) §7 (`tests/runtime/world2d.cpp::size_report`):

| Type | bytes | Type | bytes |
| --- | --- | --- | --- |
| `Camera2D` | 24 | `Collider2D` | 36 |
| `ColliderEntry2D` | 52 | `Aabb2D` | 16 |
| `Affine2D` | 24 | `Transform2D` | 24 |
| `SpatialHit2D` | 32 | `MoveResult2D` | 24 |
| `Pick2DEntry` | 32 | sine table (rodata) | 1028 |
| `CollisionWorld2D<32>` (64 pairs) | 2976 | `CollisionWorld2D<32, 256>` | 5280 |

`CollisionWorld2D<32>` measured at three pair capacities: `<32,1>` 2224 B,
`<32,32>` 2592 B, `<32,64>` 2976 B — 12 B per remembered trigger pair.

Fixed runtime bounds, for reference: `level_actor_capacity = 64`,
`actor_component_capacity = 8`, `level_pending_capacity = 16`,
`object_hierarchy_depth = 16`, `world2d_depth_limit = 32`,
`world2d_position_limit = 8192`, `world2d_scale_limit = 64`,
`pixels_per_unit_2d = 32`, `ObjectPool<T,4>` per concrete class,
`EPOK_OBJECT_REGISTRY_CAPACITY` = max over banks of (actors + components) + 32,
minimum 32.

## 5. Known deviations from design.md

Collected from every phase report. Each was recorded deliberately; none is a silent
divergence.

| # | Deviation | Why | Report |
| --- | --- | --- | --- |
| 1 | A family declaration is an error only when the **inherited** family is not `Object`. design.md §3 says any redeclaration is an error, but §2 has the four family roots all deriving from `epok::Object`, which declares `Family=Object`. | Without the exception the header of §2 cannot resolve at all. | P1 |
| 2 | `ComponentContract::cardinality` and `can_root` **widen only**; `requires`/`excludes`/`capabilities` accumulate. design.md fixes only the `owners` narrowing rule. | A subclass declaring any component token would otherwise silently reset an inherited `Cardinality=Multiple`. Narrowing cardinality is therefore not expressible; no base needs it. | P1 |
| 3 | `KNOWN_CAPABILITIES` was a placeholder in P1 and became `TargetCapabilities` in P9. design.md defers capability availability to the target. | P1 had no target table; P9 built one from `project::runtime_sources()`, so "does PSX provide `physics2d`" is literally "is `world2d.hpp` cooked". | P1, P9 |
| 4 | Diagnostics are collected, not fatal-on-first: `Model::from_registry` drops only the classes that failed and returns every diagnostic. | A project with one broken class still produces a reviewable list. | P1 |
| 5 | `ObjectRef`/`ActorRef`/`ComponentRef` all lower to `epok::ObjectId`. design.md §4 lists three `Type` variants without fixing an ABI. | The static class on the pin is an authoring constraint; the registry's generation and class check already covers the runtime side. Making the three differ would cost image size for nothing. | P5 |
| 6 | `SpawnActor` is a **new** builtin, not a change to `SpawnClass`. The brief says "make SpawnActor (existing spawn builtin) reject ClassRef whose base is not Actor-family". | `SpawnClass` is the existing `ClassRef` spawn and is legitimately used with Behaviour bases today; gating it would break working content and an existing test. | P5 |
| 7 | Actor/Component classes carry **no** debugger hooks. | `blueprint_class_id`, `epok_debug` and `epok::bp::trace` are `epok::Behaviour` virtuals; emitting them on an Actor would not compile or would add a dead virtual. `#line` mapping is kept. | P5 |
| 8 | The Behaviour "owner is alive" guard is not applied to Actors. | For an Actor the entity handle is routinely null (2D/UI actors), so the same guard would silently disable those classes. `Level`'s deferral governs actor lifetime instead. | P5 |
| 9 | `Editor::ensure_scene_script` does nothing when the project's model has no `epok::SceneScriptActor`. `Scene::ensure_scene_script` itself is unconditional. | Writing a parent the project cannot compile would turn a missing toolchain into a broken document. Documented in `docs/actors.md` and `docs/blueprints.md`. | P6 |
| 10 | Map-scoped references are accepted in **persisted defaults only**, never in graph literals. | A literal inside a graph is code with no per-instance slot for the loader to bind. | P6 |
| 11 | **Cooked configuration is applied between the actors' `begin_play` and the scene script's**, not before `begin_play` as design.md §6 step 2 requires. | `Level::spawn_batch` runs `begin_play` itself and exposes no defaults hook, and `Actor`'s identity fields are private to `Level`. Visible consequence: a Blueprint actor's `begin_play` graph reads class defaults, not instance overrides. The fix is a `prepare` callback overload in `runtime/object_model.hpp`. | P10 |
| 12 | `upgrade_entity_ids` **lowers** a version 5 document back to 3/4 when all its actor content is removed. | The literal reading of the design rule, and it keeps classic documents classic. P4 flagged it for confirmation; P7 did not revisit it. Documented in `docs/migration-actors.md`. | P4 |
| 13 | `actor_view` derives one actor per legacy entity, **including** UI entities the cook still treats as pure HUD data. | P4 left the 1:1-vs-collapse decision to P10; P10 cooked whatever the document contains and did not collapse canvas subtrees. Still open. | P4, P10 |
| 14 | `audio_source_retained` is an installed function pointer rather than a direct reference to `music_active`. | `runtime/music.hpp` defines `music_active` as an `inline` variable, and an earlier `extern` declaration of the same name is ill-formed ([dcl.inline]). Nothing installs the hook yet. | P9 |
| 15 | Native HUD preview protocol version 2 adds version and capability words to the frame header. design.md §9 only says "bump when its ABI changes". | A cached executable built from an older header is now rejected with a message instead of being decoded with the wrong layout. | P9 |

## 6. Notes on this delivery

- **`docs/api` was regenerated on this host** with `tools/generate-api-reference.py`
  and pinned libclang 18.1.1. Two host-specific obstacles had to be worked around and
  are worth knowing before someone runs `make api-docs-check` on macOS:
  1. the script passes `-I<engine root>` to Clang, and on a case-insensitive
     filesystem `#include <version>` from libc++ then resolves to the repository's
     `VERSION` file, which breaks every standard-library concept and silently
     degrades every `Fixed`/`FixedPoint` parameter to `int`. The generation for this
     delivery was therefore run against a directory of symlinks (`runtime`,
     `third_party`, `.git`) that contains no `VERSION`, with `CPATH`/`SDKROOT`
     pointing at the host toolchain's headers;
  2. the generator embeds the generating machine's absolute paths into the labels of
     anonymous records (`catalog.json`, `coverage.json` and five PsyQo module pages).
     The committed output already carried the original author's Windows paths; the
     regenerated output was normalized back to those spellings so no new local path
     entered the repository, and the parse diagnostics in `coverage.json` were made
     repo-relative.

  Both are pre-existing generator flaws, not actor-architecture problems. Fixing (1)
  properly means not putting the engine root on the include path; fixing (2) means
  rendering anonymous records by a relative path.
- The regenerated reference added `epok/object-model.md` (140 callables),
  `epok/world2d.md` (45), `epok/actor-tables.md` and `epok/actor-blueprint.md`, and
  also picked up headers the committed reference had never covered
  (`debug-hud`, `hud-core`, `motion-interpolation`, the `sequence-*` and `serial-*`
  families): the checked-in output was stale at 51 Epok headers and is now 66.
  `coverage.json` records three modules with host parse diagnostics
  (`actor-blueprint`, `debug-hud`, `adler32`); the committed output had none, and
  those diagnostics come from macOS system headers rather than from Epok's.
- Nothing in `src/` or `runtime/` was touched by the documentation task.
