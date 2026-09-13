# Epok API: Effect Types

> **Header:** `"effect_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/effect_types.hpp)

This module covers runtime effects and their deterministic playback state. It documents 3 public callables declared directly in this header.

## Declared types

`epok::EffectLayer`, `epok::EffectLayerHandle`

## Callable index

- [`epok::EffectLayer::burst`](#epok-effectlayer-burst-1) — Performs `burst` as part of runtime effects and their deterministic playback state.
- [`epok::EffectLayer::play`](#epok-effectlayer-play-1) — Starts play as part of runtime effects and their deterministic playback state.
- [`epok::EffectLayer::stop`](#epok-effectlayer-stop-1) — Stops stop as part of runtime effects and their deterministic playback state.

<a id="epok-effectlayer-burst-1"></a>

## `epok::EffectLayer::burst`

**Purpose.** Performs `burst` as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void burst(uint32_t count)
```

- **Declared at:** [line 19](../../../runtime/effect_types.hpp#L19)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "effect_types.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t count

epok::EffectLayer& object = /* obtain a valid instance */;

object.burst(count);
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effectlayer-play-1"></a>

## `epok::EffectLayer::play`

**Purpose.** Starts play as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 17](../../../runtime/effect_types.hpp#L17)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "effect_types.hpp"

epok::EffectLayer& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-effectlayer-stop-1"></a>

## `epok::EffectLayer::stop`

**Purpose.** Stops stop as part of runtime effects and their deterministic playback state.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 18](../../../runtime/effect_types.hpp#L18)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need runtime effects and their deterministic playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "effect_types.hpp"

epok::EffectLayer& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to runtime effects and their deterministic playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
