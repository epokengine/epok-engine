# Epok API: Epok

> **Header:** `"epok.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/epok.hpp)

This module covers the epok module. It documents 51 public callables declared directly in this header.

## Declared types

`epok::Aabb`, `epok::AnimationClip`, `epok::Animator`, `epok::AudioSource`, `epok::Behaviour`, `epok::Binding`, `epok::BlobShadow`, `epok::Bone`, `epok::BonePose`, `epok::BoneTrack`, `epok::CameraSettings`, `epok::Canvas`, `epok::ChunkVisibility`, `epok::Collider`, `epok::Entity`, `epok::EntityHandle`, `epok::Fixed`, `epok::Image`, `epok::LegacyEntityStorage`, `epok::Light`, `epok::LightingEnvironment`, `epok::LightingStats`, `epok::LightMode`, `epok::LightType`, `epok::Material`, `epok::MeshGeometry`, `epok::MeshLighting`, `epok::MeshQuad`, `epok::MeshStats`, `epok::MoveResult`, `epok::MusicStats`, `epok::PerformanceStats`, `epok::ProgressBar`, `epok::ReceiveLighting`, `epok::RectTransform`, `epok::SkeletalMesh`, `epok::SpatialHit`, `epok::Transform`

## Callable index

- [`epok::activate_texture_bank`](#epok-activate-texture-bank-1) — Performs `activate texture bank` as part of the epok module.
- [`epok::active_camera`](#epok-active-camera-1) — Performs `active camera` as part of the epok module.
- [`epok::Animator::advance`](#epok-animator-advance-1) — Performs `advance` as part of the epok module.
- [`epok::Animator::pause`](#epok-animator-pause-1) — Pauses pause as part of the epok module.
- [`epok::Animator::play`](#epok-animator-play-1) — Starts play as part of the epok module.
- [`epok::Animator::resume`](#epok-animator-resume-1) — Resumes resume as part of the epok module.
- [`epok::Animator::stop`](#epok-animator-stop-1) — Stops stop as part of the epok module.
- [`epok::AudioSource::is_playing`](#epok-audiosource-is-playing-1) — Reports whether playing as part of the epok module.
- [`epok::AudioSource::play`](#epok-audiosource-play-1) — Starts play as part of the epok module.
- [`epok::AudioSource::stop`](#epok-audiosource-stop-1) — Stops stop as part of the epok module.
- [`epok::Behaviour::bind`](#epok-behaviour-bind-1) — Performs `bind` as part of the epok module.
- [`epok::Behaviour::entity`](#epok-behaviour-entity-1) — Performs `entity` as part of the epok module.
- [`epok::Behaviour::frame_update`](#epok-behaviour-frame-update-1) — Runs once per rendered frame even while paused.
- [`epok::Behaviour::on_destroy`](#epok-behaviour-on-destroy-1) — Performs `on destroy` as part of the epok module.
- [`epok::Behaviour::on_disable`](#epok-behaviour-on-disable-1) — Performs `on disable` as part of the epok module.
- [`epok::Behaviour::on_enable`](#epok-behaviour-on-enable-1) — Performs `on enable` as part of the epok module.
- [`epok::Behaviour::on_trigger`](#epok-behaviour-on-trigger-1) — Performs `on trigger` as part of the epok module.
- [`epok::Behaviour::start`](#epok-behaviour-start-1) — Starts start as part of the epok module.
- [`epok::Behaviour::timeline_sync`](#epok-behaviour-timeline-sync-1) — Typed component adapters synchronize only the reflected property being read or written.
- [`epok::Behaviour::update`](#epok-behaviour-update-1) — Updates update as part of the epok module.
- [`epok::camera_project`](#epok-camera-project-1) — Performs `camera project` as part of the epok module.
- [`epok::collider_aabb`](#epok-collider-aabb-1) — Performs `collider aabb` as part of the epok module.
- [`epok::create_entity`](#epok-create-entity-1) — Creates entity as part of the epok module.
- [`epok::current_scene`](#epok-current-scene-1) — Performs `current scene` as part of the epok module.
- [`epok::destroy_entity`](#epok-destroy-entity-1) — Destroys entity as part of the epok module.
- [`epok::EntityHandle::get`](#epok-entityhandle-get-1) — Returns get as part of the epok module.
- [`epok::EntityHandle::operator bool`](#epok-entityhandle-operator-bool-1) — Performs `operator  bool` as part of the epok module.
- [`epok::find_entity`](#epok-find-entity-1) — Finds entity as part of the epok module.
- [`epok::handle`](#epok-handle-1) — Performs `handle` as part of the epok module.
- [`epok::hit_entity`](#epok-hit-entity-1) — Performs `hit entity` as part of the epok module.
- [`epok::is_active`](#epok-is-active-1) — Reports whether active as part of the epok module.
- [`epok::is_active_slot`](#epok-is-active-slot-1) — Same test for a slot index; skips the pointer validation of is_active.
- [`epok::LegacyEntityStorage::add`](#epok-legacyentitystorage-add-1) — Adds add as part of the epok module.
- [`epok::LegacyEntityStorage::component`](#epok-legacyentitystorage-component-1) — Performs `component` as part of the epok module.
- [`epok::LegacyEntityStorage::get`](#epok-legacyentitystorage-get-1) — Returns get as part of the epok module.
- [`epok::LegacyEntityStorage::remove`](#epok-legacyentitystorage-remove-1) — Removes remove as part of the epok module.
- [`epok::LegacyEntityStorage::set_name`](#epok-legacyentitystorage-set-name-1) — Sets name as part of the epok module.
- [`epok::move_and_slide`](#epok-move-and-slide-1) — Performs `move and slide` as part of the epok module.
- [`epok::overlap`](#epok-overlap-1) — Performs `overlap` as part of the epok module.
- [`epok::query_ground`](#epok-query-ground-1) — Performs `query ground` as part of the epok module.
- [`epok::raycast`](#epok-raycast-1) — Spatial queries use world coordinates.
- [`epok::remove_runtime_owner`](#epok-remove-runtime-owner-1) — Removes runtime owner as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-1) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-2) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-3) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-4) — Requests scene as part of the epok module.
- [`epok::reset_motion_interpolation`](#epok-reset-motion-interpolation-1) — Call after an intentional teleport/cut to snap visual position history.
- [`epok::reset_runtime_services`](#epok-reset-runtime-services-1) — Resets runtime services as part of the epok module.
- [`epok::scene_loading`](#epok-scene-loading-1) — Performs `scene loading` as part of the epok module.
- [`epok::set_active`](#epok-set-active-1) — Sets active as part of the epok module.
- [`epok::set_active_camera`](#epok-set-active-camera-1) — Sets active camera as part of the epok module.

<a id="epok-activate-texture-bank-1"></a>

## `epok::activate_texture_bank`

**Purpose.** Performs `activate texture bank` as part of the epok module.

**Exact declaration**

```cpp
void activate_texture_bank(const Texture* textures)
```

- **Declared at:** [line 197](../../../runtime/epok.hpp#L197)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `textures` | `const Texture *` | Input | Value supplied for `textures`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Texture * textures

epok::activate_texture_bank(textures);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-active-camera-1"></a>

## `epok::active_camera`

**Purpose.** Performs `active camera` as part of the epok module.

**Exact declaration**

```cpp
EntityHandle active_camera()
```

- **Declared at:** [line 195](../../../runtime/epok.hpp#L195)
- **Kind:** `function decl`

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::active_camera();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-animator-advance-1"></a>

## `epok::Animator::advance`

**Purpose.** Performs `advance` as part of the epok module.

**Exact declaration**

```cpp
void advance()
```

- **Declared at:** [line 135](../../../runtime/epok.hpp#L135)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Animator& object = /* obtain a valid instance */;

object.advance();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-animator-pause-1"></a>

## `epok::Animator::pause`

**Purpose.** Pauses pause as part of the epok module.

**Exact declaration**

```cpp
void pause()
```

- **Declared at:** [line 134](../../../runtime/epok.hpp#L134)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Animator& object = /* obtain a valid instance */;

object.pause();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-animator-play-1"></a>

## `epok::Animator::play`

**Purpose.** Starts play as part of the epok module.

**Exact declaration**

```cpp
bool play(int index,bool loop=true)
```

- **Declared at:** [line 133](../../../runtime/epok.hpp#L133)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `loop` | `bool` | Input | Value supplied for `loop`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// int index
// bool loop

epok::Animator& object = /* obtain a valid instance */;

auto result = object.play(index, loop);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-animator-resume-1"></a>

## `epok::Animator::resume`

**Purpose.** Resumes resume as part of the epok module.

**Exact declaration**

```cpp
void resume()
```

- **Declared at:** [line 134](../../../runtime/epok.hpp#L134)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Animator& object = /* obtain a valid instance */;

object.resume();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-animator-stop-1"></a>

## `epok::Animator::stop`

**Purpose.** Stops stop as part of the epok module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 134](../../../runtime/epok.hpp#L134)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Animator& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-audiosource-is-playing-1"></a>

## `epok::AudioSource::is_playing`

**Purpose.** Reports whether playing as part of the epok module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 45](../../../runtime/epok.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::AudioSource& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiosource-play-1"></a>

## `epok::AudioSource::play`

**Purpose.** Starts play as part of the epok module.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 45](../../../runtime/epok.hpp#L45)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::AudioSource& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audiosource-stop-1"></a>

## `epok::AudioSource::stop`

**Purpose.** Stops stop as part of the epok module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 45](../../../runtime/epok.hpp#L45)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::AudioSource& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-behaviour-bind-1"></a>

## `epok::Behaviour::bind`

**Purpose.** Performs `bind` as part of the epok module.

**Exact declaration**

```cpp
void bind(Entity& owner)
```

- **Declared at:** [line 218](../../../runtime/epok.hpp#L218)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Entity &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Entity & owner

epok::Behaviour& object = /* obtain a valid instance */;

object.bind(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-behaviour-entity-1"></a>

## `epok::Behaviour::entity`

**Purpose.** Performs `entity` as part of the epok module.

**Exact declaration**

```cpp
Entity& entity()
```

- **Declared at:** [line 219](../../../runtime/epok.hpp#L219)
- **Kind:** `cxx method`

**Returns.** Returns `Entity &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Behaviour& object = /* obtain a valid instance */;

auto result = object.entity();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-frame-update-1"></a>

## `epok::Behaviour::frame_update`

**Purpose.** Runs once per rendered frame even while paused.

**Details.** Use input.frame_pressed() for pause menus; update() receives buffered simulation input edges.

**Exact declaration**

```cpp
virtual void frame_update(Transform&,uint32_t elapsed_microseconds)
```

- **Declared at:** [line 238](../../../runtime/epok.hpp#L238)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Transform &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |
| `elapsed_microseconds` | `uint32_t` | Input | Value supplied for `elapsed_microseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Use input.frame_pressed() for pause menus; update() receives buffered simulation input edges.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Transform & arg1
// uint32_t elapsed_microseconds

epok::Behaviour& object = /* obtain a valid instance */;

object.frame_update(arg1, elapsed_microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-behaviour-on-destroy-1"></a>

## `epok::Behaviour::on_destroy`

**Purpose.** Performs `on destroy` as part of the epok module.

**Exact declaration**

```cpp
virtual void on_destroy()
```

- **Declared at:** [line 234](../../../runtime/epok.hpp#L234)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Behaviour& object = /* obtain a valid instance */;

object.on_destroy();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-on-disable-1"></a>

## `epok::Behaviour::on_disable`

**Purpose.** Performs `on disable` as part of the epok module.

**Exact declaration**

```cpp
virtual void on_disable()
```

- **Declared at:** [line 233](../../../runtime/epok.hpp#L233)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Behaviour& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-on-enable-1"></a>

## `epok::Behaviour::on_enable`

**Purpose.** Performs `on enable` as part of the epok module.

**Exact declaration**

```cpp
virtual void on_enable()
```

- **Declared at:** [line 232](../../../runtime/epok.hpp#L232)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::Behaviour& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-on-trigger-1"></a>

## `epok::Behaviour::on_trigger`

**Purpose.** Performs `on trigger` as part of the epok module.

**Exact declaration**

```cpp
virtual void on_trigger(EntityHandle,TriggerPhase)
```

- **Declared at:** [line 235](../../../runtime/epok.hpp#L235)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EntityHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `TriggerPhase` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle arg1
// TriggerPhase arg2

epok::Behaviour& object = /* obtain a valid instance */;

object.on_trigger(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-start-1"></a>

## `epok::Behaviour::start`

**Purpose.** Starts start as part of the epok module.

**Exact declaration**

```cpp
virtual void start(Transform&)
```

- **Declared at:** [line 231](../../../runtime/epok.hpp#L231)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Transform &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Transform & arg1

epok::Behaviour& object = /* obtain a valid instance */;

object.start(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-behaviour-timeline-sync-1"></a>

## `epok::Behaviour::timeline_sync`

**Purpose.** Typed component adapters synchronize only the reflected property being read or written.

**Details.** Reads capture live state; writes (including restoration) apply before crossing events. Ordinary behaviours need no synchronization.

**Exact declaration**

```cpp
virtual void timeline_sync(uint64_t, bool)
```

- **Declared at:** [line 223](../../../runtime/epok.hpp#L223)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Reads capture live state; writes (including restoration) apply before crossing events. Ordinary behaviours need no synchronization.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1
// bool arg2

epok::Behaviour& object = /* obtain a valid instance */;

object.timeline_sync(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-behaviour-update-1"></a>

## `epok::Behaviour::update`

**Purpose.** Updates update as part of the epok module.

**Exact declaration**

```cpp
virtual void update(Transform&, Fixed) = 0
```

- **Declared at:** [line 239](../../../runtime/epok.hpp#L239)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Transform &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `Fixed` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Transform & arg1
// Fixed arg2

epok::Behaviour& object = /* obtain a valid instance */;

object.update(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera-project-1"></a>

## `epok::camera_project`

**Purpose.** Performs `camera project` as part of the epok module.

**Exact declaration**

```cpp
bool camera_project(const Fixed* world_point,Fixed* screen_xy)
```

- **Declared at:** [line 196](../../../runtime/epok.hpp#L196)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world_point` | `const Fixed *` | Input | Value supplied for `world_point`. See the exact type and module contract. |
| `screen_xy` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `screen_xy`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * world_point
// Fixed * screen_xy

auto result = epok::camera_project(world_point, screen_xy);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collider-aabb-1"></a>

## `epok::collider_aabb`

**Purpose.** Performs `collider aabb` as part of the epok module.

**Exact declaration**

```cpp
bool collider_aabb(const Entity& entity,Aabb& output)
```

- **Declared at:** [line 206](../../../runtime/epok.hpp#L206)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const Entity &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `output` | `Aabb &` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Entity & entity
// Aabb & output

auto result = epok::collider_aabb(entity, output);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-create-entity-1"></a>

## `epok::create_entity`

**Purpose.** Creates entity as part of the epok module.

**Exact declaration**

```cpp
Entity* create_entity(const char* name,Entity* parent=nullptr)
```

- **Declared at:** [line 176](../../../runtime/epok.hpp#L176)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `parent` | `Entity *` | Input/output; inspect the function contract | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name
// Entity * parent

auto result = epok::create_entity(name, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-current-scene-1"></a>

## `epok::current_scene`

**Purpose.** Performs `current scene` as part of the epok module.

**Exact declaration**

```cpp
size_t current_scene()
```

- **Declared at:** [line 192](../../../runtime/epok.hpp#L192)
- **Kind:** `function decl`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::current_scene();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-destroy-entity-1"></a>

## `epok::destroy_entity`

**Purpose.** Destroys entity as part of the epok module.

**Exact declaration**

```cpp
bool destroy_entity(Entity* entity)
```

- **Declared at:** [line 183](../../../runtime/epok.hpp#L183)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `Entity *` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Entity * entity

auto result = epok::destroy_entity(entity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-entityhandle-get-1"></a>

## `epok::EntityHandle::get`

**Purpose.** Returns get as part of the epok module.

**Exact declaration**

```cpp
Entity* get() const
```

- **Declared at:** [line 179](../../../runtime/epok.hpp#L179)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::EntityHandle& object = /* obtain a valid instance */;

auto result = object.get();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-entityhandle-operator-bool-1"></a>

## `epok::EntityHandle::operator bool`

**Purpose.** Performs `operator  bool` as part of the epok module.

**Exact declaration**

```cpp
explicit operator bool() const
```

- **Declared at:** [line 180](../../../runtime/epok.hpp#L180)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::EntityHandle& object = /* obtain a valid instance */;

auto result = object.operator bool();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-find-entity-1"></a>

## `epok::find_entity`

**Purpose.** Finds entity as part of the epok module.

**Exact declaration**

```cpp
Entity* find_entity(const char* name)
```

- **Declared at:** [line 175](../../../runtime/epok.hpp#L175)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name

auto result = epok::find_entity(name);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-handle-1"></a>

## `epok::handle`

**Purpose.** Performs `handle` as part of the epok module.

**Exact declaration**

```cpp
EntityHandle handle(const Entity* entity)
```

- **Declared at:** [line 182](../../../runtime/epok.hpp#L182)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const Entity *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Entity * entity

auto result = epok::handle(entity);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hit-entity-1"></a>

## `epok::hit_entity`

**Purpose.** Performs `hit entity` as part of the epok module.

**Exact declaration**

```cpp
inline EntityHandle hit_entity(const SpatialHit& hit)
```

- **Declared at:** [line 202](../../../runtime/epok.hpp#L202)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `hit` | `const SpatialHit &` | Input | Value supplied for `hit`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const SpatialHit & hit

auto result = epok::hit_entity(hit);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-is-active-1"></a>

## `epok::is_active`

**Purpose.** Reports whether active as part of the epok module.

**Exact declaration**

```cpp
bool is_active(const Entity* entity)
```

- **Declared at:** [line 185](../../../runtime/epok.hpp#L185)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const Entity *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Entity * entity

auto result = epok::is_active(entity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-is-active-slot-1"></a>

## `epok::is_active_slot`

**Purpose.** Same test for a slot index; skips the pointer validation of is_active.

**Exact declaration**

```cpp
bool is_active_slot(size_t index)
```

- **Declared at:** [line 187](../../../runtime/epok.hpp#L187)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

auto result = epok::is_active_slot(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-legacyentitystorage-add-1"></a>

## `epok::LegacyEntityStorage::add`

**Purpose.** Adds add as part of the epok module.

**Exact declaration**

```cpp
template<class T>T& add()
```

- **Declared at:** [line 152](../../../runtime/epok.hpp#L152)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::LegacyEntityStorage& object = /* obtain a valid instance */;

auto result = object.add<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-legacyentitystorage-component-1"></a>

## `epok::LegacyEntityStorage::component`

**Purpose.** Performs `component` as part of the epok module.

**Exact declaration**

```cpp
template<class T>T& component()
```

- **Declared at:** [line 150](../../../runtime/epok.hpp#L150)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::LegacyEntityStorage& object = /* obtain a valid instance */;

auto result = object.component<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-legacyentitystorage-get-1"></a>

## `epok::LegacyEntityStorage::get`

**Purpose.** Returns get as part of the epok module.

**Exact declaration**

```cpp
template<class T>T* get()
```

- **Declared at:** [line 151](../../../runtime/epok.hpp#L151)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::LegacyEntityStorage& object = /* obtain a valid instance */;

auto result = object.get<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-legacyentitystorage-remove-1"></a>

## `epok::LegacyEntityStorage::remove`

**Purpose.** Removes remove as part of the epok module.

**Exact declaration**

```cpp
template<class T>void remove()
```

- **Declared at:** [line 153](../../../runtime/epok.hpp#L153)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::LegacyEntityStorage& object = /* obtain a valid instance */;

object.remove<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-legacyentitystorage-set-name-1"></a>

## `epok::LegacyEntityStorage::set_name`

**Purpose.** Sets name as part of the epok module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 149](../../../runtime/epok.hpp#L149)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const char *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * value

epok::LegacyEntityStorage& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-move-and-slide-1"></a>

## `epok::move_and_slide`

**Purpose.** Performs `move and slide` as part of the epok module.

**Exact declaration**

```cpp
MoveResult move_and_slide(Entity& entity,const Fixed* world_displacement,uint32_t mask=0xffffffffu)
```

- **Declared at:** [line 203](../../../runtime/epok.hpp#L203)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `Entity &` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |
| `world_displacement` | `const Fixed *` | Input | Value supplied for `world_displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Entity & entity
// const Fixed * world_displacement
// uint32_t mask

auto result = epok::move_and_slide(entity, world_displacement, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-overlap-1"></a>

## `epok::overlap`

**Purpose.** Performs `overlap` as part of the epok module.

**Exact declaration**

```cpp
size_t overlap(const Aabb& box,EntityHandle* output,size_t capacity,uint32_t mask=0xffffffffu,const Entity* ignore=nullptr,bool triggers=true)
```

- **Declared at:** [line 200](../../../runtime/epok.hpp#L200)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `output` | `EntityHandle *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const Entity *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb & box
// EntityHandle * output
// size_t capacity
// uint32_t mask
// const Entity * ignore
// bool triggers

auto result = epok::overlap(box, output, capacity, mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-query-ground-1"></a>

## `epok::query_ground`

**Purpose.** Performs `query ground` as part of the epok module.

**Exact declaration**

```cpp
SpatialHit query_ground(const Entity& entity,Fixed distance,uint32_t mask=0xffffffffu)
```

- **Declared at:** [line 201](../../../runtime/epok.hpp#L201)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const Entity &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Entity & entity
// Fixed distance
// uint32_t mask

auto result = epok::query_ground(entity, distance, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-raycast-1"></a>

## `epok::raycast`

**Purpose.** Spatial queries use world coordinates.

**Details.** Ray displacement defines a finite segment.

**Exact declaration**

```cpp
SpatialHit raycast(const Fixed* origin,const Fixed* displacement,uint32_t mask=0xffffffffu,const Entity* ignore=nullptr,bool triggers=false)
```

- **Declared at:** [line 199](../../../runtime/epok.hpp#L199)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Fixed *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const Entity *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit`. Check the purpose and failure notes before using the value.

**Use it when.** Ray displacement defines a finite segment.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * origin
// const Fixed * displacement
// uint32_t mask
// const Entity * ignore
// bool triggers

auto result = epok::raycast(origin, displacement, mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-remove-runtime-owner-1"></a>

## `epok::remove_runtime_owner`

**Purpose.** Removes runtime owner as part of the epok module.

**Exact declaration**

```cpp
void remove_runtime_owner(size_t index)
```

- **Declared at:** [line 208](../../../runtime/epok.hpp#L208)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::remove_runtime_owner(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-request-scene-1"></a>

## `epok::request_scene`

**Purpose.** Requests scene as part of the epok module.

**Exact declaration**

```cpp
bool request_scene(const char* name)
```

- **Declared at:** [line 189](../../../runtime/epok.hpp#L189)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name

auto result = epok::request_scene(name);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-request-scene-2"></a>

## `epok::request_scene`

**Purpose.** Requests scene as part of the epok module.

**Exact declaration**

```cpp
bool request_scene(const char* name,const TransitionOptions& options)
```

- **Declared at:** [line 191](../../../runtime/epok.hpp#L191)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `options` | `const TransitionOptions &` | Input | Value supplied for `options`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name
// const TransitionOptions & options

auto result = epok::request_scene(name, options);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-request-scene-3"></a>

## `epok::request_scene`

**Purpose.** Requests scene as part of the epok module.

**Exact declaration**

```cpp
bool request_scene(size_t index)
```

- **Declared at:** [line 188](../../../runtime/epok.hpp#L188)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

auto result = epok::request_scene(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-request-scene-4"></a>

## `epok::request_scene`

**Purpose.** Requests scene as part of the epok module.

**Exact declaration**

```cpp
bool request_scene(size_t index,const TransitionOptions& options)
```

- **Declared at:** [line 190](../../../runtime/epok.hpp#L190)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `options` | `const TransitionOptions &` | Input | Value supplied for `options`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const TransitionOptions & options

auto result = epok::request_scene(index, options);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-reset-motion-interpolation-1"></a>

## `epok::reset_motion_interpolation`

**Purpose.** Call after an intentional teleport/cut to snap visual position history.

**Exact declaration**

```cpp
void reset_motion_interpolation()
```

- **Declared at:** [line 205](../../../runtime/epok.hpp#L205)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::reset_motion_interpolation();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-reset-runtime-services-1"></a>

## `epok::reset_runtime_services`

**Purpose.** Resets runtime services as part of the epok module.

**Exact declaration**

```cpp
void reset_runtime_services()
```

- **Declared at:** [line 207](../../../runtime/epok.hpp#L207)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::reset_runtime_services();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-loading-1"></a>

## `epok::scene_loading`

**Purpose.** Performs `scene loading` as part of the epok module.

**Exact declaration**

```cpp
bool scene_loading()
```

- **Declared at:** [line 193](../../../runtime/epok.hpp#L193)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::scene_loading();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-set-active-1"></a>

## `epok::set_active`

**Purpose.** Sets active as part of the epok module.

**Exact declaration**

```cpp
bool set_active(Entity* entity,bool active)
```

- **Declared at:** [line 184](../../../runtime/epok.hpp#L184)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `Entity *` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Entity * entity
// bool active

auto result = epok::set_active(entity, active);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-set-active-camera-1"></a>

## `epok::set_active_camera`

**Purpose.** Sets active camera as part of the epok module.

**Exact declaration**

```cpp
bool set_active_camera(Entity* camera)
```

- **Declared at:** [line 194](../../../runtime/epok.hpp#L194)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `Entity *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// Entity * camera

auto result = epok::set_active_camera(camera);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
