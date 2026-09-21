# Epok API: Retained

> **Header:** `"retained.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/retained.hpp)

This module covers the retained module. It documents 11 public callables declared directly in this header.

## Declared types

`epok::MeshMaterialCache`, `epok::RetainedGeometry`, `epok::RetainedGeometry::State`, `epok::RetainedKey`, `epok::RetainedQuad`, `epok::RetainedStats`

## Callable index

- [`epok::mesh_faces_unlit`](#epok-mesh-faces-unlit-1) — An imported editable mesh owns per-face materials.
- [`epok::MeshMaterialCache::faces_unlit`](#epok-meshmaterialcache-faces-unlit-1) — Performs `faces unlit` as part of the retained module.
- [`epok::retain_mesh_packets`](#epok-retain-mesh-packets-1) — Performs `retain mesh packets` as part of the retained module.
- [`epok::RetainedGeometry::allocate`](#epok-retainedgeometry-allocate-1) — `limit_slot` is the lowest slot the per-frame path used this frame, which a new allocation must not reach.
- [`epok::RetainedGeometry::first_slot`](#epok-retainedgeometry-first-slot-1) — Performs `first slot` as part of the retained module.
- [`epok::RetainedGeometry::forget`](#epok-retainedgeometry-forget-1) — A reused entity slot keeps its quads; a different geometry reallocates.
- [`epok::RetainedGeometry::quads`](#epok-retainedgeometry-quads-1) — Performs `quads` as part of the retained module.
- [`epok::RetainedGeometry::reset`](#epok-retainedgeometry-reset-1) — Resets reset as part of the retained module.
- [`epok::RetainedGeometry::slot_top`](#epok-retainedgeometry-slot-top-1) — First fragment slot beyond the pool; the per-frame path allocates downwards from the capacity.
- [`epok::RetainedGeometry::state`](#epok-retainedgeometry-state-1) — Performs `state` as part of the retained module.
- [`epok::RetainedKey::operator==`](#epok-retainedkey-operator-1) — Performs `operator ==` as part of the retained module.

<a id="epok-mesh-faces-unlit-1"></a>

## `epok::mesh_faces_unlit`

**Purpose.** An imported editable mesh owns per-face materials.

**Details.** The actor tint can still have its default lit flag even when every face explicitly opts out.

**Exact declaration**

```cpp
inline bool mesh_faces_unlit(const MeshGeometry* geometry)
```

- **Declared at:** [line 19](../../../runtime/retained.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The actor tint can still have its default lit flag even when every face explicitly opts out.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * geometry

auto result = epok::mesh_faces_unlit(geometry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshmaterialcache-faces-unlit-1"></a>

## `epok::MeshMaterialCache::faces_unlit`

**Purpose.** Performs `faces unlit` as part of the retained module.

**Exact declaration**

```cpp
bool faces_unlit(const MeshGeometry* source)
```

- **Declared at:** [line 32](../../../runtime/retained.hpp#L32)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const MeshGeometry *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * source

epok::MeshMaterialCache& object = /* obtain a valid instance */;

auto result = object.faces_unlit(source);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-retain-mesh-packets-1"></a>

## `epok::retain_mesh_packets`

**Purpose.** Performs `retain mesh packets` as part of the retained module.

**Exact declaration**

```cpp
inline bool retain_mesh_packets(bool skeletal,bool object_lit,bool baked_colors)
```

- **Declared at:** [line 37](../../../runtime/retained.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `skeletal` | `bool` | Input | Value supplied for `skeletal`. See the exact type and module contract. |
| `object_lit` | `bool` | Input | Value supplied for `object_lit`. See the exact type and module contract. |
| `baked_colors` | `bool` | Input | Value supplied for `baked_colors`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// bool skeletal
// bool object_lit
// bool baked_colors

auto result = epok::retain_mesh_packets(skeletal, object_lit, baked_colors);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-allocate-1"></a>

## `epok::RetainedGeometry::allocate`

**Purpose.** `limit_slot` is the lowest slot the per-frame path used this frame, which a new allocation must not reach.

**Details.** Freed ranges are not reclaimed until reset().

**Exact declaration**

```cpp
bool allocate(size_t index,const MeshGeometry* geometry,size_t quads,uint32_t limit_slot)
```

- **Declared at:** [line 84](../../../runtime/retained.hpp#L84)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |
| `quads` | `int` | Input | Value supplied for `quads`. See the exact type and module contract. |
| `limit_slot` | `int` | Input | Value supplied for `limit_slot`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Freed ranges are not reclaimed until reset().

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// int index
// const MeshGeometry * geometry
// int quads
// int limit_slot

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.allocate(index, geometry, quads, limit_slot);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-retainedgeometry-first-slot-1"></a>

## `epok::RetainedGeometry::first_slot`

**Purpose.** Performs `first slot` as part of the retained module.

**Exact declaration**

```cpp
uint32_t first_slot(size_t index) const
```

- **Declared at:** [line 95](../../../runtime/retained.hpp#L95)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.first_slot(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-forget-1"></a>

## `epok::RetainedGeometry::forget`

**Purpose.** A reused entity slot keeps its quads; a different geometry reallocates.

**Exact declaration**

```cpp
void forget(size_t index)
```

- **Declared at:** [line 79](../../../runtime/retained.hpp#L79)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::RetainedGeometry& object = /* obtain a valid instance */;

object.forget(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-quads-1"></a>

## `epok::RetainedGeometry::quads`

**Purpose.** Performs `quads` as part of the retained module.

**Exact declaration**

```cpp
RetainedQuad* quads(size_t index)
```

- **Declared at:** [line 94](../../../runtime/retained.hpp#L94)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `RetainedQuad *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.quads(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-reset-1"></a>

## `epok::RetainedGeometry::reset`

**Purpose.** Resets reset as part of the retained module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 77](../../../runtime/retained.hpp#L77)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::RetainedGeometry& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-slot-top-1"></a>

## `epok::RetainedGeometry::slot_top`

**Purpose.** First fragment slot beyond the pool; the per-frame path allocates downwards from the capacity.

**Exact declaration**

```cpp
uint32_t slot_top() const
```

- **Declared at:** [line 81](../../../runtime/retained.hpp#L81)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.slot_top();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-state-1"></a>

## `epok::RetainedGeometry::state`

**Purpose.** Performs `state` as part of the retained module.

**Exact declaration**

```cpp
State& state(size_t index)
```

- **Declared at:** [line 93](../../../runtime/retained.hpp#L93)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `State &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.state(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedkey-operator-1"></a>

## `epok::RetainedKey::operator==`

**Purpose.** Performs `operator ==` as part of the retained module.

**Exact declaration**

```cpp
bool operator==(const RetainedKey& o) const
```

- **Declared at:** [line 46](../../../runtime/retained.hpp#L46)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `o` | `const RetainedKey &` | Input | Value supplied for `o`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const RetainedKey & o

epok::RetainedKey& object = /* obtain a valid instance */;

auto result = object.operator==(o);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
