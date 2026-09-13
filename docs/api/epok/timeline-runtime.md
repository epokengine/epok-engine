# Epok API: Timeline Runtime

> **Header:** `"timeline_runtime.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/timeline_runtime.hpp)

This module covers fixed-step simulation time and frame timing. It documents 25 public callables declared directly in this header.

## Declared types

`epok::timeline::Asset`, `epok::timeline::BoundTarget`, `epok::timeline::Diagnostic`, `epok::timeline::Director`, `epok::timeline::Event`, `epok::timeline::Property`, `epok::timeline::Reason`, `epok::timeline::State`, `epok::timeline::Stats`, `epok::timeline::Target`, `epok::timeline::Value`

## Callable index

- [`epok::timeline::BoundTarget::active`](#epok-timeline-boundtarget-active-1) — Performs `active` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::BoundTarget`](#epok-timeline-boundtarget-boundtarget-1) — Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::BoundTarget`](#epok-timeline-boundtarget-boundtarget-2) — Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::BoundTarget`](#epok-timeline-boundtarget-boundtarget-3) — Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::effect_layer`](#epok-timeline-boundtarget-effect-layer-1) — Performs `effect layer` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::get`](#epok-timeline-boundtarget-get-1) — Returns get as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::operator EntityHandle`](#epok-timeline-boundtarget-operator-entityhandle-1) — Performs `operator  entity handle` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::same`](#epok-timeline-boundtarget-same-1) — Performs `same` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::valid`](#epok-timeline-boundtarget-valid-1) — Performs `valid` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::BoundTarget::visible`](#epok-timeline-boundtarget-visible-1) — Performs `visible` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::advance`](#epok-timeline-director-advance-1) — Performs `advance` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::cancel_all`](#epok-timeline-director-cancel-all-1) — Performs `cancel all` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::cancel_owner`](#epok-timeline-director-cancel-owner-1) — Performs `cancel owner` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::capacity_drop`](#epok-timeline-director-capacity-drop-1) — Performs `capacity drop` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::marker`](#epok-timeline-director-marker-1) — Performs `marker` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::marker_revision`](#epok-timeline-director-marker-revision-1) — Performs `marker revision` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::pause`](#epok-timeline-director-pause-1) — Pauses pause as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::play`](#epok-timeline-director-play-1) — Starts play as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::play`](#epok-timeline-director-play-2) — Starts play as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::poll_diagnostic`](#epok-timeline-director-poll-diagnostic-1) — Polls diagnostic as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::seek`](#epok-timeline-director-seek-1) — Performs `seek` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::snapshot`](#epok-timeline-director-snapshot-1) — Performs `snapshot` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::state`](#epok-timeline-director-state-1) — Performs `state` as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::stop`](#epok-timeline-director-stop-1) — Stops stop as part of fixed-step simulation time and frame timing.
- [`epok::timeline::Director::tick`](#epok-timeline-director-tick-1) — Performs `tick` as part of fixed-step simulation time and frame timing.

<a id="epok-timeline-boundtarget-active-1"></a>

## `epok::timeline::BoundTarget::active`

**Purpose.** Performs `active` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool active()const
```

- **Declared at:** [line 22](../../../runtime/timeline_runtime.hpp#L22)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-boundtarget-1"></a>

## `epok::timeline::BoundTarget::BoundTarget`

**Purpose.** Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
BoundTarget():ent
```

- **Declared at:** [line 15](../../../runtime/timeline_runtime.hpp#L15)
- **Kind:** `constructor`

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget value();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-boundtarget-2"></a>

## `epok::timeline::BoundTarget::BoundTarget`

**Purpose.** Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
BoundTarget(EffectLayerHandle value):lay
```

- **Declared at:** [line 17](../../../runtime/timeline_runtime.hpp#L17)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `EffectLayerHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EffectLayerHandle value

epok::timeline::BoundTarget value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-boundtarget-3"></a>

## `epok::timeline::BoundTarget::BoundTarget`

**Purpose.** Constructs `epok::timeline::BoundTarget` for fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
BoundTarget(EntityHandle value):ent
```

- **Declared at:** [line 16](../../../runtime/timeline_runtime.hpp#L16)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `EntityHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle value

epok::timeline::BoundTarget value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-effect-layer-1"></a>

## `epok::timeline::BoundTarget::effect_layer`

**Purpose.** Performs `effect layer` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
EffectLayer* effect_layer()const
```

- **Declared at:** [line 18](../../../runtime/timeline_runtime.hpp#L18)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `EffectLayer *`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.effect_layer();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-get-1"></a>

## `epok::timeline::BoundTarget::get`

**Purpose.** Returns get as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
Entity* get()const
```

- **Declared at:** [line 19](../../../runtime/timeline_runtime.hpp#L19)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.get();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-operator-entityhandle-1"></a>

## `epok::timeline::BoundTarget::operator EntityHandle`

**Purpose.** Performs `operator  entity handle` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
operator EntityHandle()const
```

- **Declared at:** [line 20](../../../runtime/timeline_runtime.hpp#L20)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.operator EntityHandle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-same-1"></a>

## `epok::timeline::BoundTarget::same`

**Purpose.** Performs `same` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool same(const BoundTarget& other)const
```

- **Declared at:** [line 24](../../../runtime/timeline_runtime.hpp#L24)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const BoundTarget &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const BoundTarget & other

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.same(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-boundtarget-valid-1"></a>

## `epok::timeline::BoundTarget::valid`

**Purpose.** Performs `valid` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool valid()const
```

- **Declared at:** [line 21](../../../runtime/timeline_runtime.hpp#L21)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-boundtarget-visible-1"></a>

## `epok::timeline::BoundTarget::visible`

**Purpose.** Performs `visible` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool visible()const
```

- **Declared at:** [line 23](../../../runtime/timeline_runtime.hpp#L23)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::BoundTarget& object = /* obtain a valid instance */;

auto result = object.visible();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-advance-1"></a>

## `epok::timeline::Director::advance`

**Purpose.** Performs `advance` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void advance(Fixed dt,uint32_t scene,bool paused=false)
```

- **Declared at:** [line 249](../../../runtime/timeline_runtime.hpp#L249)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// uint32_t scene
// bool paused

epok::timeline::Director& object = /* obtain a valid instance */;

object.advance(dt, scene, paused);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-cancel-all-1"></a>

## `epok::timeline::Director::cancel_all`

**Purpose.** Performs `cancel all` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void cancel_all()
```

- **Declared at:** [line 236](../../../runtime/timeline_runtime.hpp#L236)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

epok::timeline::Director& object = /* obtain a valid instance */;

object.cancel_all();
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-cancel-owner-1"></a>

## `epok::timeline::Director::cancel_owner`

**Purpose.** Performs `cancel owner` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void cancel_owner(BoundTarget owner)
```

- **Declared at:** [line 237](../../../runtime/timeline_runtime.hpp#L237)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `BoundTarget` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// BoundTarget owner

epok::timeline::Director& object = /* obtain a valid instance */;

object.cancel_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-capacity-drop-1"></a>

## `epok::timeline::Director::capacity_drop`

**Purpose.** Performs `capacity drop` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
void capacity_drop(const Asset& asset)
```

- **Declared at:** [line 169](../../../runtime/timeline_runtime.hpp#L169)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Asset & asset

epok::timeline::Director& object = /* obtain a valid instance */;

object.capacity_drop(asset);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-director-marker-1"></a>

## `epok::timeline::Director::marker`

**Purpose.** Performs `marker` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
uint32_t marker(Handle h,uint16_t index)const
```

- **Declared at:** [line 189](../../../runtime/timeline_runtime.hpp#L189)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `index` | `uint16_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// uint16_t index

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.marker(h, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-marker-revision-1"></a>

## `epok::timeline::Director::marker_revision`

**Purpose.** Performs `marker revision` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
uint32_t marker_revision(Handle h,uint64_t id)const
```

- **Declared at:** [line 192](../../../runtime/timeline_runtime.hpp#L192)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `id` | `uint64_t` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// uint64_t id

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.marker_revision(h, id);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-pause-1"></a>

## `epok::timeline::Director::pause`

**Purpose.** Pauses pause as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool pause(Handle h,bool paused)
```

- **Declared at:** [line 235](../../../runtime/timeline_runtime.hpp#L235)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// bool paused

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.pause(h, paused);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-play-1"></a>

## `epok::timeline::Director::play`

**Purpose.** Starts play as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
Handle play(const Asset& asset,BoundTarget owner,const BoundTarget* targets,uint32_t scene)
```

- **Declared at:** [line 203](../../../runtime/timeline_runtime.hpp#L203)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `owner` | `BoundTarget` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `targets` | `const BoundTarget *` | Input | Value supplied for `targets`. See the exact type and module contract. |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Asset & asset
// BoundTarget owner
// const BoundTarget * targets
// uint32_t scene

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.play(asset, owner, targets, scene);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-director-play-2"></a>

## `epok::timeline::Director::play`

**Purpose.** Starts play as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
Handle play(const Asset& asset,EntityHandle owner,const EntityHandle* targets,uint32_t scene)
```

- **Declared at:** [line 197](../../../runtime/timeline_runtime.hpp#L197)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `targets` | `const EntityHandle *` | Input | Value supplied for `targets`. See the exact type and module contract. |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Asset & asset
// EntityHandle owner
// const EntityHandle * targets
// uint32_t scene

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.play(asset, owner, targets, scene);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-director-poll-diagnostic-1"></a>

## `epok::timeline::Director::poll_diagnostic`

**Purpose.** Polls diagnostic as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool poll_diagnostic(Diagnostic& result)
```

- **Declared at:** [line 170](../../../runtime/timeline_runtime.hpp#L170)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `Diagnostic &` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Diagnostic & result

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.poll_diagnostic(result);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-director-seek-1"></a>

## `epok::timeline::Director::seek`

**Purpose.** Performs `seek` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool seek(Handle h,int32_t tick,uint32_t scene)
```

- **Declared at:** [line 238](../../../runtime/timeline_runtime.hpp#L238)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `tick` | `int32_t` | Input | Value supplied for `tick`. See the exact type and module contract. |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// int32_t tick
// uint32_t scene

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.seek(h, tick, scene);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-snapshot-1"></a>

## `epok::timeline::Director::snapshot`

**Purpose.** Performs `snapshot` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bp::PlaybackSnapshot snapshot(Handle h,uint64_t asset=0,uint64_t marker=0)const
```

- **Declared at:** [line 175](../../../runtime/timeline_runtime.hpp#L175)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `marker` | `uint64_t` | Input | Value supplied for `marker`. See the exact type and module contract. |

**Returns.** Returns `bp::PlaybackSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// uint64_t asset
// uint64_t marker

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.snapshot(h, asset, marker);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-state-1"></a>

## `epok::timeline::Director::state`

**Purpose.** Performs `state` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
State state(Handle h)const
```

- **Declared at:** [line 174](../../../runtime/timeline_runtime.hpp#L174)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `State`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.state(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-stop-1"></a>

## `epok::timeline::Director::stop`

**Purpose.** Stops stop as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
bool stop(Handle h)
```

- **Declared at:** [line 234](../../../runtime/timeline_runtime.hpp#L234)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.stop(h);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-timeline-director-tick-1"></a>

## `epok::timeline::Director::tick`

**Purpose.** Performs `tick` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
int32_t tick(Handle h)const
```

- **Declared at:** [line 188](../../../runtime/timeline_runtime.hpp#L188)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::timeline::Director& object = /* obtain a valid instance */;

auto result = object.tick(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
