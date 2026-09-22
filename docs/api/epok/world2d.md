# Epok API: World2D

> **Header:** `"world2d.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d.hpp)

This module covers the world2d module. It documents 45 public callables declared directly in this header.

## Declared types

`epok::Aabb2D`, `epok::Affine2D`, `epok::Camera2D`, `epok::Collider2D`, `epok::Collider2D::Shape`, `epok::ColliderEntry2D`, `epok::CollisionWorld2D`, `epok::detail::SineTable2D`, `epok::MoveResult2D`, `epok::Pick2DEntry`, `epok::SpatialHit2D`

## Callable index

- [`epok::aabb2d_contains`](#epok-aabb2d-contains-1) — Performs `aabb2d contains` as part of the world2d module.
- [`epok::aabb2d_overlap`](#epok-aabb2d-overlap-1) — Performs `aabb2d overlap` as part of the world2d module.
- [`epok::aabb2d_touch`](#epok-aabb2d-touch-1) — Performs `aabb2d touch` as part of the world2d module.
- [`epok::aabb2d_translated`](#epok-aabb2d-translated-1) — Performs `aabb2d translated` as part of the world2d module.
- [`epok::Affine2D::direction`](#epok-affine2d-direction-1) — Same as point() without the translation: for directions, extents and velocities.
- [`epok::Affine2D::identity`](#epok-affine2d-identity-1) — Performs `identity` as part of the world2d module.
- [`epok::Affine2D::point`](#epok-affine2d-point-1) — Performs `point` as part of the world2d module.
- [`epok::box_circle_overlap_2d`](#epok-box-circle-overlap-2d-1) — Closest point on the box to the circle centre; touching counts only when inclusive.
- [`epok::camera2d_pixels_per_unit`](#epok-camera2d-pixels-per-unit-1) — Performs `camera2d pixels per unit` as part of the world2d module.
- [`epok::camera2d_viewport_center`](#epok-camera2d-viewport-center-1) — Performs `camera2d viewport center` as part of the world2d module.
- [`epok::circle_overlap_2d`](#epok-circle-overlap-2d-1) — Performs `circle overlap 2d` as part of the world2d module.
- [`epok::collider2d_bounds`](#epok-collider2d-bounds-1) — Performs `collider2d bounds` as part of the world2d module.
- [`epok::collider2d_center`](#epok-collider2d-center-1) — World centre of a collider: the owner's world position plus the (unrotated) offset.
- [`epok::collider2d_overlap`](#epok-collider2d-overlap-1) — Shape-aware pair test in world space.
- [`epok::CollisionWorld2D::bounds`](#epok-collisionworld2d-bounds-1) — Performs `bounds` as part of the world2d module.
- [`epok::CollisionWorld2D::clear`](#epok-collisionworld2d-clear-1) — Clears clear as part of the world2d module.
- [`epok::CollisionWorld2D::disable`](#epok-collisionworld2d-disable-1) — Performs `disable` as part of the world2d module.
- [`epok::CollisionWorld2D::live`](#epok-collisionworld2d-live-1) — Performs `live` as part of the world2d module.
- [`epok::CollisionWorld2D::move_and_slide_2d`](#epok-collisionworld2d-move-and-slide-2d-1) — Axis-separated resolution: X is applied first and clipped against every blocking box whose Y span overlaps, then Y is applied against the updated position.
- [`epok::CollisionWorld2D::overlap`](#epok-collisionworld2d-overlap-1) — Returns the total number of matches; only the first `output_capacity` are written.
- [`epok::CollisionWorld2D::position`](#epok-collisionworld2d-position-1) — Performs `position` as part of the world2d module.
- [`epok::CollisionWorld2D::raycast2d`](#epok-collisionworld2d-raycast2d-1) — `displacement` is the complete segment, not a unit direction.
- [`epok::CollisionWorld2D::set`](#epok-collisionworld2d-set-1) — Sets set as part of the world2d module.
- [`epok::CollisionWorld2D::sync`](#epok-collisionworld2d-sync-1) — Replaces the whole live set in one call; slots beyond `count` become disabled and therefore produce trigger exits on the next update_triggers().
- [`epok::CollisionWorld2D::update_triggers`](#epok-collisionworld2d-update-triggers-1) — Enter/Stay/Exit per pair, exactly once each, using the exact shape tests.
- [`epok::compose`](#epok-compose-1) — parent * child: the child's local matrix expressed in the parent's space.
- [`epok::cos_degrees`](#epok-cos-degrees-1) — Performs `cos degrees` as part of the world2d module.
- [`epok::detail::clamp_raw_2d`](#epok-detail-clamp-raw-2d-1) — Saturating conversion of a raw Q12 accumulator back into Fixed.
- [`epok::detail::make_sine_table_2d`](#epok-detail-make-sine-table-2d-1) — Performs `make sine table 2d` as part of the world2d module.
- [`epok::detail::mul_2d`](#epok-detail-mul-2d-1) — Performs `mul 2d` as part of the world2d module.
- [`epok::detail::sine_quarter_2d`](#epok-detail-sine-quarter-2d-1) — sin of an angle inside the first quadrant, expressed in raw Q12 degrees [0, 90*4096].
- [`epok::detail::sine_series_2d`](#epok-detail-sine-series-2d-1) — Performs `sine series 2d` as part of the world2d module.
- [`epok::distance_squared_raw_2d`](#epok-distance-squared-raw-2d-1) — Squared distance in raw Q24; |position| <= 8192 keeps this inside 2^51.
- [`epok::draw_key_2d`](#epok-draw-key-2d-1) — ---- draw order ---------------------------------------------------------------------- A single monotonically comparable key so the renderer never needs a multi-field comparator: layer (int8, coarse) above draw_order (int16) above the creation index (uint16, the tie breaker that makes the order stable and reproducible across frames).
- [`epok::fixed_in_range`](#epok-fixed-in-range-1) — Performs `fixed in range` as part of the world2d module.
- [`epok::local_matrix`](#epok-local-matrix-1) — Local matrix of a Transform2D: translate * rotate * scale, applied in that order to a point (scale first, then rotation, then translation).
- [`epok::pick_2d`](#epok-pick-2d-1) — Topmost entry under a screen point (highest draw key wins; ties go to the lowest index, which keeps the result stable).
- [`epok::screen_to_world`](#epok-screen-to-world-1) — Exact inverse of world_to_screen up to Q12 rounding.
- [`epok::sin_degrees`](#epok-sin-degrees-1) — Sine of an angle in degrees.
- [`epok::sort_draw_order`](#epok-sort-draw-order-1) — Bounded, allocation-free, stable insertion sort of an index array by draw key.
- [`epok::SpatialHit2D::operator bool`](#epok-spatialhit2d-operator-bool-1) — Performs `operator  bool` as part of the world2d module.
- [`epok::transform2d_clamp`](#epok-transform2d-clamp-1) — Clamps into the documented range instead of rejecting.
- [`epok::transform2d_valid`](#epok-transform2d-valid-1) — Documented validity of a Transform2D.
- [`epok::world_matrix_2d`](#epok-world-matrix-2d-1) — World matrix of a SceneComponent2D by walking its attach_parent chain.
- [`epok::world_to_screen`](#epok-world-to-screen-1) — Performs `world to screen` as part of the world2d module.

<a id="epok-aabb2d-contains-1"></a>

## `epok::aabb2d_contains`

**Purpose.** Performs `aabb2d contains` as part of the world2d module.

**Exact declaration**

```cpp
inline bool aabb2d_contains(const Aabb2D& box, const Fixed* point)
```

- **Declared at:** [line 324](../../../runtime/world2d.hpp#L324)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb2D &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `point` | `const Fixed *` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & box
// const Fixed * point

auto result = epok::aabb2d_contains(box, point);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-aabb2d-overlap-1"></a>

## `epok::aabb2d_overlap`

**Purpose.** Performs `aabb2d overlap` as part of the world2d module.

**Exact declaration**

```cpp
inline bool aabb2d_overlap(const Aabb2D& a, const Aabb2D& b)
```

- **Declared at:** [line 314](../../../runtime/world2d.hpp#L314)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Aabb2D &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Aabb2D &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & a
// const Aabb2D & b

auto result = epok::aabb2d_overlap(a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-aabb2d-touch-1"></a>

## `epok::aabb2d_touch`

**Purpose.** Performs `aabb2d touch` as part of the world2d module.

**Exact declaration**

```cpp
inline bool aabb2d_touch(const Aabb2D& a, const Aabb2D& b)
```

- **Declared at:** [line 319](../../../runtime/world2d.hpp#L319)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Aabb2D &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Aabb2D &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & a
// const Aabb2D & b

auto result = epok::aabb2d_touch(a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-aabb2d-translated-1"></a>

## `epok::aabb2d_translated`

**Purpose.** Performs `aabb2d translated` as part of the world2d module.

**Exact declaration**

```cpp
inline Aabb2D aabb2d_translated(Aabb2D box, const Fixed* delta)
```

- **Declared at:** [line 329](../../../runtime/world2d.hpp#L329)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `Aabb2D` | Input | Value supplied for `box`. See the exact type and module contract. |
| `delta` | `const Fixed *` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** Returns `Aabb2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Aabb2D box
// const Fixed * delta

auto result = epok::aabb2d_translated(box, delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-affine2d-direction-1"></a>

## `epok::Affine2D::direction`

**Purpose.** Same as point() without the translation: for directions, extents and velocities.

**Exact declaration**

```cpp
void direction(const Fixed* in, Fixed* out) const
```

- **Declared at:** [line 151](../../../runtime/world2d.hpp#L151)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Fixed *` | Input | Value supplied for `in`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * in
// Fixed * out

epok::Affine2D& object = /* obtain a valid instance */;

object.direction(in, out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-affine2d-identity-1"></a>

## `epok::Affine2D::identity`

**Purpose.** Performs `identity` as part of the world2d module.

**Exact declaration**

```cpp
static Affine2D identity()
```

- **Declared at:** [line 140](../../../runtime/world2d.hpp#L140)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Affine2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

auto result = epok::Affine2D::identity();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-affine2d-point-1"></a>

## `epok::Affine2D::point`

**Purpose.** Performs `point` as part of the world2d module.

**Exact declaration**

```cpp
void point(const Fixed* in, Fixed* out) const
```

- **Declared at:** [line 146](../../../runtime/world2d.hpp#L146)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Fixed *` | Input | Value supplied for `in`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * in
// Fixed * out

epok::Affine2D& object = /* obtain a valid instance */;

object.point(in, out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-box-circle-overlap-2d-1"></a>

## `epok::box_circle_overlap_2d`

**Purpose.** Closest point on the box to the circle centre; touching counts only when inclusive.

**Exact declaration**

```cpp
inline bool box_circle_overlap_2d(const Aabb2D& box, const Fixed* center, Fixed radius, bool inclusive = false)
```

- **Declared at:** [line 363](../../../runtime/world2d.hpp#L363)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb2D &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `center` | `const Fixed *` | Input | Value supplied for `center`. See the exact type and module contract. |
| `radius` | `Fixed` | Input | Value supplied for `radius`. See the exact type and module contract. |
| `inclusive` | `bool` | Input | Value supplied for `inclusive`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & box
// const Fixed * center
// Fixed radius
// bool inclusive

auto result = epok::box_circle_overlap_2d(box, center, radius, inclusive);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera2d-pixels-per-unit-1"></a>

## `epok::camera2d_pixels_per_unit`

**Purpose.** Performs `camera2d pixels per unit` as part of the world2d module.

**Exact declaration**

```cpp
inline Fixed camera2d_pixels_per_unit(const Camera2D& camera)
```

- **Declared at:** [line 221](../../../runtime/world2d.hpp#L221)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera

auto result = epok::camera2d_pixels_per_unit(camera);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera2d-viewport-center-1"></a>

## `epok::camera2d_viewport_center`

**Purpose.** Performs `camera2d viewport center` as part of the world2d module.

**Exact declaration**

```cpp
inline void camera2d_viewport_center(const Camera2D& camera, Fixed* out)
```

- **Declared at:** [line 224](../../../runtime/world2d.hpp#L224)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera
// Fixed * out

epok::camera2d_viewport_center(camera, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-circle-overlap-2d-1"></a>

## `epok::circle_overlap_2d`

**Purpose.** Performs `circle overlap 2d` as part of the world2d module.

**Exact declaration**

```cpp
inline bool circle_overlap_2d(const Fixed* a, Fixed radius_a, const Fixed* b, Fixed radius_b, bool inclusive = false)
```

- **Declared at:** [line 357](../../../runtime/world2d.hpp#L357)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Fixed *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `radius_a` | `Fixed` | Input | Value supplied for `radius_a`. See the exact type and module contract. |
| `b` | `const Fixed *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `radius_b` | `Fixed` | Input | Value supplied for `radius_b`. See the exact type and module contract. |
| `inclusive` | `bool` | Input | Value supplied for `inclusive`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * a
// Fixed radius_a
// const Fixed * b
// Fixed radius_b
// bool inclusive

auto result = epok::circle_overlap_2d(a, radius_a, b, radius_b, inclusive);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collider2d-bounds-1"></a>

## `epok::collider2d_bounds`

**Purpose.** Performs `collider2d bounds` as part of the world2d module.

**Exact declaration**

```cpp
inline Aabb2D collider2d_bounds(const Collider2D& collider, const Fixed* world_xy)
```

- **Declared at:** [line 341](../../../runtime/world2d.hpp#L341)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `collider` | `const Collider2D &` | Input | Value supplied for `collider`. See the exact type and module contract. |
| `world_xy` | `const Fixed *` | Input | Value supplied for `world_xy`. See the exact type and module contract. |

**Returns.** Returns `Aabb2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Collider2D & collider
// const Fixed * world_xy

auto result = epok::collider2d_bounds(collider, world_xy);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collider2d-center-1"></a>

## `epok::collider2d_center`

**Purpose.** World centre of a collider: the owner's world position plus the (unrotated) offset.

**Exact declaration**

```cpp
inline void collider2d_center(const Collider2D& collider, const Fixed* world_xy, Fixed* out)
```

- **Declared at:** [line 337](../../../runtime/world2d.hpp#L337)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `collider` | `const Collider2D &` | Input | Value supplied for `collider`. See the exact type and module contract. |
| `world_xy` | `const Fixed *` | Input | Value supplied for `world_xy`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Collider2D & collider
// const Fixed * world_xy
// Fixed * out

epok::collider2d_center(collider, world_xy, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collider2d-overlap-1"></a>

## `epok::collider2d_overlap`

**Purpose.** Shape-aware pair test in world space.

**Exact declaration**

```cpp
inline bool collider2d_overlap(const Collider2D& a, const Fixed* world_a, const Collider2D& b, const Fixed* world_b, bool inclusive = false)
```

- **Declared at:** [line 375](../../../runtime/world2d.hpp#L375)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Collider2D &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `world_a` | `const Fixed *` | Input | Value supplied for `world_a`. See the exact type and module contract. |
| `b` | `const Collider2D &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `world_b` | `const Fixed *` | Input | Value supplied for `world_b`. See the exact type and module contract. |
| `inclusive` | `bool` | Input | Value supplied for `inclusive`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Collider2D & a
// const Fixed * world_a
// const Collider2D & b
// const Fixed * world_b
// bool inclusive

auto result = epok::collider2d_overlap(a, world_a, b, world_b, inclusive);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-bounds-1"></a>

## `epok::CollisionWorld2D::bounds`

**Purpose.** Performs `bounds` as part of the world2d module.

**Exact declaration**

```cpp
const Aabb2D* bounds(size_t index) const
```

- **Declared at:** [line 514](../../../runtime/world2d.hpp#L514)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const Aabb2D *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.bounds(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld2d-clear-1"></a>

## `epok::CollisionWorld2D::clear`

**Purpose.** Clears clear as part of the world2d module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 481](../../../runtime/world2d.hpp#L481)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

epok::CollisionWorld2D& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld2d-disable-1"></a>

## `epok::CollisionWorld2D::disable`

**Purpose.** Performs `disable` as part of the world2d module.

**Exact declaration**

```cpp
void disable(size_t index)
```

- **Declared at:** [line 507](../../../runtime/world2d.hpp#L507)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::CollisionWorld2D& object = /* obtain a valid instance */;

object.disable(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld2d-live-1"></a>

## `epok::CollisionWorld2D::live`

**Purpose.** Performs `live` as part of the world2d module.

**Exact declaration**

```cpp
size_t live() const
```

- **Declared at:** [line 508](../../../runtime/world2d.hpp#L508)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.live();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld2d-move-and-slide-2d-1"></a>

## `epok::CollisionWorld2D::move_and_slide_2d`

**Purpose.** Axis-separated resolution: X is applied first and clipped against every blocking box whose Y span overlaps, then Y is applied against the updated position.

**Details.** A mover blocked on one axis therefore keeps sliding on the other. Triggers never block. The entry's stored position and bounds are updated with the resolved displacement.

**Exact declaration**

```cpp
MoveResult2D move_and_slide_2d(size_t index, const Fixed* displacement, uint32_t mask = 0xffffffffu)
```

- **Declared at:** [line 560](../../../runtime/world2d.hpp#L560)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** A mover blocked on one axis therefore keeps sliding on the other. Triggers never block. The entry's stored position and bounds are updated with the resolved displacement.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const Fixed * displacement
// uint32_t mask

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.move_and_slide_2d(index, displacement, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-overlap-1"></a>

## `epok::CollisionWorld2D::overlap`

**Purpose.** Returns the total number of matches; only the first `output_capacity` are written.

**Details.** Broad phase only: circles are tested through their bounding box.

**Exact declaration**

```cpp
size_t overlap(const Aabb2D& box, uint16_t* output, size_t output_capacity, uint32_t mask = 0xffffffffu, int ignore = -1, bool triggers = true) const
```

- **Declared at:** [line 522](../../../runtime/world2d.hpp#L522)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb2D &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `output` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `output_capacity` | `size_t` | Input | Value supplied for `output_capacity`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** Broad phase only: circles are tested through their bounding box.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & box
// uint16_t * output
// size_t output_capacity
// uint32_t mask
// int ignore
// bool triggers

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.overlap(box, output, output_capacity, mask, ignore, triggers);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-position-1"></a>

## `epok::CollisionWorld2D::position`

**Purpose.** Performs `position` as part of the world2d module.

**Exact declaration**

```cpp
const Fixed* position(size_t index) const
```

- **Declared at:** [line 517](../../../runtime/world2d.hpp#L517)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const Fixed *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.position(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld2d-raycast2d-1"></a>

## `epok::CollisionWorld2D::raycast2d`

**Purpose.** `displacement` is the complete segment, not a unit direction.

**Details.** Nearest hit wins; circles are approximated by their AABB, as in the broad phase.

**Exact declaration**

```cpp
SpatialHit2D raycast2d(const Fixed* origin, const Fixed* displacement, uint32_t mask = 0xffffffffu, int ignore = -1, bool triggers = false) const
```

- **Declared at:** [line 534](../../../runtime/world2d.hpp#L534)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Fixed *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit2D`. Check the purpose and failure notes before using the value.

**Use it when.** Nearest hit wins; circles are approximated by their AABB, as in the broad phase.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * origin
// const Fixed * displacement
// uint32_t mask
// int ignore
// bool triggers

epok::CollisionWorld2D& object = /* obtain a valid instance */;

auto result = object.raycast2d(origin, displacement, mask, ignore, triggers);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-set-1"></a>

## `epok::CollisionWorld2D::set`

**Purpose.** Sets set as part of the world2d module.

**Exact declaration**

```cpp
void set(size_t index, const ColliderEntry2D& value)
```

- **Declared at:** [line 496](../../../runtime/world2d.hpp#L496)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `const ColliderEntry2D &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const ColliderEntry2D & value

epok::CollisionWorld2D& object = /* obtain a valid instance */;

object.set(index, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-sync-1"></a>

## `epok::CollisionWorld2D::sync`

**Purpose.** Replaces the whole live set in one call; slots beyond `count` become disabled and therefore produce trigger exits on the next update_triggers().

**Exact declaration**

```cpp
void sync(const ColliderEntry2D* list, size_t count)
```

- **Declared at:** [line 489](../../../runtime/world2d.hpp#L489)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `list` | `const ColliderEntry2D *` | Input | Value supplied for `list`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const ColliderEntry2D * list
// size_t count

epok::CollisionWorld2D& object = /* obtain a valid instance */;

object.sync(list, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld2d-update-triggers-1"></a>

## `epok::CollisionWorld2D::update_triggers`

**Purpose.** Enter/Stay/Exit per pair, exactly once each, using the exact shape tests.

**Details.** A pair whose generation changed (slot reused by another owner) exits and re-enters, so a destroyed partner always produces its Exit.

**Exact declaration**

```cpp
template <class Callback> void update_triggers(Callback&& callback)
```

- **Declared at:** [line 601](../../../runtime/world2d.hpp#L601)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `Callback &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A pair whose generation changed (slot reused by another owner) exits and re-enters, so a destroyed partner always produces its Exit.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Callback

// Assume these named values have been initialized with valid data:
// Callback && callback

epok::CollisionWorld2D& object = /* obtain a valid instance */;

object.update_triggers<Callback>(callback);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-compose-1"></a>

## `epok::compose`

**Purpose.** parent * child: the child's local matrix expressed in the parent's space.

**Exact declaration**

```cpp
inline Affine2D compose(const Affine2D& parent, const Affine2D& child)
```

- **Declared at:** [line 156](../../../runtime/world2d.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `const Affine2D &` | Input | Value supplied for `parent`. See the exact type and module contract. |
| `child` | `const Affine2D &` | Input | Value supplied for `child`. See the exact type and module contract. |

**Returns.** Returns `Affine2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Affine2D & parent
// const Affine2D & child

auto result = epok::compose(parent, child);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-cos-degrees-1"></a>

## `epok::cos_degrees`

**Purpose.** Performs `cos degrees` as part of the world2d module.

**Exact declaration**

```cpp
inline Fixed cos_degrees(Fixed degrees)
```

- **Declared at:** [line 131](../../../runtime/world2d.hpp#L131)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::cos_degrees(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-clamp-raw-2d-1"></a>

## `epok::detail::clamp_raw_2d`

**Purpose.** Saturating conversion of a raw Q12 accumulator back into Fixed.

**Exact declaration**

```cpp
inline Fixed clamp_raw_2d(int64_t value)
```

- **Declared at:** [line 111](../../../runtime/world2d.hpp#L111)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value

auto result = epok::detail::clamp_raw_2d(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-make-sine-table-2d-1"></a>

## `epok::detail::make_sine_table_2d`

**Purpose.** Performs `make sine table 2d` as part of the world2d module.

**Exact declaration**

```cpp
consteval SineTable2D make_sine_table_2d()
```

- **Declared at:** [line 90](../../../runtime/world2d.hpp#L90)
- **Kind:** `function decl`

**Returns.** Returns `SineTable2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

auto result = epok::detail::make_sine_table_2d();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-mul-2d-1"></a>

## `epok::detail::mul_2d`

**Purpose.** Performs `mul 2d` as part of the world2d module.

**Exact declaration**

```cpp
inline Fixed mul_2d(Fixed a, Fixed b)
```

- **Declared at:** [line 116](../../../runtime/world2d.hpp#L116)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b

auto result = epok::detail::mul_2d(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-sine-quarter-2d-1"></a>

## `epok::detail::sine_quarter_2d`

**Purpose.** sin of an angle inside the first quadrant, expressed in raw Q12 degrees [0, 90*4096].

**Exact declaration**

```cpp
inline int32_t sine_quarter_2d(int32_t raw_degrees)
```

- **Declared at:** [line 103](../../../runtime/world2d.hpp#L103)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw_degrees` | `int32_t` | Input | Value supplied for `raw_degrees`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// int32_t raw_degrees

auto result = epok::detail::sine_quarter_2d(raw_degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-sine-series-2d-1"></a>

## `epok::detail::sine_series_2d`

**Purpose.** Performs `sine series 2d` as part of the world2d module.

**Exact declaration**

```cpp
constexpr double sine_series_2d(double x)
```

- **Declared at:** [line 81](../../../runtime/world2d.hpp#L81)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `double` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `double`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// double x

auto result = epok::detail::sine_series_2d(x);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-distance-squared-raw-2d-1"></a>

## `epok::distance_squared_raw_2d`

**Purpose.** Squared distance in raw Q24; |position| <= 8192 keeps this inside 2^51.

**Exact declaration**

```cpp
inline int64_t distance_squared_raw_2d(const Fixed* a, const Fixed* b)
```

- **Declared at:** [line 353](../../../runtime/world2d.hpp#L353)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Fixed *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Fixed *` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * a
// const Fixed * b

auto result = epok::distance_squared_raw_2d(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-draw-key-2d-1"></a>

## `epok::draw_key_2d`

**Purpose.** ---- draw order ---------------------------------------------------------------------- A single monotonically comparable key so the renderer never needs a multi-field comparator: layer (int8, coarse) above draw_order (int16) above the creation index (uint16, the tie breaker that makes the order stable and reproducible across frames).

**Details.** 40 significant bits; ascending key means "drawn later", i.e. on top.

**Exact declaration**

```cpp
inline uint64_t draw_key_2d(int8_t layer, int16_t draw_order, uint16_t creation)
```

- **Declared at:** [line 262](../../../runtime/world2d.hpp#L262)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `int8_t` | Input | Value supplied for `layer`. See the exact type and module contract. |
| `draw_order` | `int16_t` | Input | Value supplied for `draw_order`. See the exact type and module contract. |
| `creation` | `uint16_t` | Input | Value supplied for `creation`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** 40 significant bits; ascending key means "drawn later", i.e. on top.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// int8_t layer
// int16_t draw_order
// uint16_t creation

auto result = epok::draw_key_2d(layer, draw_order, creation);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-fixed-in-range-1"></a>

## `epok::fixed_in_range`

**Purpose.** Performs `fixed in range` as part of the world2d module.

**Exact declaration**

```cpp
inline bool fixed_in_range(Fixed value, Fixed limit)
```

- **Declared at:** [line 47](../../../runtime/world2d.hpp#L47)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `limit` | `Fixed` | Input | Value supplied for `limit`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// Fixed limit

auto result = epok::fixed_in_range(value, limit);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-local-matrix-1"></a>

## `epok::local_matrix`

**Purpose.** Local matrix of a Transform2D: translate * rotate * scale, applied in that order to a point (scale first, then rotation, then translation).

**Exact declaration**

```cpp
inline Affine2D local_matrix(const Transform2D& transform)
```

- **Declared at:** [line 167](../../../runtime/world2d.hpp#L167)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `transform` | `const Transform2D &` | Input | Value supplied for `transform`. See the exact type and module contract. |

**Returns.** Returns `Affine2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Transform2D & transform

auto result = epok::local_matrix(transform);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pick-2d-1"></a>

## `epok::pick_2d`

**Purpose.** Topmost entry under a screen point (highest draw key wins; ties go to the lowest index, which keeps the result stable).

**Details.** Returns false when nothing is hit. Shared by the runtime and the editor so both select the same object for the same click.

**Exact declaration**

```cpp
inline bool pick_2d(const Camera2D& camera, const Fixed* screen_xy, const Pick2DEntry* entries, size_t count, size_t& out_index)
```

- **Declared at:** [line 652](../../../runtime/world2d.hpp#L652)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `screen_xy` | `const Fixed *` | Input | Value supplied for `screen_xy`. See the exact type and module contract. |
| `entries` | `const Pick2DEntry *` | Input | Value supplied for `entries`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `out_index` | `size_t &` | Input/output; inspect the function contract | Value supplied for `out_index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Returns false when nothing is hit. Shared by the runtime and the editor so both select the same object for the same click.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera
// const Fixed * screen_xy
// const Pick2DEntry * entries
// size_t count
// size_t & out_index

auto result = epok::pick_2d(camera, screen_xy, entries, count, out_index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-screen-to-world-1"></a>

## `epok::screen_to_world`

**Purpose.** Exact inverse of world_to_screen up to Q12 rounding.

**Details.** A zoom of zero has no inverse and returns the camera position.

**Exact declaration**

```cpp
inline void screen_to_world(const Camera2D& camera, const Fixed* screen_xy, Fixed* world_xy)
```

- **Declared at:** [line 241](../../../runtime/world2d.hpp#L241)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `screen_xy` | `const Fixed *` | Input | Value supplied for `screen_xy`. See the exact type and module contract. |
| `world_xy` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `world_xy`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A zoom of zero has no inverse and returns the camera position.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera
// const Fixed * screen_xy
// Fixed * world_xy

epok::screen_to_world(camera, screen_xy, world_xy);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sin-degrees-1"></a>

## `epok::sin_degrees`

**Purpose.** Sine of an angle in degrees.

**Details.** Deterministic and identical on host and MIPS.

**Exact declaration**

```cpp
inline Fixed sin_degrees(Fixed degrees)
```

- **Declared at:** [line 120](../../../runtime/world2d.hpp#L120)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** Deterministic and identical on host and MIPS.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::sin_degrees(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sort-draw-order-1"></a>

## `epok::sort_draw_order`

**Purpose.** Bounded, allocation-free, stable insertion sort of an index array by draw key.

**Details.** `key` is any callable `uint64_t (uint16_t index)`. Insertion sort is the right shape here: the draw list is small (a few dozen sprites), already almost sorted between frames, and the algorithm needs no scratch memory.

**Exact declaration**

```cpp
template <class Key> inline void sort_draw_order(uint16_t* indices, size_t count, Key&& key)
```

- **Declared at:** [line 272](../../../runtime/world2d.hpp#L272)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `indices` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `indices`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `key` | `Key &&` | Consumed or moved input | Value supplied for `key`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** `key` is any callable `uint64_t (uint16_t index)`. Insertion sort is the right shape here: the draw list is small (a few dozen sprites), already almost sorted between frames, and the algorithm needs no scratch memory.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Key

// Assume these named values have been initialized with valid data:
// uint16_t * indices
// size_t count
// Key && key

epok::sort_draw_order<Key>(indices, count, key);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spatialhit2d-operator-bool-1"></a>

## `epok::SpatialHit2D::operator bool`

**Purpose.** Performs `operator  bool` as part of the world2d module.

**Exact declaration**

```cpp
explicit operator bool() const
```

- **Declared at:** [line 404](../../../runtime/world2d.hpp#L404)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

epok::SpatialHit2D& object = /* obtain a valid instance */;

auto result = object.operator bool();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-transform2d-clamp-1"></a>

## `epok::transform2d_clamp`

**Purpose.** Clamps into the documented range instead of rejecting.

**Details.** Scale is clamped to the smallest representable positive Q12 value rather than to zero.

**Exact declaration**

```cpp
inline void transform2d_clamp(Transform2D& transform)
```

- **Declared at:** [line 60](../../../runtime/world2d.hpp#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `transform` | `Transform2D &` | Input/output; inspect the function contract | Value supplied for `transform`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Scale is clamped to the smallest representable positive Q12 value rather than to zero.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// Transform2D & transform

epok::transform2d_clamp(transform);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-transform2d-valid-1"></a>

## `epok::transform2d_valid`

**Purpose.** Documented validity of a Transform2D.

**Details.** Rotation is always valid (it is wrapped), so only position and scale can fail.

**Exact declaration**

```cpp
inline bool transform2d_valid(const Transform2D& transform)
```

- **Declared at:** [line 51](../../../runtime/world2d.hpp#L51)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `transform` | `const Transform2D &` | Input | Value supplied for `transform`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Rotation is always valid (it is wrapped), so only position and scale can fail.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Transform2D & transform

auto result = epok::transform2d_valid(transform);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-matrix-2d-1"></a>

## `epok::world_matrix_2d`

**Purpose.** World matrix of a SceneComponent2D by walking its attach_parent chain.

**Details.** `resolve` is any callable `const SceneComponent2D* (ObjectId)`, so this never depends on Level internals and host tests can supply a plain array lookup. Returns false and leaves `out` untouched when the chain exceeds `world2d_depth_limit` or contains a cycle; a parent that fails to resolve ends the chain, exactly as a detached component would.

**Exact declaration**

```cpp
template <class Resolver> inline bool world_matrix_2d(const SceneComponent2D& leaf, Resolver&& resolve, Affine2D& out)
```

- **Declared at:** [line 185](../../../runtime/world2d.hpp#L185)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `leaf` | `const SceneComponent2D &` | Input | Value supplied for `leaf`. See the exact type and module contract. |
| `resolve` | `Resolver &&` | Consumed or moved input | Value supplied for `resolve`. See the exact type and module contract. |
| `out` | `Affine2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** `resolve` is any callable `const SceneComponent2D* (ObjectId)`, so this never depends on Level internals and host tests can supply a plain array lookup. Returns false and leaves `out` untouched when the chain exceeds `world2d_depth_limit` or contains a cycle; a parent that fails to resolve ends the chain, exactly as a detached component would.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Resolver

// Assume these named values have been initialized with valid data:
// const SceneComponent2D & leaf
// Resolver && resolve
// Affine2D & out

auto result = epok::world_matrix_2d<Resolver>(leaf, resolve, out);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-to-screen-1"></a>

## `epok::world_to_screen`

**Purpose.** Performs `world to screen` as part of the world2d module.

**Exact declaration**

```cpp
inline void world_to_screen(const Camera2D& camera, const Fixed* world_xy, Fixed* screen_xy)
```

- **Declared at:** [line 228](../../../runtime/world2d.hpp#L228)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `world_xy` | `const Fixed *` | Input | Value supplied for `world_xy`. See the exact type and module contract. |
| `screen_xy` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `screen_xy`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera
// const Fixed * world_xy
// Fixed * screen_xy

epok::world_to_screen(camera, world_xy, screen_xy);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
