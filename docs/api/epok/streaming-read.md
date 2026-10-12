# Epok API: Streaming Read

> **Header:** `"streaming_read.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/streaming_read.hpp)

This module covers bounded CD or PC geometry-page streaming. It documents 8 public callables declared directly in this header.

## Declared types

`epok::StreamReadAction`, `epok::StreamReadState`

## Callable index

- [`epok::stream_read_committed_sectors`](#epok-stream-read-committed-sectors-1) — A scheduled sector is not necessarily in RAM yet.
- [`epok::StreamReadAction::acknowledge`](#epok-streamreadaction-acknowledge-1) — Performs `acknowledge` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::available_bytes`](#epok-streamreadaction-available-bytes-1) — Performs `available bytes` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::complete`](#epok-streamreadaction-complete-1) — Performs `complete` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::dataReady`](#epok-streamreadaction-dataready-1) — Performs `data ready` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::dma_idle`](#epok-streamreadaction-dma-idle-1) — Performs `dma idle` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::start`](#epok-streamreadaction-start-1) — Starts start as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamReadAction::StreamReadAction`](#epok-streamreadaction-streamreadaction-1) — Constructs `epok::StreamReadAction` for bounded CD or PC geometry-page streaming.

<a id="epok-stream-read-committed-sectors-1"></a>

## `epok::stream_read_committed_sectors`

**Purpose.** A scheduled sector is not necessarily in RAM yet.

**Details.** Recheck the ISR's sequence around CHCR: an interrupt between the two reads makes the older prefix the conservative answer. The previous sector was drained before the next launch.

**Exact declaration**

```cpp
constexpr uint32_t stream_read_committed_sectors(uint32_t before,uint32_t after,bool busy)
```

- **Declared at:** [line 37](../../../runtime/streaming_read.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `before` | `uint32_t` | Input | Value supplied for `before`. See the exact type and module contract. |
| `after` | `uint32_t` | Input | Value supplied for `after`. See the exact type and module contract. |
| `busy` | `bool` | Input | Value supplied for `busy`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Recheck the ISR's sequence around CHCR: an interrupt between the two reads makes the older prefix the conservative answer. The previous sector was drained before the next launch.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t before
// uint32_t after
// bool busy

auto result = epok::stream_read_committed_sectors(before, after, busy);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streamreadaction-acknowledge-1"></a>

## `epok::StreamReadAction::acknowledge`

**Purpose.** Performs `acknowledge` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool acknowledge(const psyqo::CDRomDevice::Response&) override
```

- **Declared at:** [line 92](../../../runtime/streaming_read.hpp#L92)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::StreamReadAction& object = /* obtain a valid instance */;

auto result = object.acknowledge(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streamreadaction-available-bytes-1"></a>

## `epok::StreamReadAction::available_bytes`

**Purpose.** Performs `available bytes` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
uint32_t available_bytes() const
```

- **Declared at:** [line 54](../../../runtime/streaming_read.hpp#L54)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

epok::StreamReadAction& object = /* obtain a valid instance */;

auto result = object.available_bytes();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streamreadaction-complete-1"></a>

## `epok::StreamReadAction::complete`

**Purpose.** Performs `complete` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool complete(const psyqo::CDRomDevice::Response&) override
```

- **Declared at:** [line 103](../../../runtime/streaming_read.hpp#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::StreamReadAction& object = /* obtain a valid instance */;

auto result = object.complete(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streamreadaction-dataready-1"></a>

## `epok::StreamReadAction::dataReady`

**Purpose.** Performs `data ready` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool dataReady(const psyqo::CDRomDevice::Response&) override
```

- **Declared at:** [line 72](../../../runtime/streaming_read.hpp#L72)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::StreamReadAction& object = /* obtain a valid instance */;

auto result = object.dataReady(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streamreadaction-dma-idle-1"></a>

## `epok::StreamReadAction::dma_idle`

**Purpose.** Performs `dma idle` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
static bool dma_idle()
```

- **Declared at:** [line 53](../../../runtime/streaming_read.hpp#L53)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

auto result = epok::StreamReadAction::dma_idle();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streamreadaction-start-1"></a>

## `epok::StreamReadAction::start`

**Purpose.** Starts start as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
void start(psyqo::CDRomDevice* device,uint32_t sector,uint32_t count,void* buffer, eastl::function<void(bool)>&& callback)
```

- **Declared at:** [line 62](../../../runtime/streaming_read.hpp#L62)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `psyqo::CDRomDevice *` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |
| `sector` | `uint32_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::CDRomDevice * device
// uint32_t sector
// uint32_t count
// void * buffer
// eastl::function<void (bool)> && callback

epok::StreamReadAction& object = /* obtain a valid instance */;

object.start(device, sector, count, buffer, callback);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streamreadaction-streamreadaction-1"></a>

## `epok::StreamReadAction::StreamReadAction`

**Purpose.** Constructs `epok::StreamReadAction` for bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamReadAction() :
```

- **Declared at:** [line 52](../../../runtime/streaming_read.hpp#L52)
- **Kind:** `constructor`

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming_read.hpp"

epok::StreamReadAction value();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
