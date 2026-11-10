# Epok API: Lua Value Storage

> **Header:** `"lua_value_storage.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/lua_value_storage.hpp)

This module covers the lua value storage module. It documents 5 public callables declared directly in this header.

## Declared types

`epok::lua::detail::ValueStorage`

## Callable index

- [`epok::lua::detail::ValueStorage::begin`](#epok-lua-detail-valuestorage-begin-1) — Begins begin as part of the lua value storage module.
- [`epok::lua::detail::ValueStorage::contains`](#epok-lua-detail-valuestorage-contains-1) — Performs `contains` as part of the lua value storage module.
- [`epok::lua::detail::ValueStorage::retain`](#epok-lua-detail-valuestorage-retain-1) — Performs `retain` as part of the lua value storage module.
- [`epok::lua::detail::ValueStorage::root`](#epok-lua-detail-valuestorage-root-1) — Performs `root` as part of the lua value storage module.
- [`epok::lua::detail::ValueStorage::sweep`](#epok-lua-detail-valuestorage-sweep-1) — Performs `sweep` as part of the lua value storage module.

<a id="epok-lua-detail-valuestorage-begin-1"></a>

## `epok::lua::detail::ValueStorage::begin`

**Purpose.** Begins begin as part of the lua value storage module.

**Exact declaration**

```cpp
void begin(size_t frame)
```

- **Declared at:** [line 14](../../../runtime/lua_value_storage.hpp#L14)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `size_t` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the lua value storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lua_value_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t frame

epok::lua::detail::ValueStorage& object = /* obtain a valid instance */;

object.begin(frame);
```

**Why choose it.** It provides direct, allocation-conscious access to the lua value storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-lua-detail-valuestorage-contains-1"></a>

## `epok::lua::detail::ValueStorage::contains`

**Purpose.** Performs `contains` as part of the lua value storage module.

**Exact declaration**

```cpp
bool contains(size_t frame, const void* candidate) const
```

- **Declared at:** [line 15](../../../runtime/lua_value_storage.hpp#L15)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `size_t` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `candidate` | `const void *` | Input | Value supplied for `candidate`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lua value storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lua_value_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t frame
// const void * candidate

epok::lua::detail::ValueStorage& object = /* obtain a valid instance */;

auto result = object.contains(frame, candidate);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lua-detail-valuestorage-retain-1"></a>

## `epok::lua::detail::ValueStorage::retain`

**Purpose.** Performs `retain` as part of the lua value storage module.

**Exact declaration**

```cpp
Value* retain(size_t frame, const Value& value)
```

- **Declared at:** [line 21](../../../runtime/lua_value_storage.hpp#L21)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `size_t` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `value` | `const Value &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Value *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lua value storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lua_value_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t frame
// const Value & value

epok::lua::detail::ValueStorage& object = /* obtain a valid instance */;

auto result = object.retain(frame, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the lua value storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lua-detail-valuestorage-root-1"></a>

## `epok::lua::detail::ValueStorage::root`

**Purpose.** Performs `root` as part of the lua value storage module.

**Exact declaration**

```cpp
uint64_t root(size_t frame, const void* candidate) const
```

- **Declared at:** [line 33](../../../runtime/lua_value_storage.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `size_t` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `candidate` | `const void *` | Input | Value supplied for `candidate`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lua value storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lua_value_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t frame
// const void * candidate

epok::lua::detail::ValueStorage& object = /* obtain a valid instance */;

auto result = object.root(frame, candidate);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lua-detail-valuestorage-sweep-1"></a>

## `epok::lua::detail::ValueStorage::sweep`

**Purpose.** Performs `sweep` as part of the lua value storage module.

**Exact declaration**

```cpp
void sweep(size_t frame, uint64_t roots)
```

- **Declared at:** [line 39](../../../runtime/lua_value_storage.hpp#L39)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `size_t` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `roots` | `uint64_t` | Input | Value supplied for `roots`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the lua value storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lua_value_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t frame
// uint64_t roots

epok::lua::detail::ValueStorage& object = /* obtain a valid instance */;

object.sweep(frame, roots);
```

**Why choose it.** It provides direct, allocation-conscious access to the lua value storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
