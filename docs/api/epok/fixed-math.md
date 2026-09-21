# Epok API: Fixed Math

> **Header:** `"fixed_math.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/fixed_math.hpp)

This module covers shared Q12 integer math primitives. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::fixed_math::powq`](#epok-fixed-math-powq-1) — `t` raised to `n` in Q12.
- [`epok::fixed_math::q_sqrt`](#epok-fixed-math-q-sqrt-1) — Square root of a Q12 value, in Q12.
- [`epok::fixed_math::sqrt64`](#epok-fixed-math-sqrt64-1) — Truncating integer square root.

<a id="epok-fixed-math-powq-1"></a>

## `epok::fixed_math::powq`

**Purpose.** `t` raised to `n` in Q12.

**Details.** Truncating once per multiply keeps the error inside two raw units up to the fifth power; scaling after the truncation does not.

**Exact declaration**

```cpp
inline int32_t powq(int32_t t,int n)
```

- **Declared at:** [line 12](../../../runtime/fixed_math.hpp#L12)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `int` | Input | Value supplied for `t`. See the exact type and module contract. |
| `n` | `int` | Input | Value supplied for `n`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Truncating once per multiply keeps the error inside two raw units up to the fifth power; scaling after the truncation does not.

**Usage pattern**

```cpp
#include "fixed_math.hpp"

// Assume these named values have been initialized with valid data:
// int t
// int n

auto result = epok::fixed_math::powq(t, n);
```

**Why choose it.** It provides direct, allocation-conscious access to shared Q12 integer math primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fixed-math-q-sqrt-1"></a>

## `epok::fixed_math::q_sqrt`

**Purpose.** Square root of a Q12 value, in Q12.

**Details.** Negatives clamp instead of wrapping.

**Exact declaration**

```cpp
inline int32_t q_sqrt(int32_t raw)
```

- **Declared at:** [line 14](../../../runtime/fixed_math.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw` | `int` | Input | Value supplied for `raw`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Negatives clamp instead of wrapping.

**Usage pattern**

```cpp
#include "fixed_math.hpp"

// Assume these named values have been initialized with valid data:
// int raw

auto result = epok::fixed_math::q_sqrt(raw);
```

**Why choose it.** It provides direct, allocation-conscious access to shared Q12 integer math primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fixed-math-sqrt64-1"></a>

## `epok::fixed_math::sqrt64`

**Purpose.** Truncating integer square root.

**Details.** Restoring, two bits per step, no floats.

**Exact declaration**

```cpp
inline uint32_t sqrt64(uint64_t v)
```

- **Declared at:** [line 9](../../../runtime/fixed_math.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `uint64_t` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Restoring, two bits per step, no floats.

**Usage pattern**

```cpp
#include "fixed_math.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t v

auto result = epok::fixed_math::sqrt64(v);
```

**Why choose it.** It provides direct, allocation-conscious access to shared Q12 integer math primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
