# Epok API: Polygon

> **Header:** `"polygon.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/polygon.hpp)

This module covers PSX polygon preparation, clipping and submission. It documents 11 public callables declared directly in this header.

## Declared types

`epok::CameraQ12`, `epok::CameraQ8`, `epok::CameraUnits`, `epok::ClipVertex`, `epok::ProjectedVertex`, `epok::QuadPacket`

## Callable index

- [`epok::blend_fog`](#epok-blend-fog-1) — Performs `blend fog` as part of PSX polygon preparation, clipping and submission.
- [`epok::CameraUnits::bucket`](#epok-cameraunits-bucket-1) — Ordering table bucket from three depths: (za+zb+zc) / (3 << DepthShift), exact for the frustum range without an integer division.
- [`epok::clip_polygon`](#epok-clip-polygon-1) — Sutherland-Hodgman against the frustum planes named by `planes` (bit i = plane i of frustum_outcode).
- [`epok::floor_div`](#epok-floor-div-1) — Floor division for a positive divisor, matching the GTE's arithmetic shift.
- [`epok::fog_amount`](#epok-fog-amount-1) — Linear fog: 0 before start, 4096 at or beyond end.
- [`epok::modulate_channel`](#epok-modulate-channel-1) — (c * 128 + 127) / 255 for c in 0..255, exact without a division.
- [`epok::modulate_color`](#epok-modulate-color-1) — Performs `modulate color` as part of PSX polygon preparation, clipping and submission.
- [`epok::pack_color`](#epok-pack-color-1) — Performs `pack color` as part of PSX polygon preparation, clipping and submission.
- [`epok::project_cpu`](#epok-project-cpu-1) — CPU perspective for clipped or fallback vertices, rounding like the GTE so mixed triangles share edges.
- [`epok::scale_channel`](#epok-scale-channel-1) — a * b / 255 for 0..255 inputs, exact.
- [`epok::screen_area`](#epok-screen-area-1) — Screen area sign in the GPU's coordinate system; zero for degenerate triangles.

<a id="epok-blend-fog-1"></a>

## `epok::blend_fog`

**Purpose.** Performs `blend fog` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t blend_fog(uint32_t packed, uint32_t amount, const uint8_t* fog)
```

- **Declared at:** [line 57](../../../runtime/polygon.hpp#L57)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |
| `amount` | `uint32_t` | Input | Value supplied for `amount`. See the exact type and module contract. |
| `fog` | `const uint8_t *` | Input | Value supplied for `fog`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t packed
// uint32_t amount
// const uint8_t * fog

auto result = epok::blend_fog(packed, amount, fog);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-cameraunits-bucket-1"></a>

## `epok::CameraUnits::bucket`

**Purpose.** Ordering table bucket from three depths: (za+zb+zc) / (3 << DepthShift), exact for the frustum range without an integer division.

**Exact declaration**

```cpp
static int bucket(int32_t za, int32_t zb, int32_t zc)
```

- **Declared at:** [line 33](../../../runtime/polygon.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `za` | `int32_t` | Input | Value supplied for `za`. See the exact type and module contract. |
| `zb` | `int32_t` | Input | Value supplied for `zb`. See the exact type and module contract. |
| `zc` | `int32_t` | Input | Value supplied for `zc`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// int32_t za
// int32_t zb
// int32_t zc

auto result = epok::CameraUnits::bucket(za, zb, zc);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-clip-polygon-1"></a>

## `epok::clip_polygon`

**Purpose.** Sutherland-Hodgman against the frustum planes named by `planes` (bit i = plane i of frustum_outcode).

**Details.** Returns the vertex count in buffers[from].

**Exact declaration**

```cpp
template<class Units> inline int clip_polygon(ClipVertex (&buffers)[2][12], int count, uint8_t planes, int& from)
```

- **Declared at:** [line 96](../../../runtime/polygon.hpp#L96)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buffers` | `ClipVertex (&)[2][12]` | Input/output; inspect the function contract | Value supplied for `buffers`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `planes` | `uint8_t` | Input | Value supplied for `planes`. See the exact type and module contract. |
| `from` | `int &` | Input/output; inspect the function contract | Value supplied for `from`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Returns the vertex count in buffers[from].

**Usage pattern**

```cpp
#include "polygon.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Units

// Assume these named values have been initialized with valid data:
// ClipVertex (&)[2][12] buffers
// int count
// uint8_t planes
// int & from

auto result = epok::clip_polygon<Units>(buffers, count, planes, from);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-floor-div-1"></a>

## `epok::floor_div`

**Purpose.** Floor division for a positive divisor, matching the GTE's arithmetic shift.

**Exact declaration**

```cpp
inline int32_t floor_div(int64_t numerator, int32_t divisor)
```

- **Declared at:** [line 71](../../../runtime/polygon.hpp#L71)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `numerator` | `int64_t` | Input | Value supplied for `numerator`. See the exact type and module contract. |
| `divisor` | `int32_t` | Input | Value supplied for `divisor`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// int64_t numerator
// int32_t divisor

auto result = epok::floor_div(numerator, divisor);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fog-amount-1"></a>

## `epok::fog_amount`

**Purpose.** Linear fog: 0 before start, 4096 at or beyond end.

**Details.** Depth and range share one unit; the range must be positive.

**Exact declaration**

```cpp
inline uint16_t fog_amount(int32_t depth, int32_t start, int32_t end)
```

- **Declared at:** [line 52](../../../runtime/polygon.hpp#L52)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `depth` | `int32_t` | Input | Value supplied for `depth`. See the exact type and module contract. |
| `start` | `int32_t` | Input | Value supplied for `start`. See the exact type and module contract. |
| `end` | `int32_t` | Input | Value supplied for `end`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** Depth and range share one unit; the range must be positive.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// int32_t depth
// int32_t start
// int32_t end

auto result = epok::fog_amount(depth, start, end);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-modulate-channel-1"></a>

## `epok::modulate_channel`

**Purpose.** (c * 128 + 127) / 255 for c in 0..255, exact without a division.

**Exact declaration**

```cpp
inline uint32_t modulate_channel(uint32_t c)
```

- **Declared at:** [line 44](../../../runtime/polygon.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `uint32_t` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t c

auto result = epok::modulate_channel(c);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-modulate-color-1"></a>

## `epok::modulate_color`

**Purpose.** Performs `modulate color` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t modulate_color(uint32_t packed)
```

- **Declared at:** [line 45](../../../runtime/polygon.hpp#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t packed

auto result = epok::modulate_color(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pack-color-1"></a>

## `epok::pack_color`

**Purpose.** Performs `pack color` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t pack_color(uint32_t r, uint32_t g, uint32_t b)
```

- **Declared at:** [line 42](../../../runtime/polygon.hpp#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `r` | `uint32_t` | Input | Value supplied for `r`. See the exact type and module contract. |
| `g` | `uint32_t` | Input | Value supplied for `g`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t r
// uint32_t g
// uint32_t b

auto result = epok::pack_color(r, g, b);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-project-cpu-1"></a>

## `epok::project_cpu`

**Purpose.** CPU perspective for clipped or fallback vertices, rounding like the GTE so mixed triangles share edges.

**Details.** Inputs inside the frustum after clipping never overflow; the guard band keeps far outliers software-clipped.

**Exact declaration**

```cpp
template<class Units, int Width, int Height> inline void project_cpu(ProjectedVertex& point)
```

- **Declared at:** [line 77](../../../runtime/polygon.hpp#L77)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `ProjectedVertex &` | Input/output; inspect the function contract | Value supplied for `point`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Inputs inside the frustum after clipping never overflow; the guard band keeps far outliers software-clipped.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Units, Width, Height

// Assume these named values have been initialized with valid data:
// ProjectedVertex & point

epok::project_cpu<Units, Width, Height>(point);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scale-channel-1"></a>

## `epok::scale_channel`

**Purpose.** a * b / 255 for 0..255 inputs, exact.

**Exact declaration**

```cpp
inline uint32_t scale_channel(uint32_t a, uint32_t b)
```

- **Declared at:** [line 49](../../../runtime/polygon.hpp#L49)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::scale_channel(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screen-area-1"></a>

## `epok::screen_area`

**Purpose.** Screen area sign in the GPU's coordinate system; zero for degenerate triangles.

**Exact declaration**

```cpp
inline int32_t screen_area(const ProjectedVertex& a, const ProjectedVertex& b, const ProjectedVertex& c)
```

- **Declared at:** [line 66](../../../runtime/polygon.hpp#L66)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const ProjectedVertex &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const ProjectedVertex &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const ProjectedVertex &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// const ProjectedVertex & a
// const ProjectedVertex & b
// const ProjectedVertex & c

auto result = epok::screen_area(a, b, c);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
