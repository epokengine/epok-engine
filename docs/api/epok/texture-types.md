# Epok API: Texture Types

> **Header:** `"texture_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/texture_types.hpp)

This module covers native text data and HUD text components. It documents 7 public callables declared directly in this header.

## Declared types

`epok::BakedTexturePacket`, `epok::BakedTextureTable`, `epok::BlendMode`, `epok::Texture`

## Callable index

- [`epok::bake_texture_packets`](#epok-bake-texture-packets-1) — Performs `bake texture packets` as part of native text data and HUD text components.
- [`epok::baked_texture_packet`](#epok-baked-texture-packet-1) — Performs `baked texture packet` as part of native text data and HUD text components.
- [`epok::texture_base_u`](#epok-texture-base-u-1) — Performs `texture base u` as part of native text data and HUD text components.
- [`epok::texture_colors`](#epok-texture-colors-1) — Performs `texture colors` as part of native text data and HUD text components.
- [`epok::texture_packed_uv`](#epok-texture-packed-uv-1) — Performs `texture packed uv` as part of native text data and HUD text components.
- [`epok::texture_present`](#epok-texture-present-1) — Performs `texture present` as part of native text data and HUD text components.
- [`epok::textures_clear_of_display`](#epok-textures-clear-of-display-1) — Every cooked bank asserts this.

<a id="epok-bake-texture-packets-1"></a>

## `epok::bake_texture_packets`

**Purpose.** Performs `bake texture packets` as part of native text data and HUD text components.

**Exact declaration**

```cpp
template<size_t N> constexpr BakedTextureTable<N> bake_texture_packets(const Texture (&bank)[N])
```

- **Declared at:** [line 34](../../../runtime/texture_types.hpp#L34)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const Texture (&)[N]` | Input | Value supplied for `bank`. See the exact type and module contract. |

**Returns.** Returns `BakedTextureTable<N>`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// const Texture (&)[N] bank

auto result = epok::bake_texture_packets<N>(bank);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-baked-texture-packet-1"></a>

## `epok::baked_texture_packet`

**Purpose.** Performs `baked texture packet` as part of native text data and HUD text components.

**Exact declaration**

```cpp
template<bool Interlaced,bool Dithering> inline BakedTexturePacket baked_texture_packet(const uint32_t* bank,size_t count, uint16_t index,BlendMode blend)
```

- **Declared at:** [line 47](../../../runtime/texture_types.hpp#L47)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const uint32_t *` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `index` | `uint16_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `blend` | `BlendMode` | Input | Value supplied for `blend`. See the exact type and module contract. |

**Returns.** Returns `BakedTexturePacket`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Interlaced, Dithering

// Assume these named values have been initialized with valid data:
// const uint32_t * bank
// size_t count
// uint16_t index
// BlendMode blend

auto result = epok::baked_texture_packet<Interlaced, Dithering>(bank, count, index, blend);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-base-u-1"></a>

## `epok::texture_base_u`

**Purpose.** Performs `texture base u` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline constexpr uint16_t texture_base_u(const Texture& t)
```

- **Declared at:** [line 13](../../../runtime/texture_types.hpp#L13)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t

auto result = epok::texture_base_u(t);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-colors-1"></a>

## `epok::texture_colors`

**Purpose.** Performs `texture colors` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline constexpr uint16_t texture_colors(const Texture& t)
```

- **Declared at:** [line 12](../../../runtime/texture_types.hpp#L12)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t

auto result = epok::texture_colors(t);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-packed-uv-1"></a>

## `epok::texture_packed_uv`

**Purpose.** Performs `texture packed uv` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline constexpr uint16_t texture_packed_uv(const Texture& t,int32_t u,int32_t v)
```

- **Declared at:** [line 14](../../../runtime/texture_types.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `u` | `int32_t` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `int32_t` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t
// int32_t u
// int32_t v

auto result = epok::texture_packed_uv(t, u, v);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-present-1"></a>

## `epok::texture_present`

**Purpose.** Performs `texture present` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline constexpr bool texture_present(const Texture& t)
```

- **Declared at:** [line 11](../../../runtime/texture_types.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t

auto result = epok::texture_present(t);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-textures-clear-of-display-1"></a>

## `epok::textures_clear_of_display`

**Purpose.** Every cooked bank asserts this.

**Details.** The allocator leaves the framebuffer columns of the configured display free; a texture or CLUT inside them would be overwritten by rendering, so a stale or mismatched layout fails to compile.

**Exact declaration**

```cpp
template<size_t N> constexpr bool textures_clear_of_display(const Texture (&bank)[N],int width)
```

- **Declared at:** [line 21](../../../runtime/texture_types.hpp#L21)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const Texture (&)[N]` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The allocator leaves the framebuffer columns of the configured display free; a texture or CLUT inside them would be overwritten by rendering, so a stale or mismatched layout fails to compile.

**Usage pattern**

```cpp
#include "texture_types.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// const Texture (&)[N] bank
// int width

auto result = epok::textures_clear_of_display<N>(bank, width);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
