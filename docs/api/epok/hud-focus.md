# Epok API: Hud Focus

> **Header:** `"hud_focus.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/hud_focus.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::hud_focus_edges`](#epok-hud-focus-edges-1) — This frame's direction edges across every port, for callers that read the shared Input rather than a wire message.
- [`epok::hud_focus_reachable`](#epok-hud-focus-reachable-1) — D-pad focus movement over the authored neighbour table.
- [`epok::hud_focus_update`](#epok-hud-focus-update-1) — `pressed` holds this frame's press edges in Button bit order.

<a id="epok-hud-focus-edges-1"></a>

## `epok::hud_focus_edges`

**Purpose.** This frame's direction edges across every port, for callers that read the shared Input rather than a wire message.

**Exact declaration**

```cpp
inline uint32_t hud_focus_edges()
```

- **Declared at:** [line 34](../../../runtime/hud_focus.hpp#L34)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_focus.hpp"

auto result = epok::hud_focus_edges();
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-hud-focus-reachable-1"></a>

## `epok::hud_focus_reachable`

**Purpose.** D-pad focus movement over the authored neighbour table.

**Details.** Focus is one index on the canvas, not a flag per element, and a move is a static table lookup with no search. The editor's Interact mode sends the same Button bit numbering, so the console, the preview child and the viewport all move focus identically.

**Exact declaration**

```cpp
inline bool hud_focus_reachable(const ActorData* entities,size_t count,int index)
```

- **Declared at:** [line 9](../../../runtime/hud_focus.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entities` | `const ActorData *` | Input | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Focus is one index on the canvas, not a flag per element, and a move is a static table lookup with no search. The editor's Interact mode sends the same Button bit numbering, so the console, the preview child and the viewport all move focus identically.

**Usage pattern**

```cpp
#include "hud_focus.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entities
// size_t count
// int index

auto result = epok::hud_focus_reachable(entities, count, index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-hud-focus-update-1"></a>

## `epok::hud_focus_update`

**Purpose.** `pressed` holds this frame's press edges in Button bit order.

**Exact declaration**

```cpp
inline void hud_focus_update(ActorData* entities,size_t count,uint32_t pressed)
```

- **Declared at:** [line 15](../../../runtime/hud_focus.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entities` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `entities`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `pressed` | `uint32_t` | Input | Value supplied for `pressed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "hud_focus.hpp"

// Assume these named values have been initialized with valid data:
// ActorData * entities
// size_t count
// uint32_t pressed

epok::hud_focus_update(entities, count, pressed);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
