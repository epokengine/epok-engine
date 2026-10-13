# Epok API: Sky Gradient

> **Header:** `"sky_gradient.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sky_gradient.hpp)

This module covers the sky gradient module. It documents 6 public callables declared directly in this header.

## Declared types

`epok::SkyGradient`, `epok::SkyGradientColors`, `epok::SkyGradientView`

## Callable index

- [`epok::sky_gradient_color`](#epok-sky-gradient-color-1) — Performs `sky gradient color` as part of the sky gradient module.
- [`epok::sky_gradient_mix`](#epok-sky-gradient-mix-1) — Performs `sky gradient mix` as part of the sky gradient module.
- [`epok::sky_gradient_view`](#epok-sky-gradient-view-1) — basis: camera-from-world rotation (Q12 rows).
- [`epok::SkyGradient::draw`](#epok-skygradient-draw-1) — Returns false when the camera cannot be resolved; the caller then clears with the flat sky colour instead.
- [`epok::SkyGradientView::elevation`](#epok-skygradientview-elevation-1) — Sine of the ray's elevation, Q12.
- [`epok::SkyGradientView::level`](#epok-skygradientview-level-1) — Performs `level` as part of the sky gradient module.

<a id="epok-sky-gradient-color-1"></a>

## `epok::sky_gradient_color`

**Purpose.** Performs `sky gradient color` as part of the sky gradient module.

**Exact declaration**

```cpp
inline psyqo::Color sky_gradient_color(const SkyGradientColors& colors, int32_t elevation)
```

- **Declared at:** [line 73](../../../runtime/sky_gradient.hpp#L73)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `colors` | `const SkyGradientColors &` | Input | Value supplied for `colors`. See the exact type and module contract. |
| `elevation` | `int32_t` | Input | Value supplied for `elevation`. See the exact type and module contract. |

**Returns.** Returns `psyqo::Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sky gradient module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Assume these named values have been initialized with valid data:
// const SkyGradientColors & colors
// int32_t elevation

auto result = epok::sky_gradient_color(colors, elevation);
```

**Why choose it.** It provides direct, allocation-conscious access to the sky gradient module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sky-gradient-mix-1"></a>

## `epok::sky_gradient_mix`

**Purpose.** Performs `sky gradient mix` as part of the sky gradient module.

**Exact declaration**

```cpp
inline uint8_t sky_gradient_mix(uint8_t from, uint8_t to, int32_t t)
```

- **Declared at:** [line 70](../../../runtime/sky_gradient.hpp#L70)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `uint8_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `uint8_t` | Input | Value supplied for `to`. See the exact type and module contract. |
| `t` | `int32_t` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sky gradient module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t from
// uint8_t to
// int32_t t

auto result = epok::sky_gradient_mix(from, to, t);
```

**Why choose it.** It provides direct, allocation-conscious access to the sky gradient module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sky-gradient-view-1"></a>

## `epok::sky_gradient_view`

**Purpose.** basis: camera-from-world rotation (Q12 rows).

**Details.** row_scale: the projection's per-row scale (Q12, row 0 negative when mirrored). focal: GTE H in pixels.

**Exact declaration**

```cpp
template <class Basis> inline bool sky_gradient_view(const Basis& basis, int32_t row_scale_x, int32_t row_scale_y, int32_t focal, int32_t centre_x, int32_t centre_y, SkyGradientView& out)
```

- **Declared at:** [line 52](../../../runtime/sky_gradient.hpp#L52)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `basis` | `const Basis &` | Input | Value supplied for `basis`. See the exact type and module contract. |
| `row_scale_x` | `int32_t` | Input | Value supplied for `row_scale_x`. See the exact type and module contract. |
| `row_scale_y` | `int32_t` | Input | Value supplied for `row_scale_y`. See the exact type and module contract. |
| `focal` | `int32_t` | Input | Value supplied for `focal`. See the exact type and module contract. |
| `centre_x` | `int32_t` | Input | Value supplied for `centre_x`. See the exact type and module contract. |
| `centre_y` | `int32_t` | Input | Value supplied for `centre_y`. See the exact type and module contract. |
| `out` | `SkyGradientView &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** row_scale: the projection's per-row scale (Q12, row 0 negative when mirrored). focal: GTE H in pixels.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Basis

// Assume these named values have been initialized with valid data:
// const Basis & basis
// int32_t row_scale_x
// int32_t row_scale_y
// int32_t focal
// int32_t centre_x
// int32_t centre_y
// SkyGradientView & out

auto result = epok::sky_gradient_view<Basis>(basis, row_scale_x, row_scale_y, focal, centre_x, centre_y, out);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skygradient-draw-1"></a>

## `epok::SkyGradient::draw`

**Purpose.** Returns false when the camera cannot be resolved; the caller then clears with the flat sky colour instead.

**Exact declaration**

```cpp
template <class Gpu> bool draw(Gpu& gpu, const SkyGradientView& view, const SkyGradientColors& colors)
```

- **Declared at:** [line 130](../../../runtime/sky_gradient.hpp#L130)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `Gpu &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `view` | `const SkyGradientView &` | Input | Value supplied for `view`. See the exact type and module contract. |
| `colors` | `const SkyGradientColors &` | Input | Value supplied for `colors`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sky gradient module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Gpu

// Assume these named values have been initialized with valid data:
// Gpu & gpu
// const SkyGradientView & view
// const SkyGradientColors & colors

epok::SkyGradient& object = /* obtain a valid instance */;

auto result = object.draw<Gpu>(gpu, view, colors);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skygradientview-elevation-1"></a>

## `epok::SkyGradientView::elevation`

**Purpose.** Sine of the ray's elevation, Q12.

**Exact declaration**

```cpp
int32_t elevation(int32_t x, int32_t y) const
```

- **Declared at:** [line 41](../../../runtime/sky_gradient.hpp#L41)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sky gradient module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::SkyGradientView& object = /* obtain a valid instance */;

auto result = object.elevation(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skygradientview-level-1"></a>

## `epok::SkyGradientView::level`

**Purpose.** Performs `level` as part of the sky gradient module.

**Exact declaration**

```cpp
int32_t level(int32_t x, int32_t y) const
```

- **Declared at:** [line 36](../../../runtime/sky_gradient.hpp#L36)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sky gradient module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sky_gradient.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::SkyGradientView& object = /* obtain a valid instance */;

auto result = object.level(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
