# Epok API: Sprite Types

> **Header:** `"sprite_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sprite_types.hpp)

This module covers sprites, flipbooks and screen-facing rendering. It documents 8 public callables declared directly in this header.

## Declared types

`epok::Fixed`, `epok::Sprite`, `epok::SpriteAnimator`, `epok::SpriteClip`, `epok::SpriteFrame`, `epok::SpriteOrientation`, `epok::SpriteStats`

## Callable index

- [`epok::SpriteAnimator::advance`](#epok-spriteanimator-advance-1) — Performs `advance` as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::apply`](#epok-spriteanimator-apply-1) — Performs `apply` as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::emit`](#epok-spriteanimator-emit-1) — Performs `emit` as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::pause`](#epok-spriteanimator-pause-1) — Pauses pause as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::play`](#epok-spriteanimator-play-1) — Starts play as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::poll_event`](#epok-spriteanimator-poll-event-1) — Polls event as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::resume`](#epok-spriteanimator-resume-1) — Resumes resume as part of sprites, flipbooks and screen-facing rendering.
- [`epok::SpriteAnimator::take_completion`](#epok-spriteanimator-take-completion-1) — Performs `take completion` as part of sprites, flipbooks and screen-facing rendering.

<a id="epok-spriteanimator-advance-1"></a>

## `epok::SpriteAnimator::advance`

**Purpose.** Performs `advance` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void advance(Fixed dt,Sprite& sprite)
```

- **Declared at:** [line 27](../../../runtime/sprite_types.hpp#L27)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `sprite` | `Sprite &` | Input/output; inspect the function contract | Value supplied for `sprite`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// Sprite & sprite

epok::SpriteAnimator& object = /* obtain a valid instance */;

object.advance(dt, sprite);
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spriteanimator-apply-1"></a>

## `epok::SpriteAnimator::apply`

**Purpose.** Performs `apply` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void apply(Sprite& sprite) const
```

- **Declared at:** [line 26](../../../runtime/sprite_types.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sprite` | `Sprite &` | Input/output; inspect the function contract | Value supplied for `sprite`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

// Assume these named values have been initialized with valid data:
// Sprite & sprite

epok::SpriteAnimator& object = /* obtain a valid instance */;

object.apply(sprite);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spriteanimator-emit-1"></a>

## `epok::SpriteAnimator::emit`

**Purpose.** Performs `emit` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void emit(uint16_t event)
```

- **Declared at:** [line 21](../../../runtime/sprite_types.hpp#L21)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `uint16_t` | Input | Value supplied for `event`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t event

epok::SpriteAnimator& object = /* obtain a valid instance */;

object.emit(event);
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spriteanimator-pause-1"></a>

## `epok::SpriteAnimator::pause`

**Purpose.** Pauses pause as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void pause()
```

- **Declared at:** [line 25](../../../runtime/sprite_types.hpp#L25)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

epok::SpriteAnimator& object = /* obtain a valid instance */;

object.pause();
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spriteanimator-play-1"></a>

## `epok::SpriteAnimator::play`

**Purpose.** Starts play as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
bool play(uint16_t index)
```

- **Declared at:** [line 23](../../../runtime/sprite_types.hpp#L23)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint16_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t index

epok::SpriteAnimator& object = /* obtain a valid instance */;

auto result = object.play(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-spriteanimator-poll-event-1"></a>

## `epok::SpriteAnimator::poll_event`

**Purpose.** Polls event as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
bool poll_event(uint16_t& event)
```

- **Declared at:** [line 22](../../../runtime/sprite_types.hpp#L22)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `uint16_t &` | Input/output; inspect the function contract | Value supplied for `event`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t & event

epok::SpriteAnimator& object = /* obtain a valid instance */;

auto result = object.poll_event(event);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-spriteanimator-resume-1"></a>

## `epok::SpriteAnimator::resume`

**Purpose.** Resumes resume as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
void resume()
```

- **Declared at:** [line 25](../../../runtime/sprite_types.hpp#L25)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

epok::SpriteAnimator& object = /* obtain a valid instance */;

object.resume();
```

**Why choose it.** It provides direct, allocation-conscious access to sprites, flipbooks and screen-facing rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spriteanimator-take-completion-1"></a>

## `epok::SpriteAnimator::take_completion`

**Purpose.** Performs `take completion` as part of sprites, flipbooks and screen-facing rendering.

**Exact declaration**

```cpp
bool take_completion()
```

- **Declared at:** [line 24](../../../runtime/sprite_types.hpp#L24)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need sprites, flipbooks and screen-facing rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sprite_types.hpp"

epok::SpriteAnimator& object = /* obtain a valid instance */;

auto result = object.take_completion();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
