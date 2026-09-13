# Epok API: Retained

> **Header:** `"retained.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/retained.hpp)

This module covers the retained module. It documents 8 public callables declared directly in this header.

## Declared types

`epok::RetainedGeometry`, `epok::RetainedGeometry::State`, `epok::RetainedKey`, `epok::RetainedQuad`, `epok::RetainedStats`

## Callable index

- [`epok::RetainedGeometry::allocate`](#epok-retainedgeometry-allocate-1) — `limit_slot` is the lowest slot the per-frame path used this frame, which a new allocation must not reach.
- [`epok::RetainedGeometry::first_slot`](#epok-retainedgeometry-first-slot-1) — Performs `first slot` as part of the retained module.
- [`epok::RetainedGeometry::forget`](#epok-retainedgeometry-forget-1) — A reused entity slot keeps its quads; a different geometry reallocates.
- [`epok::RetainedGeometry::quads`](#epok-retainedgeometry-quads-1) — Performs `quads` as part of the retained module.
- [`epok::RetainedGeometry::reset`](#epok-retainedgeometry-reset-1) — Resets reset as part of the retained module.
- [`epok::RetainedGeometry::slot_top`](#epok-retainedgeometry-slot-top-1) — First fragment slot beyond the pool; the per-frame path allocates downwards from the capacity.
- [`epok::RetainedGeometry::state`](#epok-retainedgeometry-state-1) — Performs `state` as part of the retained module.
- [`epok::RetainedKey::operator==`](#epok-retainedkey-operator-1) — Performs `operator ==` as part of the retained module.

<a id="epok-retainedgeometry-allocate-1"></a>

## `epok::RetainedGeometry::allocate`

**Purpose.** `limit_slot` is the lowest slot the per-frame path used this frame, which a new allocation must not reach.

**Details.** Freed ranges are not reclaimed until reset().

**Exact declaration**

```cpp
bool allocate(size_t index,const MeshGeometry* geometry,size_t quads,uint32_t limit_slot)
```

- **Declared at:** [line 61](../../../runtime/retained.hpp#L61)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |
| `quads` | `size_t` | Input | Value supplied for `quads`. See the exact type and module contract. |
| `limit_slot` | `uint32_t` | Input | Value supplied for `limit_slot`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Freed ranges are not reclaimed until reset().

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const MeshGeometry * geometry
// size_t quads
// uint32_t limit_slot

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

- **Declared at:** [line 72](../../../runtime/retained.hpp#L72)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

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

- **Declared at:** [line 56](../../../runtime/retained.hpp#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

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

- **Declared at:** [line 71](../../../runtime/retained.hpp#L71)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `RetainedQuad *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

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

- **Declared at:** [line 54](../../../runtime/retained.hpp#L54)
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

- **Declared at:** [line 58](../../../runtime/retained.hpp#L58)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 70](../../../runtime/retained.hpp#L70)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `State &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

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

- **Declared at:** [line 23](../../../runtime/retained.hpp#L23)
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
