# Epok API: Object Storage

> **Header:** `"object_storage.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/object_storage.hpp)

This module covers the object storage module. It documents 9 public callables declared directly in this header.

## Declared types

`epok::object_storage::Arena`, `epok::object_storage::Binding`, `epok::object_storage::Hooks`, `epok::object_storage::Stats`

## Callable index

- [`epok::object_storage::<deduction guide for Binding>`](#epok-object-storage-deduction-guide-for-binding-1) — Performs `<deduction guide for binding>` as part of the object storage module.
- [`epok::object_storage::Arena::acquire`](#epok-object-storage-arena-acquire-1) — Performs `acquire` as part of the object storage module.
- [`epok::object_storage::Arena::mark`](#epok-object-storage-arena-mark-1) — Performs `mark` as part of the object storage module.
- [`epok::object_storage::Arena::release`](#epok-object-storage-arena-release-1) — Performs `release` as part of the object storage module.
- [`epok::object_storage::Arena::used`](#epok-object-storage-arena-used-1) — Performs `used` as part of the object storage module.
- [`epok::object_storage::arena_bytes`](#epok-object-storage-arena-bytes-1) — Performs `arena bytes` as part of the object storage module.
- [`epok::object_storage::Binding::Binding<Units>`](#epok-object-storage-binding-binding-units-1) — Constructs `epok::object_storage::Binding` for the object storage module.
- [`epok::object_storage::capacity_units`](#epok-object-storage-capacity-units-1) — Performs `capacity units` as part of the object storage module.
- [`epok::object_storage::rounded`](#epok-object-storage-rounded-1) — Performs `rounded` as part of the object storage module.

<a id="epok-object-storage-deduction-guide-for-binding-1"></a>

## `epok::object_storage::<deduction guide for Binding>`

**Purpose.** Performs `<deduction guide for binding>` as part of the object storage module.

**Exact declaration**

```cpp
template<size_t Units> Binding(Arena<Units>&)->Binding<Units>
```

- **Declared at:** [line 80](../../../runtime/object_storage.hpp#L80)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Arena<Units> &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Binding<Units>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Units

// Assume these named values have been initialized with valid data:
// Arena<Units> & arg1

epok::object_storage& object = /* obtain a valid instance */;

auto result = object.<deduction guide for Binding><Units>(arg1);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-storage-arena-acquire-1"></a>

## `epok::object_storage::Arena::acquire`

**Purpose.** Performs `acquire` as part of the object storage module.

**Exact declaration**

```cpp
void* acquire(size_t bytes,size_t alignment)
```

- **Declared at:** [line 31](../../../runtime/object_storage.hpp#L31)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `alignment` | `size_t` | Input | Value supplied for `alignment`. See the exact type and module contract. |

**Returns.** Returns `void *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t bytes
// size_t alignment

epok::object_storage::Arena& object = /* obtain a valid instance */;

auto result = object.acquire(bytes, alignment);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-arena-mark-1"></a>

## `epok::object_storage::Arena::mark`

**Purpose.** Performs `mark` as part of the object storage module.

**Exact declaration**

```cpp
void mark(size_t i,bool value)
```

- **Declared at:** [line 30](../../../runtime/object_storage.hpp#L30)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `size_t` | Input | Value supplied for `i`. See the exact type and module contract. |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t i
// bool value

epok::object_storage::Arena& object = /* obtain a valid instance */;

object.mark(i, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-arena-release-1"></a>

## `epok::object_storage::Arena::release`

**Purpose.** Performs `release` as part of the object storage module.

**Exact declaration**

```cpp
bool release(void* block,size_t bytes)
```

- **Declared at:** [line 49](../../../runtime/object_storage.hpp#L49)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `block` | `void *` | Input/output; inspect the function contract | Value supplied for `block`. See the exact type and module contract. |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// void * block
// size_t bytes

epok::object_storage::Arena& object = /* obtain a valid instance */;

auto result = object.release(block, bytes);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-storage-arena-used-1"></a>

## `epok::object_storage::Arena::used`

**Purpose.** Performs `used` as part of the object storage module.

**Exact declaration**

```cpp
bool used(size_t i)const
```

- **Declared at:** [line 29](../../../runtime/object_storage.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `size_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t i

epok::object_storage::Arena& object = /* obtain a valid instance */;

auto result = object.used(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-object-storage-arena-bytes-1"></a>

## `epok::object_storage::arena_bytes`

**Purpose.** Performs `arena bytes` as part of the object storage module.

**Exact declaration**

```cpp
constexpr size_t arena_bytes(size_t units)
```

- **Declared at:** [line 15](../../../runtime/object_storage.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `units` | `size_t` | Input | Value supplied for `units`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t units

auto result = epok::object_storage::arena_bytes(units);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-binding-binding-units-1"></a>

## `epok::object_storage::Binding::Binding<Units>`

**Purpose.** Constructs `epok::object_storage::Binding` for the object storage module.

**Exact declaration**

```cpp
explicit Binding(Arena<Units>& arena)
```

- **Declared at:** [line 74](../../../runtime/object_storage.hpp#L74)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arena` | `Arena<Units> &` | Input/output; inspect the function contract | Value supplied for `arena`. See the exact type and module contract. |

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// Arena<Units> & arena

epok::object_storage::Binding value(arena);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-object-storage-capacity-units-1"></a>

## `epok::object_storage::capacity_units`

**Purpose.** Performs `capacity units` as part of the object storage module.

**Exact declaration**

```cpp
constexpr size_t capacity_units(size_t fixed,size_t demand,size_t limit=65536)
```

- **Declared at:** [line 16](../../../runtime/object_storage.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fixed` | `size_t` | Input | Value supplied for `fixed`. See the exact type and module contract. |
| `demand` | `size_t` | Input | Value supplied for `demand`. See the exact type and module contract. |
| `limit` | `size_t` | Input | Value supplied for `limit`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t fixed
// size_t demand
// size_t limit

auto result = epok::object_storage::capacity_units(fixed, demand, limit);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-object-storage-rounded-1"></a>

## `epok::object_storage::rounded`

**Purpose.** Performs `rounded` as part of the object storage module.

**Exact declaration**

```cpp
constexpr size_t rounded(size_t bytes)
```

- **Declared at:** [line 14](../../../runtime/object_storage.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the object storage module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "object_storage.hpp"

// Assume these named values have been initialized with valid data:
// size_t bytes

auto result = epok::object_storage::rounded(bytes);
```

**Why choose it.** It provides direct, allocation-conscious access to the object storage module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
