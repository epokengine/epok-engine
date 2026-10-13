# Epok API: World2D Render

> **Header:** `"world2d_render.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_render.hpp)

This module covers the world2d render module. It documents 1 public callable declared directly in this header.

## Declared types

`epok::world2d::Renderer`

## Callable index

- [`epok::world2d::Renderer::draw`](#epok-world2d-renderer-draw-1) — Builds and chains this frame's 2D packets.

<a id="epok-world2d-renderer-draw-1"></a>

## `epok::world2d::Renderer::draw`

**Purpose.** Builds and chains this frame's 2D packets.

**Details.** Call after the 3D ordering table and before the HUD.

**Exact declaration**

```cpp
void draw(psyqo::GPU& gpu)
```

- **Declared at:** [line 301](../../../runtime/world2d_render.hpp#L301)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Call after the 3D ordering table and before the HUD.

**Usage pattern**

```cpp
#include "world2d_render.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::world2d::Renderer& object = /* obtain a valid instance */;

object.draw(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
