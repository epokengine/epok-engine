# Epok API: Character Commands

> **Header:** `"character_commands.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/character_commands.hpp)

This module covers extensible command producers and point/target movement policies for shared character motors. It documents 44 public callables declared directly in this header.

## Declared types

`epok::CharacterCommandResult2D`, `epok::CharacterCommandResult3D`, `epok::CharacterCommandSource2DComponent`, `epok::CharacterCommandSource3DComponent`, `epok::CharacterCommandStatus`, `epok::detail::CharacterCommandLease`, `epok::detail::CharacterCommandUpdateScope`, `epok::MoveTo2DComponent`, `epok::MoveTo3DComponent`, `epok::NPC2D`, `epok::NPC3D`, `epok::ScriptedCommand2DComponent`, `epok::ScriptedCommand3DComponent`

## Callable index

- [`epok::CharacterCommandSource2DComponent::begin_play`](#epok-charactercommandsource2dcomponent-begin-play-1) — Begins play as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::calculate_command`](#epok-charactercommandsource2dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::class_id`](#epok-charactercommandsource2dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::command_submitted`](#epok-charactercommandsource2dcomponent-command-submitted-1) — Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::end_play`](#epok-charactercommandsource2dcomponent-end-play-1) — Ends play as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::on_disable`](#epok-charactercommandsource2dcomponent-on-disable-1) — Performs `on disable` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::release_command`](#epok-charactercommandsource2dcomponent-release-command-1) — Performs `release command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::sample`](#epok-charactercommandsource2dcomponent-sample-1) — Performs `sample` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::set_enabled`](#epok-charactercommandsource2dcomponent-set-enabled-1) — Sets enabled as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::tick`](#epok-charactercommandsource2dcomponent-tick-1) — Performs `tick` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource2DComponent::update`](#epok-charactercommandsource2dcomponent-update-1) — Updates update as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::begin_play`](#epok-charactercommandsource3dcomponent-begin-play-1) — Begins play as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::calculate_command`](#epok-charactercommandsource3dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::class_id`](#epok-charactercommandsource3dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::command_submitted`](#epok-charactercommandsource3dcomponent-command-submitted-1) — Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::end_play`](#epok-charactercommandsource3dcomponent-end-play-1) — Ends play as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::on_disable`](#epok-charactercommandsource3dcomponent-on-disable-1) — Performs `on disable` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::release_command`](#epok-charactercommandsource3dcomponent-release-command-1) — Performs `release command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::sample`](#epok-charactercommandsource3dcomponent-sample-1) — Performs `sample` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::set_enabled`](#epok-charactercommandsource3dcomponent-set-enabled-1) — Sets enabled as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::tick`](#epok-charactercommandsource3dcomponent-tick-1) — Performs `tick` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::CharacterCommandSource3DComponent::update`](#epok-charactercommandsource3dcomponent-update-1) — Updates update as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::character_planar_velocity`](#epok-detail-character-planar-velocity-1) — Performs `character planar velocity` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::character_position_2d`](#epok-detail-character-position-2d-1) — Performs `character position 2d` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::CharacterCommandLease::release`](#epok-detail-charactercommandlease-release-1) — Performs `release` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::CharacterCommandLease::resolve`](#epok-detail-charactercommandlease-resolve-1) — Performs `resolve` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::CharacterCommandLease::submit`](#epok-detail-charactercommandlease-submit-1) — Performs `submit` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::CharacterCommandUpdateScope::CharacterCommandUpdateScope`](#epok-detail-charactercommandupdatescope-charactercommandupdatescope-1) — Constructs `epok::detail::CharacterCommandUpdateScope` for extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::CharacterCommandUpdateScope::~CharacterCommandUpdateScope`](#epok-detail-charactercommandupdatescope-charactercommandupdatescope-2) — Releases the resources owned by `epok::detail::CharacterCommandUpdateScope`.
- [`epok::detail::prepare_character_command`](#epok-detail-prepare-character-command-1) — Performs `prepare character command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::valid_character_command`](#epok-detail-valid-character-command-1) — Performs `valid character command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::detail::valid_character_command`](#epok-detail-valid-character-command-2) — Performs `valid character command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::MoveTo2DComponent::calculate_command`](#epok-moveto2dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::MoveTo2DComponent::class_id`](#epok-moveto2dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::MoveTo3DComponent::calculate_command`](#epok-moveto3dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::MoveTo3DComponent::class_id`](#epok-moveto3dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::NPC2D::class_id`](#epok-npc2d-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::NPC3D::class_id`](#epok-npc3d-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand2DComponent::calculate_command`](#epok-scriptedcommand2dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand2DComponent::class_id`](#epok-scriptedcommand2dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand2DComponent::command_submitted`](#epok-scriptedcommand2dcomponent-command-submitted-1) — Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand3DComponent::calculate_command`](#epok-scriptedcommand3dcomponent-calculate-command-1) — Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand3DComponent::class_id`](#epok-scriptedcommand3dcomponent-class-id-1) — Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.
- [`epok::ScriptedCommand3DComponent::command_submitted`](#epok-scriptedcommand3dcomponent-command-submitted-1) — Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.

<a id="epok-charactercommandsource2dcomponent-begin-play-1"></a>

## `epok::CharacterCommandSource2DComponent::begin_play`

**Purpose.** Begins play as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 137](../../../runtime/character_commands.hpp#L137)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-calculate-command-1"></a>

## `epok::CharacterCommandSource2DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
virtual CharacterCommandResult2D calculate_command(Fixed delta_seconds)
```

- **Declared at:** [line 125](../../../runtime/character_commands.hpp#L125)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-class-id-1"></a>

## `epok::CharacterCommandSource2DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 121](../../../runtime/character_commands.hpp#L121)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-command-submitted-1"></a>

## `epok::CharacterCommandSource2DComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
virtual void command_submitted(CharacterCommandResult2D result)
```

- **Declared at:** [line 126](../../../runtime/character_commands.hpp#L126)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `CharacterCommandResult2D` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult2D result

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.command_submitted(result);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-end-play-1"></a>

## `epok::CharacterCommandSource2DComponent::end_play`

**Purpose.** Ends play as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 140](../../../runtime/character_commands.hpp#L140)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-on-disable-1"></a>

## `epok::CharacterCommandSource2DComponent::on_disable`

**Purpose.** Performs `on disable` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 139](../../../runtime/character_commands.hpp#L139)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-release-command-1"></a>

## `epok::CharacterCommandSource2DComponent::release_command`

**Purpose.** Performs `release command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
bool release_command()
```

- **Declared at:** [line 127](../../../runtime/character_commands.hpp#L127)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

auto result = object.release_command();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-charactercommandsource2dcomponent-sample-1"></a>

## `epok::CharacterCommandSource2DComponent::sample`

**Purpose.** Performs `sample` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult2D sample()const
```

- **Declared at:** [line 129](../../../runtime/character_commands.hpp#L129)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-set-enabled-1"></a>

## `epok::CharacterCommandSource2DComponent::set_enabled`

**Purpose.** Sets enabled as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 128](../../../runtime/character_commands.hpp#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-tick-1"></a>

## `epok::CharacterCommandSource2DComponent::tick`

**Purpose.** Performs `tick` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 138](../../../runtime/character_commands.hpp#L138)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource2dcomponent-update-1"></a>

## `epok::CharacterCommandSource2DComponent::update`

**Purpose.** Updates update as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult2D update(Fixed delta_seconds)
```

- **Declared at:** [line 130](../../../runtime/character_commands.hpp#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterCommandSource2DComponent& object = /* obtain a valid instance */;

auto result = object.update(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-begin-play-1"></a>

## `epok::CharacterCommandSource3DComponent::begin_play`

**Purpose.** Begins play as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 108](../../../runtime/character_commands.hpp#L108)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-calculate-command-1"></a>

## `epok::CharacterCommandSource3DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
virtual CharacterCommandResult3D calculate_command(Fixed delta_seconds)
```

- **Declared at:** [line 96](../../../runtime/character_commands.hpp#L96)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-class-id-1"></a>

## `epok::CharacterCommandSource3DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 92](../../../runtime/character_commands.hpp#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-command-submitted-1"></a>

## `epok::CharacterCommandSource3DComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
virtual void command_submitted(CharacterCommandResult3D result)
```

- **Declared at:** [line 97](../../../runtime/character_commands.hpp#L97)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `CharacterCommandResult3D` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult3D result

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.command_submitted(result);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-end-play-1"></a>

## `epok::CharacterCommandSource3DComponent::end_play`

**Purpose.** Ends play as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 111](../../../runtime/character_commands.hpp#L111)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-on-disable-1"></a>

## `epok::CharacterCommandSource3DComponent::on_disable`

**Purpose.** Performs `on disable` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 110](../../../runtime/character_commands.hpp#L110)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-release-command-1"></a>

## `epok::CharacterCommandSource3DComponent::release_command`

**Purpose.** Performs `release command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
bool release_command()
```

- **Declared at:** [line 98](../../../runtime/character_commands.hpp#L98)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

auto result = object.release_command();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-charactercommandsource3dcomponent-sample-1"></a>

## `epok::CharacterCommandSource3DComponent::sample`

**Purpose.** Performs `sample` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult3D sample()const
```

- **Declared at:** [line 100](../../../runtime/character_commands.hpp#L100)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-set-enabled-1"></a>

## `epok::CharacterCommandSource3DComponent::set_enabled`

**Purpose.** Sets enabled as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 99](../../../runtime/character_commands.hpp#L99)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-tick-1"></a>

## `epok::CharacterCommandSource3DComponent::tick`

**Purpose.** Performs `tick` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 109](../../../runtime/character_commands.hpp#L109)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-charactercommandsource3dcomponent-update-1"></a>

## `epok::CharacterCommandSource3DComponent::update`

**Purpose.** Updates update as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult3D update(Fixed delta_seconds)
```

- **Declared at:** [line 101](../../../runtime/character_commands.hpp#L101)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed delta_seconds

epok::CharacterCommandSource3DComponent& object = /* obtain a valid instance */;

auto result = object.update(delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-character-planar-velocity-1"></a>

## `epok::detail::character_planar_velocity`

**Purpose.** Performs `character planar velocity` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
inline bool character_planar_velocity(Fixed from_x,Fixed from_y,Fixed to_x,Fixed to_y,Fixed speed,Fixed radius,Fixed dt,GameplayVector2& velocity)
```

- **Declared at:** [line 76](../../../runtime/character_commands.hpp#L76)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from_x` | `Fixed` | Input | Value supplied for `from_x`. See the exact type and module contract. |
| `from_y` | `Fixed` | Input | Value supplied for `from_y`. See the exact type and module contract. |
| `to_x` | `Fixed` | Input | Value supplied for `to_x`. See the exact type and module contract. |
| `to_y` | `Fixed` | Input | Value supplied for `to_y`. See the exact type and module contract. |
| `speed` | `Fixed` | Input | Value supplied for `speed`. See the exact type and module contract. |
| `radius` | `Fixed` | Input | Value supplied for `radius`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `velocity` | `GameplayVector2 &` | Input/output; inspect the function contract | Value supplied for `velocity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed from_x
// Fixed from_y
// Fixed to_x
// Fixed to_y
// Fixed speed
// Fixed radius
// Fixed dt
// GameplayVector2 & velocity

auto result = epok::detail::character_planar_velocity(from_x, from_y, to_x, to_y, speed, radius, dt, velocity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-character-position-2d-1"></a>

## `epok::detail::character_position_2d`

**Purpose.** Performs `character position 2d` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
inline bool character_position_2d(Actor& actor,GameplayVector2& result)
```

- **Declared at:** [line 71](../../../runtime/character_commands.hpp#L71)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor &` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |
| `result` | `GameplayVector2 &` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Actor & actor
// GameplayVector2 & result

auto result = epok::detail::character_position_2d(actor, result);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-charactercommandlease-release-1"></a>

## `epok::detail::CharacterCommandLease::release`

**Purpose.** Performs `release` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
bool release()
```

- **Declared at:** [line 22](../../../runtime/character_commands.hpp#L22)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::detail::CharacterCommandLease& object = /* obtain a valid instance */;

auto result = object.release();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-detail-charactercommandlease-resolve-1"></a>

## `epok::detail::CharacterCommandLease::resolve`

**Purpose.** Performs `resolve` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
static Motor* resolve(ActorComponent& source,ObjectId binding,ObjectRegistry* registry)
```

- **Declared at:** [line 27](../../../runtime/character_commands.hpp#L27)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |
| `binding` | `ObjectId` | Input | Value supplied for `binding`. See the exact type and module contract. |
| `registry` | `ObjectRegistry *` | Input/output; inspect the function contract | Value supplied for `registry`. See the exact type and module contract. |

**Returns.** Returns `Motor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & source
// ObjectId binding
// ObjectRegistry * registry

auto result = epok::detail::CharacterCommandLease::resolve(source, binding, registry);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-charactercommandlease-submit-1"></a>

## `epok::detail::CharacterCommandLease::submit`

**Purpose.** Performs `submit` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
template<class Command> void submit(Motor& value,Command command,ObjectRegistry* current)
```

- **Declared at:** [line 35](../../../runtime/character_commands.hpp#L35)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Motor &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |
| `command` | `Command` | Input | Value supplied for `command`. See the exact type and module contract. |
| `current` | `ObjectRegistry *` | Input/output; inspect the function contract | Value supplied for `current`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Command

// Assume these named values have been initialized with valid data:
// Motor & value
// Command command
// ObjectRegistry * current

epok::detail::CharacterCommandLease& object = /* obtain a valid instance */;

object.submit<Command>(value, command, current);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-charactercommandupdatescope-charactercommandupdatescope-1"></a>

## `epok::detail::CharacterCommandUpdateScope::CharacterCommandUpdateScope`

**Purpose.** Constructs `epok::detail::CharacterCommandUpdateScope` for extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
explicit CharacterCommandUpdateScope(bool& flag):updating(flag)
```

- **Declared at:** [line 41](../../../runtime/character_commands.hpp#L41)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `flag` | `bool &` | Input/output; inspect the function contract | Value supplied for `flag`. See the exact type and module contract. |

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// bool & flag

epok::detail::CharacterCommandUpdateScope value(flag);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-charactercommandupdatescope-charactercommandupdatescope-2"></a>

## `epok::detail::CharacterCommandUpdateScope::~CharacterCommandUpdateScope`

**Purpose.** Releases the resources owned by `epok::detail::CharacterCommandUpdateScope`.

**Exact declaration**

```cpp
~CharacterCommandUpdateScope()
```

- **Declared at:** [line 42](../../../runtime/character_commands.hpp#L42)
- **Kind:** `destructor`

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// `epok::detail::CharacterCommandUpdateScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-detail-prepare-character-command-1"></a>

## `epok::detail::prepare_character_command`

**Purpose.** Performs `prepare character command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
template<class Source,class Motor,class Result> Result prepare_character_command(Source& source,CharacterCommandLease<Motor>& lease,Fixed dt,ObjectRegistry* registry)
```

- **Declared at:** [line 49](../../../runtime/character_commands.hpp#L49)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `Source &` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |
| `lease` | `CharacterCommandLease<Motor> &` | Input/output; inspect the function contract | Value supplied for `lease`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `registry` | `ObjectRegistry *` | Input/output; inspect the function contract | Value supplied for `registry`. See the exact type and module contract. |

**Returns.** Returns `Result`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Source, Motor, Result

// Assume these named values have been initialized with valid data:
// Source & source
// CharacterCommandLease<Motor> & lease
// Fixed dt
// ObjectRegistry * registry

epok::detail& object = /* obtain a valid instance */;

auto result = object.prepare_character_command<Source, Motor, Result>(source, lease, dt, registry);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-valid-character-command-1"></a>

## `epok::detail::valid_character_command`

**Purpose.** Performs `valid character command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
inline bool valid_character_command(CharacterCommand2D& command)
```

- **Declared at:** [line 48](../../../runtime/character_commands.hpp#L48)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand2D &` | Input/output; inspect the function contract | Value supplied for `command`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand2D & command

auto result = epok::detail::valid_character_command(command);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-detail-valid-character-command-2"></a>

## `epok::detail::valid_character_command`

**Purpose.** Performs `valid character command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
inline bool valid_character_command(CharacterCommand3D& command)
```

- **Declared at:** [line 44](../../../runtime/character_commands.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `command` | `CharacterCommand3D &` | Input/output; inspect the function contract | Value supplied for `command`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommand3D & command

auto result = epok::detail::valid_character_command(command);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-moveto2dcomponent-calculate-command-1"></a>

## `epok::MoveTo2DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult2D calculate_command(Fixed dt)override
```

- **Declared at:** [line 195](../../../runtime/character_commands.hpp#L195)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::MoveTo2DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-moveto2dcomponent-class-id-1"></a>

## `epok::MoveTo2DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 189](../../../runtime/character_commands.hpp#L189)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::MoveTo2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-moveto3dcomponent-calculate-command-1"></a>

## `epok::MoveTo3DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult3D calculate_command(Fixed dt)override
```

- **Declared at:** [line 168](../../../runtime/character_commands.hpp#L168)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::MoveTo3DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-moveto3dcomponent-class-id-1"></a>

## `epok::MoveTo3DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 161](../../../runtime/character_commands.hpp#L161)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::MoveTo3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-npc2d-class-id-1"></a>

## `epok::NPC2D::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 229](../../../runtime/character_commands.hpp#L229)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::NPC2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-npc3d-class-id-1"></a>

## `epok::NPC3D::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 223](../../../runtime/character_commands.hpp#L223)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::NPC3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand2dcomponent-calculate-command-1"></a>

## `epok::ScriptedCommand2DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult2D calculate_command(Fixed)override
```

- **Declared at:** [line 155](../../../runtime/character_commands.hpp#L155)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::ScriptedCommand2DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand2dcomponent-class-id-1"></a>

## `epok::ScriptedCommand2DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 153](../../../runtime/character_commands.hpp#L153)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::ScriptedCommand2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand2dcomponent-command-submitted-1"></a>

## `epok::ScriptedCommand2DComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void command_submitted(CharacterCommandResult2D)override
```

- **Declared at:** [line 156](../../../runtime/character_commands.hpp#L156)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `CharacterCommandResult2D` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult2D arg1

epok::ScriptedCommand2DComponent& object = /* obtain a valid instance */;

object.command_submitted(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand3dcomponent-calculate-command-1"></a>

## `epok::ScriptedCommand3DComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
CharacterCommandResult3D calculate_command(Fixed)override
```

- **Declared at:** [line 147](../../../runtime/character_commands.hpp#L147)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::ScriptedCommand3DComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand3dcomponent-class-id-1"></a>

## `epok::ScriptedCommand3DComponent::class_id`

**Purpose.** Performs `class id` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 145](../../../runtime/character_commands.hpp#L145)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

epok::ScriptedCommand3DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scriptedcommand3dcomponent-command-submitted-1"></a>

## `epok::ScriptedCommand3DComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of extensible command producers and point/target movement policies for shared character motors.

**Exact declaration**

```cpp
void command_submitted(CharacterCommandResult3D)override
```

- **Declared at:** [line 148](../../../runtime/character_commands.hpp#L148)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `CharacterCommandResult3D` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need extensible command producers and point/target movement policies for shared character motors and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_commands.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult3D arg1

epok::ScriptedCommand3DComponent& object = /* obtain a valid instance */;

object.command_submitted(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to extensible command producers and point/target movement policies for shared character motors. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
