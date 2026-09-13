# PsyQo API: Fragments

> **Header:** `"psyqo/fragments.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh)

This module covers the fragments module. It documents 16 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Fragments::ChainEntry`, `psyqo::Fragments::ChainEntryPC`, `psyqo::Fragments::ChainEntryPS1`, `psyqo::Fragments::FixedFragment`, `psyqo::Fragments::FixedFragmentWithPrologue`, `psyqo::Fragments::SimpleFragment`

## Callable index

- [`psyqo::Fragments::ChainEntryPC::set`](#psyqo-fragments-chainentrypc-set-1) — Sets set as part of the fragments module.
- [`psyqo::Fragments::ChainEntryPC::setEndMarker`](#psyqo-fragments-chainentrypc-setendmarker-1) — Sets end marker as part of the fragments module.
- [`psyqo::Fragments::ChainEntryPS1::set`](#psyqo-fragments-chainentryps1-set-1) — Sets set as part of the fragments module.
- [`psyqo::Fragments::ChainEntryPS1::setEndMarker`](#psyqo-fragments-chainentryps1-setendmarker-1) — Sets end marker as part of the fragments module.
- [`psyqo::Fragments::FixedFragment::FixedFragment<Prim, N>`](#psyqo-fragments-fixedfragment-fixedfragment-prim-n-1) — Constructs `psyqo::Fragments::FixedFragment` for the fragments module.
- [`psyqo::Fragments::FixedFragment::FixedFragment<Prim, N>`](#psyqo-fragments-fixedfragment-fixedfragment-prim-n-2) — Constructs `psyqo::Fragments::FixedFragment` for the fragments module.
- [`psyqo::Fragments::FixedFragment::getActualFragmentSize`](#psyqo-fragments-fixedfragment-getactualfragmentsize-1) — Returns actual fragment size as part of the fragments module.
- [`psyqo::Fragments::FixedFragment::maxSize`](#psyqo-fragments-fixedfragment-maxsize-1) — Performs `max size` as part of the fragments module.
- [`psyqo::Fragments::FixedFragmentWithPrologue::FixedFragmentWithPrologue<P, Prim, N>`](#psyqo-fragments-fixedfragmentwithprologue-fixedfragmentwithprologue-p-prim-n-1) — Constructs `psyqo::Fragments::FixedFragmentWithPrologue` for the fragments module.
- [`psyqo::Fragments::FixedFragmentWithPrologue::FixedFragmentWithPrologue<P, Prim, N>`](#psyqo-fragments-fixedfragmentwithprologue-fixedfragmentwithprologue-p-prim-n-2) — Constructs `psyqo::Fragments::FixedFragmentWithPrologue` for the fragments module.
- [`psyqo::Fragments::FixedFragmentWithPrologue::getActualFragmentSize`](#psyqo-fragments-fixedfragmentwithprologue-getactualfragmentsize-1) — Returns actual fragment size as part of the fragments module.
- [`psyqo::Fragments::FixedFragmentWithPrologue::maxSize`](#psyqo-fragments-fixedfragmentwithprologue-maxsize-1) — Performs `max size` as part of the fragments module.
- [`psyqo::Fragments::SimpleFragment::getActualFragmentSize`](#psyqo-fragments-simplefragment-getactualfragmentsize-1) — Returns actual fragment size as part of the fragments module.
- [`psyqo::Fragments::SimpleFragment::maxSize`](#psyqo-fragments-simplefragment-maxsize-1) — Performs `max size` as part of the fragments module.
- [`psyqo::Fragments::SimpleFragment::SimpleFragment<Prim>`](#psyqo-fragments-simplefragment-simplefragment-prim-1) — Constructs `psyqo::Fragments::SimpleFragment` for the fragments module.
- [`psyqo::Fragments::SimpleFragment::SimpleFragment<Prim>`](#psyqo-fragments-simplefragment-simplefragment-prim-2) — Performs `simple fragment<prim>` as part of the fragments module.

<a id="psyqo-fragments-chainentrypc-set-1"></a>

## `psyqo::Fragments::ChainEntryPC::set`

**Purpose.** Sets set as part of the fragments module.

**Exact declaration**

```cpp
void set(ChainEntry* next_, unsigned size_)
```

- **Declared at:** [line 82](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L82)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `next_` | `ChainEntry *` | Input/output; inspect the function contract | Value supplied for `next_`. See the exact type and module contract. |
| `size_` | `unsigned int` | Input | Value supplied for `size_`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Assume these named values have been initialized with valid data:
// ChainEntry * next_
// unsigned int size_

psyqo::Fragments::ChainEntryPC& object = /* obtain a valid instance */;

object.set(next_, size_);
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fragments-chainentrypc-setendmarker-1"></a>

## `psyqo::Fragments::ChainEntryPC::setEndMarker`

**Purpose.** Sets end marker as part of the fragments module.

**Exact declaration**

```cpp
void setEndMarker()
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L78)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::ChainEntryPC& object = /* obtain a valid instance */;

object.setEndMarker();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-chainentryps1-set-1"></a>

## `psyqo::Fragments::ChainEntryPS1::set`

**Purpose.** Sets set as part of the fragments module.

**Exact declaration**

```cpp
void set(ChainEntryPS1* next, unsigned size)
```

- **Declared at:** [line 93](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L93)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `next` | `ChainEntryPS1 *` | Input/output; inspect the function contract | Value supplied for `next`. See the exact type and module contract. |
| `size` | `unsigned int` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Assume these named values have been initialized with valid data:
// ChainEntryPS1 * next
// unsigned int size

psyqo::Fragments::ChainEntryPS1& object = /* obtain a valid instance */;

object.set(next, size);
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fragments-chainentryps1-setendmarker-1"></a>

## `psyqo::Fragments::ChainEntryPS1::setEndMarker`

**Purpose.** Sets end marker as part of the fragments module.

**Exact declaration**

```cpp
void setEndMarker()
```

- **Declared at:** [line 90](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L90)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::ChainEntryPS1& object = /* obtain a valid instance */;

object.setEndMarker();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragment-fixedfragment-prim-n-1"></a>

## `psyqo::Fragments::FixedFragment::FixedFragment<Prim, N>`

**Purpose.** Constructs `psyqo::Fragments::FixedFragment` for the fragments module.

**Exact declaration**

```cpp
FixedFragment()
```

- **Declared at:** [line 127](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L127)
- **Kind:** `constructor`

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragment value();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragment-fixedfragment-prim-n-2"></a>

## `psyqo::Fragments::FixedFragment::FixedFragment<Prim, N>`

**Purpose.** Constructs `psyqo::Fragments::FixedFragment` for the fragments module.

**Exact declaration**

```cpp
explicit FixedFragment(const FixedFragment &) = default
```

- **Declared at:** [line 133](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L133)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const FixedFragment<Prim, N> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Assume these named values have been initialized with valid data:
// const FixedFragment<Prim, N> & arg1

psyqo::Fragments::FixedFragment value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fragments-fixedfragment-getactualfragmentsize-1"></a>

## `psyqo::Fragments::FixedFragment::getActualFragmentSize`

**Purpose.** Returns actual fragment size as part of the fragments module.

**Exact declaration**

```cpp
size_t getActualFragmentSize() const
```

- **Declared at:** [line 135](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L135)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragment& object = /* obtain a valid instance */;

auto result = object.getActualFragmentSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragment-maxsize-1"></a>

## `psyqo::Fragments::FixedFragment::maxSize`

**Purpose.** Performs `max size` as part of the fragments module.

**Exact declaration**

```cpp
constexpr size_t maxSize() const
```

- **Declared at:** [line 126](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L126)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragment& object = /* obtain a valid instance */;

auto result = object.maxSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragmentwithprologue-fixedfragmentwithprologue-p-prim-n-1"></a>

## `psyqo::Fragments::FixedFragmentWithPrologue::FixedFragmentWithPrologue<P, Prim, N>`

**Purpose.** Constructs `psyqo::Fragments::FixedFragmentWithPrologue` for the fragments module.

**Exact declaration**

```cpp
FixedFragmentWithPrologue()
```

- **Declared at:** [line 157](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L157)
- **Kind:** `constructor`

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragmentWithPrologue value();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragmentwithprologue-fixedfragmentwithprologue-p-prim-n-2"></a>

## `psyqo::Fragments::FixedFragmentWithPrologue::FixedFragmentWithPrologue<P, Prim, N>`

**Purpose.** Constructs `psyqo::Fragments::FixedFragmentWithPrologue` for the fragments module.

**Exact declaration**

```cpp
explicit FixedFragmentWithPrologue(const FixedFragmentWithPrologue &) = default
```

- **Declared at:** [line 163](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L163)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const FixedFragmentWithPrologue<P, Prim, N> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Assume these named values have been initialized with valid data:
// const FixedFragmentWithPrologue<P, Prim, N> & arg1

psyqo::Fragments::FixedFragmentWithPrologue value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fragments-fixedfragmentwithprologue-getactualfragmentsize-1"></a>

## `psyqo::Fragments::FixedFragmentWithPrologue::getActualFragmentSize`

**Purpose.** Returns actual fragment size as part of the fragments module.

**Exact declaration**

```cpp
size_t getActualFragmentSize() const
```

- **Declared at:** [line 165](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L165)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragmentWithPrologue& object = /* obtain a valid instance */;

auto result = object.getActualFragmentSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-fixedfragmentwithprologue-maxsize-1"></a>

## `psyqo::Fragments::FixedFragmentWithPrologue::maxSize`

**Purpose.** Performs `max size` as part of the fragments module.

**Exact declaration**

```cpp
constexpr size_t maxSize() const
```

- **Declared at:** [line 156](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L156)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::FixedFragmentWithPrologue& object = /* obtain a valid instance */;

auto result = object.maxSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-simplefragment-getactualfragmentsize-1"></a>

## `psyqo::Fragments::SimpleFragment::getActualFragmentSize`

**Purpose.** Returns actual fragment size as part of the fragments module.

**Exact declaration**

```cpp
constexpr size_t getActualFragmentSize() const
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L109)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::SimpleFragment& object = /* obtain a valid instance */;

auto result = object.getActualFragmentSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-simplefragment-maxsize-1"></a>

## `psyqo::Fragments::SimpleFragment::maxSize`

**Purpose.** Performs `max size` as part of the fragments module.

**Exact declaration**

```cpp
constexpr size_t maxSize() const
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L100)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

psyqo::Fragments::SimpleFragment& object = /* obtain a valid instance */;

auto result = object.maxSize();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fragments-simplefragment-simplefragment-prim-1"></a>

## `psyqo::Fragments::SimpleFragment::SimpleFragment<Prim>`

**Purpose.** Constructs `psyqo::Fragments::SimpleFragment` for the fragments module.

**Exact declaration**

```cpp
explicit SimpleFragment(const SimpleFragment &) = default
```

- **Declared at:** [line 107](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L107)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const SimpleFragment<Prim> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Assume these named values have been initialized with valid data:
// const SimpleFragment<Prim> & arg1

psyqo::Fragments::SimpleFragment value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the fragments module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fragments-simplefragment-simplefragment-prim-2"></a>

## `psyqo::Fragments::SimpleFragment::SimpleFragment<Prim>`

**Purpose.** Performs `simple fragment<prim>` as part of the fragments module.

**Exact declaration**

```cpp
template <typename... Args> SimpleFragment(Args &&...args)
```

- **Declared at:** [line 102](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/fragments.hh#L102)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `args` | `Args &&...` | Consumed or moved input | Value supplied for `args`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragments module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fragments.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Args

// Assume these named values have been initialized with valid data:
// Args &&... args

psyqo::Fragments::SimpleFragment& object = /* obtain a valid instance */;

object.SimpleFragment<Prim><Args>(args);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
