# PsyQo API: Memory Card

> **Header:** `"psyqo/memory-card.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh)

This module covers asynchronous Memory Card access and files. It documents 23 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::MemoryCard`, `psyqo::MemoryCard::Error`, `psyqo::MemoryCard::Port`, `psyqo::MemoryCard::ReadSectorAwaiter`, `psyqo::MemoryCard::WriteSectorAwaiter`

## Callable index

- [`psyqo::MemoryCard::blockCount`](#psyqo-memorycard-blockcount-1) — Performs `block count` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::blockSize`](#psyqo-memorycard-blocksize-1) — Performs `block size` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::errorMessage`](#psyqo-memorycard-errormessage-1) — Returns a human readable string for an error code.
- [`psyqo::MemoryCard::prepare`](#psyqo-memorycard-prepare-1) — Prepares the SIO0 bus for memory card access.
- [`psyqo::MemoryCard::probeBlocking`](#psyqo-memorycard-probeblocking-1) — Probes the card for presence.
- [`psyqo::MemoryCard::readSector`](#psyqo-memorycard-readsector-1) — Reads sector as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::readSector`](#psyqo-memorycard-readsector-2) — --- Callback variants -------------------------------------------------
- [`psyqo::MemoryCard::ReadSectorAwaiter::await_ready`](#psyqo-memorycard-readsectorawaiter-await-ready-1) — Performs `await ready` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::ReadSectorAwaiter::await_resume`](#psyqo-memorycard-readsectorawaiter-await-resume-1) — Performs `await resume` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::ReadSectorAwaiter::await_suspend`](#psyqo-memorycard-readsectorawaiter-await-suspend-1) — Performs `await suspend` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::ReadSectorAwaiter::ReadSectorAwaiter`](#psyqo-memorycard-readsectorawaiter-readsectorawaiter-1) — Constructs `psyqo::MemoryCard::ReadSectorAwaiter` for asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::readSectorBlocking`](#psyqo-memorycard-readsectorblocking-1) — Reads a single 128-byte sector synchronously.
- [`psyqo::MemoryCard::scheduleReadSector`](#psyqo-memorycard-schedulereadsector-1) — --- TaskQueue schedulers ---------------------------------------------
- [`psyqo::MemoryCard::scheduleWriteSector`](#psyqo-memorycard-schedulewritesector-1) — Performs `schedule write sector` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::sectorCount`](#psyqo-memorycard-sectorcount-1) — Performs `sector count` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::sectorSize`](#psyqo-memorycard-sectorsize-1) — Performs `sector size` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::writeSector`](#psyqo-memorycard-writesector-1) — Writes sector as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::writeSector`](#psyqo-memorycard-writesector-2) — Writes sector as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::WriteSectorAwaiter::await_ready`](#psyqo-memorycard-writesectorawaiter-await-ready-1) — Performs `await ready` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::WriteSectorAwaiter::await_resume`](#psyqo-memorycard-writesectorawaiter-await-resume-1) — Performs `await resume` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::WriteSectorAwaiter::await_suspend`](#psyqo-memorycard-writesectorawaiter-await-suspend-1) — Performs `await suspend` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::WriteSectorAwaiter::WriteSectorAwaiter`](#psyqo-memorycard-writesectorawaiter-writesectorawaiter-1) — Constructs `psyqo::MemoryCard::WriteSectorAwaiter` for asynchronous Memory Card access and files.
- [`psyqo::MemoryCard::writeSectorBlocking`](#psyqo-memorycard-writesectorblocking-1) — Writes a single 128-byte sector synchronously.

<a id="psyqo-memorycard-blockcount-1"></a>

## `psyqo::MemoryCard::blockCount`

**Purpose.** Performs `block count` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
static constexpr uint32_t blockCount()
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L110)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

auto result = psyqo::MemoryCard::blockCount();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-blocksize-1"></a>

## `psyqo::MemoryCard::blockSize`

**Purpose.** Performs `block size` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
static constexpr uint32_t blockSize()
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L109)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

auto result = psyqo::MemoryCard::blockSize();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-errormessage-1"></a>

## `psyqo::MemoryCard::errorMessage`

**Purpose.** Returns a human readable string for an error code.

**Exact declaration**

```cpp
static const char *errorMessage(Error error)
```

- **Declared at:** [line 115](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L115)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `error` | `Error` | Input | Value supplied for `error`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Error error

auto result = psyqo::MemoryCard::errorMessage(error);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-prepare-1"></a>

## `psyqo::MemoryCard::prepare`

**Purpose.** Prepares the SIO0 bus for memory card access.

**Details.** Should be called once from `Application::prepare`. It is safe to also use `AdvancedPad` / `SimplePad` alongside this; they share the bus and access is naturally serialized on the main thread.

**Exact declaration**

```cpp
void prepare()
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L124)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Should be called once from `Application::prepare`. It is safe to also use `AdvancedPad` / `SimplePad` alongside this; they share the bus and access is naturally serialized on the main thread.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

psyqo::MemoryCard& object = /* obtain a valid instance */;

object.prepare();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-probeblocking-1"></a>

## `psyqo::MemoryCard::probeBlocking`

**Purpose.** Probes the card for presence.

**Exact declaration**

```cpp
Error probeBlocking(Port port)
```

- **Declared at:** [line 152](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L152)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Error::OK if a card acknowledges on the given port, Error::NoCard otherwise.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.probeBlocking(port);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-readsector-1"></a>

## `psyqo::MemoryCard::readSector`

**Purpose.** Reads sector as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
ReadSectorAwaiter readSector(Port port, uint16_t sector, void *buffer)
```

- **Declared at:** [line 205](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L205)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |

**Returns.** Returns `ReadSectorAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// void * buffer

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.readSector(port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-readsector-2"></a>

## `psyqo::MemoryCard::readSector`

**Purpose.** --- Callback variants -------------------------------------------------

**Exact declaration**

```cpp
void readSector(Port port, uint16_t sector, void *buffer, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 155](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L155)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// void * buffer
// eastl::function<void (Error)> && callback

psyqo::MemoryCard& object = /* obtain a valid instance */;

object.readSector(port, sector, buffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-readsectorawaiter-await-ready-1"></a>

## `psyqo::MemoryCard::ReadSectorAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 166](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L166)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

psyqo::MemoryCard::ReadSectorAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-readsectorawaiter-await-resume-1"></a>

## `psyqo::MemoryCard::ReadSectorAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error await_resume()
```

- **Declared at:** [line 174](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L174)
- **Kind:** `cxx method`

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

psyqo::MemoryCard::ReadSectorAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-readsectorawaiter-await-suspend-1"></a>

## `psyqo::MemoryCard::ReadSectorAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 168](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L168)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::MemoryCard::ReadSectorAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-readsectorawaiter-readsectorawaiter-1"></a>

## `psyqo::MemoryCard::ReadSectorAwaiter::ReadSectorAwaiter`

**Purpose.** Constructs `psyqo::MemoryCard::ReadSectorAwaiter` for asynchronous Memory Card access and files.

**Exact declaration**

```cpp
ReadSectorAwaiter(MemoryCard &device, Port port, uint16_t sector, void *buffer)
```

- **Declared at:** [line 164](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L164)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `MemoryCard &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// MemoryCard & device
// Port port
// uint16_t sector
// void * buffer

psyqo::MemoryCard::ReadSectorAwaiter value(device, port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-readsectorblocking-1"></a>

## `psyqo::MemoryCard::readSectorBlocking`

**Purpose.** Reads a single 128-byte sector synchronously.

**Exact declaration**

```cpp
Error readSectorBlocking(Port port, uint16_t sector, void *buffer)
```

- **Declared at:** [line 134](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L134)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | The port to read from. |
| `sector` | `uint16_t` | Input | The sector index (0..1023). |
| `buffer` | `void *` | Input/output; inspect the function contract | A buffer of at least 128 bytes. |

**Returns.** Error::OK on success.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// void * buffer

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.readSectorBlocking(port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-schedulereadsector-1"></a>

## `psyqo::MemoryCard::scheduleReadSector`

**Purpose.** --- TaskQueue schedulers ---------------------------------------------

**Exact declaration**

```cpp
TaskQueue::Task scheduleReadSector(Port port, uint16_t sector, void *buffer, Error *resultOut)
```

- **Declared at:** [line 159](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L159)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `resultOut` | `Error *` | Input/output; inspect the function contract | Value supplied for `resultOut`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// void * buffer
// Error * resultOut

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.scheduleReadSector(port, sector, buffer, resultOut);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-schedulewritesector-1"></a>

## `psyqo::MemoryCard::scheduleWriteSector`

**Purpose.** Performs `schedule write sector` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
TaskQueue::Task scheduleWriteSector(Port port, uint16_t sector, const void *buffer, Error *resultOut)
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L160)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `const void *` | Input | Value supplied for `buffer`. See the exact type and module contract. |
| `resultOut` | `Error *` | Input/output; inspect the function contract | Value supplied for `resultOut`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// const void * buffer
// Error * resultOut

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.scheduleWriteSector(port, sector, buffer, resultOut);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-sectorcount-1"></a>

## `psyqo::MemoryCard::sectorCount`

**Purpose.** Performs `sector count` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
static constexpr uint32_t sectorCount()
```

- **Declared at:** [line 108](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L108)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

auto result = psyqo::MemoryCard::sectorCount();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-sectorsize-1"></a>

## `psyqo::MemoryCard::sectorSize`

**Purpose.** Performs `sector size` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
static constexpr uint32_t sectorSize()
```

- **Declared at:** [line 107](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L107)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

auto result = psyqo::MemoryCard::sectorSize();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-writesector-1"></a>

## `psyqo::MemoryCard::writeSector`

**Purpose.** Writes sector as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
WriteSectorAwaiter writeSector(Port port, uint16_t sector, const void *buffer)
```

- **Declared at:** [line 206](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L206)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `const void *` | Input | Value supplied for `buffer`. See the exact type and module contract. |

**Returns.** Returns `WriteSectorAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// const void * buffer

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.writeSector(port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-writesector-2"></a>

## `psyqo::MemoryCard::writeSector`

**Purpose.** Writes sector as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void writeSector(Port port, uint16_t sector, const void *buffer, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 156](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L156)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `const void *` | Input | Value supplied for `buffer`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// const void * buffer
// eastl::function<void (Error)> && callback

psyqo::MemoryCard& object = /* obtain a valid instance */;

object.writeSector(port, sector, buffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-writesectorawaiter-await-ready-1"></a>

## `psyqo::MemoryCard::WriteSectorAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 187](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L187)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

psyqo::MemoryCard::WriteSectorAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-writesectorawaiter-await-resume-1"></a>

## `psyqo::MemoryCard::WriteSectorAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error await_resume()
```

- **Declared at:** [line 195](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L195)
- **Kind:** `cxx method`

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

psyqo::MemoryCard::WriteSectorAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-writesectorawaiter-await-suspend-1"></a>

## `psyqo::MemoryCard::WriteSectorAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 189](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L189)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::MemoryCard::WriteSectorAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycard-writesectorawaiter-writesectorawaiter-1"></a>

## `psyqo::MemoryCard::WriteSectorAwaiter::WriteSectorAwaiter`

**Purpose.** Constructs `psyqo::MemoryCard::WriteSectorAwaiter` for asynchronous Memory Card access and files.

**Exact declaration**

```cpp
WriteSectorAwaiter(MemoryCard &device, Port port, uint16_t sector, const void *buffer)
```

- **Declared at:** [line 185](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L185)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `MemoryCard &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `buffer` | `const void *` | Input | Value supplied for `buffer`. See the exact type and module contract. |

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// MemoryCard & device
// Port port
// uint16_t sector
// const void * buffer

psyqo::MemoryCard::WriteSectorAwaiter value(device, port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycard-writesectorblocking-1"></a>

## `psyqo::MemoryCard::writeSectorBlocking`

**Purpose.** Writes a single 128-byte sector synchronously.

**Exact declaration**

```cpp
Error writeSectorBlocking(Port port, uint16_t sector, const void *buffer)
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card.hh#L144)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | The port to write to. |
| `sector` | `uint16_t` | Input | The sector index (0..1023). |
| `buffer` | `const void *` | Input | A buffer of at least 128 bytes. |

**Returns.** Error::OK on success.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint16_t sector
// const void * buffer

psyqo::MemoryCard& object = /* obtain a valid instance */;

auto result = object.writeSectorBlocking(port, sector, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
