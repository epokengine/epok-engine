# Epok API: Device Io

> **Header:** `"device_io.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/device_io.hpp)

This module covers the device io module. It documents 13 public callables declared directly in this header.

## Declared types

`epok::PocketTransactionComponent`, `epok::PocketTransferHandle`, `epok::PocketTransferSnapshot`

## Callable index

- [`epok::PocketTransactionComponent::begin`](#epok-pockettransactioncomponent-begin-1) — Begins begin as part of the device io module.
- [`epok::PocketTransactionComponent::begin_play`](#epok-pockettransactioncomponent-begin-play-1) — Begins play as part of the device io module.
- [`epok::PocketTransactionComponent::cancel`](#epok-pockettransactioncomponent-cancel-1) — Performs `cancel` as part of the device io module.
- [`epok::PocketTransactionComponent::class_id`](#epok-pockettransactioncomponent-class-id-1) — Performs `class id` as part of the device io module.
- [`epok::PocketTransactionComponent::clear_request`](#epok-pockettransactioncomponent-clear-request-1) — Clears request as part of the device io module.
- [`epok::PocketTransactionComponent::end_play`](#epok-pockettransactioncomponent-end-play-1) — Ends play as part of the device io module.
- [`epok::PocketTransactionComponent::last_rejection`](#epok-pockettransactioncomponent-last-rejection-1) — Performs `last rejection` as part of the device io module.
- [`epok::PocketTransactionComponent::on_disable`](#epok-pockettransactioncomponent-on-disable-1) — Performs `on disable` as part of the device io module.
- [`epok::PocketTransactionComponent::poll`](#epok-pockettransactioncomponent-poll-1) — Polls poll as part of the device io module.
- [`epok::PocketTransactionComponent::reply_byte`](#epok-pockettransactioncomponent-reply-byte-1) — Performs `reply byte` as part of the device io module.
- [`epok::PocketTransactionComponent::set_request_byte`](#epok-pockettransactioncomponent-set-request-byte-1) — Sets request byte as part of the device io module.
- [`epok::PocketTransactionComponent::snapshot`](#epok-pockettransactioncomponent-snapshot-1) — Performs `snapshot` as part of the device io module.
- [`epok::PocketTransactionComponent::tick`](#epok-pockettransactioncomponent-tick-1) — Performs `tick` as part of the device io module.

<a id="epok-pockettransactioncomponent-begin-1"></a>

## `epok::PocketTransactionComponent::begin`

**Purpose.** Begins begin as part of the device io module.

**Exact declaration**

```cpp
PocketTransferHandle begin(uint32_t length,int32_t length_at)
```

- **Declared at:** [line 45](../../../runtime/device_io.hpp#L45)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `length` | `uint32_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `length_at` | `int32_t` | Input | Value supplied for `length_at`. See the exact type and module contract. |

**Returns.** Returns `PocketTransferHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t length
// int32_t length_at

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.begin(length, length_at);
```

**Why choose it.** It provides direct, allocation-conscious access to the device io module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-begin-play-1"></a>

## `epok::PocketTransactionComponent::begin_play`

**Purpose.** Begins play as part of the device io module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 78](../../../runtime/device_io.hpp#L78)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the device io module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-cancel-1"></a>

## `epok::PocketTransactionComponent::cancel`

**Purpose.** Performs `cancel` as part of the device io module.

**Exact declaration**

```cpp
bool cancel(PocketTransferHandle value)
```

- **Declared at:** [line 68](../../../runtime/device_io.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `PocketTransferHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// PocketTransferHandle value

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.cancel(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pockettransactioncomponent-class-id-1"></a>

## `epok::PocketTransactionComponent::class_id`

**Purpose.** Performs `class id` as part of the device io module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 33](../../../runtime/device_io.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-clear-request-1"></a>

## `epok::PocketTransactionComponent::clear_request`

**Purpose.** Clears request as part of the device io module.

**Exact declaration**

```cpp
bool clear_request()
```

- **Declared at:** [line 37](../../../runtime/device_io.hpp#L37)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.clear_request();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pockettransactioncomponent-end-play-1"></a>

## `epok::PocketTransactionComponent::end_play`

**Purpose.** Ends play as part of the device io module.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 80](../../../runtime/device_io.hpp#L80)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the device io module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-last-rejection-1"></a>

## `epok::PocketTransactionComponent::last_rejection`

**Purpose.** Performs `last rejection` as part of the device io module.

**Exact declaration**

```cpp
PocketError last_rejection()const
```

- **Declared at:** [line 36](../../../runtime/device_io.hpp#L36)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.last_rejection();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-on-disable-1"></a>

## `epok::PocketTransactionComponent::on_disable`

**Purpose.** Performs `on disable` as part of the device io module.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 79](../../../runtime/device_io.hpp#L79)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the device io module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-poll-1"></a>

## `epok::PocketTransactionComponent::poll`

**Purpose.** Polls poll as part of the device io module.

**Exact declaration**

```cpp
bool poll(PocketTransferHandle value,uint32_t microseconds)
```

- **Declared at:** [line 55](../../../runtime/device_io.hpp#L55)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `PocketTransferHandle` | Input | Value supplied for `value`. See the exact type and module contract. |
| `microseconds` | `uint32_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// PocketTransferHandle value
// uint32_t microseconds

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.poll(value, microseconds);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pockettransactioncomponent-reply-byte-1"></a>

## `epok::PocketTransactionComponent::reply_byte`

**Purpose.** Performs `reply byte` as part of the device io module.

**Exact declaration**

```cpp
uint32_t reply_byte(PocketTransferHandle value,uint32_t index)const
```

- **Declared at:** [line 65](../../../runtime/device_io.hpp#L65)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `PocketTransferHandle` | Input | Value supplied for `value`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// PocketTransferHandle value
// uint32_t index

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.reply_byte(value, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-set-request-byte-1"></a>

## `epok::PocketTransactionComponent::set_request_byte`

**Purpose.** Sets request byte as part of the device io module.

**Exact declaration**

```cpp
bool set_request_byte(uint32_t index,uint32_t value)
```

- **Declared at:** [line 41](../../../runtime/device_io.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// uint32_t value

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.set_request_byte(index, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pockettransactioncomponent-snapshot-1"></a>

## `epok::PocketTransactionComponent::snapshot`

**Purpose.** Performs `snapshot` as part of the device io module.

**Exact declaration**

```cpp
PocketTransferSnapshot snapshot(PocketTransferHandle value)const
```

- **Declared at:** [line 61](../../../runtime/device_io.hpp#L61)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `PocketTransferHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `PocketTransferSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// PocketTransferHandle value

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

auto result = object.snapshot(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pockettransactioncomponent-tick-1"></a>

## `epok::PocketTransactionComponent::tick`

**Purpose.** Performs `tick` as part of the device io module.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 72](../../../runtime/device_io.hpp#L72)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the device io module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "device_io.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::PocketTransactionComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the device io module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
