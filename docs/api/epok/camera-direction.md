# Epok API: Camera Direction

> **Header:** `"camera_direction.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/camera_direction.hpp)

This module covers camera selection, projection and shared render resources. It documents 4 public callables declared directly in this header.

## Declared types

`epok::camera_direction_detail::AtanTable`

## Callable index

- [`epok::atan2_turn12`](#epok-atan2-turn12-1) — Performs `atan2 turn12` as part of camera selection, projection and shared render resources.
- [`epok::camera_direction_detail::atan_series`](#epok-camera-direction-detail-atan-series-1) — Performs `atan series` as part of camera selection, projection and shared render resources.
- [`epok::camera_direction_detail::make_atan_table`](#epok-camera-direction-detail-make-atan-table-1) — Performs `make atan table` as part of camera selection, projection and shared render resources.
- [`epok::camera_target_direction`](#epok-camera-target-direction-1) — Rig convention: +pitch looks up, yaw zero looks along -Z, +roll about Z.

<a id="epok-atan2-turn12-1"></a>

## `epok::atan2_turn12`

**Purpose.** Performs `atan2 turn12` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline uint16_t atan2_turn12(int64_t y,int64_t x)
```

- **Declared at:** [line 24](../../../runtime/camera_direction.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `y` | `int64_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `x` | `int64_t` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_direction.hpp"

// Assume these named values have been initialized with valid data:
// int64_t y
// int64_t x

auto result = epok::atan2_turn12(y, x);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera-direction-detail-atan-series-1"></a>

## `epok::camera_direction_detail::atan_series`

**Purpose.** Performs `atan series` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
constexpr double atan_series(double x)
```

- **Declared at:** [line 6](../../../runtime/camera_direction.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `double` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `double`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_direction.hpp"

// Assume these named values have been initialized with valid data:
// double x

auto result = epok::camera_direction_detail::atan_series(x);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera-direction-detail-make-atan-table-1"></a>

## `epok::camera_direction_detail::make_atan_table`

**Purpose.** Performs `make atan table` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
consteval AtanTable make_atan_table()
```

- **Declared at:** [line 12](../../../runtime/camera_direction.hpp#L12)
- **Kind:** `function decl`

**Returns.** Returns `AtanTable`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_direction.hpp"

auto result = epok::camera_direction_detail::make_atan_table();
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera-target-direction-1"></a>

## `epok::camera_target_direction`

**Purpose.** Rig convention: +pitch looks up, yaw zero looks along -Z, +roll about Z.

**Details.** The adapter to Epok's local Euler convention is kept outside this function.

**Exact declaration**

```cpp
inline void camera_target_direction(const int32_t* camera,const int32_t* target, bool reflect_x,bool front_only,uint16_t reference_yaw, uint16_t* result)
```

- **Declared at:** [line 39](../../../runtime/camera_direction.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const int32_t *` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `target` | `const int32_t *` | Input | Value supplied for `target`. See the exact type and module contract. |
| `reflect_x` | `bool` | Input | Value supplied for `reflect_x`. See the exact type and module contract. |
| `front_only` | `bool` | Input | Value supplied for `front_only`. See the exact type and module contract. |
| `reference_yaw` | `uint16_t` | Input | Value supplied for `reference_yaw`. See the exact type and module contract. |
| `result` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The adapter to Epok's local Euler convention is kept outside this function.

**Usage pattern**

```cpp
#include "camera_direction.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * camera
// const int32_t * target
// bool reflect_x
// bool front_only
// uint16_t reference_yaw
// uint16_t * result

epok::camera_target_direction(camera, target, reflect_x, front_only, reference_yaw, result);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
