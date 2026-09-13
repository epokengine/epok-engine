# Epok API: Object Model

> **Header:** `"object_model.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/object_model.hpp)

This module covers the object model module. It documents 140 public callables declared directly in this header.

## Declared types

`epok::Actor`, `epok::Actor2D`, `epok::Actor3D`, `epok::ActorComponent`, `epok::ActorSpawnRequest`, `epok::AudioComponent`, `epok::ClassDescriptor`, `epok::EndPlayReason`, `epok::LegacyBehaviourComponent`, `epok::Level`, `epok::LevelPendingOp`, `epok::Object`, `epok::ObjectClassFlags`, `epok::ObjectDispatchScope`, `epok::ObjectDomain`, `epok::ObjectFamily`, `epok::ObjectId`, `epok::ObjectPool`, `epok::ObjectRegistry`, `epok::ObjectRegistryStorage`, `epok::ObjectSlot`, `epok::ObjectState`, `epok::ObjectStats`, `epok::RectTransformComponent`, `epok::SceneComponent2D`, `epok::SceneComponent3D`, `epok::SceneScriptActor`, `epok::Transform2D`, `epok::UIActor`, `epok::UIComponent`, `epok::World`

## Callable index

- [`epok::Actor2D::class_id`](#epok-actor2d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor2D::default_root`](#epok-actor2d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor3D::class_id`](#epok-actor3d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor3D::default_root`](#epok-actor3d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor::active`](#epok-actor-active-1) — Self flag only; Level::actor_active() folds the logical parent chain.
- [`epok::Actor::begin_play`](#epok-actor-begin-play-1) — Begins play as part of the object model module.
- [`epok::Actor::class_id`](#epok-actor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor::component_count`](#epok-actor-component-count-1) — Performs `component count` as part of the object model module.
- [`epok::Actor::component_id`](#epok-actor-component-id-1) — Performs `component id` as part of the object model module.
- [`epok::Actor::default_root`](#epok-actor-default-root-1) — Runtime hook, not reflected: the declarative root component embedded in the actor.
- [`epok::Actor::end_play`](#epok-actor-end-play-1) — Ends play as part of the object model module.
- [`epok::Actor::entity`](#epok-actor-entity-1) — Legacy adapter: the canonical slot behind the root scene component, or nullptr.
- [`epok::Actor::level_id`](#epok-actor-level-id-1) — Performs `level id` as part of the object model module.
- [`epok::Actor::logical_parent`](#epok-actor-logical-parent-1) — Performs `logical parent` as part of the object model module.
- [`epok::Actor::name`](#epok-actor-name-1) — Performs `name` as part of the object model module.
- [`epok::Actor::on_disable`](#epok-actor-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::Actor::on_enable`](#epok-actor-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::Actor::root_id`](#epok-actor-root-id-1) — Performs `root id` as part of the object model module.
- [`epok::Actor::set_name`](#epok-actor-set-name-1) — Sets name as part of the object model module.
- [`epok::Actor::set_wants_tick`](#epok-actor-set-wants-tick-1) — Sets wants tick as part of the object model module.
- [`epok::Actor::tick`](#epok-actor-tick-1) — Performs `tick` as part of the object model module.
- [`epok::Actor::wants_tick`](#epok-actor-wants-tick-1) — Performs `wants tick` as part of the object model module.
- [`epok::ActorComponent::attach_slot`](#epok-actorcomponent-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::ActorComponent::begin_play`](#epok-actorcomponent-begin-play-1) — Begins play as part of the object model module.
- [`epok::ActorComponent::class_id`](#epok-actorcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ActorComponent::end_play`](#epok-actorcomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::ActorComponent::frame_update`](#epok-actorcomponent-frame-update-1) — Runtime hooks, not reflected.
- [`epok::ActorComponent::get_owner`](#epok-actorcomponent-get-owner-1) — Returns owner as part of the object model module.
- [`epok::ActorComponent::name`](#epok-actorcomponent-name-1) — Performs `name` as part of the object model module.
- [`epok::ActorComponent::on_disable`](#epok-actorcomponent-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::ActorComponent::on_enable`](#epok-actorcomponent-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::ActorComponent::on_trigger`](#epok-actorcomponent-on-trigger-1) — Forward-only collision hook.
- [`epok::ActorComponent::owner_id`](#epok-actorcomponent-owner-id-1) — Performs `owner id` as part of the object model module.
- [`epok::ActorComponent::releasable`](#epok-actorcomponent-releasable-1) — False while a service still holds this component's storage; the registry then keeps the (already dead) slot quarantined instead of returning it to the pool.
- [`epok::ActorComponent::set_name`](#epok-actorcomponent-set-name-1) — Sets name as part of the object model module.
- [`epok::ActorComponent::tick`](#epok-actorcomponent-tick-1) — Performs `tick` as part of the object model module.
- [`epok::attach_component`](#epok-attach-component-1) — Spatial attachment between components of the same domain.
- [`epok::AudioComponent::begin_play`](#epok-audiocomponent-begin-play-1) — play_on_start policy.
- [`epok::AudioComponent::bind_local`](#epok-audiocomponent-bind-local-1) — Performs `bind local` as part of the object model module.
- [`epok::AudioComponent::bind_slot`](#epok-audiocomponent-bind-slot-1) — Bind to the legacy slot's AudioSource, or to component-owned storage.
- [`epok::AudioComponent::class_id`](#epok-audiocomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::AudioComponent::end_play`](#epok-audiocomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::AudioComponent::entity_slot`](#epok-audiocomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::AudioComponent::is_playing`](#epok-audiocomponent-is-playing-1) — Reports whether playing as part of the object model module.
- [`epok::AudioComponent::on_disable`](#epok-audiocomponent-on-disable-1) — Deactivation and teardown stop both kinds: lifecycle.hpp::set_active stops a legacy slot's audio through the slot table, and stopping an already stopped source is a no-op, so the two paths are idempotent rather than conflicting.
- [`epok::AudioComponent::owns_source`](#epok-audiocomponent-owns-source-1) — True only for component-owned storage.
- [`epok::AudioComponent::play`](#epok-audiocomponent-play-1) — Starts play as part of the object model module.
- [`epok::AudioComponent::releasable`](#epok-audiocomponent-releasable-1) — Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring create_entity's legacy-slot rule.
- [`epok::AudioComponent::stop`](#epok-audiocomponent-stop-1) — Stops stop as part of the object model module.
- [`epok::detail::compact_class_id`](#epok-detail-compact-class-id-1) — Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).
- [`epok::detail::sha256_rotr`](#epok-detail-sha256-rotr-1) — Performs `sha256 rotr` as part of the object model module.
- [`epok::dispatch_trigger`](#epok-dispatch-trigger-1) — Collision is owned by the spatial services, not by the Level: they resolve the legacy slot to its actor and call this, which fans the event out to the owner's components in registration order.
- [`epok::find_object_class`](#epok-find-object-class-1) — Finds object class as part of the object model module.
- [`epok::LegacyBehaviourComponent::begin_play`](#epok-legacybehaviourcomponent-begin-play-1) — Begins play as part of the object model module.
- [`epok::LegacyBehaviourComponent::bind`](#epok-legacybehaviourcomponent-bind-1) — Resolve the slot from the owner's root scene component (Actor3D only).
- [`epok::LegacyBehaviourComponent::bind`](#epok-legacybehaviourcomponent-bind-2) — The bound entity must NOT also appear in the scene bank's Binding table: the actor events below are the only source of start/update/frame_update/enable/disable/destroy for this Behaviour, so a migrated entity never receives an event twice.
- [`epok::LegacyBehaviourComponent::class_id`](#epok-legacybehaviourcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::LegacyBehaviourComponent::end_play`](#epok-legacybehaviourcomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::LegacyBehaviourComponent::ensure_started`](#epok-legacybehaviourcomponent-ensure-started-1) — start() runs once, on the first event after the Behaviour is bound, and always before update(); binding after add_component therefore never loses the event.
- [`epok::LegacyBehaviourComponent::entity_slot`](#epok-legacybehaviourcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::LegacyBehaviourComponent::frame_update`](#epok-legacybehaviourcomponent-frame-update-1) — Legacy contract, preserved: frame_update runs once per rendered frame even while the simulation clock is paused, and tick()/update() does not.
- [`epok::LegacyBehaviourComponent::on_disable`](#epok-legacybehaviourcomponent-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::LegacyBehaviourComponent::on_enable`](#epok-legacybehaviourcomponent-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::LegacyBehaviourComponent::on_trigger`](#epok-legacybehaviourcomponent-on-trigger-1) — Forwarded by dispatch_trigger(); the object model never generates the event.
- [`epok::LegacyBehaviourComponent::tick`](#epok-legacybehaviourcomponent-tick-1) — Performs `tick` as part of the object model module.
- [`epok::Level::actor_active`](#epok-level-actor-active-1) — ---- activation ----------------------------------------------------------------
- [`epok::Level::actor_at`](#epok-level-actor-at-1) — Performs `actor at` as part of the object model module.
- [`epok::Level::actor_count`](#epok-level-actor-count-1) — Performs `actor count` as part of the object model module.
- [`epok::Level::add_component`](#epok-level-add-component-1) — ---- components ----------------------------------------------------------------
- [`epok::Level::bind`](#epok-level-bind-1) — Binds the slot table and registers the Level itself.
- [`epok::Level::class_id`](#epok-level-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Level::create_scene_script`](#epok-level-create-scene-script-1) — The scene script is created by the loader; it is never part of the actor table.
- [`epok::Level::destroy_actor`](#epok-level-destroy-actor-1) — ---- destruction --------------------------------------------------------------- A request made inside a callback marks the actor immediately (it receives no further events) and runs the teardown when the current batch finishes.
- [`epok::Level::end_play_all`](#epok-level-end-play-all-1) — Exit order: scene script first while the actors are still alive, then every actor.
- [`epok::Level::frame_update`](#epok-level-frame-update-1) — Once per rendered frame, even while paused (legacy Behaviour parity).
- [`epok::Level::get_component`](#epok-level-get-component-1) — Returns component as part of the object model module.
- [`epok::Level::get_components`](#epok-level-get-components-1) — Returns components as part of the object model module.
- [`epok::Level::registry`](#epok-level-registry-1) — Performs `registry` as part of the object model module.
- [`epok::Level::remove_component`](#epok-level-remove-component-1) — Removes component as part of the object model module.
- [`epok::Level::scene_script`](#epok-level-scene-script-1) — Performs `scene script` as part of the object model module.
- [`epok::Level::set_active`](#epok-level-set-active-1) — Sets active as part of the object model module.
- [`epok::Level::spawn_actor`](#epok-level-spawn-actor-1) — Performs `spawn actor` as part of the object model module.
- [`epok::Level::spawn_batch`](#epok-level-spawn-batch-1) — ---- spawning ------------------------------------------------------------------ All or nothing.
- [`epok::Level::stats`](#epok-level-stats-1) — Performs `stats` as part of the object model module.
- [`epok::Level::tick`](#epok-level-tick-1) — ---- ticking -------------------------------------------------------------------
- [`epok::Object::class_id`](#epok-object-class-id-1) — Runtime-owned identity hooks; not an authoring API.
- [`epok::Object::id`](#epok-object-id-1) — Performs `id` as part of the object model module.
- [`epok::Object::is_a`](#epok-object-is-a-1) — Reports whether a as part of the object model module.
- [`epok::Object::state`](#epok-object-state-1) — Performs `state` as part of the object model module.
- [`epok::Object::~Object`](#epok-object-object-1) — Releases the resources owned by `epok::Object`.
- [`epok::object_class_is_a`](#epok-object-class-is-a-1) — Ancestry walk bounded by the table size; the cook rejects cycles before emitting.
- [`epok::object_construct`](#epok-object-construct-1) — Placement-new into caller storage; destroy runs the virtual destructor of the concrete type, so a base pointer never slices.
- [`epok::object_destruct`](#epok-object-destruct-1) — Performs `object destruct` as part of the object model module.
- [`epok::object_domain_bit`](#epok-object-domain-bit-1) — Owner domain masks for components.
- [`epok::object_storage_quarantined`](#epok-object-storage-quarantined-1) — Defined after AudioComponent; `false` for every object that owns no retained storage.
- [`epok::ObjectDispatchScope::ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-1) — Constructs `epok::ObjectDispatchScope` for the object model module.
- [`epok::ObjectDispatchScope::ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-2) — Constructs `epok::ObjectDispatchScope` for the object model module.
- [`epok::ObjectDispatchScope::operator=`](#epok-objectdispatchscope-operator-1) — Performs `operator =` as part of the object model module.
- [`epok::ObjectDispatchScope::~ObjectDispatchScope`](#epok-objectdispatchscope-objectdispatchscope-3) — Releases the resources owned by `epok::ObjectDispatchScope`.
- [`epok::ObjectId::operator!=`](#epok-objectid-operator-1) — Performs `operator !=` as part of the object model module.
- [`epok::ObjectId::operator==`](#epok-objectid-operator-2) — Performs `operator ==` as part of the object model module.
- [`epok::ObjectId::valid`](#epok-objectid-valid-1) — Performs `valid` as part of the object model module.
- [`epok::ObjectPool::acquire`](#epok-objectpool-acquire-1) — Performs `acquire` as part of the object model module.
- [`epok::ObjectPool::live`](#epok-objectpool-live-1) — Performs `live` as part of the object model module.
- [`epok::ObjectPool::release`](#epok-objectpool-release-1) — Performs `release` as part of the object model module.
- [`epok::ObjectRegistry::acquire`](#epok-objectregistry-acquire-1) — Pool-backed construction of a concrete class.
- [`epok::ObjectRegistry::adopt`](#epok-objectregistry-adopt-1) — Adopt storage owned by somebody else (an actor's embedded default component, the Level itself).
- [`epok::ObjectRegistry::class_of`](#epok-objectregistry-class-of-1) — Performs `class of` as part of the object model module.
- [`epok::ObjectRegistry::collect_quarantined`](#epok-objectregistry-collect-quarantined-1) — Retry the quarantined slots.
- [`epok::ObjectRegistry::finish_release`](#epok-objectregistry-finish-release-1) — Called when the last dispatch scope unwinds: no callback can still be on the stack of a destroyed object, so its typed storage returns to the pool.
- [`epok::ObjectRegistry::get`](#epok-objectregistry-get-1) — Returns get as part of the object model module.
- [`epok::ObjectRegistry::live`](#epok-objectregistry-live-1) — Performs `live` as part of the object model module.
- [`epok::ObjectRegistry::next_generation`](#epok-objectregistry-next-generation-1) — Performs `next generation` as part of the object model module.
- [`epok::ObjectRegistry::ObjectRegistry`](#epok-objectregistry-objectregistry-1) — Constructs `epok::ObjectRegistry` for the object model module.
- [`epok::ObjectRegistry::ObjectRegistry`](#epok-objectregistry-objectregistry-2) — Constructs `epok::ObjectRegistry` for the object model module.
- [`epok::ObjectRegistry::operator=`](#epok-objectregistry-operator-1) — Performs `operator =` as part of the object model module.
- [`epok::ObjectRegistry::release`](#epok-objectregistry-release-1) — Invalidates the handle immediately; storage is quarantined while callbacks run.
- [`epok::ObjectRegistry::resolve`](#epok-objectregistry-resolve-1) — Generation and class checked; the runtime class must derive from T's static class.
- [`epok::ObjectRegistry::resolve`](#epok-objectregistry-resolve-2) — Performs `resolve` as part of the object model module.
- [`epok::ObjectRegistry::slot`](#epok-objectregistry-slot-1) — Performs `slot` as part of the object model module.
- [`epok::ObjectRegistry::slot`](#epok-objectregistry-slot-2) — Performs `slot` as part of the object model module.
- [`epok::ObjectRegistryStorage::ObjectRegistryStorage<Capacity>`](#epok-objectregistrystorage-objectregistrystorage-capacity-1) — Constructs `epok::ObjectRegistryStorage` for the object model module.
- [`epok::RectTransformComponent::attach_slot`](#epok-recttransformcomponent-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::RectTransformComponent::bind_local`](#epok-recttransformcomponent-bind-local-1) — Performs `bind local` as part of the object model module.
- [`epok::RectTransformComponent::bind_slot`](#epok-recttransformcomponent-bind-slot-1) — Performs `bind slot` as part of the object model module.
- [`epok::RectTransformComponent::class_id`](#epok-recttransformcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::RectTransformComponent::entity_slot`](#epok-recttransformcomponent-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::SceneComponent2D::attach_slot`](#epok-scenecomponent2d-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::SceneComponent2D::class_id`](#epok-scenecomponent2d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::SceneComponent3D::attach_slot`](#epok-scenecomponent3d-attach-slot-1) — Performs `attach slot` as part of the object model module.
- [`epok::SceneComponent3D::bind_local`](#epok-scenecomponent3d-bind-local-1) — No legacy slot: the component owns the transform.
- [`epok::SceneComponent3D::bind_slot`](#epok-scenecomponent3d-bind-slot-1) — Canonical storage: the legacy entity slot owns the transform and everything that already reads it (collision, rendering, motion interpolation) keeps working.
- [`epok::SceneComponent3D::class_id`](#epok-scenecomponent3d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::SceneComponent3D::entity_slot`](#epok-scenecomponent3d-entity-slot-1) — Performs `entity slot` as part of the object model module.
- [`epok::SceneScriptActor::class_id`](#epok-scenescriptactor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::UIActor::class_id`](#epok-uiactor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::UIActor::default_root`](#epok-uiactor-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::UIComponent::class_id`](#epok-uicomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::World::bind`](#epok-world-bind-1) — Performs `bind` as part of the object model module.
- [`epok::World::class_id`](#epok-world-class-id-1) — Performs `class id` as part of the object model module.

<a id="epok-actor2d-class-id-1"></a>

## `epok::Actor2D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 604](../../../runtime/object_model.hpp#L604)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor2d-default-root-1"></a>

## `epok::Actor2D::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 606](../../../runtime/object_model.hpp#L606)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor2D& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor3d-class-id-1"></a>

## `epok::Actor3D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 597](../../../runtime/object_model.hpp#L597)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor3d-default-root-1"></a>

## `epok::Actor3D::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 599](../../../runtime/object_model.hpp#L599)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor3D& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-active-1"></a>

## `epok::Actor::active`

**Purpose.** Self flag only; Level::actor_active() folds the logical parent chain.

**Exact declaration**

```cpp
bool active() const
```

- **Declared at:** [line 402](../../../runtime/object_model.hpp#L402)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actor-begin-play-1"></a>

## `epok::Actor::begin_play`

**Purpose.** Begins play as part of the object model module.

**Exact declaration**

```cpp
virtual void begin_play()
```

- **Declared at:** [line 381](../../../runtime/object_model.hpp#L381)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-class-id-1"></a>

## `epok::Actor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 380](../../../runtime/object_model.hpp#L380)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-component-count-1"></a>

## `epok::Actor::component_count`

**Purpose.** Performs `component count` as part of the object model module.

**Exact declaration**

```cpp
size_t component_count() const
```

- **Declared at:** [line 400](../../../runtime/object_model.hpp#L400)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.component_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-component-id-1"></a>

## `epok::Actor::component_id`

**Purpose.** Performs `component id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId component_id(size_t index) const
```

- **Declared at:** [line 399](../../../runtime/object_model.hpp#L399)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::Actor& object = /* obtain a valid instance */;

auto result = object.component_id(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-default-root-1"></a>

## `epok::Actor::default_root`

**Purpose.** Runtime hook, not reflected: the declarative root component embedded in the actor.

**Exact declaration**

```cpp
virtual ActorComponent* default_root()
```

- **Declared at:** [line 387](../../../runtime/object_model.hpp#L387)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-end-play-1"></a>

## `epok::Actor::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
virtual void end_play(EndPlayReason)
```

- **Declared at:** [line 383](../../../runtime/object_model.hpp#L383)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::Actor& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-entity-1"></a>

## `epok::Actor::entity`

**Purpose.** Legacy adapter: the canonical slot behind the root scene component, or nullptr.

**Exact declaration**

```cpp
Entity* entity()
```

- **Declared at:** [line 389](../../../runtime/object_model.hpp#L389)
- **Kind:** `cxx method`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.entity();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-level-id-1"></a>

## `epok::Actor::level_id`

**Purpose.** Performs `level id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId level_id() const
```

- **Declared at:** [line 396](../../../runtime/object_model.hpp#L396)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.level_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-logical-parent-1"></a>

## `epok::Actor::logical_parent`

**Purpose.** Performs `logical parent` as part of the object model module.

**Exact declaration**

```cpp
ObjectId logical_parent() const
```

- **Declared at:** [line 398](../../../runtime/object_model.hpp#L398)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.logical_parent();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-name-1"></a>

## `epok::Actor::name`

**Purpose.** Performs `name` as part of the object model module.

**Exact declaration**

```cpp
const char* name() const
```

- **Declared at:** [line 390](../../../runtime/object_model.hpp#L390)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.name();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-on-disable-1"></a>

## `epok::Actor::on_disable`

**Purpose.** Performs `on disable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_disable()
```

- **Declared at:** [line 385](../../../runtime/object_model.hpp#L385)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-on-enable-1"></a>

## `epok::Actor::on_enable`

**Purpose.** Performs `on enable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_enable()
```

- **Declared at:** [line 384](../../../runtime/object_model.hpp#L384)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-root-id-1"></a>

## `epok::Actor::root_id`

**Purpose.** Performs `root id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId root_id() const
```

- **Declared at:** [line 397](../../../runtime/object_model.hpp#L397)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.root_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-set-name-1"></a>

## `epok::Actor::set_name`

**Purpose.** Sets name as part of the object model module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 391](../../../runtime/object_model.hpp#L391)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const char *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * value

epok::Actor& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actor-set-wants-tick-1"></a>

## `epok::Actor::set_wants_tick`

**Purpose.** Sets wants tick as part of the object model module.

**Exact declaration**

```cpp
void set_wants_tick(bool value)
```

- **Declared at:** [line 404](../../../runtime/object_model.hpp#L404)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Actor& object = /* obtain a valid instance */;

object.set_wants_tick(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-tick-1"></a>

## `epok::Actor::tick`

**Purpose.** Performs `tick` as part of the object model module.

**Exact declaration**

```cpp
virtual void tick(Fixed)
```

- **Declared at:** [line 382](../../../runtime/object_model.hpp#L382)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::Actor& object = /* obtain a valid instance */;

object.tick(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actor-wants-tick-1"></a>

## `epok::Actor::wants_tick`

**Purpose.** Performs `wants tick` as part of the object model module.

**Exact declaration**

```cpp
bool wants_tick() const
```

- **Declared at:** [line 403](../../../runtime/object_model.hpp#L403)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.wants_tick();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actorcomponent-attach-slot-1"></a>

## `epok::ActorComponent::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
virtual ObjectId* attach_slot()
```

- **Declared at:** [line 428](../../../runtime/object_model.hpp#L428)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-begin-play-1"></a>

## `epok::ActorComponent::begin_play`

**Purpose.** Begins play as part of the object model module.

**Exact declaration**

```cpp
virtual void begin_play()
```

- **Declared at:** [line 420](../../../runtime/object_model.hpp#L420)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-class-id-1"></a>

## `epok::ActorComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 419](../../../runtime/object_model.hpp#L419)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-end-play-1"></a>

## `epok::ActorComponent::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
virtual void end_play(EndPlayReason)
```

- **Declared at:** [line 422](../../../runtime/object_model.hpp#L422)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::ActorComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-frame-update-1"></a>

## `epok::ActorComponent::frame_update`

**Purpose.** Runtime hooks, not reflected.

**Details.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Exact declaration**

```cpp
virtual void frame_update(uint32_t)
```

- **Declared at:** [line 427](../../../runtime/object_model.hpp#L427)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint32_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t arg1

epok::ActorComponent& object = /* obtain a valid instance */;

object.frame_update(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-get-owner-1"></a>

## `epok::ActorComponent::get_owner`

**Purpose.** Returns owner as part of the object model module.

**Exact declaration**

```cpp
Actor* get_owner()
```

- **Declared at:** [line 437](../../../runtime/object_model.hpp#L437)
- **Kind:** `cxx method`

**Returns.** Returns `Actor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.get_owner();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-name-1"></a>

## `epok::ActorComponent::name`

**Purpose.** Performs `name` as part of the object model module.

**Exact declaration**

```cpp
const char* name() const
```

- **Declared at:** [line 438](../../../runtime/object_model.hpp#L438)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.name();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-disable-1"></a>

## `epok::ActorComponent::on_disable`

**Purpose.** Performs `on disable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_disable()
```

- **Declared at:** [line 424](../../../runtime/object_model.hpp#L424)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-enable-1"></a>

## `epok::ActorComponent::on_enable`

**Purpose.** Performs `on enable` as part of the object model module.

**Exact declaration**

```cpp
virtual void on_enable()
```

- **Declared at:** [line 423](../../../runtime/object_model.hpp#L423)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-on-trigger-1"></a>

## `epok::ActorComponent::on_trigger`

**Purpose.** Forward-only collision hook.

**Details.** The Level does not own collision; the collision service calls dispatch_trigger(Level&, ...) which fans the event out to the owner's components. Nothing in the object model generates trigger events.

**Exact declaration**

```cpp
virtual void on_trigger(EntityHandle, TriggerPhase)
```

- **Declared at:** [line 432](../../../runtime/object_model.hpp#L432)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EntityHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `TriggerPhase` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The Level does not own collision; the collision service calls dispatch_trigger(Level&, ...) which fans the event out to the owner's components. Nothing in the object model generates trigger events.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle arg1
// TriggerPhase arg2

epok::ActorComponent& object = /* obtain a valid instance */;

object.on_trigger(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-owner-id-1"></a>

## `epok::ActorComponent::owner_id`

**Purpose.** Performs `owner id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId owner_id() const
```

- **Declared at:** [line 436](../../../runtime/object_model.hpp#L436)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.owner_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-releasable-1"></a>

## `epok::ActorComponent::releasable`

**Purpose.** False while a service still holds this component's storage; the registry then keeps the (already dead) slot quarantined instead of returning it to the pool.

**Exact declaration**

```cpp
virtual bool releasable() const
```

- **Declared at:** [line 435](../../../runtime/object_model.hpp#L435)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

auto result = object.releasable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-actorcomponent-set-name-1"></a>

## `epok::ActorComponent::set_name`

**Purpose.** Sets name as part of the object model module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 439](../../../runtime/object_model.hpp#L439)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const char *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * value

epok::ActorComponent& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actorcomponent-tick-1"></a>

## `epok::ActorComponent::tick`

**Purpose.** Performs `tick` as part of the object model module.

**Exact declaration**

```cpp
virtual void tick(Fixed)
```

- **Declared at:** [line 421](../../../runtime/object_model.hpp#L421)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::ActorComponent& object = /* obtain a valid instance */;

object.tick(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-attach-component-1"></a>

## `epok::attach_component`

**Purpose.** Spatial attachment between components of the same domain.

**Details.** Logical actor parenting never inherits matrices; only this attachment does.

**Exact declaration**

```cpp
inline bool attach_component(ObjectId child, ObjectId parent)
```

- **Declared at:** [line 1120](../../../runtime/object_model.hpp#L1120)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `ObjectId` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Logical actor parenting never inherits matrices; only this attachment does.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId child
// ObjectId parent

auto result = epok::attach_component(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-audiocomponent-begin-play-1"></a>

## `epok::AudioComponent::begin_play`

**Purpose.** play_on_start policy.

**Details.** Exactly one path starts a given AudioSource: * slot-backed: the scene bank's bank-load loop and bp::activate_spawn_audio (runtime/lifecycle.hpp) start it. begin_play here must NOT play, or a migrated entity would be heard twice. * component-owned: no legacy path knows about `local`, so begin_play starts it when play_on_start is set, the source is enabled and the owner is active.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 522](../../../runtime/object_model.hpp#L522)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Exactly one path starts a given AudioSource: * slot-backed: the scene bank's bank-load loop and bp::activate_spawn_audio (runtime/lifecycle.hpp) start it. begin_play here must NOT play, or a migrated entity would be heard twice. * component-owned: no legacy path knows about `local`, so begin_play starts it when play_on_start is set, the source is enabled and the owner is active.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-bind-local-1"></a>

## `epok::AudioComponent::bind_local`

**Purpose.** Performs `bind local` as part of the object model module.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 509](../../../runtime/object_model.hpp#L509)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-bind-slot-1"></a>

## `epok::AudioComponent::bind_slot`

**Purpose.** Bind to the legacy slot's AudioSource, or to component-owned storage.

**Exact declaration**

```cpp
void bind_slot(Entity& value)
```

- **Declared at:** [line 508](../../../runtime/object_model.hpp#L508)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Entity &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Entity & value

epok::AudioComponent& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-audiocomponent-class-id-1"></a>

## `epok::AudioComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 502](../../../runtime/object_model.hpp#L502)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-end-play-1"></a>

## `epok::AudioComponent::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 531](../../../runtime/object_model.hpp#L531)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::AudioComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-entity-slot-1"></a>

## `epok::AudioComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
Entity* entity_slot() const
```

- **Declared at:** [line 510](../../../runtime/object_model.hpp#L510)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-is-playing-1"></a>

## `epok::AudioComponent::is_playing`

**Purpose.** Reports whether playing as part of the object model module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 506](../../../runtime/object_model.hpp#L506)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-on-disable-1"></a>

## `epok::AudioComponent::on_disable`

**Purpose.** Deactivation and teardown stop both kinds: lifecycle.hpp::set_active stops a legacy slot's audio through the slot table, and stopping an already stopped source is a no-op, so the two paths are idempotent rather than conflicting.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 530](../../../runtime/object_model.hpp#L530)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-owns-source-1"></a>

## `epok::AudioComponent::owns_source`

**Purpose.** True only for component-owned storage.

**Details.** A slot-backed component is a *view* over a legacy AudioSource that the scene bank and lifecycle.hpp already drive.

**Exact declaration**

```cpp
bool owns_source() const
```

- **Declared at:** [line 513](../../../runtime/object_model.hpp#L513)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A slot-backed component is a *view* over a legacy AudioSource that the scene bank and lifecycle.hpp already drive.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.owns_source();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-play-1"></a>

## `epok::AudioComponent::play`

**Purpose.** Starts play as part of the object model module.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 504](../../../runtime/object_model.hpp#L504)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-releasable-1"></a>

## `epok::AudioComponent::releasable`

**Purpose.** Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring create_entity's legacy-slot rule.

**Details.** Slot storage belongs to the scene bank, so this component never holds it back.

**Exact declaration**

```cpp
bool releasable() const override
```

- **Declared at:** [line 535](../../../runtime/object_model.hpp#L535)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Slot storage belongs to the scene bank, so this component never holds it back.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

auto result = object.releasable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiocomponent-stop-1"></a>

## `epok::AudioComponent::stop`

**Purpose.** Stops stop as part of the object model module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 505](../../../runtime/object_model.hpp#L505)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::AudioComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-detail-compact-class-id-1"></a>

## `epok::detail::compact_class_id`

**Purpose.** Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).

**Exact declaration**

```cpp
constexpr uint64_t compact_class_id(const char* text)
```

- **Declared at:** [line 81](../../../runtime/object_model.hpp#L81)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const char * text

auto result = epok::detail::compact_class_id(text);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-sha256-rotr-1"></a>

## `epok::detail::sha256_rotr`

**Purpose.** Performs `sha256 rotr` as part of the object model module.

**Exact declaration**

```cpp
constexpr uint32_t sha256_rotr(uint32_t value, unsigned bits)
```

- **Declared at:** [line 77](../../../runtime/object_model.hpp#L77)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `bits` | `unsigned int` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value
// unsigned int bits

auto result = epok::detail::sha256_rotr(value, bits);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dispatch-trigger-1"></a>

## `epok::dispatch_trigger`

**Purpose.** Collision is owned by the spatial services, not by the Level: they resolve the legacy slot to its actor and call this, which fans the event out to the owner's components in registration order.

**Details.** Inactive, unstarted and doomed actors receive nothing.

**Exact declaration**

```cpp
inline size_t dispatch_trigger(Level& level, ObjectId actor, EntityHandle other, TriggerPhase phase)
```

- **Declared at:** [line 1097](../../../runtime/object_model.hpp#L1097)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `level` | `Level &` | Input/output; inspect the function contract | Value supplied for `level`. See the exact type and module contract. |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `other` | `EntityHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** Inactive, unstarted and doomed actors receive nothing.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Level & level
// ObjectId actor
// EntityHandle other
// TriggerPhase phase

auto result = epok::dispatch_trigger(level, actor, other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-find-object-class-1"></a>

## `epok::find_object_class`

**Purpose.** Finds object class as part of the object model module.

**Exact declaration**

```cpp
inline const ClassDescriptor* find_object_class(uint64_t id)
```

- **Declared at:** [line 142](../../../runtime/object_model.hpp#L142)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uint64_t` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassDescriptor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t id

auto result = epok::find_object_class(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-begin-play-1"></a>

## `epok::LegacyBehaviourComponent::begin_play`

**Purpose.** Begins play as part of the object model module.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 567](../../../runtime/object_model.hpp#L567)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-bind-1"></a>

## `epok::LegacyBehaviourComponent::bind`

**Purpose.** Resolve the slot from the owner's root scene component (Actor3D only).

**Exact declaration**

```cpp
bool bind(Behaviour& value)
```

- **Declared at:** [line 558](../../../runtime/object_model.hpp#L558)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Behaviour &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Behaviour & value

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

auto result = object.bind(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-legacybehaviourcomponent-bind-2"></a>

## `epok::LegacyBehaviourComponent::bind`

**Purpose.** The bound entity must NOT also appear in the scene bank's Binding table: the actor events below are the only source of start/update/frame_update/enable/disable/destroy for this Behaviour, so a migrated entity never receives an event twice.

**Exact declaration**

```cpp
void bind(Behaviour& value, Entity& owner_slot)
```

- **Declared at:** [line 552](../../../runtime/object_model.hpp#L552)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Behaviour &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |
| `owner_slot` | `Entity &` | Input/output; inspect the function contract | Value supplied for `owner_slot`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Behaviour & value
// Entity & owner_slot

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.bind(value, owner_slot);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-legacybehaviourcomponent-class-id-1"></a>

## `epok::LegacyBehaviourComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 547](../../../runtime/object_model.hpp#L547)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-end-play-1"></a>

## `epok::LegacyBehaviourComponent::end_play`

**Purpose.** Ends play as part of the object model module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 580](../../../runtime/object_model.hpp#L580)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-ensure-started-1"></a>

## `epok::LegacyBehaviourComponent::ensure_started`

**Purpose.** start() runs once, on the first event after the Behaviour is bound, and always before update(); binding after add_component therefore never loses the event.

**Exact declaration**

```cpp
bool ensure_started()
```

- **Declared at:** [line 562](../../../runtime/object_model.hpp#L562)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

auto result = object.ensure_started();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-legacybehaviourcomponent-entity-slot-1"></a>

## `epok::LegacyBehaviourComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
Entity* entity_slot() const
```

- **Declared at:** [line 559](../../../runtime/object_model.hpp#L559)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-frame-update-1"></a>

## `epok::LegacyBehaviourComponent::frame_update`

**Purpose.** Legacy contract, preserved: frame_update runs once per rendered frame even while the simulation clock is paused, and tick()/update() does not.

**Details.** main.cpp drives the two from separate places (Level::frame_update every frame, Level::tick only for the fixed steps the Time service schedules), so a pause menu keeps receiving it.

**Exact declaration**

```cpp
void frame_update(uint32_t elapsed) override
```

- **Declared at:** [line 573](../../../runtime/object_model.hpp#L573)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** main.cpp drives the two from separate places (Level::frame_update every frame, Level::tick only for the fixed steps the Time service schedules), so a pause menu keeps receiving it.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t elapsed

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.frame_update(elapsed);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-on-disable-1"></a>

## `epok::LegacyBehaviourComponent::on_disable`

**Purpose.** Performs `on disable` as part of the object model module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 579](../../../runtime/object_model.hpp#L579)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-on-enable-1"></a>

## `epok::LegacyBehaviourComponent::on_enable`

**Purpose.** Performs `on enable` as part of the object model module.

**Exact declaration**

```cpp
void on_enable() override
```

- **Declared at:** [line 578](../../../runtime/object_model.hpp#L578)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-on-trigger-1"></a>

## `epok::LegacyBehaviourComponent::on_trigger`

**Purpose.** Forwarded by dispatch_trigger(); the object model never generates the event.

**Exact declaration**

```cpp
void on_trigger(EntityHandle other, TriggerPhase phase) override
```

- **Declared at:** [line 575](../../../runtime/object_model.hpp#L575)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `EntityHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle other
// TriggerPhase phase

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.on_trigger(other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-legacybehaviourcomponent-tick-1"></a>

## `epok::LegacyBehaviourComponent::tick`

**Purpose.** Performs `tick` as part of the object model module.

**Exact declaration**

```cpp
void tick(Fixed delta) override
```

- **Declared at:** [line 568](../../../runtime/object_model.hpp#L568)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta` | `Fixed` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta

epok::LegacyBehaviourComponent& object = /* obtain a valid instance */;

object.tick(delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-actor-active-1"></a>

## `epok::Level::actor_active`

**Purpose.** ---- activation ----------------------------------------------------------------

**Exact declaration**

```cpp
bool actor_active(const Actor& actor) const
```

- **Declared at:** [line 789](../../../runtime/object_model.hpp#L789)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `const Actor &` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const Actor & actor

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_active(actor);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-actor-at-1"></a>

## `epok::Level::actor_at`

**Purpose.** Performs `actor at` as part of the object model module.

**Exact declaration**

```cpp
ObjectId actor_at(size_t index) const
```

- **Declared at:** [line 656](../../../runtime/object_model.hpp#L656)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_at(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-actor-count-1"></a>

## `epok::Level::actor_count`

**Purpose.** Performs `actor count` as part of the object model module.

**Exact declaration**

```cpp
size_t actor_count() const
```

- **Declared at:** [line 655](../../../runtime/object_model.hpp#L655)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.actor_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-add-component-1"></a>

## `epok::Level::add_component`

**Purpose.** ---- components ----------------------------------------------------------------

**Exact declaration**

```cpp
template<class T> T* add_component(Actor& owner, const char* name = nullptr)
```

- **Declared at:** [line 829](../../../runtime/object_model.hpp#L829)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// Actor & owner
// const char * name

epok::Level& object = /* obtain a valid instance */;

auto result = object.add_component<T>(owner, name);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-bind-1"></a>

## `epok::Level::bind`

**Purpose.** Binds the slot table and registers the Level itself.

**Details.** Also publishes the registry as the process-wide one used by Actor::entity()/ActorComponent::get_owner().

**Exact declaration**

```cpp
bool bind(ObjectRegistry& value)
```

- **Declared at:** [line 646](../../../runtime/object_model.hpp#L646)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Also publishes the registry as the process-wide one used by Actor::entity()/ActorComponent::get_owner().

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectRegistry & value

epok::Level& object = /* obtain a valid instance */;

auto result = object.bind(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-class-id-1"></a>

## `epok::Level::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 642](../../../runtime/object_model.hpp#L642)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-create-scene-script-1"></a>

## `epok::Level::create_scene_script`

**Purpose.** The scene script is created by the loader; it is never part of the actor table.

**Exact declaration**

```cpp
ObjectId create_scene_script(const ClassDescriptor& type, const char* name)
```

- **Declared at:** [line 720](../../../runtime/object_model.hpp#L720)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type
// const char * name

epok::Level& object = /* obtain a valid instance */;

auto result = object.create_scene_script(type, name);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-destroy-actor-1"></a>

## `epok::Level::destroy_actor`

**Purpose.** ---- destruction --------------------------------------------------------------- A request made inside a callback marks the actor immediately (it receives no further events) and runs the teardown when the current batch finishes.

**Exact declaration**

```cpp
bool destroy_actor(ObjectId actor, EndPlayReason reason = EndPlayReason::Destroyed)
```

- **Declared at:** [line 737](../../../runtime/object_model.hpp#L737)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `reason` | `EndPlayReason` | Input | Value supplied for `reason`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// EndPlayReason reason

epok::Level& object = /* obtain a valid instance */;

auto result = object.destroy_actor(actor, reason);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-end-play-all-1"></a>

## `epok::Level::end_play_all`

**Purpose.** Exit order: scene script first while the actors are still alive, then every actor.

**Exact declaration**

```cpp
void end_play_all(EndPlayReason reason)
```

- **Declared at:** [line 757](../../../runtime/object_model.hpp#L757)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reason` | `EndPlayReason` | Input | Value supplied for `reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason reason

epok::Level& object = /* obtain a valid instance */;

object.end_play_all(reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-frame-update-1"></a>

## `epok::Level::frame_update`

**Purpose.** Once per rendered frame, even while paused (legacy Behaviour parity).

**Exact declaration**

```cpp
void frame_update(uint32_t elapsed)
```

- **Declared at:** [line 778](../../../runtime/object_model.hpp#L778)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t elapsed

epok::Level& object = /* obtain a valid instance */;

object.frame_update(elapsed);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-get-component-1"></a>

## `epok::Level::get_component`

**Purpose.** Returns component as part of the object model module.

**Exact declaration**

```cpp
template<class T> T* get_component(const Actor& owner) const
```

- **Declared at:** [line 847](../../../runtime/object_model.hpp#L847)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const Actor &` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// const Actor & owner

epok::Level& object = /* obtain a valid instance */;

auto result = object.get_component<T>(owner);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-get-components-1"></a>

## `epok::Level::get_components`

**Purpose.** Returns components as part of the object model module.

**Exact declaration**

```cpp
template<class T> size_t get_components(const Actor& owner, T** out, size_t capacity) const
```

- **Declared at:** [line 854](../../../runtime/object_model.hpp#L854)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const Actor &` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `out` | `T **` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// const Actor & owner
// T ** out
// size_t capacity

epok::Level& object = /* obtain a valid instance */;

auto result = object.get_components<T>(owner, out, capacity);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-registry-1"></a>

## `epok::Level::registry`

**Purpose.** Performs `registry` as part of the object model module.

**Exact declaration**

```cpp
ObjectRegistry* registry() const
```

- **Declared at:** [line 653](../../../runtime/object_model.hpp#L653)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectRegistry *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.registry();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-remove-component-1"></a>

## `epok::Level::remove_component`

**Purpose.** Removes component as part of the object model module.

**Exact declaration**

```cpp
bool remove_component(Actor& owner, ObjectId component)
```

- **Declared at:** [line 865](../../../runtime/object_model.hpp#L865)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `component` | `ObjectId` | Input | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Actor & owner
// ObjectId component

epok::Level& object = /* obtain a valid instance */;

auto result = object.remove_component(owner, component);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-scene-script-1"></a>

## `epok::Level::scene_script`

**Purpose.** Performs `scene script` as part of the object model module.

**Exact declaration**

```cpp
ObjectId scene_script() const
```

- **Declared at:** [line 657](../../../runtime/object_model.hpp#L657)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.scene_script();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-set-active-1"></a>

## `epok::Level::set_active`

**Purpose.** Sets active as part of the object model module.

**Exact declaration**

```cpp
bool set_active(ObjectId actor, bool active)
```

- **Declared at:** [line 798](../../../runtime/object_model.hpp#L798)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// bool active

epok::Level& object = /* obtain a valid instance */;

auto result = object.set_active(actor, active);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-spawn-actor-1"></a>

## `epok::Level::spawn_actor`

**Purpose.** Performs `spawn actor` as part of the object model module.

**Exact declaration**

```cpp
ObjectId spawn_actor(const ClassDescriptor& type, const char* name, ObjectId logical_parent = {})
```

- **Declared at:** [line 711](../../../runtime/object_model.hpp#L711)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `logical_parent` | `ObjectId` | Input | Value supplied for `logical_parent`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type
// const char * name
// ObjectId logical_parent

epok::Level& object = /* obtain a valid instance */;

auto result = object.spawn_actor(type, name, logical_parent);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-spawn-batch-1"></a>

## `epok::Level::spawn_batch`

**Purpose.** ---- spawning ------------------------------------------------------------------ All or nothing.

**Details.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Exact declaration**

```cpp
size_t spawn_batch(const ActorSpawnRequest* requests, size_t count, ObjectId* out)
```

- **Declared at:** [line 662](../../../runtime/object_model.hpp#L662)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `requests` | `const ActorSpawnRequest *` | Input | Value supplied for `requests`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `out` | `ObjectId *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ActorSpawnRequest * requests
// size_t count
// ObjectId * out

epok::Level& object = /* obtain a valid instance */;

auto result = object.spawn_batch(requests, count, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-stats-1"></a>

## `epok::Level::stats`

**Purpose.** Performs `stats` as part of the object model module.

**Exact declaration**

```cpp
ObjectStats stats() const
```

- **Declared at:** [line 654](../../../runtime/object_model.hpp#L654)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectStats`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Level& object = /* obtain a valid instance */;

auto result = object.stats();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-level-tick-1"></a>

## `epok::Level::tick`

**Purpose.** ---- ticking -------------------------------------------------------------------

**Exact declaration**

```cpp
void tick(Fixed delta)
```

- **Declared at:** [line 768](../../../runtime/object_model.hpp#L768)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta` | `Fixed` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta

epok::Level& object = /* obtain a valid instance */;

object.tick(delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-class-id-1"></a>

## `epok::Object::class_id`

**Purpose.** Runtime-owned identity hooks; not an authoring API.

**Exact declaration**

```cpp
virtual uint64_t class_id() const
```

- **Declared at:** [line 164](../../../runtime/object_model.hpp#L164)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-id-1"></a>

## `epok::Object::id`

**Purpose.** Performs `id` as part of the object model module.

**Exact declaration**

```cpp
ObjectId id() const
```

- **Declared at:** [line 165](../../../runtime/object_model.hpp#L165)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-is-a-1"></a>

## `epok::Object::is_a`

**Purpose.** Reports whether a as part of the object model module.

**Exact declaration**

```cpp
bool is_a(uint64_t parent) const
```

- **Declared at:** [line 167](../../../runtime/object_model.hpp#L167)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `uint64_t` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t parent

epok::Object& object = /* obtain a valid instance */;

auto result = object.is_a(parent);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-object-state-1"></a>

## `epok::Object::state`

**Purpose.** Performs `state` as part of the object model module.

**Exact declaration**

```cpp
ObjectState state() const
```

- **Declared at:** [line 166](../../../runtime/object_model.hpp#L166)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Object& object = /* obtain a valid instance */;

auto result = object.state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-object-1"></a>

## `epok::Object::~Object`

**Purpose.** Releases the resources owned by `epok::Object`.

**Exact declaration**

```cpp
virtual ~Object() = default
```

- **Declared at:** [line 162](../../../runtime/object_model.hpp#L162)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// `epok::Object` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-class-is-a-1"></a>

## `epok::object_class_is_a`

**Purpose.** Ancestry walk bounded by the table size; the cook rejects cycles before emitting.

**Exact declaration**

```cpp
inline bool object_class_is_a(uint64_t child, uint64_t parent)
```

- **Declared at:** [line 148](../../../runtime/object_model.hpp#L148)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `uint64_t` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `uint64_t` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t child
// uint64_t parent

auto result = epok::object_class_is_a(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-object-construct-1"></a>

## `epok::object_construct`

**Purpose.** Placement-new into caller storage; destroy runs the virtual destructor of the concrete type, so a base pointer never slices.

**Exact declaration**

```cpp
template<class T> Object* object_construct(void* storage)
```

- **Declared at:** [line 177](../../../runtime/object_model.hpp#L177)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `storage` | `void *` | Input/output; inspect the function contract | Value supplied for `storage`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// void * storage

auto result = epok::object_construct<T>(storage);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-destruct-1"></a>

## `epok::object_destruct`

**Purpose.** Performs `object destruct` as part of the object model module.

**Exact declaration**

```cpp
inline void object_destruct(Object* instance)
```

- **Declared at:** [line 178](../../../runtime/object_model.hpp#L178)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

epok::object_destruct(instance);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-domain-bit-1"></a>

## `epok::object_domain_bit`

**Purpose.** Owner domain masks for components.

**Details.** None never appears in an owners mask.

**Exact declaration**

```cpp
constexpr uint8_t object_domain_bit(ObjectDomain domain)
```

- **Declared at:** [line 58](../../../runtime/object_model.hpp#L58)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `domain` | `ObjectDomain` | Input | Value supplied for `domain`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** None never appears in an owners mask.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectDomain domain

auto result = epok::object_domain_bit(domain);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-quarantined-1"></a>

## `epok::object_storage_quarantined`

**Purpose.** Defined after AudioComponent; `false` for every object that owns no retained storage.

**Exact declaration**

```cpp
inline bool object_storage_quarantined(Object* instance)
```

- **Declared at:** [line 238](../../../runtime/object_model.hpp#L238)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

auto result = epok::object_storage_quarantined(instance);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-1"></a>

## `epok::ObjectDispatchScope::ObjectDispatchScope`

**Purpose.** Constructs `epok::ObjectDispatchScope` for the object model module.

**Exact declaration**

```cpp
ObjectDispatchScope(const ObjectDispatchScope&) = delete
```

- **Declared at:** [line 365](../../../runtime/object_model.hpp#L365)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectDispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectDispatchScope & arg1

epok::ObjectDispatchScope value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-2"></a>

## `epok::ObjectDispatchScope::ObjectDispatchScope`

**Purpose.** Constructs `epok::ObjectDispatchScope` for the object model module.

**Exact declaration**

```cpp
explicit ObjectDispatchScope(ObjectRegistry& value) : r
```

- **Declared at:** [line 363](../../../runtime/object_model.hpp#L363)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectRegistry & value

epok::ObjectDispatchScope value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-operator-1"></a>

## `epok::ObjectDispatchScope::operator=`

**Purpose.** Performs `operator =` as part of the object model module.

**Exact declaration**

```cpp
ObjectDispatchScope& operator=(const ObjectDispatchScope&) = delete
```

- **Declared at:** [line 366](../../../runtime/object_model.hpp#L366)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectDispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ObjectDispatchScope &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectDispatchScope & arg1

epok::ObjectDispatchScope& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectdispatchscope-objectdispatchscope-3"></a>

## `epok::ObjectDispatchScope::~ObjectDispatchScope`

**Purpose.** Releases the resources owned by `epok::ObjectDispatchScope`.

**Exact declaration**

```cpp
~ObjectDispatchScope()
```

- **Declared at:** [line 364](../../../runtime/object_model.hpp#L364)
- **Kind:** `destructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// `epok::ObjectDispatchScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectid-operator-1"></a>

## `epok::ObjectId::operator!=`

**Purpose.** Performs `operator !=` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool operator!=(const ObjectId& other) const
```

- **Declared at:** [line 33](../../../runtime/object_model.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const ObjectId &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectId & other

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.operator!=(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectid-operator-2"></a>

## `epok::ObjectId::operator==`

**Purpose.** Performs `operator ==` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool operator==(const ObjectId& other) const
```

- **Declared at:** [line 30](../../../runtime/object_model.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const ObjectId &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectId & other

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.operator==(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectid-valid-1"></a>

## `epok::ObjectId::valid`

**Purpose.** Performs `valid` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool valid() const
```

- **Declared at:** [line 29](../../../runtime/object_model.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-objectpool-acquire-1"></a>

## `epok::ObjectPool::acquire`

**Purpose.** Performs `acquire` as part of the object model module.

**Exact declaration**

```cpp
static Object* acquire()
```

- **Declared at:** [line 189](../../../runtime/object_model.hpp#L189)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

auto result = epok::ObjectPool::acquire();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectpool-live-1"></a>

## `epok::ObjectPool::live`

**Purpose.** Performs `live` as part of the object model module.

**Exact declaration**

```cpp
static size_t live()
```

- **Declared at:** [line 205](../../../runtime/object_model.hpp#L205)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

auto result = epok::ObjectPool::live();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectpool-release-1"></a>

## `epok::ObjectPool::release`

**Purpose.** Performs `release` as part of the object model module.

**Exact declaration**

```cpp
static void release(Object* instance)
```

- **Declared at:** [line 197](../../../runtime/object_model.hpp#L197)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * instance

epok::ObjectPool::release(instance);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-acquire-1"></a>

## `epok::ObjectRegistry::acquire`

**Purpose.** Pool-backed construction of a concrete class.

**Details.** Abstract classes are never created.

**Exact declaration**

```cpp
ObjectId acquire(const ClassDescriptor& type)
```

- **Declared at:** [line 277](../../../runtime/object_model.hpp#L277)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Abstract classes are never created.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.acquire(type);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-adopt-1"></a>

## `epok::ObjectRegistry::adopt`

**Purpose.** Adopt storage owned by somebody else (an actor's embedded default component, the Level itself).

**Details.** Release never destroys it.

**Exact declaration**

```cpp
ObjectId adopt(Object& instance, const ClassDescriptor& type)
```

- **Declared at:** [line 289](../../../runtime/object_model.hpp#L289)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Object &` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Release never destroys it.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object & instance
// const ClassDescriptor & type

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.adopt(instance, type);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-class-of-1"></a>

## `epok::ObjectRegistry::class_of`

**Purpose.** Performs `class of` as part of the object model module.

**Exact declaration**

```cpp
const ClassDescriptor* class_of(ObjectId id)
```

- **Declared at:** [line 271](../../../runtime/object_model.hpp#L271)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassDescriptor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.class_of(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-collect-quarantined-1"></a>

## `epok::ObjectRegistry::collect_quarantined`

**Purpose.** Retry the quarantined slots.

**Details.** The scene bank calls it once per frame, next to the point where the legacy path re-checks music_active before reusing a slot.

**Exact declaration**

```cpp
void collect_quarantined()
```

- **Declared at:** [line 331](../../../runtime/object_model.hpp#L331)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The scene bank calls it once per frame, next to the point where the legacy path re-checks music_active before reusing a slot.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

object.collect_quarantined();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-finish-release-1"></a>

## `epok::ObjectRegistry::finish_release`

**Purpose.** Called when the last dispatch scope unwinds: no callback can still be on the stack of a destroyed object, so its typed storage returns to the pool.

**Details.** Storage a service still points at stays quarantined; the handle is already dead either way, so the only effect is that the slot and the pool entry are not reused yet. Call again (`collect_quarantined`) once the service releases it.

**Exact declaration**

```cpp
void finish_release()
```

- **Declared at:** [line 313](../../../runtime/object_model.hpp#L313)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Storage a service still points at stays quarantined; the handle is already dead either way, so the only effect is that the slot and the pool entry are not reused yet. Call again (`collect_quarantined`) once the service releases it.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

object.finish_release();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-get-1"></a>

## `epok::ObjectRegistry::get`

**Purpose.** Returns get as part of the object model module.

**Exact declaration**

```cpp
Object* get(ObjectId id)
```

- **Declared at:** [line 259](../../../runtime/object_model.hpp#L259)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.get(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-live-1"></a>

## `epok::ObjectRegistry::live`

**Purpose.** Performs `live` as part of the object model module.

**Exact declaration**

```cpp
size_t live() const
```

- **Declared at:** [line 332](../../../runtime/object_model.hpp#L332)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.live();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-next-generation-1"></a>

## `epok::ObjectRegistry::next_generation`

**Purpose.** Performs `next generation` as part of the object model module.

**Exact declaration**

```cpp
static uint16_t next_generation(uint16_t value)
```

- **Declared at:** [line 251](../../../runtime/object_model.hpp#L251)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t value

auto result = epok::ObjectRegistry::next_generation(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-objectregistry-1"></a>

## `epok::ObjectRegistry::ObjectRegistry`

**Purpose.** Constructs `epok::ObjectRegistry` for the object model module.

**Exact declaration**

```cpp
ObjectRegistry(const ObjectRegistry&) = delete
```

- **Declared at:** [line 248](../../../runtime/object_model.hpp#L248)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectRegistry &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectRegistry & arg1

epok::ObjectRegistry value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-objectregistry-2"></a>

## `epok::ObjectRegistry::ObjectRegistry`

**Purpose.** Constructs `epok::ObjectRegistry` for the object model module.

**Exact declaration**

```cpp
constexpr ObjectRegistry(ObjectSlot* table, uint16_t count) : slots(table), capacity(count)
```

- **Declared at:** [line 247](../../../runtime/object_model.hpp#L247)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `ObjectSlot *` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `count` | `uint16_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectSlot * table
// uint16_t count

epok::ObjectRegistry value(table, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-operator-1"></a>

## `epok::ObjectRegistry::operator=`

**Purpose.** Performs `operator =` as part of the object model module.

**Exact declaration**

```cpp
ObjectRegistry& operator=(const ObjectRegistry&) = delete
```

- **Declared at:** [line 249](../../../runtime/object_model.hpp#L249)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ObjectRegistry &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ObjectRegistry &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ObjectRegistry & arg1

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-objectregistry-release-1"></a>

## `epok::ObjectRegistry::release`

**Purpose.** Invalidates the handle immediately; storage is quarantined while callbacks run.

**Exact declaration**

```cpp
bool release(ObjectId id)
```

- **Declared at:** [line 297](../../../runtime/object_model.hpp#L297)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.release(id);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-objectregistry-resolve-1"></a>

## `epok::ObjectRegistry::resolve`

**Purpose.** Generation and class checked; the runtime class must derive from T's static class.

**Exact declaration**

```cpp
template<class T> T* resolve(ObjectId id)
```

- **Declared at:** [line 264](../../../runtime/object_model.hpp#L264)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.resolve<T>(id);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-objectregistry-resolve-2"></a>

## `epok::ObjectRegistry::resolve`

**Purpose.** Performs `resolve` as part of the object model module.

**Exact declaration**

```cpp
template<class T> const T* resolve(ObjectId id) const
```

- **Declared at:** [line 270](../../../runtime/object_model.hpp#L270)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.resolve<T>(id);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-objectregistry-slot-1"></a>

## `epok::ObjectRegistry::slot`

**Purpose.** Performs `slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectSlot* slot(ObjectId id)
```

- **Declared at:** [line 252](../../../runtime/object_model.hpp#L252)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `ObjectSlot *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.slot(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistry-slot-2"></a>

## `epok::ObjectRegistry::slot`

**Purpose.** Performs `slot` as part of the object model module.

**Exact declaration**

```cpp
const ObjectSlot* slot(ObjectId id) const
```

- **Declared at:** [line 258](../../../runtime/object_model.hpp#L258)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ObjectSlot *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::ObjectRegistry& object = /* obtain a valid instance */;

auto result = object.slot(id);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectregistrystorage-objectregistrystorage-capacity-1"></a>

## `epok::ObjectRegistryStorage::ObjectRegistryStorage<Capacity>`

**Purpose.** Constructs `epok::ObjectRegistryStorage` for the object model module.

**Exact declaration**

```cpp
ObjectRegistryStorage() : O
```

- **Declared at:** [line 358](../../../runtime/object_model.hpp#L358)
- **Kind:** `constructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistryStorage value();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-attach-slot-1"></a>

## `epok::RectTransformComponent::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 488](../../../runtime/object_model.hpp#L488)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-bind-local-1"></a>

## `epok::RectTransformComponent::bind_local`

**Purpose.** Performs `bind local` as part of the object model module.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 492](../../../runtime/object_model.hpp#L492)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-bind-slot-1"></a>

## `epok::RectTransformComponent::bind_slot`

**Purpose.** Performs `bind slot` as part of the object model module.

**Exact declaration**

```cpp
void bind_slot(Entity& value)
```

- **Declared at:** [line 491](../../../runtime/object_model.hpp#L491)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Entity &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Entity & value

epok::RectTransformComponent& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-recttransformcomponent-class-id-1"></a>

## `epok::RectTransformComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 487](../../../runtime/object_model.hpp#L487)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-entity-slot-1"></a>

## `epok::RectTransformComponent::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
Entity* entity_slot() const
```

- **Declared at:** [line 493](../../../runtime/object_model.hpp#L493)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::RectTransformComponent& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-attach-slot-1"></a>

## `epok::SceneComponent2D::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 473](../../../runtime/object_model.hpp#L473)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent2d-class-id-1"></a>

## `epok::SceneComponent2D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 472](../../../runtime/object_model.hpp#L472)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-attach-slot-1"></a>

## `epok::SceneComponent3D::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 455](../../../runtime/object_model.hpp#L455)
- **Kind:** `cxx method`

**Returns.** Returns `ObjectId *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.attach_slot();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-bind-local-1"></a>

## `epok::SceneComponent3D::bind_local`

**Purpose.** No legacy slot: the component owns the transform.

**Exact declaration**

```cpp
void bind_local()
```

- **Declared at:** [line 463](../../../runtime/object_model.hpp#L463)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.bind_local();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-bind-slot-1"></a>

## `epok::SceneComponent3D::bind_slot`

**Purpose.** Canonical storage: the legacy entity slot owns the transform and everything that already reads it (collision, rendering, motion interpolation) keeps working.

**Exact declaration**

```cpp
void bind_slot(Entity& value)
```

- **Declared at:** [line 461](../../../runtime/object_model.hpp#L461)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Entity &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Entity & value

epok::SceneComponent3D& object = /* obtain a valid instance */;

object.bind_slot(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenecomponent3d-class-id-1"></a>

## `epok::SceneComponent3D::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 454](../../../runtime/object_model.hpp#L454)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenecomponent3d-entity-slot-1"></a>

## `epok::SceneComponent3D::entity_slot`

**Purpose.** Performs `entity slot` as part of the object model module.

**Exact declaration**

```cpp
Entity* entity_slot() const
```

- **Declared at:** [line 464](../../../runtime/object_model.hpp#L464)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneComponent3D& object = /* obtain a valid instance */;

auto result = object.entity_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenescriptactor-class-id-1"></a>

## `epok::SceneScriptActor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 619](../../../runtime/object_model.hpp#L619)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::SceneScriptActor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uiactor-class-id-1"></a>

## `epok::UIActor::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 611](../../../runtime/object_model.hpp#L611)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIActor& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uiactor-default-root-1"></a>

## `epok::UIActor::default_root`

**Purpose.** Performs `default root` as part of the object model module.

**Exact declaration**

```cpp
ActorComponent* default_root() override
```

- **Declared at:** [line 613](../../../runtime/object_model.hpp#L613)
- **Kind:** `cxx method`

**Returns.** Returns `ActorComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIActor& object = /* obtain a valid instance */;

auto result = object.default_root();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-uicomponent-class-id-1"></a>

## `epok::UIComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 482](../../../runtime/object_model.hpp#L482)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::UIComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world-bind-1"></a>

## `epok::World::bind`

**Purpose.** Performs `bind` as part of the object model module.

**Exact declaration**

```cpp
void bind(Level& value, ObjectRegistry& table)
```

- **Declared at:** [line 1068](../../../runtime/object_model.hpp#L1068)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Level &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |
| `table` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Level & value
// ObjectRegistry & table

epok::World& object = /* obtain a valid instance */;

object.bind(value, table);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-class-id-1"></a>

## `epok::World::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 1064](../../../runtime/object_model.hpp#L1064)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::World& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
