# Epok API: Frustum

> **Header:** `"frustum.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/frustum.hpp)

This module covers the frustum module. It documents 13 public callables declared directly in this header.

## Declared types

`epok::ScreenFrustum`

## Callable index

- [`epok::chunk_bounds_input_limit`](#epok-chunk-bounds-input-limit-1) — Every individual product must fit int32; the renderer checks this once per chunk using the largest coefficient magnitude.
- [`epok::chunk_bounds_isolated_x`](#epok-chunk-bounds-isolated-x-1) — Detect once per object.
- [`epok::chunk_bounds_planes_fit`](#epok-chunk-bounds-planes-fit-1) — With the narrow products, each post-shift center/span sum is <=3*524288.
- [`epok::chunk_fully_inside`](#epok-chunk-fully-inside-1) — Q12 bounds.
- [`epok::chunk_outside`](#epok-chunk-outside-1) — Performs `chunk outside` as part of the frustum module.
- [`epok::frustum_gcd`](#epok-frustum-gcd-1) — Performs `frustum gcd` as part of the frustum module.
- [`epok::frustum_outcode`](#epok-frustum-outcode-1) — Performs `frustum outcode` as part of the frustum module.
- [`epok::frustum_outcode32`](#epok-frustum-outcode32-1) — Plain 32-bit variant for inputs bounded by 2^27 in magnitude (the GTE projection path guarantees this); 3z+4y then cannot overflow.
- [`epok::frustum_outcode_units`](#epok-frustum-outcode-units-1) — Same six half-spaces as the polygon clipper, before perspective division.
- [`epok::gpu_clip_safe`](#epok-gpu-clip-safe-1) — The GPU clips pixels to its drawing area, but rejects primitives with spans >1023 horizontally or >511 vertically.
- [`epok::narrow_chunk_bounds`](#epok-narrow-chunk-bounds-1) — Performs `narrow chunk bounds` as part of the frustum module.
- [`epok::ScreenFrustum::distance`](#epok-screenfrustum-distance-1) — Performs `distance` as part of the frustum module.
- [`epok::ScreenFrustum::outcode`](#epok-screenfrustum-outcode-1) — Performs `outcode` as part of the frustum module.

<a id="epok-chunk-bounds-input-limit-1"></a>

## `epok::chunk_bounds_input_limit`

**Purpose.** Every individual product must fit int32; the renderer checks this once per chunk using the largest coefficient magnitude.

**Details.** Post-shift sums then fit too.

**Exact declaration**

```cpp
inline int32_t chunk_bounds_input_limit(int32_t peak)
```

- **Declared at:** [line 45](../../../runtime/frustum.hpp#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `peak` | `int32_t` | Input | Value supplied for `peak`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Post-shift sums then fit too.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Assume these named values have been initialized with valid data:
// int32_t peak

auto result = epok::chunk_bounds_input_limit(peak);
```

**Why choose it.** It provides direct, allocation-conscious access to the frustum module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunk-bounds-isolated-x-1"></a>

## `epok::chunk_bounds_isolated_x`

**Purpose.** Detect once per object.

**Details.** Pitch-only cameras and axis-aligned object transforms leave local X independent of the other two axes (including reflection/scale).

**Exact declaration**

```cpp
inline bool chunk_bounds_isolated_x(const int32_t rows[3][3])
```

- **Declared at:** [line 29](../../../runtime/frustum.hpp#L29)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rows` | `const int32_t[3][3]` | Input | Value supplied for `rows`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Pitch-only cameras and axis-aligned object transforms leave local X independent of the other two axes (including reflection/scale).

**Usage pattern**

```cpp
#include "frustum.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t[3][3] rows

auto result = epok::chunk_bounds_isolated_x(rows);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-chunk-bounds-planes-fit-1"></a>

## `epok::chunk_bounds_planes_fit`

**Purpose.** With the narrow products, each post-shift center/span sum is <=3*524288.

**Details.** This translation bound leaves ample room for their signed plane products.

**Exact declaration**

```cpp
template<class Frustum=ScreenFrustum<320,240>> inline bool chunk_bounds_planes_fit(const int32_t* translation)
```

- **Declared at:** [line 50](../../../runtime/frustum.hpp#L50)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `translation` | `const int32_t *` | Input | Value supplied for `translation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** This translation bound leaves ample room for their signed plane products.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Frustum

// Assume these named values have been initialized with valid data:
// const int32_t * translation

auto result = epok::chunk_bounds_planes_fit<Frustum>(translation);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-chunk-fully-inside-1"></a>

## `epok::chunk_fully_inside`

**Purpose.** Q12 bounds.

**Details.** A margin also covers Q12->Q8 GTE input and translation rounding. The near guard avoids the GTE H/2 divide-overflow interval at 320x240.

**Exact declaration**

```cpp
template<class Frustum=ScreenFrustum<320,240>> inline bool chunk_fully_inside(const int32_t* center,const int32_t* extent,int32_t focal)
```

- **Declared at:** [line 34](../../../runtime/frustum.hpp#L34)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `center` | `const int32_t *` | Input | Value supplied for `center`. See the exact type and module contract. |
| `extent` | `const int32_t *` | Input | Value supplied for `extent`. See the exact type and module contract. |
| `focal` | `int32_t` | Input | Value supplied for `focal`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A margin also covers Q12->Q8 GTE input and translation rounding. The near guard avoids the GTE H/2 divide-overflow interval at 320x240.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Frustum

// Assume these named values have been initialized with valid data:
// const int32_t * center
// const int32_t * extent
// int32_t focal

auto result = epok::chunk_fully_inside<Frustum>(center, extent, focal);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-chunk-outside-1"></a>

## `epok::chunk_outside`

**Purpose.** Performs `chunk outside` as part of the frustum module.

**Exact declaration**

```cpp
template<class Frustum=ScreenFrustum<320,240>,class Number=int64_t> inline bool chunk_outside(const int32_t* center,const int32_t* extent,int32_t far)
```

- **Declared at:** [line 56](../../../runtime/frustum.hpp#L56)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `center` | `const int32_t *` | Input | Value supplied for `center`. See the exact type and module contract. |
| `extent` | `const int32_t *` | Input | Value supplied for `extent`. See the exact type and module contract. |
| `far` | `int32_t` | Input | Value supplied for `far`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Frustum

// Assume these named values have been initialized with valid data:
// const int32_t * center
// const int32_t * extent
// int32_t far

auto result = epok::chunk_outside<Frustum>(center, extent, far);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-frustum-gcd-1"></a>

## `epok::frustum_gcd`

**Purpose.** Performs `frustum gcd` as part of the frustum module.

**Exact declaration**

```cpp
constexpr int frustum_gcd(int a,int b)
```

- **Declared at:** [line 5](../../../runtime/frustum.hpp#L5)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Assume these named values have been initialized with valid data:
// int a
// int b

auto result = epok::frustum_gcd(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the frustum module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-frustum-outcode-1"></a>

## `epok::frustum_outcode`

**Purpose.** Performs `frustum outcode` as part of the frustum module.

**Exact declaration**

```cpp
inline uint8_t frustum_outcode(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 106](../../../runtime/frustum.hpp#L106)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::frustum_outcode(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the frustum module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-frustum-outcode32-1"></a>

## `epok::frustum_outcode32`

**Purpose.** Plain 32-bit variant for inputs bounded by 2^27 in magnitude (the GTE projection path guarantees this); 3z+4y then cannot overflow.

**Exact declaration**

```cpp
template<int32_t Near,int32_t FarExclusive> inline uint8_t frustum_outcode32(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 109](../../../runtime/frustum.hpp#L109)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Near, FarExclusive

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::frustum_outcode32<Near, FarExclusive>(x, y, z);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-frustum-outcode-units-1"></a>

## `epok::frustum_outcode_units`

**Purpose.** Same six half-spaces as the polygon clipper, before perspective division.

**Details.** Wide intermediates also make classification safe for far off-screen points.

**Exact declaration**

```cpp
template<int32_t Near,int32_t FarExclusive> inline uint8_t frustum_outcode_units(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 96](../../../runtime/frustum.hpp#L96)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** Wide intermediates also make classification safe for far off-screen points.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Near, FarExclusive

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::frustum_outcode_units<Near, FarExclusive>(x, y, z);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-gpu-clip-safe-1"></a>

## `epok::gpu_clip_safe`

**Purpose.** The GPU clips pixels to its drawing area, but rejects primitives with spans >1023 horizontally or >511 vertically.

**Details.** Keep software clipping for those and for vertices crossing near/far or the signed screen coordinate guard band.

**Exact declaration**

```cpp
template<class Point> bool gpu_clip_safe(const Point& a,const Point& b,const Point& c)
```

- **Declared at:** [line 122](../../../runtime/frustum.hpp#L122)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Point &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Point &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Point &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Keep software clipping for those and for vertices crossing near/far or the signed screen coordinate guard band.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Point

// Assume these named values have been initialized with valid data:
// const Point & a
// const Point & b
// const Point & c

auto result = epok::gpu_clip_safe<Point>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-narrow-chunk-bounds-1"></a>

## `epok::narrow_chunk_bounds`

**Purpose.** Performs `narrow chunk bounds` as part of the frustum module.

**Exact declaration**

```cpp
__attribute__((always_inline)) #endif inline void narrow_chunk_bounds(const int32_t rows[3][3],const int32_t absolute[3][3], const int32_t translation[3],const int32_t vector[3], const int32_t extent[3],int32_t center[3],int32_t span[3], bool isolated_x)
```

- **Declared at:** [line 72](../../../runtime/frustum.hpp#L72)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rows` | `const int32_t[3][3]` | Input | Value supplied for `rows`. See the exact type and module contract. |
| `absolute` | `const int32_t[3][3]` | Input | Value supplied for `absolute`. See the exact type and module contract. |
| `translation` | `const int32_t[3]` | Input | Value supplied for `translation`. See the exact type and module contract. |
| `vector` | `const int32_t[3]` | Input | Value supplied for `vector`. See the exact type and module contract. |
| `extent` | `const int32_t[3]` | Input | Value supplied for `extent`. See the exact type and module contract. |
| `center` | `int32_t[3]` | Input | Value supplied for `center`. See the exact type and module contract. |
| `span` | `int32_t[3]` | Input | Value supplied for `span`. See the exact type and module contract. |
| `isolated_x` | `bool` | Input | Value supplied for `isolated_x`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t[3][3] rows
// const int32_t[3][3] absolute
// const int32_t[3] translation
// const int32_t[3] vector
// const int32_t[3] extent
// int32_t[3] center
// int32_t[3] span
// bool isolated_x

epok::narrow_chunk_bounds(rows, absolute, translation, vector, extent, center, span, isolated_x);
```

**Why choose it.** It provides direct, allocation-conscious access to the frustum module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenfrustum-distance-1"></a>

## `epok::ScreenFrustum::distance`

**Purpose.** Performs `distance` as part of the frustum module.

**Exact declaration**

```cpp
template<class Number> static Number distance(int plane,Number x,Number y,Number z)
```

- **Declared at:** [line 12](../../../runtime/frustum.hpp#L12)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `plane` | `int` | Input | Value supplied for `plane`. See the exact type and module contract. |
| `x` | `Number` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Number` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Number` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `Number`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// int plane
// Number x
// Number y
// Number z

auto result = epok::ScreenFrustum::distance<Number>(plane, x, y, z);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-screenfrustum-outcode-1"></a>

## `epok::ScreenFrustum::outcode`

**Purpose.** Performs `outcode` as part of the frustum module.

**Exact declaration**

```cpp
template<int32_t Near,int32_t Far,bool Bounded=false> static uint8_t outcode(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 16](../../../runtime/frustum.hpp#L16)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the frustum module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frustum.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Near, Far, Bounded

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::ScreenFrustum::outcode<Near, Far, Bounded>(x, y, z);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.
