# P11 (runtime part) — closing the runtime/cook integration gaps (2026-09-13)

Closes the five gaps the earlier phases left open on the runtime and cook side: the P10
deviation that applied cooked overrides *after* an actor's `begin_play`, the two P9
service hooks that were deferred because `main.cpp` and `scene_service.hpp` belonged to
another phase, the P6 `scene_references` that were resolved but never cooked, the capacity
policy that silently grew instead of failing, and the byte-neutrality guarantee for
projects that use none of this.

Nothing is persisted by this phase. Every table is derived from the map document and the
resolved `object_model::Model`; no handle, index or UUID is written into a document.

## 1. Overrides before `begin_play` (P10 deviation 1, closed)

`Level::spawn_batch` gained an optional preparation hook:

```cpp
using ActorPrepareFn = void (*)(Level&, Actor&, size_t batch_index);
size_t spawn_batch(const ActorSpawnRequest*, size_t count, ObjectId* out,
                   ActorPrepareFn prepare = nullptr);
```

It runs once per reserved actor **after** the defaults, roots, owners and registration of
step 2/4 and **before** any `begin_play` of the batch (design.md section 6, between steps
2 and 5). A function pointer rather than a virtual on purpose: a nested batch — a spawn
deferred out of a callback and flushed at the end of the current one — passes no hook, so
it can never be mistaken for a row of the cooked table being loaded.

`SceneLevel` (runtime/actor_tables.hpp) now does all of its cooked configuration there:
authored components by class id, `bind_slot` for a root backed by a legacy entity,
component and cross-actor attachment, and the generated `apply` for the property
overrides. The order a cooked bank now runs is:

```
Level::spawn_batch -> reserve, defaults, roots, owners, register
                   -> prepare_cooked_actor (components, bind_slot, attach, overrides)
                   -> begin_play: components first, then the owning actor
create_bank_scene_script -> create, bind scene references, begin_play
```

Consequences:

- A Blueprint actor class with an authored `begin_play` graph reads its **per-instance
  overrides**, its authored components and its bound legacy slot — the open issue from
  p10-cook-runtime.md section 10 is gone.
- `SceneLevel::finish_components` is removed: the cooked components exist before
  `begin_play_actor` runs, so the base `Level` begins and enables them, once, on the
  normal path. No second dispatch and no second `on_enable`.
- Cross-actor attachment whose target belongs to a *later* spawn generation cannot be
  resolved during prepare (the target has no `ObjectId` yet). `load_bank` retries exactly
  those rows after every batch; an attachment the runtime *rejects* (domain mismatch,
  cycle) is counted once by `attach_component` and not retried.
- Every `class EPOK_CLASS(...)` declaration line in `runtime/object_model.hpp` is
  unchanged, so the Rust identity test still parses the header.

## 2. The two P9 service hooks

Both predicates are *installed*, not hard-wired: `runtime/object_model.hpp` is compiled
before `runtime/music.hpp` defines `music_active` as an `inline` variable and before the
generated scene bank declares the legacy `bindings` table. With no hook installed the
behaviour is exactly what it was.

`runtime/scene_service.hpp` (which already sees both) owns them:

| Hook | Predicate | Installed by |
| --- | --- | --- |
| `audio_source_retained` | `music_retains_audio_source` — `music_active == source \|\| music_requested == source \|\| (music_lookup && music_active == source)`, the same rule `create_entity` applies to a legacy slot's `audio` | `install_actor_service_hooks()` |
| `component_trigger_filtered` | `legacy_bindings_notify` — true for a `LegacyBehaviourComponent` whose `Behaviour` appears in the bank's `bindings` table | same |

`install_actor_service_hooks()` is called from `GameScene::start()` (before the first bank
load, so no component-owned `AudioSource` can be recycled while the XA consumer points at
it) and again at the top of `scene_tick()` (idempotent; keeps it installed across a
transition that replaced the bindings table).

`epok::collect_object_quarantine()` (actor_tables.hpp) retries the quarantined slots; it
is called once per frame from `main.cpp`, right after `audio_tick()`, next to the point
where the legacy path re-checks `music_active`.

### The trigger/audio double-delivery rule

**The legacy `bindings` table wins.** A migrated entity is reachable from both sides: the
scene bank still notifies its `Behaviour` directly through `bindings`, and the actor half
may own a `LegacyBehaviourComponent` wrapping the very same `Behaviour`. `dispatch_trigger`
therefore skips any component `component_trigger_filtered` claims, so a `Behaviour` in the
bindings table receives `on_trigger` exactly once — from the legacy loop — and a component
adapter for a `Behaviour` that is *not* in the table is the only delivery path for it. The
rule is the same one `LegacyBehaviourComponent`'s own documentation states for
start/update/frame_update.

Fan-out entry point (`runtime/actor_tables.hpp`):

```cpp
ObjectId actor_for_slot(const Entity* slot);   // root component bound to that slot
size_t dispatch_slot_trigger(EntityHandle self, EntityHandle other, TriggerPhase);
```

`main.cpp`'s `dispatch_trigger(const epok::TriggerEvent&)` calls it once per colliding
slot, after the legacy and Blueprint binding loops. An actor participates when its **root**
component is bound to the colliding slot — that root is the canonical transform the
collision world read. Inactive, unstarted, doomed and unknown actors receive nothing
(`epok::dispatch_trigger` already enforced that).

## 3. The `scene_reference` table

P6 lowers every persisted `ActorRef`/`ComponentRef`/`EntityRef` default of a map's own
Blueprint to the null identity and resolves the authored UUID against the map's scope. The
cook now emits that resolution per bank, as **table indices** — never a UUID, never a name,
never a handle in a document:

```cpp
enum class SceneRefKind : uint8_t { Actor = 0, Component = 1, Entity = 2 };
struct SceneReferenceRecord {
    uint64_t class_id;   // compact id of the owning class (the cooked scene script)
    uint64_t member;     // compact id of the persisted member the writer switches on
    SceneRefKind kind;
    int16_t actor;       // actor table index, for Actor and Component targets
    int16_t component;   // component index inside that actor, for Component targets
    int16_t entity;      // legacy entity slot index, for Entity targets
};
```

`ActorTable` gained three **trailing** fields — `references`, `reference_count` and
`bind_reference` — all defaulted, so a bank with no map-scoped reference emits exactly the
three-field initializer it always did.

Typed member access needs the C++ class, so the cook emits the writer and keeps the table
plain data:

```cpp
inline void scene_reference_bind(ObjectRegistry& registry,Actor& owner,uint64_t member,ObjectId target,EntityHandle entity){
(void)registry;(void)owner;(void)member;(void)target;(void)entity;
if(auto* self=registry.resolve<Actors_SceneScript>(owner.id())){
if(member==UINT64_C(...)){self->hero = target;}
if(member==UINT64_C(...)){self->marker_slot = entity;}
}
}
inline constexpr SceneReferenceRecord scene_references[]={...};
inline constexpr ActorTable actor_table={actor_records,2,UINT64_C(...),scene_references,2,&scene_reference_bind};
```

`SceneLevel::bind_scene_references` runs between `create_scene_script` and
`begin_play_scene_script`, so the script reads live `ObjectId`s/`EntityHandle`s in its own
`begin_play` (design.md section 6, step 3). A row whose `class_id` the instance is not an
`is_a` of binds nothing, so a stale row cannot write into the wrong class.

The cook resolves the rows through `blueprint_compile::map_scene_references(map, scene)`,
which builds the embedded `AssetFile` and reuses the same `scene_reference()` resolver
`compile` uses, rather than threading `Compilation::scene_references` through
`scripts::catalog`: the only inputs are the map document and the asset embedded in it.

**Limitation:** only the asset's own `variables` are walked. A map-scoped reference typed
into an *inherited* property override (`BlueprintAsset::properties`) is resolved by
`compile` but not cooked. No fixture produces one today; it is listed under open issues.

## 4. Capacity is a diagnostic, never a silent increase

`scene_bank::header_with_templates` mirrors the runtime's fixed bounds and refuses a map
that would exceed one, naming the map and the bound:

| Bound | Constant | Diagnostic |
| --- | --- | --- |
| 64 actors per level | `epok::level_actor_capacity` | `Actors: 65 actors exceed the runtime level capacity of 64 (epok::level_actor_capacity); split the map or raise the runtime constant` |
| 8 components per actor | `epok::actor_component_capacity` | `Actors: actor A0 has 9 components, above the runtime limit of 8 per actor (epok::actor_component_capacity)` |
| 608 registry slots | `64 * (1 + 8) + 32` | `the cooked object registry would need N slots, above the 608 EPOK_OBJECT_REGISTRY_CAPACITY supports` |

Inside the bounds the emitted `#define EPOK_OBJECT_REGISTRY_CAPACITY` is the computed
value, never rounded up. Typed pool budgets keep the existing mechanism — the cook emits
`static_assert((… ObjectPool<T,4>::storage_bytes …) <= 65536, "Object pools exceed the 64
KiB cook limit")`, which fails the *target* build rather than growing a pool; a Rust test
now pins that the assert is emitted with the pools of the cooked classes.

## 5. Byte neutrality

`a_scene_without_actors_emits_the_empty_table_and_unchanged_legacy_text` now removes the
documented additive lines from the generated header and asserts that **nothing** of the
initiative survives the removal (`ActorTable`, `ActorRecord`, `ActorComponentRecord`,
`SceneReference`, `scene_reference`, `scene_script`, `actor_registry_slots`,
`object_class`, `ClassDescriptor`, `EPOK_OBJECT_REGISTRY_CAPACITY`, `load_actor_bank`,
`ObjectPool`). The exact additive set, for a two-bank project with no actors, no scene
script and no object-model class, is:

```
#define EPOK_OBJECT_REGISTRY_CAPACITY 32
#include "actor_tables.hpp"
inline const ClassDescriptor object_classes[] = {\n{},\n};
inline const size_t object_class_count=0;
per bank: inline constexpr ActorTable actor_table={nullptr,0,UINT64_C(0)};
per bank: inline constexpr uint64_t scene_script_class=UINT64_C(0);
per bank: inline constexpr size_t actor_registry_slots=0;
per bank: inline constexpr uint64_t scene_script_class_{i}=scene_{i}::scene_script_class;
per bank: load_actor_bank(scene_{i}::actor_table,objects.data(),object_count);
```

A future phase cannot quietly add a tenth line to every existing project's header without
this test failing.

## 6. Validation on this machine

| Step | Command | Result |
| --- | --- | --- |
| Host C++ | `tests/runtime/verify_blueprint_runtime.py` | pass (MIPS syntax step skipped: no SDK) |
| Host C++ | `tests/runtime/verify_spatial.py` | pass |
| Host C++ | `tests/runtime/verify_sprites_particles.py` | pass |
| Targeted | `cargo test --locked scene_bank` | 12 passed, 0 failed, 1 ignored (3 new) |
| Targeted | `cargo test --locked object_model` | 18 passed, 0 failed |
| Full suite | `cargo test --locked` | **463 passed, 7 failed, 33 ignored** — the same 7 pre-existing failures listed in p0-baseline.md, +3 tests, no new failure |
| Lint | `cargo clippy --locked --all-targets -- -D warnings` | pre-existing findings only; none in a file or region this phase touched |
| Format | `cargo fmt -- --check` | the repository is not fmt-clean at baseline; no diff falls inside the code this phase added |
| Build | `cargo build --locked --bins` | success |
| MIPS build / export / emulator | — | **not run** |

`tests/runtime/test_actor_tables.cpp` compiles and executes the real
`runtime/actor_tables.hpp`, `object_model.hpp`, `lifecycle.hpp` and `scene_service.hpp`
against a hand-written cooked table in the exact shape `src/project.rs` emits, including
the new `SceneReferenceRecord` rows and the `scene_reference_bind` writer. It asserts:

- the actor's own `begin_play` sees `speed == 2.5`, its `Tracker` component and its bound
  legacy slot, and the component's `begin_play` sees its own override;
- the scene script reads the resolved `ActorRef` and `EntityRef` in its `begin_play`;
- `dispatch_slot_trigger` reaches the root and the tracker but **not** the
  `LegacyBehaviourComponent` whose `Behaviour` is in `bindings`; with the binding removed
  the wrapper becomes the only delivery path; an unbound and a dead slot reach nobody;
- `scene_tick` installs both hooks, and `audio_source_retained` follows
  `music_active`/`music_requested`/`music_lookup`.

**`runtime/main.cpp` remains syntax-reviewed only.** It needs PsyQo, the MIPS toolchain and
a real generated `scene.hh`, so its four additions this phase —
`epok::install_actor_service_hooks()` in `GameScene::start()`, the two
`epok::dispatch_slot_trigger(...)` calls in `dispatch_trigger`, and
`epok::collect_object_quarantine()` after `audio_tick()` — have **not** been compiled on
this machine. Every symbol and signature they use is exercised by the host test, but the
translation unit itself is unverified here.

## 7. What remains unverified

The first machine with the PSX SDK must run, in this order:

```
git submodule update --init --depth 1 third_party/nugget
cargo build --locked --bins
cargo run --locked -- --project examples/sample-game --build-psx
python tests/integration/verify_selected_scene_build.py
python tools/profile_runtime.py
python tests/runtime/verify_blueprint_runtime.py     # now also runs the MIPS syntax step
```

and confirm:

1. `main.cpp` compiles and links for MIPS with the four additions above, and the
   `#define EPOK_OBJECT_REGISTRY_CAPACITY` / `#include "actor_tables.hpp"` preamble still
   agrees with its real include order.
2. `install_actor_service_hooks()` is visible from `main.cpp` at the point it is called
   (it is defined in `scene_service.hpp` under `EPOK_ACTOR_TABLES`, which the generated
   bank header defines by including `actor_tables.hpp` first).
3. A map with a real Blueprint scene script and a typed-in `ActorRef` binds it: the
   generated `scene_reference_bind` compiles against the generated class's member, and the
   script observes a live `ObjectId` in `begin_play` on hardware.
4. The registry and the object pools stay inside the image budget with the prepare step's
   extra ordering (no size change is expected; `SceneLevel` grew five pointers/sizes and a
   64-byte flag array).
5. A scene transition in the emulator keeps the frame budget, and
   `epok::collect_object_quarantine()` per frame costs what the host says it does.

## 8. Open issues

- **Inherited-property map-scoped references.** `map_scene_references` walks the embedded
  asset's `variables` only. A reference typed into an override of an *inherited* Blueprint
  property is resolved by `compile` but not cooked; it would need the property-id → member
  name mapping the compiler builds.
- **`Compilation::scene_references` is still `#[allow(dead_code)]`.** The cook re-resolves
  from the map document instead of consuming that field. Collapsing the two would mean
  threading the `Compilation` through `scripts::catalog`, which every caller of
  `catalog()` would feel.
- **Component-family map-scoped references are untested end to end.** The record shape and
  the index resolution exist and are covered by the cook test; no fixture spawns a
  `ComponentRef` through the host runtime.
- **`dispatch_slot_trigger` is linear in the level's actors.** 64 actors × two slots per
  trigger event is acceptable at the current bounds; a slot → actor index would remove it
  if trigger density grows.
- **2D triggers.** `runtime/world2d.hpp` still does not call `dispatch_trigger`;
  `SceneComponent2D` carries no legacy slot, so the resolution needs a different key than
  `actor_for_slot`.
