# Epok API: Sequence Kernel

> **Header:** `"sequence_kernel.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_kernel.hpp)

This module covers PSX kernel ownership, interrupts and low-level services. It documents 23 public callables declared directly in this header.

## Declared types

`epok::sequence::Channel`, `epok::sequence::Error`, `epok::sequence::Event`, `epok::sequence::Kernel`, `epok::sequence::Kernel::KeyLifetime`, `epok::sequence::Note`, `epok::sequence::Op`

## Callable index

- [`epok::sequence::Channel::pitch_cents100`](#epok-sequence-channel-pitch-cents100-1) — Performs `pitch cents100` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::copy_channels`](#epok-sequence-copy-channels-1) — Performs `copy channels` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::active`](#epok-sequence-kernel-active-1) — Performs `active` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::advance`](#epok-sequence-kernel-advance-1) — Performs `advance` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::all_notes_off`](#epok-sequence-kernel-all-notes-off-1) — Performs `all notes off` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::all_sound_off`](#epok-sequence-kernel-all-sound-off-1) — Performs `all sound off` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::begin`](#epok-sequence-kernel-begin-1) — Begins begin as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::begin_validated`](#epok-sequence-kernel-begin-validated-1) — Only after startup validation of an immutable resident payload.
- [`epok::sequence::Kernel::clear_retired`](#epok-sequence-kernel-clear-retired-1) — Clears retired as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::command`](#epok-sequence-kernel-command-1) — Performs `command` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::cut_all`](#epok-sequence-kernel-cut-all-1) — Performs `cut all` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::fail`](#epok-sequence-kernel-fail-1) — Performs `fail` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::first_bit`](#epok-sequence-kernel-first-bit-1) — Performs `first bit` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::release`](#epok-sequence-kernel-release-1) — Performs `release` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::release_if_unheld`](#epok-sequence-kernel-release-if-unheld-1) — Performs `release if unheld` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::release_pedal`](#epok-sequence-kernel-release-pedal-1) — Performs `release pedal` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::reset_free_notes`](#epok-sequence-kernel-reset-free-notes-1) — Resets free notes as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::retire`](#epok-sequence-kernel-retire-1) — Performs `retire` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::stop`](#epok-sequence-kernel-stop-1) — Stops stop as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::unlink_key_note`](#epok-sequence-kernel-unlink-key-note-1) — Performs `unlink key note` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Kernel::update`](#epok-sequence-kernel-update-1) — Updates update as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::Note::initialize`](#epok-sequence-note-initialize-1) — Performs `initialize` as part of PSX kernel ownership, interrupts and low-level services.
- [`epok::sequence::valid_event`](#epok-sequence-valid-event-1) — Performs `valid event` as part of PSX kernel ownership, interrupts and low-level services.

<a id="epok-sequence-channel-pitch-cents100-1"></a>

## `epok::sequence::Channel::pitch_cents100`

**Purpose.** Performs `pitch cents100` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
int64_t pitch_cents100() const
```

- **Declared at:** [line 18](../../../runtime/sequence_kernel.hpp#L18)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

epok::sequence::Channel& object = /* obtain a valid instance */;

auto result = object.pitch_cents100();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-copy-channels-1"></a>

## `epok::sequence::copy_channels`

**Purpose.** Performs `copy channels` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
inline void copy_channels(Channel (&destination)[16],const Channel (&source)[16])
```

- **Declared at:** [line 24](../../../runtime/sequence_kernel.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `destination` | `Channel (&)[16]` | Input/output; inspect the function contract | Value supplied for `destination`. See the exact type and module contract. |
| `source` | `const Channel (&)[16]` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// Channel (&)[16] destination
// const Channel (&)[16] source

epok::sequence::copy_channels(destination, source);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-active-1"></a>

## `epok::sequence::Kernel::active`

**Purpose.** Performs `active` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
uint32_t active() const
```

- **Declared at:** [line 168](../../../runtime/sequence_kernel.hpp#L168)
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
template<class Backend> #if defined(__mips__) && defined(__GNUC__) __attribute__((optimize("O3"))) #endif void advance(uint32_t microseconds, Backend& backend)
```

- **Declared at:** [line 226](../../../runtime/sequence_kernel.hpp#L226)
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

<a id="epok-sequence-kernel-all-notes-off-1"></a>

## `epok::sequence::Kernel::all_notes_off`

**Purpose.** Performs `all notes off` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void all_notes_off(uint8_t channel, Backend& backend)
```

- **Declared at:** [line 205](../../../runtime/sequence_kernel.hpp#L205)
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

object.all_notes_off<Backend>(channel, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-all-sound-off-1"></a>

## `epok::sequence::Kernel::all_sound_off`

**Purpose.** Performs `all sound off` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void all_sound_off(uint8_t channel, Backend& backend)
```

- **Declared at:** [line 214](../../../runtime/sequence_kernel.hpp#L214)
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

object.all_sound_off<Backend>(channel, backend);
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

- **Declared at:** [line 92](../../../runtime/sequence_kernel.hpp#L92)
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

- **Declared at:** [line 105](../../../runtime/sequence_kernel.hpp#L105)
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

<a id="epok-sequence-kernel-clear-retired-1"></a>

## `epok::sequence::Kernel::clear_retired`

**Purpose.** Clears retired as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void clear_retired()
```

- **Declared at:** [line 142](../../../runtime/sequence_kernel.hpp#L142)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.clear_retired();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-command-1"></a>

## `epok::sequence::Kernel::command`

**Purpose.** Performs `command` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
bool command()
```

- **Declared at:** [line 169](../../../runtime/sequence_kernel.hpp#L169)
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

- **Declared at:** [line 173](../../../runtime/sequence_kernel.hpp#L173)
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

- **Declared at:** [line 118](../../../runtime/sequence_kernel.hpp#L118)
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

<a id="epok-sequence-kernel-first-bit-1"></a>

## `epok::sequence::Kernel::first_bit`

**Purpose.** Performs `first bit` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static unsigned first_bit(uint32_t mask)
```

- **Declared at:** [line 119](../../../runtime/sequence_kernel.hpp#L119)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `unsigned int`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t mask

auto result = epok::sequence::Kernel::first_bit(mask);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-release-1"></a>

## `epok::sequence::Kernel::release`

**Purpose.** Performs `release` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void release(uint16_t slot, Backend& backend)
```

- **Declared at:** [line 182](../../../runtime/sequence_kernel.hpp#L182)
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

<a id="epok-sequence-kernel-release-if-unheld-1"></a>

## `epok::sequence::Kernel::release_if_unheld`

**Purpose.** Performs `release if unheld` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void release_if_unheld(uint16_t slot, Backend& backend)
```

- **Declared at:** [line 190](../../../runtime/sequence_kernel.hpp#L190)
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

object.release_if_unheld<Backend>(slot, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-release-pedal-1"></a>

## `epok::sequence::Kernel::release_pedal`

**Purpose.** Performs `release pedal` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void release_pedal(uint8_t channel, bool sustain, Backend& backend)
```

- **Declared at:** [line 195](../../../runtime/sequence_kernel.hpp#L195)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channel` | `uint8_t` | Input | Value supplied for `channel`. See the exact type and module contract. |
| `sustain` | `bool` | Input | Value supplied for `sustain`. See the exact type and module contract. |
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
// bool sustain
// Backend & backend

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.release_pedal<Backend>(channel, sustain, backend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-kernel-reset-free-notes-1"></a>

## `epok::sequence::Kernel::reset_free_notes`

**Purpose.** Resets free notes as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void reset_free_notes()
```

- **Declared at:** [line 127](../../../runtime/sequence_kernel.hpp#L127)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

epok::sequence::Kernel& object = /* obtain a valid instance */;

object.reset_free_notes();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-retire-1"></a>

## `epok::sequence::Kernel::retire`

**Purpose.** Performs `retire` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void retire(uint16_t slot)
```

- **Declared at:** [line 159](../../../runtime/sequence_kernel.hpp#L159)
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

- **Declared at:** [line 181](../../../runtime/sequence_kernel.hpp#L181)
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

<a id="epok-sequence-kernel-unlink-key-note-1"></a>

## `epok::sequence::Kernel::unlink_key_note`

**Purpose.** Performs `unlink key note` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void unlink_key_note(uint16_t slot)
```

- **Declared at:** [line 133](../../../runtime/sequence_kernel.hpp#L133)
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

object.unlink_key_note(slot);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-kernel-update-1"></a>

## `epok::sequence::Kernel::update`

**Purpose.** Updates update as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
template<class Backend> void update(uint8_t channel, Backend& backend)
```

- **Declared at:** [line 364](../../../runtime/sequence_kernel.hpp#L364)
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

<a id="epok-sequence-note-initialize-1"></a>

## `epok::sequence::Note::initialize`

**Purpose.** Performs `initialize` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void initialize(uint64_t serial,uint8_t input_channel,uint8_t input_key,uint8_t input_velocity,bool enabled=true)
```

- **Declared at:** [line 43](../../../runtime/sequence_kernel.hpp#L43)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `serial` | `uint64_t` | Input | Value supplied for `serial`. See the exact type and module contract. |
| `input_channel` | `uint8_t` | Input | Value supplied for `input_channel`. See the exact type and module contract. |
| `input_key` | `uint8_t` | Input | Value supplied for `input_key`. See the exact type and module contract. |
| `input_velocity` | `uint8_t` | Input | Value supplied for `input_velocity`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t serial
// uint8_t input_channel
// uint8_t input_key
// uint8_t input_velocity
// bool enabled

epok::sequence::Note& object = /* obtain a valid instance */;

object.initialize(serial, input_channel, input_key, input_velocity, enabled);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-valid-event-1"></a>

## `epok::sequence::valid_event`

**Purpose.** Performs `valid event` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
inline bool valid_event(const Event& e, bool extended = true)
```

- **Declared at:** [line 53](../../../runtime/sequence_kernel.hpp#L53)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `e` | `const Event &` | Input | Value supplied for `e`. See the exact type and module contract. |
| `extended` | `bool` | Input | Value supplied for `extended`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_kernel.hpp"

// Assume these named values have been initialized with valid data:
// const Event & e
// bool extended

auto result = epok::sequence::valid_event(e, extended);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
