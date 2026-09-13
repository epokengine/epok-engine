# PsyQo API: Primitives / Misc

> **Header:** `"psyqo/primitives/misc.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/misc.hh)

This module covers typed PlayStation GPU primitives. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::FastFill`, `psyqo::Prim::FlushCache`

## Callable index

- [`psyqo::Prim::FastFill::FastFill`](#psyqo-prim-fastfill-fastfill-1) — Constructs `psyqo::Prim::FastFill` for typed PlayStation GPU primitives.
- [`psyqo::Prim::FastFill::FastFill`](#psyqo-prim-fastfill-fastfill-2) — Constructs `psyqo::Prim::FastFill` for typed PlayStation GPU primitives.
- [`psyqo::Prim::FastFill::setColor`](#psyqo-prim-fastfill-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::FlushCache::FlushCache`](#psyqo-prim-flushcache-flushcache-1) — Constructs `psyqo::Prim::FlushCache` for typed PlayStation GPU primitives.

<a id="psyqo-prim-fastfill-fastfill-1"></a>

## `psyqo::Prim::FastFill::FastFill`

**Purpose.** Constructs `psyqo::Prim::FastFill` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
FastFill() : c
```

- **Declared at:** [line 64](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/misc.hh#L64)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/misc.hh"

psyqo::Prim::FastFill value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-fastfill-fastfill-2"></a>

## `psyqo::Prim::FastFill::FastFill`

**Purpose.** Constructs `psyqo::Prim::FastFill` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
FastFill(Color c) : c
```

- **Declared at:** [line 65](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/misc.hh#L65)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/misc.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::FastFill value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-fastfill-setcolor-1"></a>

## `psyqo::Prim::FastFill::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
FastFill& setColor(Color c)
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/misc.hh#L66)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `FastFill &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/misc.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::FastFill& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-flushcache-flushcache-1"></a>

## `psyqo::Prim::FlushCache::FlushCache`

**Purpose.** Constructs `psyqo::Prim::FlushCache` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
FlushCache() : c
```

- **Declared at:** [line 46](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/misc.hh#L46)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/misc.hh"

psyqo::Prim::FlushCache value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
