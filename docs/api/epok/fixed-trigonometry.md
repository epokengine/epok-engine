# Epok API: Fixed Trigonometry

> **Header:** `"fixed_trigonometry.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/fixed_trigonometry.hpp)

This module covers fixed-point trigonometric helpers. It documents 4 public callables declared directly in this header.

## Declared types

`epok::fixed_trig_detail::QuarterSine`

## Callable index

- [`epok::cosine_turn12`](#epok-cosine-turn12-1) — Performs `cosine turn12` as part of fixed-point trigonometric helpers.
- [`epok::fixed_trig_detail::make_quarter_sine`](#epok-fixed-trig-detail-make-quarter-sine-1) — Performs `make quarter sine` as part of fixed-point trigonometric helpers.
- [`epok::fixed_trig_detail::sine_series`](#epok-fixed-trig-detail-sine-series-1) — Compile-time generation from the mathematical definition, not game assets.
- [`epok::sine_turn12`](#epok-sine-turn12-1) — One turn = 4096; result is signed Q12.

<a id="epok-cosine-turn12-1"></a>

## `epok::cosine_turn12`

**Purpose.** Performs `cosine turn12` as part of fixed-point trigonometric helpers.

**Exact declaration**

```cpp
inline int32_t cosine_turn12(uint32_t phase)
```

- **Declared at:** [line 34](../../../runtime/fixed_trigonometry.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `phase` | `uint32_t` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fixed_trigonometry.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t phase

auto result = epok::cosine_turn12(phase);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fixed-trig-detail-make-quarter-sine-1"></a>

## `epok::fixed_trig_detail::make_quarter_sine`

**Purpose.** Performs `make quarter sine` as part of fixed-point trigonometric helpers.

**Exact declaration**

```cpp
consteval QuarterSine make_quarter_sine()
```

- **Declared at:** [line 15](../../../runtime/fixed_trigonometry.hpp#L15)
- **Kind:** `function decl`

**Returns.** Returns `QuarterSine`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fixed_trigonometry.hpp"

auto result = epok::fixed_trig_detail::make_quarter_sine();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fixed-trig-detail-sine-series-1"></a>

## `epok::fixed_trig_detail::sine_series`

**Purpose.** Compile-time generation from the mathematical definition, not game assets.

**Details.** The argument is restricted to [0, pi/2]; twelve Taylor terms are comfortably more accurate than a half Q12 tick throughout this interval.

**Exact declaration**

```cpp
constexpr double sine_series(double x)
```

- **Declared at:** [line 9](../../../runtime/fixed_trigonometry.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `double` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `double`. Check the purpose and failure notes before using the value.

**Use it when.** The argument is restricted to [0, pi/2]; twelve Taylor terms are comfortably more accurate than a half Q12 tick throughout this interval.

**Usage pattern**

```cpp
#include "fixed_trigonometry.hpp"

// Assume these named values have been initialized with valid data:
// double x

auto result = epok::fixed_trig_detail::sine_series(x);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sine-turn12-1"></a>

## `epok::sine_turn12`

**Purpose.** One turn = 4096; result is signed Q12.

**Details.** Exact quadrant boundaries and modular arithmetic for any uint32 input. No startup generation or runtime floats.

**Exact declaration**

```cpp
inline int32_t sine_turn12(uint32_t phase)
```

- **Declared at:** [line 27](../../../runtime/fixed_trigonometry.hpp#L27)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `phase` | `uint32_t` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Exact quadrant boundaries and modular arithmetic for any uint32 input. No startup generation or runtime floats.

**Usage pattern**

```cpp
#include "fixed_trigonometry.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t phase

auto result = epok::sine_turn12(phase);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
