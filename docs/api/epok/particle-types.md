# Epok API: Particle Types

> **Header:** `"particle_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/particle_types.hpp)

This module covers bounded particle simulation and rendering. It documents 3 public callables declared directly in this header.

## Declared types

`epok::ParticleEmitter`, `epok::ParticleStats`

## Callable index

- [`epok::ParticleEmitter::burst`](#epok-particleemitter-burst-1) — Requests are consumed at the next simulation tick and saturate at the global pool size.
- [`epok::ParticleEmitter::play`](#epok-particleemitter-play-1) — Starts play as part of bounded particle simulation and rendering.
- [`epok::ParticleEmitter::stop`](#epok-particleemitter-stop-1) — Stops stop as part of bounded particle simulation and rendering.

<a id="epok-particleemitter-burst-1"></a>

## `epok::ParticleEmitter::burst`

**Purpose.** Requests are consumed at the next simulation tick and saturate at the global pool size.

**Exact declaration**

```cpp
void burst(uint16_t count=0)
```

- **Declared at:** [line 11](../../../runtime/particle_types.hpp#L11)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `count` | `uint16_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t count

epok::ParticleEmitter& object = /* obtain a valid instance */;

object.burst(count);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemitter-play-1"></a>

## `epok::ParticleEmitter::play`

**Purpose.** Starts play as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 12](../../../runtime/particle_types.hpp#L12)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_types.hpp"

epok::ParticleEmitter& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particleemitter-stop-1"></a>

## `epok::ParticleEmitter::stop`

**Purpose.** Stops stop as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 12](../../../runtime/particle_types.hpp#L12)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particle_types.hpp"

epok::ParticleEmitter& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
