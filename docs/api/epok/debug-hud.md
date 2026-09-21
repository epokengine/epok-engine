# Epok API: Debug Hud

> **Header:** `"debug_hud.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/debug_hud.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 4 public callables declared directly in this header.

## Declared types

`epok::debug_hud::State`

## Callable index

- [`epok::debug_hud::begin`](#epok-debug-hud-begin-1) — Begins begin as part of native HUD layout, drawing and focus navigation.
- [`epok::debug_hud::draw`](#epok-debug-hud-draw-1) — Draws draw as part of native HUD layout, drawing and focus navigation.
- [`epok::debug_hud::geometry`](#epok-debug-hud-geometry-1) — Performs `geometry` as part of native HUD layout, drawing and focus navigation.
- [`epok::debug_hud::initialize`](#epok-debug-hud-initialize-1) — Performs `initialize` as part of native HUD layout, drawing and focus navigation.

<a id="epok-debug-hud-begin-1"></a>

## `epok::debug_hud::begin`

**Purpose.** Begins begin as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void begin(psyqo::GPU& gpu)
```

- **Declared at:** [line 62](../../../runtime/debug_hud.hpp#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::debug_hud::begin(gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-debug-hud-draw-1"></a>

## `epok::debug_hud::draw`

**Purpose.** Draws draw as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void draw(psyqo::GPU& gpu)
```

- **Declared at:** [line 91](../../../runtime/debug_hud.hpp#L91)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::debug_hud::draw(gpu);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-debug-hud-geometry-1"></a>

## `epok::debug_hud::geometry`

**Purpose.** Performs `geometry` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void geometry(uint16_t started, bool gte)
```

- **Declared at:** [line 85](../../../runtime/debug_hud.hpp#L85)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `started` | `uint16_t` | Input | Value supplied for `started`. See the exact type and module contract. |
| `gte` | `bool` | Input | Value supplied for `gte`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t started
// bool gte

epok::debug_hud::geometry(started, gte);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-debug-hud-initialize-1"></a>

## `epok::debug_hud::initialize`

**Purpose.** Performs `initialize` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void initialize(psyqo::GPU& gpu)
```

- **Declared at:** [line 42](../../../runtime/debug_hud.hpp#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

epok::debug_hud::initialize(gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
