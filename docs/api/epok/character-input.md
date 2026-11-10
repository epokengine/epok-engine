# Epok API: Character Input

> **Header:** `"character_input.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/character_input.hpp)

This module covers routed local-player command sources and derivable player presets. It documents 8 public callables declared directly in this header.

## Declared types

`epok::LocalPlayer2DInputComponent`, `epok::LocalPlayer3DInputComponent`, `epok::Player2D`, `epok::Player3D`

## Callable index

- [`epok::LocalPlayer2DInputComponent::calculate_command`](#epok-localplayer2dinputcomponent-calculate-command-1) — Performs `calculate command` as part of routed local-player command sources and derivable player presets.
- [`epok::LocalPlayer2DInputComponent::class_id`](#epok-localplayer2dinputcomponent-class-id-1) — Performs `class id` as part of routed local-player command sources and derivable player presets.
- [`epok::LocalPlayer2DInputComponent::command_submitted`](#epok-localplayer2dinputcomponent-command-submitted-1) — Performs `command submitted` as part of routed local-player command sources and derivable player presets.
- [`epok::LocalPlayer3DInputComponent::calculate_command`](#epok-localplayer3dinputcomponent-calculate-command-1) — Performs `calculate command` as part of routed local-player command sources and derivable player presets.
- [`epok::LocalPlayer3DInputComponent::class_id`](#epok-localplayer3dinputcomponent-class-id-1) — Performs `class id` as part of routed local-player command sources and derivable player presets.
- [`epok::LocalPlayer3DInputComponent::command_submitted`](#epok-localplayer3dinputcomponent-command-submitted-1) — Performs `command submitted` as part of routed local-player command sources and derivable player presets.
- [`epok::Player2D::class_id`](#epok-player2d-class-id-1) — Performs `class id` as part of routed local-player command sources and derivable player presets.
- [`epok::Player3D::class_id`](#epok-player3d-class-id-1) — Performs `class id` as part of routed local-player command sources and derivable player presets.

<a id="epok-localplayer2dinputcomponent-calculate-command-1"></a>

## `epok::LocalPlayer2DInputComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
CharacterCommandResult2D calculate_command(Fixed)override
```

- **Declared at:** [line 57](../../../runtime/character_input.hpp#L57)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::LocalPlayer2DInputComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to routed local-player command sources and derivable player presets. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localplayer2dinputcomponent-class-id-1"></a>

## `epok::LocalPlayer2DInputComponent::class_id`

**Purpose.** Performs `class id` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 53](../../../runtime/character_input.hpp#L53)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

epok::LocalPlayer2DInputComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localplayer2dinputcomponent-command-submitted-1"></a>

## `epok::LocalPlayer2DInputComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
void command_submitted(CharacterCommandResult2D result)override
```

- **Declared at:** [line 73](../../../runtime/character_input.hpp#L73)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `CharacterCommandResult2D` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult2D result

epok::LocalPlayer2DInputComponent& object = /* obtain a valid instance */;

object.command_submitted(result);
```

**Why choose it.** It provides direct, allocation-conscious access to routed local-player command sources and derivable player presets. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localplayer3dinputcomponent-calculate-command-1"></a>

## `epok::LocalPlayer3DInputComponent::calculate_command`

**Purpose.** Performs `calculate command` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
CharacterCommandResult3D calculate_command(Fixed)override
```

- **Declared at:** [line 19](../../../runtime/character_input.hpp#L19)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Fixed` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `CharacterCommandResult3D`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

// Assume these named values have been initialized with valid data:
// Fixed arg1

epok::LocalPlayer3DInputComponent& object = /* obtain a valid instance */;

auto result = object.calculate_command(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to routed local-player command sources and derivable player presets. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localplayer3dinputcomponent-class-id-1"></a>

## `epok::LocalPlayer3DInputComponent::class_id`

**Purpose.** Performs `class id` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 12](../../../runtime/character_input.hpp#L12)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

epok::LocalPlayer3DInputComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localplayer3dinputcomponent-command-submitted-1"></a>

## `epok::LocalPlayer3DInputComponent::command_submitted`

**Purpose.** Performs `command submitted` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
void command_submitted(CharacterCommandResult3D result)override
```

- **Declared at:** [line 46](../../../runtime/character_input.hpp#L46)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `CharacterCommandResult3D` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

// Assume these named values have been initialized with valid data:
// CharacterCommandResult3D result

epok::LocalPlayer3DInputComponent& object = /* obtain a valid instance */;

object.command_submitted(result);
```

**Why choose it.** It provides direct, allocation-conscious access to routed local-player command sources and derivable player presets. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-player2d-class-id-1"></a>

## `epok::Player2D::class_id`

**Purpose.** Performs `class id` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 86](../../../runtime/character_input.hpp#L86)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

epok::Player2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-player3d-class-id-1"></a>

## `epok::Player3D::class_id`

**Purpose.** Performs `class id` as part of routed local-player command sources and derivable player presets.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 80](../../../runtime/character_input.hpp#L80)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need routed local-player command sources and derivable player presets and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_input.hpp"

epok::Player3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
