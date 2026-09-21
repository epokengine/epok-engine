# Epok API: Particle Effect Runtime

> **Header:** `"particle_effect_runtime.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/particle_effect_runtime.hpp)

This module covers runtime effects and their deterministic playback state. It documents 22 public callables declared directly in this header.

## Declared types

`epok::effects::Asset`, `epok::effects::LayerDefinition`, `epok::effects::LayerInitializer`, `epok::effects::Pool`, `epok::effects::State`, `epok::effects::Stats`

## Callable index

- [`epok::effects::Pool::advance`](#epok-effects-pool-advance-1) — Call after the director, before the single particle simulation step.
- [`epok::effects::Pool::burst`](#epok-effects-pool-burst-1) — Performs `burst` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::cancel_owner`](#epok-effects-pool-cancel-owner-1) — Performs `cancel owner` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::emitters`](#epok-effects-pool-emitters-1) — Performs `emitters` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::layer`](#epok-effects-pool-layer-1) — Performs `layer` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::lighting_owner`](#epok-effects-pool-lighting-owner-1) — Performs `lighting owner` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::move`](#epok-effects-pool-move-1) — Performs `move` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::observe`](#epok-effects-pool-observe-1) — Capture completion before gameplay can reuse a completed director slot.
- [`epok::effects::Pool::operator=`](#epok-effects-pool-operator-1) — Performs `operator =` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::pause`](#epok-effects-pool-pause-1) — Pauses pause as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::Pool`](#epok-effects-pool-pool-1) — Constructs `epok::effects::Pool` for runtime effects and their deterministic playback state.
- [`epok::effects::Pool::Pool`](#epok-effects-pool-pool-2) — Constructs `epok::effects::Pool` for runtime effects and their deterministic playback state.
- [`epok::effects::Pool::prepare`](#epok-effects-pool-prepare-1) — Call before the shared director advances.
- [`epok::effects::Pool::reset`](#epok-effects-pool-reset-1) — Resets reset as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::sequence`](#epok-effects-pool-sequence-1) — Performs `sequence` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::snapshot`](#epok-effects-pool-snapshot-1) — Performs `snapshot` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::spawn`](#epok-effects-pool-spawn-1) — Performs `spawn` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::sprites`](#epok-effects-pool-sprites-1) — Performs `sprites` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::state`](#epok-effects-pool-state-1) — Performs `state` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::stop`](#epok-effects-pool-stop-1) — Stops stop as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::tint`](#epok-effects-pool-tint-1) — Performs `tint` as part of runtime effects and their deterministic playback state.
- [`epok::effects::Pool::~Pool`](#epok-effects-pool-pool-3) — Releases the resources owned by `epok::effects::Pool`.

<a id="epok-effects-pool-advance-1"></a>

## `epok::effects::Pool::advance`

**Purpose.** Call after the director, before the single particle simulation step.

**Exact declaration**

```cpp
void advance(Fixed dt,bool paused=false)
```

- **Declared at:** [line 145](../../../runtime/particle_effect_runtime.hpp#L145)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// bool paused

epok::effects::Pool& object = /* obtain a valid instance */;

object.advance(dt, paused);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-burst-1"></a>

## `epok::effects::Pool::burst`

**Purpose.** Performs `burst` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
bool burst(Handle h,uint32_t count)
```

- **Declared at:** [line 117](../../../runtime/particle_effect_runtime.hpp#L117)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// uint32_t count

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.burst(h, count);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-cancel-owner-1"></a>

## `epok::effects::Pool::cancel_owner`

**Purpose.** Performs `cancel owner` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void cancel_owner(DataHandle owner)
```

- **Declared at:** [line 128](../../../runtime/particle_effect_runtime.hpp#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `DataHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle owner

epok::effects::Pool& object = /* obtain a valid instance */;

object.cancel_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-emitters-1"></a>

## `epok::effects::Pool::emitters`

**Purpose.** Performs `emitters` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
template<class Visit>void emitters(Visit visit)
```

- **Declared at:** [line 172](../../../runtime/particle_effect_runtime.hpp#L172)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `visit` | `Visit` | Input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// Visit visit

epok::effects::Pool& object = /* obtain a valid instance */;

object.emitters<Visit>(visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-layer-1"></a>

## `epok::effects::Pool::layer`

**Purpose.** Performs `layer` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
EffectLayerHandle layer(Handle h,uint16_t index)const
```

- **Declared at:** [line 80](../../../runtime/particle_effect_runtime.hpp#L80)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `index` | `uint16_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `EffectLayerHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// uint16_t index

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.layer(h, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-lighting-owner-1"></a>

## `epok::effects::Pool::lighting_owner`

**Purpose.** Performs `lighting owner` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
DataHandle lighting_owner(EffectLayerHandle h)const
```

- **Declared at:** [line 76](../../../runtime/particle_effect_runtime.hpp#L76)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `EffectLayerHandle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EffectLayerHandle h

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.lighting_owner(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-move-1"></a>

## `epok::effects::Pool::move`

**Purpose.** Performs `move` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
bool move(Handle h,const Affine<Fixed>& world)
```

- **Declared at:** [line 127](../../../runtime/particle_effect_runtime.hpp#L127)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// const Affine<Fixed> & world

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.move(h, world);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-observe-1"></a>

## `epok::effects::Pool::observe`

**Purpose.** Capture completion before gameplay can reuse a completed director slot.

**Details.** A draining effect no longer depends on that sequence handle's lifetime.

**Exact declaration**

```cpp
void observe()
```

- **Declared at:** [line 139](../../../runtime/particle_effect_runtime.hpp#L139)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A draining effect no longer depends on that sequence handle's lifetime.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

epok::effects::Pool& object = /* obtain a valid instance */;

object.observe();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-operator-1"></a>

## `epok::effects::Pool::operator=`

**Purpose.** Performs `operator =` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
Pool& operator=(const Pool&)=delete
```

- **Declared at:** [line 68](../../../runtime/particle_effect_runtime.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Pool &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Pool &`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Pool & arg1

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-pause-1"></a>

## `epok::effects::Pool::pause`

**Purpose.** Pauses pause as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
bool pause(Handle h,bool paused)
```

- **Declared at:** [line 126](../../../runtime/particle_effect_runtime.hpp#L126)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// bool paused

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.pause(h, paused);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-pool-1"></a>

## `epok::effects::Pool::Pool`

**Purpose.** Constructs `epok::effects::Pool` for runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
Pool(const Pool&)=delete
```

- **Declared at:** [line 68](../../../runtime/particle_effect_runtime.hpp#L68)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Pool &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Pool & arg1

epok::effects::Pool value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-pool-2"></a>

## `epok::effects::Pool::Pool`

**Purpose.** Constructs `epok::effects::Pool` for runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
explicit Pool(timeline::Director<8>& shared,void(*release)(EffectLayerHandle)=nullptr):dir
```

- **Declared at:** [line 67](../../../runtime/particle_effect_runtime.hpp#L67)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `shared` | `timeline::Director<8> &` | Input/output; inspect the function contract | Value supplied for `shared`. See the exact type and module contract. |
| `release` | `void (*)(EffectLayerHandle)` | Input/output; inspect the function contract | Value supplied for `release`. See the exact type and module contract. |

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Director<8> & shared
// void (*)(EffectLayerHandle) release

epok::effects::Pool value(shared, release);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-prepare-1"></a>

## `epok::effects::Pool::prepare`

**Purpose.** Call before the shared director advances.

**Details.** This is lifecycle preparation, not another continuation scheduler or another timeline evaluator.

**Exact declaration**

```cpp
void prepare(uint32_t scene)
```

- **Declared at:** [line 132](../../../runtime/particle_effect_runtime.hpp#L132)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This is lifecycle preparation, not another continuation scheduler or another timeline evaluator.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t scene

epok::effects::Pool& object = /* obtain a valid instance */;

object.prepare(scene);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-reset-1"></a>

## `epok::effects::Pool::reset`

**Purpose.** Resets reset as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 129](../../../runtime/particle_effect_runtime.hpp#L129)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

epok::effects::Pool& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-sequence-1"></a>

## `epok::effects::Pool::sequence`

**Purpose.** Performs `sequence` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
timeline::Handle sequence(Handle h)const
```

- **Declared at:** [line 75](../../../runtime/particle_effect_runtime.hpp#L75)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `timeline::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.sequence(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-snapshot-1"></a>

## `epok::effects::Pool::snapshot`

**Purpose.** Performs `snapshot` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
bp::PlaybackSnapshot snapshot(Handle h)const
```

- **Declared at:** [line 71](../../../runtime/particle_effect_runtime.hpp#L71)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `bp::PlaybackSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.snapshot(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-spawn-1"></a>

## `epok::effects::Pool::spawn`

**Purpose.** Performs `spawn` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
Handle spawn(const Asset& asset,const Affine<Fixed>& world,uint32_t scene, const timeline::BoundTarget* external=nullptr,DataHandle owner={},uint32_t seed=0,LayerInitializer initialize=nullptr)
```

- **Declared at:** [line 83](../../../runtime/particle_effect_runtime.hpp#L83)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `const Asset &` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `scene` | `uint32_t` | Input | Value supplied for `scene`. See the exact type and module contract. |
| `external` | `const timeline::BoundTarget *` | Input | Value supplied for `external`. See the exact type and module contract. |
| `owner` | `DataHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `seed` | `uint32_t` | Input | Value supplied for `seed`. See the exact type and module contract. |
| `initialize` | `LayerInitializer` | Input | Value supplied for `initialize`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const Asset & asset
// const Affine<Fixed> & world
// uint32_t scene
// const timeline::BoundTarget * external
// DataHandle owner
// uint32_t seed
// LayerInitializer initialize

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.spawn(asset, world, scene, external, owner, seed, initialize);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-sprites-1"></a>

## `epok::effects::Pool::sprites`

**Purpose.** Performs `sprites` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
template<class Emit>void sprites(Emit emit)const
```

- **Declared at:** [line 186](../../../runtime/particle_effect_runtime.hpp#L186)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `emit` | `Emit` | Input | Value supplied for `emit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Emit

// Assume these named values have been initialized with valid data:
// Emit emit

epok::effects::Pool& object = /* obtain a valid instance */;

object.sprites<Emit>(emit);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-state-1"></a>

## `epok::effects::Pool::state`

**Purpose.** Performs `state` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
State state(Handle h)const
```

- **Declared at:** [line 70](../../../runtime/particle_effect_runtime.hpp#L70)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `State`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.state(h);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-stop-1"></a>

## `epok::effects::Pool::stop`

**Purpose.** Stops stop as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
bool stop(Handle h)
```

- **Declared at:** [line 116](../../../runtime/particle_effect_runtime.hpp#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::effects::Pool& object = /* obtain a valid instance */;

auto result = object.stop(h);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-effects-pool-tint-1"></a>

## `epok::effects::Pool::tint`

**Purpose.** Performs `tint` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
static void tint(const EffectLayer& layer,Sprite& sprite)
```

- **Declared at:** [line 180](../../../runtime/particle_effect_runtime.hpp#L180)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `const EffectLayer &` | Input | Value supplied for `layer`. See the exact type and module contract. |
| `sprite` | `Sprite &` | Input/output; inspect the function contract | Value supplied for `sprite`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const EffectLayer & layer
// Sprite & sprite

epok::effects::Pool::tint(layer, sprite);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-effects-pool-pool-3"></a>

## `epok::effects::Pool::~Pool`

**Purpose.** Releases the resources owned by `epok::effects::Pool`.

**Exact declaration**

```cpp
~Pool()
```

- **Declared at:** [line 69](../../../runtime/particle_effect_runtime.hpp#L69)
- **Kind:** `destructor`

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_effect_runtime.hpp"

// `epok::effects::Pool` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
