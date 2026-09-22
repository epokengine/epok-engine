# Epok API: Epok

> **Header:** `"epok.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/epok.hpp)

This module covers the epok module. It documents 53 public callables declared directly in this header.

## Declared types

`epok::Aabb`, `epok::Actor`, `epok::ActorData`, `epok::AnimationClip`, `epok::Animator`, `epok::AudioSource`, `epok::BlobShadow`, `epok::Bone`, `epok::BonePose`, `epok::BoneSample`, `epok::BoneTrack`, `epok::CameraSettings`, `epok::Canvas`, `epok::ChunkVisibility`, `epok::Collider`, `epok::CoordinateSpace`, `epok::DataHandle`, `epok::Fixed`, `epok::Image`, `epok::LayoutContainer`, `epok::LayoutElement`, `epok::LayoutKind`, `epok::Light`, `epok::LightingEnvironment`, `epok::LightingStats`, `epok::LightMode`, `epok::LightType`, `epok::Material`, `epok::MaterialSnapshot`, `epok::MeshDataState`, `epok::MeshGeometry`, `epok::MeshLighting`, `epok::MeshQuad`, `epok::MeshStats`, `epok::MeshVertexError`, `epok::MeshVertexSample`, `epok::MoveResult`, `epok::MusicStats`, `epok::PaletteAnimationState`, `epok::ParticleEmitterState`, `epok::PerformanceStats`, `epok::PoseKind`, `epok::ProgressBar`, `epok::RaycastQuery`, `epok::ReceiveLighting`, `epok::RectTransform`, `epok::SkeletalError`, `epok::SkeletalMesh`, `epok::SkeletalPlaybackState`, `epok::SkeletalQueryStats`, `epok::SkeletalStorage`, `epok::SpatialHit`, `epok::SpritePlaybackState`, `epok::Transform`, `epok::VertexFrame`, `epok::VertexIndexBatch4`, `epok::VertexSample`, `epok::VertexSamples4`, `epok::WorldAffineSample`

## Callable index

- [`epok::activate_texture_bank`](#epok-activate-texture-bank-1) — Performs `activate texture bank` as part of the epok module.
- [`epok::active_camera`](#epok-active-camera-1) — Performs `active camera` as part of the epok module.
- [`epok::ActorData::add`](#epok-actordata-add-1) — Adds add as part of the epok module.
- [`epok::ActorData::component`](#epok-actordata-component-1) — Performs `component` as part of the epok module.
- [`epok::ActorData::get`](#epok-actordata-get-1) — Returns get as part of the epok module.
- [`epok::ActorData::remove`](#epok-actordata-remove-1) — Removes remove as part of the epok module.
- [`epok::ActorData::set_name`](#epok-actordata-set-name-1) — Sets name as part of the epok module.
- [`epok::allocate_actor_data`](#epok-allocate-actor-data-1) — Performs `allocate actor data` as part of the epok module.
- [`epok::Animator::advance`](#epok-animator-advance-1) — Performs `advance` as part of the epok module.
- [`epok::Animator::pause`](#epok-animator-pause-1) — Pauses pause as part of the epok module.
- [`epok::Animator::play`](#epok-animator-play-1) — Starts play as part of the epok module.
- [`epok::Animator::resume`](#epok-animator-resume-1) — Resumes resume as part of the epok module.
- [`epok::Animator::stop`](#epok-animator-stop-1) — Stops stop as part of the epok module.
- [`epok::AudioSource::is_playing`](#epok-audiosource-is-playing-1) — Reports whether playing as part of the epok module.
- [`epok::AudioSource::play`](#epok-audiosource-play-1) — Starts play as part of the epok module.
- [`epok::AudioSource::stop`](#epok-audiosource-stop-1) — Stops stop as part of the epok module.
- [`epok::camera_project`](#epok-camera-project-1) — Performs `camera project` as part of the epok module.
- [`epok::collider_aabb`](#epok-collider-aabb-1) — Performs `collider aabb` as part of the epok module.
- [`epok::current_scene`](#epok-current-scene-1) — Performs `current scene` as part of the epok module.
- [`epok::DataHandle::get`](#epok-datahandle-get-1) — Returns get as part of the epok module.
- [`epok::DataHandle::operator bool`](#epok-datahandle-operator-bool-1) — Performs `operator  bool` as part of the epok module.
- [`epok::destroy_actor_data`](#epok-destroy-actor-data-1) — Destroys actor data as part of the epok module.
- [`epok::find_actor_data`](#epok-find-actor-data-1) — Finds actor data as part of the epok module.
- [`epok::gameplay_world_affine`](#epok-gameplay-world-affine-1) — Performs `gameplay world affine` as part of the epok module.
- [`epok::handle`](#epok-handle-1) — Performs `handle` as part of the epok module.
- [`epok::hit_entity`](#epok-hit-entity-1) — Performs `hit entity` as part of the epok module.
- [`epok::is_active`](#epok-is-active-1) — Reports whether active as part of the epok module.
- [`epok::is_active_slot`](#epok-is-active-slot-1) — Same test for a slot index; skips the pointer validation of is_active.
- [`epok::mesh_geometry_state`](#epok-mesh-geometry-state-1) — These are public gameplay entry points invoked from generated script translation units.
- [`epok::move_and_slide`](#epok-move-and-slide-1) — Performs `move and slide` as part of the epok module.
- [`epok::overlap`](#epok-overlap-1) — Performs `overlap` as part of the epok module.
- [`epok::query_ground`](#epok-query-ground-1) — Performs `query ground` as part of the epok module.
- [`epok::raycast`](#epok-raycast-1) — Spatial queries use world coordinates.
- [`epok::raycast_batch`](#epok-raycast-batch-1) — One synchronized world snapshot for independent queries.
- [`epok::remove_runtime_owner`](#epok-remove-runtime-owner-1) — Removes runtime owner as part of the epok module.
- [`epok::request_mesh_geometry`](#epok-request-mesh-geometry-1) — Requests mesh geometry as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-1) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-2) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-3) — Requests scene as part of the epok module.
- [`epok::request_scene`](#epok-request-scene-4) — Requests scene as part of the epok module.
- [`epok::reset_motion_interpolation`](#epok-reset-motion-interpolation-1) — Call after an intentional teleport/cut to snap visual position history.
- [`epok::reset_runtime_services`](#epok-reset-runtime-services-1) — Resets runtime services as part of the epok module.
- [`epok::sample_mesh_vertex`](#epok-sample-mesh-vertex-1) — Performs `sample mesh vertex` as part of the epok module.
- [`epok::scene_loading`](#epok-scene-loading-1) — Performs `scene loading` as part of the epok module.
- [`epok::scene_rejected_count`](#epok-scene-rejected-count-1) — Performs `scene rejected count` as part of the epok module.
- [`epok::scene_transition_count`](#epok-scene-transition-count-1) — Performs `scene transition count` as part of the epok module.
- [`epok::scene_waiting`](#epok-scene-waiting-1) — Performs `scene waiting` as part of the epok module.
- [`epok::set_active`](#epok-set-active-1) — Sets active as part of the epok module.
- [`epok::set_active_camera`](#epok-set-active-camera-1) — Sets active camera as part of the epok module.
- [`epok::skeletal_sample_bone`](#epok-skeletal-sample-bone-1) — Performs `skeletal sample bone` as part of the epok module.
- [`epok::skeletal_sample_vertex`](#epok-skeletal-sample-vertex-1) — Performs `skeletal sample vertex` as part of the epok module.
- [`epok::skeletal_sample_vertices`](#epok-skeletal-sample-vertices-1) — Performs `skeletal sample vertices` as part of the epok module.
- [`epok::skeletal_world_point`](#epok-skeletal-world-point-1) — Performs `skeletal world point` as part of the epok module.

<a id="epok-activate-texture-bank-1"></a>

## `epok::activate_texture_bank`

**Purpose.** Performs `activate texture bank` as part of the epok module.

**Exact declaration**

```cpp
void activate_texture_bank(const Texture* textures)
```

- **Declared at:** [line 296](../../../runtime/epok.hpp#L296)
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
DataHandle active_camera()
```

- **Declared at:** [line 294](../../../runtime/epok.hpp#L294)
- **Kind:** `function decl`

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::active_camera();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-actordata-add-1"></a>

## `epok::ActorData::add`

**Purpose.** Adds add as part of the epok module.

**Exact declaration**

```cpp
template<class T>T& add()
```

- **Declared at:** [line 247](../../../runtime/epok.hpp#L247)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::ActorData& object = /* obtain a valid instance */;

auto result = object.add<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-actordata-component-1"></a>

## `epok::ActorData::component`

**Purpose.** Performs `component` as part of the epok module.

**Exact declaration**

```cpp
template<class T>T& component()
```

- **Declared at:** [line 245](../../../runtime/epok.hpp#L245)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::ActorData& object = /* obtain a valid instance */;

auto result = object.component<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-actordata-get-1"></a>

## `epok::ActorData::get`

**Purpose.** Returns get as part of the epok module.

**Exact declaration**

```cpp
template<class T>T* get()
```

- **Declared at:** [line 246](../../../runtime/epok.hpp#L246)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::ActorData& object = /* obtain a valid instance */;

auto result = object.get<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-actordata-remove-1"></a>

## `epok::ActorData::remove`

**Purpose.** Removes remove as part of the epok module.

**Exact declaration**

```cpp
template<class T>void remove()
```

- **Declared at:** [line 248](../../../runtime/epok.hpp#L248)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

epok::ActorData& object = /* obtain a valid instance */;

object.remove<T>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-actordata-set-name-1"></a>

## `epok::ActorData::set_name`

**Purpose.** Sets name as part of the epok module.

**Exact declaration**

```cpp
void set_name(const char* value)
```

- **Declared at:** [line 244](../../../runtime/epok.hpp#L244)
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

epok::ActorData& object = /* obtain a valid instance */;

object.set_name(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-allocate-actor-data-1"></a>

## `epok::allocate_actor_data`

**Purpose.** Performs `allocate actor data` as part of the epok module.

**Exact declaration**

```cpp
ActorData* allocate_actor_data(const char* name,ActorData* parent=nullptr)
```

- **Declared at:** [line 272](../../../runtime/epok.hpp#L272)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `parent` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name
// ActorData * parent

auto result = epok::allocate_actor_data(name, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-animator-advance-1"></a>

## `epok::Animator::advance`

**Purpose.** Performs `advance` as part of the epok module.

**Exact declaration**

```cpp
void advance()
```

- **Declared at:** [line 224](../../../runtime/epok.hpp#L224)
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

- **Declared at:** [line 223](../../../runtime/epok.hpp#L223)
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

- **Declared at:** [line 222](../../../runtime/epok.hpp#L222)
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

- **Declared at:** [line 223](../../../runtime/epok.hpp#L223)
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

- **Declared at:** [line 223](../../../runtime/epok.hpp#L223)
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

- **Declared at:** [line 53](../../../runtime/epok.hpp#L53)
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

- **Declared at:** [line 53](../../../runtime/epok.hpp#L53)
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

- **Declared at:** [line 53](../../../runtime/epok.hpp#L53)
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

<a id="epok-camera-project-1"></a>

## `epok::camera_project`

**Purpose.** Performs `camera project` as part of the epok module.

**Exact declaration**

```cpp
bool camera_project(const Fixed* world_point,Fixed* screen_xy)
```

- **Declared at:** [line 295](../../../runtime/epok.hpp#L295)
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
bool collider_aabb(const ActorData& entity,Aabb& output)
```

- **Declared at:** [line 308](../../../runtime/epok.hpp#L308)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `output` | `Aabb &` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & entity
// Aabb & output

auto result = epok::collider_aabb(entity, output);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-current-scene-1"></a>

## `epok::current_scene`

**Purpose.** Performs `current scene` as part of the epok module.

**Exact declaration**

```cpp
size_t current_scene()
```

- **Declared at:** [line 288](../../../runtime/epok.hpp#L288)
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

<a id="epok-datahandle-get-1"></a>

## `epok::DataHandle::get`

**Purpose.** Returns get as part of the epok module.

**Exact declaration**

```cpp
ActorData* get() const
```

- **Declared at:** [line 275](../../../runtime/epok.hpp#L275)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::DataHandle& object = /* obtain a valid instance */;

auto result = object.get();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-datahandle-operator-bool-1"></a>

## `epok::DataHandle::operator bool`

**Purpose.** Performs `operator  bool` as part of the epok module.

**Exact declaration**

```cpp
explicit operator bool() const
```

- **Declared at:** [line 276](../../../runtime/epok.hpp#L276)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

epok::DataHandle& object = /* obtain a valid instance */;

auto result = object.operator bool();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-destroy-actor-data-1"></a>

## `epok::destroy_actor_data`

**Purpose.** Destroys actor data as part of the epok module.

**Exact declaration**

```cpp
bool destroy_actor_data(ActorData* entity)
```

- **Declared at:** [line 279](../../../runtime/epok.hpp#L279)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * entity

auto result = epok::destroy_actor_data(entity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-find-actor-data-1"></a>

## `epok::find_actor_data`

**Purpose.** Finds actor data as part of the epok module.

**Exact declaration**

```cpp
ActorData* find_actor_data(const char* name)
```

- **Declared at:** [line 271](../../../runtime/epok.hpp#L271)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const char * name

auto result = epok::find_actor_data(name);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-world-affine-1"></a>

## `epok::gameplay_world_affine`

**Purpose.** Performs `gameplay world affine` as part of the epok module.

**Exact declaration**

```cpp
WorldAffineSample gameplay_world_affine(const ActorData* entity)
```

- **Declared at:** [line 316](../../../runtime/epok.hpp#L316)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `WorldAffineSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity

auto result = epok::gameplay_world_affine(entity);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-handle-1"></a>

## `epok::handle`

**Purpose.** Performs `handle` as part of the epok module.

**Exact declaration**

```cpp
DataHandle handle(const ActorData* entity)
```

- **Declared at:** [line 278](../../../runtime/epok.hpp#L278)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity

auto result = epok::handle(entity);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hit-entity-1"></a>

## `epok::hit_entity`

**Purpose.** Performs `hit entity` as part of the epok module.

**Exact declaration**

```cpp
inline DataHandle hit_entity(const SpatialHit& hit)
```

- **Declared at:** [line 304](../../../runtime/epok.hpp#L304)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `hit` | `const SpatialHit &` | Input | Value supplied for `hit`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

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
bool is_active(const ActorData* entity)
```

- **Declared at:** [line 281](../../../runtime/epok.hpp#L281)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity

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

- **Declared at:** [line 283](../../../runtime/epok.hpp#L283)
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

<a id="epok-mesh-geometry-state-1"></a>

## `epok::mesh_geometry_state`

**Purpose.** These are public gameplay entry points invoked from generated script translation units.

**Details.** `used` keeps their weak inline definitions linkable even when main.cpp itself has no direct call to a particular query.

**Exact declaration**

```cpp
MeshDataState mesh_geometry_state(const MeshGeometry* geometry)
```

- **Declared at:** [line 313](../../../runtime/epok.hpp#L313)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |

**Returns.** Returns `MeshDataState`. Check the purpose and failure notes before using the value.

**Use it when.** `used` keeps their weak inline definitions linkable even when main.cpp itself has no direct call to a particular query.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * geometry

auto result = epok::mesh_geometry_state(geometry);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-move-and-slide-1"></a>

## `epok::move_and_slide`

**Purpose.** Performs `move and slide` as part of the epok module.

**Exact declaration**

```cpp
MoveResult move_and_slide(ActorData& entity,const Fixed* world_displacement,uint32_t mask=0xffffffffu)
```

- **Declared at:** [line 305](../../../runtime/epok.hpp#L305)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |
| `world_displacement` | `const Fixed *` | Input | Value supplied for `world_displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & entity
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
size_t overlap(const Aabb& box,DataHandle* output,size_t capacity,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=true)
```

- **Declared at:** [line 302](../../../runtime/epok.hpp#L302)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `output` | `DataHandle *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb & box
// DataHandle * output
// size_t capacity
// uint32_t mask
// const ActorData * ignore
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
SpatialHit query_ground(const ActorData& entity,Fixed distance,uint32_t mask=0xffffffffu)
```

- **Declared at:** [line 303](../../../runtime/epok.hpp#L303)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & entity
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
SpatialHit raycast(const Fixed* origin,const Fixed* displacement,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=false)
```

- **Declared at:** [line 298](../../../runtime/epok.hpp#L298)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Fixed *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
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
// const ActorData * ignore
// bool triggers

auto result = epok::raycast(origin, displacement, mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-raycast-batch-1"></a>

## `epok::raycast_batch`

**Purpose.** One synchronized world snapshot for independent queries.

**Details.** Apply gameplay mutations AFTER this call; results[i] belongs to queries[i]. No allocations.

**Exact declaration**

```cpp
void raycast_batch(const RaycastQuery* queries,SpatialHit* results,size_t count,uint32_t mask=0xffffffffu,const ActorData* ignore=nullptr,bool triggers=false)
```

- **Declared at:** [line 301](../../../runtime/epok.hpp#L301)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `queries` | `const RaycastQuery *` | Input | Value supplied for `queries`. See the exact type and module contract. |
| `results` | `SpatialHit *` | Input/output; inspect the function contract | Value supplied for `results`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Apply gameplay mutations AFTER this call; results[i] belongs to queries[i]. No allocations.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const RaycastQuery * queries
// SpatialHit * results
// size_t count
// uint32_t mask
// const ActorData * ignore
// bool triggers

epok::raycast_batch(queries, results, count, mask, ignore, triggers);
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

- **Declared at:** [line 318](../../../runtime/epok.hpp#L318)
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

<a id="epok-request-mesh-geometry-1"></a>

## `epok::request_mesh_geometry`

**Purpose.** Requests mesh geometry as part of the epok module.

**Exact declaration**

```cpp
bool request_mesh_geometry(const MeshGeometry* geometry)
```

- **Declared at:** [line 314](../../../runtime/epok.hpp#L314)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * geometry

auto result = epok::request_mesh_geometry(geometry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-request-scene-1"></a>

## `epok::request_scene`

**Purpose.** Requests scene as part of the epok module.

**Exact declaration**

```cpp
bool request_scene(const char* name)
```

- **Declared at:** [line 285](../../../runtime/epok.hpp#L285)
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

- **Declared at:** [line 287](../../../runtime/epok.hpp#L287)
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

- **Declared at:** [line 284](../../../runtime/epok.hpp#L284)
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

- **Declared at:** [line 286](../../../runtime/epok.hpp#L286)
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

- **Declared at:** [line 307](../../../runtime/epok.hpp#L307)
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

- **Declared at:** [line 317](../../../runtime/epok.hpp#L317)
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

<a id="epok-sample-mesh-vertex-1"></a>

## `epok::sample_mesh_vertex`

**Purpose.** Performs `sample mesh vertex` as part of the epok module.

**Exact declaration**

```cpp
MeshVertexSample sample_mesh_vertex(const ActorData* entity,uint32_t vertex,CoordinateSpace space)
```

- **Declared at:** [line 315](../../../runtime/epok.hpp#L315)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `MeshVertexSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity
// uint32_t vertex
// CoordinateSpace space

auto result = epok::sample_mesh_vertex(entity, vertex, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scene-loading-1"></a>

## `epok::scene_loading`

**Purpose.** Performs `scene loading` as part of the epok module.

**Exact declaration**

```cpp
bool scene_loading()
```

- **Declared at:** [line 289](../../../runtime/epok.hpp#L289)
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

<a id="epok-scene-rejected-count-1"></a>

## `epok::scene_rejected_count`

**Purpose.** Performs `scene rejected count` as part of the epok module.

**Exact declaration**

```cpp
uint32_t scene_rejected_count()
```

- **Declared at:** [line 291](../../../runtime/epok.hpp#L291)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::scene_rejected_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-transition-count-1"></a>

## `epok::scene_transition_count`

**Purpose.** Performs `scene transition count` as part of the epok module.

**Exact declaration**

```cpp
uint32_t scene_transition_count()
```

- **Declared at:** [line 290](../../../runtime/epok.hpp#L290)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::scene_transition_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-waiting-1"></a>

## `epok::scene_waiting`

**Purpose.** Performs `scene waiting` as part of the epok module.

**Exact declaration**

```cpp
bool scene_waiting()
```

- **Declared at:** [line 292](../../../runtime/epok.hpp#L292)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

auto result = epok::scene_waiting();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-set-active-1"></a>

## `epok::set_active`

**Purpose.** Sets active as part of the epok module.

**Exact declaration**

```cpp
bool set_active(ActorData* entity,bool active)
```

- **Declared at:** [line 280](../../../runtime/epok.hpp#L280)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `entity`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * entity
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
bool set_active_camera(ActorData* camera)
```

- **Declared at:** [line 293](../../../runtime/epok.hpp#L293)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * camera

auto result = epok::set_active_camera(camera);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-sample-bone-1"></a>

## `epok::skeletal_sample_bone`

**Purpose.** Performs `skeletal sample bone` as part of the epok module.

**Exact declaration**

```cpp
BoneSample skeletal_sample_bone(const ActorData* entity,uint32_t bone,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 311](../../../runtime/epok.hpp#L311)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `bone` | `uint32_t` | Input | Value supplied for `bone`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `BoneSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity
// uint32_t bone
// PoseKind pose
// CoordinateSpace space

auto result = epok::skeletal_sample_bone(entity, bone, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-sample-vertex-1"></a>

## `epok::skeletal_sample_vertex`

**Purpose.** Performs `skeletal sample vertex` as part of the epok module.

**Exact declaration**

```cpp
VertexSample skeletal_sample_vertex(const ActorData* entity,uint32_t vertex,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 309](../../../runtime/epok.hpp#L309)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `VertexSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity
// uint32_t vertex
// PoseKind pose
// CoordinateSpace space

auto result = epok::skeletal_sample_vertex(entity, vertex, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-sample-vertices-1"></a>

## `epok::skeletal_sample_vertices`

**Purpose.** Performs `skeletal sample vertices` as part of the epok module.

**Exact declaration**

```cpp
VertexSamples4 skeletal_sample_vertices(const ActorData* entity,VertexIndexBatch4 indices,PoseKind pose,CoordinateSpace space)
```

- **Declared at:** [line 310](../../../runtime/epok.hpp#L310)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `indices` | `VertexIndexBatch4` | Input | Value supplied for `indices`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `VertexSamples4`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity
// VertexIndexBatch4 indices
// PoseKind pose
// CoordinateSpace space

auto result = epok::skeletal_sample_vertices(entity, indices, pose, space);
```

**Why choose it.** It provides direct, allocation-conscious access to the epok module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-world-point-1"></a>

## `epok::skeletal_world_point`

**Purpose.** Performs `skeletal world point` as part of the epok module.

**Exact declaration**

```cpp
bool skeletal_world_point(const ActorData& entity,const Fixed* model,Fixed* world)
```

- **Declared at:** [line 312](../../../runtime/epok.hpp#L312)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `model` | `const Fixed *` | Input | Value supplied for `model`. See the exact type and module contract. |
| `world` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the epok module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "epok.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & entity
// const Fixed * model
// Fixed * world

auto result = epok::skeletal_world_point(entity, model, world);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
