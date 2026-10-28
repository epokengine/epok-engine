# Epok API: World2D Render

> **Header:** `"world2d_render.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_render.hpp)

This module covers the world2d render module. It documents 4 public callables declared directly in this header.

## Declared types

`epok::world2d::DepthTable`, `epok::world2d::PagedRectangle`, `epok::world2d::Renderer`

## Callable index

- [`epok::world2d::Renderer::begin`](#epok-world2d-renderer-begin-1) — Starts this frame's 2D packets: draws the SubViewport2D pictures into VRAM, then collects the screen view.
- [`epok::world2d::Renderer::draw_over`](#epok-world2d-renderer-draw-over-1) — Chains everything else: the halves of a split screen, then the screen view's canvases over the 3D world.
- [`epok::world2d::Renderer::draw_under`](#epok-world2d-renderer-draw-under-1) — Chains the under_3d canvases of the screen view.
- [`epok::world2d::Renderer::insert_depth`](#epok-world2d-renderer-insert-depth-1) — Inserts the screen view's Depth2D entries into the 3D ordering table, each at the bucket `bucket_of(depth2d)` returns.

<a id="epok-world2d-renderer-begin-1"></a>

## `epok::world2d::Renderer::begin`

**Purpose.** Starts this frame's 2D packets: draws the SubViewport2D pictures into VRAM, then collects the screen view.

**Details.** With `with_3d` (a 3D camera is active) and no split screen, Depth2D entries are kept for `insert_depth`. Call before the 3D world's packets.

**Exact declaration**

```cpp
void begin(psyqo::GPU& gpu, bool with_3d)
```

- **Declared at:** [line 356](../../../runtime/world2d_render.hpp#L356)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `with_3d` | `bool` | Input | Value supplied for `with_3d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** With `with_3d` (a 3D camera is active) and no split screen, Depth2D entries are kept for `insert_depth`. Call before the 3D world's packets.

**Usage pattern**

```cpp
#include "world2d_render.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu
// bool with_3d

epok::world2d::Renderer& object = /* obtain a valid instance */;

object.begin(gpu, with_3d);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-renderer-draw-over-1"></a>

## `epok::world2d::Renderer::draw_over`

**Purpose.** Chains everything else: the halves of a split screen, then the screen view's canvases over the 3D world.

**Details.** Call after the 3D ordering tables and before the HUD.

**Exact declaration**

```cpp
void draw_over(psyqo::GPU& gpu)
```

- **Declared at:** [line 428](../../../runtime/world2d_render.hpp#L428)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Call after the 3D ordering tables and before the HUD.

**Usage pattern**

```cpp
#include "world2d_render.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::world2d::Renderer& object = /* obtain a valid instance */;

object.draw_over(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-renderer-draw-under-1"></a>

## `epok::world2d::Renderer::draw_under`

**Purpose.** Chains the under_3d canvases of the screen view.

**Details.** Call before the 3D ordering tables.

**Exact declaration**

```cpp
void draw_under(psyqo::GPU& gpu)
```

- **Declared at:** [line 407](../../../runtime/world2d_render.hpp#L407)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Call before the 3D ordering tables.

**Usage pattern**

```cpp
#include "world2d_render.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::world2d::Renderer& object = /* obtain a valid instance */;

object.draw_under(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-renderer-insert-depth-1"></a>

## `epok::world2d::Renderer::insert_depth`

**Purpose.** Inserts the screen view's Depth2D entries into the 3D ordering table, each at the bucket `bucket_of(depth2d)` returns.

**Details.** Call before any 3D packet joins the table: a bucket draws its packets in reverse insertion order, so the 3D packets of a bucket draw first.

**Exact declaration**

```cpp
template<class BucketOf> void insert_depth(psyqo::GPU& gpu, DepthTable& target, BucketOf&& bucket_of)
```

- **Declared at:** [line 411](../../../runtime/world2d_render.hpp#L411)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `target` | `DepthTable &` | Input/output; inspect the function contract | Value supplied for `target`. See the exact type and module contract. |
| `bucket_of` | `BucketOf &&` | Consumed or moved input | Value supplied for `bucket_of`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Call before any 3D packet joins the table: a bucket draws its packets in reverse insertion order, so the 3D packets of a bucket draw first.

**Usage pattern**

```cpp
#include "world2d_render.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// BucketOf

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu
// DepthTable & target
// BucketOf && bucket_of

epok::world2d::Renderer& object = /* obtain a valid instance */;

object.insert_depth<BucketOf>(gpu, target, bucket_of);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
