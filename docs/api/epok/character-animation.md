# Epok API: Character Animation

> **Header:** `"character_animation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/character_animation.hpp)

This module covers configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. It documents 35 public callables declared directly in this header.

## Declared types

`epok::locomotion::Runtime`, `epok::locomotion::UpdateScope`, `epok::LocomotionClips`, `epok::LocomotionSample`, `epok::LocomotionSelection`, `epok::LocomotionState`, `epok::LocomotionStatus`, `epok::LocomotionTuning`, `epok::SkeletalLocomotionComponent`, `epok::SpriteLocomotion2DComponent`

## Callable index

- [`epok::locomotion::alive`](#epok-locomotion-alive-1) — Performs `alive` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::choose`](#epok-locomotion-choose-1) — Performs `choose` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::component`](#epok-locomotion-component-1) — Performs `component` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::moving`](#epok-locomotion-moving-1) — Performs `moving` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::Runtime::applied`](#epok-locomotion-runtime-applied-1) — Performs `applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::Runtime::fresh`](#epok-locomotion-runtime-fresh-1) — Performs `fresh` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::Runtime::report`](#epok-locomotion-runtime-report-1) — Performs `report` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::Runtime::reset`](#epok-locomotion-runtime-reset-1) — Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::selection`](#epok-locomotion-selection-1) — Performs `selection` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::UpdateScope::UpdateScope`](#epok-locomotion-updatescope-updatescope-1) — Constructs `epok::locomotion::UpdateScope` for configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::UpdateScope::~UpdateScope`](#epok-locomotion-updatescope-updatescope-2) — Releases the resources owned by `epok::locomotion::UpdateScope`.
- [`epok::locomotion::valid`](#epok-locomotion-valid-1) — Performs `valid` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::locomotion::valid`](#epok-locomotion-valid-2) — Performs `valid` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::animation_applied`](#epok-skeletallocomotioncomponent-animation-applied-1) — Performs `animation applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::begin_play`](#epok-skeletallocomotioncomponent-begin-play-1) — Begins play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::class_id`](#epok-skeletallocomotioncomponent-class-id-1) — Performs `class id` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::end_play`](#epok-skeletallocomotioncomponent-end-play-1) — Ends play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::on_disable`](#epok-skeletallocomotioncomponent-on-disable-1) — Performs `on disable` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::reset`](#epok-skeletallocomotioncomponent-reset-1) — Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::sample`](#epok-skeletallocomotioncomponent-sample-1) — Performs `sample` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::select_animation`](#epok-skeletallocomotioncomponent-select-animation-1) — Performs `select animation` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::set_enabled`](#epok-skeletallocomotioncomponent-set-enabled-1) — Sets enabled as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::tick`](#epok-skeletallocomotioncomponent-tick-1) — Performs `tick` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SkeletalLocomotionComponent::update`](#epok-skeletallocomotioncomponent-update-1) — Updates update as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::animation_applied`](#epok-spritelocomotion2dcomponent-animation-applied-1) — Performs `animation applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::begin_play`](#epok-spritelocomotion2dcomponent-begin-play-1) — Begins play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::class_id`](#epok-spritelocomotion2dcomponent-class-id-1) — Performs `class id` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::end_play`](#epok-spritelocomotion2dcomponent-end-play-1) — Ends play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::on_disable`](#epok-spritelocomotion2dcomponent-on-disable-1) — Performs `on disable` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::reset`](#epok-spritelocomotion2dcomponent-reset-1) — Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::sample`](#epok-spritelocomotion2dcomponent-sample-1) — Performs `sample` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::select_animation`](#epok-spritelocomotion2dcomponent-select-animation-1) — Performs `select animation` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::set_enabled`](#epok-spritelocomotion2dcomponent-set-enabled-1) — Sets enabled as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::tick`](#epok-spritelocomotion2dcomponent-tick-1) — Performs `tick` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.
- [`epok::SpriteLocomotion2DComponent::update`](#epok-spritelocomotion2dcomponent-update-1) — Updates update as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

<a id="epok-locomotion-alive-1"></a>

## `epok::locomotion::alive`

**Purpose.** Performs `alive` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline bool alive(ActorComponent& source,ObjectRegistry* registry,ObjectId identity)
```

- **Declared at:** [line 76](../../../runtime/character_animation.hpp#L76)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |
| `registry` | `ObjectRegistry *` | Input/output; inspect the function contract | Value supplied for `registry`. See the exact type and module contract. |
| `identity` | `ObjectId` | Input | Value supplied for `identity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & source
// ObjectRegistry * registry
// ObjectId identity

auto result = epok::locomotion::alive(source, registry, identity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-choose-1"></a>

## `epok::locomotion::choose`

**Purpose.** Performs `choose` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline LocomotionState choose(LocomotionState state,bool grounded,Fixed vertical,Fixed speed,bool finished,bool jumped,const LocomotionTuning& tuning)
```

- **Declared at:** [line 38](../../../runtime/character_animation.hpp#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `LocomotionState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `grounded` | `bool` | Input | Value supplied for `grounded`. See the exact type and module contract. |
| `vertical` | `Fixed` | Input | Value supplied for `vertical`. See the exact type and module contract. |
| `speed` | `Fixed` | Input | Value supplied for `speed`. See the exact type and module contract. |
| `finished` | `bool` | Input | Value supplied for `finished`. See the exact type and module contract. |
| `jumped` | `bool` | Input | Value supplied for `jumped`. See the exact type and module contract. |
| `tuning` | `const LocomotionTuning &` | Input | Value supplied for `tuning`. See the exact type and module contract. |

**Returns.** Returns `LocomotionState`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionState state
// bool grounded
// Fixed vertical
// Fixed speed
// bool finished
// bool jumped
// const LocomotionTuning & tuning

auto result = epok::locomotion::choose(state, grounded, vertical, speed, finished, jumped, tuning);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-component-1"></a>

## `epok::locomotion::component`

**Purpose.** Performs `component` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
template<class T> T* component(ActorComponent& source,ObjectId reference,Actor& fallback)
```

- **Declared at:** [line 70](../../../runtime/character_animation.hpp#L70)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |
| `reference` | `ObjectId` | Input | Value supplied for `reference`. See the exact type and module contract. |
| `fallback` | `Actor &` | Input/output; inspect the function contract | Value supplied for `fallback`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// ActorComponent & source
// ObjectId reference
// Actor & fallback

epok::locomotion& object = /* obtain a valid instance */;

auto result = object.component<T>(source, reference, fallback);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-moving-1"></a>

## `epok::locomotion::moving`

**Purpose.** Performs `moving` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline bool moving(LocomotionState state)
```

- **Declared at:** [line 31](../../../runtime/character_animation.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `LocomotionState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionState state

auto result = epok::locomotion::moving(state);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-locomotion-runtime-applied-1"></a>

## `epok::locomotion::Runtime::applied`

**Purpose.** Performs `applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample applied(LocomotionSelection value,bool changed,ObjectId new_motor,ObjectId new_visual,const void* new_bank)
```

- **Declared at:** [line 65](../../../runtime/character_animation.hpp#L65)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LocomotionSelection` | Input | Value supplied for `value`. See the exact type and module contract. |
| `changed` | `bool` | Input | Value supplied for `changed`. See the exact type and module contract. |
| `new_motor` | `ObjectId` | Input | Value supplied for `new_motor`. See the exact type and module contract. |
| `new_visual` | `ObjectId` | Input | Value supplied for `new_visual`. See the exact type and module contract. |
| `new_bank` | `const void *` | Input | Value supplied for `new_bank`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionSelection value
// bool changed
// ObjectId new_motor
// ObjectId new_visual
// const void * new_bank

epok::locomotion::Runtime& object = /* obtain a valid instance */;

auto result = object.applied(value, changed, new_motor, new_visual, new_bank);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-runtime-fresh-1"></a>

## `epok::locomotion::Runtime::fresh`

**Purpose.** Performs `fresh` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
bool fresh(ObjectId new_motor,ObjectId new_visual,const void* new_bank)const
```

- **Declared at:** [line 64](../../../runtime/character_animation.hpp#L64)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `new_motor` | `ObjectId` | Input | Value supplied for `new_motor`. See the exact type and module contract. |
| `new_visual` | `ObjectId` | Input | Value supplied for `new_visual`. See the exact type and module contract. |
| `new_bank` | `const void *` | Input | Value supplied for `new_bank`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId new_motor
// ObjectId new_visual
// const void * new_bank

epok::locomotion::Runtime& object = /* obtain a valid instance */;

auto result = object.fresh(new_motor, new_visual, new_bank);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-runtime-report-1"></a>

## `epok::locomotion::Runtime::report`

**Purpose.** Performs `report` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample report(LocomotionStatus status)
```

- **Declared at:** [line 63](../../../runtime/character_animation.hpp#L63)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `status` | `LocomotionStatus` | Input | Value supplied for `status`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionStatus status

epok::locomotion::Runtime& object = /* obtain a valid instance */;

auto result = object.report(status);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-locomotion-runtime-reset-1"></a>

## `epok::locomotion::Runtime::reset`

**Purpose.** Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 62](../../../runtime/character_animation.hpp#L62)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::locomotion::Runtime& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-locomotion-selection-1"></a>

## `epok::locomotion::selection`

**Purpose.** Performs `selection` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline LocomotionSelection selection(LocomotionState state,Fixed speed,const LocomotionClips& clips,const LocomotionTuning& tuning,bool distance_driven)
```

- **Declared at:** [line 47](../../../runtime/character_animation.hpp#L47)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `LocomotionState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `speed` | `Fixed` | Input | Value supplied for `speed`. See the exact type and module contract. |
| `clips` | `const LocomotionClips &` | Input | Value supplied for `clips`. See the exact type and module contract. |
| `tuning` | `const LocomotionTuning &` | Input | Value supplied for `tuning`. See the exact type and module contract. |
| `distance_driven` | `bool` | Input | Value supplied for `distance_driven`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSelection`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionState state
// Fixed speed
// const LocomotionClips & clips
// const LocomotionTuning & tuning
// bool distance_driven

auto result = epok::locomotion::selection(state, speed, clips, tuning, distance_driven);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-updatescope-updatescope-1"></a>

## `epok::locomotion::UpdateScope::UpdateScope`

**Purpose.** Constructs `epok::locomotion::UpdateScope` for configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
explicit UpdateScope(bool& flag):value(flag)
```

- **Declared at:** [line 69](../../../runtime/character_animation.hpp#L69)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `flag` | `bool &` | Input/output; inspect the function contract | Value supplied for `flag`. See the exact type and module contract. |

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// bool & flag

epok::locomotion::UpdateScope value(flag);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-locomotion-updatescope-updatescope-2"></a>

## `epok::locomotion::UpdateScope::~UpdateScope`

**Purpose.** Releases the resources owned by `epok::locomotion::UpdateScope`.

**Exact declaration**

```cpp
~UpdateScope()
```

- **Declared at:** [line 69](../../../runtime/character_animation.hpp#L69)
- **Kind:** `destructor`

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// `epok::locomotion::UpdateScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-locomotion-valid-1"></a>

## `epok::locomotion::valid`

**Purpose.** Performs `valid` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline bool valid(LocomotionSelection value)
```

- **Declared at:** [line 59](../../../runtime/character_animation.hpp#L59)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LocomotionSelection` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionSelection value

auto result = epok::locomotion::valid(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-locomotion-valid-2"></a>

## `epok::locomotion::valid`

**Purpose.** Performs `valid` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
inline bool valid(const LocomotionTuning& tuning)
```

- **Declared at:** [line 32](../../../runtime/character_animation.hpp#L32)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `tuning` | `const LocomotionTuning &` | Input | Value supplied for `tuning`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// const LocomotionTuning & tuning

auto result = epok::locomotion::valid(tuning);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletallocomotioncomponent-animation-applied-1"></a>

## `epok::SkeletalLocomotionComponent::animation_applied`

**Purpose.** Performs `animation applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
virtual void animation_applied(LocomotionSample value)
```

- **Declared at:** [line 102](../../../runtime/character_animation.hpp#L102)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LocomotionSample` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionSample value

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.animation_applied(value);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-begin-play-1"></a>

## `epok::SkeletalLocomotionComponent::begin_play`

**Purpose.** Begins play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 144](../../../runtime/character_animation.hpp#L144)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-class-id-1"></a>

## `epok::SkeletalLocomotionComponent::class_id`

**Purpose.** Performs `class id` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 91](../../../runtime/character_animation.hpp#L91)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-end-play-1"></a>

## `epok::SkeletalLocomotionComponent::end_play`

**Purpose.** Ends play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 146](../../../runtime/character_animation.hpp#L146)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-on-disable-1"></a>

## `epok::SkeletalLocomotionComponent::on_disable`

**Purpose.** Performs `on disable` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 145](../../../runtime/character_animation.hpp#L145)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-reset-1"></a>

## `epok::SkeletalLocomotionComponent::reset`

**Purpose.** Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 104](../../../runtime/character_animation.hpp#L104)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-sample-1"></a>

## `epok::SkeletalLocomotionComponent::sample`

**Purpose.** Performs `sample` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample sample()const
```

- **Declared at:** [line 103](../../../runtime/character_animation.hpp#L103)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-select-animation-1"></a>

## `epok::SkeletalLocomotionComponent::select_animation`

**Purpose.** Performs `select animation` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
virtual LocomotionSelection select_animation(CharacterMotionSample3D motion,LocomotionState previous,bool finished)
```

- **Declared at:** [line 99](../../../runtime/character_animation.hpp#L99)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `motion` | `CharacterMotionSample3D` | Input | Value supplied for `motion`. See the exact type and module contract. |
| `previous` | `LocomotionState` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `finished` | `bool` | Input | Value supplied for `finished`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSelection`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// CharacterMotionSample3D motion
// LocomotionState previous
// bool finished

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

auto result = object.select_animation(motion, previous, finished);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-set-enabled-1"></a>

## `epok::SkeletalLocomotionComponent::set_enabled`

**Purpose.** Sets enabled as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 105](../../../runtime/character_animation.hpp#L105)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-tick-1"></a>

## `epok::SkeletalLocomotionComponent::tick`

**Purpose.** Performs `tick` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 143](../../../runtime/character_animation.hpp#L143)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletallocomotioncomponent-update-1"></a>

## `epok::SkeletalLocomotionComponent::update`

**Purpose.** Updates update as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample update(Fixed dt)
```

- **Declared at:** [line 106](../../../runtime/character_animation.hpp#L106)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::SkeletalLocomotionComponent& object = /* obtain a valid instance */;

auto result = object.update(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-animation-applied-1"></a>

## `epok::SpriteLocomotion2DComponent::animation_applied`

**Purpose.** Performs `animation applied` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
virtual void animation_applied(LocomotionSample value)
```

- **Declared at:** [line 178](../../../runtime/character_animation.hpp#L178)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `LocomotionSample` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// LocomotionSample value

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.animation_applied(value);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-begin-play-1"></a>

## `epok::SpriteLocomotion2DComponent::begin_play`

**Purpose.** Begins play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void begin_play()override
```

- **Declared at:** [line 213](../../../runtime/character_animation.hpp#L213)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-class-id-1"></a>

## `epok::SpriteLocomotion2DComponent::class_id`

**Purpose.** Performs `class id` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 159](../../../runtime/character_animation.hpp#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-end-play-1"></a>

## `epok::SpriteLocomotion2DComponent::end_play`

**Purpose.** Ends play as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void end_play(EndPlayReason)override
```

- **Declared at:** [line 215](../../../runtime/character_animation.hpp#L215)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-on-disable-1"></a>

## `epok::SpriteLocomotion2DComponent::on_disable`

**Purpose.** Performs `on disable` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void on_disable()override
```

- **Declared at:** [line 214](../../../runtime/character_animation.hpp#L214)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-reset-1"></a>

## `epok::SpriteLocomotion2DComponent::reset`

**Purpose.** Resets reset as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 180](../../../runtime/character_animation.hpp#L180)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-sample-1"></a>

## `epok::SpriteLocomotion2DComponent::sample`

**Purpose.** Performs `sample` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample sample()const
```

- **Declared at:** [line 179](../../../runtime/character_animation.hpp#L179)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-select-animation-1"></a>

## `epok::SpriteLocomotion2DComponent::select_animation`

**Purpose.** Performs `select animation` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
virtual LocomotionSelection select_animation(CharacterMotionSample2D motion,LocomotionState previous,bool finished)
```

- **Declared at:** [line 169](../../../runtime/character_animation.hpp#L169)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `motion` | `CharacterMotionSample2D` | Input | Value supplied for `motion`. See the exact type and module contract. |
| `previous` | `LocomotionState` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `finished` | `bool` | Input | Value supplied for `finished`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSelection`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// CharacterMotionSample2D motion
// LocomotionState previous
// bool finished

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

auto result = object.select_animation(motion, previous, finished);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-set-enabled-1"></a>

## `epok::SpriteLocomotion2DComponent::set_enabled`

**Purpose.** Sets enabled as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 181](../../../runtime/character_animation.hpp#L181)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-tick-1"></a>

## `epok::SpriteLocomotion2DComponent::tick`

**Purpose.** Performs `tick` as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
void tick(Fixed dt)override
```

- **Declared at:** [line 212](../../../runtime/character_animation.hpp#L212)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

object.tick(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-spritelocomotion2dcomponent-update-1"></a>

## `epok::SpriteLocomotion2DComponent::update`

**Purpose.** Updates update as part of configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples.

**Exact declaration**

```cpp
LocomotionSample update(Fixed dt)
```

- **Declared at:** [line 182](../../../runtime/character_animation.hpp#L182)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `LocomotionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "character_animation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::SpriteLocomotion2DComponent& object = /* obtain a valid instance */;

auto result = object.update(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to configurable skeletal and sprite locomotion drivers with extensible selection and shared motor samples. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
