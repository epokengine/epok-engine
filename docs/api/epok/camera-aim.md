# Epok API: Camera Aim

> **Header:** `"camera_aim.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/camera_aim.hpp)

This module covers camera selection, projection and shared render resources. It documents 5 public callables declared directly in this header.

## Declared types

`epok::CameraAimParameters`

## Callable index

- [`epok::angle_delta_turn12`](#epok-angle-delta-turn12-1) — Angles use 4096 steps per turn, weights use 256 = 1.
- [`epok::blend_angle_turn12`](#epok-blend-angle-turn12-1) — Performs `blend angle turn12` as part of camera selection, projection and shared render resources.
- [`epok::blend_camera_aim`](#epok-blend-camera-aim-1) — Performs `blend camera aim` as part of camera selection, projection and shared render resources.
- [`epok::limit_angle_turn12`](#epok-limit-angle-turn12-1) — Performs `limit angle turn12` as part of camera selection, projection and shared render resources.
- [`epok::smooth_angle_turn12`](#epok-smooth-angle-turn12-1) — Performs `smooth angle turn12` as part of camera selection, projection and shared render resources.

<a id="epok-angle-delta-turn12-1"></a>

## `epok::angle_delta_turn12`

**Purpose.** Angles use 4096 steps per turn, weights use 256 = 1.

**Details.** The caller supplies angles in one consistent camera convention and owns the update cadence. These helpers neither move the camera nor select visibility/streaming pages.

**Exact declaration**

```cpp
inline int32_t angle_delta_turn12(uint32_t from,uint32_t to)
```

- **Declared at:** [line 8](../../../runtime/camera_aim.hpp#L8)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `uint32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `uint32_t` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** The caller supplies angles in one consistent camera convention and owns the update cadence. These helpers neither move the camera nor select visibility/streaming pages.

**Usage pattern**

```cpp
#include "camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t from
// uint32_t to

auto result = epok::angle_delta_turn12(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-blend-angle-turn12-1"></a>

## `epok::blend_angle_turn12`

**Purpose.** Performs `blend angle turn12` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline uint16_t blend_angle_turn12(uint32_t from,uint32_t to,uint16_t weight)
```

- **Declared at:** [line 11](../../../runtime/camera_aim.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `uint32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `uint32_t` | Input | Value supplied for `to`. See the exact type and module contract. |
| `weight` | `uint16_t` | Input | Value supplied for `weight`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t from
// uint32_t to
// uint16_t weight

auto result = epok::blend_angle_turn12(from, to, weight);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-blend-camera-aim-1"></a>

## `epok::blend_camera_aim`

**Purpose.** Performs `blend camera aim` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline void blend_camera_aim(const uint16_t* base,const uint16_t* target, const uint16_t* previous,const CameraAimParameters& parameters, uint16_t* result)
```

- **Declared at:** [line 39](../../../runtime/camera_aim.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `base` | `const uint16_t *` | Input | Value supplied for `base`. See the exact type and module contract. |
| `target` | `const uint16_t *` | Input | Value supplied for `target`. See the exact type and module contract. |
| `previous` | `const uint16_t *` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `parameters` | `const CameraAimParameters &` | Input | Value supplied for `parameters`. See the exact type and module contract. |
| `result` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// const uint16_t * base
// const uint16_t * target
// const uint16_t * previous
// const CameraAimParameters & parameters
// uint16_t * result

epok::blend_camera_aim(base, target, previous, parameters, result);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-limit-angle-turn12-1"></a>

## `epok::limit_angle_turn12`

**Purpose.** Performs `limit angle turn12` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline uint16_t limit_angle_turn12(uint32_t angle,uint32_t base,uint16_t positive,uint16_t negative)
```

- **Declared at:** [line 21](../../../runtime/camera_aim.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `angle` | `uint32_t` | Input | Value supplied for `angle`. See the exact type and module contract. |
| `base` | `uint32_t` | Input | Value supplied for `base`. See the exact type and module contract. |
| `positive` | `uint16_t` | Input | Value supplied for `positive`. See the exact type and module contract. |
| `negative` | `uint16_t` | Input | Value supplied for `negative`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t angle
// uint32_t base
// uint16_t positive
// uint16_t negative

auto result = epok::limit_angle_turn12(angle, base, positive, negative);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-smooth-angle-turn12-1"></a>

## `epok::smooth_angle_turn12`

**Purpose.** Performs `smooth angle turn12` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline uint16_t smooth_angle_turn12(uint32_t from,uint32_t to,uint16_t weight,uint16_t limit)
```

- **Declared at:** [line 15](../../../runtime/camera_aim.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `uint32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `uint32_t` | Input | Value supplied for `to`. See the exact type and module contract. |
| `weight` | `uint16_t` | Input | Value supplied for `weight`. See the exact type and module contract. |
| `limit` | `uint16_t` | Input | Value supplied for `limit`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t from
// uint32_t to
// uint16_t weight
// uint16_t limit

auto result = epok::smooth_angle_turn12(from, to, weight, limit);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
