# Epok API: Camera Rig

> **Header:** `"camera_rig.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/camera_rig.hpp)

This module covers camera selection, projection and shared render resources. It documents 37 public callables declared directly in this header.

## Declared types

`epok::camera_rig::ArmState`, `epok::camera_rig::ArmTuning`, `epok::camera_rig::WorldQueries`, `epok::CameraOrbitInputComponent`, `epok::CameraRig3D`, `epok::CameraRigSample`, `epok::CameraRigStatus`, `epok::FirstPersonCamera`, `epok::IsometricCamera`, `epok::SpringArmComponent`, `epok::SpringArmSample`, `epok::ThirdPersonCamera`, `epok::TopDownCamera`

## Callable index

- [`epok::camera_rig::boom`](#epok-camera-rig-boom-1) — Performs `boom` as part of camera selection, projection and shared render resources.
- [`epok::camera_rig::bounded`](#epok-camera-rig-bounded-1) — Performs `bounded` as part of camera selection, projection and shared render resources.
- [`epok::camera_rig::solve`](#epok-camera-rig-solve-1) — Performs `solve` as part of camera selection, projection and shared render resources.
- [`epok::camera_rig::solve`](#epok-camera-rig-solve-2) — Performs `solve` as part of camera selection, projection and shared render resources.
- [`epok::camera_rig::WorldQueries::cast`](#epok-camera-rig-worldqueries-cast-1) — Performs `cast` as part of camera selection, projection and shared render resources.
- [`epok::CameraOrbitInputComponent::class_id`](#epok-cameraorbitinputcomponent-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::CameraOrbitInputComponent::tick`](#epok-cameraorbitinputcomponent-tick-1) — Performs `tick` as part of camera selection, projection and shared render resources.
- [`epok::CameraOrbitInputComponent::update`](#epok-cameraorbitinputcomponent-update-1) — Updates update as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::add_look`](#epok-camerarig3d-add-look-1) — Adds look as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::begin_play`](#epok-camerarig3d-begin-play-1) — Begins play as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::calculate_view`](#epok-camerarig3d-calculate-view-1) — Performs `calculate view` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::class_id`](#epok-camerarig3d-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::end_play`](#epok-camerarig3d-end-play-1) — Ends play as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::make_active`](#epok-camerarig3d-make-active-1) — Performs `make active` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::on_disable`](#epok-camerarig3d-on-disable-1) — Performs `on disable` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::refresh`](#epok-camerarig3d-refresh-1) — Performs `refresh` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::sample`](#epok-camerarig3d-sample-1) — Performs `sample` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::set_angles`](#epok-camerarig3d-set-angles-1) — Sets angles as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::snap`](#epok-camerarig3d-snap-1) — Performs `snap` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::tick`](#epok-camerarig3d-tick-1) — Performs `tick` as part of camera selection, projection and shared render resources.
- [`epok::CameraRig3D::view_updated`](#epok-camerarig3d-view-updated-1) — Performs `view updated` as part of camera selection, projection and shared render resources.
- [`epok::FirstPersonCamera::class_id`](#epok-firstpersoncamera-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::FirstPersonCamera::FirstPersonCamera`](#epok-firstpersoncamera-firstpersoncamera-1) — Constructs `epok::FirstPersonCamera` for camera selection, projection and shared render resources.
- [`epok::IsometricCamera::class_id`](#epok-isometriccamera-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::IsometricCamera::IsometricCamera`](#epok-isometriccamera-isometriccamera-1) — Constructs `epok::IsometricCamera` for camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::begin_play`](#epok-springarmcomponent-begin-play-1) — Begins play as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::class_id`](#epok-springarmcomponent-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::end_play`](#epok-springarmcomponent-end-play-1) — Ends play as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::on_disable`](#epok-springarmcomponent-on-disable-1) — Performs `on disable` as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::on_enable`](#epok-springarmcomponent-on-enable-1) — Performs `on enable` as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::reset`](#epok-springarmcomponent-reset-1) — Resets reset as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::sample`](#epok-springarmcomponent-sample-1) — Performs `sample` as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::solve`](#epok-springarmcomponent-solve-1) — Performs `solve` as part of camera selection, projection and shared render resources.
- [`epok::SpringArmComponent::solve_shared`](#epok-springarmcomponent-solve-shared-1) — Performs `solve shared` as part of camera selection, projection and shared render resources.
- [`epok::ThirdPersonCamera::class_id`](#epok-thirdpersoncamera-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::TopDownCamera::class_id`](#epok-topdowncamera-class-id-1) — Performs `class id` as part of camera selection, projection and shared render resources.
- [`epok::TopDownCamera::TopDownCamera`](#epok-topdowncamera-topdowncamera-1) — Constructs `epok::TopDownCamera` for camera selection, projection and shared render resources.

<a id="epok-camera-rig-boom-1"></a>

## `epok::camera_rig::boom`

**Purpose.** Performs `boom` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline GameplayVector3 boom(Fixed yaw,Fixed pitch,Fixed distance)
```

- **Declared at:** [line 24](../../../runtime/camera_rig.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `yaw` | `Fixed` | Input | Value supplied for `yaw`. See the exact type and module contract. |
| `pitch` | `Fixed` | Input | Value supplied for `pitch`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed yaw
// Fixed pitch
// Fixed distance

auto result = epok::camera_rig::boom(yaw, pitch, distance);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera-rig-bounded-1"></a>

## `epok::camera_rig::bounded`

**Purpose.** Performs `bounded` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline bool bounded(Fixed value,int32_t maximum)
```

- **Declared at:** [line 23](../../../runtime/camera_rig.hpp#L23)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `maximum` | `int32_t` | Input | Value supplied for `maximum`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// int32_t maximum

auto result = epok::camera_rig::bounded(value, maximum);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camera-rig-solve-1"></a>

## `epok::camera_rig::solve`

**Purpose.** Performs `solve` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline SpringArmSample solve(ArmState& state,const ArmTuning& tuning,GameplayVector3 origin,GameplayVector3 offset,const ActorData* ignore,Fixed dt)
```

- **Declared at:** [line 64](../../../runtime/camera_rig.hpp#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `ArmState &` | Input/output; inspect the function contract | Value supplied for `state`. See the exact type and module contract. |
| `tuning` | `const ArmTuning &` | Input | Value supplied for `tuning`. See the exact type and module contract. |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `offset` | `GameplayVector3` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `SpringArmSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// ArmState & state
// const ArmTuning & tuning
// GameplayVector3 origin
// GameplayVector3 offset
// const ActorData * ignore
// Fixed dt

auto result = epok::camera_rig::solve(state, tuning, origin, offset, ignore, dt);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera-rig-solve-2"></a>

## `epok::camera_rig::solve`

**Purpose.** Performs `solve` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
template<class Queries> inline SpringArmSample solve(ArmState& state,const ArmTuning& tuning,GameplayVector3 origin,GameplayVector3 offset,const ActorData* ignore,Fixed dt,Queries& queries)
```

- **Declared at:** [line 39](../../../runtime/camera_rig.hpp#L39)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `ArmState &` | Input/output; inspect the function contract | Value supplied for `state`. See the exact type and module contract. |
| `tuning` | `const ArmTuning &` | Input | Value supplied for `tuning`. See the exact type and module contract. |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `offset` | `GameplayVector3` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `queries` | `Queries &` | Input/output; inspect the function contract | Value supplied for `queries`. See the exact type and module contract. |

**Returns.** Returns `SpringArmSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Queries

// Assume these named values have been initialized with valid data:
// ArmState & state
// const ArmTuning & tuning
// GameplayVector3 origin
// GameplayVector3 offset
// const ActorData * ignore
// Fixed dt
// Queries & queries

epok::camera_rig& object = /* obtain a valid instance */;

auto result = object.solve<Queries>(state, tuning, origin, offset, ignore, dt, queries);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera-rig-worldqueries-cast-1"></a>

## `epok::camera_rig::WorldQueries::cast`

**Purpose.** Performs `cast` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
SpatialHit cast(const Fixed* origin,const Fixed* displacement,uint32_t mask,const ActorData* ignore)
```

- **Declared at:** [line 36](../../../runtime/camera_rig.hpp#L36)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Fixed *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `const ActorData *` | Input | Value supplied for `ignore`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * origin
// const Fixed * displacement
// uint32_t mask
// const ActorData * ignore

epok::camera_rig::WorldQueries& object = /* obtain a valid instance */;

auto result = object.cast(origin, displacement, mask, ignore);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-cameraorbitinputcomponent-class-id-1"></a>

## `epok::CameraOrbitInputComponent::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 233](../../../runtime/camera_rig.hpp#L233)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraOrbitInputComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-cameraorbitinputcomponent-tick-1"></a>

## `epok::CameraOrbitInputComponent::tick`

**Purpose.** Performs `tick` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 250](../../../runtime/camera_rig.hpp#L250)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CameraOrbitInputComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-cameraorbitinputcomponent-update-1"></a>

## `epok::CameraOrbitInputComponent::update`

**Purpose.** Updates update as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
CameraRigStatus update(Fixed delta_seconds)
```

- **Declared at:** [line 240](../../../runtime/camera_rig.hpp#L240)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CameraRigStatus`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CameraOrbitInputComponent& object = /* obtain a valid instance */;

auto result = object.update(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-add-look-1"></a>

## `epok::CameraRig3D::add_look`

**Purpose.** Adds look as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
bool add_look(GameplayVector2 delta_degrees)
```

- **Declared at:** [line 144](../../../runtime/camera_rig.hpp#L144)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_degrees` | `GameplayVector2` | Input | Value supplied for `delta_degrees`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector2 delta_degrees

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.add_look(delta_degrees);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camerarig3d-begin-play-1"></a>

## `epok::CameraRig3D::begin_play`

**Purpose.** Begins play as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 200](../../../runtime/camera_rig.hpp#L200)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-calculate-view-1"></a>

## `epok::CameraRig3D::calculate_view`

**Purpose.** Performs `calculate view` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
virtual CameraRigSample calculate_view(Fixed delta_seconds)
```

- **Declared at:** [line 149](../../../runtime/camera_rig.hpp#L149)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CameraRigSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.calculate_view(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-class-id-1"></a>

## `epok::CameraRig3D::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 125](../../../runtime/camera_rig.hpp#L125)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-end-play-1"></a>

## `epok::CameraRig3D::end_play`

**Purpose.** Ends play as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 203](../../../runtime/camera_rig.hpp#L203)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::CameraRig3D& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-make-active-1"></a>

## `epok::CameraRig3D::make_active`

**Purpose.** Performs `make active` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
bool make_active()
```

- **Declared at:** [line 194](../../../runtime/camera_rig.hpp#L194)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.make_active();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camerarig3d-on-disable-1"></a>

## `epok::CameraRig3D::on_disable`

**Purpose.** Performs `on disable` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 202](../../../runtime/camera_rig.hpp#L202)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-refresh-1"></a>

## `epok::CameraRig3D::refresh`

**Purpose.** Performs `refresh` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
CameraRigSample refresh(Fixed delta_seconds)
```

- **Declared at:** [line 175](../../../runtime/camera_rig.hpp#L175)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CameraRigSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.refresh(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-sample-1"></a>

## `epok::CameraRig3D::sample`

**Purpose.** Performs `sample` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
CameraRigSample sample()const
```

- **Declared at:** [line 193](../../../runtime/camera_rig.hpp#L193)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CameraRigSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-set-angles-1"></a>

## `epok::CameraRig3D::set_angles`

**Purpose.** Sets angles as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
bool set_angles(Fixed yaw_degrees,Fixed pitch_degrees)
```

- **Declared at:** [line 140](../../../runtime/camera_rig.hpp#L140)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `yaw_degrees` | `Fixed` | Input | Value supplied for `yaw_degrees`. See the exact type and module contract. |
| `pitch_degrees` | `Fixed` | Input | Value supplied for `pitch_degrees`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed yaw_degrees
// Fixed pitch_degrees

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.set_angles(yaw_degrees, pitch_degrees);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camerarig3d-snap-1"></a>

## `epok::CameraRig3D::snap`

**Purpose.** Performs `snap` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
CameraRigSample snap()
```

- **Declared at:** [line 192](../../../runtime/camera_rig.hpp#L192)
- **Kind:** `cxx method`

**Returns.** Returns `CameraRigSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::CameraRig3D& object = /* obtain a valid instance */;

auto result = object.snap();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-tick-1"></a>

## `epok::CameraRig3D::tick`

**Purpose.** Performs `tick` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 201](../../../runtime/camera_rig.hpp#L201)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CameraRig3D& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camerarig3d-view-updated-1"></a>

## `epok::CameraRig3D::view_updated`

**Purpose.** Performs `view updated` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
virtual void view_updated(CameraRigSample view)
```

- **Declared at:** [line 174](../../../runtime/camera_rig.hpp#L174)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `view` | `CameraRigSample` | Input | Value supplied for `view`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// CameraRigSample view

epok::CameraRig3D& object = /* obtain a valid instance */;

object.view_updated(view);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-firstpersoncamera-class-id-1"></a>

## `epok::FirstPersonCamera::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 208](../../../runtime/camera_rig.hpp#L208)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::FirstPersonCamera& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-firstpersoncamera-firstpersoncamera-1"></a>

## `epok::FirstPersonCamera::FirstPersonCamera`

**Purpose.** Constructs `epok::FirstPersonCamera` for camera selection, projection and shared render resources.

**Exact declaration**

```cpp
FirstPersonCamera()
```

- **Declared at:** [line 209](../../../runtime/camera_rig.hpp#L209)
- **Kind:** `constructor`

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::FirstPersonCamera value();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-isometriccamera-class-id-1"></a>

## `epok::IsometricCamera::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 226](../../../runtime/camera_rig.hpp#L226)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::IsometricCamera& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-isometriccamera-isometriccamera-1"></a>

## `epok::IsometricCamera::IsometricCamera`

**Purpose.** Constructs `epok::IsometricCamera` for camera selection, projection and shared render resources.

**Exact declaration**

```cpp
IsometricCamera()
```

- **Declared at:** [line 227](../../../runtime/camera_rig.hpp#L227)
- **Kind:** `constructor`

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::IsometricCamera value();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-begin-play-1"></a>

## `epok::SpringArmComponent::begin_play`

**Purpose.** Begins play as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 106](../../../runtime/camera_rig.hpp#L106)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-class-id-1"></a>

## `epok::SpringArmComponent::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 74](../../../runtime/camera_rig.hpp#L74)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-end-play-1"></a>

## `epok::SpringArmComponent::end_play`

**Purpose.** Ends play as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 106](../../../runtime/camera_rig.hpp#L106)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::SpringArmComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-on-disable-1"></a>

## `epok::SpringArmComponent::on_disable`

**Purpose.** Performs `on disable` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 106](../../../runtime/camera_rig.hpp#L106)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-on-enable-1"></a>

## `epok::SpringArmComponent::on_enable`

**Purpose.** Performs `on enable` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void on_enable()override
```

- **Declared at:** [line 106](../../../runtime/camera_rig.hpp#L106)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-reset-1"></a>

## `epok::SpringArmComponent::reset`

**Purpose.** Resets reset as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 104](../../../runtime/camera_rig.hpp#L104)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-sample-1"></a>

## `epok::SpringArmComponent::sample`

**Purpose.** Performs `sample` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
SpringArmSample sample()const
```

- **Declared at:** [line 105](../../../runtime/camera_rig.hpp#L105)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `SpringArmSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::SpringArmComponent& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-solve-1"></a>

## `epok::SpringArmComponent::solve`

**Purpose.** Performs `solve` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
SpringArmSample solve(GameplayVector3 origin,GameplayVector3 offset,ObjectId ignore,Fixed delta_seconds)
```

- **Declared at:** [line 81](../../../runtime/camera_rig.hpp#L81)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `offset` | `GameplayVector3` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `SpringArmSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 origin
// GameplayVector3 offset
// ObjectId ignore
// Fixed delta_seconds

epok::SpringArmComponent& object = /* obtain a valid instance */;

auto result = object.solve(origin, offset, ignore, delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-springarmcomponent-solve-shared-1"></a>

## `epok::SpringArmComponent::solve_shared`

**Purpose.** Performs `solve shared` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
SpringArmSample solve_shared(GameplayVector3 origin,GameplayVector3 offset,ObjectId ignore,Fixed delta_seconds,ObjectId query_context,uint32_t collision_revision)
```

- **Declared at:** [line 87](../../../runtime/camera_rig.hpp#L87)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `offset` | `GameplayVector3` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |
| `query_context` | `ObjectId` | Input | Value supplied for `query_context`. See the exact type and module contract. |
| `collision_revision` | `uint32_t` | Input | Value supplied for `collision_revision`. See the exact type and module contract. |

**Returns.** Returns `SpringArmSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 origin
// GameplayVector3 offset
// ObjectId ignore
// Fixed delta_seconds
// ObjectId query_context
// uint32_t collision_revision

epok::SpringArmComponent& object = /* obtain a valid instance */;

auto result = object.solve_shared(origin, offset, ignore, delta_seconds, query_context, collision_revision);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-thirdpersoncamera-class-id-1"></a>

## `epok::ThirdPersonCamera::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 214](../../../runtime/camera_rig.hpp#L214)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::ThirdPersonCamera& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-topdowncamera-class-id-1"></a>

## `epok::TopDownCamera::class_id`

**Purpose.** Performs `class id` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 220](../../../runtime/camera_rig.hpp#L220)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::TopDownCamera& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-topdowncamera-topdowncamera-1"></a>

## `epok::TopDownCamera::TopDownCamera`

**Purpose.** Constructs `epok::TopDownCamera` for camera selection, projection and shared render resources.

**Exact declaration**

```cpp
TopDownCamera()
```

- **Declared at:** [line 221](../../../runtime/camera_rig.hpp#L221)
- **Kind:** `constructor`

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_rig.hpp"

epok::TopDownCamera value();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
