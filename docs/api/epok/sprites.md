# Epok API: Sprites

> **Header:** `"sprites.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sprites.hpp)

This module covers sprites, flipbooks and screen-facing rendering. It documents 5 public callables declared directly in this header.

## Declared types

`epok::sprite_detail::PlaneCache`, `epok::SpriteRenderer`

## Callable index

- [`epok::sprite_detail::plane`](#epok-sprite-detail-plane-1) — Performs `plane` as part of sprites, flipbooks and screen-facing rendering.
- [`epok::sprite_detail::PlaneCache::clear`](#epok-sprite-detail-planecache-clear-1) — Clears clear as part of sprites, flipbooks and screen-facing rendering.
- [`epok::sprite_detail::PlaneCache::transform`](#epok-sprite-detail-planecache-transform-1) — Performs `transform` as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteRenderer::begin`](#epok-spriterenderer-begin-1) — Begins begin as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteRenderer::draw`](#epok-spriterenderer-draw-1) — LightingRenderer::shade(owner,objects,world,true) must prepare GTE before a lit draw.

<a id="epok-sprite-detail-plane-1"></a>

## `epok::sprite_detail::plane`

**Purpose.** Performs `plane` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
inline Affine<Fixed> plane(const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view)
```

- **Declared at:** [line 10](../../../runtime/sprites.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sprite` | `const Sprite &` | Input | Value supplied for `sprite`. See the exact type and module contract. |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `view` | `const Affine<Fixed> &` | Input | Value supplied for `view`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprites.hpp"

// Assume these named values have been initialized with valid data:
// const Sprite & sprite
// const Affine<Fixed> & world
// const Affine<Fixed> & view

auto result = epok::sprite_detail::plane(sprite, world, view);
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sprite-detail-planecache-clear-1"></a>

## `epok::sprite_detail::PlaneCache::clear`

**Purpose.** Clears clear as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 26](../../../runtime/sprites.hpp#L26)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprites.hpp"

epok::sprite_detail::PlaneCache& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite-detail-planecache-transform-1"></a>

## `epok::sprite_detail::PlaneCache::transform`

**Purpose.** Performs `transform` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void transform(const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view, Affine<Fixed>& model,Affine<Fixed>& camera)
```

- **Declared at:** [line 27](../../../runtime/sprites.hpp#L27)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sprite` | `const Sprite &` | Input | Value supplied for `sprite`. See the exact type and module contract. |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `view` | `const Affine<Fixed> &` | Input | Value supplied for `view`. See the exact type and module contract. |
| `model` | `Affine<Fixed> &` | Input/output; inspect the function contract | Value supplied for `model`. See the exact type and module contract. |
| `camera` | `Affine<Fixed> &` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprites.hpp"

// Assume these named values have been initialized with valid data:
// const Sprite & sprite
// const Affine<Fixed> & world
// const Affine<Fixed> & view
// Affine<Fixed> & model
// Affine<Fixed> & camera

epok::sprite_detail::PlaneCache& object = /* obtain a valid instance */;

object.transform(sprite, world, view, model, camera);
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spriterenderer-begin-1"></a>

## `epok::SpriteRenderer::begin`

**Purpose.** Begins begin as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void begin()
```

- **Declared at:** [line 60](../../../runtime/sprites.hpp#L60)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprites.hpp"

epok::SpriteRenderer& object = /* obtain a valid instance */;

object.begin();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-spriterenderer-draw-1"></a>

## `epok::SpriteRenderer::draw`

**Purpose.** LightingRenderer::shade(owner,objects,world,true) must prepare GTE before a lit draw.

**Exact declaration**

```cpp
template<class Table>void draw(int parity,Table& table,const Sprite& sprite,const Affine<Fixed>& world,const Affine<Fixed>& view,bool receive_lighting=true)
```

- **Declared at:** [line 62](../../../runtime/sprites.hpp#L62)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `table` | `Table &` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `sprite` | `const Sprite &` | Input | Value supplied for `sprite`. See the exact type and module contract. |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `view` | `const Affine<Fixed> &` | Input | Value supplied for `view`. See the exact type and module contract. |
| `receive_lighting` | `bool` | Input | Value supplied for `receive_lighting`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprites.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Table

// Assume these named values have been initialized with valid data:
// int parity
// Table & table
// const Sprite & sprite
// const Affine<Fixed> & world
// const Affine<Fixed> & view
// bool receive_lighting

epok::SpriteRenderer& object = /* obtain a valid instance */;

object.draw<Table>(parity, table, sprite, world, view, receive_lighting);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
