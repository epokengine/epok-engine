# Epok API: Gameplay Api

> **Header:** `"gameplay_api.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gameplay_api.hpp)

This module covers the gameplay api module. It documents 66 public callables declared directly in this header.

## Declared types

`epok::CardFileSample`, `epok::CollisionHitSample`, `epok::CollisionLibrary`, `epok::GameplayAabb`, `epok::GameplayCamera2D`, `epok::GameplayPlaybackSnapshot`, `epok::GameplayTransitionOptions`, `epok::GameplayTransitionSnapshot`, `epok::GameplayVector2`, `epok::GameplayVector3`, `epok::InputAxisSample`, `epok::InputLibrary`, `epok::MathLibrary`, `epok::MemoryCardLibrary`, `epok::MemoryCardSnapshot`, `epok::MoveSample`, `epok::PlaybackLibrary`, `epok::ProjectedPoint`, `epok::ResourceLibrary`, `epok::ResourceSnapshot`, `epok::SavePayload8`, `epok::SceneLibrary`, `epok::SceneSnapshot`, `epok::SkeletalQuerySnapshot`, `epok::TimeLibrary`, `epok::TimeSnapshot`, `epok::World2DLibrary`

## Callable index

- [`epok::CollisionLibrary::ground`](#epok-collisionlibrary-ground-1) — Performs `ground` as part of the gameplay api module.
- [`epok::CollisionLibrary::move`](#epok-collisionlibrary-move-1) — Performs `move` as part of the gameplay api module.
- [`epok::CollisionLibrary::overlap_box`](#epok-collisionlibrary-overlap-box-1) — Performs `overlap box` as part of the gameplay api module.
- [`epok::CollisionLibrary::raycast_segment`](#epok-collisionlibrary-raycast-segment-1) — Performs `raycast segment` as part of the gameplay api module.
- [`epok::gameplay_actor_data`](#epok-gameplay-actor-data-1) — Performs `gameplay actor data` as part of the gameplay api module.
- [`epok::gameplay_actor_id`](#epok-gameplay-actor-id-1) — Performs `gameplay actor id` as part of the gameplay api module.
- [`epok::gameplay_card_slot_name`](#epok-gameplay-card-slot-name-1) — Performs `gameplay card slot name` as part of the gameplay api module.
- [`epok::gameplay_i16`](#epok-gameplay-i16-1) — Performs `gameplay i16` as part of the gameplay api module.
- [`epok::gameplay_save_words`](#epok-gameplay-save-words-1) — Performs `gameplay save words` as part of the gameplay api module.
- [`epok::gameplay_u16`](#epok-gameplay-u16-1) — Performs `gameplay u16` as part of the gameplay api module.
- [`epok::gameplay_u8`](#epok-gameplay-u8-1) — Performs `gameplay u8` as part of the gameplay api module.
- [`epok::gameplay_vector`](#epok-gameplay-vector-1) — Performs `gameplay vector` as part of the gameplay api module.
- [`epok::gameplay_vector`](#epok-gameplay-vector-2) — Performs `gameplay vector` as part of the gameplay api module.
- [`epok::InputLibrary::analog`](#epok-inputlibrary-analog-1) — Performs `analog` as part of the gameplay api module.
- [`epok::InputLibrary::axis`](#epok-inputlibrary-axis-1) — Performs `axis` as part of the gameplay api module.
- [`epok::InputLibrary::connected`](#epok-inputlibrary-connected-1) — Performs `connected` as part of the gameplay api module.
- [`epok::InputLibrary::frame_pressed`](#epok-inputlibrary-frame-pressed-1) — Performs `frame pressed` as part of the gameplay api module.
- [`epok::InputLibrary::frame_released`](#epok-inputlibrary-frame-released-1) — Performs `frame released` as part of the gameplay api module.
- [`epok::InputLibrary::held`](#epok-inputlibrary-held-1) — Performs `held` as part of the gameplay api module.
- [`epok::InputLibrary::pressed`](#epok-inputlibrary-pressed-1) — Performs `pressed` as part of the gameplay api module.
- [`epok::InputLibrary::released`](#epok-inputlibrary-released-1) — Performs `released` as part of the gameplay api module.
- [`epok::MathLibrary::add`](#epok-mathlibrary-add-1) — Adds add as part of the gameplay api module.
- [`epok::MathLibrary::clamp`](#epok-mathlibrary-clamp-1) — Performs `clamp` as part of the gameplay api module.
- [`epok::MathLibrary::lerp`](#epok-mathlibrary-lerp-1) — Performs `lerp` as part of the gameplay api module.
- [`epok::MathLibrary::scale`](#epok-mathlibrary-scale-1) — Performs `scale` as part of the gameplay api module.
- [`epok::MathLibrary::smoothstep`](#epok-mathlibrary-smoothstep-1) — Performs `smoothstep` as part of the gameplay api module.
- [`epok::MathLibrary::vector3`](#epok-mathlibrary-vector3-1) — Performs `vector3` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::clear_staged_payload`](#epok-memorycardlibrary-clear-staged-payload-1) — Clears staged payload as part of the gameplay api module.
- [`epok::MemoryCardLibrary::file`](#epok-memorycardlibrary-file-1) — Performs `file` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::list`](#epok-memorycardlibrary-list-1) — Performs `list` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::loaded_word`](#epok-memorycardlibrary-loaded-word-1) — Loads ed word as part of the gameplay api module.
- [`epok::MemoryCardLibrary::payload`](#epok-memorycardlibrary-payload-1) — Performs `payload` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::probe`](#epok-memorycardlibrary-probe-1) — Performs `probe` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::read`](#epok-memorycardlibrary-read-1) — Reads read as part of the gameplay api module.
- [`epok::MemoryCardLibrary::set_staged_word`](#epok-memorycardlibrary-set-staged-word-1) — Sets staged word as part of the gameplay api module.
- [`epok::MemoryCardLibrary::snapshot`](#epok-memorycardlibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::staged_word`](#epok-memorycardlibrary-staged-word-1) — Performs `staged word` as part of the gameplay api module.
- [`epok::MemoryCardLibrary::write`](#epok-memorycardlibrary-write-1) — Writes write as part of the gameplay api module.
- [`epok::MemoryCardLibrary::write_staged`](#epok-memorycardlibrary-write-staged-1) — Writes staged as part of the gameplay api module.
- [`epok::PlaybackLibrary::burst_effect`](#epok-playbacklibrary-burst-effect-1) — Performs `burst effect` as part of the gameplay api module.
- [`epok::PlaybackLibrary::effect_sequence`](#epok-playbacklibrary-effect-sequence-1) — Performs `effect sequence` as part of the gameplay api module.
- [`epok::PlaybackLibrary::effect_state`](#epok-playbacklibrary-effect-state-1) — Performs `effect state` as part of the gameplay api module.
- [`epok::PlaybackLibrary::pause_effect`](#epok-playbacklibrary-pause-effect-1) — Pauses effect as part of the gameplay api module.
- [`epok::PlaybackLibrary::pause_sequence`](#epok-playbacklibrary-pause-sequence-1) — Pauses sequence as part of the gameplay api module.
- [`epok::PlaybackLibrary::play_effect`](#epok-playbacklibrary-play-effect-1) — Starts effect as part of the gameplay api module.
- [`epok::PlaybackLibrary::play_sequence`](#epok-playbacklibrary-play-sequence-1) — Starts sequence as part of the gameplay api module.
- [`epok::PlaybackLibrary::resume_effect`](#epok-playbacklibrary-resume-effect-1) — Resumes effect as part of the gameplay api module.
- [`epok::PlaybackLibrary::resume_sequence`](#epok-playbacklibrary-resume-sequence-1) — Resumes sequence as part of the gameplay api module.
- [`epok::PlaybackLibrary::sequence_state`](#epok-playbacklibrary-sequence-state-1) — Performs `sequence state` as part of the gameplay api module.
- [`epok::PlaybackLibrary::stop_effect`](#epok-playbacklibrary-stop-effect-1) — Stops effect as part of the gameplay api module.
- [`epok::PlaybackLibrary::stop_sequence`](#epok-playbacklibrary-stop-sequence-1) — Stops sequence as part of the gameplay api module.
- [`epok::ResourceLibrary::clear_skeletal_queries`](#epok-resourcelibrary-clear-skeletal-queries-1) — Clears skeletal queries as part of the gameplay api module.
- [`epok::ResourceLibrary::skeletal_queries`](#epok-resourcelibrary-skeletal-queries-1) — Performs `skeletal queries` as part of the gameplay api module.
- [`epok::ResourceLibrary::snapshot`](#epok-resourcelibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::SceneLibrary::active_camera_actor`](#epok-scenelibrary-active-camera-actor-1) — Performs `active camera actor` as part of the gameplay api module.
- [`epok::SceneLibrary::project`](#epok-scenelibrary-project-1) — Performs `project` as part of the gameplay api module.
- [`epok::SceneLibrary::request`](#epok-scenelibrary-request-1) — Requests request as part of the gameplay api module.
- [`epok::SceneLibrary::request_with_transition`](#epok-scenelibrary-request-with-transition-1) — Requests with transition as part of the gameplay api module.
- [`epok::SceneLibrary::set_camera`](#epok-scenelibrary-set-camera-1) — Sets camera as part of the gameplay api module.
- [`epok::SceneLibrary::snapshot`](#epok-scenelibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::SceneLibrary::transition_snapshot`](#epok-scenelibrary-transition-snapshot-1) — Performs `transition snapshot` as part of the gameplay api module.
- [`epok::TimeLibrary::paused`](#epok-timelibrary-paused-1) — Pauses d as part of the gameplay api module.
- [`epok::TimeLibrary::set_paused`](#epok-timelibrary-set-paused-1) — Sets paused as part of the gameplay api module.
- [`epok::TimeLibrary::snapshot`](#epok-timelibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::World2DLibrary::screen_to_world`](#epok-world2dlibrary-screen-to-world-1) — Performs `screen to world` as part of the gameplay api module.
- [`epok::World2DLibrary::world_to_screen`](#epok-world2dlibrary-world-to-screen-1) — Performs `world to screen` as part of the gameplay api module.

<a id="epok-collisionlibrary-ground-1"></a>

## `epok::CollisionLibrary::ground`

**Purpose.** Performs `ground` as part of the gameplay api module.

**Exact declaration**

```cpp
static CollisionHitSample ground(ObjectId actor,Fixed distance,uint32_t mask)
```

- **Declared at:** [line 168](../../../runtime/gameplay_api.hpp#L168)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `distance` | `int` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `CollisionHitSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// int distance
// int mask

auto result = epok::CollisionLibrary::ground(actor, distance, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionlibrary-move-1"></a>

## `epok::CollisionLibrary::move`

**Purpose.** Performs `move` as part of the gameplay api module.

**Exact declaration**

```cpp
static MoveSample move(ObjectId actor,GameplayVector3 displacement,uint32_t mask)
```

- **Declared at:** [line 171](../../../runtime/gameplay_api.hpp#L171)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `displacement` | `GameplayVector3` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// GameplayVector3 displacement
// int mask

auto result = epok::CollisionLibrary::move(actor, displacement, mask);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionlibrary-overlap-box-1"></a>

## `epok::CollisionLibrary::overlap_box`

**Purpose.** Performs `overlap box` as part of the gameplay api module.

**Exact declaration**

```cpp
static ObjectBatch8 overlap_box(GameplayAabb bounds,uint32_t mask,ObjectId ignore,bool triggers)
```

- **Declared at:** [line 165](../../../runtime/gameplay_api.hpp#L165)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bounds` | `GameplayAabb` | Input | Value supplied for `bounds`. See the exact type and module contract. |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `ObjectBatch8`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAabb bounds
// int mask
// ObjectId ignore
// bool triggers

auto result = epok::CollisionLibrary::overlap_box(bounds, mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collisionlibrary-raycast-segment-1"></a>

## `epok::CollisionLibrary::raycast_segment`

**Purpose.** Performs `raycast segment` as part of the gameplay api module.

**Exact declaration**

```cpp
static CollisionHitSample raycast_segment(GameplayVector3 origin,GameplayVector3 displacement,uint32_t mask,ObjectId ignore,bool triggers)
```

- **Declared at:** [line 162](../../../runtime/gameplay_api.hpp#L162)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `GameplayVector3` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `CollisionHitSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 origin
// GameplayVector3 displacement
// int mask
// ObjectId ignore
// bool triggers

auto result = epok::CollisionLibrary::raycast_segment(origin, displacement, mask, ignore, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-actor-data-1"></a>

## `epok::gameplay_actor_data`

**Purpose.** Performs `gameplay actor data` as part of the gameplay api module.

**Exact declaration**

```cpp
inline ActorData* gameplay_actor_data(ObjectId id)
```

- **Declared at:** [line 99](../../../runtime/gameplay_api.hpp#L99)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::gameplay_actor_data(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-actor-id-1"></a>

## `epok::gameplay_actor_id`

**Purpose.** Performs `gameplay actor id` as part of the gameplay api module.

**Exact declaration**

```cpp
inline ObjectId gameplay_actor_id(DataHandle handle)
```

- **Declared at:** [line 100](../../../runtime/gameplay_api.hpp#L100)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DataHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle handle

auto result = epok::gameplay_actor_id(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-card-slot-name-1"></a>

## `epok::gameplay_card_slot_name`

**Purpose.** Performs `gameplay card slot name` as part of the gameplay api module.

**Exact declaration**

```cpp
inline const char* gameplay_card_slot_name(uint32_t slot)
```

- **Declared at:** [line 101](../../../runtime/gameplay_api.hpp#L101)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int slot

auto result = epok::gameplay_card_slot_name(slot);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-i16-1"></a>

## `epok::gameplay_i16`

**Purpose.** Performs `gameplay i16` as part of the gameplay api module.

**Exact declaration**

```cpp
inline int16_t gameplay_i16(int32_t value)
```

- **Declared at:** [line 102](../../../runtime/gameplay_api.hpp#L102)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int value

auto result = epok::gameplay_i16(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-save-words-1"></a>

## `epok::gameplay_save_words`

**Purpose.** Performs `gameplay save words` as part of the gameplay api module.

**Exact declaration**

```cpp
inline uint32_t* gameplay_save_words()
```

- **Declared at:** [line 103](../../../runtime/gameplay_api.hpp#L103)
- **Kind:** `function decl`

**Returns.** Returns `int *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::gameplay_save_words();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-u16-1"></a>

## `epok::gameplay_u16`

**Purpose.** Performs `gameplay u16` as part of the gameplay api module.

**Exact declaration**

```cpp
inline uint16_t gameplay_u16(uint32_t value)
```

- **Declared at:** [line 104](../../../runtime/gameplay_api.hpp#L104)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int value

auto result = epok::gameplay_u16(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-u8-1"></a>

## `epok::gameplay_u8`

**Purpose.** Performs `gameplay u8` as part of the gameplay api module.

**Exact declaration**

```cpp
inline uint8_t gameplay_u8(uint32_t value)
```

- **Declared at:** [line 105](../../../runtime/gameplay_api.hpp#L105)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int value

auto result = epok::gameplay_u8(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-vector-1"></a>

## `epok::gameplay_vector`

**Purpose.** Performs `gameplay vector` as part of the gameplay api module.

**Exact declaration**

```cpp
inline GameplayVector3 gameplay_vector(const Fixed* value)
```

- **Declared at:** [line 97](../../../runtime/gameplay_api.hpp#L97)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const int *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// const int * value

auto result = epok::gameplay_vector(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-vector-2"></a>

## `epok::gameplay_vector`

**Purpose.** Performs `gameplay vector` as part of the gameplay api module.

**Exact declaration**

```cpp
inline void gameplay_vector(GameplayVector3 value,Fixed* output)
```

- **Declared at:** [line 98](../../../runtime/gameplay_api.hpp#L98)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayVector3` | Input | Value supplied for `value`. See the exact type and module contract. |
| `output` | `int *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 value
// int * output

epok::gameplay_vector(value, output);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-inputlibrary-analog-1"></a>

## `epok::InputLibrary::analog`

**Purpose.** Performs `analog` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool analog(uint32_t port)
```

- **Declared at:** [line 111](../../../runtime/gameplay_api.hpp#L111)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int port

auto result = epok::InputLibrary::analog(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-axis-1"></a>

## `epok::InputLibrary::axis`

**Purpose.** Performs `axis` as part of the gameplay api module.

**Exact declaration**

```cpp
static InputAxisSample axis(Axis axis,uint32_t port)
```

- **Declared at:** [line 123](../../../runtime/gameplay_api.hpp#L123)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputAxisSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// int port

auto result = epok::InputLibrary::axis(axis, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-inputlibrary-connected-1"></a>

## `epok::InputLibrary::connected`

**Purpose.** Performs `connected` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool connected(uint32_t port)
```

- **Declared at:** [line 109](../../../runtime/gameplay_api.hpp#L109)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int port

auto result = epok::InputLibrary::connected(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-frame-pressed-1"></a>

## `epok::InputLibrary::frame_pressed`

**Purpose.** Performs `frame pressed` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool frame_pressed(Button button,uint32_t port)
```

- **Declared at:** [line 119](../../../runtime/gameplay_api.hpp#L119)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// int port

auto result = epok::InputLibrary::frame_pressed(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-frame-released-1"></a>

## `epok::InputLibrary::frame_released`

**Purpose.** Performs `frame released` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool frame_released(Button button,uint32_t port)
```

- **Declared at:** [line 121](../../../runtime/gameplay_api.hpp#L121)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// int port

auto result = epok::InputLibrary::frame_released(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-held-1"></a>

## `epok::InputLibrary::held`

**Purpose.** Performs `held` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool held(Button button,uint32_t port)
```

- **Declared at:** [line 113](../../../runtime/gameplay_api.hpp#L113)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// int port

auto result = epok::InputLibrary::held(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-pressed-1"></a>

## `epok::InputLibrary::pressed`

**Purpose.** Performs `pressed` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool pressed(Button button,uint32_t port)
```

- **Declared at:** [line 115](../../../runtime/gameplay_api.hpp#L115)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// int port

auto result = epok::InputLibrary::pressed(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-inputlibrary-released-1"></a>

## `epok::InputLibrary::released`

**Purpose.** Performs `released` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool released(Button button,uint32_t port)
```

- **Declared at:** [line 117](../../../runtime/gameplay_api.hpp#L117)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// int port

auto result = epok::InputLibrary::released(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-mathlibrary-add-1"></a>

## `epok::MathLibrary::add`

**Purpose.** Adds add as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 add(GameplayVector3 a,GameplayVector3 b)
```

- **Declared at:** [line 152](../../../runtime/gameplay_api.hpp#L152)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `GameplayVector3` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `GameplayVector3` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 a
// GameplayVector3 b

auto result = epok::MathLibrary::add(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-clamp-1"></a>

## `epok::MathLibrary::clamp`

**Purpose.** Performs `clamp` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed clamp(Fixed value,Fixed minimum,Fixed maximum)
```

- **Declared at:** [line 148](../../../runtime/gameplay_api.hpp#L148)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |
| `minimum` | `int` | Input | Value supplied for `minimum`. See the exact type and module contract. |
| `maximum` | `int` | Input | Value supplied for `maximum`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int value
// int minimum
// int maximum

auto result = epok::MathLibrary::clamp(value, minimum, maximum);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-lerp-1"></a>

## `epok::MathLibrary::lerp`

**Purpose.** Performs `lerp` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed lerp(Fixed from,Fixed to,Fixed alpha)
```

- **Declared at:** [line 149](../../../runtime/gameplay_api.hpp#L149)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |
| `alpha` | `int` | Input | Value supplied for `alpha`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int from
// int to
// int alpha

auto result = epok::MathLibrary::lerp(from, to, alpha);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-scale-1"></a>

## `epok::MathLibrary::scale`

**Purpose.** Performs `scale` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 scale(GameplayVector3 value,Fixed amount)
```

- **Declared at:** [line 153](../../../runtime/gameplay_api.hpp#L153)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayVector3` | Input | Value supplied for `value`. See the exact type and module contract. |
| `amount` | `int` | Input | Value supplied for `amount`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 value
// int amount

auto result = epok::MathLibrary::scale(value, amount);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-smoothstep-1"></a>

## `epok::MathLibrary::smoothstep`

**Purpose.** Performs `smoothstep` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed smoothstep(Fixed alpha)
```

- **Declared at:** [line 150](../../../runtime/gameplay_api.hpp#L150)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `alpha` | `int` | Input | Value supplied for `alpha`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int alpha

auto result = epok::MathLibrary::smoothstep(alpha);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-vector3-1"></a>

## `epok::MathLibrary::vector3`

**Purpose.** Performs `vector3` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 vector3(Fixed x,Fixed y,Fixed z)
```

- **Declared at:** [line 151](../../../runtime/gameplay_api.hpp#L151)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `int` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int x
// int y
// int z

auto result = epok::MathLibrary::vector3(x, y, z);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-memorycardlibrary-clear-staged-payload-1"></a>

## `epok::MemoryCardLibrary::clear_staged_payload`

**Purpose.** Clears staged payload as part of the gameplay api module.

**Exact declaration**

```cpp
static void clear_staged_payload()
```

- **Declared at:** [line 224](../../../runtime/gameplay_api.hpp#L224)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

epok::MemoryCardLibrary::clear_staged_payload();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-file-1"></a>

## `epok::MemoryCardLibrary::file`

**Purpose.** Performs `file` as part of the gameplay api module.

**Exact declaration**

```cpp
static CardFileSample file(uint32_t index)
```

- **Declared at:** [line 223](../../../runtime/gameplay_api.hpp#L223)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `CardFileSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index

auto result = epok::MemoryCardLibrary::file(index);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-list-1"></a>

## `epok::MemoryCardLibrary::list`

**Purpose.** Performs `list` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool list(uint32_t port)
```

- **Declared at:** [line 219](../../../runtime/gameplay_api.hpp#L219)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int port

auto result = epok::MemoryCardLibrary::list(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-loaded-word-1"></a>

## `epok::MemoryCardLibrary::loaded_word`

**Purpose.** Loads ed word as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t loaded_word(uint32_t index)
```

- **Declared at:** [line 228](../../../runtime/gameplay_api.hpp#L228)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index

auto result = epok::MemoryCardLibrary::loaded_word(index);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-payload-1"></a>

## `epok::MemoryCardLibrary::payload`

**Purpose.** Performs `payload` as part of the gameplay api module.

**Exact declaration**

```cpp
static SavePayload8 payload()
```

- **Declared at:** [line 222](../../../runtime/gameplay_api.hpp#L222)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `SavePayload8`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::MemoryCardLibrary::payload();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-probe-1"></a>

## `epok::MemoryCardLibrary::probe`

**Purpose.** Performs `probe` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool probe(uint32_t port)
```

- **Declared at:** [line 218](../../../runtime/gameplay_api.hpp#L218)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int port

auto result = epok::MemoryCardLibrary::probe(port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-read-1"></a>

## `epok::MemoryCardLibrary::read`

**Purpose.** Reads read as part of the gameplay api module.

**Exact declaration**

```cpp
static bool read(uint32_t slot,uint32_t port)
```

- **Declared at:** [line 220](../../../runtime/gameplay_api.hpp#L220)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int slot
// int port

auto result = epok::MemoryCardLibrary::read(slot, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-set-staged-word-1"></a>

## `epok::MemoryCardLibrary::set_staged_word`

**Purpose.** Sets staged word as part of the gameplay api module.

**Exact declaration**

```cpp
static bool set_staged_word(uint32_t index,uint32_t value)
```

- **Declared at:** [line 225](../../../runtime/gameplay_api.hpp#L225)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index
// int value

auto result = epok::MemoryCardLibrary::set_staged_word(index, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-snapshot-1"></a>

## `epok::MemoryCardLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static MemoryCardSnapshot snapshot()
```

- **Declared at:** [line 217](../../../runtime/gameplay_api.hpp#L217)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `MemoryCardSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::MemoryCardLibrary::snapshot();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-staged-word-1"></a>

## `epok::MemoryCardLibrary::staged_word`

**Purpose.** Performs `staged word` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t staged_word(uint32_t index)
```

- **Declared at:** [line 226](../../../runtime/gameplay_api.hpp#L226)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index

auto result = epok::MemoryCardLibrary::staged_word(index);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-write-1"></a>

## `epok::MemoryCardLibrary::write`

**Purpose.** Writes write as part of the gameplay api module.

**Exact declaration**

```cpp
static bool write(uint32_t slot,SavePayload8 payload,uint32_t port)
```

- **Declared at:** [line 221](../../../runtime/gameplay_api.hpp#L221)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `payload` | `SavePayload8` | Input | Value supplied for `payload`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int slot
// SavePayload8 payload
// int port

auto result = epok::MemoryCardLibrary::write(slot, payload, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-memorycardlibrary-write-staged-1"></a>

## `epok::MemoryCardLibrary::write_staged`

**Purpose.** Writes staged as part of the gameplay api module.

**Exact declaration**

```cpp
static bool write_staged(uint32_t slot,uint32_t bytes,uint32_t port)
```

- **Declared at:** [line 227](../../../runtime/gameplay_api.hpp#L227)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `int` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `bytes` | `int` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int slot
// int bytes
// int port

auto result = epok::MemoryCardLibrary::write_staged(slot, bytes, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-playbacklibrary-burst-effect-1"></a>

## `epok::PlaybackLibrary::burst_effect`

**Purpose.** Performs `burst effect` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool burst_effect(effects::Handle handle,uint32_t count)
```

- **Declared at:** [line 211](../../../runtime/gameplay_api.hpp#L211)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle
// int count

auto result = epok::PlaybackLibrary::burst_effect(handle, count);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-effect-sequence-1"></a>

## `epok::PlaybackLibrary::effect_sequence`

**Purpose.** Performs `effect sequence` as part of the gameplay api module.

**Exact declaration**

```cpp
static timeline::Handle effect_sequence(effects::Handle handle)
```

- **Declared at:** [line 213](../../../runtime/gameplay_api.hpp#L213)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `timeline::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle

auto result = epok::PlaybackLibrary::effect_sequence(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-playbacklibrary-effect-state-1"></a>

## `epok::PlaybackLibrary::effect_state`

**Purpose.** Performs `effect state` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayPlaybackSnapshot effect_state(effects::Handle handle)
```

- **Declared at:** [line 212](../../../runtime/gameplay_api.hpp#L212)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `GameplayPlaybackSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle

auto result = epok::PlaybackLibrary::effect_state(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-playbacklibrary-pause-effect-1"></a>

## `epok::PlaybackLibrary::pause_effect`

**Purpose.** Pauses effect as part of the gameplay api module.

**Exact declaration**

```cpp
static bool pause_effect(effects::Handle handle)
```

- **Declared at:** [line 209](../../../runtime/gameplay_api.hpp#L209)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle

auto result = epok::PlaybackLibrary::pause_effect(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-pause-sequence-1"></a>

## `epok::PlaybackLibrary::pause_sequence`

**Purpose.** Pauses sequence as part of the gameplay api module.

**Exact declaration**

```cpp
static bool pause_sequence(timeline::Handle handle)
```

- **Declared at:** [line 204](../../../runtime/gameplay_api.hpp#L204)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `timeline::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle handle

auto result = epok::PlaybackLibrary::pause_sequence(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-play-effect-1"></a>

## `epok::PlaybackLibrary::play_effect`

**Purpose.** Starts effect as part of the gameplay api module.

**Exact declaration**

```cpp
static effects::Handle play_effect(ObjectId component)
```

- **Declared at:** [line 207](../../../runtime/gameplay_api.hpp#L207)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ObjectId` | Input | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `effects::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId component

auto result = epok::PlaybackLibrary::play_effect(component);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-playbacklibrary-play-sequence-1"></a>

## `epok::PlaybackLibrary::play_sequence`

**Purpose.** Starts sequence as part of the gameplay api module.

**Exact declaration**

```cpp
static timeline::Handle play_sequence(ObjectId component)
```

- **Declared at:** [line 202](../../../runtime/gameplay_api.hpp#L202)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ObjectId` | Input | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `timeline::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId component

auto result = epok::PlaybackLibrary::play_sequence(component);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-playbacklibrary-resume-effect-1"></a>

## `epok::PlaybackLibrary::resume_effect`

**Purpose.** Resumes effect as part of the gameplay api module.

**Exact declaration**

```cpp
static bool resume_effect(effects::Handle handle)
```

- **Declared at:** [line 210](../../../runtime/gameplay_api.hpp#L210)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle

auto result = epok::PlaybackLibrary::resume_effect(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-resume-sequence-1"></a>

## `epok::PlaybackLibrary::resume_sequence`

**Purpose.** Resumes sequence as part of the gameplay api module.

**Exact declaration**

```cpp
static bool resume_sequence(timeline::Handle handle)
```

- **Declared at:** [line 205](../../../runtime/gameplay_api.hpp#L205)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `timeline::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle handle

auto result = epok::PlaybackLibrary::resume_sequence(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-sequence-state-1"></a>

## `epok::PlaybackLibrary::sequence_state`

**Purpose.** Performs `sequence state` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayPlaybackSnapshot sequence_state(timeline::Handle handle)
```

- **Declared at:** [line 206](../../../runtime/gameplay_api.hpp#L206)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `timeline::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `GameplayPlaybackSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle handle

auto result = epok::PlaybackLibrary::sequence_state(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-playbacklibrary-stop-effect-1"></a>

## `epok::PlaybackLibrary::stop_effect`

**Purpose.** Stops effect as part of the gameplay api module.

**Exact declaration**

```cpp
static bool stop_effect(effects::Handle handle)
```

- **Declared at:** [line 208](../../../runtime/gameplay_api.hpp#L208)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle

auto result = epok::PlaybackLibrary::stop_effect(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-playbacklibrary-stop-sequence-1"></a>

## `epok::PlaybackLibrary::stop_sequence`

**Purpose.** Stops sequence as part of the gameplay api module.

**Exact declaration**

```cpp
static bool stop_sequence(timeline::Handle handle)
```

- **Declared at:** [line 203](../../../runtime/gameplay_api.hpp#L203)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `timeline::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle handle

auto result = epok::PlaybackLibrary::stop_sequence(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-resourcelibrary-clear-skeletal-queries-1"></a>

## `epok::ResourceLibrary::clear_skeletal_queries`

**Purpose.** Clears skeletal queries as part of the gameplay api module.

**Exact declaration**

```cpp
static void clear_skeletal_queries()
```

- **Declared at:** [line 198](../../../runtime/gameplay_api.hpp#L198)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

epok::ResourceLibrary::clear_skeletal_queries();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-resourcelibrary-skeletal-queries-1"></a>

## `epok::ResourceLibrary::skeletal_queries`

**Purpose.** Performs `skeletal queries` as part of the gameplay api module.

**Exact declaration**

```cpp
static SkeletalQuerySnapshot skeletal_queries()
```

- **Declared at:** [line 197](../../../runtime/gameplay_api.hpp#L197)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `SkeletalQuerySnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::ResourceLibrary::skeletal_queries();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-resourcelibrary-snapshot-1"></a>

## `epok::ResourceLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static ResourceSnapshot snapshot()
```

- **Declared at:** [line 196](../../../runtime/gameplay_api.hpp#L196)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `ResourceSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::ResourceLibrary::snapshot();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-active-camera-actor-1"></a>

## `epok::SceneLibrary::active_camera_actor`

**Purpose.** Performs `active camera actor` as part of the gameplay api module.

**Exact declaration**

```cpp
static ObjectId active_camera_actor()
```

- **Declared at:** [line 190](../../../runtime/gameplay_api.hpp#L190)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::SceneLibrary::active_camera_actor();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-project-1"></a>

## `epok::SceneLibrary::project`

**Purpose.** Performs `project` as part of the gameplay api module.

**Exact declaration**

```cpp
static ProjectedPoint project(GameplayVector3 world)
```

- **Declared at:** [line 192](../../../runtime/gameplay_api.hpp#L192)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `GameplayVector3` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `ProjectedPoint`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 world

auto result = epok::SceneLibrary::project(world);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-request-1"></a>

## `epok::SceneLibrary::request`

**Purpose.** Requests request as part of the gameplay api module.

**Exact declaration**

```cpp
static bool request(uint32_t index)
```

- **Declared at:** [line 179](../../../runtime/gameplay_api.hpp#L179)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index

auto result = epok::SceneLibrary::request(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenelibrary-request-with-transition-1"></a>

## `epok::SceneLibrary::request_with_transition`

**Purpose.** Requests with transition as part of the gameplay api module.

**Exact declaration**

```cpp
static bool request_with_transition(uint32_t index,GameplayTransitionOptions value)
```

- **Declared at:** [line 180](../../../runtime/gameplay_api.hpp#L180)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `GameplayTransitionOptions` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int index
// GameplayTransitionOptions value

auto result = epok::SceneLibrary::request_with_transition(index, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenelibrary-set-camera-1"></a>

## `epok::SceneLibrary::set_camera`

**Purpose.** Sets camera as part of the gameplay api module.

**Exact declaration**

```cpp
static bool set_camera(ObjectId actor)
```

- **Declared at:** [line 191](../../../runtime/gameplay_api.hpp#L191)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::SceneLibrary::set_camera(actor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenelibrary-snapshot-1"></a>

## `epok::SceneLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static SceneSnapshot snapshot()
```

- **Declared at:** [line 177](../../../runtime/gameplay_api.hpp#L177)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `SceneSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::SceneLibrary::snapshot();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-transition-snapshot-1"></a>

## `epok::SceneLibrary::transition_snapshot`

**Purpose.** Performs `transition snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayTransitionSnapshot transition_snapshot()
```

- **Declared at:** [line 178](../../../runtime/gameplay_api.hpp#L178)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `GameplayTransitionSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::SceneLibrary::transition_snapshot();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timelibrary-paused-1"></a>

## `epok::TimeLibrary::paused`

**Purpose.** Pauses d as part of the gameplay api module.

**Exact declaration**

```cpp
static bool paused()
```

- **Declared at:** [line 142](../../../runtime/gameplay_api.hpp#L142)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::TimeLibrary::paused();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-timelibrary-set-paused-1"></a>

## `epok::TimeLibrary::set_paused`

**Purpose.** Sets paused as part of the gameplay api module.

**Exact declaration**

```cpp
static void set_paused(bool paused)
```

- **Declared at:** [line 144](../../../runtime/gameplay_api.hpp#L144)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// bool paused

epok::TimeLibrary::set_paused(paused);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timelibrary-snapshot-1"></a>

## `epok::TimeLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static TimeSnapshot snapshot()
```

- **Declared at:** [line 135](../../../runtime/gameplay_api.hpp#L135)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `TimeSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::TimeLibrary::snapshot();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2dlibrary-screen-to-world-1"></a>

## `epok::World2DLibrary::screen_to_world`

**Purpose.** Performs `screen to world` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector2 screen_to_world(GameplayCamera2D value,GameplayVector2 screen)
```

- **Declared at:** [line 158](../../../runtime/gameplay_api.hpp#L158)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayCamera2D` | Input | Value supplied for `value`. See the exact type and module contract. |
| `screen` | `GameplayVector2` | Input | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector2`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayCamera2D value
// GameplayVector2 screen

auto result = epok::World2DLibrary::screen_to_world(value, screen);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2dlibrary-world-to-screen-1"></a>

## `epok::World2DLibrary::world_to_screen`

**Purpose.** Performs `world to screen` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector2 world_to_screen(GameplayCamera2D value,GameplayVector2 world)
```

- **Declared at:** [line 157](../../../runtime/gameplay_api.hpp#L157)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayCamera2D` | Input | Value supplied for `value`. See the exact type and module contract. |
| `world` | `GameplayVector2` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector2`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayCamera2D value
// GameplayVector2 world

auto result = epok::World2DLibrary::world_to_screen(value, world);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
