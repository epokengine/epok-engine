# PsyQo API: Vector

> **Header:** `"psyqo/vector.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh)

This module covers fixed-point vector arithmetic. It documents 25 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Vector`, `psyqo::Vector::EmptyW`, `psyqo::Vector::EmptyZ`

## Callable index

- [`psyqo::Vector::BACKWARD`](#psyqo-vector-backward-1) — Performs `backward` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::DOWN`](#psyqo-vector-down-1) — Performs `down` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::FORWARD`](#psyqo-vector-forward-1) — Performs `forward` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::get`](#psyqo-vector-get-1) — Returns get as part of fixed-point vector arithmetic.
- [`psyqo::Vector::get`](#psyqo-vector-get-2) — Returns get as part of fixed-point vector arithmetic.
- [`psyqo::Vector::LEFT`](#psyqo-vector-left-1) — Performs `left` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::ONE`](#psyqo-vector-one-1) — Performs `one` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator Vertex`](#psyqo-vector-operator-vertex-1) — Performs `operator  vertex` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator*`](#psyqo-vector-operator-1) — Performs `operator *` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator*`](#psyqo-vector-operator-2) — Performs `operator *` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator*=`](#psyqo-vector-operator-3) — Performs `operator *=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator*=`](#psyqo-vector-operator-4) — Performs `operator *=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator+`](#psyqo-vector-operator-5) — Performs `operator +` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator+=`](#psyqo-vector-operator-6) — Performs `operator +=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator-`](#psyqo-vector-operator-7) — Performs `operator -` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator-`](#psyqo-vector-operator-8) — Performs `operator -` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator-=`](#psyqo-vector-operator-9) — Performs `operator -=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator/`](#psyqo-vector-operator-10) — Performs `operator /` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator/`](#psyqo-vector-operator-11) — Performs `operator /` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator/=`](#psyqo-vector-operator-12) — Performs `operator /=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator/=`](#psyqo-vector-operator-13) — Performs `operator /=` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::operator[]`](#psyqo-vector-operator-14) — Performs `operator []` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::RIGHT`](#psyqo-vector-right-1) — Performs `right` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::UP`](#psyqo-vector-up-1) — Performs `up` as part of fixed-point vector arithmetic.
- [`psyqo::Vector::ZERO`](#psyqo-vector-zero-1) — Performs `zero` as part of fixed-point vector arithmetic.

<a id="psyqo-vector-backward-1"></a>

## `psyqo::Vector::BACKWARD`

**Purpose.** Performs `backward` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector BACKWARD() requires(N == 3)
```

- **Declared at:** [line 230](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L230)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::BACKWARD();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-down-1"></a>

## `psyqo::Vector::DOWN`

**Purpose.** Performs `down` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector DOWN() requires(N <= 3)
```

- **Declared at:** [line 194](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L194)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::DOWN();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-forward-1"></a>

## `psyqo::Vector::FORWARD`

**Purpose.** Performs `forward` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector FORWARD() requires(N == 3)
```

- **Declared at:** [line 221](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L221)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::FORWARD();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-get-1"></a>

## `psyqo::Vector::get`

**Purpose.** Returns get as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr FixedPointType& get(unsigned i)
```

- **Declared at:** [line 54](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L54)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `unsigned int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `FixedPointType &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// unsigned int i

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.get(i);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-vector-get-2"></a>

## `psyqo::Vector::get`

**Purpose.** Returns get as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr const FixedPointType& get(unsigned i) const
```

- **Declared at:** [line 63](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L63)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `unsigned int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const FixedPointType &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// unsigned int i

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.get(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-vector-left-1"></a>

## `psyqo::Vector::LEFT`

**Purpose.** Performs `left` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector LEFT() requires(N <= 3)
```

- **Declared at:** [line 203](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L203)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::LEFT();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-one-1"></a>

## `psyqo::Vector::ONE`

**Purpose.** Performs `one` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector ONE() requires(N <= 3)
```

- **Declared at:** [line 176](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L176)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::ONE();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-operator-vertex-1"></a>

## `psyqo::Vector::operator Vertex`

**Purpose.** Performs `operator  vertex` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr operator Vertex() const requires((N == 2) && std::is_signed<T>::value)
```

- **Declared at:** [line 73](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L73)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `Vertex`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator Vertex();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-operator-1"></a>

## `psyqo::Vector::operator*`

**Purpose.** Performs `operator *` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector operator*(const FixedPointType& rhs) const
```

- **Declared at:** [line 137](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L137)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const FixedPointType &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const FixedPointType & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator*(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-2"></a>

## `psyqo::Vector::operator*`

**Purpose.** Performs `operator *` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
template <std::integral U> constexpr Vector operator*(U rhs) const
```

- **Declared at:** [line 152](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L152)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `U` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator*<U>(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-vector-operator-3"></a>

## `psyqo::Vector::operator*=`

**Purpose.** Performs `operator *=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector& operator*=(const FixedPointType& rhs)
```

- **Declared at:** [line 90](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L90)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const FixedPointType &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const FixedPointType & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator*=(rhs);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-4"></a>

## `psyqo::Vector::operator*=`

**Purpose.** Performs `operator *=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
template <std::integral U> constexpr Vector& operator*=(U rhs)
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L103)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `U` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator*=<U>(rhs);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-vector-operator-5"></a>

## `psyqo::Vector::operator+`

**Purpose.** Performs `operator +` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector operator+(const Vector& rhs) const
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L123)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const Vector<N, precisionBits, T> &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const Vector<N, precisionBits, T> & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator+(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-6"></a>

## `psyqo::Vector::operator+=`

**Purpose.** Performs `operator +=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector& operator+=(const Vector& rhs)
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const Vector<N, precisionBits, T> &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const Vector<N, precisionBits, T> & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator+=(rhs);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-7"></a>

## `psyqo::Vector::operator-`

**Purpose.** Performs `operator -` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector operator-() const
```

- **Declared at:** [line 116](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L116)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator-();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-operator-8"></a>

## `psyqo::Vector::operator-`

**Purpose.** Performs `operator -` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector operator-(const Vector& rhs) const
```

- **Declared at:** [line 130](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L130)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const Vector<N, precisionBits, T> &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const Vector<N, precisionBits, T> & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator-(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-9"></a>

## `psyqo::Vector::operator-=`

**Purpose.** Performs `operator -=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector& operator-=(const Vector& rhs)
```

- **Declared at:** [line 84](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L84)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const Vector<N, precisionBits, T> &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const Vector<N, precisionBits, T> & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator-=(rhs);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-10"></a>

## `psyqo::Vector::operator/`

**Purpose.** Performs `operator /` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector operator/(const FixedPointType& rhs) const
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L144)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const FixedPointType &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const FixedPointType & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator/(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-11"></a>

## `psyqo::Vector::operator/`

**Purpose.** Performs `operator /` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
template <std::integral U> constexpr Vector operator/(U rhs) const
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L160)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `U` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator/<U>(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-vector-operator-12"></a>

## `psyqo::Vector::operator/=`

**Purpose.** Performs `operator /=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr Vector& operator/=(const FixedPointType& rhs)
```

- **Declared at:** [line 96](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L96)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const FixedPointType &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// const FixedPointType & rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator/=(rhs);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-vector-operator-13"></a>

## `psyqo::Vector::operator/=`

**Purpose.** Performs `operator /=` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
template <std::integral U> constexpr Vector& operator/=(U rhs)
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L110)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `U` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `Vector<N, precisionBits, T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// U rhs

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator/=<U>(rhs);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-vector-operator-14"></a>

## `psyqo::Vector::operator[]`

**Purpose.** Performs `operator []` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
constexpr FixedPointType& operator[](unsigned i)
```

- **Declared at:** [line 72](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L72)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `unsigned int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `FixedPointType &`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

// Assume these named values have been initialized with valid data:
// unsigned int i

psyqo::Vector& object = /* obtain a valid instance */;

auto result = object.operator[](i);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-vector-right-1"></a>

## `psyqo::Vector::RIGHT`

**Purpose.** Performs `right` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector RIGHT() requires(N <= 3)
```

- **Declared at:** [line 212](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L212)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::RIGHT();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-up-1"></a>

## `psyqo::Vector::UP`

**Purpose.** Performs `up` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector UP() requires(N <= 3)
```

- **Declared at:** [line 185](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L185)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::UP();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-vector-zero-1"></a>

## `psyqo::Vector::ZERO`

**Purpose.** Performs `zero` as part of fixed-point vector arithmetic.

**Exact declaration**

```cpp
static constexpr Vector ZERO() requires(N <= 3)
```

- **Declared at:** [line 167](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/vector.hh#L167)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Vector<N, precisionBits, T>`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-point vector arithmetic and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/vector.hh"

auto result = psyqo::Vector::ZERO();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-point vector arithmetic. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
