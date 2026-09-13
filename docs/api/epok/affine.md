# Epok API: Affine

> **Header:** `"affine.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/affine.hpp)

This module covers fixed-point affine transforms and matrix composition. It documents 4 public callables declared directly in this header.

## Declared types

`epok::Affine`

## Callable index

- [`epok::Affine::compose`](#epok-affine-compose-1) — Performs `compose` as part of fixed-point affine transforms and matrix composition.
- [`epok::Affine::identity`](#epok-affine-identity-1) — Performs `identity` as part of fixed-point affine transforms and matrix composition.
- [`epok::Affine::point`](#epok-affine-point-1) — Performs `point` as part of fixed-point affine transforms and matrix composition.
- [`epok::Affine::translated`](#epok-affine-translated-1) — Right-compose a translation without multiplying the unchanged basis by an identity matrix.

<a id="epok-affine-compose-1"></a>

## `epok::Affine::compose`

**Purpose.** Performs `compose` as part of fixed-point affine transforms and matrix composition.

**Exact declaration**

```cpp
Affine compose(const Affine& b) const
```

- **Declared at:** [line 26](../../../runtime/affine.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `const Affine<Number> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Affine<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point affine transforms and matrix composition and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "affine.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Number> & b

epok::Affine& object = /* obtain a valid instance */;

auto result = object.compose(b);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-affine-identity-1"></a>

## `epok::Affine::identity`

**Purpose.** Performs `identity` as part of fixed-point affine transforms and matrix composition.

**Exact declaration**

```cpp
static Affine identity()
```

- **Declared at:** [line 7](../../../runtime/affine.hpp#L7)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Affine<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point affine transforms and matrix composition and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "affine.hpp"

auto result = epok::Affine::identity();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point affine transforms and matrix composition. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-affine-point-1"></a>

## `epok::Affine::point`

**Purpose.** Performs `point` as part of fixed-point affine transforms and matrix composition.

**Exact declaration**

```cpp
void point(const Number* in, Number* out) const
```

- **Declared at:** [line 12](../../../runtime/affine.hpp#L12)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Number *` | Input | Value supplied for `in`. See the exact type and module contract. |
| `out` | `Number *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-point affine transforms and matrix composition and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "affine.hpp"

// Assume these named values have been initialized with valid data:
// const Number * in
// Number * out

epok::Affine& object = /* obtain a valid instance */;

object.point(in, out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-affine-translated-1"></a>

## `epok::Affine::translated`

**Purpose.** Right-compose a translation without multiplying the unchanged basis by an identity matrix.

**Details.** Keep point()'s accumulation order and Q12 rounding.

**Exact declaration**

```cpp
Affine translated(const Number* offset) const
```

- **Declared at:** [line 20](../../../runtime/affine.hpp#L20)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `offset` | `const Number *` | Input | Value supplied for `offset`. See the exact type and module contract. |

**Returns.** Returns `Affine<Number>`. Check the purpose and failure notes before using the value.

**Use it when.** Keep point()'s accumulation order and Q12 rounding.

**Usage pattern**

```cpp
#include "affine.hpp"

// Assume these named values have been initialized with valid data:
// const Number * offset

epok::Affine& object = /* obtain a valid instance */;

auto result = object.translated(offset);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
