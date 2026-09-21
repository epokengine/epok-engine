# PsyQo API: Ordering Table

> **Header:** `"psyqo/ordering-table.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/ordering-table.hh)

This module covers GPU ordering tables and packet ordering. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::OrderingTable`, `psyqo::OrderingTableBase`

## Callable index

- [`psyqo::OrderingTable::clear`](#psyqo-orderingtable-clear-1) — Clears the ordering table.
- [`psyqo::OrderingTable::insert`](#psyqo-orderingtable-insert-1) — Inserts a fragment into the ordering table.
- [`psyqo::OrderingTable::OrderingTable<N, safety>`](#psyqo-orderingtable-orderingtable-n-safety-1) — Constructs `psyqo::OrderingTable` for GPU ordering tables and packet ordering.
- [`psyqo::OrderingTableBase::clear`](#psyqo-orderingtablebase-clear-1) — Clears clear as part of GPU ordering tables and packet ordering.

<a id="psyqo-orderingtable-clear-1"></a>

## `psyqo::OrderingTable::clear`

**Purpose.** Clears the ordering table.

**Details.** This function clears the ordering table. The table is automatically cleared by the `GPU` class after it has been sent to the GPU, so this function is only useful if you want to clear the ordering table without sending it to the GPU, which should be a rare use case.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 72](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/ordering-table.hh#L72)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function clears the ordering table. The table is automatically cleared by the `GPU` class after it has been sent to the GPU, so this function is only useful if you want to clear the ordering table without sending it to the GPU, which should be a rare use case.

**Usage pattern**

```cpp
#include "psyqo/ordering-table.hh"

psyqo::OrderingTable& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-orderingtable-insert-1"></a>

## `psyqo::OrderingTable::insert`

**Purpose.** Inserts a fragment into the ordering table.

**Details.** This function inserts a fragment into the ordering table. The fragment will be inserted into the bucket corresponding to its Z value. Any value outside of the range [0, N - 1] will be clamped to the nearest valid value when `safety` is set to `Safe::Yes`, which is the default.

**Exact declaration**

```cpp
template <Fragment Frag> void insert(Frag& frag, int32_t z)
```

- **Declared at:** [line 86](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/ordering-table.hh#L86)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frag` | `Frag &` | Input/output; inspect the function contract | The fragment to insert. |
| `z` | `int32_t` | Input | The Z value of the fragment. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function inserts a fragment into the ordering table. The fragment will be inserted into the bucket corresponding to its Z value. Any value outside of the range [0, N - 1] will be clamped to the nearest valid value when `safety` is set to `Safe::Yes`, which is the default.

**Usage pattern**

```cpp
#include "psyqo/ordering-table.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Frag

// Assume these named values have been initialized with valid data:
// Frag & frag
// int32_t z

psyqo::OrderingTable& object = /* obtain a valid instance */;

object.insert<Frag>(frag, z);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-orderingtable-orderingtable-n-safety-1"></a>

## `psyqo::OrderingTable::OrderingTable<N, safety>`

**Purpose.** Constructs `psyqo::OrderingTable` for GPU ordering tables and packet ordering.

**Exact declaration**

```cpp
OrderingTable()
```

- **Declared at:** [line 62](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/ordering-table.hh#L62)
- **Kind:** `constructor`

**Use it when.** You need GPU ordering tables and packet ordering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/ordering-table.hh"

psyqo::OrderingTable value();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-orderingtablebase-clear-1"></a>

## `psyqo::OrderingTableBase::clear`

**Purpose.** Clears clear as part of GPU ordering tables and packet ordering.

**Exact declaration**

```cpp
static void clear(psyqo::Fragments::ChainEntry* table, size_t size)
```

- **Declared at:** [line 42](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/ordering-table.hh#L42)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `psyqo::Fragments::ChainEntry *` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `size` | `size_t` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU ordering tables and packet ordering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/ordering-table.hh"

// Assume these named values have been initialized with valid data:
// psyqo::Fragments::ChainEntry * table
// size_t size

psyqo::OrderingTableBase::clear(table, size);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
