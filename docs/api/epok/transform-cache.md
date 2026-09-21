# Epok API: Transform Cache

> **Header:** `"transform_cache.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/transform_cache.hpp)

This module covers the transform cache module. It documents 4 public callables declared directly in this header.

## Declared types

`epok::TransformCache`, `epok::TransformCache::Entry`

## Callable index

- [`epok::TransformCache::clear`](#epok-transformcache-clear-1) — Clears clear as part of the transform cache module.
- [`epok::TransformCache::revision`](#epok-transformcache-revision-1) — Performs `revision` as part of the transform cache module.
- [`epok::TransformCache::sync`](#epok-transformcache-sync-1) — Performs `sync` as part of the transform cache module.
- [`epok::TransformCache::sync_root`](#epok-transformcache-sync-root-1) — Common physics case: an unparented collider.

<a id="epok-transformcache-clear-1"></a>

## `epok::TransformCache::clear`

**Purpose.** Clears clear as part of the transform cache module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 21](../../../runtime/transform_cache.hpp#L21)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the transform cache module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transform_cache.hpp"

epok::TransformCache& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the transform cache module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-transformcache-revision-1"></a>

## `epok::TransformCache::revision`

**Purpose.** Performs `revision` as part of the transform cache module.

**Exact declaration**

```cpp
uint32_t revision(size_t index) const
```

- **Declared at:** [line 22](../../../runtime/transform_cache.hpp#L22)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the transform cache module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transform_cache.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::TransformCache& object = /* obtain a valid instance */;

auto result = object.revision(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-transformcache-sync-1"></a>

## `epok::TransformCache::sync`

**Purpose.** Performs `sync` as part of the transform cache module.

**Exact declaration**

```cpp
template<class Objects,class Matrices,class MakeLocal> void sync(const Objects& objects,Matrices& world,size_t count,MakeLocal make_local,const bool* selected=nullptr)
```

- **Declared at:** [line 44](../../../runtime/transform_cache.hpp#L44)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `Matrices &` | Input/output; inspect the function contract | Value supplied for `world`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `make_local` | `MakeLocal` | Input | Value supplied for `make_local`. See the exact type and module contract. |
| `selected` | `const bool *` | Input | Value supplied for `selected`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the transform cache module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transform_cache.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects, Matrices, MakeLocal

// Assume these named values have been initialized with valid data:
// const Objects & objects
// Matrices & world
// int count
// MakeLocal make_local
// const bool * selected

epok::TransformCache& object = /* obtain a valid instance */;

object.sync<Objects, Matrices, MakeLocal>(objects, world, count, make_local, selected);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-transformcache-sync-root-1"></a>

## `epok::TransformCache::sync_root`

**Purpose.** Common physics case: an unparented collider.

**Details.** A query can synchronize just that root; a later full/subset sync still propagates its change to visual or collider children through descendants_dirty.

**Exact declaration**

```cpp
template<class Object,class MakeLocal> void sync_root(const Object& object,Affine<Number>& world,size_t index,MakeLocal make_local)
```

- **Declared at:** [line 27](../../../runtime/transform_cache.hpp#L27)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `object` | `const Object &` | Input | Value supplied for `object`. See the exact type and module contract. |
| `world` | `Affine<Number> &` | Input/output; inspect the function contract | Value supplied for `world`. See the exact type and module contract. |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `make_local` | `MakeLocal` | Input | Value supplied for `make_local`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A query can synchronize just that root; a later full/subset sync still propagates its change to visual or collider children through descendants_dirty.

**Usage pattern**

```cpp
#include "transform_cache.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Object, MakeLocal

// Assume these named values have been initialized with valid data:
// const Object & object
// Affine<Number> & world
// int index
// MakeLocal make_local

epok::TransformCache& object = /* obtain a valid instance */;

object.sync_root<Object, MakeLocal>(object, world, index, make_local);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
