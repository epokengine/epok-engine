# Epok API: Spu Transfer

> **Header:** `"spu_transfer.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/spu_transfer.hpp)

This module covers SPU RAM, voices, ADSR and sound transfer. It documents 6 public callables declared directly in this header.

## Declared types

`epok::spu::Hardware`

## Callable index

- [`epok::spu::Hardware::cancel`](#epok-spu-hardware-cancel-1) — Performs `cancel` as part of SPU RAM, voices, ADSR and sound transfer.
- [`epok::spu::Hardware::dma_busy`](#epok-spu-hardware-dma-busy-1) — Performs `dma busy` as part of SPU RAM, voices, ADSR and sound transfer.
- [`epok::spu::Hardware::read`](#epok-spu-hardware-read-1) — Reads read as part of SPU RAM, voices, ADSR and sound transfer.
- [`epok::spu::Hardware::start`](#epok-spu-hardware-start-1) — Starts start as part of SPU RAM, voices, ADSR and sound transfer.
- [`epok::spu::Hardware::write`](#epok-spu-hardware-write-1) — Writes write as part of SPU RAM, voices, ADSR and sound transfer.
- [`epok::spu::upload`](#epok-spu-upload-1) — Main-thread DMA only.

<a id="epok-spu-hardware-cancel-1"></a>

## `epok::spu::Hardware::cancel`

**Purpose.** Performs `cancel` as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static void cancel()
```

- **Declared at:** [line 16](../../../runtime/spu_transfer.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

epok::spu::Hardware::cancel();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-spu-hardware-dma-busy-1"></a>

## `epok::spu::Hardware::dma_busy`

**Purpose.** Performs `dma busy` as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static bool dma_busy()
```

- **Declared at:** [line 10](../../../runtime/spu_transfer.hpp#L10)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

auto result = epok::spu::Hardware::dma_busy();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-spu-hardware-read-1"></a>

## `epok::spu::Hardware::read`

**Purpose.** Reads read as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static uint16_t read(uintptr_t address)
```

- **Declared at:** [line 8](../../../runtime/spu_transfer.hpp#L8)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `int` | Input | Value supplied for `address`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

// Assume these named values have been initialized with valid data:
// int address

auto result = epok::spu::Hardware::read(address);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-spu-hardware-start-1"></a>

## `epok::spu::Hardware::start`

**Purpose.** Starts start as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static void start(const void* data,uint32_t bytes)
```

- **Declared at:** [line 11](../../../runtime/spu_transfer.hpp#L11)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `bytes` | `int` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

// Assume these named values have been initialized with valid data:
// const void * data
// int bytes

epok::spu::Hardware::start(data, bytes);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spu-hardware-write-1"></a>

## `epok::spu::Hardware::write`

**Purpose.** Writes write as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static void write(uintptr_t address,uint16_t value)
```

- **Declared at:** [line 9](../../../runtime/spu_transfer.hpp#L9)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `int` | Input | Value supplied for `address`. See the exact type and module contract. |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

// Assume these named values have been initialized with valid data:
// int address
// int value

epok::spu::Hardware::write(address, value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-spu-upload-1"></a>

## `epok::spu::upload`

**Purpose.** Main-thread DMA only.

**Details.** A CPU DMA completion does not certify that the SPU applied a new transfer mode or drained its FIFO. Acknowledge Stop before changing the address, then DMA Write before submitting another block.

**Exact declaration**

```cpp
template<class H=Hardware> bool upload(const void* data,uint32_t address,uint32_t bytes,uint32_t budget=10000000)
```

- **Declared at:** [line 23](../../../runtime/spu_transfer.hpp#L23)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `address` | `int` | Input | Value supplied for `address`. See the exact type and module contract. |
| `bytes` | `int` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `budget` | `int` | Input | Value supplied for `budget`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A CPU DMA completion does not certify that the SPU applied a new transfer mode or drained its FIFO. Acknowledge Stop before changing the address, then DMA Write before submitting another block.

**Usage pattern**

```cpp
#include "spu_transfer.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// H

// Assume these named values have been initialized with valid data:
// const void * data
// int address
// int bytes
// int budget

epok::spu& object = /* obtain a valid instance */;

auto result = object.upload<H>(data, address, bytes, budget);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
