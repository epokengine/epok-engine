# Epok API: Transition

> **Header:** `"transition.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/transition.hpp)

This module covers scene loading transitions and fade state. It documents 7 public callables declared directly in this header.

## Declared types

`epok::LoadingImage`, `epok::LoadingOptions`, `epok::TransitionOptions`, `epok::TransitionPhase`, `epok::TransitionState`

## Callable index

- [`epok::transition_audio_gain`](#epok-transition-audio-gain-1) — Read-only to gameplay.
- [`epok::TransitionState::advance`](#epok-transitionstate-advance-1) — Performs `advance` as part of scene loading transitions and fade state.
- [`epok::TransitionState::begin`](#epok-transitionstate-begin-1) — Begins begin as part of scene loading transitions and fade state.
- [`epok::TransitionState::busy`](#epok-transitionstate-busy-1) — Performs `busy` as part of scene loading transitions and fade state.
- [`epok::TransitionState::fail`](#epok-transitionstate-fail-1) — Performs `fail` as part of scene loading transitions and fade state.
- [`epok::TransitionState::loaded`](#epok-transitionstate-loaded-1) — Loads ed as part of scene loading transitions and fade state.
- [`epok::TransitionState::loading`](#epok-transitionstate-loading-1) — Loads ing as part of scene loading transitions and fade state.

<a id="epok-transition-audio-gain-1"></a>

## `epok::transition_audio_gain`

**Purpose.** Read-only to gameplay.

**Details.** The transition gain never overwrites AudioSource.volume.

**Exact declaration**

```cpp
inline uint16_t transition_audio_gain()
```

- **Declared at:** [line 60](../../../runtime/transition.hpp#L60)
- **Kind:** `function decl`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** The transition gain never overwrites AudioSource.volume.

**Usage pattern**

```cpp
#include "transition.hpp"

auto result = epok::transition_audio_gain();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-transitionstate-advance-1"></a>

## `epok::TransitionState::advance`

**Purpose.** Performs `advance` as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
void advance(uint32_t now)
```

- **Declared at:** [line 45](../../../runtime/transition.hpp#L45)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::TransitionState& object = /* obtain a valid instance */;

object.advance(now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene loading transitions and fade state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-transitionstate-begin-1"></a>

## `epok::TransitionState::begin`

**Purpose.** Begins begin as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
void begin(const TransitionOptions& value,uint32_t now,bool initial=false)
```

- **Declared at:** [line 35](../../../runtime/transition.hpp#L35)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const TransitionOptions &` | Input | Value supplied for `value`. See the exact type and module contract. |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |
| `initial` | `bool` | Input | Value supplied for `initial`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

// Assume these named values have been initialized with valid data:
// const TransitionOptions & value
// uint32_t now
// bool initial

epok::TransitionState& object = /* obtain a valid instance */;

object.begin(value, now, initial);
```

**Why choose it.** It provides direct, allocation-conscious access to scene loading transitions and fade state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-transitionstate-busy-1"></a>

## `epok::TransitionState::busy`

**Purpose.** Performs `busy` as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
bool busy() const
```

- **Declared at:** [line 43](../../../runtime/transition.hpp#L43)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

epok::TransitionState& object = /* obtain a valid instance */;

auto result = object.busy();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-transitionstate-fail-1"></a>

## `epok::TransitionState::fail`

**Purpose.** Performs `fail` as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
void fail()
```

- **Declared at:** [line 56](../../../runtime/transition.hpp#L56)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

epok::TransitionState& object = /* obtain a valid instance */;

object.fail();
```

**Why choose it.** It provides direct, allocation-conscious access to scene loading transitions and fade state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-transitionstate-loaded-1"></a>

## `epok::TransitionState::loaded`

**Purpose.** Loads ed as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
void loaded(uint32_t now)
```

- **Declared at:** [line 55](../../../runtime/transition.hpp#L55)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::TransitionState& object = /* obtain a valid instance */;

object.loaded(now);
```

**Why choose it.** It provides direct, allocation-conscious access to scene loading transitions and fade state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-transitionstate-loading-1"></a>

## `epok::TransitionState::loading`

**Purpose.** Loads ing as part of scene loading transitions and fade state.

**Exact declaration**

```cpp
bool loading() const
```

- **Declared at:** [line 44](../../../runtime/transition.hpp#L44)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene loading transitions and fade state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "transition.hpp"

epok::TransitionState& object = /* obtain a valid instance */;

auto result = object.loading();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
