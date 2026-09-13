# Epok API: Collision

> **Header:** `"collision.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/collision.hpp)

This module covers bounded AABB collision queries and movement. It documents 16 public callables declared directly in this header.

## Declared types

`epok::AabbT`, `epok::ColliderT`, `epok::CollisionWorld`, `epok::MoveResultT`, `epok::SpatialHitT`, `epok::TriggerEvent`, `epok::TriggerPhase`

## Callable index

- [`epok::aabb_may_touch`](#epok-aabb-may-touch-1) — Performs `aabb may touch` as part of bounded AABB collision queries and movement.
- [`epok::aabb_overlap`](#epok-aabb-overlap-1) — Performs `aabb overlap` as part of bounded AABB collision queries and movement.
- [`epok::collider_bounds`](#epok-collider-bounds-1) — Performs `collider bounds` as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::begin_sync`](#epok-collisionworld-begin-sync-1) — Begins sync as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::bounds`](#epok-collisionworld-bounds-1) — Performs `bounds` as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::clear`](#epok-collisionworld-clear-1) — Clears clear as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::ground`](#epok-collisionworld-ground-1) — Downward box sweep uses the complete footprint, including ledges missed by a center ray.
- [`epok::CollisionWorld::move_and_slide`](#epok-collisionworld-move-and-slide-1) — Performs `move and slide` as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::note_enabled`](#epok-collisionworld-note-enabled-1) — Performs `note enabled` as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::overlap`](#epok-collisionworld-overlap-1) — Returns total matches; only the first output_capacity indices are written.
- [`epok::CollisionWorld::raycast`](#epok-collisionworld-raycast-1) — Segment query: delta is the complete displacement, not a unit direction.
- [`epok::CollisionWorld::set`](#epok-collisionworld-set-1) — Sets set as part of bounded AABB collision queries and movement.
- [`epok::CollisionWorld::set_cached`](#epok-collisionworld-set-cached-1) — Returns whether bounds were rebuilt.
- [`epok::CollisionWorld::update_triggers`](#epok-collisionworld-update-triggers-1) — Updates triggers as part of bounded AABB collision queries and movement.
- [`epok::SpatialHitT::operator bool`](#epok-spatialhitt-operator-bool-1) — Performs `operator  bool` as part of bounded AABB collision queries and movement.
- [`epok::swept_bounds`](#epok-swept-bounds-1) — Performs `swept bounds` as part of bounded AABB collision queries and movement.

<a id="epok-aabb-may-touch-1"></a>

## `epok::aabb_may_touch`

**Purpose.** Performs `aabb may touch` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
template<class Number> inline bool aabb_may_touch(const AabbT<Number>& a,const AabbT<Number>& b)
```

- **Declared at:** [line 29](../../../runtime/collision.hpp#L29)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const AabbT<Number> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const AabbT<Number> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const AabbT<Number> & a
// const AabbT<Number> & b

auto result = epok::aabb_may_touch<Number>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-aabb-overlap-1"></a>

## `epok::aabb_overlap`

**Purpose.** Performs `aabb overlap` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
template<class Number> inline bool aabb_overlap(const AabbT<Number>& a,const AabbT<Number>& b)
```

- **Declared at:** [line 25](../../../runtime/collision.hpp#L25)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const AabbT<Number> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const AabbT<Number> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const AabbT<Number> & a
// const AabbT<Number> & b

auto result = epok::aabb_overlap<Number>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collider-bounds-1"></a>

## `epok::collider_bounds`

**Purpose.** Performs `collider bounds` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
template<class Number> inline AabbT<Number> collider_bounds(const ColliderT<Number>& c,const Affine<Number>& world)
```

- **Declared at:** [line 41](../../../runtime/collision.hpp#L41)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `const ColliderT<Number> &` | Input | Value supplied for `c`. See the exact type and module contract. |
| `world` | `const Affine<Number> &` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `AabbT<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const ColliderT<Number> & c
// const Affine<Number> & world

auto result = epok::collider_bounds<Number>(c, world);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-begin-sync-1"></a>

## `epok::CollisionWorld::begin_sync`

**Purpose.** Begins sync as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
void begin_sync()
```

- **Declared at:** [line 95](../../../runtime/collision.hpp#L95)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

epok::CollisionWorld& object = /* obtain a valid instance */;

object.begin_sync();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded AABB collision queries and movement. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld-bounds-1"></a>

## `epok::CollisionWorld::bounds`

**Purpose.** Performs `bounds` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
const AabbT<Number>* bounds(size_t index) const
```

- **Declared at:** [line 118](../../../runtime/collision.hpp#L118)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const AabbT<Number> *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.bounds(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld-clear-1"></a>

## `epok::CollisionWorld::clear`

**Purpose.** Clears clear as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 94](../../../runtime/collision.hpp#L94)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

epok::CollisionWorld& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded AABB collision queries and movement. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld-ground-1"></a>

## `epok::CollisionWorld::ground`

**Purpose.** Downward box sweep uses the complete footprint, including ledges missed by a center ray.

**Details.** distance must be nonnegative; normal points upwards.

**Exact declaration**

```cpp
SpatialHitT<Number> ground(const AabbT<Number>& box,Number distance,uint32_t mask=0xffffffffu,int ignore=-1) const
```

- **Declared at:** [line 143](../../../runtime/collision.hpp#L143)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const AabbT<Number> &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `distance` | `Number` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |

**Returns.** Returns `SpatialHitT<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** distance must be nonnegative; normal points upwards.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// const AabbT<Number> & box
// Number distance
// uint32_t mask
// int ignore

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.ground(box, distance, mask, ignore);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-move-and-slide-1"></a>

## `epok::CollisionWorld::move_and_slide`

**Purpose.** Performs `move and slide` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
MoveResultT<Number> move_and_slide(AabbT<Number> box,const Number* displacement,uint32_t mask=0xffffffffu,int ignore=-1) const
```

- **Declared at:** [line 158](../../../runtime/collision.hpp#L158)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `AabbT<Number>` | Input | Value supplied for `box`. See the exact type and module contract. |
| `displacement` | `const Number *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |

**Returns.** Returns `MoveResultT<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// AabbT<Number> box
// const Number * displacement
// uint32_t mask
// int ignore

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.move_and_slide(box, displacement, mask, ignore);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-note-enabled-1"></a>

## `epok::CollisionWorld::note_enabled`

**Purpose.** Performs `note enabled` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
void note_enabled(size_t index)
```

- **Declared at:** [line 96](../../../runtime/collision.hpp#L96)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::CollisionWorld& object = /* obtain a valid instance */;

object.note_enabled(index);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded AABB collision queries and movement. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionworld-overlap-1"></a>

## `epok::CollisionWorld::overlap`

**Purpose.** Returns total matches; only the first output_capacity indices are written.

**Exact declaration**

```cpp
size_t overlap(const AabbT<Number>& box,uint16_t* output,size_t output_capacity,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=true) const
```

- **Declared at:** [line 120](../../../runtime/collision.hpp#L120)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const AabbT<Number> &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `output` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `output_capacity` | `size_t` | Input | Value supplied for `output_capacity`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// const AabbT<Number> & box
// uint16_t * output
// size_t output_capacity
// uint32_t mask
// int ignore
// bool triggers

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.overlap(box, output, output_capacity, mask, ignore, triggers);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-raycast-1"></a>

## `epok::CollisionWorld::raycast`

**Purpose.** Segment query: delta is the complete displacement, not a unit direction.

**Exact declaration**

```cpp
SpatialHitT<Number> raycast(const Number* origin,const Number* delta,uint32_t mask=0xffffffffu,int ignore=-1,bool triggers=false) const
```

- **Declared at:** [line 128](../../../runtime/collision.hpp#L128)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `const Number *` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `delta` | `const Number *` | Input | Value supplied for `delta`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `int` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `SpatialHitT<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// const Number * origin
// const Number * delta
// uint32_t mask
// int ignore
// bool triggers

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.raycast(origin, delta, mask, ignore, triggers);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-set-1"></a>

## `epok::CollisionWorld::set`

**Purpose.** Sets set as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
void set(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,bool active=true,uint32_t generation=0)
```

- **Declared at:** [line 97](../../../runtime/collision.hpp#L97)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `collider` | `const ColliderT<Number> &` | Input | Value supplied for `collider`. See the exact type and module contract. |
| `matrix` | `const Affine<Number> &` | Input | Value supplied for `matrix`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |
| `generation` | `uint32_t` | Input | Value supplied for `generation`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const ColliderT<Number> & collider
// const Affine<Number> & matrix
// bool active
// uint32_t generation

epok::CollisionWorld& object = /* obtain a valid instance */;

object.set(index, collider, matrix, active, generation);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded AABB collision queries and movement. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-set-cached-1"></a>

## `epok::CollisionWorld::set_cached`

**Purpose.** Returns whether bounds were rebuilt.

**Details.** Metadata remains live even when a static box is reused; direct collider edits and slot generations are safe.

**Exact declaration**

```cpp
bool set_cached(size_t index,const ColliderT<Number>& collider,const Affine<Number>& matrix,uint32_t revision,bool active=true,uint32_t generation=0)
```

- **Declared at:** [line 104](../../../runtime/collision.hpp#L104)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `collider` | `const ColliderT<Number> &` | Input | Value supplied for `collider`. See the exact type and module contract. |
| `matrix` | `const Affine<Number> &` | Input | Value supplied for `matrix`. See the exact type and module contract. |
| `revision` | `uint32_t` | Input | Value supplied for `revision`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |
| `generation` | `uint32_t` | Input | Value supplied for `generation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Metadata remains live even when a static box is reused; direct collider edits and slot generations are safe.

**Usage pattern**

```cpp
#include "collision.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const ColliderT<Number> & collider
// const Affine<Number> & matrix
// uint32_t revision
// bool active
// uint32_t generation

epok::CollisionWorld& object = /* obtain a valid instance */;

auto result = object.set_cached(index, collider, matrix, revision, active, generation);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collisionworld-update-triggers-1"></a>

## `epok::CollisionWorld::update_triggers`

**Purpose.** Updates triggers as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
template<class Callback> void update_triggers(Callback&& callback)
```

- **Declared at:** [line 202](../../../runtime/collision.hpp#L202)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `Callback &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Callback

// Assume these named values have been initialized with valid data:
// Callback && callback

epok::CollisionWorld& object = /* obtain a valid instance */;

object.update_triggers<Callback>(callback);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spatialhitt-operator-bool-1"></a>

## `epok::SpatialHitT::operator bool`

**Purpose.** Performs `operator  bool` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
explicit operator bool() const
```

- **Declared at:** [line 16](../../../runtime/collision.hpp#L16)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

epok::SpatialHitT& object = /* obtain a valid instance */;

auto result = object.operator bool();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-swept-bounds-1"></a>

## `epok::swept_bounds`

**Purpose.** Performs `swept bounds` as part of bounded AABB collision queries and movement.

**Exact declaration**

```cpp
template<class Number> inline AabbT<Number> swept_bounds(AabbT<Number> box,const Number* delta)
```

- **Declared at:** [line 33](../../../runtime/collision.hpp#L33)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `AabbT<Number>` | Input | Value supplied for `box`. See the exact type and module contract. |
| `delta` | `const Number *` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** Returns `AabbT<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded AABB collision queries and movement and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "collision.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// AabbT<Number> box
// const Number * delta

auto result = epok::swept_bounds<Number>(box, delta);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
