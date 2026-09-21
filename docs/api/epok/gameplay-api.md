# Epok API: Gameplay Api

> **Header:** `"gameplay_api.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gameplay_api.hpp)

This module covers the gameplay api module. It documents 77 public callables declared directly in this header.

## Declared types

`epok::CardFileSample`, `epok::CollisionHitSample`, `epok::CollisionLibrary`, `epok::FogSettings`, `epok::GameplayAabb`, `epok::GameplayCamera2D`, `epok::GameplayPlaybackSnapshot`, `epok::GameplayTransitionOptions`, `epok::GameplayTransitionSnapshot`, `epok::GameplayVector2`, `epok::GameplayVector3`, `epok::GameplayVector3TweenState`, `epok::InputAxisSample`, `epok::InputLibrary`, `epok::MathLibrary`, `epok::MemoryCardLibrary`, `epok::MemoryCardSnapshot`, `epok::MoveSample`, `epok::PlaybackLibrary`, `epok::ProjectedPoint`, `epok::ResourceLibrary`, `epok::ResourceSnapshot`, `epok::SavePayload8`, `epok::SceneLibrary`, `epok::SceneSnapshot`, `epok::SkeletalQuerySnapshot`, `epok::TimeLibrary`, `epok::TimeSnapshot`, `epok::UtilityVectorLibrary`, `epok::Vector3TweenAdvanceSample`, `epok::World2DLibrary`

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
- [`epok::gameplay_vector_tween`](#epok-gameplay-vector-tween-1) — Performs `gameplay vector tween` as part of the gameplay api module.
- [`epok::gameplay_vector_tween_value`](#epok-gameplay-vector-tween-value-1) — Performs `gameplay vector tween value` as part of the gameplay api module.
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
- [`epok::SceneLibrary::fog`](#epok-scenelibrary-fog-1) — Performs `fog` as part of the gameplay api module.
- [`epok::SceneLibrary::project`](#epok-scenelibrary-project-1) — Performs `project` as part of the gameplay api module.
- [`epok::SceneLibrary::request`](#epok-scenelibrary-request-1) — Requests request as part of the gameplay api module.
- [`epok::SceneLibrary::request_with_transition`](#epok-scenelibrary-request-with-transition-1) — Requests with transition as part of the gameplay api module.
- [`epok::SceneLibrary::screen_fade`](#epok-scenelibrary-screen-fade-1) — The post-HUD fade to black, 0 clear to 255 opaque, clamped rather than rejected because every amount above the range has one nearest valid value.
- [`epok::SceneLibrary::set_camera`](#epok-scenelibrary-set-camera-1) — Sets camera as part of the gameplay api module.
- [`epok::SceneLibrary::set_fog`](#epok-scenelibrary-set-fog-1) — Sets fog as part of the gameplay api module.
- [`epok::SceneLibrary::set_screen_fade`](#epok-scenelibrary-set-screen-fade-1) — Sets screen fade as part of the gameplay api module.
- [`epok::SceneLibrary::snapshot`](#epok-scenelibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::SceneLibrary::transition_snapshot`](#epok-scenelibrary-transition-snapshot-1) — Performs `transition snapshot` as part of the gameplay api module.
- [`epok::TimeLibrary::paused`](#epok-timelibrary-paused-1) — Pauses d as part of the gameplay api module.
- [`epok::TimeLibrary::set_paused`](#epok-timelibrary-set-paused-1) — Sets paused as part of the gameplay api module.
- [`epok::TimeLibrary::snapshot`](#epok-timelibrary-snapshot-1) — Performs `snapshot` as part of the gameplay api module.
- [`epok::UtilityVectorLibrary::vector_tween_advance`](#epok-utilityvectorlibrary-vector-tween-advance-1) — Performs `vector tween advance` as part of the gameplay api module.
- [`epok::UtilityVectorLibrary::vector_tween_cancel`](#epok-utilityvectorlibrary-vector-tween-cancel-1) — Performs `vector tween cancel` as part of the gameplay api module.
- [`epok::UtilityVectorLibrary::vector_tween_schedule`](#epok-utilityvectorlibrary-vector-tween-schedule-1) — Performs `vector tween schedule` as part of the gameplay api module.
- [`epok::UtilityVectorLibrary::vector_tween_start`](#epok-utilityvectorlibrary-vector-tween-start-1) — Performs `vector tween start` as part of the gameplay api module.
- [`epok::UtilityVectorLibrary::vector_tween_value`](#epok-utilityvectorlibrary-vector-tween-value-1) — Performs `vector tween value` as part of the gameplay api module.
- [`epok::World2DLibrary::screen_to_world`](#epok-world2dlibrary-screen-to-world-1) — Performs `screen to world` as part of the gameplay api module.
- [`epok::World2DLibrary::world_to_screen`](#epok-world2dlibrary-world-to-screen-1) — Performs `world to screen` as part of the gameplay api module.

<a id="epok-collisionlibrary-ground-1"></a>

## `epok::CollisionLibrary::ground`

**Purpose.** Performs `ground` as part of the gameplay api module.

**Exact declaration**

```cpp
static CollisionHitSample ground(ObjectId actor,Fixed distance,uint32_t mask)
```

- **Declared at:** [line 197](../../../runtime/gameplay_api.hpp#L197)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `CollisionHitSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// Fixed distance
// uint32_t mask

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

- **Declared at:** [line 200](../../../runtime/gameplay_api.hpp#L200)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |
| `displacement` | `GameplayVector3` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `MoveSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor
// GameplayVector3 displacement
// uint32_t mask

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

- **Declared at:** [line 194](../../../runtime/gameplay_api.hpp#L194)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bounds` | `GameplayAabb` | Input | Value supplied for `bounds`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `ignore` | `ObjectId` | Input | Value supplied for `ignore`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `ObjectBatch8`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAabb bounds
// uint32_t mask
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

- **Declared at:** [line 191](../../../runtime/gameplay_api.hpp#L191)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector3` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `GameplayVector3` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
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
// uint32_t mask
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

- **Declared at:** [line 114](../../../runtime/gameplay_api.hpp#L114)
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

- **Declared at:** [line 115](../../../runtime/gameplay_api.hpp#L115)
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

- **Declared at:** [line 116](../../../runtime/gameplay_api.hpp#L116)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot

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

- **Declared at:** [line 117](../../../runtime/gameplay_api.hpp#L117)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

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

- **Declared at:** [line 118](../../../runtime/gameplay_api.hpp#L118)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t *`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 119](../../../runtime/gameplay_api.hpp#L119)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

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

- **Declared at:** [line 120](../../../runtime/gameplay_api.hpp#L120)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

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

- **Declared at:** [line 112](../../../runtime/gameplay_api.hpp#L112)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const Fixed *` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * value

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

- **Declared at:** [line 113](../../../runtime/gameplay_api.hpp#L113)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayVector3` | Input | Value supplied for `value`. See the exact type and module contract. |
| `output` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 value
// Fixed * output

epok::gameplay_vector(value, output);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-vector-tween-1"></a>

## `epok::gameplay_vector_tween`

**Purpose.** Performs `gameplay vector tween` as part of the gameplay api module.

**Exact declaration**

```cpp
inline GameplayVector3TweenState gameplay_vector_tween(GameplayVector3 from,GameplayVector3 to,Tween timing)
```

- **Declared at:** [line 121](../../../runtime/gameplay_api.hpp#L121)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `GameplayVector3` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `GameplayVector3` | Input | Value supplied for `to`. See the exact type and module contract. |
| `timing` | `Tween` | Input | Value supplied for `timing`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3TweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 from
// GameplayVector3 to
// Tween timing

auto result = epok::gameplay_vector_tween(from, to, timing);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-vector-tween-value-1"></a>

## `epok::gameplay_vector_tween_value`

**Purpose.** Performs `gameplay vector tween value` as part of the gameplay api module.

**Exact declaration**

```cpp
inline GameplayVector3 gameplay_vector_tween_value(GameplayVector3TweenState state)
```

- **Declared at:** [line 122](../../../runtime/gameplay_api.hpp#L122)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayVector3TweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3TweenState state

auto result = epok::gameplay_vector_tween_value(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-inputlibrary-analog-1"></a>

## `epok::InputLibrary::analog`

**Purpose.** Performs `analog` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool analog(uint32_t port)
```

- **Declared at:** [line 128](../../../runtime/gameplay_api.hpp#L128)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t port

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

- **Declared at:** [line 140](../../../runtime/gameplay_api.hpp#L140)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputAxisSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// uint32_t port

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

- **Declared at:** [line 126](../../../runtime/gameplay_api.hpp#L126)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t port

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

- **Declared at:** [line 136](../../../runtime/gameplay_api.hpp#L136)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

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

- **Declared at:** [line 138](../../../runtime/gameplay_api.hpp#L138)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

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

- **Declared at:** [line 130](../../../runtime/gameplay_api.hpp#L130)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

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

- **Declared at:** [line 132](../../../runtime/gameplay_api.hpp#L132)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

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

- **Declared at:** [line 134](../../../runtime/gameplay_api.hpp#L134)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

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

- **Declared at:** [line 169](../../../runtime/gameplay_api.hpp#L169)
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

- **Declared at:** [line 165](../../../runtime/gameplay_api.hpp#L165)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |
| `minimum` | `Fixed` | Input | Value supplied for `minimum`. See the exact type and module contract. |
| `maximum` | `Fixed` | Input | Value supplied for `maximum`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value
// Fixed minimum
// Fixed maximum

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

- **Declared at:** [line 166](../../../runtime/gameplay_api.hpp#L166)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `Fixed` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `Fixed` | Input | Value supplied for `to`. See the exact type and module contract. |
| `alpha` | `Fixed` | Input | Value supplied for `alpha`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed from
// Fixed to
// Fixed alpha

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

- **Declared at:** [line 170](../../../runtime/gameplay_api.hpp#L170)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayVector3` | Input | Value supplied for `value`. See the exact type and module contract. |
| `amount` | `Fixed` | Input | Value supplied for `amount`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 value
// Fixed amount

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

- **Declared at:** [line 167](../../../runtime/gameplay_api.hpp#L167)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `alpha` | `Fixed` | Input | Value supplied for `alpha`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed alpha

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

- **Declared at:** [line 168](../../../runtime/gameplay_api.hpp#L168)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |
| `z` | `Fixed` | Input | Value supplied for `z`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y
// Fixed z

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

- **Declared at:** [line 277](../../../runtime/gameplay_api.hpp#L277)
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

- **Declared at:** [line 276](../../../runtime/gameplay_api.hpp#L276)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `CardFileSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

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

- **Declared at:** [line 272](../../../runtime/gameplay_api.hpp#L272)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t port

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

- **Declared at:** [line 281](../../../runtime/gameplay_api.hpp#L281)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

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

- **Declared at:** [line 275](../../../runtime/gameplay_api.hpp#L275)
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

- **Declared at:** [line 271](../../../runtime/gameplay_api.hpp#L271)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t port

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

- **Declared at:** [line 273](../../../runtime/gameplay_api.hpp#L273)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot
// uint32_t port

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

- **Declared at:** [line 278](../../../runtime/gameplay_api.hpp#L278)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// uint32_t value

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

- **Declared at:** [line 270](../../../runtime/gameplay_api.hpp#L270)
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

- **Declared at:** [line 279](../../../runtime/gameplay_api.hpp#L279)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

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

- **Declared at:** [line 274](../../../runtime/gameplay_api.hpp#L274)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `payload` | `SavePayload8` | Input | Value supplied for `payload`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot
// SavePayload8 payload
// uint32_t port

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

- **Declared at:** [line 280](../../../runtime/gameplay_api.hpp#L280)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot
// uint32_t bytes
// uint32_t port

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

- **Declared at:** [line 264](../../../runtime/gameplay_api.hpp#L264)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `effects::Handle` | Input | Value supplied for `handle`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle handle
// uint32_t count

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

- **Declared at:** [line 266](../../../runtime/gameplay_api.hpp#L266)
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

- **Declared at:** [line 265](../../../runtime/gameplay_api.hpp#L265)
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

- **Declared at:** [line 262](../../../runtime/gameplay_api.hpp#L262)
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

- **Declared at:** [line 257](../../../runtime/gameplay_api.hpp#L257)
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

- **Declared at:** [line 260](../../../runtime/gameplay_api.hpp#L260)
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

- **Declared at:** [line 255](../../../runtime/gameplay_api.hpp#L255)
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

- **Declared at:** [line 263](../../../runtime/gameplay_api.hpp#L263)
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

- **Declared at:** [line 258](../../../runtime/gameplay_api.hpp#L258)
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

- **Declared at:** [line 259](../../../runtime/gameplay_api.hpp#L259)
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

- **Declared at:** [line 261](../../../runtime/gameplay_api.hpp#L261)
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

- **Declared at:** [line 256](../../../runtime/gameplay_api.hpp#L256)
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

- **Declared at:** [line 251](../../../runtime/gameplay_api.hpp#L251)
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

- **Declared at:** [line 250](../../../runtime/gameplay_api.hpp#L250)
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

- **Declared at:** [line 249](../../../runtime/gameplay_api.hpp#L249)
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

- **Declared at:** [line 243](../../../runtime/gameplay_api.hpp#L243)
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

<a id="epok-scenelibrary-fog-1"></a>

## `epok::SceneLibrary::fog`

**Purpose.** Performs `fog` as part of the gameplay api module.

**Exact declaration**

```cpp
static FogSettings fog()
```

- **Declared at:** [line 219](../../../runtime/gameplay_api.hpp#L219)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `FogSettings`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::SceneLibrary::fog();
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

- **Declared at:** [line 245](../../../runtime/gameplay_api.hpp#L245)
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

- **Declared at:** [line 208](../../../runtime/gameplay_api.hpp#L208)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

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

- **Declared at:** [line 209](../../../runtime/gameplay_api.hpp#L209)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `value` | `GameplayTransitionOptions` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// GameplayTransitionOptions value

auto result = epok::SceneLibrary::request_with_transition(index, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenelibrary-screen-fade-1"></a>

## `epok::SceneLibrary::screen_fade`

**Purpose.** The post-HUD fade to black, 0 clear to 255 opaque, clamped rather than rejected because every amount above the range has one nearest valid value.

**Details.** The getter reports the authored amount; what is drawn is the larger of it and a running transition's opacity, which `transition_snapshot` already reports. The amount survives scene activation, so a game can fade out, request a scene and fade back in.

**Exact declaration**

```cpp
static uint32_t screen_fade()
```

- **Declared at:** [line 236](../../../runtime/gameplay_api.hpp#L236)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** The getter reports the authored amount; what is drawn is the larger of it and a running transition's opacity, which `transition_snapshot` already reports. The amount survives scene activation, so a game can fade out, request a scene and fade back in.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::SceneLibrary::screen_fade();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-set-camera-1"></a>

## `epok::SceneLibrary::set_camera`

**Purpose.** Sets camera as part of the gameplay api module.

**Exact declaration**

```cpp
static bool set_camera(ObjectId actor)
```

- **Declared at:** [line 244](../../../runtime/gameplay_api.hpp#L244)
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

<a id="epok-scenelibrary-set-fog-1"></a>

## `epok::SceneLibrary::set_fog`

**Purpose.** Sets fog as part of the gameplay api module.

**Exact declaration**

```cpp
static bool set_fog(FogSettings value)
```

- **Declared at:** [line 220](../../../runtime/gameplay_api.hpp#L220)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `FogSettings` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// FogSettings value

auto result = epok::SceneLibrary::set_fog(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-scenelibrary-set-screen-fade-1"></a>

## `epok::SceneLibrary::set_screen_fade`

**Purpose.** Sets screen fade as part of the gameplay api module.

**Exact declaration**

```cpp
static void set_screen_fade(uint32_t value)
```

- **Declared at:** [line 237](../../../runtime/gameplay_api.hpp#L237)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::SceneLibrary::set_screen_fade(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelibrary-snapshot-1"></a>

## `epok::SceneLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the gameplay api module.

**Exact declaration**

```cpp
static SceneSnapshot snapshot()
```

- **Declared at:** [line 206](../../../runtime/gameplay_api.hpp#L206)
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

- **Declared at:** [line 207](../../../runtime/gameplay_api.hpp#L207)
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

- **Declared at:** [line 159](../../../runtime/gameplay_api.hpp#L159)
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

- **Declared at:** [line 161](../../../runtime/gameplay_api.hpp#L161)
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

- **Declared at:** [line 152](../../../runtime/gameplay_api.hpp#L152)
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

<a id="epok-utilityvectorlibrary-vector-tween-advance-1"></a>

## `epok::UtilityVectorLibrary::vector_tween_advance`

**Purpose.** Performs `vector tween advance` as part of the gameplay api module.

**Exact declaration**

```cpp
static Vector3TweenAdvanceSample vector_tween_advance(GameplayVector3TweenState state,Fixed delta_seconds)
```

- **Declared at:** [line 180](../../../runtime/gameplay_api.hpp#L180)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayVector3TweenState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `delta_seconds` | `Fixed` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `Vector3TweenAdvanceSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3TweenState state
// Fixed delta_seconds

auto result = epok::UtilityVectorLibrary::vector_tween_advance(state, delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilityvectorlibrary-vector-tween-cancel-1"></a>

## `epok::UtilityVectorLibrary::vector_tween_cancel`

**Purpose.** Performs `vector tween cancel` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3TweenState vector_tween_cancel(GameplayVector3TweenState state)
```

- **Declared at:** [line 181](../../../runtime/gameplay_api.hpp#L181)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayVector3TweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3TweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3TweenState state

auto result = epok::UtilityVectorLibrary::vector_tween_cancel(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilityvectorlibrary-vector-tween-schedule-1"></a>

## `epok::UtilityVectorLibrary::vector_tween_schedule`

**Purpose.** Performs `vector tween schedule` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3TweenState vector_tween_schedule(GameplayVector3 from,GameplayVector3 to,Fixed seconds,Ease easing,Fixed delay_seconds,TweenLoop loop,uint32_t legs)
```

- **Declared at:** [line 179](../../../runtime/gameplay_api.hpp#L179)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `GameplayVector3` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `GameplayVector3` | Input | Value supplied for `to`. See the exact type and module contract. |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `easing` | `Ease` | Input | Value supplied for `easing`. See the exact type and module contract. |
| `delay_seconds` | `Fixed` | Input | Value supplied for `delay_seconds`. See the exact type and module contract. |
| `loop` | `TweenLoop` | Input | Value supplied for `loop`. See the exact type and module contract. |
| `legs` | `uint32_t` | Input | Value supplied for `legs`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3TweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 from
// GameplayVector3 to
// Fixed seconds
// Ease easing
// Fixed delay_seconds
// TweenLoop loop
// uint32_t legs

auto result = epok::UtilityVectorLibrary::vector_tween_schedule(from, to, seconds, easing, delay_seconds, loop, legs);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-utilityvectorlibrary-vector-tween-start-1"></a>

## `epok::UtilityVectorLibrary::vector_tween_start`

**Purpose.** Performs `vector tween start` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3TweenState vector_tween_start(GameplayVector3 from,GameplayVector3 to,Fixed seconds,Ease easing)
```

- **Declared at:** [line 178](../../../runtime/gameplay_api.hpp#L178)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `GameplayVector3` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `GameplayVector3` | Input | Value supplied for `to`. See the exact type and module contract. |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `easing` | `Ease` | Input | Value supplied for `easing`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3TweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 from
// GameplayVector3 to
// Fixed seconds
// Ease easing

auto result = epok::UtilityVectorLibrary::vector_tween_start(from, to, seconds, easing);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilityvectorlibrary-vector-tween-value-1"></a>

## `epok::UtilityVectorLibrary::vector_tween_value`

**Purpose.** Performs `vector tween value` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 vector_tween_value(GameplayVector3TweenState state)
```

- **Declared at:** [line 182](../../../runtime/gameplay_api.hpp#L182)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayVector3TweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3TweenState state

auto result = epok::UtilityVectorLibrary::vector_tween_value(state);
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

- **Declared at:** [line 187](../../../runtime/gameplay_api.hpp#L187)
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

- **Declared at:** [line 186](../../../runtime/gameplay_api.hpp#L186)
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
