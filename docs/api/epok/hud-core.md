# Epok API: Hud Core

> **Header:** `"hud_core.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud_core.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 34 public callables declared directly in this header.

## Declared types

`epok::hud_core::Affine2`, `epok::hud_core::Budget`, `epok::hud_core::Compiler`, `epok::hud_core::ImagePatchRectangle`, `epok::hud_core::ImagePatchRectangles`, `epok::hud_core::ImageRectangle`, `epok::hud_core::ImageRectangles`, `epok::hud_core::ImageReuseCache`, `epok::hud_core::ImageReuseCache::Entry`, `epok::hud_core::ImageReuseCache::Patch`, `epok::hud_core::NodeCache`, `epok::hud_core::NodeCache::Counts`, `epok::hud_core::Rect`, `epok::hud_core::TrigTable`

## Callable index

- [`epok::hud_core::apply`](#epok-hud-core-apply-1) — Performs `apply` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::build_image_rectangles`](#epok-hud-core-build-image-rectangles-1) — Shared emission/clipping for ordinary builds and uninitialized POD scratch.
- [`epok::hud_core::clamp`](#epok-hud-core-clamp-1) — Performs `clamp` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::clipped_rectangle`](#epok-hud-core-clipped-rectangle-1) — Performs `clipped rectangle` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::Compiler<Sink>`](#epok-hud-core-compiler-compiler-sink-1) — Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::draw`](#epok-hud-core-compiler-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::layout`](#epok-hud-core-compiler-layout-1) — Measure and arrange without drawing: `rects` holds every laid-out node and zeroes elsewhere, and `affines` the accumulated rotation about each pivot.
- [`epok::hud_core::compose`](#epok-hud-core-compose-1) — `b` applied first, then `a`: the parent's accumulated transform composed onto a child's.
- [`epok::hud_core::cosine`](#epok-hud-core-cosine-1) — Performs `cosine` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::identity`](#epok-hud-core-identity-1) — Performs `identity` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImagePatchRectangles::build`](#epok-hud-core-imagepatchrectangles-build-1) — Performs `build` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageRectangles::build`](#epok-hud-core-imagerectangles-build-1) — Performs `build` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::applied`](#epok-hud-core-imagereusecache-applied-1) — Commit only after every prepare check succeeded and this packet was written.
- [`epok::hud_core::ImageReuseCache::begin`](#epok-hud-core-imagereusecache-begin-1) — Begins begin as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::capture`](#epok-hud-core-imagereusecache-capture-1) — Performs `capture` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::end`](#epok-hud-core-imagereusecache-end-1) — Ends end as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::invalidate`](#epok-hud-core-imagereusecache-invalidate-1) — Performs `invalidate` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::prepare`](#epok-hud-core-imagereusecache-prepare-1) — Performs `prepare` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::ImageReuseCache::seal`](#epok-hud-core-imagereusecache-seal-1) — Performs `seal` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::begin`](#epok-hud-core-nodecache-begin-1) — Begins begin as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::Counts::operator==`](#epok-hud-core-nodecache-counts-operator-1) — Performs `operator ==` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::end`](#epok-hud-core-nodecache-end-1) — Ends end as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::invalidate`](#epok-hud-core-nodecache-invalidate-1) — Performs `invalidate` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::invalidate_node`](#epok-hud-core-nodecache-invalidate-node-1) — Performs `invalidate node` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::NodeCache::rebuild`](#epok-hud-core-nodecache-rebuild-1) — Performs `rebuild` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::pixel`](#epok-hud-core-pixel-1) — Performs `pixel` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::render_key_mix`](#epok-hud-core-render-key-mix-1) — Packet keys are shared with host validation.
- [`epok::hud_core::render_node_key`](#epok-hud-core-render-node-key-1) — Renders node key as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::resolve`](#epok-hud-core-resolve-1) — Performs `resolve` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::rotation_about`](#epok-hud-core-rotation-about-1) — T(pivot) * R(degrees) * T(-pivot).
- [`epok::hud_core::sine`](#epok-hud-core-sine-1) — Performs `sine` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::TrigTable::TrigTable`](#epok-hud-core-trigtable-trigtable-1) — Constructs `epok::hud_core::TrigTable` for native HUD layout, drawing and focus navigation.
- [`epok::hud_core::turn_units`](#epok-hud-core-turn-units-1) — Degrees to 1/2048 of a turn, rounded once per rotated node rather than per primitive.
- [`epok::hud_core::untextured_image`](#epok-hud-core-untextured-image-1) — Performs `untextured image` as part of native HUD layout, drawing and focus navigation.

<a id="epok-hud-core-apply-1"></a>

## `epok::hud_core::apply`

**Purpose.** Performs `apply` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void apply(const Affine2& m,Fixed x,Fixed y,Fixed& out_x,Fixed& out_y)
```

- **Declared at:** [line 132](../../../runtime/hud_core.hpp#L132)
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

<a id="epok-hud-core-build-image-rectangles-1"></a>

## `epok::hud_core::build_image_rectangles`

**Purpose.** Shared emission/clipping for ordinary builds and uninitialized POD scratch.

**Details.** Each callback writes every field of the accepted entry before increasing count. A border emits at most four callbacks, so no fifth entry is touched.

**Exact declaration**

```cpp
template<class Output> void build_image_rectangles(Output& output,Rect rect,const Image& image,int width,int height)
```

- **Declared at:** [line 46](../../../runtime/hud_core.hpp#L46)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `output` | `Output &` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `rect` | `Rect` | Input | Value supplied for `rect`. See the exact type and module contract. |
| `image` | `const Image &` | Input | Value supplied for `image`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Each callback writes every field of the accepted entry before increasing count. A border emits at most four callbacks, so no fifth entry is touched.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Output

// Assume these named values have been initialized with valid data:
// Output & output
// Rect rect
// const Image & image
// int width
// int height

epok::hud_core& object = /* obtain a valid instance */;

object.build_image_rectangles<Output>(output, rect, image, width, height);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-clamp-1"></a>

## `epok::hud_core::clamp`

**Purpose.** Performs `clamp` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int clamp(int v,int hi)
```

- **Declared at:** [line 20](../../../runtime/hud_core.hpp#L20)
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

<a id="epok-hud-core-clipped-rectangle-1"></a>

## `epok::hud_core::clipped_rectangle`

**Purpose.** Performs `clipped rectangle` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
template<class Rectangle> inline bool clipped_rectangle(Rect r,int width,int height,Rectangle& result)
```

- **Declared at:** [line 24](../../../runtime/hud_core.hpp#L24)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `r` | `Rect` | Input | Value supplied for `r`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |
| `result` | `Rectangle &` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Rectangle

// Assume these named values have been initialized with valid data:
// Rect r
// int width
// int height
// Rectangle & result

epok::hud_core& object = /* obtain a valid instance */;

auto result = object.clipped_rectangle<Rectangle>(r, width, height, result);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compiler-compiler-sink-1"></a>

## `epok::hud_core::Compiler::Compiler<Sink>`

**Purpose.** Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
Compiler(Sink& sink,int width,int height,Budget budget):sin
```

- **Declared at:** [line 693](../../../runtime/hud_core.hpp#L693)
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
void draw(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines,bool measure_intrinsics=false)
```

- **Declared at:** [line 707](../../../runtime/hud_core.hpp#L707)
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
| `measure_intrinsics` | `bool` | Input | Value supplied for `measure_intrinsics`. See the exact type and module contract. |

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
// bool measure_intrinsics

epok::hud_core::Compiler& object = /* obtain a valid instance */;

object.draw(entities, count, first, next, sizes, boxes, affines, measure_intrinsics);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compiler-layout-1"></a>

## `epok::hud_core::Compiler::layout`

**Purpose.** Measure and arrange without drawing: `rects` holds every laid-out node and zeroes elsewhere, and `affines` the accumulated rotation about each pivot.

**Exact declaration**

```cpp
void layout(ActorData* entities,size_t count,int* first,int* next,Fixed (*sizes)[2],Rect* boxes,Affine2* affines,bool measure_intrinsics=true)
```

- **Declared at:** [line 696](../../../runtime/hud_core.hpp#L696)
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
| `measure_intrinsics` | `bool` | Input | Value supplied for `measure_intrinsics`. See the exact type and module contract. |

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
// bool measure_intrinsics

epok::hud_core::Compiler& object = /* obtain a valid instance */;

object.layout(entities, count, first, next, sizes, boxes, affines, measure_intrinsics);
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

- **Declared at:** [line 88](../../../runtime/hud_core.hpp#L88)
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

- **Declared at:** [line 76](../../../runtime/hud_core.hpp#L76)
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

- **Declared at:** [line 86](../../../runtime/hud_core.hpp#L86)
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

<a id="epok-hud-core-imagepatchrectangles-build-1"></a>

## `epok::hud_core::ImagePatchRectangles::build`

**Purpose.** Performs `build` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void build(Rect rect,const Image& image,int width,int height)
```

- **Declared at:** [line 59](../../../runtime/hud_core.hpp#L59)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rect` | `Rect` | Input | Value supplied for `rect`. See the exact type and module contract. |
| `image` | `const Image &` | Input | Value supplied for `image`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Rect rect
// const Image & image
// int width
// int height

epok::hud_core::ImagePatchRectangles& object = /* obtain a valid instance */;

object.build(rect, image, width, height);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-imagerectangles-build-1"></a>

## `epok::hud_core::ImageRectangles::build`

**Purpose.** Performs `build` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void build(Rect rect,const Image& image,int width,int height)
```

- **Declared at:** [line 55](../../../runtime/hud_core.hpp#L55)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rect` | `Rect` | Input | Value supplied for `rect`. See the exact type and module contract. |
| `image` | `const Image &` | Input | Value supplied for `image`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Rect rect
// const Image & image
// int width
// int height

epok::hud_core::ImageRectangles& object = /* obtain a valid instance */;

object.build(rect, image, width, height);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-imagereusecache-applied-1"></a>

## `epok::hud_core::ImageReuseCache::applied`

**Purpose.** Commit only after every prepare check succeeded and this packet was written.

**Details.** Failed prepares remain read-only, including mixed-image failure.

**Exact declaration**

```cpp
void applied(unsigned parity,const Patch& patch,uint32_t full_key)
```

- **Declared at:** [line 255](../../../runtime/hud_core.hpp#L255)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `patch` | `const Patch &` | Input | Value supplied for `patch`. See the exact type and module contract. |
| `full_key` | `uint32_t` | Input | Value supplied for `full_key`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Failed prepares remain read-only, including mixed-image failure.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// const Patch & patch
// uint32_t full_key

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.applied(parity, patch, full_key);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-imagereusecache-begin-1"></a>

## `epok::hud_core::ImageReuseCache::begin`

**Purpose.** Begins begin as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void begin(unsigned parity)
```

- **Declared at:** [line 213](../../../runtime/hud_core.hpp#L213)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.begin(parity);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-imagereusecache-capture-1"></a>

## `epok::hud_core::ImageReuseCache::capture`

**Purpose.** Performs `capture` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void capture(unsigned parity,size_t owner,const ActorData* entities,size_t count,Rect rect,const Affine2& transform, uint32_t first,uintptr_t textures,uintptr_t fonts)
```

- **Declared at:** [line 214](../../../runtime/hud_core.hpp#L214)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `owner` | `size_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `entities` | `const ActorData *` | Input | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `rect` | `Rect` | Input | Value supplied for `rect`. See the exact type and module contract. |
| `transform` | `const Affine2 &` | Input | Value supplied for `transform`. See the exact type and module contract. |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `textures` | `uintptr_t` | Input | Value supplied for `textures`. See the exact type and module contract. |
| `fonts` | `uintptr_t` | Input | Value supplied for `fonts`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// size_t owner
// const ActorData * entities
// size_t count
// Rect rect
// const Affine2 & transform
// uint32_t first
// uintptr_t textures
// uintptr_t fonts

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.capture(parity, owner, entities, count, rect, transform, first, textures, fonts);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-imagereusecache-end-1"></a>

## `epok::hud_core::ImageReuseCache::end`

**Purpose.** Ends end as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void end(unsigned parity,size_t owner,uint32_t used)
```

- **Declared at:** [line 222](../../../runtime/hud_core.hpp#L222)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `owner` | `size_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `used` | `uint32_t` | Input | Value supplied for `used`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// size_t owner
// uint32_t used

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.end(parity, owner, used);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-imagereusecache-invalidate-1"></a>

## `epok::hud_core::ImageReuseCache::invalidate`

**Purpose.** Performs `invalidate` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void invalidate()
```

- **Declared at:** [line 212](../../../runtime/hud_core.hpp#L212)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.invalidate();
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-imagereusecache-prepare-1"></a>

## `epok::hud_core::ImageReuseCache::prepare`

**Purpose.** Performs `prepare` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
bool prepare(unsigned parity,const ActorData* entities,size_t count,const uint32_t* keys,uintptr_t textures,uintptr_t fonts, int width,int height,Patch (&patches)[Capacity],unsigned& patch_count)const
```

- **Declared at:** [line 235](../../../runtime/hud_core.hpp#L235)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `entities` | `const ActorData *` | Input | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `keys` | `const uint32_t *` | Input | Value supplied for `keys`. See the exact type and module contract. |
| `textures` | `uintptr_t` | Input | Value supplied for `textures`. See the exact type and module contract. |
| `fonts` | `uintptr_t` | Input | Value supplied for `fonts`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |
| `patches` | `Patch (&)[Capacity]` | Input/output; inspect the function contract | Value supplied for `patches`. See the exact type and module contract. |
| `patch_count` | `unsigned int &` | Input/output; inspect the function contract | Value supplied for `patch_count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// const ActorData * entities
// size_t count
// const uint32_t * keys
// uintptr_t textures
// uintptr_t fonts
// int width
// int height
// Patch (&)[Capacity] patches
// unsigned int & patch_count

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

auto result = object.prepare(parity, entities, count, keys, textures, fonts, width, height, patches, patch_count);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-imagereusecache-seal-1"></a>

## `epok::hud_core::ImageReuseCache::seal`

**Purpose.** Performs `seal` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void seal(unsigned parity,const ActorData* entities,size_t count,const uint32_t* keys,uintptr_t textures,uintptr_t fonts)
```

- **Declared at:** [line 225](../../../runtime/hud_core.hpp#L225)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `entities` | `const ActorData *` | Input | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `keys` | `const uint32_t *` | Input | Value supplied for `keys`. See the exact type and module contract. |
| `textures` | `uintptr_t` | Input | Value supplied for `textures`. See the exact type and module contract. |
| `fonts` | `uintptr_t` | Input | Value supplied for `fonts`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// const ActorData * entities
// size_t count
// const uint32_t * keys
// uintptr_t textures
// uintptr_t fonts

epok::hud_core::ImageReuseCache& object = /* obtain a valid instance */;

object.seal(parity, entities, count, keys, textures, fonts);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-nodecache-begin-1"></a>

## `epok::hud_core::NodeCache::begin`

**Purpose.** Begins begin as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
bool begin(size_t owner,uint32_t key,Counts& counts,HudStats& stats)
```

- **Declared at:** [line 121](../../../runtime/hud_core.hpp#L121)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `size_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `key` | `uint32_t` | Input | Value supplied for `key`. See the exact type and module contract. |
| `counts` | `Counts &` | Input/output; inspect the function contract | Value supplied for `counts`. See the exact type and module contract. |
| `stats` | `HudStats &` | Input/output; inspect the function contract | Value supplied for `stats`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// size_t owner
// uint32_t key
// Counts & counts
// HudStats & stats

epok::hud_core::NodeCache& object = /* obtain a valid instance */;

auto result = object.begin(owner, key, counts, stats);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-nodecache-counts-operator-1"></a>

## `epok::hud_core::NodeCache::Counts::operator==`

**Purpose.** Performs `operator ==` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
bool operator==(const Counts&)const=default
```

- **Declared at:** [line 101](../../../runtime/hud_core.hpp#L101)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Counts &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// const Counts & arg1

epok::hud_core::NodeCache::Counts& object = /* obtain a valid instance */;

auto result = object.operator==(arg1);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-nodecache-end-1"></a>

## `epok::hud_core::NodeCache::end`

**Purpose.** Ends end as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void end(size_t owner,const Counts& counts,const HudStats& stats)
```

- **Declared at:** [line 128](../../../runtime/hud_core.hpp#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `size_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `counts` | `const Counts &` | Input | Value supplied for `counts`. See the exact type and module contract. |
| `stats` | `const HudStats &` | Input | Value supplied for `stats`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// size_t owner
// const Counts & counts
// const HudStats & stats

epok::hud_core::NodeCache& object = /* obtain a valid instance */;

object.end(owner, counts, stats);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-nodecache-invalidate-1"></a>

## `epok::hud_core::NodeCache::invalidate`

**Purpose.** Performs `invalidate` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void invalidate()
```

- **Declared at:** [line 118](../../../runtime/hud_core.hpp#L118)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

epok::hud_core::NodeCache& object = /* obtain a valid instance */;

object.invalidate();
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-nodecache-invalidate-node-1"></a>

## `epok::hud_core::NodeCache::invalidate_node`

**Purpose.** Performs `invalidate node` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void invalidate_node(unsigned p,size_t owner)
```

- **Declared at:** [line 119](../../../runtime/hud_core.hpp#L119)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `owner` | `size_t` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// size_t owner

epok::hud_core::NodeCache& object = /* obtain a valid instance */;

object.invalidate_node(p, owner);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-nodecache-rebuild-1"></a>

## `epok::hud_core::NodeCache::rebuild`

**Purpose.** Performs `rebuild` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void rebuild(unsigned p)
```

- **Declared at:** [line 120](../../../runtime/hud_core.hpp#L120)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p

epok::hud_core::NodeCache& object = /* obtain a valid instance */;

object.rebuild(p);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-pixel-1"></a>

## `epok::hud_core::pixel`

**Purpose.** Performs `pixel` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int pixel(Fixed v)
```

- **Declared at:** [line 19](../../../runtime/hud_core.hpp#L19)
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

<a id="epok-hud-core-render-key-mix-1"></a>

## `epok::hud_core::render_key_mix`

**Purpose.** Packet keys are shared with host validation.

**Details.** Nodes without Canvas/Rect do not participate in HUD layout or emission.

**Exact declaration**

```cpp
inline uint32_t render_key_mix(uint32_t h,uint32_t value)
```

- **Declared at:** [line 145](../../../runtime/hud_core.hpp#L145)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `uint32_t` | Input | Value supplied for `h`. See the exact type and module contract. |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Nodes without Canvas/Rect do not participate in HUD layout or emission.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t h
// uint32_t value

auto result = epok::hud_core::render_key_mix(h, value);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-hud-core-render-node-key-1"></a>

## `epok::hud_core::render_node_key`

**Purpose.** Renders node key as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline uint32_t render_node_key(const ActorData& e,uintptr_t textures,uintptr_t fonts,bool ignore_image_appearance=false)
```

- **Declared at:** [line 146](../../../runtime/hud_core.hpp#L146)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `e` | `const ActorData &` | Input | Value supplied for `e`. See the exact type and module contract. |
| `textures` | `uintptr_t` | Input | Value supplied for `textures`. See the exact type and module contract. |
| `fonts` | `uintptr_t` | Input | Value supplied for `fonts`. See the exact type and module contract. |
| `ignore_image_appearance` | `bool` | Input | Value supplied for `ignore_image_appearance`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & e
// uintptr_t textures
// uintptr_t fonts
// bool ignore_image_appearance

auto result = epok::hud_core::render_node_key(e, textures, fonts, ignore_image_appearance);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-resolve-1"></a>

## `epok::hud_core::resolve`

**Purpose.** Performs `resolve` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline Rect resolve(Rect parent,const RectTransform& r)
```

- **Declared at:** [line 133](../../../runtime/hud_core.hpp#L133)
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

- **Declared at:** [line 92](../../../runtime/hud_core.hpp#L92)
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

- **Declared at:** [line 81](../../../runtime/hud_core.hpp#L81)
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

- **Declared at:** [line 67](../../../runtime/hud_core.hpp#L67)
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

- **Declared at:** [line 83](../../../runtime/hud_core.hpp#L83)
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

<a id="epok-hud-core-untextured-image-1"></a>

## `epok::hud_core::untextured_image`

**Purpose.** Performs `untextured image` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
template<class Fill> void untextured_image(Rect r,const Image& image,Fill fill)
```

- **Declared at:** [line 29](../../../runtime/hud_core.hpp#L29)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `r` | `Rect` | Input | Value supplied for `r`. See the exact type and module contract. |
| `image` | `const Image &` | Input | Value supplied for `image`. See the exact type and module contract. |
| `fill` | `Fill` | Input | Value supplied for `fill`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Fill

// Assume these named values have been initialized with valid data:
// Rect r
// const Image & image
// Fill fill

epok::hud_core& object = /* obtain a valid instance */;

object.untextured_image<Fill>(r, image, fill);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
