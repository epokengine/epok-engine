# Epok API: Pocketstation

> **Header:** `"pocketstation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/pocketstation.hpp)

This module covers the pocketstation module. It documents 25 public callables declared directly in this header.

## Declared types

`epok::PocketApp`, `epok::PocketError`, `epok::PocketFrame`, `epok::PocketMailbox`, `epok::PocketStationService`, `epok::PocketStatus`

## Callable index

- [`epok::pocket_detail::delay`](#epok-pocket-detail-delay-1) — Performs `delay` as part of the pocketstation module.
- [`epok::pocket_detail::mailbox_attached`](#epok-pocket-detail-mailbox-attached-1) — Performs `mailbox attached` as part of the pocketstation module.
- [`epok::pocket_detail::mailbox_exchange`](#epok-pocket-detail-mailbox-exchange-1) — Hands frames to the emulator bridge and waits for its answer.
- [`epok::pocket_detail::sio_exchange`](#epok-pocket-detail-sio-exchange-1) — SIO0 transaction, polled like PsyQo's blocking memory card path.
- [`epok::pocket_error_message`](#epok-pocket-error-message-1) — Performs `pocket error message` as part of the pocketstation module.
- [`epok::PocketStationService::call`](#epok-pocketstationservice-call-1) — Calls link function `function` of the running program.
- [`epok::PocketStationService::connected`](#epok-pocketstationservice-connected-1) — Performs `connected` as part of the pocketstation module.
- [`epok::PocketStationService::find`](#epok-pocketstationservice-find-1) — Finds a file by name; `*block` receives its first block.
- [`epok::PocketStationService::install`](#epok-pocketstationservice-install-1) — Copies an embedded program to the card (replacing an older copy) and returns its first block.
- [`epok::PocketStationService::install_and_launch`](#epok-pocketstationservice-install-and-launch-1) — Installs (if needed) and starts an embedded program.
- [`epok::PocketStationService::launch`](#epok-pocketstationservice-launch-1) — Starts the program whose first block is `block` (0 = start-up screen).
- [`epok::PocketStationService::probe`](#epok-pocketstationservice-probe-1) — True when a PocketStation (not a plain memory card) is in the slot.
- [`epok::PocketStationService::read_directory`](#epok-pocketstationservice-read-directory-1) — Reads the 16 directory frames (block 0) in one batch.
- [`epok::PocketStationService::read_memory`](#epok-pocketstationservice-read-memory-1) — Kernel memory access (RAM, I/O, flash), up to 128 bytes.
- [`epok::PocketStationService::read_sector`](#epok-pocketstationservice-read-sector-1) — Reads sector as part of the pocketstation module.
- [`epok::PocketStationService::receive`](#epok-pocketstationservice-receive-1) — Pocket Lua link: the message published with `link.send(n)`; size 0 when none.
- [`epok::PocketStationService::send`](#epok-pocketstationservice-send-1) — Pocket Lua link: a message for `link.received()` / `link.byte(i)`.
- [`epok::PocketStationService::set_led`](#epok-pocketstationservice-set-led-1) — The red LED (through the PocketStation's I/O port).
- [`epok::PocketStationService::set_time`](#epok-pocketstationservice-set-time-1) — Sets the clock (year 1900-2099).
- [`epok::PocketStationService::set_value`](#epok-pocketstationservice-set-value-1) — Pocket Lua link: sets value `index` (1-16), read with `link.value(index)`.
- [`epok::PocketStationService::status`](#epok-pocketstationservice-status-1) — Performs `status` as part of the pocketstation module.
- [`epok::PocketStationService::transfer`](#epok-pocketstationservice-transfer-1) — Runs memory card transactions on `port` (0 = first slot).
- [`epok::PocketStationService::values`](#epok-pocketstationservice-values-1) — Pocket Lua link: the 16 values set with `link.set_value(i, v)`.
- [`epok::PocketStationService::write_memory`](#epok-pocketstationservice-write-memory-1) — Writes memory as part of the pocketstation module.
- [`epok::PocketStationService::write_sector`](#epok-pocketstationservice-write-sector-1) — Writes sector as part of the pocketstation module.

<a id="epok-pocket-detail-delay-1"></a>

## `epok::pocket_detail::delay`

**Purpose.** Performs `delay` as part of the pocketstation module.

**Exact declaration**

```cpp
inline void delay(unsigned n)
```

- **Declared at:** [line 124](../../../runtime/pocketstation.hpp#L124)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `n` | `unsigned int` | Input | Value supplied for `n`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int n

epok::pocket_detail::delay(n);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocket-detail-mailbox-attached-1"></a>

## `epok::pocket_detail::mailbox_attached`

**Purpose.** Performs `mailbox attached` as part of the pocketstation module.

**Exact declaration**

```cpp
inline bool mailbox_attached()
```

- **Declared at:** [line 176](../../../runtime/pocketstation.hpp#L176)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

auto result = epok::pocket_detail::mailbox_attached();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pocket-detail-mailbox-exchange-1"></a>

## `epok::pocket_detail::mailbox_exchange`

**Purpose.** Hands frames to the emulator bridge and waits for its answer.

**Exact declaration**

```cpp
inline bool mailbox_exchange(unsigned port, PocketFrame* frames, unsigned count)
```

- **Declared at:** [line 179](../../../runtime/pocketstation.hpp#L179)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `frames` | `PocketFrame *` | Input/output; inspect the function contract | Value supplied for `frames`. See the exact type and module contract. |
| `count` | `unsigned int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// PocketFrame * frames
// unsigned int count

auto result = epok::pocket_detail::mailbox_exchange(port, frames, count);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocket-detail-sio-exchange-1"></a>

## `epok::pocket_detail::sio_exchange`

**Purpose.** SIO0 transaction, polled like PsyQo's blocking memory card path.

**Details.** Builds for the PC (native Play, editor previews) have no memory card bus.

**Exact declaration**

```cpp
inline uint16_t sio_exchange(unsigned port, const uint8_t* send, uint8_t* reply, uint16_t length, int16_t length_at)
```

- **Declared at:** [line 131](../../../runtime/pocketstation.hpp#L131)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `send` | `const uint8_t *` | Input | Value supplied for `send`. See the exact type and module contract. |
| `reply` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `reply`. See the exact type and module contract. |
| `length` | `uint16_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `length_at` | `int16_t` | Input | Value supplied for `length_at`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** Builds for the PC (native Play, editor previews) have no memory card bus.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// const uint8_t * send
// uint8_t * reply
// uint16_t length
// int16_t length_at

auto result = epok::pocket_detail::sio_exchange(port, send, reply, length, length_at);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocket-error-message-1"></a>

## `epok::pocket_error_message`

**Purpose.** Performs `pocket error message` as part of the pocketstation module.

**Exact declaration**

```cpp
inline const char* pocket_error_message(PocketError error)
```

- **Declared at:** [line 50](../../../runtime/pocketstation.hpp#L50)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `error` | `PocketError` | Input | Value supplied for `error`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// PocketError error

auto result = epok::pocket_error_message(error);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-call-1"></a>

## `epok::PocketStationService::call`

**Purpose.** Calls link function `function` of the running program.

**Details.** With `to_playstation` the program's data is read into `data` (up to `capacity` bytes, `*size` receives the count); otherwise `size` bytes of `data` are sent. `parameters` are sent first.

**Exact declaration**

```cpp
PocketError call(uint8_t function, bool to_playstation, const uint8_t* parameters, uint8_t parameter_count, uint8_t* data, uint8_t capacity, uint8_t* size, unsigned port = 0)
```

- **Declared at:** [line 272](../../../runtime/pocketstation.hpp#L272)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `function` | `uint8_t` | Input | Value supplied for `function`. See the exact type and module contract. |
| `to_playstation` | `bool` | Input | Value supplied for `to_playstation`. See the exact type and module contract. |
| `parameters` | `const uint8_t *` | Input | Value supplied for `parameters`. See the exact type and module contract. |
| `parameter_count` | `uint8_t` | Input | Value supplied for `parameter_count`. See the exact type and module contract. |
| `data` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `data`. See the exact type and module contract. |
| `capacity` | `uint8_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `size` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** With `to_playstation` the program's data is read into `data` (up to `capacity` bytes, `*size` receives the count); otherwise `size` bytes of `data` are sent. `parameters` are sent first.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t function
// bool to_playstation
// const uint8_t * parameters
// uint8_t parameter_count
// uint8_t * data
// uint8_t capacity
// uint8_t * size
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.call(function, to_playstation, parameters, parameter_count, data, capacity, size, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-connected-1"></a>

## `epok::PocketStationService::connected`

**Purpose.** Performs `connected` as part of the pocketstation module.

**Exact declaration**

```cpp
bool connected(unsigned port = 0)
```

- **Declared at:** [line 236](../../../runtime/pocketstation.hpp#L236)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.connected(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pocketstationservice-find-1"></a>

## `epok::PocketStationService::find`

**Purpose.** Finds a file by name; `*block` receives its first block.

**Exact declaration**

```cpp
PocketError find(const char* filename, uint8_t* block, unsigned port = 0)
```

- **Declared at:** [line 381](../../../runtime/pocketstation.hpp#L381)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `filename` | `const char *` | Input | Value supplied for `filename`. See the exact type and module contract. |
| `block` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `block`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// const char * filename
// uint8_t * block
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.find(filename, block, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-install-1"></a>

## `epok::PocketStationService::install`

**Purpose.** Copies an embedded program to the card (replacing an older copy) and returns its first block.

**Details.** Takes about a second for a one-block program.

**Exact declaration**

```cpp
PocketError install(const PocketApp& app, uint8_t* first_block = nullptr, unsigned port = 0)
```

- **Declared at:** [line 396](../../../runtime/pocketstation.hpp#L396)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `app` | `const PocketApp &` | Input | Value supplied for `app`. See the exact type and module contract. |
| `first_block` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `first_block`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** Takes about a second for a one-block program.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// const PocketApp & app
// uint8_t * first_block
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.install(app, first_block, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-install-and-launch-1"></a>

## `epok::PocketStationService::install_and_launch`

**Purpose.** Installs (if needed) and starts an embedded program.

**Exact declaration**

```cpp
PocketError install_and_launch(const PocketApp& app, uint32_t parameter = 0, unsigned port = 0)
```

- **Declared at:** [line 478](../../../runtime/pocketstation.hpp#L478)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `app` | `const PocketApp &` | Input | Value supplied for `app`. See the exact type and module contract. |
| `parameter` | `uint32_t` | Input | Value supplied for `parameter`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// const PocketApp & app
// uint32_t parameter
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.install_and_launch(app, parameter, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-launch-1"></a>

## `epok::PocketStationService::launch`

**Purpose.** Starts the program whose first block is `block` (0 = start-up screen).

**Details.** The PocketStation switches when the running program allows it.

**Exact declaration**

```cpp
PocketError launch(uint8_t block, uint32_t parameter = 0, unsigned port = 0)
```

- **Declared at:** [line 259](../../../runtime/pocketstation.hpp#L259)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `block` | `uint8_t` | Input | Value supplied for `block`. See the exact type and module contract. |
| `parameter` | `uint32_t` | Input | Value supplied for `parameter`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** The PocketStation switches when the running program allows it.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t block
// uint32_t parameter
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.launch(block, parameter, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-probe-1"></a>

## `epok::PocketStationService::probe`

**Purpose.** True when a PocketStation (not a plain memory card) is in the slot.

**Exact declaration**

```cpp
PocketError probe(unsigned port = 0)
```

- **Declared at:** [line 228](../../../runtime/pocketstation.hpp#L228)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.probe(port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-read-directory-1"></a>

## `epok::PocketStationService::read_directory`

**Purpose.** Reads the 16 directory frames (block 0) in one batch.

**Exact declaration**

```cpp
PocketError read_directory(uint8_t (*out)[128], unsigned port = 0)
```

- **Declared at:** [line 359](../../../runtime/pocketstation.hpp#L359)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `uint8_t (*)[128]` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t (*)[128] out
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.read_directory(out, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-read-memory-1"></a>

## `epok::PocketStationService::read_memory`

**Purpose.** Kernel memory access (RAM, I/O, flash), up to 128 bytes.

**Exact declaration**

```cpp
PocketError read_memory(uint32_t address, void* out, uint8_t size, unsigned port = 0)
```

- **Declared at:** [line 332](../../../runtime/pocketstation.hpp#L332)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |
| `out` | `void *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `size` | `uint8_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address
// void * out
// uint8_t size
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.read_memory(address, out, size, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-read-sector-1"></a>

## `epok::PocketStationService::read_sector`

**Purpose.** Reads sector as part of the pocketstation module.

**Exact declaration**

```cpp
PocketError read_sector(uint16_t sector, uint8_t* out, unsigned port = 0)
```

- **Declared at:** [line 486](../../../runtime/pocketstation.hpp#L486)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `out` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t sector
// uint8_t * out
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.read_sector(sector, out, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-receive-1"></a>

## `epok::PocketStationService::receive`

**Purpose.** Pocket Lua link: the message published with `link.send(n)`; size 0 when none.

**Exact declaration**

```cpp
PocketError receive(void* message, uint8_t capacity, uint8_t* size, unsigned port = 0)
```

- **Declared at:** [line 310](../../../runtime/pocketstation.hpp#L310)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `message` | `void *` | Input/output; inspect the function contract | Value supplied for `message`. See the exact type and module contract. |
| `capacity` | `uint8_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `size` | `uint8_t *` | Input/output; inspect the function contract | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// void * message
// uint8_t capacity
// uint8_t * size
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.receive(message, capacity, size, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-send-1"></a>

## `epok::PocketStationService::send`

**Purpose.** Pocket Lua link: a message for `link.received()` / `link.byte(i)`.

**Exact declaration**

```cpp
PocketError send(const void* message, uint8_t size, unsigned port = 0)
```

- **Declared at:** [line 304](../../../runtime/pocketstation.hpp#L304)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `message` | `const void *` | Input | Value supplied for `message`. See the exact type and module contract. |
| `size` | `uint8_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// const void * message
// uint8_t size
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.send(message, size, port);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-set-led-1"></a>

## `epok::PocketStationService::set_led`

**Purpose.** The red LED (through the PocketStation's I/O port).

**Exact declaration**

```cpp
PocketError set_led(bool on, unsigned port = 0)
```

- **Declared at:** [line 347](../../../runtime/pocketstation.hpp#L347)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `on` | `bool` | Input | Value supplied for `on`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// bool on
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.set_led(on, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-set-time-1"></a>

## `epok::PocketStationService::set_time`

**Purpose.** Sets the clock (year 1900-2099).

**Exact declaration**

```cpp
PocketError set_time(uint16_t year, uint8_t month, uint8_t day, uint8_t hour, uint8_t minute, uint8_t second, uint8_t weekday, unsigned port = 0)
```

- **Declared at:** [line 352](../../../runtime/pocketstation.hpp#L352)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `year` | `uint16_t` | Input | Value supplied for `year`. See the exact type and module contract. |
| `month` | `uint8_t` | Input | Value supplied for `month`. See the exact type and module contract. |
| `day` | `uint8_t` | Input | Value supplied for `day`. See the exact type and module contract. |
| `hour` | `uint8_t` | Input | Value supplied for `hour`. See the exact type and module contract. |
| `minute` | `uint8_t` | Input | Value supplied for `minute`. See the exact type and module contract. |
| `second` | `uint8_t` | Input | Value supplied for `second`. See the exact type and module contract. |
| `weekday` | `uint8_t` | Input | Value supplied for `weekday`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t year
// uint8_t month
// uint8_t day
// uint8_t hour
// uint8_t minute
// uint8_t second
// uint8_t weekday
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.set_time(year, month, day, hour, minute, second, weekday, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-set-value-1"></a>

## `epok::PocketStationService::set_value`

**Purpose.** Pocket Lua link: sets value `index` (1-16), read with `link.value(index)`.

**Exact declaration**

```cpp
PocketError set_value(uint8_t index, int32_t value, unsigned port = 0)
```

- **Declared at:** [line 323](../../../runtime/pocketstation.hpp#L323)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint8_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t index
// int32_t value
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.set_value(index, value, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-status-1"></a>

## `epok::PocketStationService::status`

**Purpose.** Performs `status` as part of the pocketstation module.

**Exact declaration**

```cpp
PocketError status(PocketStatus& out, unsigned port = 0)
```

- **Declared at:** [line 238](../../../runtime/pocketstation.hpp#L238)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `PocketStatus &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// PocketStatus & out
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.status(out, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-transfer-1"></a>

## `epok::PocketStationService::transfer`

**Purpose.** Runs memory card transactions on `port` (0 = first slot).

**Exact declaration**

```cpp
void transfer(unsigned port, PocketFrame* frames, unsigned count)
```

- **Declared at:** [line 216](../../../runtime/pocketstation.hpp#L216)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `frames` | `PocketFrame *` | Input/output; inspect the function contract | Value supplied for `frames`. See the exact type and module contract. |
| `count` | `unsigned int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// PocketFrame * frames
// unsigned int count

epok::PocketStationService& object = /* obtain a valid instance */;

object.transfer(port, frames, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-values-1"></a>

## `epok::PocketStationService::values`

**Purpose.** Pocket Lua link: the 16 values set with `link.set_value(i, v)`.

**Exact declaration**

```cpp
PocketError values(int32_t out[value_count], unsigned port = 0)
```

- **Declared at:** [line 314](../../../runtime/pocketstation.hpp#L314)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `int32_t[16]` | Input | Value supplied for `out`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// int32_t[16] out
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.values(out, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pocketstationservice-write-memory-1"></a>

## `epok::PocketStationService::write_memory`

**Purpose.** Writes memory as part of the pocketstation module.

**Exact declaration**

```cpp
PocketError write_memory(uint32_t address, const void* data, uint8_t size, unsigned port = 0)
```

- **Declared at:** [line 340](../../../runtime/pocketstation.hpp#L340)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `size` | `uint8_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address
// const void * data
// uint8_t size
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.write_memory(address, data, size, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocketstationservice-write-sector-1"></a>

## `epok::PocketStationService::write_sector`

**Purpose.** Writes sector as part of the pocketstation module.

**Exact declaration**

```cpp
PocketError write_sector(uint16_t sector, const uint8_t* data, unsigned port = 0)
```

- **Declared at:** [line 494](../../../runtime/pocketstation.hpp#L494)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint16_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `PocketError`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t sector
// const uint8_t * data
// unsigned int port

epok::PocketStationService& object = /* obtain a valid instance */;

auto result = object.write_sector(sector, data, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
