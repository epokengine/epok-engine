# Epok API: Loading Renderer

> **Header:** `"loading_renderer.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/loading_renderer.hpp)

This module covers loading-screen rendering and transitions. It documents 1 public callable declared directly in this header.

## Declared types

`epok::LoadingRenderer`

## Callable index

- [`epok::LoadingRenderer::draw`](#epok-loadingrenderer-draw-1) — Draws draw as part of loading-screen rendering and transitions.

<a id="epok-loadingrenderer-draw-1"></a>

## `epok::LoadingRenderer::draw`

**Purpose.** Draws draw as part of loading-screen rendering and transitions.

**Exact declaration**

```cpp
void draw(psyqo::GPU& gpu)
```

- **Declared at:** [line 18](../../../runtime/loading_renderer.hpp#L18)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need loading-screen rendering and transitions and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "loading_renderer.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::LoadingRenderer& object = /* obtain a valid instance */;

object.draw(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
