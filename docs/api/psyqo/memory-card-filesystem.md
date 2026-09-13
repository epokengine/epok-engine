# PsyQo API: Memory Card Filesystem

> **Header:** `"psyqo/memory-card-filesystem.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh)

This module covers asynchronous Memory Card access and files. It documents 20 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::MemoryCardFileSystem`, `psyqo::MemoryCardFileSystem::Error`, `psyqo::MemoryCardFileSystem::FileEntry`, `psyqo::MemoryCardFileSystem::FileInfo`, `psyqo::MemoryCardFileSystem::Icon`, `psyqo::MemoryCardFileSystem::Port`

## Callable index

- [`psyqo::MemoryCardFileSystem::deleteFile`](#psyqo-memorycardfilesystem-deletefile-1) — Deletes a file, freeing all of its blocks.
- [`psyqo::MemoryCardFileSystem::deleteFileBlocking`](#psyqo-memorycardfilesystem-deletefileblocking-1) — Performs `delete file blocking` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::fileExists`](#psyqo-memorycardfilesystem-fileexists-1) — Reports whether a named file exists, into *outExists.
- [`psyqo::MemoryCardFileSystem::fileExistsBlocking`](#psyqo-memorycardfilesystem-fileexistsblocking-1) — Performs `file exists blocking` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::format`](#psyqo-memorycardfilesystem-format-1) — Writes a fresh, empty Sony filesystem to the card.
- [`psyqo::MemoryCardFileSystem::formatBlocking`](#psyqo-memorycardfilesystem-formatblocking-1) — Performs `format blocking` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::getCardState`](#psyqo-memorycardfilesystem-getcardstate-1) — Determines whether a usable, formatted card is present.
- [`psyqo::MemoryCardFileSystem::getCardStateBlocking`](#psyqo-memorycardfilesystem-getcardstateblocking-1) — -- Blocking variants -------------------------------------------------- These run the same transaction but pump the GPU until it finishes and return the error directly.
- [`psyqo::MemoryCardFileSystem::getFreeBlockCount`](#psyqo-memorycardfilesystem-getfreeblockcount-1) — Counts the free 8KiB blocks (0..15) into *outFreeBlocks.
- [`psyqo::MemoryCardFileSystem::getFreeBlockCountBlocking`](#psyqo-memorycardfilesystem-getfreeblockcountblocking-1) — Returns free block count blocking as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::isIdle`](#psyqo-memorycardfilesystem-isidle-1) — Whether the filesystem is ready to accept a new operation.
- [`psyqo::MemoryCardFileSystem::listFiles`](#psyqo-memorycardfilesystem-listfiles-1) — Lists the files on the card.
- [`psyqo::MemoryCardFileSystem::listFilesBlocking`](#psyqo-memorycardfilesystem-listfilesblocking-1) — Performs `list files blocking` as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::MemoryCardFileSystem`](#psyqo-memorycardfilesystem-memorycardfilesystem-1) — Constructs `psyqo::MemoryCardFileSystem` for asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::readFile`](#psyqo-memorycardfilesystem-readfile-1) — Reads the payload of a file.
- [`psyqo::MemoryCardFileSystem::readFileBlocking`](#psyqo-memorycardfilesystem-readfileblocking-1) — Reads file blocking as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::readFileInfo`](#psyqo-memorycardfilesystem-readfileinfo-1) — Reads a file's title and icon.
- [`psyqo::MemoryCardFileSystem::readFileInfoBlocking`](#psyqo-memorycardfilesystem-readfileinfoblocking-1) — Reads file info blocking as part of asynchronous Memory Card access and files.
- [`psyqo::MemoryCardFileSystem::writeFile`](#psyqo-memorycardfilesystem-writefile-1) — Creates or overwrites a file.
- [`psyqo::MemoryCardFileSystem::writeFileBlocking`](#psyqo-memorycardfilesystem-writefileblocking-1) — Writes file blocking as part of asynchronous Memory Card access and files.

<a id="psyqo-memorycardfilesystem-deletefile-1"></a>

## `psyqo::MemoryCardFileSystem::deleteFile`

**Purpose.** Deletes a file, freeing all of its blocks.

**Exact declaration**

```cpp
void deleteFile(Port port, const char *name, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 219](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L219)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// const char * name
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.deleteFile(port, name, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-deletefileblocking-1"></a>

## `psyqo::MemoryCardFileSystem::deleteFileBlocking`

**Purpose.** Performs `delete file blocking` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error deleteFileBlocking(GPU &gpu, Port port, const char *name)
```

- **Declared at:** [line 235](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L235)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// const char * name

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.deleteFileBlocking(gpu, port, name);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-fileexists-1"></a>

## `psyqo::MemoryCardFileSystem::fileExists`

**Purpose.** Reports whether a named file exists, into *outExists.

**Exact declaration**

```cpp
void fileExists(Port port, const char *name, bool *outExists, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L162)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `outExists` | `bool *` | Input/output; inspect the function contract | Value supplied for `outExists`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// const char * name
// bool * outExists
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.fileExists(port, name, outExists, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-fileexistsblocking-1"></a>

## `psyqo::MemoryCardFileSystem::fileExistsBlocking`

**Purpose.** Performs `file exists blocking` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error fileExistsBlocking(GPU &gpu, Port port, const char *name, bool *outExists)
```

- **Declared at:** [line 230](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L230)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `outExists` | `bool *` | Input/output; inspect the function contract | Value supplied for `outExists`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// const char * name
// bool * outExists

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.fileExistsBlocking(gpu, port, name, outExists);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-format-1"></a>

## `psyqo::MemoryCardFileSystem::format`

**Purpose.** Writes a fresh, empty Sony filesystem to the card.

**Details.** This erases the directory; any existing files become unreachable. The 15 file blocks themselves are not touched (they are simply marked free), matching what the BIOS does.

**Exact declaration**

```cpp
void format(Port port, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 142](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L142)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This erases the directory; any existing files become unreachable. The 15 file blocks themselves are not touched (they are simply marked free), matching what the BIOS does.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.format(port, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-formatblocking-1"></a>

## `psyqo::MemoryCardFileSystem::formatBlocking`

**Purpose.** Performs `format blocking` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error formatBlocking(GPU &gpu, Port port)
```

- **Declared at:** [line 227](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L227)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.formatBlocking(gpu, port);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-getcardstate-1"></a>

## `psyqo::MemoryCardFileSystem::getCardState`

**Purpose.** Determines whether a usable, formatted card is present.

**Exact declaration**

```cpp
void getCardState(Port port, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 133](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L133)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** Via the callback: Error::OK if formatted, Error::NoCard if absent, Error::NotFormatted if present but not a Sony card.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.getCardState(port, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-getcardstateblocking-1"></a>

## `psyqo::MemoryCardFileSystem::getCardStateBlocking`

**Purpose.** -- Blocking variants -------------------------------------------------- These run the same transaction but pump the GPU until it finishes and return the error directly.

**Details.** They still take a few hundred milliseconds for a non-trivial payload, so they are best used at a deliberate save point.

**Exact declaration**

```cpp
Error getCardStateBlocking(GPU &gpu, Port port)
```

- **Declared at:** [line 226](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L226)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** They still take a few hundred milliseconds for a non-trivial payload, so they are best used at a deliberate save point.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.getCardStateBlocking(gpu, port);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-getfreeblockcount-1"></a>

## `psyqo::MemoryCardFileSystem::getFreeBlockCount`

**Purpose.** Counts the free 8KiB blocks (0..15) into *outFreeBlocks.

**Exact declaration**

```cpp
void getFreeBlockCount(Port port, uint32_t *outFreeBlocks, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 147](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L147)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `outFreeBlocks` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `outFreeBlocks`. See the exact type and module contract. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// uint32_t * outFreeBlocks
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.getFreeBlockCount(port, outFreeBlocks, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-getfreeblockcountblocking-1"></a>

## `psyqo::MemoryCardFileSystem::getFreeBlockCountBlocking`

**Purpose.** Returns free block count blocking as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error getFreeBlockCountBlocking(GPU &gpu, Port port, uint32_t *outFreeBlocks)
```

- **Declared at:** [line 228](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L228)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `outFreeBlocks` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `outFreeBlocks`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// uint32_t * outFreeBlocks

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.getFreeBlockCountBlocking(gpu, port, outFreeBlocks);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-isidle-1"></a>

## `psyqo::MemoryCardFileSystem::isIdle`

**Purpose.** Whether the filesystem is ready to accept a new operation.

**Exact declaration**

```cpp
bool isIdle() const
```

- **Declared at:** [line 118](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L118)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.isIdle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-memorycardfilesystem-listfiles-1"></a>

## `psyqo::MemoryCardFileSystem::listFiles`

**Purpose.** Lists the files on the card.

**Exact declaration**

```cpp
void listFiles(Port port, FileEntry *out, uint32_t maxEntries, uint32_t *outCount, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 156](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L156)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `out` | `FileEntry *` | Input/output; inspect the function contract | An array to receive up to `maxEntries` entries. |
| `maxEntries` | `uint32_t` | Input | Value supplied for `maxEntries`. See the exact type and module contract. |
| `outCount` | `uint32_t *` | Input/output; inspect the function contract | Receives the number of files found (may exceed `maxEntries`, in which case only `maxEntries` were written). |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// FileEntry * out
// uint32_t maxEntries
// uint32_t * outCount
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.listFiles(port, out, maxEntries, outCount, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-listfilesblocking-1"></a>

## `psyqo::MemoryCardFileSystem::listFilesBlocking`

**Purpose.** Performs `list files blocking` as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error listFilesBlocking(GPU &gpu, Port port, FileEntry *out, uint32_t maxEntries, uint32_t *outCount)
```

- **Declared at:** [line 229](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L229)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `out` | `FileEntry *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `maxEntries` | `uint32_t` | Input | Value supplied for `maxEntries`. See the exact type and module contract. |
| `outCount` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `outCount`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// FileEntry * out
// uint32_t maxEntries
// uint32_t * outCount

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.listFilesBlocking(gpu, port, out, maxEntries, outCount);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-memorycardfilesystem-1"></a>

## `psyqo::MemoryCardFileSystem::MemoryCardFileSystem`

**Purpose.** Constructs `psyqo::MemoryCardFileSystem` for asynchronous Memory Card access and files.

**Exact declaration**

```cpp
explicit MemoryCardFileSystem(MemoryCard &card) : m
```

- **Declared at:** [line 113](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L113)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `card` | `MemoryCard &` | Input/output; inspect the function contract | Value supplied for `card`. See the exact type and module contract. |

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// MemoryCard & card

psyqo::MemoryCardFileSystem value(card);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-readfile-1"></a>

## `psyqo::MemoryCardFileSystem::readFile`

**Purpose.** Reads the payload of a file.

**Details.** Returns the bytes that follow the title and icon frames, i.e. exactly the `data` region passed to `writeFile`, rounded up to whole frames. The caller is responsible for knowing the logical length of its own payload (typically via a small header it embeds in `data`).

**Exact declaration**

```cpp
void readFile(Port port, const char *name, void *buffer, uint32_t maxLen, uint32_t *outLen, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 176](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L176)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Receives up to `maxLen` payload bytes. |
| `maxLen` | `uint32_t` | Input | Value supplied for `maxLen`. See the exact type and module contract. |
| `outLen` | `uint32_t *` | Input/output; inspect the function contract | Receives the number of payload bytes available (capped at `maxLen`). |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Returns the bytes that follow the title and icon frames, i.e. exactly the `data` region passed to `writeFile`, rounded up to whole frames. The caller is responsible for knowing the logical length of its own payload (typically via a small header it embeds in `data`).

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// const char * name
// void * buffer
// uint32_t maxLen
// uint32_t * outLen
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.readFile(port, name, buffer, maxLen, outLen, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-readfileblocking-1"></a>

## `psyqo::MemoryCardFileSystem::readFileBlocking`

**Purpose.** Reads file blocking as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error readFileBlocking(GPU &gpu, Port port, const char *name, void *buffer, uint32_t maxLen, uint32_t *outLen)
```

- **Declared at:** [line 231](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L231)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `maxLen` | `uint32_t` | Input | Value supplied for `maxLen`. See the exact type and module contract. |
| `outLen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `outLen`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// const char * name
// void * buffer
// uint32_t maxLen
// uint32_t * outLen

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.readFileBlocking(gpu, port, name, buffer, maxLen, outLen);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-readfileinfo-1"></a>

## `psyqo::MemoryCardFileSystem::readFileInfo`

**Purpose.** Reads a file's title and icon.

**Details.** Fills `*out` with the file's title frame (the raw Shift-JIS title and the icon palette) and its icon bitmap frames, so a caller can display saved blocks the way the BIOS manager does. This is the read counterpart to `writeFile`'s title and icon; it reads only the file's first block, never the payload.

**Exact declaration**

```cpp
void readFileInfo(Port port, const char *name, FileInfo *out, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 193](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L193)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | The Sony filename to look up. Must stay valid until the callback fires. |
| `out` | `FileInfo *` | Input/output; inspect the function contract | Receives the title and icon. Must stay valid until the callback fires. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Fills `*out` with the file's title frame (the raw Shift-JIS title and the icon palette) and its icon bitmap frames, so a caller can display saved blocks the way the BIOS manager does. This is the read counterpart to `writeFile`'s title and icon; it reads only the file's first block, never the payload.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// const char * name
// FileInfo * out
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.readFileInfo(port, name, out, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-readfileinfoblocking-1"></a>

## `psyqo::MemoryCardFileSystem::readFileInfoBlocking`

**Purpose.** Reads file info blocking as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error readFileInfoBlocking(GPU &gpu, Port port, const char *name, FileInfo *out)
```

- **Declared at:** [line 232](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L232)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `out` | `FileInfo *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// const char * name
// FileInfo * out

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.readFileInfoBlocking(gpu, port, name, out);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-writefile-1"></a>

## `psyqo::MemoryCardFileSystem::writeFile`

**Purpose.** Creates or overwrites a file.

**Details.** The file is sized to hold the title frame, the icon frames and `dataLen` payload bytes, rounded up to whole 8KiB blocks. If a file with the same name already exists it is replaced. The data is written first and the directory committed last, so an interrupted write never leaves a referenced but corrupt file.

**Exact declaration**

```cpp
void writeFile(Port port, const char *name, const char *title, const Icon &icon, const void *data, uint32_t dataLen, eastl::function<void(Error)> &&callback)
```

- **Declared at:** [line 213](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L213)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | The Sony filename (up to 20 characters). The pointer must stay valid until the callback fires. |
| `title` | `const char *` | Input | The save title, as a UTF-8 string, encoded to the 64-byte Shift-JIS field the BIOS manager displays, with printable ASCII promoted to its fullwidth form. Must stay valid until the callback fires. |
| `icon` | `const Icon &` | Input | The save icon. Copied, so it need not outlive the call. |
| `data` | `const void *` | Input | The payload bytes. Must stay valid until the callback fires. |
| `dataLen` | `uint32_t` | Input | The number of payload bytes. |
| `callback` | `eastl::function<void (Error)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The file is sized to hold the title frame, the icon frames and `dataLen` payload bytes, rounded up to whole 8KiB blocks. If a file with the same name already exists it is replaced. The data is written first and the directory committed last, so an interrupted write never leaves a referenced but corrupt file.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// Port port
// const char * name
// const char * title
// const Icon & icon
// const void * data
// uint32_t dataLen
// eastl::function<void (Error)> && callback

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

object.writeFile(port, name, title, icon, data, dataLen, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-memorycardfilesystem-writefileblocking-1"></a>

## `psyqo::MemoryCardFileSystem::writeFileBlocking`

**Purpose.** Writes file blocking as part of asynchronous Memory Card access and files.

**Exact declaration**

```cpp
Error writeFileBlocking(GPU &gpu, Port port, const char *name, const char *title, const Icon &icon, const void *data, uint32_t dataLen)
```

- **Declared at:** [line 233](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/memory-card-filesystem.hh#L233)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `port` | `Port` | Input | Value supplied for `port`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `title` | `const char *` | Input | Value supplied for `title`. See the exact type and module contract. |
| `icon` | `const Icon &` | Input | Value supplied for `icon`. See the exact type and module contract. |
| `data` | `const void *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `dataLen` | `uint32_t` | Input | Value supplied for `dataLen`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need asynchronous Memory Card access and files and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/memory-card-filesystem.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Port port
// const char * name
// const char * title
// const Icon & icon
// const void * data
// uint32_t dataLen

psyqo::MemoryCardFileSystem& object = /* obtain a valid instance */;

auto result = object.writeFileBlocking(gpu, port, name, title, icon, data, dataLen);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
