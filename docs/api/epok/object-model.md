# Epok API: Object Model

> **Header:** `"object_model.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/object_model.hpp)

This module covers the object model module. It documents 150 public callables declared directly in this header.

## Declared types

`epok::Actor`, `epok::Actor2D`, `epok::Actor3D`, `epok::ActorComponent`, `epok::ActorSpawnRequest`, `epok::AudioComponent`, `epok::BlobShadowComponent`, `epok::Camera3DComponent`, `epok::CanvasComponent`, `epok::ClassDescriptor`, `epok::Collider3DComponent`, `epok::EndPlayReason`, `epok::ImageComponent`, `epok::Level`, `epok::Level::ActorPrepareFn`, `epok::LevelPendingOp`, `epok::Light3DComponent`, `epok::Mesh3DComponent`, `epok::NativeComponentDefault`, `epok::Object`, `epok::ObjectClassFlags`, `epok::ObjectDispatchScope`, `epok::ObjectDomain`, `epok::ObjectFamily`, `epok::ObjectId`, `epok::ObjectPool`, `epok::ObjectRegistry`, `epok::ObjectRegistryStorage`, `epok::ObjectSlot`, `epok::ObjectState`, `epok::ObjectStats`, `epok::PaletteAnimatorComponent`, `epok::ParticleEffectComponent`, `epok::ParticleEmitterComponent`, `epok::ProgressBarComponent`, `epok::RectTransformComponent`, `epok::SceneComponent2D`, `epok::SceneComponent3D`, `epok::SceneScriptActor`, `epok::Sprite3DComponent`, `epok::TextComponent`, `epok::TimelineComponent`, `epok::Transform2D`, `epok::UIActor`, `epok::UIComponent`, `epok::World`

## Callable index

- [`epok::Actor2D::class_id`](#epok-actor2d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor2D::default_root`](#epok-actor2d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor3D::class_id`](#epok-actor3d-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor3D::default_root`](#epok-actor3d-default-root-1) — Performs `default root` as part of the object model module.
- [`epok::Actor::active`](#epok-actor-active-1) — Self flag only; Level::actor_active() folds the logical parent chain.
- [`epok::Actor::begin_play`](#epok-actor-begin-play-1) — Begins play as part of the object model module.
- [`epok::Actor::bind_data`](#epok-actor-bind-data-1) — Performs `bind data` as part of the object model module.
- [`epok::Actor::blueprint_observe`](#epok-actor-blueprint-observe-1) — Performs `blueprint observe` as part of the object model module.
- [`epok::Actor::class_id`](#epok-actor-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Actor::component_count`](#epok-actor-component-count-1) — Performs `component count` as part of the object model module.
- [`epok::Actor::component_id`](#epok-actor-component-id-1) — Performs `component id` as part of the object model module.
- [`epok::Actor::data`](#epok-actor-data-1) — Legacy adapter: the canonical slot behind the root scene component, or nullptr.
- [`epok::Actor::default_root`](#epok-actor-default-root-1) — Runtime hook, not reflected: the declarative root component embedded in the actor.
- [`epok::Actor::end_play`](#epok-actor-end-play-1) — Ends play as part of the object model module.
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
- [`epok::ActorComponent::blueprint_observe`](#epok-actorcomponent-blueprint-observe-1) — Performs `blueprint observe` as part of the object model module.
- [`epok::ActorComponent::class_id`](#epok-actorcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ActorComponent::end_play`](#epok-actorcomponent-end-play-1) — Ends play as part of the object model module.
- [`epok::ActorComponent::frame_update`](#epok-actorcomponent-frame-update-1) — Performs `frame update` as part of the object model module.
- [`epok::ActorComponent::get_owner`](#epok-actorcomponent-get-owner-1) — ---- out-of-line definitions -------------------------------------------------------
- [`epok::ActorComponent::name`](#epok-actorcomponent-name-1) — Performs `name` as part of the object model module.
- [`epok::ActorComponent::on_disable`](#epok-actorcomponent-on-disable-1) — Performs `on disable` as part of the object model module.
- [`epok::ActorComponent::on_enable`](#epok-actorcomponent-on-enable-1) — Performs `on enable` as part of the object model module.
- [`epok::ActorComponent::on_trigger`](#epok-actorcomponent-on-trigger-1) — Forward-only collision hook.
- [`epok::ActorComponent::owner_id`](#epok-actorcomponent-owner-id-1) — Performs `owner id` as part of the object model module.
- [`epok::ActorComponent::releasable`](#epok-actorcomponent-releasable-1) — False while a service still holds this component's storage; the registry then keeps the (already dead) slot quarantined instead of returning it to the pool.
- [`epok::ActorComponent::set_name`](#epok-actorcomponent-set-name-1) — Sets name as part of the object model module.
- [`epok::ActorComponent::tick`](#epok-actorcomponent-tick-1) — Performs `tick` as part of the object model module.
- [`epok::ActorComponent::timeline_sync`](#epok-actorcomponent-timeline-sync-1) — Runtime hooks, not reflected.
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
- [`epok::AudioComponent::releasable`](#epok-audiocomponent-releasable-1) — Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring allocate_actor_data's legacy-slot rule.
- [`epok::AudioComponent::stop`](#epok-audiocomponent-stop-1) — Stops stop as part of the object model module.
- [`epok::BlobShadowComponent::class_id`](#epok-blobshadowcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Camera3DComponent::class_id`](#epok-camera3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::CanvasComponent::class_id`](#epok-canvascomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Collider3DComponent::class_id`](#epok-collider3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::detail::compact_class_id`](#epok-detail-compact-class-id-1) — Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).
- [`epok::detail::sha256_rotr`](#epok-detail-sha256-rotr-1) — Performs `sha256 rotr` as part of the object model module.
- [`epok::dispatch_trigger`](#epok-dispatch-trigger-1) — Collision is owned by the spatial services, not by the Level: they resolve the legacy slot to its actor and call this, which fans the event out to the owner's components in registration order.
- [`epok::find_object_class`](#epok-find-object-class-1) — Finds object class as part of the object model module.
- [`epok::ImageComponent::class_id`](#epok-imagecomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::is_active`](#epok-is-active-1) — Reports whether active as part of the object model module.
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
- [`epok::Level::set_logical_parent`](#epok-level-set-logical-parent-1) — Sets logical parent as part of the object model module.
- [`epok::Level::spawn_actor`](#epok-level-spawn-actor-1) — Performs `spawn actor` as part of the object model module.
- [`epok::Level::spawn_batch`](#epok-level-spawn-batch-1) — All or nothing.
- [`epok::Level::stats`](#epok-level-stats-1) — Performs `stats` as part of the object model module.
- [`epok::Level::tick`](#epok-level-tick-1) — ---- ticking -------------------------------------------------------------------
- [`epok::Light3DComponent::class_id`](#epok-light3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Mesh3DComponent::class_id`](#epok-mesh3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::Object::class_id`](#epok-object-class-id-1) — Runtime-owned identity hooks; not an authoring API.
- [`epok::Object::id`](#epok-object-id-1) — Performs `id` as part of the object model module.
- [`epok::Object::is_a`](#epok-object-is-a-1) — Reports whether a as part of the object model module.
- [`epok::Object::state`](#epok-object-state-1) — Performs `state` as part of the object model module.
- [`epok::Object::timeline_sync`](#epok-object-timeline-sync-1) — Performs `timeline sync` as part of the object model module.
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
- [`epok::ObjectId::get`](#epok-objectid-get-1) — Returns get as part of the object model module.
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
- [`epok::PaletteAnimatorComponent::class_id`](#epok-paletteanimatorcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ParticleEffectComponent::class_id`](#epok-particleeffectcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ParticleEmitterComponent::class_id`](#epok-particleemittercomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::ProgressBarComponent::class_id`](#epok-progressbarcomponent-class-id-1) — Performs `class id` as part of the object model module.
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
- [`epok::Sprite3DComponent::class_id`](#epok-sprite3dcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::TextComponent::class_id`](#epok-textcomponent-class-id-1) — Performs `class id` as part of the object model module.
- [`epok::TimelineComponent::class_id`](#epok-timelinecomponent-class-id-1) — Performs `class id` as part of the object model module.
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

- **Declared at:** [line 647](../../../runtime/object_model.hpp#L647)
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

- **Declared at:** [line 649](../../../runtime/object_model.hpp#L649)
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

- **Declared at:** [line 640](../../../runtime/object_model.hpp#L640)
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

- **Declared at:** [line 642](../../../runtime/object_model.hpp#L642)
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

- **Declared at:** [line 421](../../../runtime/object_model.hpp#L421)
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

- **Declared at:** [line 398](../../../runtime/object_model.hpp#L398)
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

<a id="epok-actor-bind-data-1"></a>

## `epok::Actor::bind_data`

**Purpose.** Performs `bind data` as part of the object model module.

**Exact declaration**

```cpp
void bind_data(ActorData& value)
```

- **Declared at:** [line 408](../../../runtime/object_model.hpp#L408)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

epok::Actor& object = /* obtain a valid instance */;

object.bind_data(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-actor-blueprint-observe-1"></a>

## `epok::Actor::blueprint_observe`

**Purpose.** Performs `blueprint observe` as part of the object model module.

**Exact declaration**

```cpp
virtual void blueprint_observe()
```

- **Declared at:** [line 399](../../../runtime/object_model.hpp#L399)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

object.blueprint_observe();
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

- **Declared at:** [line 397](../../../runtime/object_model.hpp#L397)
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

- **Declared at:** [line 419](../../../runtime/object_model.hpp#L419)
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

- **Declared at:** [line 418](../../../runtime/object_model.hpp#L418)
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

<a id="epok-actor-data-1"></a>

## `epok::Actor::data`

**Purpose.** Legacy adapter: the canonical slot behind the root scene component, or nullptr.

**Exact declaration**

```cpp
ActorData* data() const
```

- **Declared at:** [line 407](../../../runtime/object_model.hpp#L407)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Actor& object = /* obtain a valid instance */;

auto result = object.data();
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

- **Declared at:** [line 405](../../../runtime/object_model.hpp#L405)
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
virtual void end_play(EndPlayReason end_play_reason)
```

- **Declared at:** [line 401](../../../runtime/object_model.hpp#L401)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `end_play_reason` | `EndPlayReason` | Input | Value supplied for `end_play_reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason end_play_reason

epok::Actor& object = /* obtain a valid instance */;

object.end_play(end_play_reason);
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

- **Declared at:** [line 415](../../../runtime/object_model.hpp#L415)
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

- **Declared at:** [line 417](../../../runtime/object_model.hpp#L417)
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

- **Declared at:** [line 409](../../../runtime/object_model.hpp#L409)
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

- **Declared at:** [line 403](../../../runtime/object_model.hpp#L403)
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

- **Declared at:** [line 402](../../../runtime/object_model.hpp#L402)
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

- **Declared at:** [line 416](../../../runtime/object_model.hpp#L416)
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

- **Declared at:** [line 410](../../../runtime/object_model.hpp#L410)
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

- **Declared at:** [line 423](../../../runtime/object_model.hpp#L423)
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
virtual void tick(Fixed delta_seconds)
```

- **Declared at:** [line 400](../../../runtime/object_model.hpp#L400)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::Actor& object = /* obtain a valid instance */;

object.tick(delta_seconds);
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

- **Declared at:** [line 422](../../../runtime/object_model.hpp#L422)
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

- **Declared at:** [line 450](../../../runtime/object_model.hpp#L450)
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

- **Declared at:** [line 440](../../../runtime/object_model.hpp#L440)
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

<a id="epok-actorcomponent-blueprint-observe-1"></a>

## `epok::ActorComponent::blueprint_observe`

**Purpose.** Performs `blueprint observe` as part of the object model module.

**Exact declaration**

```cpp
virtual void blueprint_observe()
```

- **Declared at:** [line 448](../../../runtime/object_model.hpp#L448)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ActorComponent& object = /* obtain a valid instance */;

object.blueprint_observe();
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

- **Declared at:** [line 439](../../../runtime/object_model.hpp#L439)
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
virtual void end_play(EndPlayReason end_play_reason)
```

- **Declared at:** [line 442](../../../runtime/object_model.hpp#L442)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `end_play_reason` | `EndPlayReason` | Input | Value supplied for `end_play_reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason end_play_reason

epok::ActorComponent& object = /* obtain a valid instance */;

object.end_play(end_play_reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-frame-update-1"></a>

## `epok::ActorComponent::frame_update`

**Purpose.** Performs `frame update` as part of the object model module.

**Exact declaration**

```cpp
virtual void frame_update(uint32_t)
```

- **Declared at:** [line 449](../../../runtime/object_model.hpp#L449)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint32_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

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

**Purpose.** ---- out-of-line definitions -------------------------------------------------------

**Exact declaration**

```cpp
Actor* get_owner()
```

- **Declared at:** [line 459](../../../runtime/object_model.hpp#L459)
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

- **Declared at:** [line 460](../../../runtime/object_model.hpp#L460)
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

- **Declared at:** [line 444](../../../runtime/object_model.hpp#L444)
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

- **Declared at:** [line 443](../../../runtime/object_model.hpp#L443)
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
virtual void on_trigger(DataHandle, TriggerPhase)
```

- **Declared at:** [line 454](../../../runtime/object_model.hpp#L454)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `DataHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `TriggerPhase` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The Level does not own collision; the collision service calls dispatch_trigger(Level&, ...) which fans the event out to the owner's components. Nothing in the object model generates trigger events.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle arg1
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

- **Declared at:** [line 458](../../../runtime/object_model.hpp#L458)
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

- **Declared at:** [line 457](../../../runtime/object_model.hpp#L457)
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

- **Declared at:** [line 461](../../../runtime/object_model.hpp#L461)
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
virtual void tick(Fixed delta_seconds)
```

- **Declared at:** [line 441](../../../runtime/object_model.hpp#L441)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::ActorComponent& object = /* obtain a valid instance */;

object.tick(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actorcomponent-timeline-sync-1"></a>

## `epok::ActorComponent::timeline_sync`

**Purpose.** Runtime hooks, not reflected.

**Details.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Exact declaration**

```cpp
virtual void timeline_sync(uint64_t, bool)
```

- **Declared at:** [line 447](../../../runtime/object_model.hpp#L447)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** frame_update runs once per rendered frame even while paused; attach_slot exposes the spatial parent of components that have one.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1
// bool arg2

epok::ActorComponent& object = /* obtain a valid instance */;

object.timeline_sync(arg1, arg2);
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

- **Declared at:** [line 1248](../../../runtime/object_model.hpp#L1248)
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

- **Declared at:** [line 544](../../../runtime/object_model.hpp#L544)
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

- **Declared at:** [line 531](../../../runtime/object_model.hpp#L531)
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
void bind_slot(ActorData& value)
```

- **Declared at:** [line 530](../../../runtime/object_model.hpp#L530)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

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

- **Declared at:** [line 524](../../../runtime/object_model.hpp#L524)
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

- **Declared at:** [line 553](../../../runtime/object_model.hpp#L553)
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
ActorData* entity_slot() const
```

- **Declared at:** [line 532](../../../runtime/object_model.hpp#L532)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 528](../../../runtime/object_model.hpp#L528)
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

- **Declared at:** [line 552](../../../runtime/object_model.hpp#L552)
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

- **Declared at:** [line 535](../../../runtime/object_model.hpp#L535)
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

- **Declared at:** [line 526](../../../runtime/object_model.hpp#L526)
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

**Purpose.** Component-owned storage stays quarantined while an asynchronous consumer (the XA music service) still points at it, mirroring allocate_actor_data's legacy-slot rule.

**Details.** Slot storage belongs to the scene bank, so this component never holds it back.

**Exact declaration**

```cpp
bool releasable() const override
```

- **Declared at:** [line 557](../../../runtime/object_model.hpp#L557)
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

- **Declared at:** [line 527](../../../runtime/object_model.hpp#L527)
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

<a id="epok-blobshadowcomponent-class-id-1"></a>

## `epok::BlobShadowComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 634](../../../runtime/object_model.hpp#L634)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::BlobShadowComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera3dcomponent-class-id-1"></a>

## `epok::Camera3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 579](../../../runtime/object_model.hpp#L579)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Camera3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvascomponent-class-id-1"></a>

## `epok::CanvasComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 594](../../../runtime/object_model.hpp#L594)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::CanvasComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider3dcomponent-class-id-1"></a>

## `epok::Collider3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 589](../../../runtime/object_model.hpp#L589)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Collider3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-compact-class-id-1"></a>

## `epok::detail::compact_class_id`

**Purpose.** Single-block SHA-256: enough for a 36 character UUID (message + padding <= 64 bytes).

**Exact declaration**

```cpp
constexpr uint64_t compact_class_id(const char* text)
```

- **Declared at:** [line 90](../../../runtime/object_model.hpp#L90)
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

- **Declared at:** [line 86](../../../runtime/object_model.hpp#L86)
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
inline size_t dispatch_trigger(Level& level, ObjectId actor, DataHandle other, TriggerPhase phase)
```

- **Declared at:** [line 1222](../../../runtime/object_model.hpp#L1222)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `level` | `Level &` | Input/output; inspect the function contract | Value supplied for `level`. See the exact type and module contract. |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `other` | `DataHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** Inactive, unstarted and doomed actors receive nothing.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Level & level
// ObjectId actor
// DataHandle other
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

- **Declared at:** [line 153](../../../runtime/object_model.hpp#L153)
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

<a id="epok-imagecomponent-class-id-1"></a>

## `epok::ImageComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 599](../../../runtime/object_model.hpp#L599)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ImageComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-is-active-1"></a>

## `epok::is_active`

**Purpose.** Reports whether active as part of the object model module.

**Exact declaration**

```cpp
inline bool is_active(Object* value)
```

- **Declared at:** [line 1238](../../../runtime/object_model.hpp#L1238)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Object *` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// Object * value

auto result = epok::is_active(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-level-actor-active-1"></a>

## `epok::Level::actor_active`

**Purpose.** ---- activation ----------------------------------------------------------------

**Exact declaration**

```cpp
bool actor_active(const Actor& actor) const
```

- **Declared at:** [line 865](../../../runtime/object_model.hpp#L865)
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

- **Declared at:** [line 699](../../../runtime/object_model.hpp#L699)
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

- **Declared at:** [line 698](../../../runtime/object_model.hpp#L698)
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

- **Declared at:** [line 906](../../../runtime/object_model.hpp#L906)
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

- **Declared at:** [line 689](../../../runtime/object_model.hpp#L689)
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

- **Declared at:** [line 685](../../../runtime/object_model.hpp#L685)
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

- **Declared at:** [line 796](../../../runtime/object_model.hpp#L796)
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

- **Declared at:** [line 813](../../../runtime/object_model.hpp#L813)
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

- **Declared at:** [line 833](../../../runtime/object_model.hpp#L833)
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

- **Declared at:** [line 854](../../../runtime/object_model.hpp#L854)
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

- **Declared at:** [line 924](../../../runtime/object_model.hpp#L924)
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

- **Declared at:** [line 931](../../../runtime/object_model.hpp#L931)
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

- **Declared at:** [line 696](../../../runtime/object_model.hpp#L696)
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

- **Declared at:** [line 942](../../../runtime/object_model.hpp#L942)
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

- **Declared at:** [line 700](../../../runtime/object_model.hpp#L700)
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

- **Declared at:** [line 874](../../../runtime/object_model.hpp#L874)
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

<a id="epok-level-set-logical-parent-1"></a>

## `epok::Level::set_logical_parent`

**Purpose.** Sets logical parent as part of the object model module.

**Exact declaration**

```cpp
bool set_logical_parent(ObjectId child,ObjectId parent)
```

- **Declared at:** [line 701](../../../runtime/object_model.hpp#L701)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `ObjectId` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId child
// ObjectId parent

epok::Level& object = /* obtain a valid instance */;

auto result = object.set_logical_parent(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-level-spawn-actor-1"></a>

## `epok::Level::spawn_actor`

**Purpose.** Performs `spawn actor` as part of the object model module.

**Exact declaration**

```cpp
virtual ObjectId spawn_actor(const ClassDescriptor& type, const char* name, ObjectId logical_parent = {})
```

- **Declared at:** [line 786](../../../runtime/object_model.hpp#L786)
- **Kind:** `cxx method`; qualifiers: `virtual`

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

**Purpose.** All or nothing.

**Details.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Exact declaration**

```cpp
size_t spawn_batch(const ActorSpawnRequest* requests, size_t count, ObjectId* out, ActorPrepareFn prepare = nullptr)
```

- **Declared at:** [line 729](../../../runtime/object_model.hpp#L729)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `requests` | `const ActorSpawnRequest *` | Input | Value supplied for `requests`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `out` | `ObjectId *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `prepare` | `ActorPrepareFn` | Input | Value supplied for `prepare`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** On any failure every reservation of this batch is released, so a half-built actor never keeps orphan components.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// const ActorSpawnRequest * requests
// size_t count
// ObjectId * out
// ActorPrepareFn prepare

epok::Level& object = /* obtain a valid instance */;

auto result = object.spawn_batch(requests, count, out, prepare);
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

- **Declared at:** [line 697](../../../runtime/object_model.hpp#L697)
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

- **Declared at:** [line 844](../../../runtime/object_model.hpp#L844)
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

<a id="epok-light3dcomponent-class-id-1"></a>

## `epok::Light3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 584](../../../runtime/object_model.hpp#L584)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Light3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mesh3dcomponent-class-id-1"></a>

## `epok::Mesh3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 569](../../../runtime/object_model.hpp#L569)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Mesh3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-class-id-1"></a>

## `epok::Object::class_id`

**Purpose.** Runtime-owned identity hooks; not an authoring API.

**Exact declaration**

```cpp
virtual uint64_t class_id() const
```

- **Declared at:** [line 175](../../../runtime/object_model.hpp#L175)
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

- **Declared at:** [line 177](../../../runtime/object_model.hpp#L177)
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

- **Declared at:** [line 179](../../../runtime/object_model.hpp#L179)
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

- **Declared at:** [line 178](../../../runtime/object_model.hpp#L178)
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

<a id="epok-object-timeline-sync-1"></a>

## `epok::Object::timeline_sync`

**Purpose.** Performs `timeline sync` as part of the object model module.

**Exact declaration**

```cpp
virtual void timeline_sync(uint64_t, bool)
```

- **Declared at:** [line 176](../../../runtime/object_model.hpp#L176)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1
// bool arg2

epok::Object& object = /* obtain a valid instance */;

object.timeline_sync(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-object-1"></a>

## `epok::Object::~Object`

**Purpose.** Releases the resources owned by `epok::Object`.

**Exact declaration**

```cpp
virtual ~Object() = default
```

- **Declared at:** [line 173](../../../runtime/object_model.hpp#L173)
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

- **Declared at:** [line 159](../../../runtime/object_model.hpp#L159)
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

- **Declared at:** [line 190](../../../runtime/object_model.hpp#L190)
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

- **Declared at:** [line 191](../../../runtime/object_model.hpp#L191)
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

- **Declared at:** [line 67](../../../runtime/object_model.hpp#L67)
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

- **Declared at:** [line 251](../../../runtime/object_model.hpp#L251)
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

- **Declared at:** [line 381](../../../runtime/object_model.hpp#L381)
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

- **Declared at:** [line 379](../../../runtime/object_model.hpp#L379)
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

- **Declared at:** [line 382](../../../runtime/object_model.hpp#L382)
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

- **Declared at:** [line 380](../../../runtime/object_model.hpp#L380)
- **Kind:** `destructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// `epok::ObjectDispatchScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectid-get-1"></a>

## `epok::ObjectId::get`

**Purpose.** Returns get as part of the object model module.

**Exact declaration**

```cpp
Object* get() const
```

- **Declared at:** [line 38](../../../runtime/object_model.hpp#L38)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectId& object = /* obtain a valid instance */;

auto result = object.get();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-objectid-operator-1"></a>

## `epok::ObjectId::operator!=`

**Purpose.** Performs `operator !=` as part of the object model module.

**Exact declaration**

```cpp
constexpr bool operator!=(const ObjectId& other) const
```

- **Declared at:** [line 42](../../../runtime/object_model.hpp#L42)
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

- **Declared at:** [line 39](../../../runtime/object_model.hpp#L39)
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

- **Declared at:** [line 37](../../../runtime/object_model.hpp#L37)
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

- **Declared at:** [line 202](../../../runtime/object_model.hpp#L202)
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

- **Declared at:** [line 218](../../../runtime/object_model.hpp#L218)
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

- **Declared at:** [line 210](../../../runtime/object_model.hpp#L210)
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

- **Declared at:** [line 292](../../../runtime/object_model.hpp#L292)
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

- **Declared at:** [line 304](../../../runtime/object_model.hpp#L304)
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

- **Declared at:** [line 286](../../../runtime/object_model.hpp#L286)
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

- **Declared at:** [line 346](../../../runtime/object_model.hpp#L346)
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

- **Declared at:** [line 328](../../../runtime/object_model.hpp#L328)
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

- **Declared at:** [line 274](../../../runtime/object_model.hpp#L274)
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

- **Declared at:** [line 347](../../../runtime/object_model.hpp#L347)
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

- **Declared at:** [line 266](../../../runtime/object_model.hpp#L266)
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

- **Declared at:** [line 263](../../../runtime/object_model.hpp#L263)
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

- **Declared at:** [line 262](../../../runtime/object_model.hpp#L262)
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

- **Declared at:** [line 264](../../../runtime/object_model.hpp#L264)
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

- **Declared at:** [line 312](../../../runtime/object_model.hpp#L312)
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

- **Declared at:** [line 279](../../../runtime/object_model.hpp#L279)
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

- **Declared at:** [line 285](../../../runtime/object_model.hpp#L285)
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

- **Declared at:** [line 267](../../../runtime/object_model.hpp#L267)
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

- **Declared at:** [line 273](../../../runtime/object_model.hpp#L273)
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

- **Declared at:** [line 374](../../../runtime/object_model.hpp#L374)
- **Kind:** `constructor`

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ObjectRegistryStorage value();
```

**Why choose it.** It provides direct, allocation-conscious access to the object model module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimatorcomponent-class-id-1"></a>

## `epok::PaletteAnimatorComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 629](../../../runtime/object_model.hpp#L629)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::PaletteAnimatorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleeffectcomponent-class-id-1"></a>

## `epok::ParticleEffectComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 624](../../../runtime/object_model.hpp#L624)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEffectComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemittercomponent-class-id-1"></a>

## `epok::ParticleEmitterComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 614](../../../runtime/object_model.hpp#L614)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ParticleEmitterComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-progressbarcomponent-class-id-1"></a>

## `epok::ProgressBarComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 609](../../../runtime/object_model.hpp#L609)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::ProgressBarComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-recttransformcomponent-attach-slot-1"></a>

## `epok::RectTransformComponent::attach_slot`

**Purpose.** Performs `attach slot` as part of the object model module.

**Exact declaration**

```cpp
ObjectId* attach_slot() override
```

- **Declared at:** [line 510](../../../runtime/object_model.hpp#L510)
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

- **Declared at:** [line 514](../../../runtime/object_model.hpp#L514)
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
void bind_slot(ActorData& value)
```

- **Declared at:** [line 513](../../../runtime/object_model.hpp#L513)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

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

- **Declared at:** [line 509](../../../runtime/object_model.hpp#L509)
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
ActorData* entity_slot() const
```

- **Declared at:** [line 515](../../../runtime/object_model.hpp#L515)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 495](../../../runtime/object_model.hpp#L495)
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

- **Declared at:** [line 494](../../../runtime/object_model.hpp#L494)
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

- **Declared at:** [line 477](../../../runtime/object_model.hpp#L477)
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

- **Declared at:** [line 485](../../../runtime/object_model.hpp#L485)
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
void bind_slot(ActorData& value)
```

- **Declared at:** [line 483](../../../runtime/object_model.hpp#L483)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & value

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

- **Declared at:** [line 476](../../../runtime/object_model.hpp#L476)
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
ActorData* entity_slot() const
```

- **Declared at:** [line 486](../../../runtime/object_model.hpp#L486)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 662](../../../runtime/object_model.hpp#L662)
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

<a id="epok-sprite3dcomponent-class-id-1"></a>

## `epok::Sprite3DComponent::class_id`

**Purpose.** Performs `class id` as part of the object model module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 574](../../../runtime/object_model.hpp#L574)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object model module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_model.hpp"

epok::Sprite3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-textcomponent-class-id-1"></a>

## `epok::TextComponent::class_id`

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

epok::TextComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timelinecomponent-class-id-1"></a>

## `epok::TimelineComponent::class_id`

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

epok::TimelineComponent& object = /* obtain a valid instance */;

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

- **Declared at:** [line 654](../../../runtime/object_model.hpp#L654)
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

- **Declared at:** [line 656](../../../runtime/object_model.hpp#L656)
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

- **Declared at:** [line 504](../../../runtime/object_model.hpp#L504)
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

- **Declared at:** [line 1198](../../../runtime/object_model.hpp#L1198)
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

- **Declared at:** [line 1194](../../../runtime/object_model.hpp#L1194)
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
