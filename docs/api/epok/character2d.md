# Epok API: Character2D

> **Header:** `"character2d.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/character2d.hpp)

This module covers pixel-unit character commands and derivable actors over the existing 2D body solver. It documents 17 public callables declared directly in this header.

## Declared types

`epok::Character2D`, `epok::CharacterCommand2D`, `epok::CharacterMotionSample2D`, `epok::CharacterMotor2DComponent`

## Callable index

- [`epok::character2d::bounded`](#epok-character2d-bounded-1) — Performs `bounded` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::Character2D::class_id`](#epok-character2d-class-id-1) — Performs `class id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::Character2D::motion`](#epok-character2d-motion-1) — Performs `motion` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::Character2D::motor_id`](#epok-character2d-motor-id-1) — Performs `motor id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::character2d::normalized_up`](#epok-character2d-normalized-up-1) — Performs `normalized up` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::Character2D::stop_movement`](#epok-character2d-stop-movement-1) — Stops movement as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::Character2D::submit_command`](#epok-character2d-submit-command-1) — Performs `submit command` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::begin_play`](#epok-charactermotor2dcomponent-begin-play-1) — Begins play as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::class_id`](#epok-charactermotor2dcomponent-class-id-1) — Performs `class id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::end_play`](#epok-charactermotor2dcomponent-end-play-1) — Ends play as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::motion`](#epok-charactermotor2dcomponent-motion-1) — Performs `motion` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::on_disable`](#epok-charactermotor2dcomponent-on-disable-1) — Performs `on disable` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::reset_motion`](#epok-charactermotor2dcomponent-reset-motion-1) — Resets motion as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::simulate`](#epok-charactermotor2dcomponent-simulate-1) — Performs `simulate` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::stop`](#epok-charactermotor2dcomponent-stop-1) — Stops stop as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::submit`](#epok-charactermotor2dcomponent-submit-1) — Performs `submit` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.
- [`epok::CharacterMotor2DComponent::tick`](#epok-charactermotor2dcomponent-tick-1) — Performs `tick` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

<a id="epok-character2d-bounded-1"></a>

## `epok::character2d::bounded`

**Purpose.** Performs `bounded` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
inline bool bounded(Fixed value,int32_t limit=32768)
```

- **Declared at:** [line 20](../../../runtime/character2d.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `limit` | `int32_t` | Input | Value supplied for `limit`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// int32_t limit

auto result = epok::character2d::bounded(value, limit);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-character2d-class-id-1"></a>

## `epok::Character2D::class_id`

**Purpose.** Performs `class id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 101](../../../runtime/character2d.hpp#L101)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::Character2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character2d-motion-1"></a>

## `epok::Character2D::motion`

**Purpose.** Performs `motion` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
CharacterMotionSample2D motion()const
```

- **Declared at:** [line 109](../../../runtime/character2d.hpp#L109)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterMotionSample2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::Character2D& object = /* obtain a valid instance */;

auto result = object.motion();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character2d-motor-id-1"></a>

## `epok::Character2D::motor_id`

**Purpose.** Performs `motor id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
ObjectId motor_id()const
```

- **Declared at:** [line 110](../../../runtime/character2d.hpp#L110)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::Character2D& object = /* obtain a valid instance */;

auto result = object.motor_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character2d-normalized-up-1"></a>

## `epok::character2d::normalized_up`

**Purpose.** Performs `normalized up` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
inline bool normalized_up(Fixed x,Fixed y)
```

- **Declared at:** [line 21](../../../runtime/character2d.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::character2d::normalized_up(x, y);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-character2d-stop-movement-1"></a>

## `epok::Character2D::stop_movement`

**Purpose.** Stops movement as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void stop_movement()
```

- **Declared at:** [line 108](../../../runtime/character2d.hpp#L108)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::Character2D& object = /* obtain a valid instance */;

object.stop_movement();
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-character2d-submit-command-1"></a>

## `epok::Character2D::submit_command`

**Purpose.** Performs `submit command` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void submit_command(CharacterCommand2D command)
```

- **Declared at:** [line 107](../../../runtime/character2d.hpp#L107)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand2D` | Input | Value supplied for `command`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand2D command

epok::Character2D& object = /* obtain a valid instance */;

object.submit_command(command);
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-begin-play-1"></a>

## `epok::CharacterMotor2DComponent::begin_play`

**Purpose.** Begins play as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 93](../../../runtime/character2d.hpp#L93)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-class-id-1"></a>

## `epok::CharacterMotor2DComponent::class_id`

**Purpose.** Performs `class id` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 42](../../../runtime/character2d.hpp#L42)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-end-play-1"></a>

## `epok::CharacterMotor2DComponent::end_play`

**Purpose.** Ends play as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 96](../../../runtime/character2d.hpp#L96)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-motion-1"></a>

## `epok::CharacterMotor2DComponent::motion`

**Purpose.** Performs `motion` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
CharacterMotionSample2D motion()const
```

- **Declared at:** [line 49](../../../runtime/character2d.hpp#L49)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterMotionSample2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

auto result = object.motion();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-on-disable-1"></a>

## `epok::CharacterMotor2DComponent::on_disable`

**Purpose.** Performs `on disable` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 95](../../../runtime/character2d.hpp#L95)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-reset-motion-1"></a>

## `epok::CharacterMotor2DComponent::reset_motion`

**Purpose.** Resets motion as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void reset_motion()
```

- **Declared at:** [line 50](../../../runtime/character2d.hpp#L50)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.reset_motion();
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-simulate-1"></a>

## `epok::CharacterMotor2DComponent::simulate`

**Purpose.** Performs `simulate` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
CharacterMotionSample2D simulate(Fixed delta_seconds)
```

- **Declared at:** [line 58](../../../runtime/character2d.hpp#L58)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterMotionSample2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

auto result = object.simulate(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-stop-1"></a>

## `epok::CharacterMotor2DComponent::stop`

**Purpose.** Stops stop as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 48](../../../runtime/character2d.hpp#L48)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-submit-1"></a>

## `epok::CharacterMotor2DComponent::submit`

**Purpose.** Performs `submit` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void submit(CharacterCommand2D command)
```

- **Declared at:** [line 47](../../../runtime/character2d.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand2D` | Input | Value supplied for `command`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand2D command

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.submit(command);
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactermotor2dcomponent-tick-1"></a>

## `epok::CharacterMotor2DComponent::tick`

**Purpose.** Performs `tick` as part of pixel-unit character commands and derivable actors over the existing 2D body solver.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 94](../../../runtime/character2d.hpp#L94)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need pixel-unit character commands and derivable actors over the existing 2D body solver and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character2d.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CharacterMotor2DComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to pixel-unit character commands and derivable actors over the existing 2D body solver. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
