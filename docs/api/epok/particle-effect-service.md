# Epok API: Particle Effect Service

> **Header:** `"particle_effect_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/particle_effect_service.hpp)

This module covers runtime effects and their deterministic playback state. It documents 14 public callables declared directly in this header.

## Declared types

`epok::effects::Component`

## Callable index

- [`epok::effect_matrix`](#epok-effect-matrix-1) — Performs `effect matrix` as part of runtime effects and their deterministic playback state.
- [`epok::effect_world`](#epok-effect-world-1) — Performs `effect world` as part of runtime effects and their deterministic playback state.
- [`epok::effects::advance`](#epok-effects-advance-1) — Performs `advance` as part of runtime effects and their deterministic playback state.
- [`epok::effects::after_timeline`](#epok-effects-after-timeline-1) — Performs `after timeline` as part of runtime effects and their deterministic playback state.
- [`epok::effects::component`](#epok-effects-component-1) — Performs `component` as part of runtime effects and their deterministic playback state.
- [`epok::effects::configure`](#epok-effects-configure-1) — Performs `configure` as part of runtime effects and their deterministic playback state.
- [`epok::effects::play`](#epok-effects-play-1) — Starts play as part of runtime effects and their deterministic playback state.
- [`epok::effects::prepare`](#epok-effects-prepare-1) — Performs `prepare` as part of runtime effects and their deterministic playback state.
- [`epok::effects::publish_stats`](#epok-effects-publish-stats-1) — Performs `publish stats` as part of runtime effects and their deterministic playback state.
- [`epok::effects::remove_owner`](#epok-effects-remove-owner-1) — Removes owner as part of runtime effects and their deterministic playback state.
- [`epok::effects::reset_scene`](#epok-effects-reset-scene-1) — Resets scene as part of runtime effects and their deterministic playback state.
- [`epok::effects::spawn`](#epok-effects-spawn-1) — Performs `spawn` as part of runtime effects and their deterministic playback state.
- [`epok::effects::stop`](#epok-effects-stop-1) — Stops stop as part of runtime effects and their deterministic playback state.
- [`epok::remove_effect_particles`](#epok-remove-effect-particles-1) — Removes effect particles as part of runtime effects and their deterministic playback state.

<a id="epok-effect-matrix-1"></a>

## `epok::effect_matrix`

**Purpose.** Performs `effect matrix` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
Affine<Fixed> effect_matrix(const Transform&)
```

- **Declared at:** [line 6](../../../runtime/particle_effect_service.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Transform &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// const Transform & arg1

auto result = epok::effect_matrix(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effect-world-1"></a>

## `epok::effect_world`

**Purpose.** Performs `effect world` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
Affine<Fixed> effect_world(EntityHandle)
```

- **Declared at:** [line 5](../../../runtime/particle_effect_service.hpp#L5)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EntityHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle arg1

auto result = epok::effect_world(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-advance-1"></a>

## `epok::effects::advance`

**Purpose.** Performs `advance` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void advance(Fixed dt)
```

- **Declared at:** [line 57](../../../runtime/particle_effect_service.hpp#L57)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::effects::advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-after-timeline-1"></a>

## `epok::effects::after_timeline`

**Purpose.** Performs `after timeline` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void after_timeline()
```

- **Declared at:** [line 61](../../../runtime/particle_effect_service.hpp#L61)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

epok::effects::after_timeline();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-component-1"></a>

## `epok::effects::component`

**Purpose.** Performs `component` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline Component* component(EntityHandle owner)
```

- **Declared at:** [line 23](../../../runtime/particle_effect_service.hpp#L23)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Component *`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::effects::component(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-configure-1"></a>

## `epok::effects::configure`

**Purpose.** Performs `configure` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline Component* configure(EntityHandle owner,const Asset& asset,bool enabled,bool automatic,uint32_t seed)
```

- **Declared at:** [line 28](../../../runtime/particle_effect_service.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |
| `automatic` | `bool` | Input | Value supplied for `automatic`. See the exact type and module contract. |
| `seed` | `uint32_t` | Input | Value supplied for `seed`. See the exact type and module contract. |

**Returns.** Returns `Component *`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner
// const Asset & asset
// bool enabled
// bool automatic
// uint32_t seed

auto result = epok::effects::configure(owner, asset, enabled, automatic, seed);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-play-1"></a>

## `epok::effects::play`

**Purpose.** Starts play as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline Handle play(EntityHandle owner)
```

- **Declared at:** [line 38](../../../runtime/particle_effect_service.hpp#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::effects::play(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-prepare-1"></a>

## `epok::effects::prepare`

**Purpose.** Performs `prepare` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void prepare()
```

- **Declared at:** [line 48](../../../runtime/particle_effect_service.hpp#L48)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

epok::effects::prepare();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-publish-stats-1"></a>

## `epok::effects::publish_stats`

**Purpose.** Performs `publish stats` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void publish_stats()
```

- **Declared at:** [line 22](../../../runtime/particle_effect_service.hpp#L22)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

epok::effects::publish_stats();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-remove-owner-1"></a>

## `epok::effects::remove_owner`

**Purpose.** Removes owner as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void remove_owner(EntityHandle owner)
```

- **Declared at:** [line 62](../../../runtime/particle_effect_service.hpp#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

epok::effects::remove_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-reset-scene-1"></a>

## `epok::effects::reset_scene`

**Purpose.** Resets scene as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline void reset_scene()
```

- **Declared at:** [line 63](../../../runtime/particle_effect_service.hpp#L63)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

epok::effects::reset_scene();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effects-spawn-1"></a>

## `epok::effects::spawn`

**Purpose.** Performs `spawn` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline Handle spawn(const Asset& asset,const Transform& transform,uint32_t seed=0,EntityHandle owner={},const timeline::BoundTarget* bindings=nullptr)
```

- **Declared at:** [line 44](../../../runtime/particle_effect_service.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `transform` | `const Transform &` | Input | Value supplied for `transform`. See the exact type and module contract. |
| `seed` | `uint32_t` | Input | Value supplied for `seed`. See the exact type and module contract. |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `bindings` | `const timeline::BoundTarget *` | Input | Value supplied for `bindings`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// const Asset & asset
// const Transform & transform
// uint32_t seed
// EntityHandle owner
// const timeline::BoundTarget * bindings

auto result = epok::effects::spawn(asset, transform, seed, owner, bindings);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-stop-1"></a>

## `epok::effects::stop`

**Purpose.** Stops stop as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
inline bool stop(Handle handle)
```

- **Declared at:** [line 47](../../../runtime/particle_effect_service.hpp#L47)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// Handle handle

auto result = epok::effects::stop(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-remove-effect-particles-1"></a>

## `epok::remove_effect_particles`

**Purpose.** Removes effect particles as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void remove_effect_particles(EffectLayerHandle)
```

- **Declared at:** [line 7](../../../runtime/particle_effect_service.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EffectLayerHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_service.hpp"

// Assume these named values have been initialized with valid data:
// EffectLayerHandle arg1

epok::remove_effect_particles(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
