# Epok API: Rail Camera Aim

> **Header:** `"rail_camera_aim.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/rail_camera_aim.hpp)

This module covers camera selection, projection and shared render resources. It documents 2 public callables declared directly in this header.

## Declared types

`epok::RailAimData`, `epok::RailAimState`

## Callable index

- [`epok::advance_rail_aim`](#epok-advance-rail-aim-1) — Separate from transport and object storage: both PSX and native runners can apply the same controller after publishing a ready rail pose.
- [`epok::interpolate_rail_aim_parameters`](#epok-interpolate-rail-aim-parameters-1) — Performs `interpolate rail aim parameters` as part of camera selection, projection and shared render resources.

<a id="epok-advance-rail-aim-1"></a>

## `epok::advance_rail_aim`

**Purpose.** Separate from transport and object storage: both PSX and native runners can apply the same controller after publishing a ready rail pose.

**Details.** Elapsed time excludes paused/loading time. No target or camera authoring transform moves.

**Exact declaration**

```cpp
inline void advance_rail_aim(const RailAimData& data,RailAimState& state,uint32_t elapsed, const int32_t* camera,const int32_t* target,int32_t* rotation, uint16_t current,uint16_t next,uint32_t fraction,bool cut)
```

- **Declared at:** [line 35](../../../runtime/rail_camera_aim.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const RailAimData &` | Input | Value supplied for `data`. See the exact type and module contract. |
| `state` | `RailAimState &` | Input/output; inspect the function contract | Value supplied for `state`. See the exact type and module contract. |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |
| `camera` | `const int32_t *` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `target` | `const int32_t *` | Input | Value supplied for `target`. See the exact type and module contract. |
| `rotation` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `rotation`. See the exact type and module contract. |
| `current` | `uint16_t` | Input | Value supplied for `current`. See the exact type and module contract. |
| `next` | `uint16_t` | Input | Value supplied for `next`. See the exact type and module contract. |
| `fraction` | `uint32_t` | Input | Value supplied for `fraction`. See the exact type and module contract. |
| `cut` | `bool` | Input | Value supplied for `cut`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Elapsed time excludes paused/loading time. No target or camera authoring transform moves.

**Usage pattern**

```cpp
#include "rail_camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// const RailAimData & data
// RailAimState & state
// uint32_t elapsed
// const int32_t * camera
// const int32_t * target
// int32_t * rotation
// uint16_t current
// uint16_t next
// uint32_t fraction
// bool cut

epok::advance_rail_aim(data, state, elapsed, camera, target, rotation, current, next, fraction, cut);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-interpolate-rail-aim-parameters-1"></a>

## `epok::interpolate_rail_aim_parameters`

**Purpose.** Performs `interpolate rail aim parameters` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline CameraAimParameters interpolate_rail_aim_parameters(const RailAimData& data,uint16_t current,uint16_t next,uint32_t fraction)
```

- **Declared at:** [line 17](../../../runtime/rail_camera_aim.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const RailAimData &` | Input | Value supplied for `data`. See the exact type and module contract. |
| `current` | `uint16_t` | Input | Value supplied for `current`. See the exact type and module contract. |
| `next` | `uint16_t` | Input | Value supplied for `next`. See the exact type and module contract. |
| `fraction` | `uint32_t` | Input | Value supplied for `fraction`. See the exact type and module contract. |

**Returns.** Returns `CameraAimParameters`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_camera_aim.hpp"

// Assume these named values have been initialized with valid data:
// const RailAimData & data
// uint16_t current
// uint16_t next
// uint32_t fraction

auto result = epok::interpolate_rail_aim_parameters(data, current, next, fraction);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
