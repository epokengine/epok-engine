# Epok API: Gte Geometry

> **Header:** `"gte_geometry.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gte_geometry.hpp)

This module covers Geometry Transformation Engine math and register operations. It documents 26 public callables declared directly in this header.

## Declared types

`epok::CompactProjectionOffset`, `epok::GteProjectionInput`

## Callable index

- [`epok::begin_average_depth`](#epok-begin-average-depth-1) — gte_average_depth in two halves, so CPU work can run during AVSZ.
- [`epok::begin_loaded_projection_triple`](#epok-begin-loaded-projection-triple-1) — Begins loaded projection triple as part of Geometry Transformation Engine math and register operations.
- [`epok::begin_packed_vertex`](#epok-begin-packed-vertex-1) — project_packed_vertex in two halves, so independent CPU work can run while the GTE computes RTPS: begin loads the vertex and issues RTPS; finish (which waits for it) stores the same outputs and returns the same flag verdict.
- [`epok::begin_projection_triple`](#epok-begin-projection-triple-1) — Begins projection triple as part of Geometry Transformation Engine math and register operations.
- [`epok::checked_compact_projection_input`](#epok-checked-compact-projection-input-1) — Per-vertex form for a block whose bounds do not all fit: `sum` is the block center plus the lattice offset.
- [`epok::compact_projection_input`](#epok-compact-projection-input-1) — Performs `compact projection input` as part of Geometry Transformation Engine math and register operations.
- [`epok::finish_packed_vertex`](#epok-finish-packed-vertex-1) — Performs `finish packed vertex` as part of Geometry Transformation Engine math and register operations.
- [`epok::finish_projection_triple`](#epok-finish-projection-triple-1) — Performs `finish projection triple` as part of Geometry Transformation Engine math and register operations.
- [`epok::gte_average_depth`](#epok-gte-average-depth-1) — Average of the SZ FIFO after RTPT (SZ1..3) or RTPT+RTPS (SZ0..3).
- [`epok::gte_vertex_bounds_fit`](#epok-gte-vertex-bounds-fit-1) — Performs `gte vertex bounds fit` as part of Geometry Transformation Engine math and register operations.
- [`epok::load_average_depth_scale`](#epok-load-average-depth-scale-1) — AVSZ3/AVSZ4 scale factors; see ordered_gte_depth_average for the contract.
- [`epok::load_compact_projection_input`](#epok-load-compact-projection-input-1) — Loads compact projection input as part of Geometry Transformation Engine math and register operations.
- [`epok::load_geometry_matrix`](#epok-load-geometry-matrix-1) — Use MAC1..3, not the saturated 16-bit IR registers: the existing renderer's Q12 camera coordinates regularly exceed +/-8 world units.
- [`epok::load_projection_input`](#epok-load-projection-input-1) — Feed each prepared corner straight into its vector registers.
- [`epok::load_projection_matrix`](#epok-load-projection-matrix-1) — Loads projection matrix as part of Geometry Transformation Engine math and register operations.
- [`epok::load_projection_screen`](#epok-load-projection-screen-1) — H is the pixel focal length shared by both axes; offsets are the screen centre.
- [`epok::load_projection_translation`](#epok-load-projection-translation-1) — Caller validates the signed Q8 translation and retains the loaded rotation.
- [`epok::load_projection_words`](#epok-load-projection-words-1) — Loads projection words as part of Geometry Transformation Engine math and register operations.
- [`epok::pack_projection_input`](#epok-pack-projection-input-1) — Performs `pack projection input` as part of Geometry Transformation Engine math and register operations.
- [`epok::project_geometry_triple`](#epok-project-geometry-triple-1) — Performs `project geometry triple` as part of Geometry Transformation Engine math and register operations.
- [`epok::project_geometry_triple`](#epok-project-geometry-triple-2) — Performs `project geometry triple` as part of Geometry Transformation Engine math and register operations.
- [`epok::project_geometry_vertex`](#epok-project-geometry-vertex-1) — Camera output is Q8 with Y up.
- [`epok::project_packed_vertex`](#epok-project-packed-vertex-1) — Performs `project packed vertex` as part of Geometry Transformation Engine math and register operations.
- [`epok::read_average_depth`](#epok-read-average-depth-1) — Reads average depth as part of Geometry Transformation Engine math and register operations.
- [`epok::store_projection_triple`](#epok-store-projection-triple-1) — Same outputs as finish_projection_triple, stored even when the GTE flagged the projection; returns whether it was unflagged.
- [`epok::transform_geometry_vertex`](#epok-transform-geometry-vertex-1) — Performs `transform geometry vertex` as part of Geometry Transformation Engine math and register operations.

<a id="epok-begin-average-depth-1"></a>

## `epok::begin_average_depth`

**Purpose.** gte_average_depth in two halves, so CPU work can run during AVSZ.

**Details.** No other GTE command may come between them.

**Exact declaration**

```cpp
inline void begin_average_depth(bool quad)
```

- **Declared at:** [line 203](../../../runtime/gte_geometry.hpp#L203)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** No other GTE command may come between them.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// bool quad

epok::begin_average_depth(quad);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-begin-loaded-projection-triple-1"></a>

## `epok::begin_loaded_projection_triple`

**Purpose.** Begins loaded projection triple as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline void begin_loaded_projection_triple()
```

- **Declared at:** [line 211](../../../runtime/gte_geometry.hpp#L211)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

epok::begin_loaded_projection_triple();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-begin-packed-vertex-1"></a>

## `epok::begin_packed_vertex`

**Purpose.** project_packed_vertex in two halves, so independent CPU work can run while the GTE computes RTPS: begin loads the vertex and issues RTPS; finish (which waits for it) stores the same outputs and returns the same flag verdict.

**Details.** No other GTE access may come between them.

**Exact declaration**

```cpp
inline void begin_packed_vertex(const GteProjectionInput& vertex)
```

- **Declared at:** [line 301](../../../runtime/gte_geometry.hpp#L301)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `const GteProjectionInput &` | Input | Value supplied for `vertex`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** No other GTE access may come between them.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const GteProjectionInput & vertex

epok::begin_packed_vertex(vertex);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-begin-projection-triple-1"></a>

## `epok::begin_projection_triple`

**Purpose.** Begins projection triple as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline void begin_projection_triple(const GteProjectionInput* vertices)
```

- **Declared at:** [line 220](../../../runtime/gte_geometry.hpp#L220)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertices` | `const GteProjectionInput *` | Input | Value supplied for `vertices`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const GteProjectionInput * vertices

epok::begin_projection_triple(vertices);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-checked-compact-projection-input-1"></a>

## `epok::checked_compact_projection_input`

**Purpose.** Per-vertex form for a block whose bounds do not all fit: `sum` is the block center plus the lattice offset.

**Details.** Same packed words as pack_projection_input.

**Exact declaration**

```cpp
inline bool checked_compact_projection_input(const uint8_t* entry,const int32_t* sum,GteProjectionInput& out)
```

- **Declared at:** [line 156](../../../runtime/gte_geometry.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const uint8_t *` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `sum` | `const int32_t *` | Input | Value supplied for `sum`. See the exact type and module contract. |
| `out` | `GteProjectionInput &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Same packed words as pack_projection_input.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * entry
// const int32_t * sum
// GteProjectionInput & out

auto result = epok::checked_compact_projection_input(entry, sum, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-compact-projection-input-1"></a>

## `epok::compact_projection_input`

**Purpose.** Performs `compact projection input` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline GteProjectionInput compact_projection_input(const uint8_t* entry,const CompactProjectionOffset& offset)
```

- **Declared at:** [line 144](../../../runtime/gte_geometry.hpp#L144)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const uint8_t *` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `offset` | `const CompactProjectionOffset &` | Input | Value supplied for `offset`. See the exact type and module contract. |

**Returns.** Returns `GteProjectionInput`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * entry
// const CompactProjectionOffset & offset

auto result = epok::compact_projection_input(entry, offset);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-finish-packed-vertex-1"></a>

## `epok::finish_packed_vertex`

**Purpose.** Performs `finish packed vertex` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool finish_packed_vertex(uint32_t& screen,int32_t& depth)
```

- **Declared at:** [line 313](../../../runtime/gte_geometry.hpp#L313)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t & screen
// int32_t & depth

auto result = epok::finish_packed_vertex(screen, depth);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-finish-projection-triple-1"></a>

## `epok::finish_projection_triple`

**Purpose.** Performs `finish projection triple` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool finish_projection_triple(uint32_t* screen,int32_t* depth)
```

- **Declared at:** [line 241](../../../runtime/gte_geometry.hpp#L241)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * screen
// int32_t * depth

auto result = epok::finish_projection_triple(screen, depth);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gte-average-depth-1"></a>

## `epok::gte_average_depth`

**Purpose.** Average of the SZ FIFO after RTPT (SZ1..3) or RTPT+RTPS (SZ0..3).

**Exact declaration**

```cpp
inline int32_t gte_average_depth(bool quad)
```

- **Declared at:** [line 196](../../../runtime/gte_geometry.hpp#L196)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// bool quad

auto result = epok::gte_average_depth(quad);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gte-vertex-bounds-fit-1"></a>

## `epok::gte_vertex_bounds_fit`

**Purpose.** Performs `gte vertex bounds fit` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool gte_vertex_bounds_fit(const int32_t* origin8,const int32_t* center12,const int32_t* extent12)
```

- **Declared at:** [line 48](../../../runtime/gte_geometry.hpp#L48)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin8` | `const int32_t *` | Input | Value supplied for `origin8`. See the exact type and module contract. |
| `center12` | `const int32_t *` | Input | Value supplied for `center12`. See the exact type and module contract. |
| `extent12` | `const int32_t *` | Input | Value supplied for `extent12`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * origin8
// const int32_t * center12
// const int32_t * extent12

auto result = epok::gte_vertex_bounds_fit(origin8, center12, extent12);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-average-depth-scale-1"></a>

## `epok::load_average_depth_scale`

**Purpose.** AVSZ3/AVSZ4 scale factors; see ordered_gte_depth_average for the contract.

**Exact declaration**

```cpp
inline void load_average_depth_scale(int32_t zsf3,int32_t zsf4)
```

- **Declared at:** [line 190](../../../runtime/gte_geometry.hpp#L190)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `zsf3` | `int32_t` | Input | Value supplied for `zsf3`. See the exact type and module contract. |
| `zsf4` | `int32_t` | Input | Value supplied for `zsf4`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// int32_t zsf3
// int32_t zsf4

epok::load_average_depth_scale(zsf3, zsf4);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-load-compact-projection-input-1"></a>

## `epok::load_compact_projection_input`

**Purpose.** Loads compact projection input as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template<unsigned Slot> inline void load_compact_projection_input(const uint8_t* entry,const CompactProjectionOffset& offset)
```

- **Declared at:** [line 176](../../../runtime/gte_geometry.hpp#L176)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `const uint8_t *` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `offset` | `const CompactProjectionOffset &` | Input | Value supplied for `offset`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Slot

// Assume these named values have been initialized with valid data:
// const uint8_t * entry
// const CompactProjectionOffset & offset

epok::load_compact_projection_input<Slot>(entry, offset);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-geometry-matrix-1"></a>

## `epok::load_geometry_matrix`

**Purpose.** Use MAC1..3, not the saturated 16-bit IR registers: the existing renderer's Q12 camera coordinates regularly exceed +/-8 world units.

**Details.** Perspective and near/far clipping retain the software path and its full coordinate range.

**Exact declaration**

```cpp
template<class Number> bool load_geometry_matrix(const Affine<Number>& m)
```

- **Declared at:** [line 11](../../../runtime/gte_geometry.hpp#L11)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Number> &` | Input | Value supplied for `m`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Perspective and near/far clipping retain the software path and its full coordinate range.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const Affine<Number> & m

auto result = epok::load_geometry_matrix<Number>(m);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-projection-input-1"></a>

## `epok::load_projection_input`

**Purpose.** Feed each prepared corner straight into its vector registers.

**Details.** Loading an out-of-range value is harmless: the caller must not issue RTPT unless ALL corners fit, and must reload all three after changing the projection anchor. Unlike begin_projection_triple, this path needs no temporary RAM array.

**Exact declaration**

```cpp
template<unsigned Slot> inline bool load_projection_input(const int32_t* world,const int32_t* offset)
```

- **Declared at:** [line 120](../../../runtime/gte_geometry.hpp#L120)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const int32_t *` | Input | Value supplied for `world`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Loading an out-of-range value is harmless: the caller must not issue RTPT unless ALL corners fit, and must reload all three after changing the projection anchor. Unlike begin_projection_triple, this path needs no temporary RAM array.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Slot

// Assume these named values have been initialized with valid data:
// const int32_t * world
// const int32_t * offset

auto result = epok::load_projection_input<Slot>(world, offset);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-projection-matrix-1"></a>

## `epok::load_projection_matrix`

**Purpose.** Loads projection matrix as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template<class Number> bool load_projection_matrix(const Affine<Number>& m)
```

- **Declared at:** [line 56](../../../runtime/gte_geometry.hpp#L56)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Number> &` | Input | Value supplied for `m`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const Affine<Number> & m

auto result = epok::load_projection_matrix<Number>(m);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-projection-screen-1"></a>

## `epok::load_projection_screen`

**Purpose.** H is the pixel focal length shared by both axes; offsets are the screen centre.

**Exact declaration**

```cpp
inline void load_projection_screen(int32_t focal,int32_t centre_x,int32_t centre_y)
```

- **Declared at:** [line 74](../../../runtime/gte_geometry.hpp#L74)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `focal` | `int32_t` | Input | Value supplied for `focal`. See the exact type and module contract. |
| `centre_x` | `int32_t` | Input | Value supplied for `centre_x`. See the exact type and module contract. |
| `centre_y` | `int32_t` | Input | Value supplied for `centre_y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// int32_t focal
// int32_t centre_x
// int32_t centre_y

epok::load_projection_screen(focal, centre_x, centre_y);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-load-projection-translation-1"></a>

## `epok::load_projection_translation`

**Purpose.** Caller validates the signed Q8 translation and retains the loaded rotation.

**Exact declaration**

```cpp
inline void load_projection_translation(const int32_t* translation)
```

- **Declared at:** [line 104](../../../runtime/gte_geometry.hpp#L104)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `translation` | `const int32_t *` | Input | Value supplied for `translation`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * translation

epok::load_projection_translation(translation);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-load-projection-words-1"></a>

## `epok::load_projection_words`

**Purpose.** Loads projection words as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template<unsigned Slot> inline void load_projection_words(const GteProjectionInput& input)
```

- **Declared at:** [line 163](../../../runtime/gte_geometry.hpp#L163)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `input` | `const GteProjectionInput &` | Input | Value supplied for `input`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Slot

// Assume these named values have been initialized with valid data:
// const GteProjectionInput & input

epok::load_projection_words<Slot>(input);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pack-projection-input-1"></a>

## `epok::pack_projection_input`

**Purpose.** Performs `pack projection input` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool pack_projection_input(const int32_t* world,const int32_t* offset,GteProjectionInput& out)
```

- **Declared at:** [line 110](../../../runtime/gte_geometry.hpp#L110)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const int32_t *` | Input | Value supplied for `world`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `out` | `GteProjectionInput &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * world
// const int32_t * offset
// GteProjectionInput & out

auto result = epok::pack_projection_input(world, offset, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-project-geometry-triple-1"></a>

## `epok::project_geometry_triple`

**Purpose.** Performs `project geometry triple` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool project_geometry_triple(MeshVertices vertices,const int32_t* offset, uint32_t* screen,int32_t* depth)
```

- **Declared at:** [line 341](../../../runtime/gte_geometry.hpp#L341)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertices` | `MeshVertices` | Input | Value supplied for `vertices`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `screen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// MeshVertices vertices
// const int32_t * offset
// uint32_t * screen
// int32_t * depth

auto result = epok::project_geometry_triple(vertices, offset, screen, depth);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-project-geometry-triple-2"></a>

## `epok::project_geometry_triple`

**Purpose.** Performs `project geometry triple` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template<class Number> inline bool project_geometry_triple(const Number (*vertices)[3],const int32_t* offset, uint32_t* screen,int32_t* depth)
```

- **Declared at:** [line 327](../../../runtime/gte_geometry.hpp#L327)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertices` | `const Number (*)[3]` | Callback | Value supplied for `vertices`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `screen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const Number (*)[3] vertices
// const int32_t * offset
// uint32_t * screen
// int32_t * depth

auto result = epok::project_geometry_triple<Number>(vertices, offset, screen, depth);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-project-geometry-vertex-1"></a>

## `epok::project_geometry_vertex`

**Purpose.** Camera output is Q8 with Y up.

**Details.** `offset` is the chunk origin in Q8 added to each vertex so every chunk of an object shares one matrix and identical GTE inputs for shared vertices. `screen` holds SXY2; `flags` is the GTE FLAG register, valid only when no gte_projection_flags bit is set.

**Exact declaration**

```cpp
template<class Vertex> inline void project_geometry_vertex(const Vertex& vertex,const int32_t* offset,int32_t* camera,uint32_t& screen,uint32_t& flags)
```

- **Declared at:** [line 86](../../../runtime/gte_geometry.hpp#L86)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `const Vertex &` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `camera` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |
| `screen` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `flags` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `flags`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** `offset` is the chunk origin in Q8 added to each vertex so every chunk of an object shares one matrix and identical GTE inputs for shared vertices. `screen` holds SXY2; `flags` is the GTE FLAG register, valid only when no gte_projection_flags bit is set.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Vertex

// Assume these named values have been initialized with valid data:
// const Vertex & vertex
// const int32_t * offset
// int32_t * camera
// uint32_t & screen
// uint32_t & flags

epok::project_geometry_vertex<Vertex>(vertex, offset, camera, screen, flags);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-project-packed-vertex-1"></a>

## `epok::project_packed_vertex`

**Purpose.** Performs `project packed vertex` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline bool project_packed_vertex(const GteProjectionInput& vertex,uint32_t& screen,int32_t& depth)
```

- **Declared at:** [line 277](../../../runtime/gte_geometry.hpp#L277)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `const GteProjectionInput &` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `screen` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const GteProjectionInput & vertex
// uint32_t & screen
// int32_t & depth

auto result = epok::project_packed_vertex(vertex, screen, depth);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-read-average-depth-1"></a>

## `epok::read_average_depth`

**Purpose.** Reads average depth as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline int32_t read_average_depth()
```

- **Declared at:** [line 207](../../../runtime/gte_geometry.hpp#L207)
- **Kind:** `function decl`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

auto result = epok::read_average_depth();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-store-projection-triple-1"></a>

## `epok::store_projection_triple`

**Purpose.** Same outputs as finish_projection_triple, stored even when the GTE flagged the projection; returns whether it was unflagged.

**Details.** Used where a saturated result is accepted as is (inexact near clipping) and by validation.

**Exact declaration**

```cpp
inline bool store_projection_triple(uint32_t* screen,int32_t* depth)
```

- **Declared at:** [line 262](../../../runtime/gte_geometry.hpp#L262)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Used where a saturated result is accepted as is (inexact near clipping) and by validation.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * screen
// int32_t * depth

auto result = epok::store_projection_triple(screen, depth);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-transform-geometry-vertex-1"></a>

## `epok::transform_geometry_vertex`

**Purpose.** Performs `transform geometry vertex` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline void transform_geometry_vertex(const int16_t* vertex,int32_t* camera)
```

- **Declared at:** [line 29](../../../runtime/gte_geometry.hpp#L29)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `const int16_t *` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `camera` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const int16_t * vertex
// int32_t * camera

epok::transform_geometry_vertex(vertex, camera);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
