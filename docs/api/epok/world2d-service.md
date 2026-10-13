# Epok API: World2D Service

> **Header:** `"world2d_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_service.hpp)

This module covers the world2d service module. It documents 16 public callables declared directly in this header.

## Declared types

`epok::world2d::DrawItem`, `epok::world2d::Projector`, `epok::world2d::Service`, `epok::world2d::SpriteDraw`, `epok::world2d::Stats`

## Callable index

- [`epok::world2d::collect_sprites`](#epok-world2d-collect-sprites-1) — Resolves every visible sprite through the active camera into `draws`, and writes the back-to-front order into `items` (`scratch` is sort workspace).
- [`epok::world2d::install`](#epok-world2d-install-1) — Performs `install` as part of the world2d service module.
- [`epok::world2d::Projector::point`](#epok-world2d-projector-point-1) — Performs `point` as part of the world2d service module.
- [`epok::world2d::Projector::Projector`](#epok-world2d-projector-projector-1) — Constructs `epok::world2d::Projector` for the world2d service module.
- [`epok::world2d::screen_pixel`](#epok-world2d-screen-pixel-1) — Performs `screen pixel` as part of the world2d service module.
- [`epok::world2d::Service::active_camera`](#epok-world2d-service-active-camera-1) — Performs `active camera` as part of the world2d service module.
- [`epok::world2d::Service::actor_at`](#epok-world2d-service-actor-at-1) — Performs `actor at` as part of the world2d service module.
- [`epok::world2d::Service::background`](#epok-world2d-service-background-1) — Background colour of the active camera, or null to keep the engine default.
- [`epok::world2d::Service::dispatch`](#epok-world2d-service-dispatch-1) — Performs `dispatch` as part of the world2d service module.
- [`epok::world2d::Service::move`](#epok-world2d-service-move-1) — Performs `move` as part of the world2d service module.
- [`epok::world2d::Service::overlap`](#epok-world2d-service-overlap-1) — Performs `overlap` as part of the world2d service module.
- [`epok::world2d::Service::raycast`](#epok-world2d-service-raycast-1) — Performs `raycast` as part of the world2d service module.
- [`epok::world2d::Service::reset`](#epok-world2d-service-reset-1) — Resets reset as part of the world2d service module.
- [`epok::world2d::Service::step`](#epok-world2d-service-step-1) — ---- simulation step --------------------------------------------------------------- After the level tick: animate, resolve this step's colliders and triggers, then move the cameras to where the actors ended up.
- [`epok::world2d::Service::sync`](#epok-world2d-service-sync-1) — ---- collision ---------------------------------------------------------------------
- [`epok::world2d::sort_draw_items`](#epok-world2d-sort-draw-items-1) — Performs `sort draw items` as part of the world2d service module.

<a id="epok-world2d-collect-sprites-1"></a>

## `epok::world2d::collect_sprites`

**Purpose.** Resolves every visible sprite through the active camera into `draws`, and writes the back-to-front order into `items` (`scratch` is sort workspace).

**Details.** `texture_size(index, width, height)` reports a texture's size and returns false when it is absent. Returns the number of sprites to draw; extra sprites are counted as dropped.

**Exact declaration**

```cpp
template<class TextureSize> size_t collect_sprites(SpriteDraw* draws, DrawItem* items, DrawItem* scratch, size_t capacity, TextureSize&& texture_size)
```

- **Declared at:** [line 327](../../../runtime/world2d_service.hpp#L327)
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

**Use it when.** `texture_size(index, width, height)` reports a texture's size and returns false when it is absent. Returns the number of sprites to draw; extra sprites are counted as dropped.

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

<a id="epok-world2d-install-1"></a>

## `epok::world2d::install`

**Purpose.** Performs `install` as part of the world2d service module.

**Exact declaration**

```cpp
inline void install()
```

- **Declared at:** [line 293](../../../runtime/world2d_service.hpp#L293)
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

<a id="epok-world2d-projector-point-1"></a>

## `epok::world2d::Projector::point`

**Purpose.** Performs `point` as part of the world2d service module.

**Exact declaration**

```cpp
void point(const Fixed* world, Fixed* screen) const
```

- **Declared at:** [line 79](../../../runtime/world2d_service.hpp#L79)
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
explicit Projector(const Camera2D& camera)
```

- **Declared at:** [line 70](../../../runtime/world2d_service.hpp#L70)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Camera2D &` | Input | Value supplied for `camera`. See the exact type and module contract. |

**Use it when.** You need the world2d service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_service.hpp"

// Assume these named values have been initialized with valid data:
// const Camera2D & camera

epok::world2d::Projector value(camera);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-screen-pixel-1"></a>

## `epok::world2d::screen_pixel`

**Purpose.** Performs `screen pixel` as part of the world2d service module.

**Exact declaration**

```cpp
inline int16_t screen_pixel(Fixed value)
```

- **Declared at:** [line 92](../../../runtime/world2d_service.hpp#L92)
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

- **Declared at:** [line 270](../../../runtime/world2d_service.hpp#L270)
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

- **Declared at:** [line 221](../../../runtime/world2d_service.hpp#L221)
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

- **Declared at:** [line 278](../../../runtime/world2d_service.hpp#L278)
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

<a id="epok-world2d-service-dispatch-1"></a>

## `epok::world2d::Service::dispatch`

**Purpose.** Performs `dispatch` as part of the world2d service module.

**Exact declaration**

```cpp
template<class LevelType> static void dispatch(LevelType& level, ObjectId actor, ObjectId other, TriggerPhase phase)
```

- **Declared at:** [line 259](../../../runtime/world2d_service.hpp#L259)
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

<a id="epok-world2d-service-move-1"></a>

## `epok::world2d::Service::move`

**Purpose.** Performs `move` as part of the world2d service module.

**Exact declaration**

```cpp
bool move(ActorComponent& body, const Fixed* delta, uint32_t mask, MoveResult2D& result)
```

- **Declared at:** [line 205](../../../runtime/world2d_service.hpp#L205)
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

<a id="epok-world2d-service-overlap-1"></a>

## `epok::world2d::Service::overlap`

**Purpose.** Performs `overlap` as part of the world2d service module.

**Exact declaration**

```cpp
size_t overlap(const Aabb2D& box, ObjectId* actors, size_t capacity, uint32_t mask, bool triggers)
```

- **Declared at:** [line 228](../../../runtime/world2d_service.hpp#L228)
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

<a id="epok-world2d-service-raycast-1"></a>

## `epok::world2d::Service::raycast`

**Purpose.** Performs `raycast` as part of the world2d service module.

**Exact declaration**

```cpp
SpatialHit2D raycast(const Fixed* origin, const Fixed* delta, uint32_t mask, bool triggers)
```

- **Declared at:** [line 224](../../../runtime/world2d_service.hpp#L224)
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

<a id="epok-world2d-service-reset-1"></a>

## `epok::world2d::Service::reset`

**Purpose.** Resets reset as part of the world2d service module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 282](../../../runtime/world2d_service.hpp#L282)
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

<a id="epok-world2d-service-step-1"></a>

## `epok::world2d::Service::step`

**Purpose.** ---- simulation step --------------------------------------------------------------- After the level tick: animate, resolve this step's colliders and triggers, then move the cameras to where the actors ended up.

**Exact declaration**

```cpp
template<class LevelType> void step(Fixed dt, LevelType* level)
```

- **Declared at:** [line 239](../../../runtime/world2d_service.hpp#L239)
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

- **Declared at:** [line 184](../../../runtime/world2d_service.hpp#L184)
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

<a id="epok-world2d-sort-draw-items-1"></a>

## `epok::world2d::sort_draw_items`

**Purpose.** Performs `sort draw items` as part of the world2d service module.

**Exact declaration**

```cpp
inline void sort_draw_items(DrawItem* items, DrawItem* scratch, size_t count)
```

- **Declared at:** [line 44](../../../runtime/world2d_service.hpp#L44)
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
