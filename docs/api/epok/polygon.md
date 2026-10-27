# Epok API: Polygon

> **Header:** `"polygon.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/polygon.hpp)

This module covers PSX polygon preparation, clipping and submission. It documents 21 public callables declared directly in this header.

## Declared types

`epok::CameraQ12`, `epok::CameraQ8`, `epok::CameraUnits`, `epok::CameraUnits::Frustum`, `epok::ClipVertex`, `epok::CompactProjectedVertex`, `epok::CompactProjectedVertex::Depth`, `epok::ProjectedVertex`, `epok::QuadPacket`

## Callable index

- [`epok::blend_fog`](#epok-blend-fog-1) — Performs `blend fog` as part of PSX polygon preparation, clipping and submission.
- [`epok::blend_vertex_fog`](#epok-blend-vertex-fog-1) — Performs `blend vertex fog` as part of PSX polygon preparation, clipping and submission.
- [`epok::CameraUnits::bucket`](#epok-cameraunits-bucket-1) — Ordering table bucket from three depths: (za+zb+zc) / (3 << DepthShift), exact for the frustum range without an integer division.
- [`epok::CameraUnits::outcode`](#epok-cameraunits-outcode-1) — Performs `outcode` as part of PSX polygon preparation, clipping and submission.
- [`epok::clip_fraction16`](#epok-clip-fraction16-1) — Performs `clip fraction16` as part of PSX polygon preparation, clipping and submission.
- [`epok::clip_polygon`](#epok-clip-polygon-1) — Performs `clip polygon` as part of PSX polygon preparation, clipping and submission.
- [`epok::clip_polygon_distances`](#epok-clip-polygon-distances-1) — Sutherland-Hodgman against the frustum planes named by `planes` (bit i = plane i of frustum_outcode).
- [`epok::compact_gpu_safe`](#epok-compact-gpu-safe-1) — Screen/depth-only projection is also sufficient for off-screen chunks when every vertex is depth-valid and the complete packet span fits the GPU.
- [`epok::compact_mesh_gpu_safe`](#epok-compact-mesh-gpu-safe-1) — Unrelated faces in one spatial chunk can span more than the GPU limit even though every submitted triangle fits.
- [`epok::CompactProjectedVertex::Depth::operator[]`](#epok-compactprojectedvertex-depth-operator-1) — Performs `operator []` as part of PSX polygon preparation, clipping and submission.
- [`epok::floor_div`](#epok-floor-div-1) — Floor division for a positive divisor, matching the GTE's arithmetic shift.
- [`epok::fog_amount`](#epok-fog-amount-1) — Linear fog: 0 before start, 4096 at or beyond end.
- [`epok::modulate_channel`](#epok-modulate-channel-1) — (c * 128 + 127) / 255 for c in 0..255, exact without a division.
- [`epok::modulate_color`](#epok-modulate-color-1) — Performs `modulate color` as part of PSX polygon preparation, clipping and submission.
- [`epok::pack_color`](#epok-pack-color-1) — Performs `pack color` as part of PSX polygon preparation, clipping and submission.
- [`epok::packet_color`](#epok-packet-color-1) — Performs `packet color` as part of PSX polygon preparation, clipping and submission.
- [`epok::project_cpu`](#epok-project-cpu-1) — CPU perspective for clipped or fallback vertices, rounding like the GTE so mixed triangles share edges.
- [`epok::scale_channel`](#epok-scale-channel-1) — a * b / 255 for 0..255 inputs, exact.
- [`epok::screen_area`](#epok-screen-area-1) — Performs `screen area` as part of PSX polygon preparation, clipping and submission.
- [`epok::screen_backface`](#epok-screen-backface-1) — Screen area sign in the GPU's coordinate system; zero for degenerate triangles.
- [`epok::tint_color`](#epok-tint-color-1) — Performs `tint color` as part of PSX polygon preparation, clipping and submission.

<a id="epok-blend-fog-1"></a>

## `epok::blend_fog`

**Purpose.** Performs `blend fog` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t blend_fog(uint32_t packed, uint32_t amount, const uint8_t* fog)
```

- **Declared at:** [line 120](../../../runtime/polygon.hpp#L120)
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

<a id="epok-blend-vertex-fog-1"></a>

## `epok::blend_vertex_fog`

**Purpose.** Performs `blend vertex fog` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t blend_vertex_fog(uint32_t packed, uint32_t amount, const uint8_t* fog, bool textured_gpu_colors)
```

- **Declared at:** [line 128](../../../runtime/polygon.hpp#L128)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |
| `amount` | `uint32_t` | Input | Value supplied for `amount`. See the exact type and module contract. |
| `fog` | `const uint8_t *` | Input | Value supplied for `fog`. See the exact type and module contract. |
| `textured_gpu_colors` | `bool` | Input | Value supplied for `textured_gpu_colors`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t packed
// uint32_t amount
// const uint8_t * fog
// bool textured_gpu_colors

auto result = epok::blend_vertex_fog(packed, amount, fog, textured_gpu_colors);
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

- **Declared at:** [line 88](../../../runtime/polygon.hpp#L88)
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

<a id="epok-cameraunits-outcode-1"></a>

## `epok::CameraUnits::outcode`

**Purpose.** Performs `outcode` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
static uint8_t outcode(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 85](../../../runtime/polygon.hpp#L85)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::CameraUnits::outcode(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-clip-fraction16-1"></a>

## `epok::clip_fraction16`

**Purpose.** Performs `clip fraction16` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t clip_fraction16(uint32_t numerator,uint32_t denominator)
```

- **Declared at:** [line 177](../../../runtime/polygon.hpp#L177)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `numerator` | `uint32_t` | Input | Value supplied for `numerator`. See the exact type and module contract. |
| `denominator` | `uint32_t` | Input | Value supplied for `denominator`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t numerator
// uint32_t denominator

auto result = epok::clip_fraction16(numerator, denominator);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-clip-polygon-1"></a>

## `epok::clip_polygon`

**Purpose.** Performs `clip polygon` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
template<class Units> inline int clip_polygon(ClipVertex (&buffers)[2][12], int count, uint8_t planes, int& from)
```

- **Declared at:** [line 236](../../../runtime/polygon.hpp#L236)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buffers` | `ClipVertex (&)[2][12]` | Input/output; inspect the function contract | Value supplied for `buffers`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `planes` | `uint8_t` | Input | Value supplied for `planes`. See the exact type and module contract. |
| `from` | `int &` | Input/output; inspect the function contract | Value supplied for `from`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

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

<a id="epok-clip-polygon-distances-1"></a>

## `epok::clip_polygon_distances`

**Purpose.** Sutherland-Hodgman against the frustum planes named by `planes` (bit i = plane i of frustum_outcode).

**Details.** Returns the vertex count in buffers[from].

**Exact declaration**

```cpp
template<class Units,class Distance=int64_t> __attribute__((noinline,optimize("O3"))) int clip_polygon_distances(ClipVertex (&buffers)[2][12], int count, uint8_t planes, int& from)
```

- **Declared at:** [line 191](../../../runtime/polygon.hpp#L191)
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
// Units, Distance

// Assume these named values have been initialized with valid data:
// ClipVertex (&)[2][12] buffers
// int count
// uint8_t planes
// int & from

auto result = epok::clip_polygon_distances<Units, Distance>(buffers, count, planes, from);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-compact-gpu-safe-1"></a>

## `epok::compact_gpu_safe`

**Purpose.** Screen/depth-only projection is also sufficient for off-screen chunks when every vertex is depth-valid and the complete packet span fits the GPU.

**Details.** The drawing-area scissor clips pixels; near/far and oversized primitives still use the full camera-space clipper.

**Exact declaration**

```cpp
inline bool compact_gpu_safe(const CompactProjectedVertex* vertices,size_t count,int32_t near,int32_t far)
```

- **Declared at:** [line 33](../../../runtime/polygon.hpp#L33)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertices` | `const CompactProjectedVertex *` | Input | Value supplied for `vertices`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `near` | `int32_t` | Input | Value supplied for `near`. See the exact type and module contract. |
| `far` | `int32_t` | Input | Value supplied for `far`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The drawing-area scissor clips pixels; near/far and oversized primitives still use the full camera-space clipper.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// const CompactProjectedVertex * vertices
// size_t count
// int32_t near
// int32_t far

auto result = epok::compact_gpu_safe(vertices, count, near, far);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-compact-mesh-gpu-safe-1"></a>

## `epok::compact_mesh_gpu_safe`

**Purpose.** Unrelated faces in one spatial chunk can span more than the GPU limit even though every submitted triangle fits.

**Details.** Check those primitives before falling back to full camera coordinates and software clipping for the entire chunk.

**Exact declaration**

```cpp
template<class Quad> inline bool compact_mesh_gpu_safe(const CompactProjectedVertex* vertices,size_t count, const Quad* quads,size_t faces,int32_t near,int32_t far)
```

- **Declared at:** [line 48](../../../runtime/polygon.hpp#L48)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertices` | `const CompactProjectedVertex *` | Input | Value supplied for `vertices`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `quads` | `const Quad *` | Input | Value supplied for `quads`. See the exact type and module contract. |
| `faces` | `size_t` | Input | Value supplied for `faces`. See the exact type and module contract. |
| `near` | `int32_t` | Input | Value supplied for `near`. See the exact type and module contract. |
| `far` | `int32_t` | Input | Value supplied for `far`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Check those primitives before falling back to full camera coordinates and software clipping for the entire chunk.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Quad

// Assume these named values have been initialized with valid data:
// const CompactProjectedVertex * vertices
// size_t count
// const Quad * quads
// size_t faces
// int32_t near
// int32_t far

auto result = epok::compact_mesh_gpu_safe<Quad>(vertices, count, quads, faces, near, far);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-compactprojectedvertex-depth-operator-1"></a>

## `epok::CompactProjectedVertex::Depth::operator[]`

**Purpose.** Performs `operator []` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
int32_t operator[](size_t axis) const
```

- **Declared at:** [line 25](../../../runtime/polygon.hpp#L25)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `size_t` | Input | Value supplied for `axis`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// size_t axis

epok::CompactProjectedVertex::Depth& object = /* obtain a valid instance */;

auto result = object.operator[](axis);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-floor-div-1"></a>

## `epok::floor_div`

**Purpose.** Floor division for a positive divisor, matching the GTE's arithmetic shift.

**Exact declaration**

```cpp
inline int32_t floor_div(int64_t numerator, int32_t divisor)
```

- **Declared at:** [line 142](../../../runtime/polygon.hpp#L142)
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

- **Declared at:** [line 115](../../../runtime/polygon.hpp#L115)
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

- **Declared at:** [line 99](../../../runtime/polygon.hpp#L99)
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

- **Declared at:** [line 100](../../../runtime/polygon.hpp#L100)
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

- **Declared at:** [line 97](../../../runtime/polygon.hpp#L97)
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

<a id="epok-packet-color-1"></a>

## `epok::packet_color`

**Purpose.** Performs `packet color` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t packet_color(uint32_t color, bool textured, bool gpu_colors)
```

- **Declared at:** [line 110](../../../runtime/polygon.hpp#L110)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `color` | `uint32_t` | Input | Value supplied for `color`. See the exact type and module contract. |
| `textured` | `bool` | Input | Value supplied for `textured`. See the exact type and module contract. |
| `gpu_colors` | `bool` | Input | Value supplied for `gpu_colors`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t color
// bool textured
// bool gpu_colors

auto result = epok::packet_color(color, textured, gpu_colors);
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

- **Declared at:** [line 154](../../../runtime/polygon.hpp#L154)
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

- **Declared at:** [line 104](../../../runtime/polygon.hpp#L104)
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

**Purpose.** Performs `screen area` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
template<class Vertex> inline int32_t screen_area(const Vertex& a, const Vertex& b, const Vertex& c)
```

- **Declared at:** [line 137](../../../runtime/polygon.hpp#L137)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vertex &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vertex &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Vertex &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Vertex

// Assume these named values have been initialized with valid data:
// const Vertex & a
// const Vertex & b
// const Vertex & c

auto result = epok::screen_area<Vertex>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-screen-backface-1"></a>

## `epok::screen_backface`

**Purpose.** Screen area sign in the GPU's coordinate system; zero for degenerate triangles.

**Exact declaration**

```cpp
inline bool screen_backface(int32_t area, bool mirror_x)
```

- **Declared at:** [line 134](../../../runtime/polygon.hpp#L134)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `area` | `int32_t` | Input | Value supplied for `area`. See the exact type and module contract. |
| `mirror_x` | `bool` | Input | Value supplied for `mirror_x`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// int32_t area
// bool mirror_x

auto result = epok::screen_backface(area, mirror_x);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tint-color-1"></a>

## `epok::tint_color`

**Purpose.** Performs `tint color` as part of PSX polygon preparation, clipping and submission.

**Exact declaration**

```cpp
inline uint32_t tint_color(uint32_t color, uint32_t shade)
```

- **Declared at:** [line 105](../../../runtime/polygon.hpp#L105)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `color` | `uint32_t` | Input | Value supplied for `color`. See the exact type and module contract. |
| `shade` | `uint32_t` | Input | Value supplied for `shade`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX polygon preparation, clipping and submission and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "polygon.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t color
// uint32_t shade

auto result = epok::tint_color(color, shade);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX polygon preparation, clipping and submission. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
