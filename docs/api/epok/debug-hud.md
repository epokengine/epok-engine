# Epok API: Debug Hud

> **Header:** `"debug_hud.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/debug_hud.hpp)

This module covers native HUD layout, drawing and focus navigation. It documents 4 public callables declared directly in this header.

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
inline void begin(psyqo::GPU&)
```

- **Declared at:** [line 153](../../../runtime/debug_hud.hpp#L153)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// int & arg1

epok::debug_hud::begin(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-debug-hud-draw-1"></a>

## `epok::debug_hud::draw`

**Purpose.** Draws draw as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void draw(psyqo::GPU&)
```

- **Declared at:** [line 155](../../../runtime/debug_hud.hpp#L155)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// int & arg1

epok::debug_hud::draw(arg1);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-debug-hud-geometry-1"></a>

## `epok::debug_hud::geometry`

**Purpose.** Performs `geometry` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void geometry(uint16_t, bool)
```

- **Declared at:** [line 154](../../../runtime/debug_hud.hpp#L154)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `bool` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// int arg1
// bool arg2

epok::debug_hud::geometry(arg1, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-debug-hud-initialize-1"></a>

## `epok::debug_hud::initialize`

**Purpose.** Performs `initialize` as part of native HUD layout, drawing and focus navigation.

**Exact declaration**

```cpp
inline void initialize(psyqo::GPU&)
```

- **Declared at:** [line 152](../../../runtime/debug_hud.hpp#L152)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native HUD layout, drawing and focus navigation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "debug_hud.hpp"

// Assume these named values have been initialized with valid data:
// int & arg1

epok::debug_hud::initialize(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to native HUD layout, drawing and focus navigation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
