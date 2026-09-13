# Epok API: Scene Service

> **Header:** `"scene_service.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/scene_service.hpp)

This module covers scene lifecycle and scene-stack control. It documents 1 public callable declared directly in this header.

## Declared types

`epok::SceneStats`

## Callable index

- [`epok::scene_tick`](#epok-scene-tick-1) — Performs `scene tick` as part of scene lifecycle and scene-stack control.

<a id="epok-scene-tick-1"></a>

## `epok::scene_tick`

**Purpose.** Performs `scene tick` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline bool scene_tick(psyqo::GPU& gpu)
```

- **Declared at:** [line 38](../../../runtime/scene_service.hpp#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_service.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

auto result = epok::scene_tick(gpu);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
