# Epok API: Navigation Components

> **Header:** `"navigation_components.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/navigation_components.hpp)

This module covers the navigation components module. It documents 21 public callables declared directly in this header.

## Declared types

`epok::NavigationAgentComponent`, `epok::NavigationBakeVolumeComponent`, `epok::NavigationLinkComponent`, `epok::NavigationObstacleComponent`, `epok::NavigationSurfaceComponent`

## Callable index

- [`epok::nav::move_walk`](#epok-nav-move-walk-1) — Performs `move walk` as part of the navigation components module.
- [`epok::NavigationAgentComponent::arrived`](#epok-navigationagentcomponent-arrived-1) — Performs `arrived` as part of the navigation components module.
- [`epok::NavigationAgentComponent::begin_play`](#epok-navigationagentcomponent-begin-play-1) — Begins play as part of the navigation components module.
- [`epok::NavigationAgentComponent::class_id`](#epok-navigationagentcomponent-class-id-1) — Performs `class id` as part of the navigation components module.
- [`epok::NavigationAgentComponent::end_play`](#epok-navigationagentcomponent-end-play-1) — Ends play as part of the navigation components module.
- [`epok::NavigationAgentComponent::failed`](#epok-navigationagentcomponent-failed-1) — Performs `failed` as part of the navigation components module.
- [`epok::NavigationAgentComponent::move_to`](#epok-navigationagentcomponent-move-to-1) — Performs `move to` as part of the navigation components module.
- [`epok::NavigationAgentComponent::on_disable`](#epok-navigationagentcomponent-on-disable-1) — Performs `on disable` as part of the navigation components module.
- [`epok::NavigationAgentComponent::status`](#epok-navigationagentcomponent-status-1) — Performs `status` as part of the navigation components module.
- [`epok::NavigationAgentComponent::stop`](#epok-navigationagentcomponent-stop-1) — Stops stop as part of the navigation components module.
- [`epok::NavigationAgentComponent::tick`](#epok-navigationagentcomponent-tick-1) — Performs `tick` as part of the navigation components module.
- [`epok::NavigationBakeVolumeComponent::class_id`](#epok-navigationbakevolumecomponent-class-id-1) — Performs `class id` as part of the navigation components module.
- [`epok::NavigationLinkComponent::class_id`](#epok-navigationlinkcomponent-class-id-1) — Performs `class id` as part of the navigation components module.
- [`epok::NavigationObstacleComponent::begin_play`](#epok-navigationobstaclecomponent-begin-play-1) — Begins play as part of the navigation components module.
- [`epok::NavigationObstacleComponent::class_id`](#epok-navigationobstaclecomponent-class-id-1) — Performs `class id` as part of the navigation components module.
- [`epok::NavigationObstacleComponent::end_play`](#epok-navigationobstaclecomponent-end-play-1) — Ends play as part of the navigation components module.
- [`epok::NavigationObstacleComponent::on_disable`](#epok-navigationobstaclecomponent-on-disable-1) — Performs `on disable` as part of the navigation components module.
- [`epok::NavigationObstacleComponent::on_enable`](#epok-navigationobstaclecomponent-on-enable-1) — Performs `on enable` as part of the navigation components module.
- [`epok::NavigationObstacleComponent::registered`](#epok-navigationobstaclecomponent-registered-1) — Performs `registered` as part of the navigation components module.
- [`epok::NavigationObstacleComponent::tick`](#epok-navigationobstaclecomponent-tick-1) — Performs `tick` as part of the navigation components module.
- [`epok::NavigationSurfaceComponent::class_id`](#epok-navigationsurfacecomponent-class-id-1) — Performs `class id` as part of the navigation components module.

<a id="epok-nav-move-walk-1"></a>

## `epok::nav::move_walk`

**Purpose.** Performs `move walk` as part of the navigation components module.

**Exact declaration**

```cpp
inline bool move_walk(ActorData& data,const Fixed* delta,uint32_t mask)
```

- **Declared at:** [line 9](../../../runtime/navigation_components.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `data`. See the exact type and module contract. |
| `delta` | `const int *` | Input | Value supplied for `delta`. See the exact type and module contract. |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & data
// const int * delta
// int mask

auto result = epok::nav::move_walk(data, delta, mask);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-navigationagentcomponent-arrived-1"></a>

## `epok::NavigationAgentComponent::arrived`

**Purpose.** Performs `arrived` as part of the navigation components module.

**Exact declaration**

```cpp
bool arrived() const
```

- **Declared at:** [line 112](../../../runtime/navigation_components.hpp#L112)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

auto result = object.arrived();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagentcomponent-begin-play-1"></a>

## `epok::NavigationAgentComponent::begin_play`

**Purpose.** Begins play as part of the navigation components module.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 114](../../../runtime/navigation_components.hpp#L114)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-class-id-1"></a>

## `epok::NavigationAgentComponent::class_id`

**Purpose.** Performs `class id` as part of the navigation components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 87](../../../runtime/navigation_components.hpp#L87)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-end-play-1"></a>

## `epok::NavigationAgentComponent::end_play`

**Purpose.** Ends play as part of the navigation components module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 115](../../../runtime/navigation_components.hpp#L115)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-failed-1"></a>

## `epok::NavigationAgentComponent::failed`

**Purpose.** Performs `failed` as part of the navigation components module.

**Exact declaration**

```cpp
bool failed() const
```

- **Declared at:** [line 113](../../../runtime/navigation_components.hpp#L113)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

auto result = object.failed();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagentcomponent-move-to-1"></a>

## `epok::NavigationAgentComponent::move_to`

**Purpose.** Performs `move to` as part of the navigation components module.

**Exact declaration**

```cpp
bool move_to(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 101](../../../runtime/navigation_components.hpp#L101)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// int x
// int y
// int z

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

auto result = object.move_to(x, y, z);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagentcomponent-on-disable-1"></a>

## `epok::NavigationAgentComponent::on_disable`

**Purpose.** Performs `on disable` as part of the navigation components module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 116](../../../runtime/navigation_components.hpp#L116)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-status-1"></a>

## `epok::NavigationAgentComponent::status`

**Purpose.** Performs `status` as part of the navigation components module.

**Exact declaration**

```cpp
uint32_t status() const
```

- **Declared at:** [line 111](../../../runtime/navigation_components.hpp#L111)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

auto result = object.status();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-stop-1"></a>

## `epok::NavigationAgentComponent::stop`

**Purpose.** Stops stop as part of the navigation components module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 110](../../../runtime/navigation_components.hpp#L110)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagentcomponent-tick-1"></a>

## `epok::NavigationAgentComponent::tick`

**Purpose.** Performs `tick` as part of the navigation components module.

**Exact declaration**

```cpp
void tick(Fixed dt) override
```

- **Declared at:** [line 117](../../../runtime/navigation_components.hpp#L117)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `int` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// int dt

epok::NavigationAgentComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationbakevolumecomponent-class-id-1"></a>

## `epok::NavigationBakeVolumeComponent::class_id`

**Purpose.** Performs `class id` as part of the navigation components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 39](../../../runtime/navigation_components.hpp#L39)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationBakeVolumeComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationlinkcomponent-class-id-1"></a>

## `epok::NavigationLinkComponent::class_id`

**Purpose.** Performs `class id` as part of the navigation components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 56](../../../runtime/navigation_components.hpp#L56)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationLinkComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-begin-play-1"></a>

## `epok::NavigationObstacleComponent::begin_play`

**Purpose.** Begins play as part of the navigation components module.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 69](../../../runtime/navigation_components.hpp#L69)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-class-id-1"></a>

## `epok::NavigationObstacleComponent::class_id`

**Purpose.** Performs `class id` as part of the navigation components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 68](../../../runtime/navigation_components.hpp#L68)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-end-play-1"></a>

## `epok::NavigationObstacleComponent::end_play`

**Purpose.** Ends play as part of the navigation components module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 73](../../../runtime/navigation_components.hpp#L73)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-on-disable-1"></a>

## `epok::NavigationObstacleComponent::on_disable`

**Purpose.** Performs `on disable` as part of the navigation components module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 72](../../../runtime/navigation_components.hpp#L72)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-on-enable-1"></a>

## `epok::NavigationObstacleComponent::on_enable`

**Purpose.** Performs `on enable` as part of the navigation components module.

**Exact declaration**

```cpp
void on_enable() override
```

- **Declared at:** [line 70](../../../runtime/navigation_components.hpp#L70)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

object.on_enable();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationobstaclecomponent-registered-1"></a>

## `epok::NavigationObstacleComponent::registered`

**Purpose.** Performs `registered` as part of the navigation components module.

**Exact declaration**

```cpp
bool registered() const
```

- **Declared at:** [line 74](../../../runtime/navigation_components.hpp#L74)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

auto result = object.registered();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationobstaclecomponent-tick-1"></a>

## `epok::NavigationObstacleComponent::tick`

**Purpose.** Performs `tick` as part of the navigation components module.

**Exact declaration**

```cpp
void tick(Fixed) override
```

- **Declared at:** [line 71](../../../runtime/navigation_components.hpp#L71)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

// Assume these named values have been initialized with valid data:
// int arg1

epok::NavigationObstacleComponent& object = /* obtain a valid instance */;

object.tick(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationsurfacecomponent-class-id-1"></a>

## `epok::NavigationSurfaceComponent::class_id`

**Purpose.** Performs `class id` as part of the navigation components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 51](../../../runtime/navigation_components.hpp#L51)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation_components.hpp"

epok::NavigationSurfaceComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
