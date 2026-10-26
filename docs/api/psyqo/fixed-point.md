# PsyQo API: Fixed Point

> **Header:** `"psyqo/fixed-point.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh)

This module covers the fixed point module. It documents 50 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `bfc3f65010efaaf979e10852e4cf203a2eff1b13`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::FixedPoint`, `psyqo::FixedPoint::Raw`

## Callable index

- [`psyqo::fixed_point_literals::operator""_fp`](#psyqo-fixed-point-literals-operator-fp-1) — User-defined literal for constructing a 20.12 fixed point number.
- [`psyqo::FixedPoint::abs`](#psyqo-fixedpoint-abs-1) — Performs `abs` as part of the fixed point module.
- [`psyqo::FixedPoint::ceil`](#psyqo-fixedpoint-ceil-1) — Returns the ceiling of the fixed point number.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-1) — Constructs a fixed point number from a floating point number.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-2) — Constructs `psyqo::FixedPoint` for the fixed point module.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-3) — Constructs `psyqo::FixedPoint` for the fixed point module.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-4) — Constructs `psyqo::FixedPoint` for the fixed point module.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-5) — Constructs `psyqo::FixedPoint` for the fixed point module.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-6) — Constructs a fixed point number from an integer and a fraction. Specifying a fraction different from 0 when the scale is not a power of 2 is undefined behavior.
- [`psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`](#psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-7) — Construct a new Fixed Point number from a different fixed point number.
- [`psyqo::FixedPoint::floor`](#psyqo-fixedpoint-floor-1) — Returns the floor of the fixed point number.
- [`psyqo::FixedPoint::integer`](#psyqo-fixedpoint-integer-1) — Returns the integer part of the fixed point number.
- [`psyqo::FixedPoint::integer`](#psyqo-fixedpoint-integer-2) — Performs `integer` as part of the fixed point module.
- [`psyqo::FixedPoint::operator!`](#psyqo-fixedpoint-operator-1) — Performs `operator !` as part of the fixed point module.
- [`psyqo::FixedPoint::operator*`](#psyqo-fixedpoint-operator-2) — Performs `operator *` as part of the fixed point module.
- [`psyqo::FixedPoint::operator*`](#psyqo-fixedpoint-operator-3) — Performs `operator *` as part of the fixed point module.
- [`psyqo::FixedPoint::operator*=`](#psyqo-fixedpoint-operator-4) — Performs `operator *=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator*=`](#psyqo-fixedpoint-operator-5) — Performs `operator *=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator+`](#psyqo-fixedpoint-operator-6) — Performs `operator +` as part of the fixed point module.
- [`psyqo::FixedPoint::operator+`](#psyqo-fixedpoint-operator-7) — Performs `operator +` as part of the fixed point module.
- [`psyqo::FixedPoint::operator++`](#psyqo-fixedpoint-operator-8) — Performs `operator ++` as part of the fixed point module.
- [`psyqo::FixedPoint::operator++`](#psyqo-fixedpoint-operator-9) — Performs `operator ++` as part of the fixed point module.
- [`psyqo::FixedPoint::operator+=`](#psyqo-fixedpoint-operator-10) — Performs `operator +=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator+=`](#psyqo-fixedpoint-operator-11) — Performs `operator +=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator-`](#psyqo-fixedpoint-operator-12) — Performs `operator -` as part of the fixed point module.
- [`psyqo::FixedPoint::operator-`](#psyqo-fixedpoint-operator-13) — Performs `operator -` as part of the fixed point module.
- [`psyqo::FixedPoint::operator-`](#psyqo-fixedpoint-operator-14) — Performs `operator -` as part of the fixed point module.
- [`psyqo::FixedPoint::operator--`](#psyqo-fixedpoint-operator-15) — Performs `operator --` as part of the fixed point module.
- [`psyqo::FixedPoint::operator--`](#psyqo-fixedpoint-operator-16) — Performs `operator --` as part of the fixed point module.
- [`psyqo::FixedPoint::operator-=`](#psyqo-fixedpoint-operator-17) — Performs `operator -=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator-=`](#psyqo-fixedpoint-operator-18) — Performs `operator -=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator/`](#psyqo-fixedpoint-operator-19) — Performs `operator /` as part of the fixed point module.
- [`psyqo::FixedPoint::operator/`](#psyqo-fixedpoint-operator-20) — Performs `operator /` as part of the fixed point module.
- [`psyqo::FixedPoint::operator/=`](#psyqo-fixedpoint-operator-21) — Performs `operator /=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator/=`](#psyqo-fixedpoint-operator-22) — Performs `operator /=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator<<`](#psyqo-fixedpoint-operator-23) — Performs `operator <<` as part of the fixed point module.
- [`psyqo::FixedPoint::operator<<=`](#psyqo-fixedpoint-operator-24) — Performs `operator <<=` as part of the fixed point module.
- [`psyqo::FixedPoint::operator<=>`](#psyqo-fixedpoint-operator-25) — Performs `operator <=>` as part of the fixed point module.
- [`psyqo::FixedPoint::operator=`](#psyqo-fixedpoint-operator-26) — Performs `operator =` as part of the fixed point module.
- [`psyqo::FixedPoint::operator>>`](#psyqo-fixedpoint-operator-27) — Performs `operator >>` as part of the fixed point module.
- [`psyqo::FixedPoint::operator>>=`](#psyqo-fixedpoint-operator-28) — Performs `operator >>=` as part of the fixed point module.
- [`psyqo::FixedPoint::print`](#psyqo-fixedpoint-print-1) — Prints out the fixed point number.
- [`psyqo::FixedPoint::raw`](#psyqo-fixedpoint-raw-1) — Performs `raw` as part of the fixed point module.
- [`psyqo::FixedPointInternals::dDiv`](#psyqo-fixedpointinternals-ddiv-1) — Performs `d div` as part of the fixed point module.
- [`psyqo::FixedPointInternals::iDiv`](#psyqo-fixedpointinternals-idiv-1) — Performs `i div` as part of the fixed point module.
- [`psyqo::FixedPointInternals::printInt`](#psyqo-fixedpointinternals-printint-1) — Performs `print int` as part of the fixed point module.
- [`psyqo::operator*`](#psyqo-operator-1) — Performs `operator *` as part of the fixed point module.
- [`psyqo::operator+`](#psyqo-operator-2) — Performs `operator +` as part of the fixed point module.
- [`psyqo::operator-`](#psyqo-operator-3) — Performs `operator -` as part of the fixed point module.
- [`psyqo::operator/`](#psyqo-operator-4) — Performs `operator /` as part of the fixed point module.

<a id="psyqo-fixed-point-literals-operator-fp-1"></a>

## `psyqo::fixed_point_literals::operator""_fp`

**Purpose.** User-defined literal for constructing a 20.12 fixed point number.

**Exact declaration**

```cpp
consteval FixedPoint<> operator""_fp(long double value)
```

- **Declared at:** [line 513](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L513)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `long double` | Input | The value to construct the fixed point number from. |

**Returns.** consteval FixedPoint<> The constructed fixed point number.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// long double value

auto result = psyqo::fixed_point_literals::operator""_fp(value);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-abs-1"></a>

## `psyqo::FixedPoint::abs`

**Purpose.** Performs `abs` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint abs() const
```

- **Declared at:** [line 282](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L282)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.abs();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-ceil-1"></a>

## `psyqo::FixedPoint::ceil`

**Purpose.** Returns the ceiling of the fixed point number.

**Details.** This returns the smallest integer greater than or equal to the fixed point number. For example, ceil of 3.2 is 4, ceil of -3.2 is -3.

**Exact declaration**

```cpp
template <std::integral U = T> constexpr U ceil() const
```

- **Declared at:** [line 250](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L250)
- **Kind:** `function template`; qualifiers: `const, template`

**Returns.** constexpr U The ceiling value of the fixed point number.

**Use it when.** This returns the smallest integer greater than or equal to the fixed point number. For example, ceil of 3.2 is 4, ceil of -3.2 is -3.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.ceil<U>();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-1"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs a fixed point number from a floating point number.

**Details.** Note that this is a `consteval` function, so it can only be used with compile-time constants. This is intentional, as the conversion from floating point to fixed point is basically impossible to do at runtime without using floating point arithmetic, which is not available on the PSX.

**Exact declaration**

```cpp
consteval FixedPoint(long double ld)
```

- **Declared at:** [line 151](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L151)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ld` | `long double` | Input | Value supplied for `ld`. See the exact type and module contract. |

**Use it when.** Note that this is a `consteval` function, so it can only be used with compile-time constants. This is intentional, as the conversion from floating point to fixed point is basically impossible to do at runtime without using floating point arithmetic, which is not available on the PSX.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// long double ld

psyqo::FixedPoint value(ld);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-2"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs `psyqo::FixedPoint` for the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint() : value(0)
```

- **Declared at:** [line 158](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L158)
- **Kind:** `constructor`

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint value();
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-3"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs `psyqo::FixedPoint` for the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint(FixedPoint&&) = default
```

- **Declared at:** [line 160](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L160)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `FixedPoint<precisionBits, T, Scale> &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> && arg1

psyqo::FixedPoint value(arg1);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-4"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs `psyqo::FixedPoint` for the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint(T raw, Raw) : value(raw)
```

- **Declared at:** [line 164](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L164)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw` | `T` | Input | Value supplied for `raw`. See the exact type and module contract. |
| `arg2` | `Raw` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// T raw
// Raw arg2

psyqo::FixedPoint value(raw, arg2);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-5"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs `psyqo::FixedPoint` for the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint(const FixedPoint&) = default
```

- **Declared at:** [line 159](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L159)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const FixedPoint<precisionBits, T, Scale> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// const FixedPoint<precisionBits, T, Scale> & arg1

psyqo::FixedPoint value(arg1);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-6"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Constructs a fixed point number from an integer and a fraction. Specifying a fraction different from 0 when the scale is not a power of 2 is undefined behavior.

**Exact declaration**

```cpp
explicit constexpr FixedPoint(T integer, T fraction) : value(integer * scale + fraction)
```

- **Declared at:** [line 137](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L137)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `integer` | `T` | Input | Value supplied for `integer`. See the exact type and module contract. |
| `fraction` | `T` | Input | Value supplied for `fraction`. See the exact type and module contract. |

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// T integer
// T fraction

psyqo::FixedPoint value(integer, fraction);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-fixedpoint-precisionbits-t-scale-7"></a>

## `psyqo::FixedPoint::FixedPoint<precisionBits, T, Scale>`

**Purpose.** Construct a new Fixed Point number from a different fixed point number.

**Exact declaration**

```cpp
template <unsigned otherPrecisionBits = 12, std::integral U = int32_t> explicit FixedPoint(FixedPoint<otherPrecisionBits, U> other)
```

- **Declared at:** [line 171](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L171)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<otherPrecisionBits, U>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// otherPrecisionBits, U

// Assume these named values have been initialized with valid data:
// FixedPoint<otherPrecisionBits, U> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

object.FixedPoint<precisionBits, T, Scale><otherPrecisionBits, U>(other);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-floor-1"></a>

## `psyqo::FixedPoint::floor`

**Purpose.** Returns the floor of the fixed point number.

**Details.** This returns the largest integer less than or equal to the fixed point number. For example, floor of 3.7 is 3, floor of -3.7 is -4.

**Exact declaration**

```cpp
template <std::integral U = T> constexpr U floor() const
```

- **Declared at:** [line 229](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L229)
- **Kind:** `function template`; qualifiers: `const, template`

**Returns.** constexpr U The floor value of the fixed point number.

**Use it when.** This returns the largest integer less than or equal to the fixed point number. For example, floor of 3.7 is 3, floor of -3.7 is -4.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.floor<U>();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-fixedpoint-integer-1"></a>

## `psyqo::FixedPoint::integer`

**Purpose.** Returns the integer part of the fixed point number.

**Details.** This returns the integer part of the fixed point, rounded to the nearest integer. Note that this is not the same as truncating the fixed point number, as it rounds to the nearest integer, rather than towards zero.

**Exact declaration**

```cpp
template <size_t factor = 1> constexpr T integer() const
```

- **Declared at:** [line 198](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L198)
- **Kind:** `function template`; qualifiers: `const, template`

**Returns.** constexpr T The integer part of the fixed point number.

**Use it when.** This returns the integer part of the fixed point, rounded to the nearest integer. Note that this is not the same as truncating the fixed point number, as it rounds to the nearest integer, rather than towards zero.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// factor

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.integer<factor>();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-fixedpoint-integer-2"></a>

## `psyqo::FixedPoint::integer`

**Purpose.** Performs `integer` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr U integer() const
```

- **Declared at:** [line 208](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L208)
- **Kind:** `function template`; qualifiers: `const, template`

**Returns.** Returns `U`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.integer<U>();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-fixedpoint-operator-1"></a>

## `psyqo::FixedPoint::operator!`

**Purpose.** Performs `operator !` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr bool operator!() const
```

- **Declared at:** [line 468](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L468)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator!();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-fixedpoint-operator-2"></a>

## `psyqo::FixedPoint::operator*`

**Purpose.** Performs `operator *` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator*(FixedPoint other) const
```

- **Declared at:** [line 318](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L318)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator*(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-3"></a>

## `psyqo::FixedPoint::operator*`

**Purpose.** Performs `operator *` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint operator*(U other) const
```

- **Declared at:** [line 328](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L328)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator*<U>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-4"></a>

## `psyqo::FixedPoint::operator*=`

**Purpose.** Performs `operator *=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator*=(FixedPoint other)
```

- **Declared at:** [line 386](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L386)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator*=(other);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-5"></a>

## `psyqo::FixedPoint::operator*=`

**Purpose.** Performs `operator *=` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint& operator*=(U other)
```

- **Declared at:** [line 395](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L395)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator*=<U>(other);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-6"></a>

## `psyqo::FixedPoint::operator+`

**Purpose.** Performs `operator +` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator+(FixedPoint other) const
```

- **Declared at:** [line 292](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L292)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator+(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-7"></a>

## `psyqo::FixedPoint::operator+`

**Purpose.** Performs `operator +` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint operator+(U other) const
```

- **Declared at:** [line 299](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L299)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator+<U>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-8"></a>

## `psyqo::FixedPoint::operator++`

**Purpose.** Performs `operator ++` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator++()
```

- **Declared at:** [line 446](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L446)
- **Kind:** `cxx method`

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator++();
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-9"></a>

## `psyqo::FixedPoint::operator++`

**Purpose.** Performs `operator ++` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator++(int)
```

- **Declared at:** [line 451](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L451)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// int arg1

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator++(arg1);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-10"></a>

## `psyqo::FixedPoint::operator+=`

**Purpose.** Performs `operator +=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator+=(FixedPoint other)
```

- **Declared at:** [line 364](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L364)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator+=(other);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-11"></a>

## `psyqo::FixedPoint::operator+=`

**Purpose.** Performs `operator +=` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint& operator+=(U other)
```

- **Declared at:** [line 370](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L370)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator+=<U>(other);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-12"></a>

## `psyqo::FixedPoint::operator-`

**Purpose.** Performs `operator -` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator-() const
```

- **Declared at:** [line 358](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L358)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator-();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-13"></a>

## `psyqo::FixedPoint::operator-`

**Purpose.** Performs `operator -` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator-(FixedPoint other) const
```

- **Declared at:** [line 305](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L305)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator-(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-14"></a>

## `psyqo::FixedPoint::operator-`

**Purpose.** Performs `operator -` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint operator-(U other) const
```

- **Declared at:** [line 312](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L312)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator-<U>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-15"></a>

## `psyqo::FixedPoint::operator--`

**Purpose.** Performs `operator --` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator--()
```

- **Declared at:** [line 457](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L457)
- **Kind:** `cxx method`

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator--();
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-16"></a>

## `psyqo::FixedPoint::operator--`

**Purpose.** Performs `operator --` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator--(int)
```

- **Declared at:** [line 462](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L462)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// int arg1

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator--(arg1);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-17"></a>

## `psyqo::FixedPoint::operator-=`

**Purpose.** Performs `operator -=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator-=(FixedPoint other)
```

- **Declared at:** [line 375](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L375)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator-=(other);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-18"></a>

## `psyqo::FixedPoint::operator-=`

**Purpose.** Performs `operator -=` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint& operator-=(U other)
```

- **Declared at:** [line 381](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L381)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator-=<U>(other);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-19"></a>

## `psyqo::FixedPoint::operator/`

**Purpose.** Performs `operator /` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator/(FixedPoint other) const
```

- **Declared at:** [line 334](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L334)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator/(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-20"></a>

## `psyqo::FixedPoint::operator/`

**Purpose.** Performs `operator /` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint operator/(U other) const
```

- **Declared at:** [line 352](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L352)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator/<U>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-21"></a>

## `psyqo::FixedPoint::operator/=`

**Purpose.** Performs `operator /=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator/=(FixedPoint other)
```

- **Declared at:** [line 400](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L400)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `FixedPoint<precisionBits, T, Scale>` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<precisionBits, T, Scale> other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator/=(other);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-22"></a>

## `psyqo::FixedPoint::operator/=`

**Purpose.** Performs `operator /=` as part of the fixed point module.

**Exact declaration**

```cpp
template <std::integral U> constexpr FixedPoint& operator/=(U other)
```

- **Declared at:** [line 417](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L417)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `U` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator/=<U>(other);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-23"></a>

## `psyqo::FixedPoint::operator<<`

**Purpose.** Performs `operator <<` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator<<(unsigned shift) const
```

- **Declared at:** [line 424](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L424)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `shift` | `unsigned int` | Input | Value supplied for `shift`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// unsigned int shift

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator<<(shift);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-24"></a>

## `psyqo::FixedPoint::operator<<=`

**Purpose.** Performs `operator <<=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator<<=(unsigned shift)
```

- **Declared at:** [line 436](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L436)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `shift` | `unsigned int` | Input | Value supplied for `shift`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// unsigned int shift

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator<<=(shift);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-25"></a>

## `psyqo::FixedPoint::operator<=>`

**Purpose.** Performs `operator <=>` as part of the fixed point module.

**Exact declaration**

```cpp
auto operator<=>(const FixedPoint& other) const = default
```

- **Declared at:** [line 422](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L422)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const FixedPoint<precisionBits, T, Scale> &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `auto`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// const FixedPoint<precisionBits, T, Scale> & other

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator<=>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fixedpoint-operator-26"></a>

## `psyqo::FixedPoint::operator=`

**Purpose.** Performs `operator =` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator=(const FixedPoint&) = default
```

- **Declared at:** [line 161](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L161)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const FixedPoint<precisionBits, T, Scale> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// const FixedPoint<precisionBits, T, Scale> & arg1

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fixedpoint-operator-27"></a>

## `psyqo::FixedPoint::operator>>`

**Purpose.** Performs `operator >>` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint operator>>(unsigned shift) const
```

- **Declared at:** [line 430](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L430)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `shift` | `unsigned int` | Input | Value supplied for `shift`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// unsigned int shift

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator>>(shift);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-operator-28"></a>

## `psyqo::FixedPoint::operator>>=`

**Purpose.** Performs `operator >>=` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr FixedPoint& operator>>=(unsigned shift)
```

- **Declared at:** [line 441](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L441)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `shift` | `unsigned int` | Input | Value supplied for `shift`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, Scale> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// unsigned int shift

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.operator>>=(shift);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-fixedpoint-print-1"></a>

## `psyqo::FixedPoint::print`

**Purpose.** Prints out the fixed point number.

**Details.** This prints out the fixed point number using the provided function for emitting characters out. Note that the formatting is pretty basic for now, and only supports printing out the number in decimal format, with no padding or precision setting. Maximum displayed precision is 5 decimal places.

**Exact declaration**

```cpp
void print(const eastl::function<void(char)>& charPrinter) const
```

- **Declared at:** [line 271](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L271)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `charPrinter` | `const eastl::function<void (char)> &` | Callback | A function that prints a single character, to be used to print the fixed point number. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This prints out the fixed point number using the provided function for emitting characters out. Note that the formatting is pretty basic for now, and only supports printing out the number in decimal format, with no padding or precision setting. Maximum displayed precision is 5 decimal places.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// const eastl::function<void (char)> & charPrinter

psyqo::FixedPoint& object = /* obtain a valid instance */;

object.print(charPrinter);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fixedpoint-raw-1"></a>

## `psyqo::FixedPoint::raw`

**Purpose.** Performs `raw` as part of the fixed point module.

**Exact declaration**

```cpp
T raw() const
```

- **Declared at:** [line 124](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L124)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

psyqo::FixedPoint& object = /* obtain a valid instance */;

auto result = object.raw();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fixedpointinternals-ddiv-1"></a>

## `psyqo::FixedPointInternals::dDiv`

**Purpose.** Performs `d div` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr int32_t dDiv(int32_t a, int32_t b, unsigned scale)
```

- **Declared at:** [line 75](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L75)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |
| `scale` | `unsigned int` | Input | Value supplied for `scale`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b
// unsigned int scale

auto result = psyqo::FixedPointInternals::dDiv(a, b, scale);
```

**Why choose it.** It provides direct, allocation-conscious access to the fixed point module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fixedpointinternals-idiv-1"></a>

## `psyqo::FixedPointInternals::iDiv`

**Purpose.** Performs `i div` as part of the fixed point module.

**Exact declaration**

```cpp
constexpr uint32_t iDiv(uint64_t rem, uint32_t base, unsigned scale)
```

- **Declared at:** [line 42](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rem` | `uint64_t` | Input | Value supplied for `rem`. See the exact type and module contract. |
| `base` | `uint32_t` | Input | Value supplied for `base`. See the exact type and module contract. |
| `scale` | `unsigned int` | Input | Value supplied for `scale`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// uint64_t rem
// uint32_t base
// unsigned int scale

auto result = psyqo::FixedPointInternals::iDiv(rem, base, scale);
```

**Why choose it.** It provides direct, allocation-conscious access to the fixed point module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fixedpointinternals-printint-1"></a>

## `psyqo::FixedPointInternals::printInt`

**Purpose.** Performs `print int` as part of the fixed point module.

**Exact declaration**

```cpp
void printInt(uint32_t value, const eastl::function<void(char)>&, unsigned scale)
```

- **Declared at:** [line 40](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L40)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `arg2` | `const eastl::function<void (char)> &` | Callback | Value supplied for `arg2`. See the exact type and module contract. |
| `scale` | `unsigned int` | Input | Value supplied for `scale`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Assume these named values have been initialized with valid data:
// uint32_t value
// const eastl::function<void (char)> & arg2
// unsigned int scale

psyqo::FixedPointInternals::printInt(value, arg2, scale);
```

**Why choose it.** It provides direct, allocation-conscious access to the fixed point module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-operator-1"></a>

## `psyqo::operator*`

**Purpose.** Performs `operator *` as part of the fixed point module.

**Exact declaration**

```cpp
template <unsigned precisionBits = 12, std::integral T = int32_t, unsigned scale = 1 <<precisionBits, std::integral U = int32_t> constexpr FixedPoint<precisionBits, T, scale> operator*(U a, FixedPoint<precisionBits, T, scale> b)
```

- **Declared at:** [line 485](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L485)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `U` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `FixedPoint<precisionBits, T, scale>` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// precisionBits, T, scale, U

// Assume these named values have been initialized with valid data:
// U a
// FixedPoint<precisionBits, T, scale> b

auto result = psyqo::operator*<precisionBits, T, scale, U>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-operator-2"></a>

## `psyqo::operator+`

**Purpose.** Performs `operator +` as part of the fixed point module.

**Exact declaration**

```cpp
template <unsigned precisionBits = 12, std::integral T = int32_t, unsigned scale = 1 <<precisionBits, std::integral U = int32_t> constexpr FixedPoint<precisionBits, T, scale> operator+(U a, FixedPoint<precisionBits, T, scale> b)
```

- **Declared at:** [line 473](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L473)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `U` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `FixedPoint<precisionBits, T, scale>` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// precisionBits, T, scale, U

// Assume these named values have been initialized with valid data:
// U a
// FixedPoint<precisionBits, T, scale> b

auto result = psyqo::operator+<precisionBits, T, scale, U>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-operator-3"></a>

## `psyqo::operator-`

**Purpose.** Performs `operator -` as part of the fixed point module.

**Exact declaration**

```cpp
template <unsigned precisionBits = 12, std::integral T = int32_t, unsigned scale = 1 <<precisionBits, std::integral U = int32_t> constexpr FixedPoint<precisionBits, T, scale> operator-(U a, FixedPoint<precisionBits, T, scale> b)
```

- **Declared at:** [line 479](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L479)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `U` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `FixedPoint<precisionBits, T, scale>` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// precisionBits, T, scale, U

// Assume these named values have been initialized with valid data:
// U a
// FixedPoint<precisionBits, T, scale> b

auto result = psyqo::operator-<precisionBits, T, scale, U>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-operator-4"></a>

## `psyqo::operator/`

**Purpose.** Performs `operator /` as part of the fixed point module.

**Exact declaration**

```cpp
template <unsigned precisionBits = 12, std::integral T = int32_t, unsigned scale = 1 <<precisionBits, std::integral U = int32_t> constexpr FixedPoint<precisionBits, T, scale> operator/(U a, FixedPoint<precisionBits, T, scale> b)
```

- **Declared at:** [line 491](https://github.com/epokengine/nugget/blob/bfc3f65010efaaf979e10852e4cf203a2eff1b13/psyqo/fixed-point.hh#L491)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `U` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `FixedPoint<precisionBits, T, scale>` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<precisionBits, T, scale>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fixed point module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/fixed-point.hh"

// Replace these template arguments with types or values accepted by the declaration:
// precisionBits, T, scale, U

// Assume these named values have been initialized with valid data:
// U a
// FixedPoint<precisionBits, T, scale> b

auto result = psyqo::operator/<precisionBits, T, scale, U>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Stay within the documented range and account for quantization before chaining several operations.
