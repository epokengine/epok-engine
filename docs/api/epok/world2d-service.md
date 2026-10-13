# Epok API: World2D Service

> **Header:** `"world2d_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_service.hpp)

This module covers the world2d service module. It documents 50 public callables declared directly in this header.

## Declared types

`epok::world2d::DrawItem`, `epok::world2d::DrawKind`, `epok::world2d::Projector`, `epok::world2d::Service`, `epok::world2d::ShapeQuad`, `epok::world2d::ShapeTriangle`, `epok::world2d::SpriteDraw`, `epok::world2d::Stats`, `epok::world2d::TileDraw`, `epok::world2d::Tints`

## Callable index

- [`epok::world2d::collect_sprites`](#epok-world2d-collect-sprites-1) — Performs `collect sprites` as part of the world2d service module.
- [`epok::world2d::each_line_quad`](#epok-world2d-each-line-quad-1) — The quads of a Line2D entry, one per segment, extended by half the width at inner joints so corners close.
- [`epok::world2d::each_particle`](#epok-world2d-each-particle-1) — The particles of an emitter entry as sprites (texel rectangle, colour, corners).
- [`epok::world2d::each_polygon_triangle`](#epok-world2d-each-polygon-triangle-1) — The triangles of a Polygon2D entry.
- [`epok::world2d::each_tile`](#epok-world2d-each-tile-1) — Every visible tile of a tile-layer entry, in row order.
- [`epok::world2d::entry_space`](#epok-world2d-entry-space-1) — The projection an expanded entry draws through, and its owner's world matrix.
- [`epok::world2d::floor_div64`](#epok-world2d-floor-div64-1) — Performs `floor div64` as part of the world2d service module.
- [`epok::world2d::install`](#epok-world2d-install-1) — Performs `install` as part of the world2d service module.
- [`epok::world2d::mix`](#epok-world2d-mix-1) — Performs `mix` as part of the world2d service module.
- [`epok::world2d::Projector::point`](#epok-world2d-projector-point-1) — Performs `point` as part of the world2d service module.
- [`epok::world2d::Projector::Projector`](#epok-world2d-projector-projector-1) — Constructs `epok::world2d::Projector` for the world2d service module.
- [`epok::world2d::Projector::screen`](#epok-world2d-projector-screen-1) — A projection that maps world pixels straight to screen pixels (screen canvases).
- [`epok::world2d::repeat_copies`](#epok-world2d-repeat-copies-1) — Screen copies of a drawable under a repeating Parallax2D: calls `visit(dx, dy)` for every shift (in screen pixels) whose copy of `bounds` (min x, min y, max x, max y) is on screen.
- [`epok::world2d::screen_pixel`](#epok-world2d-screen-pixel-1) — Performs `screen pixel` as part of the world2d service module.
- [`epok::world2d::Service::active_camera`](#epok-world2d-service-active-camera-1) — Performs `active camera` as part of the world2d service module.
- [`epok::world2d::Service::actor_at`](#epok-world2d-service-actor-at-1) — Performs `actor at` as part of the world2d service module.
- [`epok::world2d::Service::background`](#epok-world2d-service-background-1) — Background colour of the active camera, or null to keep the engine default.
- [`epok::world2d::Service::constrain`](#epok-world2d-service-constrain-1) — Performs `constrain` as part of the world2d service module.
- [`epok::world2d::Service::dispatch`](#epok-world2d-service-dispatch-1) — Performs `dispatch` as part of the world2d service module.
- [`epok::world2d::Service::each_particle_of`](#epok-world2d-service-each-particle-of-1) — Visits (position, age fraction) of every live particle of `emitter`, oldest slots first.
- [`epok::world2d::Service::edit_forget`](#epok-world2d-service-edit-forget-1) — Performs `edit forget` as part of the world2d service module.
- [`epok::world2d::Service::edit_lookup`](#epok-world2d-service-edit-lookup-1) — ---- runtime tile edits ------------------------------------------------------------
- [`epok::world2d::Service::edit_store`](#epok-world2d-service-edit-store-1) — Performs `edit store` as part of the world2d service module.
- [`epok::world2d::Service::exchange`](#epok-world2d-service-exchange-1) — A one-dimensional collision along `axis`, with the lower restitution of the two.
- [`epok::world2d::Service::follow`](#epok-world2d-service-follow-1) — Performs `follow` as part of the world2d service module.
- [`epok::world2d::Service::joint_ends`](#epok-world2d-service-joint-ends-1) — The two ends of a joint: positions, bodies (null for a fixed end) and whether valid.
- [`epok::world2d::Service::magnitude`](#epok-world2d-service-magnitude-1) — Performs `magnitude` as part of the world2d service module.
- [`epok::world2d::Service::move`](#epok-world2d-service-move-1) — Performs `move` as part of the world2d service module.
- [`epok::world2d::Service::navigate`](#epok-world2d-service-navigate-1) — ---- simulation step --------------------------------------------------------------- After the level tick: animate, resolve this step's colliders and triggers, move the cameras to where the actors ended up, then scroll the parallax layers after them.
- [`epok::world2d::Service::notice`](#epok-world2d-service-notice-1) — Whether a notifier's rectangle is inside the view; reports changes and, when asked, deactivates its owner off screen.
- [`epok::world2d::Service::overlap`](#epok-world2d-service-overlap-1) — Performs `overlap` as part of the world2d service module.
- [`epok::world2d::Service::plan`](#epok-world2d-service-plan-1) — Performs `plan` as part of the world2d service module.
- [`epok::world2d::Service::position_of`](#epok-world2d-service-position-of-1) — Performs `position of` as part of the world2d service module.
- [`epok::world2d::Service::raycast`](#epok-world2d-service-raycast-1) — Performs `raycast` as part of the world2d service module.
- [`epok::world2d::Service::release`](#epok-world2d-service-release-1) — ---- particles ---------------------------------------------------------------------
- [`epok::world2d::Service::reset`](#epok-world2d-service-reset-1) — Resets reset as part of the world2d service module.
- [`epok::world2d::Service::rigid_of`](#epok-world2d-service-rigid-of-1) — ---- rigid bodies --------------------------------------------------------------------
- [`epok::world2d::Service::simulate_bodies`](#epok-world2d-service-simulate-bodies-1) — Performs `simulate bodies` as part of the world2d service module.
- [`epok::world2d::Service::spring`](#epok-world2d-service-spring-1) — Performs `spring` as part of the world2d service module.
- [`epok::world2d::Service::step`](#epok-world2d-service-step-1) — Performs `step` as part of the world2d service module.
- [`epok::world2d::Service::sync`](#epok-world2d-service-sync-1) — ---- collision ---------------------------------------------------------------------
- [`epok::world2d::Service::tile_at`](#epok-world2d-service-tile-at-1) — Performs `tile at` as part of the world2d service module.
- [`epok::world2d::Service::times`](#epok-world2d-service-times-1) — Performs `times` as part of the world2d service module.
- [`epok::world2d::sort_draw_items`](#epok-world2d-sort-draw-items-1) — Performs `sort draw items` as part of the world2d service module.
- [`epok::world2d::tile_box`](#epok-world2d-tile-box-1) — World box of one tile's collision shape, honouring the cell's flip and transpose flags.
- [`epok::world2d::tint`](#epok-world2d-tint-1) — Performs `tint` as part of the world2d service module.
- [`epok::world2d::Tints::of`](#epok-world2d-tints-of-1) — Performs `of` as part of the world2d service module.
- [`epok::world2d::Tints::Tints`](#epok-world2d-tints-tints-1) — Constructs `epok::world2d::Tints` for the world2d service module.
- [`epok::world2d::view_camera`](#epok-world2d-view-camera-1) — Resolves every visible sprite and tile layer through the active camera (or the screen, for a screen canvas) into `draws`, and writes the back-to-front order into `items` (`scratch` is sort workspace).
- [`epok::world2d::z_key`](#epok-world2d-z-key-1) — Performs `z key` as part of the world2d service module.

<a id="epok-world2d-collect-sprites-1"></a>

## `epok::world2d::collect_sprites`

**Purpose.** Performs `collect sprites` as part of the world2d service module.

**Exact declaration**

```cpp
template<class TextureSize> size_t collect_sprites(SpriteDraw* draws, DrawItem* items, DrawItem* scratch, size_t capacity, TextureSize&& texture_size)
```

- **Declared at:** [line 1187](../../../runtime/world2d_service.hpp#L1187)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `draws` | `SpriteDraw *` | Input/output; inspect the function contract | Value supplied for `draws`. See the exact type and module contract. |
| `items` | `DrawItem *` | Input/output; inspect the function contract | Value supplied for `items`. See the exact type and module contract. |
| `scratch` | `DrawItem *` | Input/output; inspect the function contract | Value supplied for `scratch`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `texture_size` | `TextureSize &&` | Consumed or moved input | Value supplied for `texture_size`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// TextureSize

// Assume these named values have been initialized with valid data:
// SpriteDraw * draws
// DrawItem * items
// DrawItem * scratch
// size_t capacity
// TextureSize && texture_size

epok::world2d& object = /* obtain a valid instance */;

auto result = object.collect_sprites<TextureSize>(draws, items, scratch, capacity, texture_size);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-each-line-quad-1"></a>

## `epok::world2d::each_line_quad`

**Purpose.** The quads of a Line2D entry, one per segment, extended by half the width at inner joints so corners close.

**Exact declaration**

```cpp
template<class Visit> inline void each_line_quad(const SpriteDraw& entry, Visit&& visit)
```

- **Declared at:** [line 1414](../../../runtime/world2d_service.hpp#L1414)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const SpriteDraw &` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const SpriteDraw & entry
// Visit && visit

epok::world2d& object = /* obtain a valid instance */;

object.each_line_quad<Visit>(entry, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-each-particle-1"></a>

## `epok::world2d::each_particle`

**Purpose.** The particles of an emitter entry as sprites (texel rectangle, colour, corners).

**Exact declaration**

```cpp
template<class Visit> inline void each_particle(const SpriteDraw& entry, uint16_t texture_width, uint16_t texture_height, Visit&& visit)
```

- **Declared at:** [line 1502](../../../runtime/world2d_service.hpp#L1502)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const SpriteDraw &` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `texture_width` | `uint16_t` | Input | Value supplied for `texture_width`. See the exact type and module contract. |
| `texture_height` | `uint16_t` | Input | Value supplied for `texture_height`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const SpriteDraw & entry
// uint16_t texture_width
// uint16_t texture_height
// Visit && visit

epok::world2d& object = /* obtain a valid instance */;

object.each_particle<Visit>(entry, texture_width, texture_height, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-each-polygon-triangle-1"></a>

## `epok::world2d::each_polygon_triangle`

**Purpose.** The triangles of a Polygon2D entry.

**Exact declaration**

```cpp
template<class Visit> inline void each_polygon_triangle(const SpriteDraw& entry, Visit&& visit)
```

- **Declared at:** [line 1468](../../../runtime/world2d_service.hpp#L1468)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const SpriteDraw &` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const SpriteDraw & entry
// Visit && visit

epok::world2d& object = /* obtain a valid instance */;

object.each_polygon_triangle<Visit>(entry, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-each-tile-1"></a>

## `epok::world2d::each_tile`

**Purpose.** Every visible tile of a tile-layer entry, in row order.

**Details.** `texture_width/height` bound the atlas; tiles whose frame falls outside it are skipped.

**Exact declaration**

```cpp
template<class Visit> inline void each_tile(const SpriteDraw& entry, uint16_t texture_width, uint16_t texture_height, Visit&& visit)
```

- **Declared at:** [line 1548](../../../runtime/world2d_service.hpp#L1548)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const SpriteDraw &` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `texture_width` | `uint16_t` | Input | Value supplied for `texture_width`. See the exact type and module contract. |
| `texture_height` | `uint16_t` | Input | Value supplied for `texture_height`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** `texture_width/height` bound the atlas; tiles whose frame falls outside it are skipped.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const SpriteDraw & entry
// uint16_t texture_width
// uint16_t texture_height
// Visit && visit

epok::world2d& object = /* obtain a valid instance */;

object.each_tile<Visit>(entry, texture_width, texture_height, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-entry-space-1"></a>

## `epok::world2d::entry_space`

**Purpose.** The projection an expanded entry draws through, and its owner's world matrix.

**Exact declaration**

```cpp
inline bool entry_space(const SpriteDraw& entry, Camera2D& camera, Affine2D& matrix)
```

- **Declared at:** [line 1398](../../../runtime/world2d_service.hpp#L1398)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const SpriteDraw &` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `camera` | `Camera2D &` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |
| `matrix` | `Affine2D &` | Input/output; inspect the function contract | Value supplied for `matrix`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const SpriteDraw & entry
// Camera2D & camera
// Affine2D & matrix

auto result = epok::world2d::entry_space(entry, camera, matrix);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-floor-div64-1"></a>

## `epok::world2d::floor_div64`

**Purpose.** Performs `floor div64` as part of the world2d service module.

**Exact declaration**

```cpp
inline int64_t floor_div64(int64_t value, int64_t size)
```

- **Declared at:** [line 166](../../../runtime/world2d_service.hpp#L166)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `size` | `int64_t` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value
// int64_t size

auto result = epok::world2d::floor_div64(value, size);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-install-1"></a>

## `epok::world2d::install`

**Purpose.** Performs `install` as part of the world2d service module.

**Exact declaration**

```cpp
inline void install()
```

- **Declared at:** [line 1033](../../../runtime/world2d_service.hpp#L1033)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

epok::world2d::install();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-mix-1"></a>

## `epok::world2d::mix`

**Purpose.** Performs `mix` as part of the world2d service module.

**Exact declaration**

```cpp
inline uint8_t mix(uint8_t a, uint8_t b, Fixed t, uint8_t tint)
```

- **Declared at:** [line 1408](../../../runtime/world2d_service.hpp#L1408)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint8_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint8_t` | Input | Value supplied for `b`. See the exact type and module contract. |
| `t` | `Fixed` | Input | Value supplied for `t`. See the exact type and module contract. |
| `tint` | `uint8_t` | Input | Value supplied for `tint`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t a
// uint8_t b
// Fixed t
// uint8_t tint

auto result = epok::world2d::mix(a, b, t, tint);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-projector-point-1"></a>

## `epok::world2d::Projector::point`

**Purpose.** Performs `point` as part of the world2d service module.

**Exact declaration**

```cpp
void point(const Fixed* world, Fixed* screen) const
```

- **Declared at:** [line 118](../../../runtime/world2d_service.hpp#L118)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const Fixed *` | Input | Value supplied for `world`. See the exact type and module contract. |
| `screen` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * world
// Fixed * screen

epok::world2d::Projector& object = /* obtain a valid instance */;

object.point(world, screen);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-projector-projector-1"></a>

## `epok::world2d::Projector::Projector`

**Purpose.** Constructs `epok::world2d::Projector` for the world2d service module.

**Exact declaration**

```cpp
explicit Projector(const Camera2D& view) : c
```

- **Declared at:** [line 102](../../../runtime/world2d_service.hpp#L102)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `view` | `const Camera2D &` | Input | Value supplied for `view`. See the exact type and module contract. |

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & view

epok::world2d::Projector value(view);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-projector-screen-1"></a>

## `epok::world2d::Projector::screen`

**Purpose.** A projection that maps world pixels straight to screen pixels (screen canvases).

**Exact declaration**

```cpp
static Projector screen()
```

- **Declared at:** [line 112](../../../runtime/world2d_service.hpp#L112)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Projector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

auto result = epok::world2d::Projector::screen();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-repeat-copies-1"></a>

## `epok::world2d::repeat_copies`

**Purpose.** Screen copies of a drawable under a repeating Parallax2D: calls `visit(dx, dy)` for every shift (in screen pixels) whose copy of `bounds` (min x, min y, max x, max y) is on screen.

**Exact declaration**

```cpp
template<class Visit> inline void repeat_copies(const Canvas& canvas, const Projector& projector, const int16_t* bounds, Visit&& visit)
```

- **Declared at:** [line 1109](../../../runtime/world2d_service.hpp#L1109)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `canvas` | `const Canvas &` | Input | Value supplied for `canvas`. See the exact type and module contract. |
| `projector` | `const Projector &` | Input | Value supplied for `projector`. See the exact type and module contract. |
| `bounds` | `const int16_t *` | Input | Value supplied for `bounds`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const Canvas & canvas
// const Projector & projector
// const int16_t * bounds
// Visit && visit

epok::world2d& object = /* obtain a valid instance */;

object.repeat_copies<Visit>(canvas, projector, bounds, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-screen-pixel-1"></a>

## `epok::world2d::screen_pixel`

**Purpose.** Performs `screen pixel` as part of the world2d service module.

**Exact declaration**

```cpp
inline int16_t screen_pixel(Fixed value)
```

- **Declared at:** [line 131](../../../runtime/world2d_service.hpp#L131)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

auto result = epok::world2d::screen_pixel(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-active-camera-1"></a>

## `epok::world2d::Service::active_camera`

**Purpose.** Performs `active camera` as part of the world2d service module.

**Exact declaration**

```cpp
Camera2DComponent* active_camera()
```

- **Declared at:** [line 1004](../../../runtime/world2d_service.hpp#L1004)
- **Kind:** `cxx method`

**Returns.** Returns `Camera2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.active_camera();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-actor-at-1"></a>

## `epok::world2d::Service::actor_at`

**Purpose.** Performs `actor at` as part of the world2d service module.

**Exact declaration**

```cpp
ObjectId actor_at(int index) const
```

- **Declared at:** [line 536](../../../runtime/world2d_service.hpp#L536)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.actor_at(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-background-1"></a>

## `epok::world2d::Service::background`

**Purpose.** Background colour of the active camera, or null to keep the engine default.

**Exact declaration**

```cpp
const uint8_t* background()
```

- **Declared at:** [line 1012](../../../runtime/world2d_service.hpp#L1012)
- **Kind:** `cxx method`

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.background();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-constrain-1"></a>

## `epok::world2d::Service::constrain`

**Purpose.** Performs `constrain` as part of the world2d service module.

**Exact declaration**

```cpp
void constrain(Joint2DComponent& joint)
```

- **Declared at:** [line 861](../../../runtime/world2d_service.hpp#L861)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `joint` | `Joint2DComponent &` | Input/output; inspect the function contract | Value supplied for `joint`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Joint2DComponent & joint

epok::world2d::Service& object = /* obtain a valid instance */;

object.constrain(joint);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-dispatch-1"></a>

## `epok::world2d::Service::dispatch`

**Purpose.** Performs `dispatch` as part of the world2d service module.

**Exact declaration**

```cpp
template<class LevelType> static void dispatch(LevelType& level, ObjectId actor, ObjectId other, TriggerPhase phase)
```

- **Declared at:** [line 993](../../../runtime/world2d_service.hpp#L993)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `level` | `LevelType &` | Input/output; inspect the function contract | Value supplied for `level`. See the exact type and module contract. |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `other` | `ObjectId` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// LevelType

// Assume these named values have been initialized with valid data:
// LevelType & level
// ObjectId actor
// ObjectId other
// TriggerPhase phase

epok::world2d::Service::dispatch<LevelType>(level, actor, other, phase);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-each-particle-of-1"></a>

## `epok::world2d::Service::each_particle_of`

**Purpose.** Visits (position, age fraction) of every live particle of `emitter`, oldest slots first.

**Details.** Positions are world pixels, or emitter-local with local coordinates.

**Exact declaration**

```cpp
template<class Visit> void each_particle_of(const Particles2DComponent& emitter, Visit&& visit) const
```

- **Declared at:** [line 584](../../../runtime/world2d_service.hpp#L584)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `emitter` | `const Particles2DComponent &` | Input | Value supplied for `emitter`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Positions are world pixels, or emitter-local with local coordinates.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// const Particles2DComponent & emitter
// Visit && visit

epok::world2d::Service& object = /* obtain a valid instance */;

object.each_particle_of<Visit>(emitter, visit);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-edit-forget-1"></a>

## `epok::world2d::Service::edit_forget`

**Purpose.** Performs `edit forget` as part of the world2d service module.

**Exact declaration**

```cpp
void edit_forget(TileMapLayerComponent& layer)
```

- **Declared at:** [line 624](../../../runtime/world2d_service.hpp#L624)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `TileMapLayerComponent &` | Input/output; inspect the function contract | Value supplied for `layer`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// TileMapLayerComponent & layer

epok::world2d::Service& object = /* obtain a valid instance */;

object.edit_forget(layer);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-edit-lookup-1"></a>

## `epok::world2d::Service::edit_lookup`

**Purpose.** ---- runtime tile edits ------------------------------------------------------------

**Exact declaration**

```cpp
void edit_lookup(const TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t& value) const
```

- **Declared at:** [line 590](../../../runtime/world2d_service.hpp#L590)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `const TileMapLayerComponent &` | Input | Value supplied for `layer`. See the exact type and module contract. |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `value` | `uint16_t &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const TileMapLayerComponent & layer
// int32_t x
// int32_t y
// uint16_t & value

epok::world2d::Service& object = /* obtain a valid instance */;

object.edit_lookup(layer, x, y, value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-edit-store-1"></a>

## `epok::world2d::Service::edit_store`

**Purpose.** Performs `edit store` as part of the world2d service module.

**Exact declaration**

```cpp
bool edit_store(TileMapLayerComponent& layer, int32_t x, int32_t y, uint16_t value)
```

- **Declared at:** [line 595](../../../runtime/world2d_service.hpp#L595)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `TileMapLayerComponent &` | Input/output; inspect the function contract | Value supplied for `layer`. See the exact type and module contract. |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// TileMapLayerComponent & layer
// int32_t x
// int32_t y
// uint16_t value

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.edit_store(layer, x, y, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-exchange-1"></a>

## `epok::world2d::Service::exchange`

**Purpose.** A one-dimensional collision along `axis`, with the lower restitution of the two.

**Exact declaration**

```cpp
static void exchange(RigidBody2DComponent& a, RigidBody2DComponent& b, int axis)
```

- **Declared at:** [line 731](../../../runtime/world2d_service.hpp#L731)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `RigidBody2DComponent &` | Input/output; inspect the function contract | Value supplied for `a`. See the exact type and module contract. |
| `b` | `RigidBody2DComponent &` | Input/output; inspect the function contract | Value supplied for `b`. See the exact type and module contract. |
| `axis` | `int` | Input | Value supplied for `axis`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// RigidBody2DComponent & a
// RigidBody2DComponent & b
// int axis

epok::world2d::Service::exchange(a, b, axis);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-follow-1"></a>

## `epok::world2d::Service::follow`

**Purpose.** Performs `follow` as part of the world2d service module.

**Exact declaration**

```cpp
static void follow(NavigationAgent2DComponent& agent, const Fixed* at, Fixed dt, SceneComponent2D& root)
```

- **Declared at:** [line 684](../../../runtime/world2d_service.hpp#L684)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `agent` | `NavigationAgent2DComponent &` | Input/output; inspect the function contract | Value supplied for `agent`. See the exact type and module contract. |
| `at` | `const Fixed *` | Input | Value supplied for `at`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `root` | `SceneComponent2D &` | Input/output; inspect the function contract | Value supplied for `root`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// NavigationAgent2DComponent & agent
// const Fixed * at
// Fixed dt
// SceneComponent2D & root

epok::world2d::Service::follow(agent, at, dt, root);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-joint-ends-1"></a>

## `epok::world2d::Service::joint_ends`

**Purpose.** The two ends of a joint: positions, bodies (null for a fixed end) and whether valid.

**Exact declaration**

```cpp
bool joint_ends(Joint2DComponent& joint, Fixed* pa, Fixed* pb, RigidBody2DComponent*& a, RigidBody2DComponent*& b)
```

- **Declared at:** [line 831](../../../runtime/world2d_service.hpp#L831)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `joint` | `Joint2DComponent &` | Input/output; inspect the function contract | Value supplied for `joint`. See the exact type and module contract. |
| `pa` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `pa`. See the exact type and module contract. |
| `pb` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `pb`. See the exact type and module contract. |
| `a` | `RigidBody2DComponent *&` | Input/output; inspect the function contract | Value supplied for `a`. See the exact type and module contract. |
| `b` | `RigidBody2DComponent *&` | Input/output; inspect the function contract | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Joint2DComponent & joint
// Fixed * pa
// Fixed * pb
// RigidBody2DComponent *& a
// RigidBody2DComponent *& b

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.joint_ends(joint, pa, pb, a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-magnitude-1"></a>

## `epok::world2d::Service::magnitude`

**Purpose.** Performs `magnitude` as part of the world2d service module.

**Exact declaration**

```cpp
static Fixed magnitude(Fixed v)
```

- **Declared at:** [line 722](../../../runtime/world2d_service.hpp#L722)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Fixed` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed v

auto result = epok::world2d::Service::magnitude(v);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-move-1"></a>

## `epok::world2d::Service::move`

**Purpose.** Performs `move` as part of the world2d service module.

**Exact declaration**

```cpp
bool move(ActorComponent& body, const Fixed* delta, uint32_t mask, MoveResult2D& result)
```

- **Declared at:** [line 510](../../../runtime/world2d_service.hpp#L510)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `body` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `body`. See the exact type and module contract. |
| `delta` | `const Fixed *` | Input | Value supplied for `delta`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `result` | `MoveResult2D &` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & body
// const Fixed * delta
// uint32_t mask
// MoveResult2D & result

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.move(body, delta, mask, result);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-navigate-1"></a>

## `epok::world2d::Service::navigate`

**Purpose.** ---- simulation step --------------------------------------------------------------- After the level tick: animate, resolve this step's colliders and triggers, move the cameras to where the actors ended up, then scroll the parallax layers after them.

**Details.** ---- navigation ---------------------------------------------------------------------- Agents follow their target actor, plan when asked (one path per step for the whole map, so a crowd never stalls a frame), then walk their path.

**Exact declaration**

```cpp
void navigate(Fixed dt)
```

- **Declared at:** [line 639](../../../runtime/world2d_service.hpp#L639)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** ---- navigation ---------------------------------------------------------------------- Agents follow their target actor, plan when asked (one path per step for the whole map, so a crowd never stalls a frame), then walk their path.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::world2d::Service& object = /* obtain a valid instance */;

object.navigate(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-notice-1"></a>

## `epok::world2d::Service::notice`

**Purpose.** Whether a notifier's rectangle is inside the view; reports changes and, when asked, deactivates its owner off screen.

**Exact declaration**

```cpp
static void notice(VisibleOnScreenNotifier2DComponent& notifier, const Projector& projector)
```

- **Declared at:** [line 969](../../../runtime/world2d_service.hpp#L969)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `notifier` | `VisibleOnScreenNotifier2DComponent &` | Input/output; inspect the function contract | Value supplied for `notifier`. See the exact type and module contract. |
| `projector` | `const Projector &` | Input | Value supplied for `projector`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// VisibleOnScreenNotifier2DComponent & notifier
// const Projector & projector

epok::world2d::Service::notice(notifier, projector);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-overlap-1"></a>

## `epok::world2d::Service::overlap`

**Purpose.** Performs `overlap` as part of the world2d service module.

**Exact declaration**

```cpp
size_t overlap(const Aabb2D& box, ObjectId* actors, size_t capacity, uint32_t mask, bool triggers)
```

- **Declared at:** [line 553](../../../runtime/world2d_service.hpp#L553)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Aabb2D &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `actors` | `ObjectId *` | Input/output; inspect the function contract | Value supplied for `actors`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const Aabb2D & box
// ObjectId * actors
// size_t capacity
// uint32_t mask
// bool triggers

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.overlap(box, actors, capacity, mask, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-plan-1"></a>

## `epok::world2d::Service::plan`

**Purpose.** Performs `plan` as part of the world2d service module.

**Exact declaration**

```cpp
static void plan(NavigationAgent2DComponent& agent, const Fixed* at)
```

- **Declared at:** [line 667](../../../runtime/world2d_service.hpp#L667)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `agent` | `NavigationAgent2DComponent &` | Input/output; inspect the function contract | Value supplied for `agent`. See the exact type and module contract. |
| `at` | `const Fixed *` | Input | Value supplied for `at`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// NavigationAgent2DComponent & agent
// const Fixed * at

epok::world2d::Service::plan(agent, at);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-position-of-1"></a>

## `epok::world2d::Service::position_of`

**Purpose.** Performs `position of` as part of the world2d service module.

**Exact declaration**

```cpp
static bool position_of(ActorComponent& component, Fixed* out)
```

- **Declared at:** [line 723](../../../runtime/world2d_service.hpp#L723)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & component
// Fixed * out

auto result = epok::world2d::Service::position_of(component, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-raycast-1"></a>

## `epok::world2d::Service::raycast`

**Purpose.** Performs `raycast` as part of the world2d service module.

**Exact declaration**

```cpp
SpatialHit2D raycast(const Fixed* origin, const Fixed* delta, uint32_t mask, bool triggers)
```

- **Declared at:** [line 547](../../../runtime/world2d_service.hpp#L547)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Fixed *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `delta` | `const Fixed *` | Input | Value supplied for `delta`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * origin
// const Fixed * delta
// uint32_t mask
// bool triggers

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.raycast(origin, delta, mask, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-release-1"></a>

## `epok::world2d::Service::release`

**Purpose.** ---- particles ---------------------------------------------------------------------

**Exact declaration**

```cpp
void release(Particles2DComponent& emitter)
```

- **Declared at:** [line 577](../../../runtime/world2d_service.hpp#L577)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `emitter` | `Particles2DComponent &` | Input/output; inspect the function contract | Value supplied for `emitter`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Particles2DComponent & emitter

epok::world2d::Service& object = /* obtain a valid instance */;

object.release(emitter);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-reset-1"></a>

## `epok::world2d::Service::reset`

**Purpose.** Resets reset as part of the world2d service module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 1016](../../../runtime/world2d_service.hpp#L1016)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

epok::world2d::Service& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-rigid-of-1"></a>

## `epok::world2d::Service::rigid_of`

**Purpose.** ---- rigid bodies --------------------------------------------------------------------

**Exact declaration**

```cpp
static RigidBody2DComponent* rigid_of(ObjectId actor)
```

- **Declared at:** [line 716](../../../runtime/world2d_service.hpp#L716)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `RigidBody2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::world2d::Service::rigid_of(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-simulate-bodies-1"></a>

## `epok::world2d::Service::simulate_bodies`

**Purpose.** Performs `simulate bodies` as part of the world2d service module.

**Exact declaration**

```cpp
void simulate_bodies(Fixed dt)
```

- **Declared at:** [line 744](../../../runtime/world2d_service.hpp#L744)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::world2d::Service& object = /* obtain a valid instance */;

object.simulate_bodies(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-spring-1"></a>

## `epok::world2d::Service::spring`

**Purpose.** Performs `spring` as part of the world2d service module.

**Exact declaration**

```cpp
void spring(Joint2DComponent& joint)
```

- **Declared at:** [line 846](../../../runtime/world2d_service.hpp#L846)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `joint` | `Joint2DComponent &` | Input/output; inspect the function contract | Value supplied for `joint`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Joint2DComponent & joint

epok::world2d::Service& object = /* obtain a valid instance */;

object.spring(joint);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-step-1"></a>

## `epok::world2d::Service::step`

**Purpose.** Performs `step` as part of the world2d service module.

**Exact declaration**

```cpp
template<class LevelType> void step(Fixed dt, LevelType* level)
```

- **Declared at:** [line 897](../../../runtime/world2d_service.hpp#L897)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `level` | `LevelType *` | Input/output; inspect the function contract | Value supplied for `level`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// LevelType

// Assume these named values have been initialized with valid data:
// Fixed dt
// LevelType * level

epok::world2d::Service& object = /* obtain a valid instance */;

object.step<LevelType>(dt, level);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-sync-1"></a>

## `epok::world2d::Service::sync`

**Purpose.** ---- collision ---------------------------------------------------------------------

**Exact declaration**

```cpp
void sync(bool force = false)
```

- **Declared at:** [line 487](../../../runtime/world2d_service.hpp#L487)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `force` | `bool` | Input | Value supplied for `force`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// bool force

epok::world2d::Service& object = /* obtain a valid instance */;

object.sync(force);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-service-tile-at-1"></a>

## `epok::world2d::Service::tile_at`

**Purpose.** Performs `tile at` as part of the world2d service module.

**Exact declaration**

```cpp
bool tile_at(int index, int32_t& x, int32_t& y) const
```

- **Declared at:** [line 541](../../../runtime/world2d_service.hpp#L541)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `x` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// int index
// int32_t & x
// int32_t & y

epok::world2d::Service& object = /* obtain a valid instance */;

auto result = object.tile_at(index, x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-service-times-1"></a>

## `epok::world2d::Service::times`

**Purpose.** Performs `times` as part of the world2d service module.

**Exact declaration**

```cpp
static Fixed times(Fixed value, int64_t q12)
```

- **Declared at:** [line 721](../../../runtime/world2d_service.hpp#L721)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `q12` | `int64_t` | Input | Value supplied for `q12`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// int64_t q12

auto result = epok::world2d::Service::times(value, q12);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-sort-draw-items-1"></a>

## `epok::world2d::sort_draw_items`

**Purpose.** Performs `sort draw items` as part of the world2d service module.

**Exact declaration**

```cpp
inline void sort_draw_items(DrawItem* items, DrawItem* scratch, size_t count)
```

- **Declared at:** [line 74](../../../runtime/world2d_service.hpp#L74)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `items` | `DrawItem *` | Input/output; inspect the function contract | Value supplied for `items`. See the exact type and module contract. |
| `scratch` | `DrawItem *` | Input/output; inspect the function contract | Value supplied for `scratch`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// DrawItem * items
// DrawItem * scratch
// size_t count

epok::world2d::sort_draw_items(items, scratch, count);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-tile-box-1"></a>

## `epok::world2d::tile_box`

**Purpose.** World box of one tile's collision shape, honouring the cell's flip and transpose flags.

**Details.** A cell maps screen to texels by transposing first (when set), then mirroring the texel axes: flip_h mirrors the tile's columns, flip_v its rows.

**Exact declaration**

```cpp
inline bool tile_box(const TileInfo& tile, uint16_t value, const Fixed* cell_origin, uint16_t width, uint16_t height, Aabb2D& out)
```

- **Declared at:** [line 141](../../../runtime/world2d_service.hpp#L141)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `tile` | `const TileInfo &` | Input | Value supplied for `tile`. See the exact type and module contract. |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `cell_origin` | `const Fixed *` | Input | Value supplied for `cell_origin`. See the exact type and module contract. |
| `width` | `uint16_t` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `uint16_t` | Input | Value supplied for `height`. See the exact type and module contract. |
| `out` | `Aabb2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A cell maps screen to texels by transposing first (when set), then mirroring the texel axes: flip_h mirrors the tile's columns, flip_v its rows.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const TileInfo & tile
// uint16_t value
// const Fixed * cell_origin
// uint16_t width
// uint16_t height
// Aabb2D & out

auto result = epok::world2d::tile_box(tile, value, cell_origin, width, height, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-tint-1"></a>

## `epok::world2d::tint`

**Purpose.** Performs `tint` as part of the world2d service module.

**Exact declaration**

```cpp
inline void tint(uint8_t* color, const uint8_t* by)
```

- **Declared at:** [line 1102](../../../runtime/world2d_service.hpp#L1102)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `color` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `color`. See the exact type and module contract. |
| `by` | `const uint8_t *` | Input | Value supplied for `by`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t * color
// const uint8_t * by

epok::world2d::tint(color, by);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-tints-of-1"></a>

## `epok::world2d::Tints::of`

**Purpose.** Performs `of` as part of the world2d service module.

**Exact declaration**

```cpp
const uint8_t* of(int8_t layer) const
```

- **Declared at:** [line 1147](../../../runtime/world2d_service.hpp#L1147)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `int8_t` | Input | Value supplied for `layer`. See the exact type and module contract. |

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// int8_t layer

epok::world2d::Tints& object = /* obtain a valid instance */;

auto result = object.of(layer);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-tints-tints-1"></a>

## `epok::world2d::Tints::Tints`

**Purpose.** Constructs `epok::world2d::Tints` for the world2d service module.

**Exact declaration**

```cpp
Tints()
```

- **Declared at:** [line 1137](../../../runtime/world2d_service.hpp#L1137)
- **Kind:** `constructor`

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

epok::world2d::Tints value();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-view-camera-1"></a>

## `epok::world2d::view_camera`

**Purpose.** Resolves every visible sprite and tile layer through the active camera (or the screen, for a screen canvas) into `draws`, and writes the back-to-front order into `items` (`scratch` is sort workspace).

**Details.** `texture_size(index, width, height)` reports a texture's size and returns false when it is absent. Returns the number of entries to draw; extra entries are counted as dropped. The camera of the view being collected: the render pass's, else the active one.

**Exact declaration**

```cpp
inline bool view_camera(Camera2D& out)
```

- **Declared at:** [line 1164](../../../runtime/world2d_service.hpp#L1164)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `Camera2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** `texture_size(index, width, height)` reports a texture's size and returns false when it is absent. Returns the number of entries to draw; extra entries are counted as dropped. The camera of the view being collected: the render pass's, else the active one.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// Camera2D & out

auto result = epok::world2d::view_camera(out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-z-key-1"></a>

## `epok::world2d::z_key`

**Purpose.** Performs `z key` as part of the world2d service module.

**Exact declaration**

```cpp
inline uint32_t z_key(int32_t z)
```

- **Declared at:** [line 1153](../../../runtime/world2d_service.hpp#L1153)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// int32_t z

auto result = epok::world2d::z_key(z);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
