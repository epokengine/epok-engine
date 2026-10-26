# Epok API: Lifecycle

> **Header:** `"lifecycle.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/lifecycle.hpp)

This module covers entity creation, activation and destruction. It documents 4 public callables declared directly in this header.

## Declared types

`epok::SlotActivationMemo`

## Callable index

- [`epok::begin_render_activation_snapshot`](#epok-begin-render-activation-snapshot-1) — Begins render activation snapshot as part of entity creation, activation and destruction.
- [`epok::descendant`](#epok-descendant-1) — Performs `descendant` as part of entity creation, activation and destruction.
- [`epok::end_render_activation_snapshot`](#epok-end-render-activation-snapshot-1) — Ends render activation snapshot as part of entity creation, activation and destruction.
- [`epok::next_generation`](#epok-next-generation-1) — Performs `next generation` as part of entity creation, activation and destruction.

<a id="epok-begin-render-activation-snapshot-1"></a>

## `epok::begin_render_activation_snapshot`

**Purpose.** Begins render activation snapshot as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline void begin_render_activation_snapshot()
```

- **Declared at:** [line 59](../../../runtime/lifecycle.hpp#L59)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

epok::begin_render_activation_snapshot();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-descendant-1"></a>

## `epok::descendant`

**Purpose.** Performs `descendant` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline bool descendant(size_t child,size_t ancestor)
```

- **Declared at:** [line 69](../../../runtime/lifecycle.hpp#L69)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `size_t` | Input | Value supplied for `child`. See the exact type and module contract. |
| `ancestor` | `size_t` | Input | Value supplied for `ancestor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// size_t child
// size_t ancestor

auto result = epok::descendant(child, ancestor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-end-render-activation-snapshot-1"></a>

## `epok::end_render_activation_snapshot`

**Purpose.** Ends render activation snapshot as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline void end_render_activation_snapshot()
```

- **Declared at:** [line 64](../../../runtime/lifecycle.hpp#L64)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

epok::end_render_activation_snapshot();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-next-generation-1"></a>

## `epok::next_generation`

**Purpose.** Performs `next generation` as part of entity creation, activation and destruction.

**Exact declaration**

```cpp
inline uint32_t next_generation(uint32_t n)
```

- **Declared at:** [line 30](../../../runtime/lifecycle.hpp#L30)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `n` | `uint32_t` | Input | Value supplied for `n`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need entity creation, activation and destruction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lifecycle.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t n

auto result = epok::next_generation(n);
```

**Why choose it.** It provides direct, allocation-conscious access to entity creation, activation and destruction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
