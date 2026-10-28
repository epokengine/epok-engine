# Epok API: Data Table

> **Header:** `"data_table.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/data_table.hpp)

This module covers the data table module. It documents 6 public callables declared directly in this header.

## Declared types

`epok::assets::Kind`

## Callable index

- [`epok::assets::audio_clip`](#epok-assets-audio-clip-1) — Performs `audio clip` as part of the data table module.
- [`epok::assets::cast`](#epok-assets-cast-1) — Performs `cast` as part of the data table module.
- [`epok::assets::find`](#epok-assets-find-1) — Persistent authoring UUIDs are cooked to 64-bit handles.
- [`epok::bp::asset_index`](#epok-bp-asset-index-1) — Performs `asset index` as part of the data table module.
- [`epok::bp::data_table_cast`](#epok-bp-data-table-cast-1) — Performs `data table cast` as part of the data table module.
- [`epok::bp::data_table_find`](#epok-bp-data-table-find-1) — Performs `data table find` as part of the data table module.

<a id="epok-assets-audio-clip-1"></a>

## `epok::assets::audio_clip`

**Purpose.** Performs `audio clip` as part of the data table module.

**Exact declaration**

```cpp
int audio_clip(uint64_t asset)
```

- **Declared at:** [line 15](../../../runtime/data_table.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the data table module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t asset

auto result = epok::assets::audio_clip(asset);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-assets-cast-1"></a>

## `epok::assets::cast`

**Purpose.** Performs `cast` as part of the data table module.

**Exact declaration**

```cpp
uint64_t cast(uint64_t asset,Kind kind)
```

- **Declared at:** [line 14](../../../runtime/data_table.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `kind` | `Kind` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the data table module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t asset
// Kind kind

auto result = epok::assets::cast(asset, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the data table module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-assets-find-1"></a>

## `epok::assets::find`

**Purpose.** Persistent authoring UUIDs are cooked to 64-bit handles.

**Details.** Missing/wrong-kind references return zero; they never alias asset index zero.

**Exact declaration**

```cpp
uint64_t find(uint64_t table,const char* key,Kind kind=Kind::Any)
```

- **Declared at:** [line 13](../../../runtime/data_table.hpp#L13)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `uint64_t` | Input | Value supplied for `table`. See the exact type and module contract. |
| `key` | `const char *` | Input | Value supplied for `key`. See the exact type and module contract. |
| `kind` | `Kind` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** Missing/wrong-kind references return zero; they never alias asset index zero.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t table
// const char * key
// Kind kind

auto result = epok::assets::find(table, key, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the data table module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-asset-index-1"></a>

## `epok::bp::asset_index`

**Purpose.** Performs `asset index` as part of the data table module.

**Exact declaration**

```cpp
int asset_index(uint64_t asset,uint32_t kind)
```

- **Declared at:** [line 5](../../../runtime/data_table.hpp#L5)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `kind` | `uint32_t` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the data table module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t asset
// uint32_t kind

auto result = epok::bp::asset_index(asset, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the data table module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-data-table-cast-1"></a>

## `epok::bp::data_table_cast`

**Purpose.** Performs `data table cast` as part of the data table module.

**Exact declaration**

```cpp
uint64_t data_table_cast(uint64_t asset,uint32_t kind)
```

- **Declared at:** [line 7](../../../runtime/data_table.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `kind` | `uint32_t` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the data table module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t asset
// uint32_t kind

auto result = epok::bp::data_table_cast(asset, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the data table module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-data-table-find-1"></a>

## `epok::bp::data_table_find`

**Purpose.** Performs `data table find` as part of the data table module.

**Exact declaration**

```cpp
uint64_t data_table_find(uint64_t table,const char* key,uint32_t kind=0)
```

- **Declared at:** [line 6](../../../runtime/data_table.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `uint64_t` | Input | Value supplied for `table`. See the exact type and module contract. |
| `key` | `const char *` | Input | Value supplied for `key`. See the exact type and module contract. |
| `kind` | `uint32_t` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the data table module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "data_table.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t table
// const char * key
// uint32_t kind

auto result = epok::bp::data_table_find(table, key, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the data table module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
