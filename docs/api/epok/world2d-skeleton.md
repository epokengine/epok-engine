# Epok API: World2D Skeleton

> **Header:** `"world2d_skeleton.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_skeleton.hpp)

This module covers the world2d skeleton module. It documents 24 public callables declared directly in this header.

## Declared types

`epok::Skeleton2DComponent`, `epok::SkeletonClip2D`, `epok::SkeletonIk2D`

## Callable index

- [`epok::Skeleton2DComponent::advance`](#epok-skeleton2dcomponent-advance-1) — Called by the service every simulation step: advances the animation, poses the bones, then solves the IK chains.
- [`epok::Skeleton2DComponent::apply`](#epok-skeleton2dcomponent-apply-1) — Poses the bones at `seconds` of the current animation.
- [`epok::Skeleton2DComponent::class_id`](#epok-skeleton2dcomponent-class-id-1) — Performs `class id` as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_animation`](#epok-skeleton2dcomponent-get-animation-1) — Returns animation as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_animation_count`](#epok-skeleton2dcomponent-get-animation-count-1) — Returns animation count as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_bone`](#epok-skeleton2dcomponent-get-bone-1) — Returns bone as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_bone_count`](#epok-skeleton2dcomponent-get-bone-count-1) — Returns bone count as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_speed`](#epok-skeleton2dcomponent-get-speed-1) — Returns speed as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::get_time`](#epok-skeleton2dcomponent-get-time-1) — Returns time as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::is_ik_enabled`](#epok-skeleton2dcomponent-is-ik-enabled-1) — Reports whether ik enabled as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::is_playing`](#epok-skeleton2dcomponent-is-playing-1) — Reports whether playing as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::level_attached`](#epok-skeleton2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::play`](#epok-skeleton2dcomponent-play-1) — Starts animation `index` from its beginning; an index out of range stops.
- [`epok::Skeleton2DComponent::reset_pose`](#epok-skeleton2dcomponent-reset-pose-1) — Stops and puts every bone back in its rest pose.
- [`epok::Skeleton2DComponent::seek`](#epok-skeleton2dcomponent-seek-1) — Jumps the current animation to `seconds` and poses the bones at once.
- [`epok::Skeleton2DComponent::set_ik_enabled`](#epok-skeleton2dcomponent-set-ik-enabled-1) — Sets ik enabled as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::set_ik_target`](#epok-skeleton2dcomponent-set-ik-target-1) — Sets ik target as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::set_speed`](#epok-skeleton2dcomponent-set-speed-1) — Sets speed as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::stop`](#epok-skeleton2dcomponent-stop-1) — Stops stop as part of the world2d skeleton module.
- [`epok::Skeleton2DComponent::take_finished`](#epok-skeleton2dcomponent-take-finished-1) — True once after a non-looping animation reaches its end.
- [`epok::Skeleton2DComponent::~Skeleton2DComponent`](#epok-skeleton2dcomponent-skeleton2dcomponent-1) — Releases the resources owned by `epok::Skeleton2DComponent`.
- [`epok::world2d::actor_root`](#epok-world2d-actor-root-1) — The root transform of an actor, or null.
- [`epok::world2d::root_world`](#epok-world2d-root-world-1) — Performs `root world` as part of the world2d skeleton module.
- [`epok::world2d::wrap_degrees_raw`](#epok-world2d-wrap-degrees-raw-1) — An angle in raw Q12 degrees brought into (-180, 180].

<a id="epok-skeleton2dcomponent-advance-1"></a>

## `epok::Skeleton2DComponent::advance`

**Purpose.** Called by the service every simulation step: advances the animation, poses the bones, then solves the IK chains.

**Exact declaration**

```cpp
void advance(Fixed dt)
```

- **Declared at:** [line 136](../../../runtime/world2d_skeleton.hpp#L136)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-apply-1"></a>

## `epok::Skeleton2DComponent::apply`

**Purpose.** Poses the bones at `seconds` of the current animation.

**Exact declaration**

```cpp
void apply(Fixed seconds)
```

- **Declared at:** [line 151](../../../runtime/world2d_skeleton.hpp#L151)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// Fixed seconds

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.apply(seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-class-id-1"></a>

## `epok::Skeleton2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d skeleton module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 66](../../../runtime/world2d_skeleton.hpp#L66)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-animation-1"></a>

## `epok::Skeleton2DComponent::get_animation`

**Purpose.** Returns animation as part of the world2d skeleton module.

**Exact declaration**

```cpp
int32_t get_animation() const
```

- **Declared at:** [line 98](../../../runtime/world2d_skeleton.hpp#L98)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_animation();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-animation-count-1"></a>

## `epok::Skeleton2DComponent::get_animation_count`

**Purpose.** Returns animation count as part of the world2d skeleton module.

**Exact declaration**

```cpp
int32_t get_animation_count() const
```

- **Declared at:** [line 99](../../../runtime/world2d_skeleton.hpp#L99)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_animation_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-bone-1"></a>

## `epok::Skeleton2DComponent::get_bone`

**Purpose.** Returns bone as part of the world2d skeleton module.

**Exact declaration**

```cpp
ObjectId get_bone(int32_t index) const
```

- **Declared at:** [line 121](../../../runtime/world2d_skeleton.hpp#L121)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t index

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_bone(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-bone-count-1"></a>

## `epok::Skeleton2DComponent::get_bone_count`

**Purpose.** Returns bone count as part of the world2d skeleton module.

**Exact declaration**

```cpp
int32_t get_bone_count() const
```

- **Declared at:** [line 120](../../../runtime/world2d_skeleton.hpp#L120)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_bone_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-speed-1"></a>

## `epok::Skeleton2DComponent::get_speed`

**Purpose.** Returns speed as part of the world2d skeleton module.

**Exact declaration**

```cpp
Fixed get_speed() const
```

- **Declared at:** [line 101](../../../runtime/world2d_skeleton.hpp#L101)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_speed();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-get-time-1"></a>

## `epok::Skeleton2DComponent::get_time`

**Purpose.** Returns time as part of the world2d skeleton module.

**Exact declaration**

```cpp
Fixed get_time() const
```

- **Declared at:** [line 108](../../../runtime/world2d_skeleton.hpp#L108)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.get_time();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-is-ik-enabled-1"></a>

## `epok::Skeleton2DComponent::is_ik_enabled`

**Purpose.** Reports whether ik enabled as part of the world2d skeleton module.

**Exact declaration**

```cpp
bool is_ik_enabled(int32_t chain) const
```

- **Declared at:** [line 127](../../../runtime/world2d_skeleton.hpp#L127)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chain` | `int32_t` | Input | Value supplied for `chain`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t chain

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.is_ik_enabled(chain);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-skeleton2dcomponent-is-playing-1"></a>

## `epok::Skeleton2DComponent::is_playing`

**Purpose.** Reports whether playing as part of the world2d skeleton module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 97](../../../runtime/world2d_skeleton.hpp#L97)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-skeleton2dcomponent-level-attached-1"></a>

## `epok::Skeleton2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d skeleton module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 67](../../../runtime/world2d_skeleton.hpp#L67)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-play-1"></a>

## `epok::Skeleton2DComponent::play`

**Purpose.** Starts animation `index` from its beginning; an index out of range stops.

**Exact declaration**

```cpp
void play(int32_t index)
```

- **Declared at:** [line 85](../../../runtime/world2d_skeleton.hpp#L85)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t index

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.play(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-reset-pose-1"></a>

## `epok::Skeleton2DComponent::reset_pose`

**Purpose.** Stops and puts every bone back in its rest pose.

**Exact declaration**

```cpp
void reset_pose()
```

- **Declared at:** [line 116](../../../runtime/world2d_skeleton.hpp#L116)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.reset_pose();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-seek-1"></a>

## `epok::Skeleton2DComponent::seek`

**Purpose.** Jumps the current animation to `seconds` and poses the bones at once.

**Exact declaration**

```cpp
void seek(Fixed seconds)
```

- **Declared at:** [line 103](../../../runtime/world2d_skeleton.hpp#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// Fixed seconds

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.seek(seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-set-ik-enabled-1"></a>

## `epok::Skeleton2DComponent::set_ik_enabled`

**Purpose.** Sets ik enabled as part of the world2d skeleton module.

**Exact declaration**

```cpp
void set_ik_enabled(int32_t chain, bool value)
```

- **Declared at:** [line 124](../../../runtime/world2d_skeleton.hpp#L124)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chain` | `int32_t` | Input | Value supplied for `chain`. See the exact type and module contract. |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t chain
// bool value

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.set_ik_enabled(chain, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-set-ik-target-1"></a>

## `epok::Skeleton2DComponent::set_ik_target`

**Purpose.** Sets ik target as part of the world2d skeleton module.

**Exact declaration**

```cpp
void set_ik_target(int32_t chain, ObjectId actor)
```

- **Declared at:** [line 130](../../../runtime/world2d_skeleton.hpp#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `chain` | `int32_t` | Input | Value supplied for `chain`. See the exact type and module contract. |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t chain
// ObjectId actor

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.set_ik_target(chain, actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-set-speed-1"></a>

## `epok::Skeleton2DComponent::set_speed`

**Purpose.** Sets speed as part of the world2d skeleton module.

**Exact declaration**

```cpp
void set_speed(Fixed value)
```

- **Declared at:** [line 100](../../../runtime/world2d_skeleton.hpp#L100)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.set_speed(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-stop-1"></a>

## `epok::Skeleton2DComponent::stop`

**Purpose.** Stops stop as part of the world2d skeleton module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 96](../../../runtime/world2d_skeleton.hpp#L96)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeleton2dcomponent-take-finished-1"></a>

## `epok::Skeleton2DComponent::take_finished`

**Purpose.** True once after a non-looping animation reaches its end.

**Exact declaration**

```cpp
bool take_finished()
```

- **Declared at:** [line 110](../../../runtime/world2d_skeleton.hpp#L110)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

epok::Skeleton2DComponent& object = /* obtain a valid instance */;

auto result = object.take_finished();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-skeleton2dcomponent-skeleton2dcomponent-1"></a>

## `epok::Skeleton2DComponent::~Skeleton2DComponent`

**Purpose.** Releases the resources owned by `epok::Skeleton2DComponent`.

**Exact declaration**

```cpp
~Skeleton2DComponent() override
```

- **Declared at:** [line 68](../../../runtime/world2d_skeleton.hpp#L68)
- **Kind:** `destructor`

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// `epok::Skeleton2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-actor-root-1"></a>

## `epok::world2d::actor_root`

**Purpose.** The root transform of an actor, or null.

**Exact declaration**

```cpp
inline SceneComponent2D* actor_root(ObjectId actor)
```

- **Declared at:** [line 34](../../../runtime/world2d_skeleton.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `SceneComponent2D *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::world2d::actor_root(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-root-world-1"></a>

## `epok::world2d::root_world`

**Purpose.** Performs `root world` as part of the world2d skeleton module.

**Exact declaration**

```cpp
inline bool root_world(const SceneComponent2D& root, Affine2D& out)
```

- **Declared at:** [line 39](../../../runtime/world2d_skeleton.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `const SceneComponent2D &` | Input | Value supplied for `root`. See the exact type and module contract. |
| `out` | `Affine2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// const SceneComponent2D & root
// Affine2D & out

auto result = epok::world2d::root_world(root, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-wrap-degrees-raw-1"></a>

## `epok::world2d::wrap_degrees_raw`

**Purpose.** An angle in raw Q12 degrees brought into (-180, 180].

**Exact declaration**

```cpp
inline int32_t wrap_degrees_raw(int32_t raw)
```

- **Declared at:** [line 27](../../../runtime/world2d_skeleton.hpp#L27)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw` | `int32_t` | Input | Value supplied for `raw`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d skeleton module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_skeleton.hpp"

// Assume these named values have been initialized with valid data:
// int32_t raw

auto result = epok::world2d::wrap_degrees_raw(raw);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d skeleton module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
