# Epok API: Palette

> **Header:** `"palette.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/palette.hpp)

This module covers CLUT animation and palette ownership. It documents 2 public callables declared directly in this header.

## Declared types

`epok::PaletteRenderer`

## Callable index

- [`epok::PaletteRenderer::clear`](#epok-paletterenderer-clear-1) — Clears clear as part of CLUT animation and palette ownership.
- [`epok::PaletteRenderer::upload`](#epok-paletterenderer-upload-1) — Call before chaining this frame's geometry.

<a id="epok-paletterenderer-clear-1"></a>

## `epok::PaletteRenderer::clear`

**Purpose.** Clears clear as part of CLUT animation and palette ownership.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 11](../../../runtime/palette.hpp#L11)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need CLUT animation and palette ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "palette.hpp"

epok::PaletteRenderer& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-paletterenderer-upload-1"></a>

## `epok::PaletteRenderer::upload`

**Purpose.** Call before chaining this frame's geometry.

**Details.** Blocking uploads retain no borrowed scratch pointer; wait once before changing any in-flight CLUT.

**Exact declaration**

```cpp
template<class Objects> void upload(psyqo::GPU& gpu,const Objects& objects,size_t count)
```

- **Declared at:** [line 14](../../../runtime/palette.hpp#L14)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Blocking uploads retain no borrowed scratch pointer; wait once before changing any in-flight CLUT.

**Usage pattern**

```cpp
#include "palette.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu
// const Objects & objects
// size_t count

epok::PaletteRenderer& object = /* obtain a valid instance */;

object.upload<Objects>(gpu, objects, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
