# Epok API: Hud Core

> **Header:** `"hud_core.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud_core.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 14 public callables declared directly in this header.

## Declared types

`epok::hud_core::Affine2`, `epok::hud_core::Budget`, `epok::hud_core::Compiler`, `epok::hud_core::Rect`, `epok::hud_core::TrigTable`

## Callable index

- [`epok::hud_core::apply`](#epok-hud-core-apply-1) — Performs `apply` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::clamp`](#epok-hud-core-clamp-1) — Performs `clamp` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::Compiler<Sink>`](#epok-hud-core-compiler-compiler-sink-1) — Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::draw`](#epok-hud-core-compiler-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::layout`](#epok-hud-core-compiler-layout-1) — Measure and arrange without drawing: `rects` holds every laid-out node and zeroes elsewhere, and `affines` the accumulated rotation about each pivot.
- [`epok::hud_core::compose`](#epok-hud-core-compose-1) — `b` applied first, then `a`: the parent's accumulated transform composed onto a child's.
- [`epok::hud_core::cosine`](#epok-hud-core-cosine-1) — Performs `cosine` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::identity`](#epok-hud-core-identity-1) — Performs `identity` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::pixel`](#epok-hud-core-pixel-1) — Performs `pixel` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::resolve`](#epok-hud-core-resolve-1) — Performs `resolve` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::rotation_about`](#epok-hud-core-rotation-about-1) — T(pivot) * R(degrees) * T(-pivot).
- [`epok::hud_core::sine`](#epok-hud-core-sine-1) — Performs `sine` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::TrigTable::TrigTable`](#epok-hud-core-trigtable-trigtable-1) — Constructs `epok::hud_core::TrigTable` for native HUD layout, drawing and focus navigation.
- [`epok::hud_core::turn_units`](#epok-hud-core-turn-units-1) — Degrees to 1/2048 of a turn, rounded once per rotated node rather than per primitive.

<a id="epok-hud-core-apply-1"></a>

## `epok::hud_core::apply`

**Purpose.** Performs `apply` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void apply(const Affine2& m,Fixed x,Fixed y,Fixed& out_x,Fixed& out_y)
```

- **Declared at:** [line 47](../../../runtime/hud_core.hpp#L47)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine2 &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `out_x` | `Fixed &` | Input/output; inspect the function contract | Value supplied for `out_x`. See the exact type and module contract. |
| `out_y` | `Fixed &` | Input/output; inspect the function contract | Value supplied for `out_y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// const Affine2 & m
// Fixed x
// Fixed y
// Fixed & out_x
// Fixed & out_y

epok::hud_core::apply(m, x, y, out_x, out_y);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-clamp-1"></a>

## `epok::hud_core::clamp`

**Purpose.** Performs `clamp` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int clamp(int v,int hi)
```

- **Declared at:** [line 11](../../../runtime/hud_core.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `int` | Input | Value supplied for `v`. See the exact type and module contract. |
| `hi` | `int` | Input | Value supplied for `hi`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// int v
// int hi

auto result = epok::hud_core::clamp(v, hi);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-compiler-compiler-sink-1"></a>

## `epok::hud_core::Compiler::Compiler<Sink>`

**Purpose.** Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
Compiler(Sink& sink,int width,int height,Budget budget):sin
```

- **Declared at:** [line 402](../../../runtime/hud_core.hpp#L402)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sink` | `Sink &` | Input/output; inspect the function contract | Value supplied for `sink`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |
| `budget` | `Budget` | Input | Value supplied for `budget`. See the exact type and module contract. |

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Sink & sink
// int width
// int height
// Budget budget

epok::hud_core::Compiler value(sink, width, height, budget);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compiler-draw-1"></a>

## `epok::hud_core::Compiler::draw`

**Purpose.** Draws draw as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void draw(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines)
```

- **Declared at:** [line 413](../../../runtime/hud_core.hpp#L413)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entities` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `first` | `int *` | Input/output; inspect the function contract | Value supplied for `first`. See the exact type and module contract. |
| `next` | `int *` | Input/output; inspect the function contract | Value supplied for `next`. See the exact type and module contract. |
| `sizes` | `Fixed (*)[2]` | Input/output; inspect the function contract | Value supplied for `sizes`. See the exact type and module contract. |
| `boxes` | `Rect *` | Input/output; inspect the function contract | Value supplied for `boxes`. See the exact type and module contract. |
| `affines` | `Affine2 *` | Input/output; inspect the function contract | Value supplied for `affines`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * entities
// size_t count
// int * first
// int * next
// Fixed (*)[2] sizes
// Rect * boxes
// Affine2 * affines

epok::hud_core::Compiler& object = /* obtain a valid instance */;

object.draw(entities, count, first, next, sizes, boxes, affines);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compiler-layout-1"></a>

## `epok::hud_core::Compiler::layout`

**Purpose.** Measure and arrange without drawing: `rects` holds every laid-out node and zeroes elsewhere, and `affines` the accumulated rotation about each pivot.

**Exact declaration**

```cpp
void layout(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines)
```

- **Declared at:** [line 405](../../../runtime/hud_core.hpp#L405)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entities` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `first` | `int *` | Input/output; inspect the function contract | Value supplied for `first`. See the exact type and module contract. |
| `next` | `int *` | Input/output; inspect the function contract | Value supplied for `next`. See the exact type and module contract. |
| `sizes` | `Fixed (*)[2]` | Input/output; inspect the function contract | Value supplied for `sizes`. See the exact type and module contract. |
| `boxes` | `Rect *` | Input/output; inspect the function contract | Value supplied for `boxes`. See the exact type and module contract. |
| `affines` | `Affine2 *` | Input/output; inspect the function contract | Value supplied for `affines`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * entities
// size_t count
// int * first
// int * next
// Fixed (*)[2] sizes
// Rect * boxes
// Affine2 * affines

epok::hud_core::Compiler& object = /* obtain a valid instance */;

object.layout(entities, count, first, next, sizes, boxes, affines);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compose-1"></a>

## `epok::hud_core::compose`

**Purpose.** `b` applied first, then `a`: the parent's accumulated transform composed onto a child's.

**Exact declaration**

```cpp
inline Affine2 compose(const Affine2& a,const Affine2& b)
```

- **Declared at:** [line 39](../../../runtime/hud_core.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Affine2 &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Affine2 &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Affine2`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// const Affine2 & a
// const Affine2 & b

auto result = epok::hud_core::compose(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-cosine-1"></a>

## `epok::hud_core::cosine`

**Purpose.** Performs `cosine` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline Fixed cosine(int units)
```

- **Declared at:** [line 27](../../../runtime/hud_core.hpp#L27)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `units` | `int` | Input | Value supplied for `units`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// int units

auto result = epok::hud_core::cosine(units);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-identity-1"></a>

## `epok::hud_core::identity`

**Purpose.** Performs `identity` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline bool identity(const Affine2& m)
```

- **Declared at:** [line 37](../../../runtime/hud_core.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine2 &` | Input | Value supplied for `m`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// const Affine2 & m

auto result = epok::hud_core::identity(m);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-pixel-1"></a>

## `epok::hud_core::pixel`

**Purpose.** Performs `pixel` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int pixel(Fixed v)
```

- **Declared at:** [line 10](../../../runtime/hud_core.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Fixed` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Fixed v

auto result = epok::hud_core::pixel(v);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-resolve-1"></a>

## `epok::hud_core::resolve`

**Purpose.** Performs `resolve` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline Rect resolve(Rect parent,const RectTransform& r)
```

- **Declared at:** [line 48](../../../runtime/hud_core.hpp#L48)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `Rect` | Input | Value supplied for `parent`. See the exact type and module contract. |
| `r` | `const RectTransform &` | Input | Value supplied for `r`. See the exact type and module contract. |

**Returns.** Returns `Rect`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Rect parent
// const RectTransform & r

auto result = epok::hud_core::resolve(parent, r);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-rotation-about-1"></a>

## `epok::hud_core::rotation_about`

**Purpose.** T(pivot) * R(degrees) * T(-pivot).

**Exact declaration**

```cpp
inline Affine2 rotation_about(Fixed x,Fixed y,Fixed degrees)
```

- **Declared at:** [line 43](../../../runtime/hud_core.hpp#L43)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Affine2`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed degrees

auto result = epok::hud_core::rotation_about(x, y, degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-sine-1"></a>

## `epok::hud_core::sine`

**Purpose.** Performs `sine` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline Fixed sine(int units)
```

- **Declared at:** [line 32](../../../runtime/hud_core.hpp#L32)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `units` | `int` | Input | Value supplied for `units`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// int units

auto result = epok::hud_core::sine(units);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-trigtable-trigtable-1"></a>

## `epok::hud_core::TrigTable::TrigTable`

**Purpose.** Constructs `epok::hud_core::TrigTable` for native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
constexpr TrigTable()
```

- **Declared at:** [line 18](../../../runtime/hud_core.hpp#L18)
- **Kind:** `constructor`

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

epok::hud_core::TrigTable value();
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-turn-units-1"></a>

## `epok::hud_core::turn_units`

**Purpose.** Degrees to 1/2048 of a turn, rounded once per rotated node rather than per primitive.

**Exact declaration**

```cpp
inline int turn_units(Fixed degrees)
```

- **Declared at:** [line 34](../../../runtime/hud_core.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::hud_core::turn_units(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
