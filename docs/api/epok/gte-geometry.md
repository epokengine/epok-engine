# Epok API: Gte Geometry

> **Header:** `"gte_geometry.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gte_geometry.hpp)

This module covers Geometry Transformation Engine math and register operations. It documents 5 public callables declared directly in this header.

## Callable index

- [`epok::load_geometry_matrix`](#epok-load-geometry-matrix-1) — Use MAC1..3, not the saturated 16-bit IR registers: the existing renderer's Q12 camera coordinates regularly exceed +/-8 world units.
- [`epok::load_projection_matrix`](#epok-load-projection-matrix-1) — Loads projection matrix as part of Geometry Transformation Engine math and register operations.
- [`epok::load_projection_screen`](#epok-load-projection-screen-1) — H is the pixel focal length shared by both axes; offsets are the screen centre.
- [`epok::project_geometry_vertex`](#epok-project-geometry-vertex-1) — Camera output is Q8 with Y up.
- [`epok::transform_geometry_vertex`](#epok-transform-geometry-vertex-1) — Performs `transform geometry vertex` as part of Geometry Transformation Engine math and register operations.

<a id="epok-load-geometry-matrix-1"></a>

## `epok::load_geometry_matrix`

**Purpose.** Use MAC1..3, not the saturated 16-bit IR registers: the existing renderer's Q12 camera coordinates regularly exceed +/-8 world units.

**Details.** Perspective and near/far clipping retain the software path and its full coordinate range.

**Exact declaration**

```cpp
template<class Number> bool load_geometry_matrix(const Affine<Number>& m)
```

- **Declared at:** [line 10](../../../runtime/gte_geometry.hpp#L10)
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

<a id="epok-load-projection-matrix-1"></a>

## `epok::load_projection_matrix`

**Purpose.** Loads projection matrix as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template<class Number> bool load_projection_matrix(const Affine<Number>& m)
```

- **Declared at:** [line 47](../../../runtime/gte_geometry.hpp#L47)
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

- **Declared at:** [line 65](../../../runtime/gte_geometry.hpp#L65)
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

<a id="epok-project-geometry-vertex-1"></a>

## `epok::project_geometry_vertex`

**Purpose.** Camera output is Q8 with Y up.

**Details.** `offset` is the chunk origin in Q8 added to each vertex so every chunk of an object shares one matrix and identical GTE inputs for shared vertices. `screen` holds SXY2; `flags` is the GTE FLAG register, valid only when no gte_projection_flags bit is set.

**Exact declaration**

```cpp
inline void project_geometry_vertex(const int16_t* vertex,const int32_t* offset,int32_t* camera,uint32_t& screen,uint32_t& flags)
```

- **Declared at:** [line 77](../../../runtime/gte_geometry.hpp#L77)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vertex` | `const int16_t *` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `camera` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |
| `screen` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `flags` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `flags`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** `offset` is the chunk origin in Q8 added to each vertex so every chunk of an object shares one matrix and identical GTE inputs for shared vertices. `screen` holds SXY2; `flags` is the GTE FLAG register, valid only when no gte_projection_flags bit is set.

**Usage pattern**

```cpp
#include "gte_geometry.hpp"

// Assume these named values have been initialized with valid data:
// const int16_t * vertex
// const int32_t * offset
// int32_t * camera
// uint32_t & screen
// uint32_t & flags

epok::project_geometry_vertex(vertex, offset, camera, screen, flags);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-transform-geometry-vertex-1"></a>

## `epok::transform_geometry_vertex`

**Purpose.** Performs `transform geometry vertex` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
inline void transform_geometry_vertex(const int16_t* vertex,int32_t* camera)
```

- **Declared at:** [line 28](../../../runtime/gte_geometry.hpp#L28)
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
