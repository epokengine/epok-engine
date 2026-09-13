# PsyQo API: Iso9660 Parser

> **Header:** `"psyqo/iso9660-parser.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh)

This module covers the iso9660 parser module. It documents 18 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::ISO9660Parser`, `psyqo::ISO9660Parser::DirEntry`, `psyqo::ISO9660Parser::DirEntry::(unnamed enum at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\iso9660-parser.hh:60:9)`, `psyqo::ISO9660Parser::GetDirentryAwaiter`, `psyqo::ISO9660Parser::InitializeAwaiter`, `psyqo::ISO9660Parser::ReadRequest`

## Callable index

- [`psyqo::ISO9660Parser::getCDRom`](#psyqo-iso9660parser-getcdrom-1) — Returns the CDRom object used by the parser.
- [`psyqo::ISO9660Parser::getDirentry`](#psyqo-iso9660parser-getdirentry-1) — Returns direntry as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::getDirentry`](#psyqo-iso9660parser-getdirentry-2) — Get the Direntry object for a given path.
- [`psyqo::ISO9660Parser::GetDirentryAwaiter::await_ready`](#psyqo-iso9660parser-getdirentryawaiter-await-ready-1) — Performs `await ready` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::GetDirentryAwaiter::await_resume`](#psyqo-iso9660parser-getdirentryawaiter-await-resume-1) — Performs `await resume` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::GetDirentryAwaiter::await_suspend`](#psyqo-iso9660parser-getdirentryawaiter-await-suspend-1) — Performs `await suspend` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::GetDirentryAwaiter::GetDirentryAwaiter`](#psyqo-iso9660parser-getdirentryawaiter-getdirentryawaiter-1) — Constructs `psyqo::ISO9660Parser::GetDirentryAwaiter` for the iso9660 parser module.
- [`psyqo::ISO9660Parser::initialize`](#psyqo-iso9660parser-initialize-1) — Performs `initialize` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::initialize`](#psyqo-iso9660parser-initialize-2) — Initializes the parser.
- [`psyqo::ISO9660Parser::InitializeAwaiter::await_ready`](#psyqo-iso9660parser-initializeawaiter-await-ready-1) — Performs `await ready` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::InitializeAwaiter::await_resume`](#psyqo-iso9660parser-initializeawaiter-await-resume-1) — Performs `await resume` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::InitializeAwaiter::await_suspend`](#psyqo-iso9660parser-initializeawaiter-await-suspend-1) — Performs `await suspend` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::InitializeAwaiter::InitializeAwaiter`](#psyqo-iso9660parser-initializeawaiter-initializeawaiter-1) — Constructs `psyqo::ISO9660Parser::InitializeAwaiter` for the iso9660 parser module.
- [`psyqo::ISO9660Parser::initialized`](#psyqo-iso9660parser-initialized-1) — Returns the state of the parser.
- [`psyqo::ISO9660Parser::ISO9660Parser`](#psyqo-iso9660parser-iso9660parser-1) — The ISO9660Parser constructor.
- [`psyqo::ISO9660Parser::scheduleGetDirentry`](#psyqo-iso9660parser-schedulegetdirentry-1) — Performs `schedule get direntry` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::scheduleInitialize`](#psyqo-iso9660parser-scheduleinitialize-1) — Performs `schedule initialize` as part of the iso9660 parser module.
- [`psyqo::ISO9660Parser::scheduleReadRequest`](#psyqo-iso9660parser-schedulereadrequest-1) — Read a file asynchronously.

<a id="psyqo-iso9660parser-getcdrom-1"></a>

## `psyqo::ISO9660Parser::getCDRom`

**Purpose.** Returns the CDRom object used by the parser.

**Exact declaration**

```cpp
CDRom* getCDRom()
```

- **Declared at:** [line 183](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L183)
- **Kind:** `cxx method`

**Returns.** The CDRom object.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.getCDRom();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-iso9660parser-getdirentry-1"></a>

## `psyqo::ISO9660Parser::getDirentry`

**Purpose.** Returns direntry as part of the iso9660 parser module.

**Exact declaration**

```cpp
GetDirentryAwaiter getDirentry(eastl::string_view path, DirEntry* entry)
```

- **Declared at:** [line 150](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L150)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `path` | `eastl::string_view` | Input | Value supplied for `path`. See the exact type and module contract. |
| `entry` | `DirEntry *` | Input/output; inspect the function contract | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `GetDirentryAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// eastl::string_view path
// DirEntry * entry

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.getDirentry(path, entry);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-getdirentry-2"></a>

## `psyqo::ISO9660Parser::getDirentry`

**Purpose.** Get the Direntry object for a given path.

**Details.** This method looks up the directory entry for a given path. It will fail if the CDRom device fails reading the disk, or if the parser hasn't been initialized yet. If the directory entry is not found, the callback or task will still be successful, but the directory entry will be invalid.

**Exact declaration**

```cpp
void getDirentry(eastl::string_view path, DirEntry* entry, eastl::function<void(bool success)> callback)
```

- **Declared at:** [line 129](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L129)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `path` | `eastl::string_view` | Input | The path to look for. |
| `entry` | `DirEntry *` | Input/output; inspect the function contract | The DirEntry object to fill. |
| `callback` | `eastl::function<void (bool)>` | Callback | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method looks up the directory entry for a given path. It will fail if the CDRom device fails reading the disk, or if the parser hasn't been initialized yet. If the directory entry is not found, the callback or task will still be successful, but the directory entry will be invalid.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// eastl::string_view path
// DirEntry * entry
// eastl::function<void (bool)> callback

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

object.getDirentry(path, entry, callback);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-getdirentryawaiter-await-ready-1"></a>

## `psyqo::ISO9660Parser::GetDirentryAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the iso9660 parser module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 134](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser::GetDirentryAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-iso9660parser-getdirentryawaiter-await-resume-1"></a>

## `psyqo::ISO9660Parser::GetDirentryAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the iso9660 parser module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 142](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L142)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser::GetDirentryAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-iso9660parser-getdirentryawaiter-await-suspend-1"></a>

## `psyqo::ISO9660Parser::GetDirentryAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the iso9660 parser module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 136](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L136)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::ISO9660Parser::GetDirentryAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-iso9660parser-getdirentryawaiter-getdirentryawaiter-1"></a>

## `psyqo::ISO9660Parser::GetDirentryAwaiter::GetDirentryAwaiter`

**Purpose.** Constructs `psyqo::ISO9660Parser::GetDirentryAwaiter` for the iso9660 parser module.

**Exact declaration**

```cpp
GetDirentryAwaiter(ISO9660Parser& parser, eastl::string_view path, DirEntry* entry)
```

- **Declared at:** [line 132](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L132)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parser` | `ISO9660Parser &` | Input/output; inspect the function contract | Value supplied for `parser`. See the exact type and module contract. |
| `path` | `eastl::string_view` | Input | Value supplied for `path`. See the exact type and module contract. |
| `entry` | `DirEntry *` | Input/output; inspect the function contract | Value supplied for `entry`. See the exact type and module contract. |

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// ISO9660Parser & parser
// eastl::string_view path
// DirEntry * entry

psyqo::ISO9660Parser::GetDirentryAwaiter value(parser, path, entry);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-initialize-1"></a>

## `psyqo::ISO9660Parser::initialize`

**Purpose.** Performs `initialize` as part of the iso9660 parser module.

**Exact declaration**

```cpp
InitializeAwaiter initialize()
```

- **Declared at:** [line 116](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L116)
- **Kind:** `cxx method`

**Returns.** Returns `InitializeAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.initialize();
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-iso9660parser-initialize-2"></a>

## `psyqo::ISO9660Parser::initialize`

**Purpose.** Initializes the parser.

**Details.** This method initializes the basic internal structures of the parser. It must be called before any other method. If the underlying CDRom device fails reading, or if the filesystem is not an ISO9660, the callback or task will fail. It can be called multiple times, and has to be called when the user changes the disc in the drive.

**Exact declaration**

```cpp
void initialize(eastl::function<void(bool success)> callback)
```

- **Declared at:** [line 98](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L98)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (bool)>` | Callback | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method initializes the basic internal structures of the parser. It must be called before any other method. If the underlying CDRom device fails reading, or if the filesystem is not an ISO9660, the callback or task will fail. It can be called multiple times, and has to be called when the user changes the disc in the drive.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (bool)> callback

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

object.initialize(callback);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-iso9660parser-initializeawaiter-await-ready-1"></a>

## `psyqo::ISO9660Parser::InitializeAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the iso9660 parser module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 102](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L102)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser::InitializeAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-iso9660parser-initializeawaiter-await-resume-1"></a>

## `psyqo::ISO9660Parser::InitializeAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the iso9660 parser module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L110)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser::InitializeAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-iso9660parser-initializeawaiter-await-suspend-1"></a>

## `psyqo::ISO9660Parser::InitializeAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the iso9660 parser module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 104](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L104)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::ISO9660Parser::InitializeAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-iso9660parser-initializeawaiter-initializeawaiter-1"></a>

## `psyqo::ISO9660Parser::InitializeAwaiter::InitializeAwaiter`

**Purpose.** Constructs `psyqo::ISO9660Parser::InitializeAwaiter` for the iso9660 parser module.

**Exact declaration**

```cpp
InitializeAwaiter(ISO9660Parser& parser) : m_par
```

- **Declared at:** [line 101](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L101)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parser` | `ISO9660Parser &` | Input/output; inspect the function contract | Value supplied for `parser`. See the exact type and module contract. |

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// ISO9660Parser & parser

psyqo::ISO9660Parser::InitializeAwaiter value(parser);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-initialized-1"></a>

## `psyqo::ISO9660Parser::initialized`

**Purpose.** Returns the state of the parser.

**Details.** This method returns true if the parser was initialized successfully, and false otherwise.

**Exact declaration**

```cpp
bool initialized()
```

- **Declared at:** [line 176](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L176)
- **Kind:** `cxx method`

**Returns.** The root directory entry.

**Use it when.** This method returns true if the parser was initialized successfully, and false otherwise.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.initialized();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-iso9660parser-iso9660parser-1"></a>

## `psyqo::ISO9660Parser::ISO9660Parser`

**Purpose.** The ISO9660Parser constructor.

**Details.** This constructor takes a CDRom device as a parameter. It will use that device to read the structure of the ISO9660 filesystem.

**Exact declaration**

```cpp
ISO9660Parser(CDRom* cdrom) : m
```

- **Declared at:** [line 87](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L87)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cdrom` | `CDRom *` | Input/output; inspect the function contract | The CDRom device to use. |

**Use it when.** This constructor takes a CDRom device as a parameter. It will use that device to read the structure of the ISO9660 filesystem.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// CDRom * cdrom

psyqo::ISO9660Parser value(cdrom);
```

**Why choose it.** It provides direct, allocation-conscious access to the iso9660 parser module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-schedulegetdirentry-1"></a>

## `psyqo::ISO9660Parser::scheduleGetDirentry`

**Purpose.** Performs `schedule get direntry` as part of the iso9660 parser module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleGetDirentry(eastl::string_view path, DirEntry* entry)
```

- **Declared at:** [line 130](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `path` | `eastl::string_view` | Input | Value supplied for `path`. See the exact type and module contract. |
| `entry` | `DirEntry *` | Input/output; inspect the function contract | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// eastl::string_view path
// DirEntry * entry

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.scheduleGetDirentry(path, entry);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-iso9660parser-scheduleinitialize-1"></a>

## `psyqo::ISO9660Parser::scheduleInitialize`

**Purpose.** Performs `schedule initialize` as part of the iso9660 parser module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleInitialize()
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L99)
- **Kind:** `cxx method`

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the iso9660 parser module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.scheduleInitialize();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-iso9660parser-schedulereadrequest-1"></a>

## `psyqo::ISO9660Parser::scheduleReadRequest`

**Purpose.** Read a file asynchronously.

**Details.** This method reads a file asynchronously. It will read the file from the given `entry` in the `ReadRequest`, and will read the number of sectors corresponding to the entry's size. The buffer is specified by the `buffer` field in the `ReadRequest`, and must be large enough to hold the whole file. This method is mainly a helper around the CDRom device's `readSectors` method.

**Exact declaration**

```cpp
TaskQueue::Task scheduleReadRequest(const ReadRequest* request)
```

- **Declared at:** [line 166](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/iso9660-parser.hh#L166)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `request` | `const ReadRequest *` | Input | The request to fill. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** This method reads a file asynchronously. It will read the file from the given `entry` in the `ReadRequest`, and will read the number of sectors corresponding to the entry's size. The buffer is specified by the `buffer` field in the `ReadRequest`, and must be large enough to hold the whole file. This method is mainly a helper around the CDRom device's `readSectors` method.

**Usage pattern**

```cpp
#include "psyqo/iso9660-parser.hh"

// Assume these named values have been initialized with valid data:
// const ReadRequest * request

psyqo::ISO9660Parser& object = /* obtain a valid instance */;

auto result = object.scheduleReadRequest(request);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
