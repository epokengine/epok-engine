# Epok API: Camera Projection

> **Header:** `"camera_projection.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/camera_projection.hpp)

This module covers camera selection, projection and shared render resources. It documents 2 public callables declared directly in this header.

## Callable index

- [`epok::camera_projection::focal`](#epok-camera-projection-focal-1) — Shared Q12 lens and clip rules for console and Native Play screen queries.
- [`epok::camera_projection::point`](#epok-camera-projection-point-1) — Camera coordinates and focal lengths are Q12; screen output is top-left +Y down.

<a id="epok-camera-projection-focal-1"></a>

## `epok::camera_projection::focal`

**Purpose.** Shared Q12 lens and clip rules for console and Native Play screen queries.

**Exact declaration**

```cpp
inline int32_t focal(int32_t degrees)
```

- **Declared at:** [line 7](../../../runtime/camera_projection.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `int32_t` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "camera_projection.hpp"

// Assume these named values have been initialized with valid data:
// int32_t degrees

auto result = epok::camera_projection::focal(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera-projection-point-1"></a>

## `epok::camera_projection::point`

**Purpose.** Camera coordinates and focal lengths are Q12; screen output is top-left +Y down.

**Details.** Failure leaves output untouched. Near is inclusive, far/right/bottom are exclusive, matching the console's existing visible-point contract.

**Exact declaration**

```cpp
inline bool point(const int32_t* camera,int32_t focal_x,int32_t focal_y,int width,int height,int32_t* screen)
```

- **Declared at:** [line 17](../../../runtime/camera_projection.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const int32_t *` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `focal_x` | `int32_t` | Input | Value supplied for `focal_x`. See the exact type and module contract. |
| `focal_y` | `int32_t` | Input | Value supplied for `focal_y`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |
| `screen` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Failure leaves output untouched. Near is inclusive, far/right/bottom are exclusive, matching the console's existing visible-point contract.

**Usage pattern**

```cpp
#include "camera_projection.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * camera
// int32_t focal_x
// int32_t focal_y
// int width
// int height
// int32_t * screen

auto result = epok::camera_projection::point(camera, focal_x, focal_y, width, height, screen);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
