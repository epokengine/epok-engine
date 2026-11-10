# Epok API: Character3D

> **Header:** `"character3d.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/character3d.hpp)

This module covers shared world-unit character commands, collision locomotion and motion samples. It documents 24 public callables declared directly in this header.

## Declared types

`epok::Character3D`, `epok::character3d::State`, `epok::character3d::Tuning`, `epok::character3d::WorldQueries`, `epok::CharacterCommand3D`, `epok::CharacterMotionSample3D`, `epok::CharacterMotionStatus`, `epok::CharacterMotor3DComponent`, `epok::detail::CharacterCommandLease`

## Callable index

- [`epok::character3d::advance`](#epok-character3d-advance-1) — Performs `advance` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::character3d::advance`](#epok-character3d-advance-2) — A query adapter can reuse an already synchronized world without changing the motor's integration/contact rules.
- [`epok::character3d::bounded`](#epok-character3d-bounded-1) — Performs `bounded` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::Character3D::class_id`](#epok-character3d-class-id-1) — Performs `class id` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::Character3D::motion`](#epok-character3d-motion-1) — Performs `motion` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::Character3D::motor_id`](#epok-character3d-motor-id-1) — Performs `motor id` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::Character3D::stop_movement`](#epok-character3d-stop-movement-1) — Stops movement as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::Character3D::submit_command`](#epok-character3d-submit-command-1) — Performs `submit command` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::character3d::valid`](#epok-character3d-valid-1) — Performs `valid` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::character3d::WorldQueries::ground`](#epok-character3d-worldqueries-ground-1) — Performs `ground` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::character3d::WorldQueries::move`](#epok-character3d-worldqueries-move-1) — Performs `move` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::begin_play`](#epok-charactermotor3dcomponent-begin-play-1) — Begins play as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::class_id`](#epok-charactermotor3dcomponent-class-id-1) — Performs `class id` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::clear_velocity`](#epok-charactermotor3dcomponent-clear-velocity-1) — Clears velocity as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::end_play`](#epok-charactermotor3dcomponent-end-play-1) — Ends play as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::motion`](#epok-charactermotor3dcomponent-motion-1) — Performs `motion` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::on_disable`](#epok-charactermotor3dcomponent-on-disable-1) — Performs `on disable` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::reset_motion`](#epok-charactermotor3dcomponent-reset-motion-1) — Resets motion as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::set_facing`](#epok-charactermotor3dcomponent-set-facing-1) — Assign project-controlled facing without applying a second turn policy.
- [`epok::CharacterMotor3DComponent::simulate`](#epok-charactermotor3dcomponent-simulate-1) — Performs `simulate` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::simulate_shared`](#epok-charactermotor3dcomponent-simulate-shared-1) — Performs `simulate shared` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::stop`](#epok-charactermotor3dcomponent-stop-1) — Stops stop as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::submit`](#epok-charactermotor3dcomponent-submit-1) — Performs `submit` as part of shared world-unit character commands, collision locomotion and motion samples.
- [`epok::CharacterMotor3DComponent::tick`](#epok-charactermotor3dcomponent-tick-1) — Performs `tick` as part of shared world-unit character commands, collision locomotion and motion samples.

<a id="epok-character3d-advance-1"></a>

## `epok::character3d::advance`

**Purpose.** Performs `advance` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
inline CharacterMotionSample3D advance(ActorData& body,State& state,const Tuning& t,const CharacterCommand3D& command,Fixed dt)
```

- **Declared at:** [line 93](../../../runtime/character3d.hpp#L93)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `body` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `body`. See the exact type and module contract. |
| `state` | `State &` | Input/output; inspect the function contract | Value supplied for `state`. See the exact type and module contract. |
| `t` | `const Tuning &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `command` | `const CharacterCommand3D &` | Input | Value supplied for `command`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & body
// State & state
// const Tuning & t
// const CharacterCommand3D & command
// Fixed dt

auto result = epok::character3d::advance(body, state, t, command, dt);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-character3d-advance-2"></a>

## `epok::character3d::advance`

**Purpose.** A query adapter can reuse an already synchronized world without changing the motor's integration/contact rules.

**Details.** Callers may defer the optional speed work when another project action owns animation for this step.

**Exact declaration**

```cpp
template<class Queries> inline CharacterMotionSample3D advance(ActorData& body,State& state,const Tuning& t,const CharacterCommand3D& command,Fixed dt,Queries& queries,bool measure_speed=true)
```

- **Declared at:** [line 53](../../../runtime/character3d.hpp#L53)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `body` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `body`. See the exact type and module contract. |
| `state` | `State &` | Input/output; inspect the function contract | Value supplied for `state`. See the exact type and module contract. |
| `t` | `const Tuning &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `command` | `const CharacterCommand3D &` | Input | Value supplied for `command`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `queries` | `Queries &` | Input/output; inspect the function contract | Value supplied for `queries`. See the exact type and module contract. |
| `measure_speed` | `bool` | Input | Value supplied for `measure_speed`. See the exact type and module contract. |

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** Callers may defer the optional speed work when another project action owns animation for this step.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Queries

// Assume these named values have been initialized with valid data:
// ActorData & body
// State & state
// const Tuning & t
// const CharacterCommand3D & command
// Fixed dt
// Queries & queries
// bool measure_speed

epok::character3d& object = /* obtain a valid instance */;

auto result = object.advance<Queries>(body, state, t, command, dt, queries, measure_speed);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-character3d-bounded-1"></a>

## `epok::character3d::bounded`

**Purpose.** Performs `bounded` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
inline bool bounded(Fixed value,int32_t maximum)
```

- **Declared at:** [line 31](../../../runtime/character3d.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `maximum` | `int32_t` | Input | Value supplied for `maximum`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// int32_t maximum

auto result = epok::character3d::bounded(value, maximum);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-character3d-class-id-1"></a>

## `epok::Character3D::class_id`

**Purpose.** Performs `class id` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 217](../../../runtime/character3d.hpp#L217)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::Character3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character3d-motion-1"></a>

## `epok::Character3D::motion`

**Purpose.** Performs `motion` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
CharacterMotionSample3D motion()const
```

- **Declared at:** [line 224](../../../runtime/character3d.hpp#L224)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::Character3D& object = /* obtain a valid instance */;

auto result = object.motion();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character3d-motor-id-1"></a>

## `epok::Character3D::motor_id`

**Purpose.** Performs `motor id` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
ObjectId motor_id()const
```

- **Declared at:** [line 225](../../../runtime/character3d.hpp#L225)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::Character3D& object = /* obtain a valid instance */;

auto result = object.motor_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character3d-stop-movement-1"></a>

## `epok::Character3D::stop_movement`

**Purpose.** Stops movement as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void stop_movement()
```

- **Declared at:** [line 223](../../../runtime/character3d.hpp#L223)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::Character3D& object = /* obtain a valid instance */;

object.stop_movement();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character3d-submit-command-1"></a>

## `epok::Character3D::submit_command`

**Purpose.** Performs `submit command` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void submit_command(CharacterCommand3D command)
```

- **Declared at:** [line 222](../../../runtime/character3d.hpp#L222)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand3D` | Input | Value supplied for `command`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand3D command

epok::Character3D& object = /* obtain a valid instance */;

object.submit_command(command);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character3d-valid-1"></a>

## `epok::character3d::valid`

**Purpose.** Performs `valid` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
inline bool valid(const Tuning& t,const State& state,const CharacterCommand3D& command,Fixed dt)
```

- **Declared at:** [line 32](../../../runtime/character3d.hpp#L32)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `const Tuning &` | Input | Value supplied for `t`. See the exact type and module contract. |
| `state` | `const State &` | Input | Value supplied for `state`. See the exact type and module contract. |
| `command` | `const CharacterCommand3D &` | Input | Value supplied for `command`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// const Tuning & t
// const State & state
// const CharacterCommand3D & command
// Fixed dt

auto result = epok::character3d::valid(t, state, command, dt);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-character3d-worldqueries-ground-1"></a>

## `epok::character3d::WorldQueries::ground`

**Purpose.** Performs `ground` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
SpatialHit ground(const ActorData& body,Fixed distance,uint32_t mask)
```

- **Declared at:** [line 47](../../../runtime/character3d.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `body` | `const ActorData &` | Input | Value supplied for `body`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `SpatialHit`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & body
// Fixed distance
// uint32_t mask

epok::character3d::WorldQueries& object = /* obtain a valid instance */;

auto result = object.ground(body, distance, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-character3d-worldqueries-move-1"></a>

## `epok::character3d::WorldQueries::move`

**Purpose.** Performs `move` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
MoveResult move(ActorData& body,const Fixed* displacement,uint32_t mask)
```

- **Declared at:** [line 46](../../../runtime/character3d.hpp#L46)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `body` | `ActorData &` | Input/output; inspect the function contract | Value supplied for `body`. See the exact type and module contract. |
| `displacement` | `const Fixed *` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// ActorData & body
// const Fixed * displacement
// uint32_t mask

epok::character3d::WorldQueries& object = /* obtain a valid instance */;

auto result = object.move(body, displacement, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-charactermotor3dcomponent-begin-play-1"></a>

## `epok::CharacterMotor3DComponent::begin_play`

**Purpose.** Begins play as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 209](../../../runtime/character3d.hpp#L209)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-class-id-1"></a>

## `epok::CharacterMotor3DComponent::class_id`

**Purpose.** Performs `class id` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 169](../../../runtime/character3d.hpp#L169)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-clear-velocity-1"></a>

## `epok::CharacterMotor3DComponent::clear_velocity`

**Purpose.** Clears velocity as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void clear_velocity()
```

- **Declared at:** [line 188](../../../runtime/character3d.hpp#L188)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.clear_velocity();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-end-play-1"></a>

## `epok::CharacterMotor3DComponent::end_play`

**Purpose.** Ends play as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 212](../../../runtime/character3d.hpp#L212)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-motion-1"></a>

## `epok::CharacterMotor3DComponent::motion`

**Purpose.** Performs `motion` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
CharacterMotionSample3D motion()const
```

- **Declared at:** [line 182](../../../runtime/character3d.hpp#L182)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

auto result = object.motion();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-on-disable-1"></a>

## `epok::CharacterMotor3DComponent::on_disable`

**Purpose.** Performs `on disable` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 211](../../../runtime/character3d.hpp#L211)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-reset-motion-1"></a>

## `epok::CharacterMotor3DComponent::reset_motion`

**Purpose.** Resets motion as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void reset_motion()
```

- **Declared at:** [line 192](../../../runtime/character3d.hpp#L192)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.reset_motion();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-set-facing-1"></a>

## `epok::CharacterMotor3DComponent::set_facing`

**Purpose.** Assign project-controlled facing without applying a second turn policy.

**Details.** The next successful step applies it to the body, as command turning does.

**Exact declaration**

```cpp
bool set_facing(Fixed degrees)
```

- **Declared at:** [line 185](../../../runtime/character3d.hpp#L185)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The next successful step applies it to the body, as command turning does.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

auto result = object.set_facing(degrees);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-charactermotor3dcomponent-simulate-1"></a>

## `epok::CharacterMotor3DComponent::simulate`

**Purpose.** Performs `simulate` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
CharacterMotionSample3D simulate(Fixed delta_seconds)
```

- **Declared at:** [line 195](../../../runtime/character3d.hpp#L195)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

auto result = object.simulate(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-simulate-shared-1"></a>

## `epok::CharacterMotor3DComponent::simulate_shared`

**Purpose.** Performs `simulate shared` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
CharacterMotionSample3D simulate_shared(Fixed delta_seconds,ObjectId query_context,uint32_t support_revision)
```

- **Declared at:** [line 200](../../../runtime/character3d.hpp#L200)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |
| `query_context` | `ObjectId` | Input | Value supplied for `query_context`. See the exact type and module contract. |
| `support_revision` | `uint32_t` | Input | Value supplied for `support_revision`. See the exact type and module contract. |

**Returns.** Returns `CharacterMotionSample3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds
// ObjectId query_context
// uint32_t support_revision

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

auto result = object.simulate_shared(delta_seconds, query_context, support_revision);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-stop-1"></a>

## `epok::CharacterMotor3DComponent::stop`

**Purpose.** Stops stop as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 181](../../../runtime/character3d.hpp#L181)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-submit-1"></a>

## `epok::CharacterMotor3DComponent::submit`

**Purpose.** Performs `submit` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void submit(CharacterCommand3D command)
```

- **Declared at:** [line 180](../../../runtime/character3d.hpp#L180)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand3D` | Input | Value supplied for `command`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand3D command

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.submit(command);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor3dcomponent-tick-1"></a>

## `epok::CharacterMotor3DComponent::tick`

**Purpose.** Performs `tick` as part of shared world-unit character commands, collision locomotion and motion samples.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 210](../../../runtime/character3d.hpp#L210)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need shared world-unit character commands, collision locomotion and motion samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character3d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CharacterMotor3DComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to shared world-unit character commands, collision locomotion and motion samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
