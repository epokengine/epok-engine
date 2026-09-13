# Epok API: Lifecycle

> **Header:** `"lifecycle.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/lifecycle.hpp)

This module covers entity creation, activation and destruction. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::descendant`](#epok-descendant-1) — Performs `descendant` as part of entity creation, activation and destruction.
- [`epok::entity_index`](#epok-entity-index-1) — Performs `entity index` as part of entity creation, activation and destruction.
- [`epok::next_generation`](#epok-next-generation-1) — Performs `next generation` as part of entity creation, activation and destruction.

<a id="epok-descendant-1"></a>

## `epok::descendant`

**Purpose.** Performs `descendant` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline bool descendant(size_t child,size_t ancestor)
```

- **Declared at:** [line 41](../../../runtime/lifecycle.hpp#L41)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `size_t` | Input | Value supplied for `child`. See the exact type and module contract. |
| `ancestor` | `size_t` | Input | Value supplied for `ancestor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// size_t child
// size_t ancestor

auto result = epok::descendant(child, ancestor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-entity-index-1"></a>

## `epok::entity_index`

**Purpose.** Performs `entity index` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline int entity_index(const Entity* entity)
```

- **Declared at:** [line 9](../../../runtime/lifecycle.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const Entity *` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// const Entity * entity

auto result = epok::entity_index(entity);
```

**Why choose it.** It provides direct, allocation-conscious access to entity creation, activation and destruction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-next-generation-1"></a>

## `epok::next_generation`

**Purpose.** Performs `next generation` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline uint32_t next_generation(uint32_t n)
```

- **Declared at:** [line 20](../../../runtime/lifecycle.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `n` | `uint32_t` | Input | Value supplied for `n`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t n

auto result = epok::next_generation(n);
```

**Why choose it.** It provides direct, allocation-conscious access to entity creation, activation and destruction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
