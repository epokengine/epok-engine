# Epok API: Texture

> **Header:** `"texture.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/texture.hpp)

This module covers native text data and HUD text components. It documents 5 public callables declared directly in this header.

## Callable index

- [`epok::texture`](#epok-texture-1) — Performs `texture` as part of native text data and HUD text components.
- [`epok::texture_clut`](#epok-texture-clut-1) — Performs `texture clut` as part of native text data and HUD text components.
- [`epok::texture_page`](#epok-texture-page-1) — Performs `texture page` as part of native text data and HUD text components.
- [`epok::texture_uv`](#epok-texture-uv-1) — Performs `texture uv` as part of native text data and HUD text components.
- [`epok::textures_initialize`](#epok-textures-initialize-1) — Performs `textures initialize` as part of native text data and HUD text components.

<a id="epok-texture-1"></a>

## `epok::texture`

**Purpose.** Performs `texture` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline const Texture* texture(int index)
```

- **Declared at:** [line 8](../../../runtime/texture.hpp#L8)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const Texture *`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture.hpp"

// Assume these named values have been initialized with valid data:
// int index

auto result = epok::texture(index);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-texture-clut-1"></a>

## `epok::texture_clut`

**Purpose.** Performs `texture clut` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline psyqo::PrimPieces::ClutIndex texture_clut(const Texture& t)
```

- **Declared at:** [line 10](../../../runtime/texture.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `psyqo::PrimPieces::ClutIndex`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t

auto result = epok::texture_clut(t);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-page-1"></a>

## `epok::texture_page`

**Purpose.** Performs `texture page` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline psyqo::PrimPieces::TPageAttr texture_page(const Texture& t,BlendMode blend)
```

- **Declared at:** [line 9](../../../runtime/texture.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `blend` | `BlendMode` | Input | Value supplied for `blend`. See the exact type and module contract. |

**Returns.** Returns `psyqo::PrimPieces::TPageAttr`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t
// BlendMode blend

auto result = epok::texture_page(t, blend);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-texture-uv-1"></a>

## `epok::texture_uv`

**Purpose.** Performs `texture uv` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline psyqo::PrimPieces::UVCoords texture_uv(const Texture& t,int32_t u,int32_t v)
```

- **Declared at:** [line 11](../../../runtime/texture.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Texture &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `u` | `int32_t` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `int32_t` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `psyqo::PrimPieces::UVCoords`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture.hpp"

// Assume these named values have been initialized with valid data:
// const Texture & t
// int32_t u
// int32_t v

auto result = epok::texture_uv(t, u, v);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-textures-initialize-1"></a>

## `epok::textures_initialize`

**Purpose.** Performs `textures initialize` as part of native text data and HUD text components.

**Exact declaration**

```cpp
inline void textures_initialize(psyqo::GPU& gpu)
```

- **Declared at:** [line 12](../../../runtime/texture.hpp#L12)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "texture.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::textures_initialize(gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
