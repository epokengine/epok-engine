# Epok API: Particles

> **Header:** `"particles.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/particles.hpp)

This module covers bounded particle simulation and rendering. It documents 10 public callables declared directly in this header.

## Declared types

`epok::ParticlePool`, `epok::ParticlePool::Particle`, `epok::ParticlePool::Source`

## Callable index

- [`epok::ParticlePool::advance`](#epok-particlepool-advance-1) — Performs `advance` as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::advance`](#epok-particlepool-advance-2) — Performs `advance` as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::begin`](#epok-particlepool-begin-1) — Begins begin as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::clear`](#epok-particlepool-clear-1) — Clears clear as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::each`](#epok-particlepool-each-1) — Performs `each` as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::each_all`](#epok-particlepool-each-all-1) — Performs `each all` as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::emitter`](#epok-particlepool-emitter-1) — Performs `emitter` as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::remove_layer`](#epok-particlepool-remove-layer-1) — Removes layer as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::remove_owner`](#epok-particlepool-remove-owner-1) — Removes owner as part of bounded particle simulation and rendering.
- [`epok::ParticlePool::scene_emitters`](#epok-particlepool-scene-emitters-1) — Performs `scene emitters` as part of bounded particle simulation and rendering.

<a id="epok-particlepool-advance-1"></a>

## `epok::ParticlePool::advance`

**Purpose.** Performs `advance` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
template<size_t N>void advance(std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count,Fixed dt)
```

- **Declared at:** [line 71](../../../runtime/particles.hpp#L71)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `int &` | Input/output; inspect the function contract | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const int &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `dt` | `int` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// int & objects
// const int & world
// int count
// int dt

epok::ParticlePool& object = /* obtain a valid instance */;

object.advance<N>(objects, world, count, dt);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-particlepool-advance-2"></a>

## `epok::ParticlePool::advance`

**Purpose.** Performs `advance` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void advance()
```

- **Declared at:** [line 44](../../../runtime/particles.hpp#L44)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

epok::ParticlePool& object = /* obtain a valid instance */;

object.advance();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particlepool-begin-1"></a>

## `epok::ParticlePool::begin`

**Purpose.** Begins begin as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void begin(Fixed dt)
```

- **Declared at:** [line 34](../../../runtime/particles.hpp#L34)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `int` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Assume these named values have been initialized with valid data:
// int dt

epok::ParticlePool& object = /* obtain a valid instance */;

object.begin(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particlepool-clear-1"></a>

## `epok::ParticlePool::clear`

**Purpose.** Clears clear as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 31](../../../runtime/particles.hpp#L31)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

epok::ParticlePool& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particlepool-each-1"></a>

## `epok::ParticlePool::each`

**Purpose.** Performs `each` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
template<size_t N,class Emit>void each(const std::array<Affine<Fixed>,N>&,size_t count,Emit emit)const
```

- **Declared at:** [line 84](../../../runtime/particles.hpp#L84)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const int &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `emit` | `Emit` | Input | Value supplied for `emit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N, Emit

// Assume these named values have been initialized with valid data:
// const int & arg1
// int count
// Emit emit

epok::ParticlePool& object = /* obtain a valid instance */;

object.each<N, Emit>(arg1, count, emit);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-particlepool-each-all-1"></a>

## `epok::ParticlePool::each_all`

**Purpose.** Performs `each all` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
template<class Emit>void each_all(Emit emit)const
```

- **Declared at:** [line 72](../../../runtime/particles.hpp#L72)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `emit` | `Emit` | Input | Value supplied for `emit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Emit

// Assume these named values have been initialized with valid data:
// Emit emit

epok::ParticlePool& object = /* obtain a valid instance */;

object.each_all<Emit>(emit);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-particlepool-emitter-1"></a>

## `epok::ParticlePool::emitter`

**Purpose.** Performs `emitter` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
bool emitter(timeline::BoundTarget owner,ParticleEmitter& emitter,const Affine<Fixed>& world)
```

- **Declared at:** [line 35](../../../runtime/particles.hpp#L35)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `timeline::BoundTarget` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `emitter` | `ParticleEmitter &` | Input/output; inspect the function contract | Value supplied for `emitter`. See the exact type and module contract. |
| `world` | `const int &` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Assume these named values have been initialized with valid data:
// timeline::BoundTarget owner
// ParticleEmitter & emitter
// const int & world

epok::ParticlePool& object = /* obtain a valid instance */;

auto result = object.emitter(owner, emitter, world);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-particlepool-remove-layer-1"></a>

## `epok::ParticlePool::remove_layer`

**Purpose.** Removes layer as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void remove_layer(EffectLayerHandle owner)
```

- **Declared at:** [line 33](../../../runtime/particles.hpp#L33)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EffectLayerHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Assume these named values have been initialized with valid data:
// EffectLayerHandle owner

epok::ParticlePool& object = /* obtain a valid instance */;

object.remove_layer(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particlepool-remove-owner-1"></a>

## `epok::ParticlePool::remove_owner`

**Purpose.** Removes owner as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
void remove_owner(size_t owner)
```

- **Declared at:** [line 32](../../../runtime/particles.hpp#L32)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `int` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Assume these named values have been initialized with valid data:
// int owner

epok::ParticlePool& object = /* obtain a valid instance */;

object.remove_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded particle simulation and rendering. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particlepool-scene-emitters-1"></a>

## `epok::ParticlePool::scene_emitters`

**Purpose.** Performs `scene emitters` as part of bounded particle simulation and rendering.

**Exact declaration**

```cpp
template<size_t N>void scene_emitters(std::array<ActorData,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t count)
```

- **Declared at:** [line 40](../../../runtime/particles.hpp#L40)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `int &` | Input/output; inspect the function contract | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const int &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded particle simulation and rendering and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "particles.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// int & objects
// const int & world
// int count

epok::ParticlePool& object = /* obtain a valid instance */;

object.scene_emitters<N>(objects, world, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
