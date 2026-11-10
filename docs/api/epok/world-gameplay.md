# Epok API: World Gameplay

> **Header:** `"world_gameplay.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world_gameplay.hpp)

This module covers the world gameplay module. It documents 56 public callables declared directly in this header.

## Declared types

`epok::Door2D`, `epok::Door3D`, `epok::DoorComponent`, `epok::DoorState`, `epok::ExaminableObject2D`, `epok::ExaminableObject3D`, `epok::GameplayFlagComponent`, `epok::HelperResult`, `epok::Pickup2D`, `epok::Pickup3D`, `epok::PickupComponent`, `epok::Switch2D`, `epok::Switch3D`, `epok::SwitchComponent`

## Callable index

- [`epok::Door2D::class_id`](#epok-door2d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::Door3D::class_id`](#epok-door3d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::DoorComponent::begin_play`](#epok-doorcomponent-begin-play-1) — Begins play as part of the world gameplay module.
- [`epok::DoorComponent::can_open`](#epok-doorcomponent-can-open-1) — Performs `can open` as part of the world gameplay module.
- [`epok::DoorComponent::class_id`](#epok-doorcomponent-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::DoorComponent::close`](#epok-doorcomponent-close-1) — Performs `close` as part of the world gameplay module.
- [`epok::DoorComponent::door_changed`](#epok-doorcomponent-door-changed-1) — Performs `door changed` as part of the world gameplay module.
- [`epok::DoorComponent::door_requested`](#epok-doorcomponent-door-requested-1) — Performs `door requested` as part of the world gameplay module.
- [`epok::DoorComponent::end_play`](#epok-doorcomponent-end-play-1) — Ends play as part of the world gameplay module.
- [`epok::DoorComponent::has_pending_transition`](#epok-doorcomponent-has-pending-transition-1) — Reports whether pending transition as part of the world gameplay module.
- [`epok::DoorComponent::interaction_started`](#epok-doorcomponent-interaction-started-1) — Performs `interaction started` as part of the world gameplay module.
- [`epok::DoorComponent::last_result`](#epok-doorcomponent-last-result-1) — Performs `last result` as part of the world gameplay module.
- [`epok::DoorComponent::lock`](#epok-doorcomponent-lock-1) — Performs `lock` as part of the world gameplay module.
- [`epok::DoorComponent::on_disable`](#epok-doorcomponent-on-disable-1) — Performs `on disable` as part of the world gameplay module.
- [`epok::DoorComponent::open`](#epok-doorcomponent-open-1) — Performs `open` as part of the world gameplay module.
- [`epok::DoorComponent::requirement_met`](#epok-doorcomponent-requirement-met-1) — Performs `requirement met` as part of the world gameplay module.
- [`epok::DoorComponent::reset_door`](#epok-doorcomponent-reset-door-1) — Resets door as part of the world gameplay module.
- [`epok::DoorComponent::state`](#epok-doorcomponent-state-1) — Performs `state` as part of the world gameplay module.
- [`epok::DoorComponent::tick`](#epok-doorcomponent-tick-1) — Performs `tick` as part of the world gameplay module.
- [`epok::DoorComponent::unlock`](#epok-doorcomponent-unlock-1) — Performs `unlock` as part of the world gameplay module.
- [`epok::ExaminableObject2D::class_id`](#epok-examinableobject2d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::ExaminableObject3D::class_id`](#epok-examinableobject3d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::GameplayFlagComponent::begin_play`](#epok-gameplayflagcomponent-begin-play-1) — Begins play as part of the world gameplay module.
- [`epok::GameplayFlagComponent::changing`](#epok-gameplayflagcomponent-changing-1) — Performs `changing` as part of the world gameplay module.
- [`epok::GameplayFlagComponent::class_id`](#epok-gameplayflagcomponent-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::GameplayFlagComponent::set_value`](#epok-gameplayflagcomponent-set-value-1) — Sets value as part of the world gameplay module.
- [`epok::GameplayFlagComponent::value`](#epok-gameplayflagcomponent-value-1) — Performs `value` as part of the world gameplay module.
- [`epok::GameplayFlagComponent::value_changed`](#epok-gameplayflagcomponent-value-changed-1) — Performs `value changed` as part of the world gameplay module.
- [`epok::Pickup2D::class_id`](#epok-pickup2d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::Pickup3D::class_id`](#epok-pickup3d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::PickupComponent::begin_play`](#epok-pickupcomponent-begin-play-1) — Begins play as part of the world gameplay module.
- [`epok::PickupComponent::can_collect`](#epok-pickupcomponent-can-collect-1) — Performs `can collect` as part of the world gameplay module.
- [`epok::PickupComponent::can_interact`](#epok-pickupcomponent-can-interact-1) — Performs `can interact` as part of the world gameplay module.
- [`epok::PickupComponent::class_id`](#epok-pickupcomponent-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::PickupComponent::collect`](#epok-pickupcomponent-collect-1) — Performs `collect` as part of the world gameplay module.
- [`epok::PickupComponent::collected`](#epok-pickupcomponent-collected-1) — Performs `collected` as part of the world gameplay module.
- [`epok::PickupComponent::interaction_started`](#epok-pickupcomponent-interaction-started-1) — Keep the once-only state transition native.
- [`epok::PickupComponent::is_collected`](#epok-pickupcomponent-is-collected-1) — Reports whether collected as part of the world gameplay module.
- [`epok::PickupComponent::reset_collection`](#epok-pickupcomponent-reset-collection-1) — Resets collection as part of the world gameplay module.
- [`epok::PickupComponent::trigger_event`](#epok-pickupcomponent-trigger-event-1) — Performs `trigger event` as part of the world gameplay module.
- [`epok::Switch2D::class_id`](#epok-switch2d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::Switch3D::class_id`](#epok-switch3d-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::SwitchComponent::activate`](#epok-switchcomponent-activate-1) — Performs `activate` as part of the world gameplay module.
- [`epok::SwitchComponent::begin_play`](#epok-switchcomponent-begin-play-1) — Begins play as part of the world gameplay module.
- [`epok::SwitchComponent::can_interact`](#epok-switchcomponent-can-interact-1) — Performs `can interact` as part of the world gameplay module.
- [`epok::SwitchComponent::can_switch`](#epok-switchcomponent-can-switch-1) — Performs `can switch` as part of the world gameplay module.
- [`epok::SwitchComponent::class_id`](#epok-switchcomponent-class-id-1) — Performs `class id` as part of the world gameplay module.
- [`epok::SwitchComponent::interaction_started`](#epok-switchcomponent-interaction-started-1) — Performs `interaction started` as part of the world gameplay module.
- [`epok::SwitchComponent::reset_switch`](#epok-switchcomponent-reset-switch-1) — Resets switch as part of the world gameplay module.
- [`epok::SwitchComponent::switched`](#epok-switchcomponent-switched-1) — Performs `switched` as part of the world gameplay module.
- [`epok::SwitchComponent::value`](#epok-switchcomponent-value-1) — Performs `value` as part of the world gameplay module.
- [`epok::world_gameplay::accepted`](#epok-world-gameplay-accepted-1) — Performs `accepted` as part of the world gameplay module.
- [`epok::world_gameplay::alive`](#epok-world-gameplay-alive-1) — Performs `alive` as part of the world gameplay module.
- [`epok::world_gameplay::flag`](#epok-world-gameplay-flag-1) — Performs `flag` as part of the world gameplay module.
- [`epok::world_gameplay::flag_ready`](#epok-world-gameplay-flag-ready-1) — Performs `flag ready` as part of the world gameplay module.
- [`epok::world_gameplay::interaction_result`](#epok-world-gameplay-interaction-result-1) — Performs `interaction result` as part of the world gameplay module.

<a id="epok-door2d-class-id-1"></a>

## `epok::Door2D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 270](../../../runtime/world_gameplay.hpp#L270)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Door2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-door3d-class-id-1"></a>

## `epok::Door3D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 263](../../../runtime/world_gameplay.hpp#L263)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Door3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-begin-play-1"></a>

## `epok::DoorComponent::begin_play`

**Purpose.** Begins play as part of the world gameplay module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 220](../../../runtime/world_gameplay.hpp#L220)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-can-open-1"></a>

## `epok::DoorComponent::can_open`

**Purpose.** Performs `can open` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual bool can_open(ObjectId initiator)const
```

- **Declared at:** [line 211](../../../runtime/world_gameplay.hpp#L211)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.can_open(initiator);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-doorcomponent-class-id-1"></a>

## `epok::DoorComponent::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 202](../../../runtime/world_gameplay.hpp#L202)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-close-1"></a>

## `epok::DoorComponent::close`

**Purpose.** Performs `close` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult close()
```

- **Declared at:** [line 215](../../../runtime/world_gameplay.hpp#L215)
- **Kind:** `cxx method`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.close();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-door-changed-1"></a>

## `epok::DoorComponent::door_changed`

**Purpose.** Performs `door changed` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual void door_changed(DoorState previous,DoorState current)
```

- **Declared at:** [line 212](../../../runtime/world_gameplay.hpp#L212)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `previous` | `DoorState` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `current` | `DoorState` | Input | Value supplied for `current`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// DoorState previous
// DoorState current

epok::DoorComponent& object = /* obtain a valid instance */;

object.door_changed(previous, current);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-door-requested-1"></a>

## `epok::DoorComponent::door_requested`

**Purpose.** Performs `door requested` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual void door_requested(ObjectId initiator,HelperResult result)
```

- **Declared at:** [line 213](../../../runtime/world_gameplay.hpp#L213)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `result` | `HelperResult` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// HelperResult result

epok::DoorComponent& object = /* obtain a valid instance */;

object.door_requested(initiator, result);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-end-play-1"></a>

## `epok::DoorComponent::end_play`

**Purpose.** Ends play as part of the world gameplay module.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 233](../../../runtime/world_gameplay.hpp#L233)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::DoorComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-has-pending-transition-1"></a>

## `epok::DoorComponent::has_pending_transition`

**Purpose.** Reports whether pending transition as part of the world gameplay module.

**Exact declaration**

```cpp
bool has_pending_transition()const
```

- **Declared at:** [line 209](../../../runtime/world_gameplay.hpp#L209)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.has_pending_transition();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-doorcomponent-interaction-started-1"></a>

## `epok::DoorComponent::interaction_started`

**Purpose.** Performs `interaction started` as part of the world gameplay module.

**Exact declaration**

```cpp
void interaction_started(ObjectId initiator,uint32_t)override
```

- **Declared at:** [line 219](../../../runtime/world_gameplay.hpp#L219)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `arg2` | `uint32_t` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t arg2

epok::DoorComponent& object = /* obtain a valid instance */;

object.interaction_started(initiator, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-last-result-1"></a>

## `epok::DoorComponent::last_result`

**Purpose.** Performs `last result` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult last_result()const
```

- **Declared at:** [line 208](../../../runtime/world_gameplay.hpp#L208)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.last_result();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-lock-1"></a>

## `epok::DoorComponent::lock`

**Purpose.** Performs `lock` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult lock()
```

- **Declared at:** [line 216](../../../runtime/world_gameplay.hpp#L216)
- **Kind:** `cxx method`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.lock();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-on-disable-1"></a>

## `epok::DoorComponent::on_disable`

**Purpose.** Performs `on disable` as part of the world gameplay module.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 232](../../../runtime/world_gameplay.hpp#L232)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-open-1"></a>

## `epok::DoorComponent::open`

**Purpose.** Performs `open` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult open(ObjectId initiator)
```

- **Declared at:** [line 214](../../../runtime/world_gameplay.hpp#L214)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.open(initiator);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-requirement-met-1"></a>

## `epok::DoorComponent::requirement_met`

**Purpose.** Performs `requirement met` as part of the world gameplay module.

**Exact declaration**

```cpp
bool requirement_met()const
```

- **Declared at:** [line 210](../../../runtime/world_gameplay.hpp#L210)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.requirement_met();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-doorcomponent-reset-door-1"></a>

## `epok::DoorComponent::reset_door`

**Purpose.** Resets door as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult reset_door()
```

- **Declared at:** [line 218](../../../runtime/world_gameplay.hpp#L218)
- **Kind:** `cxx method`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.reset_door();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-state-1"></a>

## `epok::DoorComponent::state`

**Purpose.** Performs `state` as part of the world gameplay module.

**Exact declaration**

```cpp
DoorState state()const
```

- **Declared at:** [line 207](../../../runtime/world_gameplay.hpp#L207)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `DoorState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.state();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-tick-1"></a>

## `epok::DoorComponent::tick`

**Purpose.** Performs `tick` as part of the world gameplay module.

**Exact declaration**

```cpp
void tick(Fixed)override
```

- **Declared at:** [line 221](../../../runtime/world_gameplay.hpp#L221)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::DoorComponent& object = /* obtain a valid instance */;

object.tick(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-doorcomponent-unlock-1"></a>

## `epok::DoorComponent::unlock`

**Purpose.** Performs `unlock` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult unlock()
```

- **Declared at:** [line 217](../../../runtime/world_gameplay.hpp#L217)
- **Kind:** `cxx method`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::DoorComponent& object = /* obtain a valid instance */;

auto result = object.unlock();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-examinableobject2d-class-id-1"></a>

## `epok::ExaminableObject2D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 245](../../../runtime/world_gameplay.hpp#L245)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::ExaminableObject2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-examinableobject3d-class-id-1"></a>

## `epok::ExaminableObject3D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 239](../../../runtime/world_gameplay.hpp#L239)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::ExaminableObject3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayflagcomponent-begin-play-1"></a>

## `epok::GameplayFlagComponent::begin_play`

**Purpose.** Begins play as part of the world gameplay module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 56](../../../runtime/world_gameplay.hpp#L56)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayflagcomponent-changing-1"></a>

## `epok::GameplayFlagComponent::changing`

**Purpose.** Performs `changing` as part of the world gameplay module.

**Exact declaration**

```cpp
bool changing()const
```

- **Declared at:** [line 46](../../../runtime/world_gameplay.hpp#L46)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

auto result = object.changing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayflagcomponent-class-id-1"></a>

## `epok::GameplayFlagComponent::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 43](../../../runtime/world_gameplay.hpp#L43)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayflagcomponent-set-value-1"></a>

## `epok::GameplayFlagComponent::set_value`

**Purpose.** Sets value as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult set_value(bool value)
```

- **Declared at:** [line 48](../../../runtime/world_gameplay.hpp#L48)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

auto result = object.set_value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayflagcomponent-value-1"></a>

## `epok::GameplayFlagComponent::value`

**Purpose.** Performs `value` as part of the world gameplay module.

**Exact declaration**

```cpp
bool value()const
```

- **Declared at:** [line 45](../../../runtime/world_gameplay.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayflagcomponent-value-changed-1"></a>

## `epok::GameplayFlagComponent::value_changed`

**Purpose.** Performs `value changed` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual void value_changed(bool previous,bool current)
```

- **Declared at:** [line 47](../../../runtime/world_gameplay.hpp#L47)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `previous` | `bool` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `current` | `bool` | Input | Value supplied for `current`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// bool previous
// bool current

epok::GameplayFlagComponent& object = /* obtain a valid instance */;

object.value_changed(previous, current);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickup2d-class-id-1"></a>

## `epok::Pickup2D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 257](../../../runtime/world_gameplay.hpp#L257)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Pickup2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickup3d-class-id-1"></a>

## `epok::Pickup3D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 251](../../../runtime/world_gameplay.hpp#L251)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Pickup3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-begin-play-1"></a>

## `epok::PickupComponent::begin_play`

**Purpose.** Begins play as part of the world gameplay module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 107](../../../runtime/world_gameplay.hpp#L107)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::PickupComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-can-collect-1"></a>

## `epok::PickupComponent::can_collect`

**Purpose.** Performs `can collect` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual bool can_collect(ObjectId initiator)const
```

- **Declared at:** [line 80](../../../runtime/world_gameplay.hpp#L80)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.can_collect(initiator);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pickupcomponent-can-interact-1"></a>

## `epok::PickupComponent::can_interact`

**Purpose.** Performs `can interact` as part of the world gameplay module.

**Exact declaration**

```cpp
bool can_interact(ObjectId initiator)const final
```

- **Declared at:** [line 82](../../../runtime/world_gameplay.hpp#L82)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.can_interact(initiator);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pickupcomponent-class-id-1"></a>

## `epok::PickupComponent::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 73](../../../runtime/world_gameplay.hpp#L73)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-collect-1"></a>

## `epok::PickupComponent::collect`

**Purpose.** Performs `collect` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult collect(ObjectId initiator,uint32_t port)
```

- **Declared at:** [line 95](../../../runtime/world_gameplay.hpp#L95)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.collect(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-collected-1"></a>

## `epok::PickupComponent::collected`

**Purpose.** Performs `collected` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual void collected(ObjectId initiator,uint32_t port)
```

- **Declared at:** [line 81](../../../runtime/world_gameplay.hpp#L81)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::PickupComponent& object = /* obtain a valid instance */;

object.collected(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-interaction-started-1"></a>

## `epok::PickupComponent::interaction_started`

**Purpose.** Keep the once-only state transition native.

**Details.** Projects extend collected().

**Exact declaration**

```cpp
void interaction_started(ObjectId initiator,uint32_t port)final
```

- **Declared at:** [line 87](../../../runtime/world_gameplay.hpp#L87)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Projects extend collected().

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::PickupComponent& object = /* obtain a valid instance */;

object.interaction_started(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pickupcomponent-is-collected-1"></a>

## `epok::PickupComponent::is_collected`

**Purpose.** Reports whether collected as part of the world gameplay module.

**Exact declaration**

```cpp
bool is_collected()const
```

- **Declared at:** [line 79](../../../runtime/world_gameplay.hpp#L79)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.is_collected();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pickupcomponent-reset-collection-1"></a>

## `epok::PickupComponent::reset_collection`

**Purpose.** Resets collection as part of the world gameplay module.

**Exact declaration**

```cpp
bool reset_collection()
```

- **Declared at:** [line 105](../../../runtime/world_gameplay.hpp#L105)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::PickupComponent& object = /* obtain a valid instance */;

auto result = object.reset_collection();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pickupcomponent-trigger-event-1"></a>

## `epok::PickupComponent::trigger_event`

**Purpose.** Performs `trigger event` as part of the world gameplay module.

**Exact declaration**

```cpp
void trigger_event(ObjectId other,TriggerPhase phase)override
```

- **Declared at:** [line 106](../../../runtime/world_gameplay.hpp#L106)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `ObjectId` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId other
// TriggerPhase phase

epok::PickupComponent& object = /* obtain a valid instance */;

object.trigger_event(other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switch2d-class-id-1"></a>

## `epok::Switch2D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 283](../../../runtime/world_gameplay.hpp#L283)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Switch2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switch3d-class-id-1"></a>

## `epok::Switch3D::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 277](../../../runtime/world_gameplay.hpp#L277)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::Switch3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-activate-1"></a>

## `epok::SwitchComponent::activate`

**Purpose.** Performs `activate` as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult activate(ObjectId initiator,uint32_t port)
```

- **Declared at:** [line 136](../../../runtime/world_gameplay.hpp#L136)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.activate(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-begin-play-1"></a>

## `epok::SwitchComponent::begin_play`

**Purpose.** Begins play as part of the world gameplay module.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 142](../../../runtime/world_gameplay.hpp#L142)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::SwitchComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-can-interact-1"></a>

## `epok::SwitchComponent::can_interact`

**Purpose.** Performs `can interact` as part of the world gameplay module.

**Exact declaration**

```cpp
bool can_interact(ObjectId initiator)const final
```

- **Declared at:** [line 134](../../../runtime/world_gameplay.hpp#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.can_interact(initiator);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-switchcomponent-can-switch-1"></a>

## `epok::SwitchComponent::can_switch`

**Purpose.** Performs `can switch` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual bool can_switch(ObjectId initiator)const
```

- **Declared at:** [line 132](../../../runtime/world_gameplay.hpp#L132)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.can_switch(initiator);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-switchcomponent-class-id-1"></a>

## `epok::SwitchComponent::class_id`

**Purpose.** Performs `class id` as part of the world gameplay module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 126](../../../runtime/world_gameplay.hpp#L126)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-interaction-started-1"></a>

## `epok::SwitchComponent::interaction_started`

**Purpose.** Performs `interaction started` as part of the world gameplay module.

**Exact declaration**

```cpp
void interaction_started(ObjectId initiator,uint32_t)final
```

- **Declared at:** [line 135](../../../runtime/world_gameplay.hpp#L135)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `arg2` | `uint32_t` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t arg2

epok::SwitchComponent& object = /* obtain a valid instance */;

object.interaction_started(initiator, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-reset-switch-1"></a>

## `epok::SwitchComponent::reset_switch`

**Purpose.** Resets switch as part of the world gameplay module.

**Exact declaration**

```cpp
HelperResult reset_switch()
```

- **Declared at:** [line 141](../../../runtime/world_gameplay.hpp#L141)
- **Kind:** `cxx method`

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.reset_switch();
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-switched-1"></a>

## `epok::SwitchComponent::switched`

**Purpose.** Performs `switched` as part of the world gameplay module.

**Exact declaration**

```cpp
virtual void switched(ObjectId initiator,bool value)
```

- **Declared at:** [line 133](../../../runtime/world_gameplay.hpp#L133)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// bool value

epok::SwitchComponent& object = /* obtain a valid instance */;

object.switched(initiator, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-switchcomponent-value-1"></a>

## `epok::SwitchComponent::value`

**Purpose.** Performs `value` as part of the world gameplay module.

**Exact declaration**

```cpp
bool value()const
```

- **Declared at:** [line 131](../../../runtime/world_gameplay.hpp#L131)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

epok::SwitchComponent& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-world-gameplay-accepted-1"></a>

## `epok::world_gameplay::accepted`

**Purpose.** Performs `accepted` as part of the world gameplay module.

**Exact declaration**

```cpp
inline bool accepted(ObjectId source,ObjectId required,bool mandatory)
```

- **Declared at:** [line 31](../../../runtime/world_gameplay.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |
| `required` | `ObjectId` | Input | Value supplied for `required`. See the exact type and module contract. |
| `mandatory` | `bool` | Input | Value supplied for `mandatory`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source
// ObjectId required
// bool mandatory

auto result = epok::world_gameplay::accepted(source, required, mandatory);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-world-gameplay-alive-1"></a>

## `epok::world_gameplay::alive`

**Purpose.** Performs `alive` as part of the world gameplay module.

**Exact declaration**

```cpp
inline bool alive(ObjectId id)
```

- **Declared at:** [line 15](../../../runtime/world_gameplay.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::world_gameplay::alive(id);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-world-gameplay-flag-1"></a>

## `epok::world_gameplay::flag`

**Purpose.** Performs `flag` as part of the world gameplay module.

**Exact declaration**

```cpp
inline GameplayFlagComponent* flag(const ActorComponent& owner,ObjectId id)
```

- **Declared at:** [line 59](../../../runtime/world_gameplay.hpp#L59)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const ActorComponent &` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `GameplayFlagComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// const ActorComponent & owner
// ObjectId id

auto result = epok::world_gameplay::flag(owner, id);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-gameplay-flag-ready-1"></a>

## `epok::world_gameplay::flag_ready`

**Purpose.** Performs `flag ready` as part of the world gameplay module.

**Exact declaration**

```cpp
inline bool flag_ready(const ActorComponent& owner,ObjectId id)
```

- **Declared at:** [line 64](../../../runtime/world_gameplay.hpp#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `const ActorComponent &` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// const ActorComponent & owner
// ObjectId id

auto result = epok::world_gameplay::flag_ready(owner, id);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world-gameplay-interaction-result-1"></a>

## `epok::world_gameplay::interaction_result`

**Purpose.** Performs `interaction result` as part of the world gameplay module.

**Exact declaration**

```cpp
inline HelperResult interaction_result(InteractionResult value)
```

- **Declared at:** [line 20](../../../runtime/world_gameplay.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `InteractionResult` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `HelperResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world gameplay module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world_gameplay.hpp"

// Assume these named values have been initialized with valid data:
// InteractionResult value

auto result = epok::world_gameplay::interaction_result(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world gameplay module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
