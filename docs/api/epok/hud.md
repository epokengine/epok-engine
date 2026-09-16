# Epok API: Hud

> **Header:** `"hud.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 8 public callables declared directly in this header.

## Declared types

`epok::HudRenderer`, `epok::HudRenderer::Nop`

## Callable index

- [`epok::HudRenderer::begin_text`](#epok-hudrenderer-begin-text-1) — Begins text as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::draw`](#epok-hudrenderer-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::glyph`](#epok-hudrenderer-glyph-1) — Performs `glyph` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::image`](#epok-hudrenderer-image-1) — Performs `image` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::initialize`](#epok-hudrenderer-initialize-1) — Performs `initialize` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::invalidate`](#epok-hudrenderer-invalidate-1) — Scene switches rebuild both parities; slot reuse alone is covered by the key.
- [`epok::HudRenderer::rectangle`](#epok-hudrenderer-rectangle-1) — Performs `rectangle` as part of native HUD layout, drawing and focus navigation.
- [`epok::HudRenderer::texture_size`](#epok-hudrenderer-texture-size-1) — Performs `texture size` as part of native HUD layout, drawing and focus navigation.

<a id="epok-hudrenderer-begin-text-1"></a>

## `epok::HudRenderer::begin_text`

**Purpose.** Begins text as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void begin_text()
```

- **Declared at:** [line 60](../../../runtime/hud.hpp#L60)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

epok::HudRenderer& object = /* obtain a valid instance */;

object.begin_text();
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

- **Declared at:** [line 71](../../../runtime/hud.hpp#L71)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `entities` | `int &` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu
// int & entities
// int count

epok::HudRenderer& object = /* obtain a valid instance */;

object.draw<N>(gpu, entities, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hudrenderer-glyph-1"></a>

## `epok::HudRenderer::glyph`

**Purpose.** Performs `glyph` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void glyph(int,unsigned c,int x0,int y0,int x1,int y1,int u,int v,const uint8_t* color)
```

- **Declared at:** [line 64](../../../runtime/hud.hpp#L64)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `c` | `unsigned int` | Input | Value supplied for `c`. See the exact type and module contract. |
| `x0` | `int` | Input | Value supplied for `x0`. See the exact type and module contract. |
| `y0` | `int` | Input | Value supplied for `y0`. See the exact type and module contract. |
| `x1` | `int` | Input | Value supplied for `x1`. See the exact type and module contract. |
| `y1` | `int` | Input | Value supplied for `y1`. See the exact type and module contract. |
| `u` | `int` | Input | Value supplied for `u`. See the exact type and module contract. |
| `v` | `int` | Input | Value supplied for `v`. See the exact type and module contract. |
| `color` | `const int *` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// unsigned int c
// int x0
// int y0
// int x1
// int y1
// int u
// int v
// const int * color

epok::HudRenderer& object = /* obtain a valid instance */;

object.glyph(arg1, c, x0, y0, x1, y1, u, v, color);
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

- **Declared at:** [line 51](../../../runtime/hud.hpp#L51)
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
| `color` | `const int *` | Input | Value supplied for `color`. See the exact type and module contract. |

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
// const int * color

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

- **Declared at:** [line 70](../../../runtime/hud.hpp#L70)
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

- **Declared at:** [line 90](../../../runtime/hud.hpp#L90)
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

<a id="epok-hudrenderer-rectangle-1"></a>

## `epok::HudRenderer::rectangle`

**Purpose.** Performs `rectangle` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void rectangle(int,int x0,int y0,int x1,int y1,const uint8_t* color)
```

- **Declared at:** [line 45](../../../runtime/hud.hpp#L45)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `x0` | `int` | Input | Value supplied for `x0`. See the exact type and module contract. |
| `y0` | `int` | Input | Value supplied for `y0`. See the exact type and module contract. |
| `x1` | `int` | Input | Value supplied for `x1`. See the exact type and module contract. |
| `y1` | `int` | Input | Value supplied for `y1`. See the exact type and module contract. |
| `color` | `const int *` | Input | Value supplied for `color`. See the exact type and module contract. |

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
// const int * color

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

- **Declared at:** [line 44](../../../runtime/hud.hpp#L44)
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
