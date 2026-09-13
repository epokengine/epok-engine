# Epok API: Frame Clear

> **Header:** `"frame_clear.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/frame_clear.hpp)

This module covers the frame clear module. It documents 2 public callables declared directly in this header.

## Declared types

`epok::FrameClear`

## Callable index

- [`epok::configure_display_field`](#epok-configure-display-field-1) — Performs `configure display field` as part of the frame clear module.
- [`epok::FrameClear::draw`](#epok-frameclear-draw-1) — Draws draw as part of the frame clear module.

<a id="epok-configure-display-field-1"></a>

## `epok::configure_display_field`

**Purpose.** Performs `configure display field` as part of the frame clear module.

**Exact declaration**

```cpp
template<bool Interlaced> inline void configure_display_field(psyqo::PrimPieces::TPageAttr& attr)
```

- **Declared at:** [line 9](../../../runtime/frame_clear.hpp#L9)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `attr` | `psyqo::PrimPieces::TPageAttr &` | Input/output; inspect the function contract | Value supplied for `attr`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the frame clear module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frame_clear.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Interlaced

// Assume these named values have been initialized with valid data:
// psyqo::PrimPieces::TPageAttr & attr

epok::configure_display_field<Interlaced>(attr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-frameclear-draw-1"></a>

## `epok::FrameClear::draw`

**Purpose.** Draws draw as part of the frame clear module.

**Exact declaration**

```cpp
template<class Gpu> void draw(Gpu& gpu, psyqo::Color color)
```

- **Declared at:** [line 22](../../../runtime/frame_clear.hpp#L22)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `Gpu &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `color` | `psyqo::Color` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the frame clear module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "frame_clear.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Gpu

// Assume these named values have been initialized with valid data:
// Gpu & gpu
// psyqo::Color color

epok::FrameClear& object = /* obtain a valid instance */;

object.draw<Gpu>(gpu, color);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
