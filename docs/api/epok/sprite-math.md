# Epok API: Sprite Math

> **Header:** `"sprite_math.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sprite_math.hpp)

This module covers sprites, flipbooks and screen-facing rendering. It documents 2 public callables declared directly in this header.

## Callable index

- [`epok::sprite_detail::interior`](#epok-sprite-detail-interior-1) — Full Q12 frustum test without overflow for arbitrary script coordinates.
- [`epok::sprite_detail::project_ratio`](#epok-sprite-detail-project-ratio-1) — Sprite projection historically truncates toward zero (not mesh floor-rounding).

<a id="epok-sprite-detail-interior-1"></a>

## `epok::sprite_detail::interior`

**Purpose.** Full Q12 frustum test without overflow for arbitrary script coordinates.

**Details.** Bound Y before multiplying; the extra bound follows from the tighter planes.

**Exact declaration**

```cpp
inline bool interior(int32_t x,int32_t y,int32_t z)
```

- **Declared at:** [line 6](../../../runtime/sprite_math.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Bound Y before multiplying; the extra bound follows from the tighter planes.

**Usage pattern**

```cpp
#include "sprite_math.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t z

auto result = epok::sprite_detail::interior(x, y, z);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite-detail-project-ratio-1"></a>

## `epok::sprite_detail::project_ratio`

**Purpose.** Sprite projection historically truncates toward zero (not mesh floor-rounding).

**Exact declaration**

```cpp
inline int32_t project_ratio(int32_t coordinate,int scale,int32_t depth)
```

- **Declared at:** [line 10](../../../runtime/sprite_math.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `coordinate` | `int32_t` | Input | Value supplied for `coordinate`. See the exact type and module contract. |
| `scale` | `int` | Input | Value supplied for `scale`. See the exact type and module contract. |
| `depth` | `int32_t` | Input | Value supplied for `depth`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_math.hpp"

// Assume these named values have been initialized with valid data:
// int32_t coordinate
// int scale
// int32_t depth

auto result = epok::sprite_detail::project_ratio(coordinate, scale, depth);
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
