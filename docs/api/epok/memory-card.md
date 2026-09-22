# Epok API: Memory Card

> **Header:** `"memory_card.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/memory_card.hpp)

This module covers asynchronous Memory Card access and files. It documents 20 public callables declared directly in this header.

## Declared types

`epok::CardError`, `epok::CardFile`, `epok::CardIcon`, `epok::CardOperation`, `epok::CardState`, `epok::CardStatus`, `epok::MemoryCardDriver`, `epok::MemoryCardDriver::Completion`, `epok::MemoryCardService`

## Callable index

- [`epok::card_error_message`](#epok-card-error-message-1) — Performs `card error message` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardDriver::idle`](#epok-memorycarddriver-idle-1) — Performs `idle` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardDriver::list`](#epok-memorycarddriver-list-1) — Performs `list` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardDriver::probe`](#epok-memorycarddriver-probe-1) — Performs `probe` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardDriver::read`](#epok-memorycarddriver-read-1) — Reads read as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardDriver::write`](#epok-memorycarddriver-write-1) — Writes write as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::attach`](#epok-memorycardservice-attach-1) — Performs `attach` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::busy`](#epok-memorycardservice-busy-1) — Performs `busy` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::data`](#epok-memorycardservice-data-1) — Performs `data` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::file_count`](#epok-memorycardservice-file-count-1) — Performs `file count` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::files`](#epok-memorycardservice-files-1) — Performs `files` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::list`](#epok-memorycardservice-list-1) — Performs `list` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::MemoryCardService`](#epok-memorycardservice-memorycardservice-1) — Constructs `epok::MemoryCardService` for asynchronous Memory Card access and files.
- [`epok::MemoryCardService::MemoryCardService`](#epok-memorycardservice-memorycardservice-2) — Constructs `epok::MemoryCardService` for asynchronous Memory Card access and files.
- [`epok::MemoryCardService::operator=`](#epok-memorycardservice-operator-1) — Performs `operator =` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::probe`](#epok-memorycardservice-probe-1) — Performs `probe` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::read`](#epok-memorycardservice-read-1) — Reads read as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::size`](#epok-memorycardservice-size-1) — Performs `size` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::status`](#epok-memorycardservice-status-1) — Performs `status` as part of asynchronous Memory Card access and files.
- [`epok::MemoryCardService::write`](#epok-memorycardservice-write-1) — Writes write as part of asynchronous Memory Card access and files.

<a id="epok-card-error-message-1"></a>

## `epok::card_error_message`

**Purpose.** Performs `card error message` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
inline const char* card_error_message(CardError error)
```

- **Declared at:** [line 162](../../../runtime/memory_card.hpp#L162)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `error` | `CardError` | Input | Value supplied for `error`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// CardError error

auto result = epok::card_error_message(error);
```

**Why choose it.** It provides direct, allocation-conscious access to asynchronous Memory Card access and files. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-memorycarddriver-idle-1"></a>

## `epok::MemoryCardDriver::idle`

**Purpose.** Performs `idle` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
virtual bool idle() const=0
```

- **Declared at:** [line 26](../../../runtime/memory_card.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardDriver& object = /* obtain a valid instance */;

auto result = object.idle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycarddriver-list-1"></a>

## `epok::MemoryCardDriver::list`

**Purpose.** Performs `list` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
virtual void list(unsigned,CardFile*,uint32_t*,void*,Completion)=0
```

- **Declared at:** [line 30](../../../runtime/memory_card.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `unsigned int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `CardFile *` | Input/output; inspect the function contract | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `arg3`. See the exact type and module contract. |
| `arg4` | `void *` | Input/output; inspect the function contract | Value supplied for `arg4`. See the exact type and module contract. |
| `arg5` | `Completion` | Input | Value supplied for `arg5`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int arg1
// CardFile * arg2
// uint32_t * arg3
// void * arg4
// Completion arg5

epok::MemoryCardDriver& object = /* obtain a valid instance */;

object.list(arg1, arg2, arg3, arg4, arg5);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycarddriver-probe-1"></a>

## `epok::MemoryCardDriver::probe`

**Purpose.** Performs `probe` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
virtual void probe(unsigned,void*,Completion)=0
```

- **Declared at:** [line 27](../../../runtime/memory_card.hpp#L27)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `unsigned int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `void *` | Input/output; inspect the function contract | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `Completion` | Input | Value supplied for `arg3`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int arg1
// void * arg2
// Completion arg3

epok::MemoryCardDriver& object = /* obtain a valid instance */;

object.probe(arg1, arg2, arg3);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycarddriver-read-1"></a>

## `epok::MemoryCardDriver::read`

**Purpose.** Reads read as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
virtual void read(unsigned,const char*,void*,uint32_t,uint32_t*,void*,Completion)=0
```

- **Declared at:** [line 28](../../../runtime/memory_card.hpp#L28)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `unsigned int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `const char *` | Input | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `void *` | Input/output; inspect the function contract | Value supplied for `arg3`. See the exact type and module contract. |
| `arg4` | `uint32_t` | Input | Value supplied for `arg4`. See the exact type and module contract. |
| `arg5` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `arg5`. See the exact type and module contract. |
| `arg6` | `void *` | Input/output; inspect the function contract | Value supplied for `arg6`. See the exact type and module contract. |
| `arg7` | `Completion` | Input | Value supplied for `arg7`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int arg1
// const char * arg2
// void * arg3
// uint32_t arg4
// uint32_t * arg5
// void * arg6
// Completion arg7

epok::MemoryCardDriver& object = /* obtain a valid instance */;

object.read(arg1, arg2, arg3, arg4, arg5, arg6, arg7);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycarddriver-write-1"></a>

## `epok::MemoryCardDriver::write`

**Purpose.** Writes write as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
virtual void write(unsigned,const char*,const char*,const CardIcon&,const void*,uint32_t,void*,Completion)=0
```

- **Declared at:** [line 29](../../../runtime/memory_card.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `unsigned int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `const char *` | Input | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `const char *` | Input | Value supplied for `arg3`. See the exact type and module contract. |
| `arg4` | `const CardIcon &` | Input | Value supplied for `arg4`. See the exact type and module contract. |
| `arg5` | `const void *` | Input | Value supplied for `arg5`. See the exact type and module contract. |
| `arg6` | `uint32_t` | Input | Value supplied for `arg6`. See the exact type and module contract. |
| `arg7` | `void *` | Input/output; inspect the function contract | Value supplied for `arg7`. See the exact type and module contract. |
| `arg8` | `Completion` | Input | Value supplied for `arg8`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int arg1
// const char * arg2
// const char * arg3
// const CardIcon & arg4
// const void * arg5
// uint32_t arg6
// void * arg7
// Completion arg8

epok::MemoryCardDriver& object = /* obtain a valid instance */;

object.write(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycardservice-attach-1"></a>

## `epok::MemoryCardService::attach`

**Purpose.** Performs `attach` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool attach(MemoryCardDriver& value)
```

- **Declared at:** [line 130](../../../runtime/memory_card.hpp#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `MemoryCardDriver &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// MemoryCardDriver & value

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.attach(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycardservice-busy-1"></a>

## `epok::MemoryCardService::busy`

**Purpose.** Performs `busy` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool busy()const
```

- **Declared at:** [line 131](../../../runtime/memory_card.hpp#L131)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.busy();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-data-1"></a>

## `epok::MemoryCardService::data`

**Purpose.** Performs `data` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
const uint8_t* data()const
```

- **Declared at:** [line 133](../../../runtime/memory_card.hpp#L133)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.data();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-file-count-1"></a>

## `epok::MemoryCardService::file_count`

**Purpose.** Performs `file count` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
uint32_t file_count()const
```

- **Declared at:** [line 136](../../../runtime/memory_card.hpp#L136)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.file_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-files-1"></a>

## `epok::MemoryCardService::files`

**Purpose.** Performs `files` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
const CardFile* files()const
```

- **Declared at:** [line 135](../../../runtime/memory_card.hpp#L135)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const CardFile *`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.files();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-list-1"></a>

## `epok::MemoryCardService::list`

**Purpose.** Performs `list` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool list(unsigned port=0)
```

- **Declared at:** [line 140](../../../runtime/memory_card.hpp#L140)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.list(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-memorycardservice-1"></a>

## `epok::MemoryCardService::MemoryCardService`

**Purpose.** Constructs `epok::MemoryCardService` for asynchronous Memory Card access and files.

**Exact declaration**

```cpp
MemoryCardService()=default
```

- **Declared at:** [line 127](../../../runtime/memory_card.hpp#L127)
- **Kind:** `constructor`

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService value();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-memorycardservice-2"></a>

## `epok::MemoryCardService::MemoryCardService`

**Purpose.** Constructs `epok::MemoryCardService` for asynchronous Memory Card access and files.

**Exact declaration**

```cpp
MemoryCardService(const MemoryCardService&)=delete
```

- **Declared at:** [line 128](../../../runtime/memory_card.hpp#L128)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const MemoryCardService &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// const MemoryCardService & arg1

epok::MemoryCardService value(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycardservice-operator-1"></a>

## `epok::MemoryCardService::operator=`

**Purpose.** Performs `operator =` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
MemoryCardService& operator=(const MemoryCardService&)=delete
```

- **Declared at:** [line 129](../../../runtime/memory_card.hpp#L129)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const MemoryCardService &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `MemoryCardService &`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// const MemoryCardService & arg1

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycardservice-probe-1"></a>

## `epok::MemoryCardService::probe`

**Purpose.** Performs `probe` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool probe(unsigned port=0)
```

- **Declared at:** [line 137](../../../runtime/memory_card.hpp#L137)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.probe(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-read-1"></a>

## `epok::MemoryCardService::read`

**Purpose.** Reads read as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool read(const char* name,unsigned port=0)
```

- **Declared at:** [line 143](../../../runtime/memory_card.hpp#L143)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// const char * name
// unsigned int port

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.read(name, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-memorycardservice-size-1"></a>

## `epok::MemoryCardService::size`

**Purpose.** Performs `size` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
uint32_t size()const
```

- **Declared at:** [line 134](../../../runtime/memory_card.hpp#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-status-1"></a>

## `epok::MemoryCardService::status`

**Purpose.** Performs `status` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
const CardStatus& status()const
```

- **Declared at:** [line 132](../../../runtime/memory_card.hpp#L132)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const CardStatus &`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.status();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardservice-write-1"></a>

## `epok::MemoryCardService::write`

**Purpose.** Writes write as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool write(const char* name,const char* title,const void* data,uint32_t size,unsigned port=0,const CardIcon* icon=nullptr)
```

- **Declared at:** [line 148](../../../runtime/memory_card.hpp#L148)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `title` | `const char *` | Input | Value supplied for `title`. See the exact type and module contract. |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `size` | `uint32_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `icon` | `const CardIcon *` | Input | Value supplied for `icon`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card.hpp"

// Assume these named values have been initialized with valid data:
// const char * name
// const char * title
// const void * data
// uint32_t size
// unsigned int port
// const CardIcon * icon

epok::MemoryCardService& object = /* obtain a valid instance */;

auto result = object.write(name, title, data, size, port, icon);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
