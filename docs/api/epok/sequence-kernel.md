# Epok API: Sequence Kernel

> **Header:** `"sequence_kernel.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_kernel.hpp)

This module covers PSX kernel ownership, interrupts and low-level services. It documents 11 public callables declared directly in this header.

## Declared types

`epok::sequence::Channel`, `epok::sequence::Error`, `epok::sequence::Event`, `epok::sequence::Kernel`, `epok::sequence::Note`, `epok::sequence::Op`

## Callable index

- [`epok::sequence::Kernel::active`](#epok-sequence-kernel-active-1) — Performs `active` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::advance`](#epok-sequence-kernel-advance-1) — Performs `advance` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::begin`](#epok-sequence-kernel-begin-1) — Begins begin as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::begin_validated`](#epok-sequence-kernel-begin-validated-1) — Only after startup validation of an immutable resident payload.
- [`epok::sequence::Kernel::command`](#epok-sequence-kernel-command-1) — Performs `command` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::cut_all`](#epok-sequence-kernel-cut-all-1) — Performs `cut all` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::fail`](#epok-sequence-kernel-fail-1) — Performs `fail` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::release`](#epok-sequence-kernel-release-1) — Performs `release` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::retire`](#epok-sequence-kernel-retire-1) — Performs `retire` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::stop`](#epok-sequence-kernel-stop-1) — Stops stop as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::update`](#epok-sequence-kernel-update-1) — Updates update as part of PSX kernel ownership, interrupts and low-level services.

<a id="epok-sequence-kernel-active-1"></a>

## `epok::sequence::Kernel::active`

**Purpose.** Performs `active` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
uint32_t active() const
```

- **Declared at:** [line 61](../../../runtime/sequence_kernel.hpp#L61)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

epok::sequence::Kernel& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-advance-1"></a>

## `epok::sequence::Kernel::advance`

**Purpose.** Performs `advance` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void advance(uint32_t microseconds, Backend& backend)
```

- **Declared at:** [line 80](../../../runtime/sequence_kernel.hpp#L80)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `microseconds` | `uint32_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |
| `backend` | `Backend &` | Input/output; inspect the function contract | Value supplied for `backend`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Backend

// Assume these named values have been initialized with valid data:
// uint32_t microseconds
// Backend & backend

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.advance<Backend>(microseconds, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-begin-1"></a>

## `epok::sequence::Kernel::begin`

**Purpose.** Begins begin as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
bool begin(const Event* data, uint32_t size, uint16_t division, uint16_t voices)
```

- **Declared at:** [line 28](../../../runtime/sequence_kernel.hpp#L28)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const Event *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `size` | `uint32_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `division` | `uint16_t` | Input | Value supplied for `division`. See the exact type and module contract. |
| `voices` | `uint16_t` | Input | Value supplied for `voices`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// const Event * data
// uint32_t size
// uint16_t division
// uint16_t voices

epok::sequence::Kernel& object = /* obtain a valid instance */;

auto result = object.begin(data, size, division, voices);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-begin-validated-1"></a>

## `epok::sequence::Kernel::begin_validated`

**Purpose.** Only after startup validation of an immutable resident payload.

**Details.** Avoid rescanning thousands of events in an IRQ. Fixed storage is reset without a large stack temporary.

**Exact declaration**

```cpp
void begin_validated(const Event* data, uint32_t size, uint16_t division, uint16_t voices)
```

- **Declared at:** [line 41](../../../runtime/sequence_kernel.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const Event *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `size` | `uint32_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `division` | `uint16_t` | Input | Value supplied for `division`. See the exact type and module contract. |
| `voices` | `uint16_t` | Input | Value supplied for `voices`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Avoid rescanning thousands of events in an IRQ. Fixed storage is reset without a large stack temporary.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// const Event * data
// uint32_t size
// uint16_t division
// uint16_t voices

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.begin_validated(data, size, division, voices);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-command-1"></a>

## `epok::sequence::Kernel::command`

**Purpose.** Performs `command` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
bool command()
```

- **Declared at:** [line 62](../../../runtime/sequence_kernel.hpp#L62)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

epok::sequence::Kernel& object = /* obtain a valid instance */;

auto result = object.command();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-kernel-cut-all-1"></a>

## `epok::sequence::Kernel::cut_all`

**Purpose.** Performs `cut all` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void cut_all(Backend& backend, bool budgeted = false)
```

- **Declared at:** [line 66](../../../runtime/sequence_kernel.hpp#L66)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `backend` | `Backend &` | Input/output; inspect the function contract | Value supplied for `backend`. See the exact type and module contract. |
| `budgeted` | `bool` | Input | Value supplied for `budgeted`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Backend

// Assume these named values have been initialized with valid data:
// Backend & backend
// bool budgeted

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.cut_all<Backend>(backend, budgeted);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-fail-1"></a>

## `epok::sequence::Kernel::fail`

**Purpose.** Performs `fail` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
bool fail(Error e)
```

- **Declared at:** [line 51](../../../runtime/sequence_kernel.hpp#L51)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `e` | `Error` | Input | Value supplied for `e`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// Error e

epok::sequence::Kernel& object = /* obtain a valid instance */;

auto result = object.fail(e);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-kernel-release-1"></a>

## `epok::sequence::Kernel::release`

**Purpose.** Performs `release` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void release(uint16_t slot, Backend& backend)
```

- **Declared at:** [line 75](../../../runtime/sequence_kernel.hpp#L75)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint16_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `backend` | `Backend &` | Input/output; inspect the function contract | Value supplied for `backend`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Backend

// Assume these named values have been initialized with valid data:
// uint16_t slot
// Backend & backend

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.release<Backend>(slot, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-retire-1"></a>

## `epok::sequence::Kernel::retire`

**Purpose.** Performs `retire` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void retire(uint16_t slot)
```

- **Declared at:** [line 52](../../../runtime/sequence_kernel.hpp#L52)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint16_t` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t slot

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.retire(slot);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-stop-1"></a>

## `epok::sequence::Kernel::stop`

**Purpose.** Stops stop as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void stop(Backend& backend)
```

- **Declared at:** [line 74](../../../runtime/sequence_kernel.hpp#L74)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `backend` | `Backend &` | Input/output; inspect the function contract | Value supplied for `backend`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Backend

// Assume these named values have been initialized with valid data:
// Backend & backend

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.stop<Backend>(backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-update-1"></a>

## `epok::sequence::Kernel::update`

**Purpose.** Updates update as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void update(uint8_t channel, Backend& backend)
```

- **Declared at:** [line 156](../../../runtime/sequence_kernel.hpp#L156)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channel` | `uint8_t` | Input | Value supplied for `channel`. See the exact type and module contract. |
| `backend` | `Backend &` | Input/output; inspect the function contract | Value supplied for `backend`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Backend

// Assume these named values have been initialized with valid data:
// uint8_t channel
// Backend & backend

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.update<Backend>(channel, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
