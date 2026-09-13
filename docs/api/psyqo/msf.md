# PsyQo API: Msf

> **Header:** `"psyqo/msf.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh)

This module covers the msf module. It documents 13 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::MSF`, `psyqo::MSF::(anonymous union at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\msf.hh:62:5)::(anonymous struct at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\msf.hh:63:9)`

## Callable index

- [`psyqo::btoi`](#psyqo-btoi-1) — Performs `btoi` as part of the msf module.
- [`psyqo::itob`](#psyqo-itob-1) — Performs `itob` as part of the msf module.
- [`psyqo::MSF::fromBCD`](#psyqo-msf-frombcd-1) — Performs `from bcd` as part of the msf module.
- [`psyqo::MSF::MSF`](#psyqo-msf-msf-1) — Constructs `psyqo::MSF` for the msf module.
- [`psyqo::MSF::MSF`](#psyqo-msf-msf-2) — Constructs `psyqo::MSF` for the msf module.
- [`psyqo::MSF::MSF`](#psyqo-msf-msf-3) — Constructs `psyqo::MSF` for the msf module.
- [`psyqo::MSF::operator++`](#psyqo-msf-operator-1) — Performs `operator ++` as part of the msf module.
- [`psyqo::MSF::operator++`](#psyqo-msf-operator-2) — Performs `operator ++` as part of the msf module.
- [`psyqo::MSF::operator<=>`](#psyqo-msf-operator-3) — Performs `operator <=>` as part of the msf module.
- [`psyqo::MSF::operator==`](#psyqo-msf-operator-4) — Performs `operator ==` as part of the msf module.
- [`psyqo::MSF::reset`](#psyqo-msf-reset-1) — Resets reset as part of the msf module.
- [`psyqo::MSF::toBCD`](#psyqo-msf-tobcd-1) — Performs `to bcd` as part of the msf module.
- [`psyqo::MSF::toLBA`](#psyqo-msf-tolba-1) — Performs `to lba` as part of the msf module.

<a id="psyqo-btoi-1"></a>

## `psyqo::btoi`

**Purpose.** Performs `btoi` as part of the msf module.

**Exact declaration**

```cpp
constexpr uint8_t btoi(uint8_t b)
```

- **Declared at:** [line 35](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `uint8_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// uint8_t b

auto result = psyqo::btoi(b);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-itob-1"></a>

## `psyqo::itob`

**Purpose.** Performs `itob` as part of the msf module.

**Exact declaration**

```cpp
constexpr uint8_t itob(uint8_t i)
```

- **Declared at:** [line 36](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L36)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `uint8_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// uint8_t i

auto result = psyqo::itob(i);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-frombcd-1"></a>

## `psyqo::MSF::fromBCD`

**Purpose.** Performs `from bcd` as part of the msf module.

**Exact declaration**

```cpp
constexpr void fromBCD(const uint8_t *src)
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `src` | `const uint8_t *` | Input | Value supplied for `src`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// const uint8_t * src

psyqo::MSF& object = /* obtain a valid instance */;

object.fromBCD(src);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-msf-msf-1"></a>

## `psyqo::MSF::MSF`

**Purpose.** Constructs `psyqo::MSF` for the msf module.

**Exact declaration**

```cpp
MSF() : m
```

- **Declared at:** [line 39](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L39)
- **Kind:** `constructor`

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

psyqo::MSF value();
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-msf-2"></a>

## `psyqo::MSF::MSF`

**Purpose.** Constructs `psyqo::MSF` for the msf module.

**Exact declaration**

```cpp
MSF(uint8_t m, uint8_t s, uint8_t f) : m
```

- **Declared at:** [line 40](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L40)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `uint8_t` | Input | Value supplied for `m`. See the exact type and module contract. |
| `s` | `uint8_t` | Input | Value supplied for `s`. See the exact type and module contract. |
| `f` | `uint8_t` | Input | Value supplied for `f`. See the exact type and module contract. |

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// uint8_t m
// uint8_t s
// uint8_t f

psyqo::MSF value(m, s, f);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-msf-3"></a>

## `psyqo::MSF::MSF`

**Purpose.** Constructs `psyqo::MSF` for the msf module.

**Exact declaration**

```cpp
explicit MSF(uint32_t lba)
```

- **Declared at:** [line 41](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L41)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lba` | `uint32_t` | Input | Value supplied for `lba`. See the exact type and module contract. |

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// uint32_t lba

psyqo::MSF value(lba);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-operator-1"></a>

## `psyqo::MSF::operator++`

**Purpose.** Performs `operator ++` as part of the msf module.

**Exact declaration**

```cpp
MSF &operator++()
```

- **Declared at:** [line 44](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L44)
- **Kind:** `cxx method`

**Returns.** Returns `MSF &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

psyqo::MSF& object = /* obtain a valid instance */;

auto result = object.operator++();
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-operator-2"></a>

## `psyqo::MSF::operator++`

**Purpose.** Performs `operator ++` as part of the msf module.

**Exact declaration**

```cpp
MSF operator++(int)
```

- **Declared at:** [line 45](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L45)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `MSF`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// int arg1

psyqo::MSF& object = /* obtain a valid instance */;

auto result = object.operator++(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-operator-3"></a>

## `psyqo::MSF::operator<=>`

**Purpose.** Performs `operator <=>` as part of the msf module.

**Exact declaration**

```cpp
auto operator<=>(const MSF &other) const
```

- **Declared at:** [line 42](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L42)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const MSF &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `auto`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// const MSF & other

psyqo::MSF& object = /* obtain a valid instance */;

auto result = object.operator<=>(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-msf-operator-4"></a>

## `psyqo::MSF::operator==`

**Purpose.** Performs `operator ==` as part of the msf module.

**Exact declaration**

```cpp
bool operator==(const MSF &other) const
```

- **Declared at:** [line 43](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L43)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const MSF &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// const MSF & other

psyqo::MSF& object = /* obtain a valid instance */;

auto result = object.operator==(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-msf-reset-1"></a>

## `psyqo::MSF::reset`

**Purpose.** Resets reset as part of the msf module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 61](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L61)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

psyqo::MSF& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the msf module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-msf-tobcd-1"></a>

## `psyqo::MSF::toBCD`

**Purpose.** Performs `to bcd` as part of the msf module.

**Exact declaration**

```cpp
constexpr void toBCD(uint8_t *dst) const
```

- **Declared at:** [line 51](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L51)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dst` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `dst`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

// Assume these named values have been initialized with valid data:
// uint8_t * dst

psyqo::MSF& object = /* obtain a valid instance */;

object.toBCD(dst);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-msf-tolba-1"></a>

## `psyqo::MSF::toLBA`

**Purpose.** Performs `to lba` as part of the msf module.

**Exact declaration**

```cpp
constexpr uint32_t toLBA() const
```

- **Declared at:** [line 50](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/msf.hh#L50)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the msf module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/msf.hh"

psyqo::MSF& object = /* obtain a valid instance */;

auto result = object.toLBA();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
