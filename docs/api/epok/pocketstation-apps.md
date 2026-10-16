# Epok API: Pocketstation Apps

> **Header:** `"pocketstation_apps.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/pocketstation_apps.hpp)

This module covers the pocketstation apps module. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::bp::pocket_app_index`](#epok-bp-pocket-app-index-1) — Performs `pocket app index` as part of the pocketstation apps module.
- [`epok::find_pocket_app`](#epok-find-pocket-app-1) — The embedded program with this memory card file name, or nullptr.
- [`epok::pocket_app_at`](#epok-pocket-app-at-1) — Performs `pocket app at` as part of the pocketstation apps module.

<a id="epok-bp-pocket-app-index-1"></a>

## `epok::bp::pocket_app_index`

**Purpose.** Performs `pocket app index` as part of the pocketstation apps module.

**Exact declaration**

```cpp
inline int pocket_app_index(uint64_t)
```

- **Declared at:** [line 16](../../../runtime/pocketstation_apps.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint64_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation apps module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_apps.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t arg1

auto result = epok::bp::pocket_app_index(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation apps module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-find-pocket-app-1"></a>

## `epok::find_pocket_app`

**Purpose.** The embedded program with this memory card file name, or nullptr.

**Exact declaration**

```cpp
inline const PocketApp* find_pocket_app(const char* filename)
```

- **Declared at:** [line 23](../../../runtime/pocketstation_apps.hpp#L23)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `filename` | `const char *` | Input | Value supplied for `filename`. See the exact type and module contract. |

**Returns.** Returns `const PocketApp *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation apps module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_apps.hpp"

// Assume these named values have been initialized with valid data:
// const char * filename

auto result = epok::find_pocket_app(filename);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation apps module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pocket-app-at-1"></a>

## `epok::pocket_app_at`

**Purpose.** Performs `pocket app at` as part of the pocketstation apps module.

**Exact declaration**

```cpp
inline const PocketApp* pocket_app_at(size_t)
```

- **Declared at:** [line 14](../../../runtime/pocketstation_apps.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `size_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `const PocketApp *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the pocketstation apps module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "pocketstation_apps.hpp"

// Assume these named values have been initialized with valid data:
// size_t arg1

auto result = epok::pocket_app_at(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the pocketstation apps module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
