# Epok API: Track Camera Pose

> **Header:** `"track_camera_pose.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/track_camera_pose.hpp)

This module covers camera selection, projection and shared render resources. It documents 2 public callables declared directly in this header.

## Declared types

`epok::TrackCameraPose`

## Callable index

- [`epok::aim_track_camera`](#epok-aim-track-camera-1) — Performs `aim track camera` as part of camera selection, projection and shared render resources.
- [`epok::interpolate_track_camera`](#epok-interpolate-track-camera-1) — Performs `interpolate track camera` as part of camera selection, projection and shared render resources.

<a id="epok-aim-track-camera-1"></a>

## `epok::aim_track_camera`

**Purpose.** Performs `aim track camera` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline void aim_track_camera(TrackCameraPose& pose,uint32_t elapsed,const int32_t* target,uint32_t previous_transitions)
```

- **Declared at:** [line 16](../../../runtime/track_camera_pose.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pose` | `TrackCameraPose &` | Input/output; inspect the function contract | Value supplied for `pose`. See the exact type and module contract. |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |
| `target` | `const int32_t *` | Input | Value supplied for `target`. See the exact type and module contract. |
| `previous_transitions` | `uint32_t` | Input | Value supplied for `previous_transitions`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "track_camera_pose.hpp"

// Assume these named values have been initialized with valid data:
// TrackCameraPose & pose
// uint32_t elapsed
// const int32_t * target
// uint32_t previous_transitions

epok::aim_track_camera(pose, elapsed, target, previous_transitions);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-interpolate-track-camera-1"></a>

## `epok::interpolate_track_camera`

**Purpose.** Performs `interpolate track camera` as part of camera selection, projection and shared render resources.

**Exact declaration**

```cpp
inline TrackCameraPose interpolate_track_camera(const VisibilityTrackSample& a,const VisibilityTrackSample& b,uint32_t fraction)
```

- **Declared at:** [line 5](../../../runtime/track_camera_pose.hpp#L5)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const VisibilityTrackSample &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const VisibilityTrackSample &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `fraction` | `uint32_t` | Input | Value supplied for `fraction`. See the exact type and module contract. |

**Returns.** Returns `TrackCameraPose`. Check the purpose and failure notes before using the value.

**Use it when.** You need camera selection, projection and shared render resources and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "track_camera_pose.hpp"

// Assume these named values have been initialized with valid data:
// const VisibilityTrackSample & a
// const VisibilityTrackSample & b
// uint32_t fraction

auto result = epok::interpolate_track_camera(a, b, fraction);
```

**Why choose it.** It provides direct, allocation-conscious access to camera selection, projection and shared render resources. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
