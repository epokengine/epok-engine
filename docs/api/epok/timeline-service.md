# Epok API: Timeline Service

> **Header:** `"timeline_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/timeline_service.hpp)

This module covers fixed-step simulation time and frame timing. It documents 9 public callables declared directly in this header.

## Declared types

`epok::timeline::Component`

## Callable index

- [`epok::timeline::advance`](#epok-timeline-advance-1) — Performs `advance` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::component`](#epok-timeline-component-1) — Performs `component` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::configure`](#epok-timeline-configure-1) — Performs `configure` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::play`](#epok-timeline-play-1) — Starts play as part of fixed-step simulation time and frame timing.
- [`epok::timeline::publish_stats`](#epok-timeline-publish-stats-1) — Performs `publish stats` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::remove_owner`](#epok-timeline-remove-owner-1) — Removes owner as part of fixed-step simulation time and frame timing.
- [`epok::timeline::reset_scene`](#epok-timeline-reset-scene-1) — Resets scene as part of fixed-step simulation time and frame timing.
- [`epok::timeline::start_components`](#epok-timeline-start-components-1) — Starts components as part of fixed-step simulation time and frame timing.
- [`epok::timeline::stop`](#epok-timeline-stop-1) — Stops stop as part of fixed-step simulation time and frame timing.

<a id="epok-timeline-advance-1"></a>

## `epok::timeline::advance`

**Purpose.** Performs `advance` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline void advance(Fixed dt)
```

- **Declared at:** [line 46](../../../runtime/timeline_service.hpp#L46)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::timeline::advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-component-1"></a>

## `epok::timeline::component`

**Purpose.** Performs `component` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline Component* component(EntityHandle owner)
```

- **Declared at:** [line 17](../../../runtime/timeline_service.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Component *`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::timeline::component(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-configure-1"></a>

## `epok::timeline::configure`

**Purpose.** Performs `configure` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline Component* configure(EntityHandle owner,const Asset& asset,bool enabled,bool automatic)
```

- **Declared at:** [line 22](../../../runtime/timeline_service.hpp#L22)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |
| `automatic` | `bool` | Input | Value supplied for `automatic`. See the exact type and module contract. |

**Returns.** Returns `Component *`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner
// const Asset & asset
// bool enabled
// bool automatic

auto result = epok::timeline::configure(owner, asset, enabled, automatic);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-play-1"></a>

## `epok::timeline::play`

**Purpose.** Starts play as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline Handle play(EntityHandle owner)
```

- **Declared at:** [line 33](../../../runtime/timeline_service.hpp#L33)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::timeline::play(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-publish-stats-1"></a>

## `epok::timeline::publish_stats`

**Purpose.** Performs `publish stats` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline void publish_stats()
```

- **Declared at:** [line 16](../../../runtime/timeline_service.hpp#L16)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

epok::timeline::publish_stats();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-remove-owner-1"></a>

## `epok::timeline::remove_owner`

**Purpose.** Removes owner as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline void remove_owner(EntityHandle owner)
```

- **Declared at:** [line 50](../../../runtime/timeline_service.hpp#L50)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

epok::timeline::remove_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-reset-scene-1"></a>

## `epok::timeline::reset_scene`

**Purpose.** Resets scene as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline void reset_scene()
```

- **Declared at:** [line 51](../../../runtime/timeline_service.hpp#L51)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

epok::timeline::reset_scene();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-start-components-1"></a>

## `epok::timeline::start_components`

**Purpose.** Starts components as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline void start_components()
```

- **Declared at:** [line 41](../../../runtime/timeline_service.hpp#L41)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

epok::timeline::start_components();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-stop-1"></a>

## `epok::timeline::stop`

**Purpose.** Stops stop as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
inline bool stop(EntityHandle owner)
```

- **Declared at:** [line 40](../../../runtime/timeline_service.hpp#L40)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::timeline::stop(owner);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
