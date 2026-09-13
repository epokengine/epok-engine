# PsyQo API: Buffer

> **Header:** `"psyqo/buffer.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/buffer.hh)

This module covers the buffer module. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Buffer`, `psyqo::PsyqoAllocator`

## Callable index

- [`psyqo::PsyqoAllocator::allocate`](#psyqo-psyqoallocator-allocate-1) — Performs `allocate` as part of the buffer module.
- [`psyqo::PsyqoAllocator::copy`](#psyqo-psyqoallocator-copy-1) — Performs `copy` as part of the buffer module.
- [`psyqo::PsyqoAllocator::deallocate`](#psyqo-psyqoallocator-deallocate-1) — Performs `deallocate` as part of the buffer module.
- [`psyqo::PsyqoAllocator::reallocate`](#psyqo-psyqoallocator-reallocate-1) — Performs `reallocate` as part of the buffer module.

<a id="psyqo-psyqoallocator-allocate-1"></a>

## `psyqo::PsyqoAllocator::allocate`

**Purpose.** Performs `allocate` as part of the buffer module.

**Exact declaration**

```cpp
template <typename T> static inline T* allocate(size_t size)
```

- **Declared at:** [line 36](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/buffer.hh#L36)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `size` | `size_t` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the buffer module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/buffer.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// size_t size

auto result = psyqo::PsyqoAllocator::allocate<T>(size);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-psyqoallocator-copy-1"></a>

## `psyqo::PsyqoAllocator::copy`

**Purpose.** Performs `copy` as part of the buffer module.

**Exact declaration**

```cpp
template <typename T> static inline void copy(T* dst, const T* src, size_t size)
```

- **Declared at:** [line 45](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/buffer.hh#L45)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dst` | `T *` | Input/output; inspect the function contract | Value supplied for `dst`. See the exact type and module contract. |
| `src` | `const T *` | Input | Value supplied for `src`. See the exact type and module contract. |
| `size` | `size_t` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the buffer module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/buffer.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// T * dst
// const T * src
// size_t size

psyqo::PsyqoAllocator::copy<T>(dst, src, size);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-psyqoallocator-deallocate-1"></a>

## `psyqo::PsyqoAllocator::deallocate`

**Purpose.** Performs `deallocate` as part of the buffer module.

**Exact declaration**

```cpp
static inline void deallocate(void* ptr)
```

- **Declared at:** [line 39](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/buffer.hh#L39)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `void *` | Input/output; inspect the function contract | Value supplied for `ptr`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the buffer module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/buffer.hh"

// Assume these named values have been initialized with valid data:
// void * ptr

psyqo::PsyqoAllocator::deallocate(ptr);
```

**Why choose it.** It provides direct, allocation-conscious access to the buffer module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-psyqoallocator-reallocate-1"></a>

## `psyqo::PsyqoAllocator::reallocate`

**Purpose.** Performs `reallocate` as part of the buffer module.

**Exact declaration**

```cpp
template <typename T> static inline T* reallocate(void* ptr, size_t size)
```

- **Declared at:** [line 41](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/buffer.hh#L41)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `void *` | Input/output; inspect the function contract | Value supplied for `ptr`. See the exact type and module contract. |
| `size` | `size_t` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the buffer module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/buffer.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// void * ptr
// size_t size

auto result = psyqo::PsyqoAllocator::reallocate<T>(ptr, size);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
