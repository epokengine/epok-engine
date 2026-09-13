# PsyQo API: Cdrom Pcdrv

> **Header:** `"psyqo/cdrom-pcdrv.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-pcdrv.hh)

This module covers the cdrom pcdrv module. It documents 3 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::CDRomPCDrv`

## Callable index

- [`psyqo::CDRomPCDrv::CDRomPCDrv`](#psyqo-cdrompcdrv-cdrompcdrv-1) — instead of eagerly opening in the constructor:
- [`psyqo::CDRomPCDrv::ensureOpen`](#psyqo-cdrompcdrv-ensureopen-1) — Performs `ensure open` as part of the cdrom pcdrv module.
- [`psyqo::CDRomPCDrv::readSectors`](#psyqo-cdrompcdrv-readsectors-1) — Reads sectors as part of the cdrom pcdrv module.

<a id="psyqo-cdrompcdrv-cdrompcdrv-1"></a>

## `psyqo::CDRomPCDrv::CDRomPCDrv`

**Purpose.** instead of eagerly opening in the constructor:

**Exact declaration**

```cpp
CDRomPCDrv(const char* isoName) : m
```

- **Declared at:** [line 37](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-pcdrv.hh#L37)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `isoName` | `const char *` | Input | Value supplied for `isoName`. See the exact type and module contract. |

**Use it when.** You need the cdrom pcdrv module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-pcdrv.hh"

// Assume these named values have been initialized with valid data:
// const char * isoName

psyqo::CDRomPCDrv value(isoName);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdrompcdrv-ensureopen-1"></a>

## `psyqo::CDRomPCDrv::ensureOpen`

**Purpose.** Performs `ensure open` as part of the cdrom pcdrv module.

**Exact declaration**

```cpp
bool ensureOpen()
```

- **Declared at:** [line 39](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-pcdrv.hh#L39)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom pcdrv module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-pcdrv.hh"

psyqo::CDRomPCDrv& object = /* obtain a valid instance */;

auto result = object.ensureOpen();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdrompcdrv-readsectors-1"></a>

## `psyqo::CDRomPCDrv::readSectors`

**Purpose.** Reads sectors as part of the cdrom pcdrv module.

**Exact declaration**

```cpp
void readSectors(uint32_t sector, uint32_t count, void *buffer, eastl::function<void(bool)> &&callback) override
```

- **Declared at:** [line 47](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-pcdrv.hh#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom pcdrv module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-pcdrv.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer
// eastl::function<void (bool)> && callback

psyqo::CDRomPCDrv& object = /* obtain a valid instance */;

object.readSectors(sector, count, buffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
