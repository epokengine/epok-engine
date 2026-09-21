# Epok API: Time

> **Header:** `"time.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/time.hpp)

This module covers fixed-step simulation time and frame timing. It documents 8 public callables declared directly in this header.

## Declared types

`epok::Time`

## Callable index

- [`epok::Time::advance`](#epok-time-advance-1) — Performs `advance` as part of fixed-step simulation time and frame timing.
- [`epok::Time::begin_tick`](#epok-time-begin-tick-1) — Begins tick as part of fixed-step simulation time and frame timing.
- [`epok::Time::interpolation_raw`](#epok-time-interpolation-raw-1) — Performs `interpolation raw` as part of fixed-step simulation time and frame timing.
- [`epok::Time::interpolation_thousandths`](#epok-time-interpolation-thousandths-1) — Performs `interpolation thousandths` as part of fixed-step simulation time and frame timing.
- [`epok::Time::paused`](#epok-time-paused-1) — Pauses d as part of fixed-step simulation time and frame timing.
- [`epok::Time::reset`](#epok-time-reset-1) — Resets reset as part of fixed-step simulation time and frame timing.
- [`epok::Time::set_paused`](#epok-time-set-paused-1) — Sets paused as part of fixed-step simulation time and frame timing.
- [`epok::Time::synchronize`](#epok-time-synchronize-1) — Explicit loading time is excluded without erasing gameplay counters or pause state.

<a id="epok-time-advance-1"></a>

## `epok::Time::advance`

**Purpose.** Performs `advance` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
unsigned advance(uint32_t now)
```

- **Declared at:** [line 19](../../../runtime/time.hpp#L19)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** Returns `unsigned int`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::Time& object = /* obtain a valid instance */;

auto result = object.advance(now);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-begin-tick-1"></a>

## `epok::Time::begin_tick`

**Purpose.** Begins tick as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void begin_tick()
```

- **Declared at:** [line 29](../../../runtime/time.hpp#L29)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

epok::Time& object = /* obtain a valid instance */;

object.begin_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-interpolation-raw-1"></a>

## `epok::Time::interpolation_raw`

**Purpose.** Performs `interpolation raw` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
uint32_t interpolation_raw() const
```

- **Declared at:** [line 31](../../../runtime/time.hpp#L31)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

epok::Time& object = /* obtain a valid instance */;

auto result = object.interpolation_raw();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-interpolation-thousandths-1"></a>

## `epok::Time::interpolation_thousandths`

**Purpose.** Performs `interpolation thousandths` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
uint32_t interpolation_thousandths() const
```

- **Declared at:** [line 30](../../../runtime/time.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

epok::Time& object = /* obtain a valid instance */;

auto result = object.interpolation_thousandths();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-paused-1"></a>

## `epok::Time::paused`

**Purpose.** Pauses d as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool paused() const
```

- **Declared at:** [line 14](../../../runtime/time.hpp#L14)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

epok::Time& object = /* obtain a valid instance */;

auto result = object.paused();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-time-reset-1"></a>

## `epok::Time::reset`

**Purpose.** Resets reset as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void reset(uint32_t now)
```

- **Declared at:** [line 16](../../../runtime/time.hpp#L16)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::Time& object = /* obtain a valid instance */;

object.reset(now);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-set-paused-1"></a>

## `epok::Time::set_paused`

**Purpose.** Sets paused as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void set_paused(bool value)
```

- **Declared at:** [line 15](../../../runtime/time.hpp#L15)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Time& object = /* obtain a valid instance */;

object.set_paused(value);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-time-synchronize-1"></a>

## `epok::Time::synchronize`

**Purpose.** Explicit loading time is excluded without erasing gameplay counters or pause state.

**Exact declaration**

```cpp
void synchronize(uint32_t now)
```

- **Declared at:** [line 18](../../../runtime/time.hpp#L18)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "time.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::Time& object = /* obtain a valid instance */;

object.synchronize(now);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
