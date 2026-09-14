# Epok API: Shadows

> **Header:** `"shadows.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/shadows.hpp)

This module covers static and blob shadow rendering. It documents 1 public callable declared directly in this header.

## Declared types

`epok::BlobRenderer`

## Callable index

- [`epok::BlobRenderer::draw`](#epok-blobrenderer-draw-1) — Draws draw as part of static and blob shadow rendering.

<a id="epok-blobrenderer-draw-1"></a>

## `epok::BlobRenderer::draw`

**Purpose.** Draws draw as part of static and blob shadow rendering.

**Exact declaration**

```cpp
template<size_t N,typename Table>void draw(int parity,Table& table,const std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count,const Affine<Fixed>& view)
```

- **Declared at:** [line 12](../../../runtime/shadows.hpp#L12)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `table` | `Table &` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `objects` | `const std::array<ActorData, N> &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const std::array<Affine<Fixed>, N> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `view` | `const Affine<Fixed> &` | Input | Value supplied for `view`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need static and blob shadow rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "shadows.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N, Table

// Assume these named values have been initialized with valid data:
// int parity
// Table & table
// const std::array<ActorData, N> & objects
// const std::array<Affine<Fixed>, N> & world
// size_t count
// const Affine<Fixed> & view

epok::BlobRenderer& object = /* obtain a valid instance */;

object.draw<N, Table>(parity, table, objects, world, count, view);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
