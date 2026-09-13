# Epok API: Effects

> **Header:** `"effects.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/effects.hpp)

This module covers runtime effects and their deterministic playback state. It documents 2 public callables declared directly in this header.

## Declared types

`epok::FogEnvironment`, `epok::UvVertex`

## Callable index

- [`epok::fog_color`](#epok-fog-color-1) — Performs `fog color` as part of runtime effects and their deterministic playback state.
- [`epok::scroll_triangle`](#epok-scroll-triangle-1) — Split at texture wrap boundaries before converting UV to 8-bit page pixels.

<a id="epok-fog-color-1"></a>

## `epok::fog_color`

**Purpose.** Performs `fog color` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
template<class Color>inline Color fog_color(Color color,int32_t depth)
```

- **Declared at:** [line 9](../../../runtime/effects.hpp#L9)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `depth` | `int32_t` | Input | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "effects.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Color

// Assume these named values have been initialized with valid data:
// Color color
// int32_t depth

auto result = epok::fog_color<Color>(color, depth);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-scroll-triangle-1"></a>

## `epok::scroll_triangle`

**Purpose.** Split at texture wrap boundaries before converting UV to 8-bit page pixels.

**Details.** At most four cells and bounded eight-vertex clip buffers, no heap allocation.

**Exact declaration**

```cpp
template<class Emit>void scroll_triangle(const UvVertex* input,const int32_t* speed,uint32_t ticks,Emit emit)
```

- **Declared at:** [line 17](../../../runtime/effects.hpp#L17)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `input` | `const UvVertex *` | Input | Value supplied for `input`. See the exact type and module contract. |
| `speed` | `const int32_t *` | Input | Value supplied for `speed`. See the exact type and module contract. |
| `ticks` | `uint32_t` | Input | Value supplied for `ticks`. See the exact type and module contract. |
| `emit` | `Emit` | Input | Value supplied for `emit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** At most four cells and bounded eight-vertex clip buffers, no heap allocation.

**Usage pattern**

```cpp
#include "effects.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Emit

// Assume these named values have been initialized with valid data:
// const UvVertex * input
// const int32_t * speed
// uint32_t ticks
// Emit emit

epok::scroll_triangle<Emit>(input, speed, ticks, emit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
