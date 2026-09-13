# P2/P3 — Object runtime, components and spaces (2026-09-13)

Scope of this phase: `runtime/object_model.hpp` (single header, staging lists unchanged),
`tests/runtime/object_model.cpp` and its registration in `tests/runtime/verify_spatial.py`.
No file under `src/` was touched; nothing was committed.

## 1. What exists now

| Area | Contract as implemented |
| --- | --- |
| Identity | `ObjectId { uint16 index, uint16 generation }`, `0xffff`/`0` is null. Every reflected class carries `static constexpr uint64_t static_class_id` and a virtual `class_id()`. |
| Class table | `ClassDescriptor` + `extern const ClassDescriptor object_classes[]` / `object_class_count` (cooked later; host tests provide their own table), `find_object_class`, `object_class_is_a` with a depth bound of `object_class_count`. |
| Storage | `ObjectPool<T, N>`: static aligned bytes, `used[]`, `handles[]`, placement-new of the concrete `T`, `release` runs the virtual destructor. `storage_bytes` mirrors `bp::TypedPool`. |
| Slot table | `ObjectRegistry` (view over `ObjectSlot[]`) + `ObjectRegistryStorage<Capacity = 256>`. `acquire`/`adopt`/`release`/`get`/`resolve<T>`/`class_of`, generation bump on release, `ObjectStats { alive, peak, rejected, spawned, deferred }`. |
| Level | Bounded actor table (`level_actor_capacity = 64`), scene script slot, deferral queue (`level_pending_capacity = 16`), `spawn_batch`, `spawn_actor`, `create_scene_script`, `destroy_actor`, `end_play_all`, `tick`, `frame_update`, `set_active`, `add_component<T>/get_component<T>/get_components<T>/remove_component`. |
| World | Placeholder context holding `Level*` and `ObjectRegistry*`. Wiring into `main.cpp` and the scene banks belongs to a later phase. |

Declaration order in the header differs from the P2 skeleton: the component classes are
declared before `Actor3D`/`Actor2D`/`UIActor` because those actors embed their root
component by value, which requires a complete type. Every `EPOK_CLASS(...)` annotation,
class name, base and `Id` is byte-identical to the skeleton and to design.md section 2.

## 2. Compact class identity — decision

`static_class_id` is computed **in the header** by a `constexpr` single-block SHA-256
(`epok::detail::compact_class_id`), taking the first eight digest bytes little endian of the
UUID string. That is exactly `crate::blueprint_refs::compact_id`, so the C++ constant and
the value the Rust cook emits are derived from the same UUID by the same algorithm and
cannot drift; no generated table of magic numbers is needed and no runtime code is emitted
(the UUID string never reaches the image). A 36-character UUID plus padding fits one
64-byte block, so the compile-time cost is 15 single-block hashes for the native bases.
`tests/runtime/object_model.cpp::compact_identities` pins all fifteen values against
constants computed independently with Python's `hashlib`, so a mistake in the constexpr
reproduction fails the host test rather than silently shipping.

## 3. Lifecycle order (`Level::spawn_batch`, design.md section 6)

1. **Reserve** every actor of the batch (`ObjectRegistry::acquire`, pool-backed).
2. **Defaults**: level id, name, active flag, logical parent, then the declarative root
   component. `Actor3D`/`Actor2D`/`UIActor` declare `EPOK_COMPONENT(Root, Name="Root")`
   fields (`SceneComponent3D` / `SceneComponent2D` / `RectTransformComponent`); the root's
   storage lives inside the actor, so it is registered with `ObjectRegistry::adopt`
   (`owns_storage = false`, release never touches the memory). The non-reflected hook
   `Actor::default_root()` is what the cook will override for Blueprint classes.
3. **Resolve references** — cook-generated, no native work yet.
4. **Initialize/register**: state `Initialized`, actor appended to the level table.
5. **begin_play**: components in declaration order, then the owning actor, in batch order;
   `on_enable` follows for an active actor (components, then actor).
6. **Scene script** `begin_play` after the level actors of the batch.
7. **tick**: for each registered actor (`wants_tick`, active) components then the actor, in
   table order; the scene script ticks last. `frame_update` follows the same order and runs
   even while simulation is paused, for legacy `Behaviour` parity.
8. **Exit** (`end_play_all`): scene script first while the actors are still alive, then each
   actor in reverse table order — `on_disable` (components, then actor), `end_play`
   (components, then actor), then storage release.

`end_play` runs exactly once per object with its `EndPlayReason`; `m_begun`/`m_ended` guard
both `Actor` and `ActorComponent`. A second `destroy_actor` for the same actor returns
`false` and dispatches nothing.

## 4. Deferral and quarantine

* `ObjectDispatchScope` increments `ObjectRegistry::dispatch` around every callback batch.
* `ObjectRegistry::release` invalidates the handle immediately (state `Destroyed`,
  generation bumped) but keeps the storage while `dispatch > 0`; `finish_release()` returns
  it to the pool when the last scope unwinds. Destroying yourself inside `tick` therefore
  never overwrites your own stack frame — the same discipline as `bp::DispatchScope`.
* `destroy_actor` inside a callback marks the actor doomed at once (it receives no further
  `tick`, `frame_update`, `begin_play`, `on_enable`/`on_disable`) and queues the teardown;
  `stats.deferred` counts it. `spawn_batch`/`spawn_actor` inside a callback queue the
  request and return an invalid id, so the new actor never receives the events of the batch
  it was born in.
* The queue is drained by `flush_pending()` at the end of the top-level operation, in at
  most 8 rounds, so a deferral chain terminates.
* A batch that fails at any step releases every reservation it made (including roots that
  were already adopted), leaving no orphan components; `stats.rejected` counts the failure.

## 5. Spaces, attachment and the legacy bridge

* `SceneComponent3D::bind_slot(Entity&)` points `transform` at the canonical
  `LegacyEntityStorage` slot so collision, rendering and motion interpolation keep reading
  the same memory; `bind_local()` uses component-owned storage (the default installed by
  the defaults step). `RectTransformComponent` does the same for `RectTransform`.
  `SceneComponent2D` owns its `Transform2D` outright: no legacy slot carries one, and no
  fictitious 3D transform is created for 2D or UI actors — `Actor::entity()` returns
  `nullptr` for them.
* `attach_component(child, parent)` attaches spatial components of the *same* domain,
  rejecting self-attachment, cross-domain attachment and cycles (walk bounded by
  `object_hierarchy_depth = 16`). Logical parenting between actors
  (`ActorSpawnRequest::logical_parent`) only affects activation, never matrices.
* `set_active` folds the logical parent chain (`Level::actor_active`) and fires
  `on_enable`/`on_disable` on the components and then the actor of every actor whose
  effective activity changed — idempotent when nothing changes.
* Component rules enforced by `add_component<T>`: owner domain must be in the class's
  `owners_mask`, `Cardinality=Single` rejects a duplicate, the actor component table holds
  `actor_component_capacity = 8` entries, abstract classes are never instantiated. Removing
  the root of a spatial actor is rejected. Every rejection returns `nullptr`/`false` and
  increments `ObjectStats::rejected`.
* `LegacyBehaviourComponent` forwards `start/update/frame_update/on_enable/on_disable/
  on_destroy` to a bound `Behaviour*` with the entity's `Transform&`. `start()` runs once,
  on the first event after `bind()`, and always before `update()`, so binding after
  `add_component` never loses the event. The bound entity must **not** also appear in the
  scene bank's `Binding` table: the actor events are then the only source of Behaviour
  events, so nothing is delivered twice.

## 6. Measured sizes (host, clang++ x86-64, 64-bit pointers)

Printed by `tests/runtime/object_model.cpp::size_report`. The MIPS image uses 32-bit
pointers and vtable pointers, so these are upper bounds; the real budget must be captured
on a machine with the SDK.

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
`ObjectPool<AudioComponent,4>` 388 B (`storage_bytes` = values + handles + used flags).

## 7. Validation

| Check | Result |
| --- | --- |
| `tests/runtime/verify_spatial.py` (now includes `object_model.cpp`) | pass |
| `tests/runtime/verify_sprites_particles.py` | pass |
| `tests/runtime/verify_blueprint_runtime.py` | pass (MIPS syntax step skipped: no SDK) |
| `cargo build --locked --bins` (build.rs compiles `native/effect_preview.cpp`, which includes this header) | pass, forced rebuild of the native TUs |
| `clang++ -std=c++20 -fsyntax-only -Wall -Wextra -fno-rtti -fno-exceptions -Wno-unused-parameter` on the header and on the test | clean (the only warning in the tree is the pre-existing `epok.hpp:238` unused parameter) |
| MIPS build / export / emulator | **not run** — no `mipsel-none-elf-g++` on this machine (see p0-baseline) |

Host test cases in `tests/runtime/object_model.cpp`:

| Case | Covers |
| --- | --- |
| `compact_identities` | all 15 native `static_class_id` values against independently computed SHA-256 constants |
| `identity_and_storage` | concrete construction/destruction without slicing (virtual `class_id()` through `Object*`, concrete destructor observed), `is_a` across three levels, handle invalidated on release, slot reuse with a bumped generation |
| `lifecycle_order` | begin_play order (components before actor, scene script after the level actors), tick order, exit order with the scene script first |
| `end_play_once` | `end_play` exactly once per object, with its reason, after a repeated destroy request |
| `deferred_mutations` | destroy requested inside `tick` (victim never ticks, teardown after the loop), spawn inside `tick` (no events from the current batch), destroy of self inside `begin_play` (no `on_enable`, teardown after the batch) |
| `capacity_and_rollback` | pool exhaustion counted in `rejected` with no crash; a batch failing at the defaults step leaves no orphan actors or components |
| `component_rules` | roots registered per domain, no 3D transform for 2D/UI actors, `AudioComponent` accepted on all three domains and `Multiple`, `Single` duplicate rejected, UI component on `Actor3D` rejected, root removal rejected, get/get_components/remove, audio stopped on teardown |
| `activation_propagation` | `set_active` propagation to components and logically parented actors, idempotence, inactive actors do not tick |
| `attachment_rules` | attach, cycle rejected, self-attach rejected, cross-domain rejected, detach |
| `legacy_adapter` | each Behaviour event forwarded exactly once through the actor events, `bind_slot` making `Actor::entity()` work |
| `size_report` | the size table above |

## 8. Not done in this phase

* No MIPS compile, export or emulator run (no SDK on this machine); the PSX budget for the
  new pools and `Level` is therefore still unmeasured.
* `object_classes[]`/`object_class_count` are not emitted yet — the Rust cook owns that, as
  it owns `Actor::default_root()` overrides and `EPOK_COMPONENT` extraction for Blueprint
  classes. Host tests supply their own table.
* `Level`/`World` are not wired into `main.cpp`, the scene banks, scene transitions or the
  editor preview; `active_object_registry` is the temporary process-wide handle used by
  `Actor::entity()`, `ActorComponent::get_owner()` and `attach_component()` until World
  ownership lands.
* Component adapters reserved in design.md section 2 (`Mesh3DComponent`, `Sprite3DComponent`,
  `Camera3DComponent`, …) are not introduced yet.
* Persistent reference resolution (step 3 of the spawn order) is a documented no-op here.
