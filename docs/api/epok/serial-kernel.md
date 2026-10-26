# Epok API: Serial Kernel

> **Header:** `"serial_kernel.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/serial_kernel.hpp)

This module covers PSX kernel ownership, interrupts and low-level services. It documents 7 public callables declared directly in this header.

## Declared types

`epok::SerialKernelState`, `epok::SerialKernelState::Saved`

## Callable index

- [`epok::serial_recover_dma`](#epok-serial-recover-dma-1) — The resident serial handler can consume a CPU IRQ while an enabled DMA flag remains asserted in DICR.
- [`epok::serial_schedule_dma_poll`](#epok-serial-schedule-dma-poll-1) — PsyQo periodic timers advance one period per pump, even when several periods have elapsed.
- [`epok::SerialKernelState::callback`](#epok-serialkernelstate-callback-1) — Performs `callback` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::capture`](#epok-serialkernelstate-capture-1) — Performs `capture` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::ram`](#epok-serialkernelstate-ram-1) — Performs `ram` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::resident`](#epok-serialkernelstate-resident-1) — Performs `resident` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::restore`](#epok-serialkernelstate-restore-1) — Performs `restore` as part of PSX kernel ownership, interrupts and low-level services.

<a id="epok-serial-recover-dma-1"></a>

## `epok::serial_recover_dma`

**Purpose.** The resident serial handler can consume a CPU IRQ while an enabled DMA flag remains asserted in DICR.

**Details.** Deliver only that real, otherwise stranded event; the normal DMA handler owns acknowledgement and continuation.

**Exact declaration**

```cpp
template<class Memory, class Deliver> bool serial_recover_dma(Memory& memory, Deliver&& deliver)
```

- **Declared at:** [line 19](../../../runtime/serial_kernel.hpp#L19)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `memory` | `Memory &` | Input/output; inspect the function contract | Value supplied for `memory`. See the exact type and module contract. |
| `deliver` | `Deliver &&` | Consumed or moved input | Value supplied for `deliver`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Deliver only that real, otherwise stranded event; the normal DMA handler owns acknowledgement and continuation.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Memory, Deliver

// Assume these named values have been initialized with valid data:
// Memory & memory
// Deliver && deliver

auto result = epok::serial_recover_dma<Memory, Deliver>(memory, deliver);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-serial-schedule-dma-poll-1"></a>

## `epok::serial_schedule_dma_poll`

**Purpose.** PsyQo periodic timers advance one period per pump, even when several periods have elapsed.

**Details.** A 2 ms timer serviced once per video frame eventually trails by more than INT32_MAX and its signed deadline check stops firing. Rearm a single one-shot from the live clock instead. The old timer remains valid until its callback returns; PsyQo then removes it and retains the newly queued timer.

**Exact declaration**

```cpp
template<class GPU, class Poll> void serial_schedule_dma_poll(GPU& gpu, Poll poll)
```

- **Declared at:** [line 10](../../../runtime/serial_kernel.hpp#L10)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `poll` | `Poll` | Input | Value supplied for `poll`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A 2 ms timer serviced once per video frame eventually trails by more than INT32_MAX and its signed deadline check stops firing. Rearm a single one-shot from the live clock instead. The old timer remains valid until its callback returns; PsyQo then removes it and retains the newly queued timer.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// GPU, Poll

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Poll poll

epok::serial_schedule_dma_poll<GPU, Poll>(gpu, poll);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-serialkernelstate-callback-1"></a>

## `epok::SerialKernelState::callback`

**Purpose.** Performs `callback` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static bool callback(uint32_t address)
```

- **Declared at:** [line 45](../../../runtime/serial_kernel.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address

auto result = epok::SerialKernelState::callback(address);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-serialkernelstate-capture-1"></a>

## `epok::SerialKernelState::capture`

**Purpose.** Performs `capture` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Memory> bool capture(Memory& memory)
```

- **Declared at:** [line 51](../../../runtime/serial_kernel.hpp#L51)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `memory` | `Memory &` | Input/output; inspect the function contract | Value supplied for `memory`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Memory

// Assume these named values have been initialized with valid data:
// Memory & memory

epok::SerialKernelState& object = /* obtain a valid instance */;

auto result = object.capture<Memory>(memory);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-serialkernelstate-ram-1"></a>

## `epok::SerialKernelState::ram`

**Purpose.** Performs `ram` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static bool ram(uint32_t address, uint32_t bytes)
```

- **Declared at:** [line 36](../../../runtime/serial_kernel.hpp#L36)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address
// uint32_t bytes

auto result = epok::SerialKernelState::ram(address, bytes);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-serialkernelstate-resident-1"></a>

## `epok::SerialKernelState::resident`

**Purpose.** Performs `resident` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static bool resident(uint32_t address)
```

- **Declared at:** [line 42](../../../runtime/serial_kernel.hpp#L42)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address

auto result = epok::SerialKernelState::resident(address);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-serialkernelstate-restore-1"></a>

## `epok::SerialKernelState::restore`

**Purpose.** Performs `restore` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Memory> void restore(Memory& memory)
```

- **Declared at:** [line 74](../../../runtime/serial_kernel.hpp#L74)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `memory` | `Memory &` | Input/output; inspect the function contract | Value supplied for `memory`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Memory

// Assume these named values have been initialized with valid data:
// Memory & memory

epok::SerialKernelState& object = /* obtain a valid instance */;

object.restore<Memory>(memory);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
