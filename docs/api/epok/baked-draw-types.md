# Epok API: Baked Draw Types

> **Header:** `"baked_draw_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/baked_draw_types.hpp)

This module covers the baked draw types module. It documents 7 public callables declared directly in this header.

## Declared types

`epok::BakedDrawFace`

## Callable index

- [`epok::baked_command`](#epok-baked-command-1) — GPU opcode: 0x30, plus 0x04 textured and 0x02 semi-transparent (only ever set on textured faces), from the flag bits 30 and 29.
- [`epok::baked_corner_color`](#epok-baked-corner-color-1) — Performs `baked corner color` as part of the baked draw types module.
- [`epok::baked_double_sided`](#epok-baked-double-sided-1) — Performs `baked double sided` as part of the baked draw types module.
- [`epok::baked_indexed_color`](#epok-baked-indexed-color-1) — Performs `baked indexed color` as part of the baked draw types module.
- [`epok::baked_ordered_colors`](#epok-baked-ordered-colors-1) — Ordered native packets use only two corner layouts: 0,1,2,3 and 1,2,3,3.
- [`epok::baked_textured`](#epok-baked-textured-1) — Without a resident texture bank, faces draw untextured, as before the words were resolved at cook time: callers pass textures_ready=false.
- [`epok::baked_uv`](#epok-baked-uv-1) — A corner's packed UV (0..3).

<a id="epok-baked-command-1"></a>

## `epok::baked_command`

**Purpose.** GPU opcode: 0x30, plus 0x04 textured and 0x02 semi-transparent (only ever set on textured faces), from the flag bits 30 and 29.

**Exact declaration**

```cpp
inline uint32_t baked_command(const BakedDrawFace& face,bool textures_ready=true)
```

- **Declared at:** [line 60](../../../runtime/baked_draw_types.hpp#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `textures_ready` | `bool` | Input | Value supplied for `textures_ready`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face
// bool textures_ready

auto result = epok::baked_command(face, textures_ready);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-corner-color-1"></a>

## `epok::baked_corner_color`

**Purpose.** Performs `baked corner color` as part of the baked draw types module.

**Exact declaration**

```cpp
inline uint32_t baked_corner_color(const BakedDrawFace& face, unsigned corner, const uint32_t* palette)
```

- **Declared at:** [line 34](../../../runtime/baked_draw_types.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `corner` | `unsigned int` | Input | Value supplied for `corner`. See the exact type and module contract. |
| `palette` | `const uint32_t *` | Input | Value supplied for `palette`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face
// unsigned int corner
// const uint32_t * palette

auto result = epok::baked_corner_color(face, corner, palette);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-double-sided-1"></a>

## `epok::baked_double_sided`

**Purpose.** Performs `baked double sided` as part of the baked draw types module.

**Exact declaration**

```cpp
inline bool baked_double_sided(const BakedDrawFace& face)
```

- **Declared at:** [line 28](../../../runtime/baked_draw_types.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face

auto result = epok::baked_double_sided(face);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-indexed-color-1"></a>

## `epok::baked_indexed_color`

**Purpose.** Performs `baked indexed color` as part of the baked draw types module.

**Exact declaration**

```cpp
inline bool baked_indexed_color(const BakedDrawFace& face)
```

- **Declared at:** [line 27](../../../runtime/baked_draw_types.hpp#L27)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face

auto result = epok::baked_indexed_color(face);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-ordered-colors-1"></a>

## `epok::baked_ordered_colors`

**Purpose.** Ordered native packets use only two corner layouts: 0,1,2,3 and 1,2,3,3.

**Details.** Decode packed pairs once, without per-corner tests or variable shifts. Keep texture/CLUT identity separate; this does not bake mutable palette contents.

**Exact declaration**

```cpp
inline void baked_ordered_colors(const BakedDrawFace& face,bool second_half, const uint32_t* palette,uint32_t* output)
```

- **Declared at:** [line 42](../../../runtime/baked_draw_types.hpp#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `second_half` | `bool` | Input | Value supplied for `second_half`. See the exact type and module contract. |
| `palette` | `const uint32_t *` | Input | Value supplied for `palette`. See the exact type and module contract. |
| `output` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Decode packed pairs once, without per-corner tests or variable shifts. Keep texture/CLUT identity separate; this does not bake mutable palette contents.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face
// bool second_half
// const uint32_t * palette
// uint32_t * output

epok::baked_ordered_colors(face, second_half, palette, output);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-textured-1"></a>

## `epok::baked_textured`

**Purpose.** Without a resident texture bank, faces draw untextured, as before the words were resolved at cook time: callers pass textures_ready=false.

**Exact declaration**

```cpp
inline bool baked_textured(const BakedDrawFace& face,bool textures_ready=true)
```

- **Declared at:** [line 31](../../../runtime/baked_draw_types.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `textures_ready` | `bool` | Input | Value supplied for `textures_ready`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face
// bool textures_ready

auto result = epok::baked_textured(face, textures_ready);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-uv-1"></a>

## `epok::baked_uv`

**Purpose.** A corner's packed UV (0..3).

**Exact declaration**

```cpp
inline uint16_t baked_uv(const BakedDrawFace& face,unsigned corner)
```

- **Declared at:** [line 65](../../../runtime/baked_draw_types.hpp#L65)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `corner` | `unsigned int` | Input | Value supplied for `corner`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face
// unsigned int corner

auto result = epok::baked_uv(face, corner);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
