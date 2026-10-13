# Epok API: Streaming Pool

> **Header:** `"streaming_pool.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/streaming_pool.hpp)

This module covers bounded CD or PC geometry-page streaming. It documents 21 public callables declared directly in this header.

## Declared types

`epok::StreamPagePool`, `epok::StreamPageWorkingSet`

## Callable index

- [`epok::stream_hash_multiply`](#epok-stream-hash-multiply-1) — Performs `stream hash multiply` as part of bounded CD or PC geometry-page streaming.
- [`epok::stream_page_hash`](#epok-stream-page-hash-1) — Performs `stream page hash` as part of bounded CD or PC geometry-page streaming.
- [`epok::stream_page_hash_update`](#epok-stream-page-hash-update-1) — Performs `stream page hash update` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::complete`](#epok-streampagepool-complete-1) — Performs `complete` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::destination`](#epok-streampagepool-destination-1) — Performs `destination` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::find`](#epok-streampagepool-find-1) — Finds find as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::has_free_slot`](#epok-streampagepool-has-free-slot-1) — Reports whether free slot as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::pin`](#epok-streampagepool-pin-1) — Performs `pin` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::pin_slot`](#epok-streampagepool-pin-slot-1) — Performs `pin slot` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::ready`](#epok-streampagepool-ready-1) — Reads y as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::ready_data`](#epok-streampagepool-ready-data-1) — The data of a resident, completed page, or null: one search instead of ready() followed by destination(find()).
- [`epok::StreamPagePool::reserve`](#epok-streampagepool-reserve-1) — Return the resident slot or reserve an unpinned slot for a read.
- [`epok::StreamPagePool::resident_count`](#epok-streampagepool-resident-count-1) — Performs `resident count` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::unpin`](#epok-streampagepool-unpin-1) — Performs `unpin` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::unpin_slot`](#epok-streampagepool-unpin-slot-1) — Performs `unpin slot` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPagePool::valid_range`](#epok-streampagepool-valid-range-1) — Performs `valid range` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageWorkingSet::clear`](#epok-streampageworkingset-clear-1) — Clears clear as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageWorkingSet::consider`](#epok-streampageworkingset-consider-1) — Performs `consider` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageWorkingSet::contains`](#epok-streampageworkingset-contains-1) — Performs `contains` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageWorkingSet::count`](#epok-streampageworkingset-count-1) — Performs `count` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageWorkingSet::page`](#epok-streampageworkingset-page-1) — Performs `page` as part of bounded CD or PC geometry-page streaming.

<a id="epok-stream-hash-multiply-1"></a>

## `epok::stream_hash_multiply`

**Purpose.** Performs `stream hash multiply` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline uint32_t stream_hash_multiply(uint32_t value)
```

- **Declared at:** [line 9](../../../runtime/streaming_pool.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

auto result = epok::stream_hash_multiply(value);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-stream-page-hash-1"></a>

## `epok::stream_page_hash`

**Purpose.** Performs `stream page hash` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline uint32_t stream_page_hash(const uint8_t *data, size_t bytes = stream_page_bytes)
```

- **Declared at:** [line 29](../../../runtime/streaming_pool.hpp#L29)
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

<a id="epok-stream-page-hash-update-1"></a>

## `epok::stream_page_hash_update`

**Purpose.** Performs `stream page hash update` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline uint32_t stream_page_hash_update(uint32_t hash, const uint8_t *data, size_t bytes)
```

- **Declared at:** [line 16](../../../runtime/streaming_pool.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `hash` | `uint32_t` | Input | Value supplied for `hash`. See the exact type and module contract. |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t hash
// const uint8_t * data
// size_t bytes

auto result = epok::stream_page_hash_update(hash, data, bytes);
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

- **Declared at:** [line 156](../../../runtime/streaming_pool.hpp#L156)
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

- **Declared at:** [line 153](../../../runtime/streaming_pool.hpp#L153)
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

- **Declared at:** [line 95](../../../runtime/streaming_pool.hpp#L95)
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

- **Declared at:** [line 114](../../../runtime/streaming_pool.hpp#L114)
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

- **Declared at:** [line 170](../../../runtime/streaming_pool.hpp#L170)
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

- **Declared at:** [line 162](../../../runtime/streaming_pool.hpp#L162)
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

- **Declared at:** [line 104](../../../runtime/streaming_pool.hpp#L104)
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

<a id="epok-streampagepool-ready-data-1"></a>

## `epok::StreamPagePool::ready_data`

**Purpose.** The data of a resident, completed page, or null: one search instead of ready() followed by destination(find()).

**Exact declaration**

```cpp
const uint8_t *ready_data(uint32_t page) const
```

- **Declared at:** [line 110](../../../runtime/streaming_pool.hpp#L110)
- **Kind:** `cxx method`; qualifiers: `const`

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

auto result = object.ready_data(page);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampagepool-reserve-1"></a>

## `epok::StreamPagePool::reserve`

**Purpose.** Return the resident slot or reserve an unpinned slot for a read.

**Details.** Existing in-flight reads remain owned by their original requester.

**Exact declaration**

```cpp
int reserve(uint32_t page)
```

- **Declared at:** [line 120](../../../runtime/streaming_pool.hpp#L120)
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

- **Declared at:** [line 182](../../../runtime/streaming_pool.hpp#L182)
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

- **Declared at:** [line 179](../../../runtime/streaming_pool.hpp#L179)
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

- **Declared at:** [line 173](../../../runtime/streaming_pool.hpp#L173)
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

- **Declared at:** [line 187](../../../runtime/streaming_pool.hpp#L187)
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

<a id="epok-streampageworkingset-clear-1"></a>

## `epok::StreamPageWorkingSet::clear`

**Purpose.** Clears clear as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 56](../../../runtime/streaming_pool.hpp#L56)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

epok::StreamPageWorkingSet& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampageworkingset-consider-1"></a>

## `epok::StreamPageWorkingSet::consider`

**Purpose.** Performs `consider` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
void consider(uint32_t page, uint32_t score)
```

- **Declared at:** [line 63](../../../runtime/streaming_pool.hpp#L63)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `score` | `uint32_t` | Input | Value supplied for `score`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// uint32_t score

epok::StreamPageWorkingSet& object = /* obtain a valid instance */;

object.consider(page, score);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampageworkingset-contains-1"></a>

## `epok::StreamPageWorkingSet::contains`

**Purpose.** Performs `contains` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool contains(uint32_t page) const
```

- **Declared at:** [line 59](../../../runtime/streaming_pool.hpp#L59)
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

epok::StreamPageWorkingSet& object = /* obtain a valid instance */;

auto result = object.contains(page);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampageworkingset-count-1"></a>

## `epok::StreamPageWorkingSet::count`

**Purpose.** Performs `count` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
size_t count() const
```

- **Declared at:** [line 57](../../../runtime/streaming_pool.hpp#L57)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

epok::StreamPageWorkingSet& object = /* obtain a valid instance */;

auto result = object.count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-streampageworkingset-page-1"></a>

## `epok::StreamPageWorkingSet::page`

**Purpose.** Performs `page` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
uint32_t page(size_t i) const
```

- **Declared at:** [line 58](../../../runtime/streaming_pool.hpp#L58)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `size_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_pool.hpp"

// Assume these named values have been initialized with valid data:
// size_t i

epok::StreamPageWorkingSet& object = /* obtain a valid instance */;

auto result = object.page(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
