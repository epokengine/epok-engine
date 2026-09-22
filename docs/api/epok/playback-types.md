# Epok API: Playback Types

> **Header:** `"playback_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/playback_types.hpp)

This module covers the playback types module. It documents 9 public callables declared directly in this header.

## Declared types

`epok::bp::PlaybackResult`, `epok::bp::PlaybackSnapshot`, `epok::bp::PlaybackSubscription`, `epok::bp::PlaybackWait`, `epok::effects::Asset`, `epok::effects::Handle`, `epok::timeline::Asset`, `epok::timeline::Handle`

## Callable index

- [`epok::bp::observe_playback`](#epok-bp-observe-playback-1) — Performs `observe playback` as part of the playback types module.
- [`epok::bp::PlaybackSubscription::capture`](#epok-bp-playbacksubscription-capture-1) — Performs `capture` as part of the playback types module.
- [`epok::bp::PlaybackSubscription::rearm`](#epok-bp-playbacksubscription-rearm-1) — Performs `rearm` as part of the playback types module.
- [`epok::bp::PlaybackSubscription::resolve`](#epok-bp-playbacksubscription-resolve-1) — Performs `resolve` as part of the playback types module.
- [`epok::bp::PlaybackSubscription::start`](#epok-bp-playbacksubscription-start-1) — Starts start as part of the playback types module.
- [`epok::bp::PlaybackWait::begin`](#epok-bp-playbackwait-begin-1) — Begins begin as part of the playback types module.
- [`epok::bp::PlaybackWait::begin`](#epok-bp-playbackwait-begin-2) — Begins begin as part of the playback types module.
- [`epok::bp::PlaybackWait::capture`](#epok-bp-playbackwait-capture-1) — Performs `capture` as part of the playback types module.
- [`epok::bp::PlaybackWait::PlaybackWait`](#epok-bp-playbackwait-playbackwait-1) — Constructs `epok::bp::PlaybackWait` for the playback types module.

<a id="epok-bp-observe-playback-1"></a>

## `epok::bp::observe_playback`

**Purpose.** Performs `observe playback` as part of the playback types module.

**Exact declaration**

```cpp
inline void observe_playback()
```

- **Declared at:** [line 20](../../../runtime/playback_types.hpp#L20)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

epok::bp::observe_playback();
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-playbacksubscription-capture-1"></a>

## `epok::bp::PlaybackSubscription::capture`

**Purpose.** Performs `capture` as part of the playback types module.

**Exact declaration**

```cpp
bool capture(PlaybackSnapshot snapshot)
```

- **Declared at:** [line 73](../../../runtime/playback_types.hpp#L73)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `snapshot` | `PlaybackSnapshot` | Input | Value supplied for `snapshot`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

// Assume these named values have been initialized with valid data:
// PlaybackSnapshot snapshot

epok::bp::PlaybackSubscription& object = /* obtain a valid instance */;

auto result = object.capture(snapshot);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-playbacksubscription-rearm-1"></a>

## `epok::bp::PlaybackSubscription::rearm`

**Purpose.** Performs `rearm` as part of the playback types module.

**Exact declaration**

```cpp
void rearm()
```

- **Declared at:** [line 80](../../../runtime/playback_types.hpp#L80)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

epok::bp::PlaybackSubscription& object = /* obtain a valid instance */;

object.rearm();
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-playbacksubscription-resolve-1"></a>

## `epok::bp::PlaybackSubscription::resolve`

**Purpose.** Performs `resolve` as part of the playback types module.

**Exact declaration**

```cpp
bool resolve()
```

- **Declared at:** [line 66](../../../runtime/playback_types.hpp#L66)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

epok::bp::PlaybackSubscription& object = /* obtain a valid instance */;

auto result = object.resolve();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-playbacksubscription-start-1"></a>

## `epok::bp::PlaybackSubscription::start`

**Purpose.** Starts start as part of the playback types module.

**Exact declaration**

```cpp
void start(PlaybackSnapshot snapshot)
```

- **Declared at:** [line 62](../../../runtime/playback_types.hpp#L62)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `snapshot` | `PlaybackSnapshot` | Input | Value supplied for `snapshot`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

// Assume these named values have been initialized with valid data:
// PlaybackSnapshot snapshot

epok::bp::PlaybackSubscription& object = /* obtain a valid instance */;

object.start(snapshot);
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-playbackwait-begin-1"></a>

## `epok::bp::PlaybackWait::begin`

**Purpose.** Begins begin as part of the playback types module.

**Exact declaration**

```cpp
void begin(effects::Handle target)
```

- **Declared at:** [line 43](../../../runtime/playback_types.hpp#L43)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `effects::Handle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle target

epok::bp::PlaybackWait& object = /* obtain a valid instance */;

object.begin(target);
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-playbackwait-begin-2"></a>

## `epok::bp::PlaybackWait::begin`

**Purpose.** Begins begin as part of the playback types module.

**Exact declaration**

```cpp
void begin(timeline::Handle target,uint64_t expected_asset=0,uint64_t marker_id=0,uint32_t after=0)
```

- **Declared at:** [line 39](../../../runtime/playback_types.hpp#L39)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `timeline::Handle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `expected_asset` | `uint64_t` | Input | Value supplied for `expected_asset`. See the exact type and module contract. |
| `marker_id` | `uint64_t` | Input | Value supplied for `marker_id`. See the exact type and module contract. |
| `after` | `uint32_t` | Input | Value supplied for `after`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle target
// uint64_t expected_asset
// uint64_t marker_id
// uint32_t after

epok::bp::PlaybackWait& object = /* obtain a valid instance */;

object.begin(target, expected_asset, marker_id, after);
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-playbackwait-capture-1"></a>

## `epok::bp::PlaybackWait::capture`

**Purpose.** Performs `capture` as part of the playback types module.

**Exact declaration**

```cpp
bool capture(PlaybackSnapshot snapshot)
```

- **Declared at:** [line 47](../../../runtime/playback_types.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `snapshot` | `PlaybackSnapshot` | Input | Value supplied for `snapshot`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

// Assume these named values have been initialized with valid data:
// PlaybackSnapshot snapshot

epok::bp::PlaybackWait& object = /* obtain a valid instance */;

auto result = object.capture(snapshot);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-playbackwait-playbackwait-1"></a>

## `epok::bp::PlaybackWait::PlaybackWait`

**Purpose.** Constructs `epok::bp::PlaybackWait` for the playback types module.

**Exact declaration**

```cpp
PlaybackWait():seq
```

- **Declared at:** [line 38](../../../runtime/playback_types.hpp#L38)
- **Kind:** `constructor`

**Use it when.** You need the playback types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "playback_types.hpp"

epok::bp::PlaybackWait value();
```

**Why choose it.** It provides direct, allocation-conscious access to the playback types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
