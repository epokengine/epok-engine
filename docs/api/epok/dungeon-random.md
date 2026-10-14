# Epok API: Dungeon Random

> **Header:** `"dungeon_random.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/dungeon_random.hpp)

This module covers the dungeon random module. It documents 6 public callables declared directly in this header.

## Declared types

`epok::dungeon::Random`

## Callable index

- [`epok::dungeon::Random::below`](#epok-dungeon-random-below-1) — Uniform in [0, n) without modulo bias; 0 when n is 0.
- [`epok::dungeon::Random::chance`](#epok-dungeon-random-chance-1) — Performs `chance` as part of the dungeon random module.
- [`epok::dungeon::Random::next`](#epok-dungeon-random-next-1) — Performs `next` as part of the dungeon random module.
- [`epok::dungeon::Random::Random`](#epok-dungeon-random-random-1) — Constructs `epok::dungeon::Random` for the dungeon random module.
- [`epok::dungeon::Random::range`](#epok-dungeon-random-range-1) — Uniform in [low, high]; low when the range is empty.
- [`epok::dungeon::Random::stream`](#epok-dungeon-random-stream-1) — An independent stream for one stage of generation.

<a id="epok-dungeon-random-below-1"></a>

## `epok::dungeon::Random::below`

**Purpose.** Uniform in [0, n) without modulo bias; 0 when n is 0.

**Exact declaration**

```cpp
uint32_t below(uint32_t n)
```

- **Declared at:** [line 27](../../../runtime/dungeon_random.hpp#L27)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `n` | `uint32_t` | Input | Value supplied for `n`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t n

epok::dungeon::Random& object = /* obtain a valid instance */;

auto result = object.below(n);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon random module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-random-chance-1"></a>

## `epok::dungeon::Random::chance`

**Purpose.** Performs `chance` as part of the dungeon random module.

**Exact declaration**

```cpp
bool chance(uint32_t percent)
```

- **Declared at:** [line 41](../../../runtime/dungeon_random.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `percent` | `uint32_t` | Input | Value supplied for `percent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t percent

epok::dungeon::Random& object = /* obtain a valid instance */;

auto result = object.chance(percent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeon-random-next-1"></a>

## `epok::dungeon::Random::next`

**Purpose.** Performs `next` as part of the dungeon random module.

**Exact declaration**

```cpp
uint32_t next()
```

- **Declared at:** [line 20](../../../runtime/dungeon_random.hpp#L20)
- **Kind:** `cxx method`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

epok::dungeon::Random& object = /* obtain a valid instance */;

auto result = object.next();
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon random module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-random-random-1"></a>

## `epok::dungeon::Random::Random`

**Purpose.** Constructs `epok::dungeon::Random` for the dungeon random module.

**Exact declaration**

```cpp
explicit Random(uint32_t seed)
```

- **Declared at:** [line 10](../../../runtime/dungeon_random.hpp#L10)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `seed` | `uint32_t` | Input | Value supplied for `seed`. See the exact type and module contract. |

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t seed

epok::dungeon::Random value(seed);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon random module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-random-range-1"></a>

## `epok::dungeon::Random::range`

**Purpose.** Uniform in [low, high]; low when the range is empty.

**Exact declaration**

```cpp
int32_t range(int32_t low,int32_t high)
```

- **Declared at:** [line 37](../../../runtime/dungeon_random.hpp#L37)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `low` | `int32_t` | Input | Value supplied for `low`. See the exact type and module contract. |
| `high` | `int32_t` | Input | Value supplied for `high`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

// Assume these named values have been initialized with valid data:
// int32_t low
// int32_t high

epok::dungeon::Random& object = /* obtain a valid instance */;

auto result = object.range(low, high);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon random module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-random-stream-1"></a>

## `epok::dungeon::Random::stream`

**Purpose.** An independent stream for one stage of generation.

**Exact declaration**

```cpp
static Random stream(uint32_t seed,uint32_t stage)
```

- **Declared at:** [line 19](../../../runtime/dungeon_random.hpp#L19)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `seed` | `uint32_t` | Input | Value supplied for `seed`. See the exact type and module contract. |
| `stage` | `uint32_t` | Input | Value supplied for `stage`. See the exact type and module contract. |

**Returns.** Returns `Random`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon random module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_random.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t seed
// uint32_t stage

auto result = epok::dungeon::Random::stream(seed, stage);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon random module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
