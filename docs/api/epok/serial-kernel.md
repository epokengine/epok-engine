# Epok API: Serial Kernel

> **Header:** `"serial_kernel.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/serial_kernel.hpp)

This module covers PSX kernel ownership, interrupts and low-level services. It documents 5 public callables declared directly in this header.

## Declared types

`epok::SerialKernelState`, `epok::SerialKernelState::Saved`

## Callable index

- [`epok::SerialKernelState::callback`](#epok-serialkernelstate-callback-1) — Performs `callback` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::capture`](#epok-serialkernelstate-capture-1) — Performs `capture` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::ram`](#epok-serialkernelstate-ram-1) — Performs `ram` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::resident`](#epok-serialkernelstate-resident-1) — Performs `resident` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::SerialKernelState::restore`](#epok-serialkernelstate-restore-1) — Performs `restore` as part of PSX kernel ownership, interrupts and low-level services.

<a id="epok-serialkernelstate-callback-1"></a>

## `epok::SerialKernelState::callback`

**Purpose.** Performs `callback` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static bool callback(uint32_t address)
```

- **Declared at:** [line 23](../../../runtime/serial_kernel.hpp#L23)
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

- **Declared at:** [line 29](../../../runtime/serial_kernel.hpp#L29)
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

- **Declared at:** [line 14](../../../runtime/serial_kernel.hpp#L14)
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

- **Declared at:** [line 20](../../../runtime/serial_kernel.hpp#L20)
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

- **Declared at:** [line 52](../../../runtime/serial_kernel.hpp#L52)
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
