# Epok API: Hud

> **Header:** `"hud.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 12 public callables declared directly in this header.

## Declared types

`epok::HudRenderer`

## Callable index

- [`epok::HudRenderer::begin_text`](#epok-hudrenderer-begin-text-1) — Begins text as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::draw`](#epok-hudrenderer-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::font`](#epok-hudrenderer-font-1) — Font index -1 is the built-in atlas, resident at (960,448) with its CLUT at (60,448); an authored font brings its own placement from the export allocator.
- [`epok::HudRenderer::glyph`](#epok-hudrenderer-glyph-1) — Source coordinates arrive as atlas texels; the page origin turns them into the texel offsets within the page begin_text() chained.
- [`epok::HudRenderer::glyph_quad`](#epok-hudrenderer-glyph-quad-1) — The same page, CLUT and atlas texels glyph() uses; only the primitive class differs, and a TexturedQuad carries its own page rather than inheriting the one begin_text() chained.
- [`epok::HudRenderer::image`](#epok-hudrenderer-image-1) — Performs `image` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::initialize`](#epok-hudrenderer-initialize-1) — Performs `initialize` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::invalidate`](#epok-hudrenderer-invalidate-1) — Scene switches rebuild both parities; slot reuse alone is covered by the key.
- [`epok::HudRenderer::quad`](#epok-hudrenderer-quad-1) — Rotated fills, images and glyphs.
- [`epok::HudRenderer::rectangle`](#epok-hudrenderer-rectangle-1) — Performs `rectangle` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::texture_size`](#epok-hudrenderer-texture-size-1) — Performs `texture size` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::textured_quad`](#epok-hudrenderer-textured-quad-1) — Performs `textured quad` as part of native HUD layout, drawing and focus navigation.

<a id="epok-hudrenderer-begin-text-1"></a>

## `epok::HudRenderer::begin_text`

**Purpose.** Begins text as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void begin_text(int index)
```

- **Declared at:** [line 81](../../../runtime/hud.hpp#L81)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::HudRenderer& object = /* obtain a valid instance */;

object.begin_text(index);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-hudrenderer-draw-1"></a>

## `epok::HudRenderer::draw`

**Purpose.** Draws draw as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
template<size_t N>void draw(psyqo::GPU& gpu,std::array<ActorData,N>& entities,size_t count)
```

- **Declared at:** [line 131](../../../runtime/hud.hpp#L131)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `entities` | `std::array<ActorData, N> &` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu
// std::array<ActorData, N> & entities
// size_t count

epok::HudRenderer& object = /* obtain a valid instance */;

object.draw<N>(gpu, entities, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-font-1"></a>

## `epok::HudRenderer::font`

**Purpose.** Font index -1 is the built-in atlas, resident at (960,448) with its CLUT at (60,448); an authored font brings its own placement from the export allocator.

**Details.** Both are 4bpp, so one TPage per text element still serves.

**Exact declaration**

```cpp
const Font* font(int index)
```

- **Declared at:** [line 80](../../../runtime/hud.hpp#L80)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const Font *`. Check the purpose and failure notes before using the value.

**Use it when.** Both are 4bpp, so one TPage per text element still serves.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int index

epok::HudRenderer& object = /* obtain a valid instance */;

auto result = object.font(index);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-hudrenderer-glyph-1"></a>

## `epok::HudRenderer::glyph`

**Purpose.** Source coordinates arrive as atlas texels; the page origin turns them into the texel offsets within the page begin_text() chained.

**Exact declaration**

```cpp
void glyph(int,int index,int u,int v,int x0,int y0,int x1,int y1,const uint8_t* color)
```

- **Declared at:** [line 88](../../../runtime/hud.hpp#L88)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `u` | `int` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `int` | Input | Value supplied for `v`. See the exact type and module contract. |
| `x0` | `int` | Input | Value supplied for `x0`. See the exact type and module contract. |
| `y0` | `int` | Input | Value supplied for `y0`. See the exact type and module contract. |
| `x1` | `int` | Input | Value supplied for `x1`. See the exact type and module contract. |
| `y1` | `int` | Input | Value supplied for `y1`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// int index
// int u
// int v
// int x0
// int y0
// int x1
// int y1
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.glyph(arg1, index, u, v, x0, y0, x1, y1, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-glyph-quad-1"></a>

## `epok::HudRenderer::glyph_quad`

**Purpose.** The same page, CLUT and atlas texels glyph() uses; only the primitive class differs, and a TexturedQuad carries its own page rather than inheriting the one begin_text() chained.

**Exact declaration**

```cpp
void glyph_quad(int,int index,const int* x,const int* y,const int* u,const int* v,const uint8_t* color)
```

- **Declared at:** [line 116](../../../runtime/hud.hpp#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `x` | `const int *` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `const int *` | Input | Value supplied for `y`. See the exact type and module contract. |
| `u` | `const int *` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `const int *` | Input | Value supplied for `v`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// int index
// const int * x
// const int * y
// const int * u
// const int * v
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.glyph_quad(arg1, index, x, y, u, v, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-image-1"></a>

## `epok::HudRenderer::image`

**Purpose.** Performs `image` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void image(int,int id,int x0,int y0,int x1,int y1,int u0,int v0,int u1,int v1,const uint8_t* color)
```

- **Declared at:** [line 68](../../../runtime/hud.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `id` | `int` | Input | Value supplied for `id`. See the exact type and module contract. |
| `x0` | `int` | Input | Value supplied for `x0`. See the exact type and module contract. |
| `y0` | `int` | Input | Value supplied for `y0`. See the exact type and module contract. |
| `x1` | `int` | Input | Value supplied for `x1`. See the exact type and module contract. |
| `y1` | `int` | Input | Value supplied for `y1`. See the exact type and module contract. |
| `u0` | `int` | Input | Value supplied for `u0`. See the exact type and module contract. |
| `v0` | `int` | Input | Value supplied for `v0`. See the exact type and module contract. |
| `u1` | `int` | Input | Value supplied for `u1`. See the exact type and module contract. |
| `v1` | `int` | Input | Value supplied for `v1`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// int id
// int x0
// int y0
// int x1
// int y1
// int u0
// int v0
// int u1
// int v1
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.image(arg1, id, x0, y0, x1, y1, u0, v0, u1, v1, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-initialize-1"></a>

## `epok::HudRenderer::initialize`

**Purpose.** Performs `initialize` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void initialize(psyqo::GPU& gpu)
```

- **Declared at:** [line 130](../../../runtime/hud.hpp#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::HudRenderer& object = /* obtain a valid instance */;

object.initialize(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-invalidate-1"></a>

## `epok::HudRenderer::invalidate`

**Purpose.** Scene switches rebuild both parities; slot reuse alone is covered by the key.

**Exact declaration**

```cpp
void invalidate()
```

- **Declared at:** [line 150](../../../runtime/hud.hpp#L150)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

epok::HudRenderer& object = /* obtain a valid instance */;

object.invalidate();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-hudrenderer-quad-1"></a>

## `epok::HudRenderer::quad`

**Purpose.** Rotated fills, images and glyphs.

**Details.** The corner arrays arrive in the A/B/C/D Z order the polygon class wants, so nothing is reordered here.

**Exact declaration**

```cpp
void quad(int,const int* x,const int* y,const uint8_t* color)
```

- **Declared at:** [line 97](../../../runtime/hud.hpp#L97)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `x` | `const int *` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `const int *` | Input | Value supplied for `y`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The corner arrays arrive in the A/B/C/D Z order the polygon class wants, so nothing is reordered here.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// const int * x
// const int * y
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.quad(arg1, x, y, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-rectangle-1"></a>

## `epok::HudRenderer::rectangle`

**Purpose.** Performs `rectangle` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void rectangle(int,int x0,int y0,int x1,int y1,const uint8_t* color)
```

- **Declared at:** [line 62](../../../runtime/hud.hpp#L62)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `x0` | `int` | Input | Value supplied for `x0`. See the exact type and module contract. |
| `y0` | `int` | Input | Value supplied for `y0`. See the exact type and module contract. |
| `x1` | `int` | Input | Value supplied for `x1`. See the exact type and module contract. |
| `y1` | `int` | Input | Value supplied for `y1`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// int x0
// int y0
// int x1
// int y1
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.rectangle(arg1, x0, y0, x1, y1, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-texture-size-1"></a>

## `epok::HudRenderer::texture_size`

**Purpose.** Performs `texture size` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
bool texture_size(int id,int& width,int& height)
```

- **Declared at:** [line 61](../../../runtime/hud.hpp#L61)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `int` | Input | Value supplied for `id`. See the exact type and module contract. |
| `width` | `int &` | Input/output; inspect the function contract | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int &` | Input/output; inspect the function contract | Value supplied for `height`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int id
// int & width
// int & height

epok::HudRenderer& object = /* obtain a valid instance */;

auto result = object.texture_size(id, width, height);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-textured-quad-1"></a>

## `epok::HudRenderer::textured_quad`

**Purpose.** Performs `textured quad` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void textured_quad(int,int id,const int* x,const int* y,const int* u,const int* v,const uint8_t* color)
```

- **Declared at:** [line 103](../../../runtime/hud.hpp#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `id` | `int` | Input | Value supplied for `id`. See the exact type and module contract. |
| `x` | `const int *` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `const int *` | Input | Value supplied for `y`. See the exact type and module contract. |
| `u` | `const int *` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `const int *` | Input | Value supplied for `v`. See the exact type and module contract. |
| `color` | `const uint8_t *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// int id
// const int * x
// const int * y
// const int * u
// const int * v
// const uint8_t * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.textured_quad(arg1, id, x, y, u, v, color);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
