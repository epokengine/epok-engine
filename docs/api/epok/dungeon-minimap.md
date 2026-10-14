# Epok API: Dungeon Minimap

> **Header:** `"dungeon_minimap.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/dungeon_minimap.hpp)

This module covers the dungeon minimap module. It documents 1 public callable declared directly in this header.

## Declared types

`epok::dungeon::MinimapRenderer`

## Callable index

- [`epok::dungeon::MinimapRenderer::draw`](#epok-dungeon-minimaprenderer-draw-1) — `player` is the player's world point (raw Q12), or null when there is none; `frame` counts frames for the player's blink.

<a id="epok-dungeon-minimaprenderer-draw-1"></a>

## `epok::dungeon::MinimapRenderer::draw`

**Purpose.** `player` is the player's world point (raw Q12), or null when there is none; `frame` counts frames for the player's blink.

**Details.** Exploring happens whether or not the map shows.

**Exact declaration**

```cpp
template<class Active> void draw(psyqo::GPU& output,int screen_width,int screen_height,const int32_t* player,uint32_t frame,Active&& active)
```

- **Declared at:** [line 56](../../../runtime/dungeon_minimap.hpp#L56)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `output` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `screen_width` | `int` | Input | Value supplied for `screen_width`. See the exact type and module contract. |
| `screen_height` | `int` | Input | Value supplied for `screen_height`. See the exact type and module contract. |
| `player` | `const int32_t *` | Input | Value supplied for `player`. See the exact type and module contract. |
| `frame` | `uint32_t` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `active` | `Active &&` | Consumed or moved input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Exploring happens whether or not the map shows.

**Usage pattern**

```cpp
#include "dungeon_minimap.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Active

// Assume these named values have been initialized with valid data:
// psyqo::GPU & output
// int screen_width
// int screen_height
// const int32_t * player
// uint32_t frame
// Active && active

epok::dungeon::MinimapRenderer& object = /* obtain a valid instance */;

object.draw<Active>(output, screen_width, screen_height, player, frame, active);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
