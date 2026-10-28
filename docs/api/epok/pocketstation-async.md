# Epok API: Pocketstation Async

> **Header:** `"pocketstation_async.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/pocketstation_async.hpp)

This module covers the pocketstation async module. It documents 13 public callables declared directly in this header.

## Declared types

`epok::PocketRequest`

## Callable index

- [`epok::PocketRequest::begin`](#epok-pocketrequest-begin-1) — Begins begin as part of the pocketstation async module.
- [`epok::PocketRequest::busy`](#epok-pocketrequest-busy-1) — Performs `busy` as part of the pocketstation async module.
- [`epok::PocketRequest::error`](#epok-pocketrequest-error-1) — Performs `error` as part of the pocketstation async module.
- [`epok::PocketRequest::operator=`](#epok-pocketrequest-operator-1) — Performs `operator =` as part of the pocketstation async module.
- [`epok::PocketRequest::operator=`](#epok-pocketrequest-operator-2) — Performs `operator =` as part of the pocketstation async module.
- [`epok::PocketRequest::PocketRequest`](#epok-pocketrequest-pocketrequest-1) — Constructs `epok::PocketRequest` for the pocketstation async module.
- [`epok::PocketRequest::PocketRequest`](#epok-pocketrequest-pocketrequest-2) — Constructs `epok::PocketRequest` for the pocketstation async module.
- [`epok::PocketRequest::PocketRequest`](#epok-pocketrequest-pocketrequest-3) — Constructs `epok::PocketRequest` for the pocketstation async module.
- [`epok::PocketRequest::poll`](#epok-pocketrequest-poll-1) — Polls poll as part of the pocketstation async module.
- [`epok::PocketRequest::ready`](#epok-pocketrequest-ready-1) — Reads y as part of the pocketstation async module.
- [`epok::PocketRequest::received`](#epok-pocketrequest-received-1) — Performs `received` as part of the pocketstation async module.
- [`epok::PocketRequest::reply`](#epok-pocketrequest-reply-1) — Performs `reply` as part of the pocketstation async module.
- [`epok::PocketRequest::~PocketRequest`](#epok-pocketrequest-pocketrequest-4) — Releases the resources owned by `epok::PocketRequest`.

<a id="epok-pocketrequest-begin-1"></a>

## `epok::PocketRequest::begin`

**Purpose.** Begins begin as part of the pocketstation async module.

**Exact declaration**

```cpp
bool begin(unsigned port,const uint8_t* data,uint16_t length,int16_t length_at=-1)
```

- **Declared at:** [line 58](../../../runtime/pocketstation_async.hpp#L58)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `length` | `uint16_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `length_at` | `int16_t` | Input | Value supplied for `length_at`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// const uint8_t * data
// uint16_t length
// int16_t length_at

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.begin(port, data, length, length_at);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketrequest-busy-1"></a>

## `epok::PocketRequest::busy`

**Purpose.** Performs `busy` as part of the pocketstation async module.

**Exact declaration**

```cpp
bool busy()const
```

- **Declared at:** [line 53](../../../runtime/pocketstation_async.hpp#L53)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.busy();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pocketrequest-error-1"></a>

## `epok::PocketRequest::error`

**Purpose.** Performs `error` as part of the pocketstation async module.

**Exact declaration**

```cpp
PocketError error()const
```

- **Declared at:** [line 55](../../../runtime/pocketstation_async.hpp#L55)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.error();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketrequest-operator-1"></a>

## `epok::PocketRequest::operator=`

**Purpose.** Performs `operator =` as part of the pocketstation async module.

**Exact declaration**

```cpp
PocketRequest& operator=(PocketRequest&& other)
```

- **Declared at:** [line 41](../../../runtime/pocketstation_async.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `PocketRequest &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `PocketRequest &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// PocketRequest && other

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.operator=(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketrequest-operator-2"></a>

## `epok::PocketRequest::operator=`

**Purpose.** Performs `operator =` as part of the pocketstation async module.

**Exact declaration**

```cpp
PocketRequest& operator=(const PocketRequest&)=delete
```

- **Declared at:** [line 39](../../../runtime/pocketstation_async.hpp#L39)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const PocketRequest &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `PocketRequest &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// const PocketRequest & arg1

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketrequest-pocketrequest-1"></a>

## `epok::PocketRequest::PocketRequest`

**Purpose.** Constructs `epok::PocketRequest` for the pocketstation async module.

**Exact declaration**

```cpp
PocketRequest()=default
```

- **Declared at:** [line 36](../../../runtime/pocketstation_async.hpp#L36)
- **Kind:** `constructor`

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest value();
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketrequest-pocketrequest-2"></a>

## `epok::PocketRequest::PocketRequest`

**Purpose.** Constructs `epok::PocketRequest` for the pocketstation async module.

**Exact declaration**

```cpp
PocketRequest(PocketRequest&& other)
```

- **Declared at:** [line 40](../../../runtime/pocketstation_async.hpp#L40)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `PocketRequest &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// PocketRequest && other

epok::PocketRequest value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketrequest-pocketrequest-3"></a>

## `epok::PocketRequest::PocketRequest`

**Purpose.** Constructs `epok::PocketRequest` for the pocketstation async module.

**Exact declaration**

```cpp
PocketRequest(const PocketRequest&)=delete
```

- **Declared at:** [line 38](../../../runtime/pocketstation_async.hpp#L38)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const PocketRequest &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// const PocketRequest & arg1

epok::PocketRequest value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketrequest-poll-1"></a>

## `epok::PocketRequest::poll`

**Purpose.** Polls poll as part of the pocketstation async module.

**Exact declaration**

```cpp
bool poll(uint32_t microseconds)
```

- **Declared at:** [line 84](../../../runtime/pocketstation_async.hpp#L84)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `microseconds` | `uint32_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t microseconds

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.poll(microseconds);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pocketrequest-ready-1"></a>

## `epok::PocketRequest::ready`

**Purpose.** Reads y as part of the pocketstation async module.

**Exact declaration**

```cpp
bool ready()const
```

- **Declared at:** [line 54](../../../runtime/pocketstation_async.hpp#L54)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pocketrequest-received-1"></a>

## `epok::PocketRequest::received`

**Purpose.** Performs `received` as part of the pocketstation async module.

**Exact declaration**

```cpp
uint16_t received()const
```

- **Declared at:** [line 56](../../../runtime/pocketstation_async.hpp#L56)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.received();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketrequest-reply-1"></a>

## `epok::PocketRequest::reply`

**Purpose.** Performs `reply` as part of the pocketstation async module.

**Exact declaration**

```cpp
const uint8_t* reply()const
```

- **Declared at:** [line 57](../../../runtime/pocketstation_async.hpp#L57)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

epok::PocketRequest& object = /* obtain a valid instance */;

auto result = object.reply();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketrequest-pocketrequest-4"></a>

## `epok::PocketRequest::~PocketRequest`

**Purpose.** Releases the resources owned by `epok::PocketRequest`.

**Exact declaration**

```cpp
~PocketRequest()
```

- **Declared at:** [line 37](../../../runtime/pocketstation_async.hpp#L37)
- **Kind:** `destructor`

**Use it when.** You need the pocketstation async module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_async.hpp"

// `epok::PocketRequest` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation async module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
