# Epok API: Visibility

> **Header:** `"visibility.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/visibility.hpp)

This module covers frustum tests and chunk visibility. It documents 13 public callables declared directly in this header.

## Declared types

`epok::ChunkBasisBounds`, `epok::ChunkVisibility`, `epok::ChunkVisibilityCache`, `epok::ChunkVisibilityMask`, `epok::ChunkVisibilityQuery`

## Callable index

- [`epok::ChunkVisibilityMask::candidate`](#epok-chunkvisibilitymask-candidate-1) — Performs `candidate` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::add_plane`](#epok-chunkvisibilityquery-add-plane-1) — x*local.x + y*local.y + z*local.z + d*4096 >= 0, all in Q12.
- [`epok::ChunkVisibilityQuery::basis_cache_active`](#epok-chunkvisibilityquery-basis-cache-active-1) — Performs `basis cache active` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::bounds_cache_active`](#epok-chunkvisibilityquery-bounds-cache-active-1) — Performs `bounds cache active` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::cached_basis_bounds`](#epok-chunkvisibilityquery-cached-basis-bounds-1) — Performs `cached basis bounds` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::cached_bounds`](#epok-chunkvisibilityquery-cached-bounds-1) — Performs `cached bounds` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::candidate`](#epok-chunkvisibilityquery-candidate-1) — Performs `candidate` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::ChunkVisibilityQuery`](#epok-chunkvisibilityquery-chunkvisibilityquery-1) — Constructs `epok::ChunkVisibilityQuery` for frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::merged`](#epok-chunkvisibilityquery-merged-1) — Performs `merged` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::merged_mask`](#epok-chunkvisibilityquery-merged-mask-1) — Generated metadata always supplies a cache.
- [`epok::ChunkVisibilityQuery::remember_basis_bounds`](#epok-chunkvisibilityquery-remember-basis-bounds-1) — Performs `remember basis bounds` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::remember_bounds`](#epok-chunkvisibilityquery-remember-bounds-1) — Performs `remember bounds` as part of frustum tests and chunk visibility.
- [`epok::ChunkVisibilityQuery::reuse_matrix`](#epok-chunkvisibilityquery-reuse-matrix-1) — Compare the source matrix before constructing its five derived planes.

<a id="epok-chunkvisibilitymask-candidate-1"></a>

## `epok::ChunkVisibilityMask::candidate`

**Purpose.** Performs `candidate` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
bool candidate(size_t chunk) const
```

- **Declared at:** [line 39](../../../runtime/visibility.hpp#L39)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk

epok::ChunkVisibilityMask& object = /* obtain a valid instance */;

auto result = object.candidate(chunk);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-chunkvisibilityquery-add-plane-1"></a>

## `epok::ChunkVisibilityQuery::add_plane`

**Purpose.** x*local.x + y*local.y + z*local.z + d*4096 >= 0, all in Q12.

**Details.** Unsupported ranges retain the normal renderer's full bounds test.

**Exact declaration**

```cpp
void add_plane(int64_t x,int64_t y,int64_t z,int64_t d)
```

- **Declared at:** [line 143](../../../runtime/visibility.hpp#L143)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int64_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int64_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int64_t` | Input | Value supplied for `z`. See the exact type and module contract. |
| `d` | `int64_t` | Input | Value supplied for `d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Unsupported ranges retain the normal renderer's full bounds test.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// int64_t x
// int64_t y
// int64_t z
// int64_t d

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

object.add_plane(x, y, z, d);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-basis-cache-active-1"></a>

## `epok::ChunkVisibilityQuery::basis_cache_active`

**Purpose.** Performs `basis cache active` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
bool basis_cache_active() const
```

- **Declared at:** [line 113](../../../runtime/visibility.hpp#L113)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.basis_cache_active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-chunkvisibilityquery-bounds-cache-active-1"></a>

## `epok::ChunkVisibilityQuery::bounds_cache_active`

**Purpose.** Performs `bounds cache active` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
bool bounds_cache_active() const
```

- **Declared at:** [line 112](../../../runtime/visibility.hpp#L112)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.bounds_cache_active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-chunkvisibilityquery-cached-basis-bounds-1"></a>

## `epok::ChunkVisibilityQuery::cached_basis_bounds`

**Purpose.** Performs `cached basis bounds` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
const ChunkBasisBounds* cached_basis_bounds(size_t chunk) const
```

- **Declared at:** [line 114](../../../runtime/visibility.hpp#L114)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |

**Returns.** Returns `const ChunkBasisBounds *`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.cached_basis_bounds(chunk);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-cached-bounds-1"></a>

## `epok::ChunkVisibilityQuery::cached_bounds`

**Purpose.** Performs `cached bounds` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
int cached_bounds(size_t chunk) const
```

- **Declared at:** [line 127](../../../runtime/visibility.hpp#L127)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.cached_bounds(chunk);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-candidate-1"></a>

## `epok::ChunkVisibilityQuery::candidate`

**Purpose.** Performs `candidate` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
bool candidate(size_t chunk) const
```

- **Declared at:** [line 190](../../../runtime/visibility.hpp#L190)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.candidate(chunk);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-chunkvisibilityquery-chunkvisibilityquery-1"></a>

## `epok::ChunkVisibilityQuery::ChunkVisibilityQuery`

**Purpose.** Constructs `epok::ChunkVisibilityQuery` for frustum tests and chunk visibility.

**Exact declaration**

```cpp
explicit ChunkVisibilityQuery(const ChunkVisibility* value):gri
```

- **Declared at:** [line 75](../../../runtime/visibility.hpp#L75)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const ChunkVisibility *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// const ChunkVisibility * value

epok::ChunkVisibilityQuery value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-chunkvisibilityquery-merged-1"></a>

## `epok::ChunkVisibilityQuery::merged`

**Purpose.** Performs `merged` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
ChunkVisibilityMask merged() const
```

- **Declared at:** [line 189](../../../runtime/visibility.hpp#L189)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ChunkVisibilityMask`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.merged();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-merged-mask-1"></a>

## `epok::ChunkVisibilityQuery::merged_mask`

**Purpose.** Generated metadata always supplies a cache.

**Details.** A null result means the current view has no useful rejection; the renderer can bypass bit tests.

**Exact declaration**

```cpp
const uint32_t* merged_mask() const
```

- **Declared at:** [line 164](../../../runtime/visibility.hpp#L164)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const uint32_t *`. Check the purpose and failure notes before using the value.

**Use it when.** A null result means the current view has no useful rejection; the renderer can bypass bit tests.

**Usage pattern**

```cpp
#include "visibility.hpp"

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.merged_mask();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-remember-basis-bounds-1"></a>

## `epok::ChunkVisibilityQuery::remember_basis_bounds`

**Purpose.** Performs `remember basis bounds` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
void remember_basis_bounds(size_t chunk,const int32_t center[3],const int32_t extent[3]) const
```

- **Declared at:** [line 120](../../../runtime/visibility.hpp#L120)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |
| `center` | `const int32_t[3]` | Input | Value supplied for `center`. See the exact type and module contract. |
| `extent` | `const int32_t[3]` | Input | Value supplied for `extent`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk
// const int32_t[3] center
// const int32_t[3] extent

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

object.remember_basis_bounds(chunk, center, extent);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-remember-bounds-1"></a>

## `epok::ChunkVisibilityQuery::remember_bounds`

**Purpose.** Performs `remember bounds` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
void remember_bounds(size_t chunk,bool accepted) const
```

- **Declared at:** [line 134](../../../runtime/visibility.hpp#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chunk` | `size_t` | Input | Value supplied for `chunk`. See the exact type and module contract. |
| `accepted` | `bool` | Input | Value supplied for `accepted`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// size_t chunk
// bool accepted

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

object.remember_bounds(chunk, accepted);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-chunkvisibilityquery-reuse-matrix-1"></a>

## `epok::ChunkVisibilityQuery::reuse_matrix`

**Purpose.** Compare the source matrix before constructing its five derived planes.

**Details.** Call merged() after rebuilding so a partial query cannot validate a key.

**Exact declaration**

```cpp
bool reuse_matrix(const int32_t rows[3][3],const int32_t translation[3], bool defer_until_stable=false)
```

- **Declared at:** [line 78](../../../runtime/visibility.hpp#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rows` | `const int32_t[3][3]` | Input | Value supplied for `rows`. See the exact type and module contract. |
| `translation` | `const int32_t[3]` | Input | Value supplied for `translation`. See the exact type and module contract. |
| `defer_until_stable` | `bool` | Input | Value supplied for `defer_until_stable`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Call merged() after rebuilding so a partial query cannot validate a key.

**Usage pattern**

```cpp
#include "visibility.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t[3][3] rows
// const int32_t[3] translation
// bool defer_until_stable

epok::ChunkVisibilityQuery& object = /* obtain a valid instance */;

auto result = object.reuse_matrix(rows, translation, defer_until_stable);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
