# Epok API: Rumble

> **Header:** `"rumble.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/rumble.hpp)

This module covers bounded controller motor pulses and cancellation. It documents 11 public callables declared directly in this header.

## Declared types

`epok::RumbleService`, `epok::RumbleService::Output`, `epok::RumbleService::Snapshot`

## Callable index

- [`epok::RumbleService::begin_frame`](#epok-rumbleservice-begin-frame-1) — Begins frame as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::device`](#epok-rumbleservice-device-1) — Performs `device` as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::enabled`](#epok-rumbleservice-enabled-1) — Performs `enabled` as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::output`](#epok-rumbleservice-output-1) — Performs `output` as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::play`](#epok-rumbleservice-play-1) — Starts play as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::set_enabled`](#epok-rumbleservice-set-enabled-1) — Sets enabled as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::snapshot`](#epok-rumbleservice-snapshot-1) — Performs `snapshot` as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::stop`](#epok-rumbleservice-stop-1) — Stops stop as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::stop_all`](#epok-rumbleservice-stop-all-1) — Stops all as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::supported`](#epok-rumbleservice-supported-1) — Performs `supported` as part of bounded controller motor pulses and cancellation.
- [`epok::RumbleService::suspend`](#epok-rumbleservice-suspend-1) — Performs `suspend` as part of bounded controller motor pulses and cancellation.

<a id="epok-rumbleservice-begin-frame-1"></a>

## `epok::RumbleService::begin_frame`

**Purpose.** Begins frame as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void begin_frame(uint32_t now,bool suspended=false)
```

- **Declared at:** [line 48](../../../runtime/rumble.hpp#L48)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |
| `suspended` | `bool` | Input | Value supplied for `suspended`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now
// bool suspended

epok::RumbleService& object = /* obtain a valid instance */;

object.begin_frame(now, suspended);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-device-1"></a>

## `epok::RumbleService::device`

**Purpose.** Performs `device` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void device(unsigned port,bool connected,bool supported,bool probing=false, bool host_enabled=true,uint32_t identity=0)
```

- **Declared at:** [line 58](../../../runtime/rumble.hpp#L58)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `connected` | `bool` | Input | Value supplied for `connected`. See the exact type and module contract. |
| `supported` | `bool` | Input | Value supplied for `supported`. See the exact type and module contract. |
| `probing` | `bool` | Input | Value supplied for `probing`. See the exact type and module contract. |
| `host_enabled` | `bool` | Input | Value supplied for `host_enabled`. See the exact type and module contract. |
| `identity` | `uint32_t` | Input | Value supplied for `identity`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// bool connected
// bool supported
// bool probing
// bool host_enabled
// uint32_t identity

epok::RumbleService& object = /* obtain a valid instance */;

object.device(port, connected, supported, probing, host_enabled, identity);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-enabled-1"></a>

## `epok::RumbleService::enabled`

**Purpose.** Performs `enabled` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
bool enabled(unsigned port=0)const
```

- **Declared at:** [line 77](../../../runtime/rumble.hpp#L77)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

auto result = object.enabled(port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rumbleservice-output-1"></a>

## `epok::RumbleService::output`

**Purpose.** Performs `output` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
Output output(unsigned port=0)const
```

- **Declared at:** [line 79](../../../runtime/rumble.hpp#L79)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `Output`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

auto result = object.output(port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-play-1"></a>

## `epok::RumbleService::play`

**Purpose.** Starts play as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
bool play(uint32_t large,bool small,uint32_t duration_ms,unsigned port=0)
```

- **Declared at:** [line 68](../../../runtime/rumble.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `large` | `uint32_t` | Input | Value supplied for `large`. See the exact type and module contract. |
| `small` | `bool` | Input | Value supplied for `small`. See the exact type and module contract. |
| `duration_ms` | `uint32_t` | Input | Value supplied for `duration_ms`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t large
// bool small
// uint32_t duration_ms
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

auto result = object.play(large, small, duration_ms, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rumbleservice-set-enabled-1"></a>

## `epok::RumbleService::set_enabled`

**Purpose.** Sets enabled as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void set_enabled(bool value,unsigned port=0)
```

- **Declared at:** [line 78](../../../runtime/rumble.hpp#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// bool value
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

object.set_enabled(value, port);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-snapshot-1"></a>

## `epok::RumbleService::snapshot`

**Purpose.** Performs `snapshot` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
Snapshot snapshot(unsigned port=0)const
```

- **Declared at:** [line 88](../../../runtime/rumble.hpp#L88)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `Snapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

auto result = object.snapshot(port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-stop-1"></a>

## `epok::RumbleService::stop`

**Purpose.** Stops stop as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void stop(unsigned port=0)
```

- **Declared at:** [line 41](../../../runtime/rumble.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

object.stop(port);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-stop-all-1"></a>

## `epok::RumbleService::stop_all`

**Purpose.** Stops all as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void stop_all()
```

- **Declared at:** [line 46](../../../runtime/rumble.hpp#L46)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

epok::RumbleService& object = /* obtain a valid instance */;

object.stop_all();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rumbleservice-supported-1"></a>

## `epok::RumbleService::supported`

**Purpose.** Performs `supported` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
bool supported(unsigned port=0)const
```

- **Declared at:** [line 76](../../../runtime/rumble.hpp#L76)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::RumbleService& object = /* obtain a valid instance */;

auto result = object.supported(port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rumbleservice-suspend-1"></a>

## `epok::RumbleService::suspend`

**Purpose.** Performs `suspend` as part of bounded controller motor pulses and cancellation.

**Exact declaration**

```cpp
void suspend(bool value=true)
```

- **Declared at:** [line 47](../../../runtime/rumble.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded controller motor pulses and cancellation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rumble.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::RumbleService& object = /* obtain a valid instance */;

object.suspend(value);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded controller motor pulses and cancellation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
