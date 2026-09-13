# PsyQo API: Hardware / Hwregs

> **Header:** `"psyqo/hardware/hwregs.hh"` · **Tier:** PsyQo low-level API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh)

This module covers the hardware/hwregs module. It documents 41 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Hardware::BasicAccess`, `psyqo::Hardware::Register`, `psyqo::Hardware::WriteQueue`

## Callable index

- [`psyqo::Hardware::BasicAccess::access`](#psyqo-hardware-basicaccess-access-1) — Performs `access` as part of the hardware/hwregs module.
- [`psyqo::Hardware::BasicAccess::accessPtr`](#psyqo-hardware-basicaccess-accessptr-1) — Performs `access ptr` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::access`](#psyqo-hardware-register-access-1) — Performs `access` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator type-parameter-0-1`](#psyqo-hardware-register-operator-type-parameter-0-1-1) — Performs `operator  type-parameter-0-1` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator!`](#psyqo-hardware-register-operator-1) — Performs `operator !` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator!=`](#psyqo-hardware-register-operator-2) — Performs `operator !=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator%`](#psyqo-hardware-register-operator-3) — Performs `operator %` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator%=`](#psyqo-hardware-register-operator-4) — Performs `operator %=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator&`](#psyqo-hardware-register-operator-5) — Performs `operator &` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator&&`](#psyqo-hardware-register-operator-6) — Performs `operator &&` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator&=`](#psyqo-hardware-register-operator-7) — Performs `operator &=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator*`](#psyqo-hardware-register-operator-8) — Performs `operator *` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator*=`](#psyqo-hardware-register-operator-9) — Performs `operator *=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator+`](#psyqo-hardware-register-operator-10) — Performs `operator +` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator++`](#psyqo-hardware-register-operator-11) — Performs `operator ++` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator++`](#psyqo-hardware-register-operator-12) — Performs `operator ++` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator+=`](#psyqo-hardware-register-operator-13) — Performs `operator +=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator-`](#psyqo-hardware-register-operator-14) — Performs `operator -` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator--`](#psyqo-hardware-register-operator-15) — Performs `operator --` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator--`](#psyqo-hardware-register-operator-16) — Performs `operator --` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator-=`](#psyqo-hardware-register-operator-17) — Performs `operator -=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator/`](#psyqo-hardware-register-operator-18) — Performs `operator /` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator/=`](#psyqo-hardware-register-operator-19) — Performs `operator /=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator<`](#psyqo-hardware-register-operator-20) — Performs `operator <` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator<<`](#psyqo-hardware-register-operator-21) — Performs `operator <<` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator<<=`](#psyqo-hardware-register-operator-22) — Performs `operator <<=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator<=`](#psyqo-hardware-register-operator-23) — Performs `operator <=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator=`](#psyqo-hardware-register-operator-24) — Performs `operator =` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator==`](#psyqo-hardware-register-operator-25) — Performs `operator ==` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator>`](#psyqo-hardware-register-operator-26) — Performs `operator >` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator>=`](#psyqo-hardware-register-operator-27) — Performs `operator >=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator>>`](#psyqo-hardware-register-operator-28) — Performs `operator >>` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator>>=`](#psyqo-hardware-register-operator-29) — Performs `operator >>=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator[]`](#psyqo-hardware-register-operator-30) — Performs `operator []` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator^`](#psyqo-hardware-register-operator-31) — Performs `operator ^` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator^=`](#psyqo-hardware-register-operator-32) — Performs `operator ^=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator|`](#psyqo-hardware-register-operator-33) — Performs `operator |` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator|=`](#psyqo-hardware-register-operator-34) — Performs `operator |=` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator||`](#psyqo-hardware-register-operator-35) — Performs `operator ||` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::operator~`](#psyqo-hardware-register-operator-36) — Performs `operator ~` as part of the hardware/hwregs module.
- [`psyqo::Hardware::Register::throwAway`](#psyqo-hardware-register-throwaway-1) — Performs `throw away` as part of the hardware/hwregs module.

<a id="psyqo-hardware-basicaccess-access-1"></a>

## `psyqo::Hardware::BasicAccess::access`

**Purpose.** Performs `access` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
static volatile T& access(int index = 0)
```

- **Declared at:** [line 40](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L40)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `volatile T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int index

auto result = psyqo::Hardware::BasicAccess::access(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the hardware/hwregs module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-basicaccess-accessptr-1"></a>

## `psyqo::Hardware::BasicAccess::accessPtr`

**Purpose.** Performs `access ptr` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
static volatile T* accessPtr(int index = 0)
```

- **Declared at:** [line 41](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L41)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `volatile T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int index

auto result = psyqo::Hardware::BasicAccess::accessPtr(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the hardware/hwregs module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-access-1"></a>

## `psyqo::Hardware::Register::access`

**Purpose.** Performs `access` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
volatile T& access(int index = 0) const
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L162)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `volatile T &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int index

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.access(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-type-parameter-0-1-1"></a>

## `psyqo::Hardware::Register::operator type-parameter-0-1`

**Purpose.** Performs `operator  type-parameter-0-1` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
operator T() const
```

- **Declared at:** [line 49](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L49)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator type-parameter-0-1();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-1"></a>

## `psyqo::Hardware::Register::operator!`

**Purpose.** Performs `operator !` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator!() const
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L151)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator!();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-2"></a>

## `psyqo::Hardware::Register::operator!=`

**Purpose.** Performs `operator !=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator!=(T value) const
```

- **Declared at:** [line 155](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L155)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator!=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-3"></a>

## `psyqo::Hardware::Register::operator%`

**Purpose.** Performs `operator %` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator%(T value) const
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L144)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator%(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-4"></a>

## `psyqo::Hardware::Register::operator%=`

**Purpose.** Performs `operator %=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator%=(T value) const
```

- **Declared at:** [line 122](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L122)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator%=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-5"></a>

## `psyqo::Hardware::Register::operator&`

**Purpose.** Performs `operator &` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator&(T value) const
```

- **Declared at:** [line 147](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L147)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator&(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-6"></a>

## `psyqo::Hardware::Register::operator&&`

**Purpose.** Performs `operator &&` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator&&(T value) const
```

- **Declared at:** [line 152](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L152)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator&&(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-7"></a>

## `psyqo::Hardware::Register::operator&=`

**Purpose.** Performs `operator &=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator&=(T value) const
```

- **Declared at:** [line 60](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L60)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator&=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-8"></a>

## `psyqo::Hardware::Register::operator*`

**Purpose.** Performs `operator *` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator*(T value) const
```

- **Declared at:** [line 142](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L142)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator*(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-9"></a>

## `psyqo::Hardware::Register::operator*=`

**Purpose.** Performs `operator *=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator*=(T value) const
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L110)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator*=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-10"></a>

## `psyqo::Hardware::Register::operator+`

**Purpose.** Performs `operator +` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator+(T value) const
```

- **Declared at:** [line 140](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L140)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator+(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-11"></a>

## `psyqo::Hardware::Register::operator++`

**Purpose.** Performs `operator ++` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator++() const
```

- **Declared at:** [line 72](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L72)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator++();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-12"></a>

## `psyqo::Hardware::Register::operator++`

**Purpose.** Performs `operator ++` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator++(int) const
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L78)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int arg1

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator++(arg1);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-13"></a>

## `psyqo::Hardware::Register::operator+=`

**Purpose.** Performs `operator +=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator+=(T value) const
```

- **Declared at:** [line 98](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L98)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator+=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-14"></a>

## `psyqo::Hardware::Register::operator-`

**Purpose.** Performs `operator -` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator-(T value) const
```

- **Declared at:** [line 141](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L141)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator-(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-15"></a>

## `psyqo::Hardware::Register::operator--`

**Purpose.** Performs `operator --` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator--() const
```

- **Declared at:** [line 85](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L85)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator--();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-16"></a>

## `psyqo::Hardware::Register::operator--`

**Purpose.** Performs `operator --` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator--(int) const
```

- **Declared at:** [line 91](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L91)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int arg1

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator--(arg1);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-17"></a>

## `psyqo::Hardware::Register::operator-=`

**Purpose.** Performs `operator -=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator-=(T value) const
```

- **Declared at:** [line 104](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L104)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator-=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-18"></a>

## `psyqo::Hardware::Register::operator/`

**Purpose.** Performs `operator /` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator/(T value) const
```

- **Declared at:** [line 143](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L143)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator/(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-19"></a>

## `psyqo::Hardware::Register::operator/=`

**Purpose.** Performs `operator /=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator/=(T value) const
```

- **Declared at:** [line 116](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L116)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator/=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-20"></a>

## `psyqo::Hardware::Register::operator<`

**Purpose.** Performs `operator <` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator<(T value) const
```

- **Declared at:** [line 156](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L156)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator<(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-21"></a>

## `psyqo::Hardware::Register::operator<<`

**Purpose.** Performs `operator <<` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator<<(T value) const
```

- **Declared at:** [line 145](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L145)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator<<(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-22"></a>

## `psyqo::Hardware::Register::operator<<=`

**Purpose.** Performs `operator <<=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator<<=(T value) const
```

- **Declared at:** [line 128](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L128)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator<<=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-23"></a>

## `psyqo::Hardware::Register::operator<=`

**Purpose.** Performs `operator <=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator<=(T value) const
```

- **Declared at:** [line 158](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L158)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator<=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-24"></a>

## `psyqo::Hardware::Register::operator=`

**Purpose.** Performs `operator =` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator=(T value) const
```

- **Declared at:** [line 50](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L50)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-25"></a>

## `psyqo::Hardware::Register::operator==`

**Purpose.** Performs `operator ==` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator==(T value) const
```

- **Declared at:** [line 154](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L154)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator==(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-26"></a>

## `psyqo::Hardware::Register::operator>`

**Purpose.** Performs `operator >` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator>(T value) const
```

- **Declared at:** [line 157](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L157)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator>(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-27"></a>

## `psyqo::Hardware::Register::operator>=`

**Purpose.** Performs `operator >=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
bool operator>=(T value) const
```

- **Declared at:** [line 159](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator>=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-28"></a>

## `psyqo::Hardware::Register::operator>>`

**Purpose.** Performs `operator >>` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator>>(T value) const
```

- **Declared at:** [line 146](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L146)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator>>(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-29"></a>

## `psyqo::Hardware::Register::operator>>=`

**Purpose.** Performs `operator >>=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator>>=(T value) const
```

- **Declared at:** [line 134](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator>>=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-30"></a>

## `psyqo::Hardware::Register::operator[]`

**Purpose.** Performs `operator []` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator[](int index) const
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L160)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// int index

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator[](index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-31"></a>

## `psyqo::Hardware::Register::operator^`

**Purpose.** Performs `operator ^` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator^(T value) const
```

- **Declared at:** [line 149](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L149)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator^(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-32"></a>

## `psyqo::Hardware::Register::operator^=`

**Purpose.** Performs `operator ^=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator^=(T value) const
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L66)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator^=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-33"></a>

## `psyqo::Hardware::Register::operator|`

**Purpose.** Performs `operator |` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator|(T value) const
```

- **Declared at:** [line 148](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L148)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator|(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-34"></a>

## `psyqo::Hardware::Register::operator|=`

**Purpose.** Performs `operator |=` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator|=(T value) const
```

- **Declared at:** [line 54](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L54)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator|=(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-35"></a>

## `psyqo::Hardware::Register::operator||`

**Purpose.** Performs `operator ||` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator||(T value) const
```

- **Declared at:** [line 153](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L153)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `T` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

// Assume these named values have been initialized with valid data:
// T value

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator||(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-operator-36"></a>

## `psyqo::Hardware::Register::operator~`

**Purpose.** Performs `operator ~` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
T operator~() const
```

- **Declared at:** [line 150](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L150)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

auto result = object.operator~();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-register-throwaway-1"></a>

## `psyqo::Hardware::Register::throwAway`

**Purpose.** Performs `throw away` as part of the hardware/hwregs module.

**Exact declaration**

```cpp
void throwAway() const
```

- **Declared at:** [line 48](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/hwregs.hh#L48)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/hwregs module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/hwregs.hh"

psyqo::Hardware::Register& object = /* obtain a valid instance */;

object.throwAway();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.
