# Epok API: Memory Card Backend

> **Header:** `"memory_card_backend.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/memory_card_backend.hpp)

This module covers asynchronous Memory Card access and files. It documents 6 public callables declared directly in this header.

## Declared types

`epok::PsyqoMemoryCardDriver`

## Callable index

- [`epok::PsyqoMemoryCardDriver::idle`](#epok-psyqomemorycarddriver-idle-1) — Performs `idle` as part of asynchronous Memory Card access and files.
- [`epok::PsyqoMemoryCardDriver::list`](#epok-psyqomemorycarddriver-list-1) — Performs `list` as part of asynchronous Memory Card access and files.
- [`epok::PsyqoMemoryCardDriver::prepare`](#epok-psyqomemorycarddriver-prepare-1) — Performs `prepare` as part of asynchronous Memory Card access and files.
- [`epok::PsyqoMemoryCardDriver::probe`](#epok-psyqomemorycarddriver-probe-1) — Performs `probe` as part of asynchronous Memory Card access and files.
- [`epok::PsyqoMemoryCardDriver::read`](#epok-psyqomemorycarddriver-read-1) — Reads read as part of asynchronous Memory Card access and files.
- [`epok::PsyqoMemoryCardDriver::write`](#epok-psyqomemorycarddriver-write-1) — Writes write as part of asynchronous Memory Card access and files.

<a id="epok-psyqomemorycarddriver-idle-1"></a>

## `epok::PsyqoMemoryCardDriver::idle`

**Purpose.** Performs `idle` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
bool idle()const override
```

- **Declared at:** [line 16](../../../runtime/memory_card_backend.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

auto result = object.idle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psyqomemorycarddriver-list-1"></a>

## `epok::PsyqoMemoryCardDriver::list`

**Purpose.** Performs `list` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void list(unsigned p,CardFile* output,uint32_t* count,void* owner,Completion complete)override
```

- **Declared at:** [line 29](../../../runtime/memory_card_backend.hpp#L29)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `output` | `CardFile *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `count` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `count`. See the exact type and module contract. |
| `owner` | `void *` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `complete` | `Completion` | Input | Value supplied for `complete`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// CardFile * output
// uint32_t * count
// void * owner
// Completion complete

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

object.list(p, output, count, owner, complete);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psyqomemorycarddriver-prepare-1"></a>

## `epok::PsyqoMemoryCardDriver::prepare`

**Purpose.** Performs `prepare` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void prepare()
```

- **Declared at:** [line 15](../../../runtime/memory_card_backend.hpp#L15)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

object.prepare();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psyqomemorycarddriver-probe-1"></a>

## `epok::PsyqoMemoryCardDriver::probe`

**Purpose.** Performs `probe` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void probe(unsigned p,void* owner,Completion complete)override
```

- **Declared at:** [line 17](../../../runtime/memory_card_backend.hpp#L17)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `owner` | `void *` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `complete` | `Completion` | Input | Value supplied for `complete`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// void * owner
// Completion complete

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

object.probe(p, owner, complete);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psyqomemorycarddriver-read-1"></a>

## `epok::PsyqoMemoryCardDriver::read`

**Purpose.** Reads read as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void read(unsigned p,const char* name,void* buffer,uint32_t capacity,uint32_t* length,void* owner,Completion complete)override
```

- **Declared at:** [line 20](../../../runtime/memory_card_backend.hpp#L20)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `capacity` | `uint32_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `length` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `length`. See the exact type and module contract. |
| `owner` | `void *` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `complete` | `Completion` | Input | Value supplied for `complete`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// const char * name
// void * buffer
// uint32_t capacity
// uint32_t * length
// void * owner
// Completion complete

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

object.read(p, name, buffer, capacity, length, owner, complete);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psyqomemorycarddriver-write-1"></a>

## `epok::PsyqoMemoryCardDriver::write`

**Purpose.** Writes write as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
void write(unsigned p,const char* name,const char* title,const CardIcon& icon,const void* data,uint32_t size,void* owner,Completion complete)override
```

- **Declared at:** [line 23](../../../runtime/memory_card_backend.hpp#L23)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `title` | `const char *` | Input | Value supplied for `title`. See the exact type and module contract. |
| `icon` | `const CardIcon &` | Input | Value supplied for `icon`. See the exact type and module contract. |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `size` | `uint32_t` | Input | Value supplied for `size`. See the exact type and module contract. |
| `owner` | `void *` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `complete` | `Completion` | Input | Value supplied for `complete`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "memory_card_backend.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// const char * name
// const char * title
// const CardIcon & icon
// const void * data
// uint32_t size
// void * owner
// Completion complete

epok::PsyqoMemoryCardDriver& object = /* obtain a valid instance */;

object.write(p, name, title, icon, data, size, owner, complete);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
