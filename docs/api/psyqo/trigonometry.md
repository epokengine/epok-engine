# PsyQo API: Trigonometry

> **Header:** `"psyqo/trigonometry.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh)

This module covers fixed-point trigonometric helpers. It documents 5 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Trig`

## Callable index

- [`psyqo::Trig::cos`](#psyqo-trig-cos-1) — Calculate the cosine of an angle.
- [`psyqo::Trig::sin`](#psyqo-trig-sin-1) — Calculate the sine of an angle.
- [`psyqo::Trig::Trig<precisionBits>`](#psyqo-trig-trig-precisionbits-1) — Constructs `psyqo::Trig` for fixed-point trigonometric helpers.
- [`psyqo::trig_literals::operator""_pi`](#psyqo-trig-literals-operator-pi-1) — A user-defined literal for angle values.
- [`psyqo::TrigInternals::generateTable`](#psyqo-triginternals-generatetable-1) — Performs `generate table` as part of fixed-point trigonometric helpers.

<a id="psyqo-trig-cos-1"></a>

## `psyqo::Trig::cos`

**Purpose.** Calculate the cosine of an angle.

**Exact declaration**

```cpp
constexpr FixedPoint<precisionBits> cos(Angle a) const
```

- **Declared at:** [line 84](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh#L84)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Angle` | Input | The angle to calculate the cosine of. |

**Returns.** FixedPoint<precisionBits> The cosine of the angle.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/trigonometry.hh"

// Assume these named values have been initialized with valid data:
// Angle a

psyqo::Trig& object = /* obtain a valid instance */;

auto result = object.cos(a);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-trig-sin-1"></a>

## `psyqo::Trig::sin`

**Purpose.** Calculate the sine of an angle.

**Exact declaration**

```cpp
constexpr FixedPoint<precisionBits> sin(Angle a) const
```

- **Declared at:** [line 113](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh#L113)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Angle` | Input | The angle to calculate the sine of. |

**Returns.** FixedPoint<precisionBits> The sine of the angle.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/trigonometry.hh"

// Assume these named values have been initialized with valid data:
// Angle a

psyqo::Trig& object = /* obtain a valid instance */;

auto result = object.sin(a);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-trig-trig-precisionbits-1"></a>

## `psyqo::Trig::Trig<precisionBits>`

**Purpose.** Constructs `psyqo::Trig` for fixed-point trigonometric helpers.

**Exact declaration**

```cpp
Trig()
```

- **Declared at:** [line 76](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh#L76)
- **Kind:** `constructor`

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/trigonometry.hh"

psyqo::Trig value();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-trig-literals-operator-pi-1"></a>

## `psyqo::trig_literals::operator""_pi`

**Purpose.** A user-defined literal for angle values.

**Exact declaration**

```cpp
consteval Angle operator""_pi(long double angle)
```

- **Declared at:** [line 58](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh#L58)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `angle` | `long double` | Input | The angle in fractions of Pi. |

**Returns.** consteval Angle The constructed angle.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/trigonometry.hh"

// Assume these named values have been initialized with valid data:
// long double angle

auto result = psyqo::trig_literals::operator""_pi(angle);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-triginternals-generatetable-1"></a>

## `psyqo::TrigInternals::generateTable`

**Purpose.** Performs `generate table` as part of fixed-point trigonometric helpers.

**Exact declaration**

```cpp
void generateTable(eastl::array<int32_t, 512>& table, unsigned precisionBits)
```

- **Declared at:** [line 38](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/trigonometry.hh#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `eastl::array<int32_t, 512> &` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `precisionBits` | `unsigned int` | Input | Value supplied for `precisionBits`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-point trigonometric helpers and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/trigonometry.hh"

// Assume these named values have been initialized with valid data:
// eastl::array<int32_t, 512> & table
// unsigned int precisionBits

psyqo::TrigInternals::generateTable(table, precisionBits);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point trigonometric helpers. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
