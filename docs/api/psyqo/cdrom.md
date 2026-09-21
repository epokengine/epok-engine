# PsyQo API: Cdrom

> **Header:** `"psyqo/cdrom.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh)

This module covers the cdrom module. It documents 5 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::CDRom`, `psyqo::CDRom::ReadRequest`

## Callable index

- [`psyqo::CDRom::readSectors`](#psyqo-cdrom-readsectors-1) — Read a sector from the CDRom.
- [`psyqo::CDRom::readSectorsForCoroutine`](#psyqo-cdrom-readsectorsforcoroutine-1) — Wrapper around the readSectors method for coroutines.
- [`psyqo::CDRom::scheduleReadRequest`](#psyqo-cdrom-schedulereadrequest-1) — Schedule a read operation.
- [`psyqo::CDRom::scheduleReadSectors`](#psyqo-cdrom-schedulereadsectors-1) — Schedule a read operation.
- [`psyqo::CDRom::~CDRom`](#psyqo-cdrom-cdrom-1) — Releases the resources owned by `psyqo::CDRom`.

<a id="psyqo-cdrom-readsectors-1"></a>

## `psyqo::CDRom::readSectors`

**Purpose.** Read a sector from the CDRom.

**Details.** The function will make reasonable attempts at reading the disk, but it is not guaranteed to succeed. Failures may be caused by the disk being faulty, the lid being opened, or no valid disk being present. Only one operation can be in progress at a time.

**Exact declaration**

```cpp
virtual void readSectors(uint32_t sector, uint32_t count, void *buffer, eastl::function<void(bool)> &&callback) = 0
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh#L99)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | The sector to read. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | The buffer to read into. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the read is done. It will be called from the main thread when possible. Its one argument is a boolean indicating whether the read was successful. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The function will make reasonable attempts at reading the disk, but it is not guaranteed to succeed. Failures may be caused by the disk being faulty, the lid being opened, or no valid disk being present. Only one operation can be in progress at a time.

**Usage pattern**

```cpp
#include "psyqo/cdrom.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer
// eastl::function<void (bool)> && callback

psyqo::CDRom& object = /* obtain a valid instance */;

object.readSectors(sector, count, buffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdrom-readsectorsforcoroutine-1"></a>

## `psyqo::CDRom::readSectorsForCoroutine`

**Purpose.** Wrapper around the readSectors method for coroutines.

**Details.** This method will return an `Awaiter` object that can be used to suspend the coroutine until the read operation is complete. This is meant to be used in conjunction with the `co_await` keyword, in a coroutine.

**Exact declaration**

```cpp
ReadSectorsAwaiter readSectorsForCoroutine(uint32_t sector, uint32_t count, void *buffer)
```

- **Declared at:** [line 139](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh#L139)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | The sector to read. |
| `count` | `uint32_t` | Input | The number of sectors to read. |
| `buffer` | `void *` | Input/output; inspect the function contract | The buffer to read into. |

**Returns.** ReadSectorsAwaiter The awaitable object to be used with the `co_await` keyword.

**Use it when.** This method will return an `Awaiter` object that can be used to suspend the coroutine until the read operation is complete. This is meant to be used in conjunction with the `co_await` keyword, in a coroutine.

**Usage pattern**

```cpp
#include "psyqo/cdrom.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer

psyqo::CDRom& object = /* obtain a valid instance */;

auto result = object.readSectorsForCoroutine(sector, count, buffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdrom-schedulereadrequest-1"></a>

## `psyqo::CDRom::scheduleReadRequest`

**Purpose.** Schedule a read operation.

**Details.** This is a convenience function that will schedule a read operation and return a task that can be waited on. The difference with `scheduleReadSectors` is that this method will read its arguments right before the operation is processed, so the request data can be filled in at the last moment.

**Exact declaration**

```cpp
TaskQueue::Task scheduleReadRequest(const ReadRequest *request)
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh#L125)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `request` | `const ReadRequest *` | Input | The request to schedule. |

**Returns.** A task that can be queued into a `TaskQueue`

**Use it when.** This is a convenience function that will schedule a read operation and return a task that can be waited on. The difference with `scheduleReadSectors` is that this method will read its arguments right before the operation is processed, so the request data can be filled in at the last moment.

**Usage pattern**

```cpp
#include "psyqo/cdrom.hh"

// Assume these named values have been initialized with valid data:
// const ReadRequest * request

psyqo::CDRom& object = /* obtain a valid instance */;

auto result = object.scheduleReadRequest(request);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdrom-schedulereadsectors-1"></a>

## `psyqo::CDRom::scheduleReadSectors`

**Purpose.** Schedule a read operation.

**Details.** This is a convenience function that will schedule a read operation and return a task that can be waited on.

**Exact declaration**

```cpp
TaskQueue::Task scheduleReadSectors(uint32_t sector, uint32_t count, void *buffer)
```

- **Declared at:** [line 112](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh#L112)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | The sector to read. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | The buffer to read into. |

**Returns.** A task that can be queued into a `TaskQueue`

**Use it when.** This is a convenience function that will schedule a read operation and return a task that can be waited on.

**Usage pattern**

```cpp
#include "psyqo/cdrom.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer

psyqo::CDRom& object = /* obtain a valid instance */;

auto result = object.scheduleReadSectors(sector, count, buffer);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdrom-cdrom-1"></a>

## `psyqo::CDRom::~CDRom`

**Purpose.** Releases the resources owned by `psyqo::CDRom`.

**Exact declaration**

```cpp
virtual ~CDRom()
```

- **Declared at:** [line 70](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom.hh#L70)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need the cdrom module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom.hh"

// `psyqo::CDRom` cleans up when its owning scope ends.
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
