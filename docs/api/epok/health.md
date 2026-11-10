# Epok API: Health

> **Header:** `"health.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/health.hpp)

This module covers the health module. It documents 5 public callables declared directly in this header.

## Declared types

`epok::HealthChange`, `epok::HealthLibrary`, `epok::HealthResult`, `epok::HealthState`

## Callable index

- [`epok::HealthLibrary::damage`](#epok-healthlibrary-damage-1) — Performs `damage` as part of the health module.
- [`epok::HealthLibrary::heal`](#epok-healthlibrary-heal-1) — Performs `heal` as part of the health module.
- [`epok::HealthLibrary::is_depleted`](#epok-healthlibrary-is-depleted-1) — Reports whether depleted as part of the health module.
- [`epok::HealthLibrary::make_state`](#epok-healthlibrary-make-state-1) — Performs `make state` as part of the health module.
- [`epok::HealthLibrary::valid`](#epok-healthlibrary-valid-1) — Performs `valid` as part of the health module.

<a id="epok-healthlibrary-damage-1"></a>

## `epok::HealthLibrary::damage`

**Purpose.** Performs `damage` as part of the health module.

**Exact declaration**

```cpp
static HealthChange damage(HealthState state,int32_t amount)
```

- **Declared at:** [line 22](../../../runtime/health.hpp#L22)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `HealthState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `amount` | `int32_t` | Input | Value supplied for `amount`. See the exact type and module contract. |

**Returns.** Returns `HealthChange`. Check the purpose and failure notes before using the value.

**Use it when.** You need the health module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "health.hpp"

// Assume these named values have been initialized with valid data:
// HealthState state
// int32_t amount

auto result = epok::HealthLibrary::damage(state, amount);
```

**Why choose it.** It provides direct, allocation-conscious access to the health module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-healthlibrary-heal-1"></a>

## `epok::HealthLibrary::heal`

**Purpose.** Performs `heal` as part of the health module.

**Exact declaration**

```cpp
static HealthChange heal(HealthState state,int32_t amount)
```

- **Declared at:** [line 31](../../../runtime/health.hpp#L31)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `HealthState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `amount` | `int32_t` | Input | Value supplied for `amount`. See the exact type and module contract. |

**Returns.** Returns `HealthChange`. Check the purpose and failure notes before using the value.

**Use it when.** You need the health module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "health.hpp"

// Assume these named values have been initialized with valid data:
// HealthState state
// int32_t amount

auto result = epok::HealthLibrary::heal(state, amount);
```

**Why choose it.** It provides direct, allocation-conscious access to the health module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-healthlibrary-is-depleted-1"></a>

## `epok::HealthLibrary::is_depleted`

**Purpose.** Reports whether depleted as part of the health module.

**Exact declaration**

```cpp
static bool is_depleted(HealthState state)
```

- **Declared at:** [line 21](../../../runtime/health.hpp#L21)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `HealthState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the health module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "health.hpp"

// Assume these named values have been initialized with valid data:
// HealthState state

auto result = epok::HealthLibrary::is_depleted(state);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-healthlibrary-make-state-1"></a>

## `epok::HealthLibrary::make_state`

**Purpose.** Performs `make state` as part of the health module.

**Exact declaration**

```cpp
static HealthState make_state(uint32_t maximum,uint32_t initial)
```

- **Declared at:** [line 20](../../../runtime/health.hpp#L20)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `maximum` | `uint32_t` | Input | Value supplied for `maximum`. See the exact type and module contract. |
| `initial` | `uint32_t` | Input | Value supplied for `initial`. See the exact type and module contract. |

**Returns.** Returns `HealthState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the health module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "health.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t maximum
// uint32_t initial

auto result = epok::HealthLibrary::make_state(maximum, initial);
```

**Why choose it.** It provides direct, allocation-conscious access to the health module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-healthlibrary-valid-1"></a>

## `epok::HealthLibrary::valid`

**Purpose.** Performs `valid` as part of the health module.

**Exact declaration**

```cpp
static bool valid(HealthState state)
```

- **Declared at:** [line 19](../../../runtime/health.hpp#L19)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `HealthState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the health module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "health.hpp"

// Assume these named values have been initialized with valid data:
// HealthState state

auto result = epok::HealthLibrary::valid(state);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
