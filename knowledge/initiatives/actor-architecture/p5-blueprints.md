# P5 — Actor and Component Blueprints (2026-09-13)

Blueprint authoring, compilation and cooking for the Object/Actor/Component model.
Everything a legacy `Behaviour` Blueprint generates is unchanged, byte for byte: the
compiler only leaves the legacy path when a class actually resolves outside the
`Behaviour` family.

## 1. Which families a Blueprint may join

`script_backend::can_derive` stays the provider/backend gate and was not touched. Family
awareness was added on top of it:

| Surface | Behaviour | Actor | Component |
| --- | --- | --- | --- |
| `blueprint_workflow::create` / parent picker (`Registry::blueprint_parents`) | yes | yes | yes |
| Seeded lifecycle graphs (`ensure_default_events`) | `start`, `update`, `on_trigger` | `begin_play`, `tick`, `end_play` | `begin_play`, `tick`, `end_play` |
| `scripts::create_in` (new C++ class) | `epok::Behaviour` with an `update` override | any `epok::` actor base, no override needed | any `epok::` component base |
| Reparent (`blueprint_editor`) | model-validated, then speculative compile | same | same |

`blueprint_workflow::parent_family` is the cheap probe both the workflow and the compiler
use: it walks the parent chain by id, honours a declared `Family=` and otherwise
recognises the native roots `epok::Actor` / `epok::ActorComponent`. Everything else is
`Behaviour`, which is also the schema default, so a project with no object-model classes
never builds the resolved `Model` at all.

`scripts::create_in` grew one branch. An `epok::` runtime base (`Actor`, `Actor3D`,
`Actor2D`, `UIActor`, `SceneScriptActor`, `ActorComponent`, `SceneComponent3D`,
`SceneComponent2D`, `UIComponent`, `RectTransformComponent`, `AudioComponent`) is not in
the `assets/scripts` catalog, so the generated header includes `epok.hpp` and the class
body is empty: every Actor/Component event has a default, so the class is concrete as
written. The `epok::Behaviour` branch is untouched and still emits the
`update(Transform&, Fixed)` override.

Reparenting now asks `object_model::Model::validate_reparent` (through
`Registry::model()`) *before* the speculative compile, and shows the diagnostic codes —
`family-crossing`, `domain-mismatch`, `parent-not-derivable`, `cyclic-parent` — which say
why a parent is impossible in a way a compile error cannot. When the model does not know
the class or the candidate parent (a Blueprint that has never compiled), the old
speculative path runs unchanged. Undo is unchanged: the old parent is restored and no
graph data is touched on rejection.

## 2. Blueprint asset version 4

`VERSION = 4`; `load` accepts `[1, 2, 3, 4]` and still migrates in memory only.

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub family: Option<crate::reflection_schema::ClassFamily>,
```

The hint is advisory. The registry's resolved family always wins, and a hint that
disagrees is a compile diagnostic naming both families. Because the field is skipped when
`None`, a version-3 asset serializes to exactly the bytes it did before, so
`semantic_hash()` is unchanged and no existing Blueprint looks stale after the upgrade.
`version_three_assets_keep_their_semantic_hash_under_the_family_hint` pins both the bytes
and the digest against values computed outside this code path.

## 3. Generated code for an Actor Blueprint

`Self` is the Actor. The parent is a runtime class, so the include is `epok.hpp`; the
Behaviour-only hooks (`blueprint_class_id`, `epok_debug`, `epok::bp::trace`,
`blueprint_tick`/`blueprint_cancel`/`blueprint_observe`) are absent. Excerpt for
`BP_Goblin : epok::Actor3D` with an authored `tick` containing a Delay:

```cpp
class BP_Goblin : public epok::Actor3D {
public:
    BP_Goblin() { using epok::Fixed; }
    virtual void begin_play() override {
        epok::Entity* epok_self = epok::bp::actor_entity(this); (void)epok_self;
        const auto epok_owner = epok_self ? epok::handle(epok_self) : epok::EntityHandle{};
        (void)epok_owner;
        epok::Actor3D::begin_play();
    }
    // Authored tick graph, moved behind a private name so the reflected event has
    // exactly one override.
    void epok_graph_tick(epok::Fixed dt) { /* ... continuation frame ... */ }
    epok::bp::Continuations<8> epok_tasks;
    void tick(epok::Fixed dt) override {
        epok::Entity* epok_self = epok::bp::actor_entity(this); (void)epok_self;
        const auto epok_owner = epok_self ? epok::handle(epok_self) : epok::EntityHandle{};
        (void)epok_owner;
        epok_tasks.advance(dt, epok::blueprint_scene_generation);
        epok::bp::Continuation epok_cont;
        while (epok_tasks.poll(epok_cont)) { switch (epok_cont.node) { /* ... */ } }
        this->epok_graph_tick(dt);
    }
    void end_play(epok::EndPlayReason reason) override {
        epok_tasks.cancel_all();
        this->epok_graph_end_play(reason);
    }
};
```

Rules behind that shape:

- **Events.** The existing event-override mechanism is reused unchanged: the graph's
  `override_id` is a reflected function id, so `begin_play`, `tick(Fixed)`,
  `end_play(EndPlayReason)`, `on_enable` and `on_disable` are overridden the same way
  `update` is for a Behaviour, and Call Parent emits one qualified
  `epok::Actor3D::tick(...)` dispatch.
- **The owner entity.** `epok::Actor` has no universal transform, so `this->entity()` may
  legitimately be null (always, for `Actor2D` and `UIActor`). Generated bodies resolve it
  once through `epok::bp::actor_entity(this)` and build a null handle when it is absent;
  every `epok::bp::api` node already treats a null `EntityHandle` as a no-op. A latent
  frame that aliases an owner `Transform&` emits `if (!epok::bp::actor_entity(this))
  return;` before the alias, so a transform is never dereferenced through a missing slot.
- **Continuations.** An Actor has no `blueprint_tick` hook, so the reflected `tick` *is*
  the pump. When the class owns latent frames, the authored `tick` and `end_play` graphs
  are emitted as private `epok_graph_tick` / `epok_graph_end_play` and a single
  synthesized override drives them. The synthesized `tick` calls `Parent::tick(dt)` only
  when there is no authored tick graph (otherwise the graph's own Call Parent would
  dispatch it twice); the same rule applies to `end_play`. The Behaviour
  `blueprint_tick`/`blueprint_cancel` emission is untouched.
- **Components.** A Component Blueprint is the same shape with `Self` as the component.
  The `GetOwner` builtin lowers to `epok::bp::component_owner_id(this)` and is rejected
  with a node-located diagnostic in any other family.

`runtime/actor_blueprint.hpp` is new (included from `runtime/blueprint_spawn.hpp`, staged
by `project::runtime_sources`). It holds the null-safe adapters — `actor_entity`,
`actor_handle`, `component_owner`, `component_owner_id`, `component_entity`,
`component_handle`, `actor_transform`, `component_transform`, `spawn_actor` — so those
null checks exist once, compile for MIPS with the rest of the runtime and are covered by
the host suite instead of being re-emitted as text per class.

## 4. Typed reference ABI — decision

`ObjectRef`, `ActorRef` and `ComponentRef` all lower to **`epok::ObjectId`**: the compact
`{index, generation}` pair, one 32-bit word, null as `epok::ObjectId{}`.

Why one representation for three pin types:

- It is an identity, not a pointer. A stale id fails the generation *and* class check in
  `ObjectRegistry::resolve<T>`, so it can never reach freed storage — the same guarantee
  `EntityHandle` gives for legacy slots, which is why `EntityRef` is left exactly as it
  was rather than being folded into this.
- The static class on the pin is an authoring constraint, not a runtime one:
  `blueprint_ir::assignable` narrows `ActorRef<A>` to `ActorRef<B>` through
  `class_is_a`, and widens Actor/Component references to `ObjectRef`. Making the three
  differ at runtime would cost image size and buy nothing the registry does not already
  check.
- Persisted content only ever carries the null identity (`script_values`), so no handle is
  ever written to a document — the rule design.md section 6 states.

`SpawnActor { base }` is a **new** builtin rather than a change to `SpawnClass`: the
existing `Spawn`/`SpawnClass` nodes keep spawning legacy Behaviour-bound entities, byte
for byte. `SpawnActor` takes a `ClassRef` plus an `ActorRef` logical parent, returns
`ActorRef<base>`, and is rejected at compile time when the base does not resolve to the
Actor family (`Spawn Actor requires an Actor class; <name> belongs to the <family>
family`) or when used from a Behaviour Blueprint. At run time
`epok::bp::spawn_actor` re-checks the descriptor family and routes through the owning
`Level`, so a stale compact id spawns nothing instead of constructing the wrong type.

## 5. Cooked `object_classes[]`

`blueprint_spawn::object_class_table(catalog)` emits the table
`runtime/object_model.hpp` declares, appended to the generated Blueprint header after the
existing `classes[]` Behaviour table (which is untouched). One row per class whose
resolved family is not `Behaviour`:

```cpp
namespace epok {
inline const ClassDescriptor object_classes[] = {
{UINT64_C(<id>),UINT64_C(<parent>),epok::ObjectFamily::Actor,epok::ObjectDomain::World3D,0,6,
 &epok::object_construct<epok::Actor3D>,&epok::object_destruct,sizeof(epok::Actor3D),
 alignof(epok::Actor3D),&epok::ObjectPool<epok::Actor3D,4>::acquire,
 &epok::ObjectPool<epok::Actor3D,4>::release},
};
[[gnu::used]] inline const size_t object_class_count=<n>;
static_assert((epok::ObjectPool<...,4>::storage_bytes+...) <= 65536,
              "Object pools exceed the 64 KiB cook limit");
}
```

- Ids and parent ids are `blueprint_refs::compact_id`, the same SHA-256 prefix the header
  computes at compile time, so the Rust and C++ values cannot drift.
- `owners_mask` is `object_domain_bit` per owner domain (World3D=1, World2D=2, UI=4);
  `flags` are Abstract=1, Placeable=2, Spawnable=4, SceneManaged=8, Root=16, Multiple=32.
- Concrete classes get `create`/`destroy` (`object_construct<T>` / `object_destruct`, the
  placement-new pair an actor's embedded default component needs), `size`/`align` and
  `acquire`/`release` from `ObjectPool<T,4>`. Abstract bases stay in the table so
  `object_class_is_a` can walk them, but carry no factories.
- Rows are sorted by compact id and the budget terms are sorted by class name, so the
  emitted text is a stable build input. The 64 KiB budget mirrors the existing
  `TypedPool` assert.
- `inline const` (not plain `const`) is deliberate: it satisfies the header's
  `extern const ClassDescriptor object_classes[]` declaration without an ODR hazard if the
  generated header is ever included from more than one translation unit.

## 6. Deferred

| Deferred | Where it belongs |
| --- | --- |
| Wiring `object_classes[]` / `World` / `Level` into the scene banks and `main.cpp` | P10 |
| Component Blueprint *templates* and per-actor component defaults/overrides | P8 |
| Editor UI for components (picker, Inspector sections, hierarchy of components) | P6–P8 |
| Blueprint debugger for Actor/Component classes (`epok_debug`, `epok::bp::trace`, `blueprint_class_id` are Behaviour virtuals; an Actor class carries the `#line` source mapping but no hooks) | P9 |
| `Actor::default_root()` overrides generated for Blueprint classes with declared default components | P8 |
| Scene Blueprint (`SceneScriptActor`) authoring flow and Map Settings | P6 |
| Typed reference *pins to a live object* in persisted defaults (only the null identity is writable today) | P8 |
| MIPS build, standalone export and emulator measurement | first machine with the SDK |

## 7. Validation

| Step | Command | Result |
| --- | --- | --- |
| Targeted | `cargo test --locked blueprint` | 82 passed, 0 failed, 11 ignored |
| Targeted | `cargo test --locked --bin epok-editor blueprint_compile::tests::actors` | 7 passed |
| Full suite | `cargo test --locked` (bin `epok-editor`) | **413 passed, 7 failed, 33 ignored** — baseline on this branch was 404/7/33; same 7 pre-existing failures, +9 tests, no new failure |
| Full suite | `cargo test --locked --bin epok-header-tool` | 9 passed, 0 failed |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` | 25 errors, identical to the pre-existing baseline; none in a file or region this phase added |
| Format | `cargo fmt --all` | new code is rustfmt-clean; the repository is not fmt-clean at baseline, so the 37 unrelated files and 3 unrelated hunks inside owned files were reverted |
| Build | `cargo build --locked --bins` | success |
| Host C++ | `python3 tests/runtime/verify_spatial.py` | pass, including the new `tests/runtime/actor_blueprint.cpp` |
| Host C++ | `python3 tests/runtime/verify_blueprint_runtime.py` | pass (MIPS syntax step skipped, no SDK) |
| MIPS / export / emulator | — | **not run**, no `mipsel-none-elf-g++` on this machine (p0-baseline.md) |

Pre-existing failures, unchanged: `audio_contract_tests::audio_authoring_valid_target_error_and_tool_identity`,
`audio_contract_tests::audio_legacy_golden_outputs`,
`console::autoscroll_preserves_read_only_selection_and_copy`,
`editor::tests::attachment_undo_restores_overrides_and_rejects_intervening_edits`,
`editor::tests::invalid_source_marks_outputs_stale_and_keeps_running_snapshot`,
`gui::interaction_tests::scene_clicks_and_hierarchy_context_menu_use_real_imgui_events`,
`file_watch::tests::native_events_detect_create_rename_delete_and_preserved_timestamp`.

### Tests added

`src/blueprint_actor_tests.rs` (included from `blueprint_compile::tests::actors`), against
a hand-built registry using the real native identities of design.md section 2:

| Test | Covers |
| --- | --- |
| `actor_blueprints_derive_across_levels_and_override_each_event_once` | `Actor3D → BP_Goblin → BP_GoblinFire`, correct derivation at both levels, one override per event, one qualified Call Parent, `epok.hpp` include, no Behaviour hooks, no raw `&this->entity()` |
| `actor_blueprint_latent_graphs_pump_tasks_from_tick_and_cancel_in_end_play` | synthesized `tick` advancing `epok_tasks` and dispatching the parent, `end_play` cancelling them |
| `authored_tick_graph_keeps_its_body_behind_the_synthesized_pump` | one `tick` override, authored graph under `epok_graph_tick`, no double parent dispatch |
| `component_blueprints_resolve_their_owner_through_a_typed_actor_reference` | Component derivation, `epok::ObjectId` ABI, `component_owner_id`, Get Owner rejected in a Behaviour Blueprint |
| `spawn_actor_requires_an_actor_class_reference` | `SpawnActor` accepted for an Actor base, rejected for a Behaviour base |
| `the_family_hint_never_overrides_the_resolved_parent_chain` | matching hint compiles, mismatching hint diagnoses |
| `reparenting_an_actor_blueprint_onto_a_component_is_rejected_by_the_model` | `validate_reparent` `family-crossing`, same-family reparent allowed |

Also `blueprint_spawn::tests::object_class_table_emits_descriptors_pools_and_a_bounded_budget`
(descriptor text, flags, owners mask, pool wiring, budget assert, deterministic order,
empty table for a Behaviour-only catalog),
`blueprint_asset::tests::version_three_assets_keep_their_semantic_hash_under_the_family_hint`,
and the host test `tests/runtime/actor_blueprint.cpp`, which compiles and runs the exact
emitted table and class shapes against the real runtime.

## 8. Deviations and notes

1. **`SpawnActor` is a new builtin, not a change to `SpawnClass`.** The brief says "make
   `SpawnActor` (existing spawn builtin) reject ClassRef whose base is not Actor-family".
   `SpawnClass` is the existing ClassRef spawn, and it is legitimately used today with
   Behaviour bases; rejecting those would break working content and an existing test.
   The Actor-family gate therefore lives on a new `Builtin::SpawnActor { base }`.
2. **Actor/Component classes carry no debugger hooks.** `blueprint_class_id`,
   `epok_debug` and `epok::bp::trace` are `epok::Behaviour` virtuals. Emitting them on an
   Actor would either not compile (`override`) or add a dead virtual. The generated text
   keeps its `#line` source mapping, so stack traces still point at nodes; binding the
   trace protocol to the object model is P9.
3. **The `epok_owner`-is-alive guard is Behaviour-only.** The Behaviour `blueprint_tick`
   bails out when its entity handle dies mid-tick. For an Actor the handle is routinely
   null (2D/UI actors), so the same guard would silently disable those classes. Actor
   lifetime is instead governed by `Level`'s deferral, which already stops ticking a
   doomed actor.
4. **The resolved model is built lazily.** `compile` probes each Blueprint's family from
   the registry first and only calls `Registry::model()` when something is outside the
   Behaviour family or carries a family hint. A project with a broken class graph that
   contains no object-model classes therefore compiles exactly as before rather than
   failing on someone else's diagnostic.
5. **`object_classes[]` is emitted but not yet consumed.** It lands in the generated
   Blueprint header next to `classes[]`. Wiring it into the scene banks (and giving
   `Level`/`World` their process lifetime) is P10, so a cooked game today still runs the
   legacy path; the table is inert until then.
6. **Extraction was not run.** libclang needs the MIPS include paths, which this machine
   does not have (p0-baseline.md). Every test here builds its registry by hand from the
   section 2 identities. The first SDK machine must confirm that the real extractor
   reports the `epok::Actor` lifecycle events with the ids the generated overrides expect.
