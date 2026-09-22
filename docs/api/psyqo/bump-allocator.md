# PsyQo API: Bump Allocator

> **Header:** `"psyqo/bump-allocator.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh)

This module covers the bump allocator module. It documents 5 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::BumpAllocator`

## Callable index

- [`psyqo::BumpAllocator::allocate`](#psyqo-bumpallocator-allocate-1) — Performs `allocate` as part of the bump allocator module.
- [`psyqo::BumpAllocator::allocateFragment`](#psyqo-bumpallocator-allocatefragment-1) — Performs `allocate fragment` as part of the bump allocator module.
- [`psyqo::BumpAllocator::remaining`](#psyqo-bumpallocator-remaining-1) — Performs `remaining` as part of the bump allocator module.
- [`psyqo::BumpAllocator::reset`](#psyqo-bumpallocator-reset-1) — Resets reset as part of the bump allocator module.
- [`psyqo::BumpAllocator::used`](#psyqo-bumpallocator-used-1) — Performs `used` as part of the bump allocator module.

<a id="psyqo-bumpallocator-allocate-1"></a>

## `psyqo::BumpAllocator::allocate`

**Purpose.** Performs `allocate` as part of the bump allocator module.

**Exact declaration**

```cpp
template <typename T, typename... Args> T &allocate(Args &&...args)
```

- **Declared at:** [line 69](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh#L69)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `args` | `Args &&...` | Consumed or moved input | Value supplied for `args`. See the exact type and module contract. |

**Returns.** Returns `T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the bump allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bump-allocator.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T, Args

// Assume these named values have been initialized with valid data:
// Args &&... args

psyqo::BumpAllocator& object = /* obtain a valid instance */;

auto result = object.allocate<T, Args>(args);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-bumpallocator-allocatefragment-1"></a>

## `psyqo::BumpAllocator::allocateFragment`

**Purpose.** Performs `allocate fragment` as part of the bump allocator module.

**Exact declaration**

```cpp
template <Primitive P, typename... Args> Fragments::SimpleFragment<P> &allocateFragment(Args &&...args)
```

- **Declared at:** [line 59](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh#L59)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `args` | `Args &&...` | Consumed or moved input | Value supplied for `args`. See the exact type and module contract. |

**Returns.** Returns `Fragments::SimpleFragment<P> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the bump allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bump-allocator.hh"

// Replace these template arguments with types or values accepted by the declaration:
// P, Args

// Assume these named values have been initialized with valid data:
// Args &&... args

psyqo::BumpAllocator& object = /* obtain a valid instance */;

auto result = object.allocateFragment<P, Args>(args);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-bumpallocator-remaining-1"></a>

## `psyqo::BumpAllocator::remaining`

**Purpose.** Performs `remaining` as part of the bump allocator module.

**Exact declaration**

```cpp
size_t remaining() const
```

- **Declared at:** [line 85](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh#L85)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the bump allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bump-allocator.hh"

psyqo::BumpAllocator& object = /* obtain a valid instance */;

auto result = object.remaining();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-bumpallocator-reset-1"></a>

## `psyqo::BumpAllocator::reset`

**Purpose.** Resets reset as part of the bump allocator module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 84](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh#L84)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the bump allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bump-allocator.hh"

psyqo::BumpAllocator& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the bump allocator module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-bumpallocator-used-1"></a>

## `psyqo::BumpAllocator::used`

**Purpose.** Performs `used` as part of the bump allocator module.

**Exact declaration**

```cpp
size_t used() const
```

- **Declared at:** [line 86](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bump-allocator.hh#L86)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the bump allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bump-allocator.hh"

psyqo::BumpAllocator& object = /* obtain a valid instance */;

auto result = object.used();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
