# Epok API: Hud Core

> **Header:** `"hud_core.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud_core.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 5 public callables declared directly in this header.

## Declared types

`epok::hud_core::Budget`, `epok::hud_core::Compiler`, `epok::hud_core::Rect`

## Callable index

- [`epok::hud_core::clamp`](#epok-hud-core-clamp-1) — Performs `clamp` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::Compiler<Sink>`](#epok-hud-core-compiler-compiler-sink-1) — Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.
- [`epok::hud_core::Compiler::draw`](#epok-hud-core-compiler-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::pixel`](#epok-hud-core-pixel-1) — Performs `pixel` as part of native HUD layout, drawing and focus navigation.
- [`epok::hud_core::resolve`](#epok-hud-core-resolve-1) — Performs `resolve` as part of native HUD layout, drawing and focus navigation.

<a id="epok-hud-core-clamp-1"></a>

## `epok::hud_core::clamp`

**Purpose.** Performs `clamp` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int clamp(int v,int hi)
```

- **Declared at:** [line 10](../../../runtime/hud_core.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `int` | Input | Value supplied for `v`. See the exact type and module contract. |
| `hi` | `int` | Input | Value supplied for `hi`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// int v
// int hi

auto result = epok::hud_core::clamp(v, hi);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-compiler-compiler-sink-1"></a>

## `epok::hud_core::Compiler::Compiler<Sink>`

**Purpose.** Constructs `epok::hud_core::Compiler` for native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
Compiler(Sink& sink,int width,int height,Budget budget):sin
```

- **Declared at:** [line 90](../../../runtime/hud_core.hpp#L90)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sink` | `Sink &` | Input/output; inspect the function contract | Value supplied for `sink`. See the exact type and module contract. |
| `width` | `int` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `int` | Input | Value supplied for `height`. See the exact type and module contract. |
| `budget` | `Budget` | Input | Value supplied for `budget`. See the exact type and module contract. |

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Sink & sink
// int width
// int height
// Budget budget

epok::hud_core::Compiler value(sink, width, height, budget);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-compiler-draw-1"></a>

## `epok::hud_core::Compiler::draw`

**Purpose.** Draws draw as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
void draw(Entity* entities,size_t count,int* first,int* next)
```

- **Declared at:** [line 91](../../../runtime/hud_core.hpp#L91)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entities` | `Entity *` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `first` | `int *` | Input/output; inspect the function contract | Value supplied for `first`. See the exact type and module contract. |
| `next` | `int *` | Input/output; inspect the function contract | Value supplied for `next`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Entity * entities
// size_t count
// int * first
// int * next

epok::hud_core::Compiler& object = /* obtain a valid instance */;

object.draw(entities, count, first, next);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-core-pixel-1"></a>

## `epok::hud_core::pixel`

**Purpose.** Performs `pixel` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline int pixel(Fixed v)
```

- **Declared at:** [line 9](../../../runtime/hud_core.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Fixed` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Fixed v

auto result = epok::hud_core::pixel(v);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-core-resolve-1"></a>

## `epok::hud_core::resolve`

**Purpose.** Performs `resolve` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline Rect resolve(Rect parent,const RectTransform& r)
```

- **Declared at:** [line 11](../../../runtime/hud_core.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parent` | `Rect` | Input | Value supplied for `parent`. See the exact type and module contract. |
| `r` | `const RectTransform &` | Input | Value supplied for `r`. See the exact type and module contract. |

**Returns.** Returns `Rect`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_core.hpp"

// Assume these named values have been initialized with valid data:
// Rect parent
// const RectTransform & r

auto result = epok::hud_core::resolve(parent, r);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
