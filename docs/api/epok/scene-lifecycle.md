# Epok API: Scene Lifecycle

> **Header:** `"scene_lifecycle.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/scene_lifecycle.hpp)

This module covers entity creation, activation and destruction. It documents 3 public callables declared directly in this header.

## Declared types

`epok::SceneStats`

## Callable index

- [`epok::scene_retire_bank`](#epok-scene-retire-bank-1) — Consume before callbacks: end-play and begin-play may queue the next change.
- [`epok::scene_start_bank`](#epok-scene-start-bank-1) — Performs `scene start bank` as part of entity creation, activation and destruction.
- [`epok::scene_stop_audio`](#epok-scene-stop-audio-1) — Performs `scene stop audio` as part of entity creation, activation and destruction.

<a id="epok-scene-retire-bank-1"></a>

## `epok::scene_retire_bank`

**Purpose.** Consume before callbacks: end-play and begin-play may queue the next change.

**Exact declaration**

```cpp
inline size_t scene_retire_bank()
```

- **Declared at:** [line 44](../../../runtime/scene_lifecycle.hpp#L44)
- **Kind:** `function decl`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_lifecycle.hpp"

auto result = epok::scene_retire_bank();
```

**Why choose it.** It provides direct, allocation-conscious access to entity creation, activation and destruction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-start-bank-1"></a>

## `epok::scene_start_bank`

**Purpose.** Performs `scene start bank` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline void scene_start_bank(size_t next)
```

- **Declared at:** [line 62](../../../runtime/scene_lifecycle.hpp#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `next` | `size_t` | Input | Value supplied for `next`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// size_t next

epok::scene_start_bank(next);
```

**Why choose it.** It provides direct, allocation-conscious access to entity creation, activation and destruction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scene-stop-audio-1"></a>

## `epok::scene_stop_audio`

**Purpose.** Performs `scene stop audio` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline void scene_stop_audio()
```

- **Declared at:** [line 38](../../../runtime/scene_lifecycle.hpp#L38)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_lifecycle.hpp"

epok::scene_stop_audio();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
