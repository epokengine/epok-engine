# Epok API: Streaming Pool

> **Header:** `"streaming_pool.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/streaming_pool.hpp)

This module covers bounded CD or PC geometry-page streaming. It documents 13 public callables declared directly in this header.

## Declared types

`epok::StreamPagePool`

## Callable index

- [`epok::stream_page_hash`](#epok-stream-page-hash-1) — Performs `stream page hash` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::complete`](#epok-streampagepool-complete-1) — Performs `complete` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::destination`](#epok-streampagepool-destination-1) — Performs `destination` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::find`](#epok-streampagepool-find-1) — Finds find as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::has_free_slot`](#epok-streampagepool-has-free-slot-1) — Reports whether free slot as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::pin`](#epok-streampagepool-pin-1) — Performs `pin` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::pin_slot`](#epok-streampagepool-pin-slot-1) — Performs `pin slot` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::ready`](#epok-streampagepool-ready-1) — Reads y as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::reserve`](#epok-streampagepool-reserve-1) — Return the resident slot or reserve an unpinned slot for a read.
- [`epok::StreamPagePool::resident_count`](#epok-streampagepool-resident-count-1) — Performs `resident count` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::unpin`](#epok-streampagepool-unpin-1) — Performs `unpin` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::unpin_slot`](#epok-streampagepool-unpin-slot-1) — Performs `unpin slot` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::valid_range`](#epok-streampagepool-valid-range-1) — Performs `valid range` as part of bounded CD or PC geometry-page streaming.

<a id="epok-stream-page-hash-1"></a>

## `epok::stream_page_hash`

**Purpose.** Performs `stream page hash` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline uint32_t stream_page_hash(const uint8_t *data, size_t bytes = stream_page_bytes)
```

- **Declared at:** [line 9](../../../runtime/streaming_pool.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * data
// size_t bytes

auto result = epok::stream_page_hash(data, bytes);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streampagepool-complete-1"></a>

## `epok::StreamPagePool::complete`

**Purpose.** Performs `complete` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
void complete(int slot, bool success)
```

- **Declared at:** [line 88](../../../runtime/streaming_pool.hpp#L88)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `success` | `bool` | Input | Value supplied for `success`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// int slot
// bool success

epok::StreamPagePool& object = /* obtain a valid instance */;

object.complete(slot, success);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-destination-1"></a>

## `epok::StreamPagePool::destination`

**Purpose.** Performs `destination` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
uint8_t *destination(int slot)
```

- **Declared at:** [line 85](../../../runtime/streaming_pool.hpp#L85)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// int slot

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.destination(slot);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-find-1"></a>

## `epok::StreamPagePool::find`

**Purpose.** Finds find as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
int find(uint32_t page) const
```

- **Declared at:** [line 33](../../../runtime/streaming_pool.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.find(page);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-has-free-slot-1"></a>

## `epok::StreamPagePool::has_free_slot`

**Purpose.** Reports whether free slot as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool has_free_slot() const
```

- **Declared at:** [line 46](../../../runtime/streaming_pool.hpp#L46)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.has_free_slot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-pin-1"></a>

## `epok::StreamPagePool::pin`

**Purpose.** Performs `pin` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
const uint8_t *pin(uint32_t page)
```

- **Declared at:** [line 102](../../../runtime/streaming_pool.hpp#L102)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.pin(page);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-pin-slot-1"></a>

## `epok::StreamPagePool::pin_slot`

**Purpose.** Performs `pin slot` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
int pin_slot(uint32_t page)
```

- **Declared at:** [line 94](../../../runtime/streaming_pool.hpp#L94)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.pin_slot(page);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-ready-1"></a>

## `epok::StreamPagePool::ready`

**Purpose.** Reads y as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool ready(uint32_t page) const
```

- **Declared at:** [line 42](../../../runtime/streaming_pool.hpp#L42)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.ready(page);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-reserve-1"></a>

## `epok::StreamPagePool::reserve`

**Purpose.** Return the resident slot or reserve an unpinned slot for a read.

**Details.** Existing in-flight reads remain owned by their original requester.

**Exact declaration**

```cpp
int reserve(uint32_t page)
```

- **Declared at:** [line 52](../../../runtime/streaming_pool.hpp#L52)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Existing in-flight reads remain owned by their original requester.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.reserve(page);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-resident-count-1"></a>

## `epok::StreamPagePool::resident_count`

**Purpose.** Performs `resident count` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
size_t resident_count() const
```

- **Declared at:** [line 114](../../../runtime/streaming_pool.hpp#L114)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.resident_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-unpin-1"></a>

## `epok::StreamPagePool::unpin`

**Purpose.** Performs `unpin` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool unpin(uint32_t page)
```

- **Declared at:** [line 111](../../../runtime/streaming_pool.hpp#L111)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.unpin(page);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-unpin-slot-1"></a>

## `epok::StreamPagePool::unpin_slot`

**Purpose.** Performs `unpin slot` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool unpin_slot(int slot, uint32_t page)
```

- **Declared at:** [line 105](../../../runtime/streaming_pool.hpp#L105)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// int slot
// uint32_t page

epok::StreamPagePool& object = /* obtain a valid instance */;

auto result = object.unpin_slot(slot, page);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-valid-range-1"></a>

## `epok::StreamPagePool::valid_range`

**Purpose.** Performs `valid range` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
static bool valid_range(uint32_t offset, uint32_t bytes, uint32_t alignment = 4)
```

- **Declared at:** [line 119](../../../runtime/streaming_pool.hpp#L119)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `offset` | `uint32_t` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `alignment` | `uint32_t` | Input | Value supplied for `alignment`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t offset
// uint32_t bytes
// uint32_t alignment

auto result = epok::StreamPagePool::valid_range(offset, bytes, alignment);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
