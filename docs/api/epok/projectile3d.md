# Epok API: Projectile3D

> **Header:** `"projectile3d.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/projectile3d.hpp)

This module covers the projectile3d module. It documents 16 public callables declared directly in this header.

## Declared types

`epok::ProjectileBatch`, `epok::ProjectileHandle`, `epok::ProjectileLaunchResult`, `epok::ProjectileLibrary`, `epok::ProjectilePool3DComponent`, `epok::ProjectileResult`, `epok::ProjectileSnapshot`, `epok::ProjectileSpawn3D`, `epok::ProjectileSpriteGrid`, `epok::ProjectileStep`, `epok::ProjectileStep4`, `epok::ProjectileStepState`, `epok::ProjectileTraceFilter`

## Callable index

- [`epok::ProjectileLibrary::filter`](#epok-projectilelibrary-filter-1) — Performs `filter` as part of the projectile3d module.
- [`epok::ProjectileLibrary::spawn`](#epok-projectilelibrary-spawn-1) — Performs `spawn` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::active_count`](#epok-projectilepool3dcomponent-active-count-1) — Performs `active count` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::begin_play`](#epok-projectilepool3dcomponent-begin-play-1) — Begins play as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::bind`](#epok-projectilepool3dcomponent-bind-1) — Performs `bind` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::binding`](#epok-projectilepool3dcomponent-binding-1) — Performs `binding` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::class_id`](#epok-projectilepool3dcomponent-class-id-1) — Performs `class id` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::end_play`](#epok-projectilepool3dcomponent-end-play-1) — Ends play as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::launch`](#epok-projectilepool3dcomponent-launch-1) — Performs `launch` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::on_disable`](#epok-projectilepool3dcomponent-on-disable-1) — Performs `on disable` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::reset`](#epok-projectilepool3dcomponent-reset-1) — Resets reset as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::result`](#epok-projectilepool3dcomponent-result-1) — Performs `result` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::snapshot`](#epok-projectilepool3dcomponent-snapshot-1) — Performs `snapshot` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::step`](#epok-projectilepool3dcomponent-step-1) — Performs `step` as part of the projectile3d module.
- [`epok::ProjectilePool3DComponent::step_shared`](#epok-projectilepool3dcomponent-step-shared-1) — Native callers may reuse a synchronized context.
- [`epok::ProjectilePool3DComponent::stop`](#epok-projectilepool3dcomponent-stop-1) — Stops stop as part of the projectile3d module.

<a id="epok-projectilelibrary-filter-1"></a>

## `epok::ProjectileLibrary::filter`

**Purpose.** Performs `filter` as part of the projectile3d module.

**Exact declaration**

```cpp
static ProjectileTraceFilter filter(uint32_t mask,ObjectId ignore,bool triggers)
```

- **Declared at:** [line 263](../../../runtime/projectile3d.hpp#L263)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `ProjectileTraceFilter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t mask
// ObjectId ignore
// bool triggers

auto result = epok::ProjectileLibrary::filter(mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilelibrary-spawn-1"></a>

## `epok::ProjectileLibrary::spawn`

**Purpose.** Performs `spawn` as part of the projectile3d module.

**Exact declaration**

```cpp
static ProjectileSpawn3D spawn(GameplayVector3 origin,GameplayVector3 velocity,Fixed lifetime,uint32_t tag)
```

- **Declared at:** [line 262](../../../runtime/projectile3d.hpp#L262)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `velocity` | `GameplayVector3` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `lifetime` | `Fixed` | Input | Value supplied for `lifetime`. See the exact type and module contract. |
| `tag` | `uint32_t` | Input | Value supplied for `tag`. See the exact type and module contract. |

**Returns.** Returns `ProjectileSpawn3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 origin
// GameplayVector3 velocity
// Fixed lifetime
// uint32_t tag

auto result = epok::ProjectileLibrary::spawn(origin, velocity, lifetime, tag);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-active-count-1"></a>

## `epok::ProjectilePool3DComponent::active_count`

**Purpose.** Performs `active count` as part of the projectile3d module.

**Exact declaration**

```cpp
uint32_t active_count()const
```

- **Declared at:** [line 157](../../../runtime/projectile3d.hpp#L157)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.active_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-begin-play-1"></a>

## `epok::ProjectilePool3DComponent::begin_play`

**Purpose.** Begins play as part of the projectile3d module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 257](../../../runtime/projectile3d.hpp#L257)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-bind-1"></a>

## `epok::ProjectilePool3DComponent::bind`

**Purpose.** Performs `bind` as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileResult bind(uint32_t index,ObjectId value)
```

- **Declared at:** [line 158](../../../runtime/projectile3d.hpp#L158)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `ObjectId` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `ProjectileResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// ObjectId value

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.bind(index, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-binding-1"></a>

## `epok::ProjectilePool3DComponent::binding`

**Purpose.** Performs `binding` as part of the projectile3d module.

**Exact declaration**

```cpp
ObjectId binding(uint32_t slot)const
```

- **Declared at:** [line 156](../../../runtime/projectile3d.hpp#L156)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.binding(slot);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-class-id-1"></a>

## `epok::ProjectilePool3DComponent::class_id`

**Purpose.** Performs `class id` as part of the projectile3d module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 144](../../../runtime/projectile3d.hpp#L144)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-end-play-1"></a>

## `epok::ProjectilePool3DComponent::end_play`

**Purpose.** Ends play as part of the projectile3d module.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 259](../../../runtime/projectile3d.hpp#L259)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-launch-1"></a>

## `epok::ProjectilePool3DComponent::launch`

**Purpose.** Performs `launch` as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileLaunchResult launch(uint32_t first,uint32_t count,ProjectileSpawn3D spawn)
```

- **Declared at:** [line 169](../../../runtime/projectile3d.hpp#L169)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `spawn` | `ProjectileSpawn3D` | Input | Value supplied for `spawn`. See the exact type and module contract. |

**Returns.** Returns `ProjectileLaunchResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t first
// uint32_t count
// ProjectileSpawn3D spawn

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.launch(first, count, spawn);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-on-disable-1"></a>

## `epok::ProjectilePool3DComponent::on_disable`

**Purpose.** Performs `on disable` as part of the projectile3d module.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 258](../../../runtime/projectile3d.hpp#L258)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-reset-1"></a>

## `epok::ProjectilePool3DComponent::reset`

**Purpose.** Resets reset as part of the projectile3d module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 251](../../../runtime/projectile3d.hpp#L251)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-result-1"></a>

## `epok::ProjectilePool3DComponent::result`

**Purpose.** Performs `result` as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileStep result(ProjectileBatch batch,uint32_t index)const
```

- **Declared at:** [line 239](../../../runtime/projectile3d.hpp#L239)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `batch` | `ProjectileBatch` | Input | Value supplied for `batch`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ProjectileStep`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// ProjectileBatch batch
// uint32_t index

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.result(batch, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-snapshot-1"></a>

## `epok::ProjectilePool3DComponent::snapshot`

**Purpose.** Performs `snapshot` as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileSnapshot snapshot(ProjectileHandle value)const
```

- **Declared at:** [line 252](../../../runtime/projectile3d.hpp#L252)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ProjectileHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `ProjectileSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// ProjectileHandle value

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.snapshot(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-step-1"></a>

## `epok::ProjectilePool3DComponent::step`

**Purpose.** Performs `step` as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileBatch step(uint32_t first,uint32_t count,Fixed dt,ProjectileTraceFilter filter)
```

- **Declared at:** [line 229](../../../runtime/projectile3d.hpp#L229)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `filter` | `ProjectileTraceFilter` | Input | Value supplied for `filter`. See the exact type and module contract. |

**Returns.** Returns `ProjectileBatch`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t first
// uint32_t count
// Fixed dt
// ProjectileTraceFilter filter

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.step(first, count, dt, filter);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-projectilepool3dcomponent-step-shared-1"></a>

## `epok::ProjectilePool3DComponent::step_shared`

**Purpose.** Native callers may reuse a synchronized context.

**Details.** Script providers use step().

**Exact declaration**

```cpp
ProjectileStep4 step_shared(uint32_t first,uint32_t count,Fixed dt,ProjectileTraceFilter filter,const SpatialQueryBatch& queries)
```

- **Declared at:** [line 190](../../../runtime/projectile3d.hpp#L190)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `filter` | `ProjectileTraceFilter` | Input | Value supplied for `filter`. See the exact type and module contract. |
| `queries` | `const SpatialQueryBatch &` | Input | Value supplied for `queries`. See the exact type and module contract. |

**Returns.** Returns `ProjectileStep4`. Check the purpose and failure notes before using the value.

**Use it when.** Script providers use step().

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t first
// uint32_t count
// Fixed dt
// ProjectileTraceFilter filter
// const SpatialQueryBatch & queries

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.step_shared(first, count, dt, filter, queries);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-projectilepool3dcomponent-stop-1"></a>

## `epok::ProjectilePool3DComponent::stop`

**Purpose.** Stops stop as part of the projectile3d module.

**Exact declaration**

```cpp
ProjectileResult stop(ProjectileHandle value)
```

- **Declared at:** [line 245](../../../runtime/projectile3d.hpp#L245)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ProjectileHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `ProjectileResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the projectile3d module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "projectile3d.hpp"

// Assume these named values have been initialized with valid data:
// ProjectileHandle value

epok::ProjectilePool3DComponent& object = /* obtain a valid instance */;

auto result = object.stop(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the projectile3d module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
