# Epok API: Scene Files

> **Header:** `"scene_files.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/scene_files.hpp)

This module covers scene lifecycle and scene-stack control. It documents 15 public callables declared directly in this header.

## Declared types

`epok::SceneFileError`, `epok::SceneFileLoad`, `epok::SceneFileState`, `epok::SceneFileStats`

## Callable index

- [`epok::scene_bank_external`](#epok-scene-bank-external-1) — Performs `scene bank external` as part of scene lifecycle and scene-stack control.
- [`epok::scene_bank_ready`](#epok-scene-bank-ready-1) — Performs `scene bank ready` as part of scene lifecycle and scene-stack control.
- [`epok::scene_file_begin`](#epok-scene-file-begin-1) — Start loading the file of an external bank.
- [`epok::scene_file_boot`](#epok-scene-file-boot-1) — At boot, an external startup bank waits for its file like a transition.
- [`epok::scene_file_fail`](#epok-scene-file-fail-1) — Performs `scene file fail` as part of scene lifecycle and scene-stack control.
- [`epok::scene_file_progress`](#epok-scene-file-progress-1) — Fraction of the current file verified so far, 0..4096 (Q12).
- [`epok::scene_file_publish`](#epok-scene-file-publish-1) — Performs `scene file publish` as part of scene lifecycle and scene-stack control.
- [`epok::scene_file_release_drive`](#epok-scene-file-release-drive-1) — Performs `scene file release drive` as part of scene lifecycle and scene-stack control.
- [`epok::scene_file_retry`](#epok-scene-file-retry-1) — After a failure, start the same file again once no read owns the region.
- [`epok::scene_file_service`](#epok-scene-file-service-1) — Advance the current load.
- [`epok::scene_file_waiting`](#epok-scene-file-waiting-1) — Performs `scene file waiting` as part of scene lifecycle and scene-stack control.
- [`epok::scene_files_prepare`](#epok-scene-files-prepare-1) — Performs `scene files prepare` as part of scene lifecycle and scene-stack control.
- [`epok::scene_region`](#epok-scene-region-1) — Performs `scene region` as part of scene lifecycle and scene-stack control.
- [`epok::scene_region_bytes`](#epok-scene-region-bytes-1) — Performs `scene region bytes` as part of scene lifecycle and scene-stack control.
- [`epok::SceneFileLoad::pending`](#epok-scenefileload-pending-1) — A CD callback still owns the region (and its directory entry).

<a id="epok-scene-bank-external-1"></a>

## `epok::scene_bank_external`

**Purpose.** Performs `scene bank external` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline bool scene_bank_external(size_t bank)
```

- **Declared at:** [line 50](../../../runtime/scene_files.hpp#L50)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `size_t` | Input | Value supplied for `bank`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// size_t bank

auto result = epok::scene_bank_external(bank);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scene-bank-ready-1"></a>

## `epok::scene_bank_ready`

**Purpose.** Performs `scene bank ready` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline bool scene_bank_ready(size_t bank)
```

- **Declared at:** [line 53](../../../runtime/scene_files.hpp#L53)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `size_t` | Input | Value supplied for `bank`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// size_t bank

auto result = epok::scene_bank_ready(bank);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scene-file-begin-1"></a>

## `epok::scene_file_begin`

**Purpose.** Start loading the file of an external bank.

**Details.** The region is no longer valid for any other bank from this point on, even if the load later fails.

**Exact declaration**

```cpp
inline void scene_file_begin(size_t bank, uint32_t now)
```

- **Declared at:** [line 99](../../../runtime/scene_files.hpp#L99)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `size_t` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The region is no longer valid for any other bank from this point on, even if the load later fails.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// size_t bank
// uint32_t now

epok::scene_file_begin(bank, now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-boot-1"></a>

## `epok::scene_file_boot`

**Purpose.** At boot, an external startup bank waits for its file like a transition.

**Exact declaration**

```cpp
inline void scene_file_boot(uint32_t now)
```

- **Declared at:** [line 241](../../../runtime/scene_files.hpp#L241)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::scene_file_boot(now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-fail-1"></a>

## `epok::scene_file_fail`

**Purpose.** Performs `scene file fail` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline void scene_file_fail(SceneFileError error)
```

- **Declared at:** [line 80](../../../runtime/scene_files.hpp#L80)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `error` | `SceneFileError` | Input | Value supplied for `error`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// SceneFileError error

epok::scene_file_fail(error);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-progress-1"></a>

## `epok::scene_file_progress`

**Purpose.** Fraction of the current file verified so far, 0..4096 (Q12).

**Details.** Real bytes, not a timer; a bank already in the region reports 4096.

**Exact declaration**

```cpp
inline uint32_t scene_file_progress()
```

- **Declared at:** [line 61](../../../runtime/scene_files.hpp#L61)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Real bytes, not a timer; a bank already in the region reports 4096.

**Usage pattern**

```cpp
#include "scene_files.hpp"

auto result = epok::scene_file_progress();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-publish-1"></a>

## `epok::scene_file_publish`

**Purpose.** Performs `scene file publish` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline void scene_file_publish(uint32_t now)
```

- **Declared at:** [line 117](../../../runtime/scene_files.hpp#L117)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::scene_file_publish(now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-release-drive-1"></a>

## `epok::scene_file_release_drive`

**Purpose.** Performs `scene file release drive` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline void scene_file_release_drive()
```

- **Declared at:** [line 73](../../../runtime/scene_files.hpp#L73)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

epok::scene_file_release_drive();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-retry-1"></a>

## `epok::scene_file_retry`

**Purpose.** After a failure, start the same file again once no read owns the region.

**Exact declaration**

```cpp
inline bool scene_file_retry(uint32_t now)
```

- **Declared at:** [line 229](../../../runtime/scene_files.hpp#L229)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

auto result = epok::scene_file_retry(now);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scene-file-service-1"></a>

## `epok::scene_file_service`

**Purpose.** Advance the current load.

**Details.** Call once per frame while scene_file_waiting().

**Exact declaration**

```cpp
inline void scene_file_service(uint32_t now)
```

- **Declared at:** [line 134](../../../runtime/scene_files.hpp#L134)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Call once per frame while scene_file_waiting().

**Usage pattern**

```cpp
#include "scene_files.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::scene_file_service(now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-file-waiting-1"></a>

## `epok::scene_file_waiting`

**Purpose.** Performs `scene file waiting` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline bool scene_file_waiting()
```

- **Declared at:** [line 58](../../../runtime/scene_files.hpp#L58)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

auto result = epok::scene_file_waiting();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scene-files-prepare-1"></a>

## `epok::scene_files_prepare`

**Purpose.** Performs `scene files prepare` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline void scene_files_prepare()
```

- **Declared at:** [line 67](../../../runtime/scene_files.hpp#L67)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

epok::scene_files_prepare();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-region-1"></a>

## `epok::scene_region`

**Purpose.** Performs `scene region` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline uint8_t* scene_region()
```

- **Declared at:** [line 46](../../../runtime/scene_files.hpp#L46)
- **Kind:** `function decl`

**Returns.** Returns `uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

auto result = epok::scene_region();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-region-bytes-1"></a>

## `epok::scene_region_bytes`

**Purpose.** Performs `scene region bytes` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline uint32_t scene_region_bytes()
```

- **Declared at:** [line 47](../../../runtime/scene_files.hpp#L47)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

auto result = epok::scene_region_bytes();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenefileload-pending-1"></a>

## `epok::SceneFileLoad::pending`

**Purpose.** A CD callback still owns the region (and its directory entry).

**Exact declaration**

```cpp
bool pending() const
```

- **Declared at:** [line 35](../../../runtime/scene_files.hpp#L35)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_files.hpp"

epok::SceneFileLoad& object = /* obtain a valid instance */;

auto result = object.pending();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
