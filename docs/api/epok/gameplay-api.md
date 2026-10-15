# Epok API: Gameplay Api

> **Header:** `"gameplay_api.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gameplay_api.hpp)

This module covers the gameplay api module. It documents 122 public callables declared directly in this header.

## Declared types

`epok::CardFileSample`, `epok::CollisionHit2DSample`, `epok::CollisionHitSample`, `epok::CollisionLibrary`, `epok::DungeonLibrary`, `epok::DungeonPoint`, `epok::DungeonRoom`, `epok::FogSettings`, `epok::GameplayAabb`, `epok::GameplayCamera2D`, `epok::GameplayPlaybackSnapshot`, `epok::GameplayTransitionOptions`, `epok::GameplayTransitionSnapshot`, `epok::GameplayVector2`, `epok::GameplayVector3`, `epok::GameplayVector3TweenState`, `epok::InputAxisSample`, `epok::InputLibrary`, `epok::LocalizationLibrary`, `epok::MathLibrary`, `epok::MemoryCardLibrary`, `epok::MemoryCardSnapshot`, `epok::MovementIntent`, `epok::MoveSample`, `epok::PlaybackLibrary`, `epok::ProjectedPoint`, `epok::ResourceLibrary`, `epok::ResourceSnapshot`, `epok::SavePayload8`, `epok::SceneLibrary`, `epok::SceneSnapshot`, `epok::SkeletalQuerySnapshot`, `epok::TimeLibrary`, `epok::TimeSnapshot`, `epok::UtilityVectorLibrary`, `epok::Vector3TweenAdvanceSample`, `epok::World2DLibrary`

## Callable index

- [`epok::CollisionLibrary::ground`](#epok-collisionlibrary-ground-1) — Performs `ground` as part of the gameplay api module.
- [`epok::CollisionLibrary::move`](#epok-collisionlibrary-move-1) — Performs `move` as part of the gameplay api module.
- [`epok::CollisionLibrary::overlap_box`](#epok-collisionlibrary-overlap-box-1) — Performs `overlap box` as part of the gameplay api module.
- [`epok::CollisionLibrary::raycast_segment`](#epok-collisionlibrary-raycast-segment-1) — Performs `raycast segment` as part of the gameplay api module.
- [`epok::DungeonLibrary::distance`](#epok-dungeonlibrary-distance-1) — Rooms between two rooms on a shortest way, or -1 when there is no way.
- [`epok::DungeonLibrary::explore_everything`](#epok-dungeonlibrary-explore-everything-1) — Performs `explore everything` as part of the gameplay api module.
- [`epok::DungeonLibrary::explore_room`](#epok-dungeonlibrary-explore-room-1) — Rooms the player has entered show on the minimap, with the corridors they open into.
- [`epok::DungeonLibrary::forget_explored`](#epok-dungeonlibrary-forget-explored-1) — Performs `forget explored` as part of the gameplay api module.
- [`epok::DungeonLibrary::full_map_shown`](#epok-dungeonlibrary-full-map-shown-1) — Performs `full map shown` as part of the gameplay api module.
- [`epok::DungeonLibrary::goal_room`](#epok-dungeonlibrary-goal-room-1) — Performs `goal room` as part of the gameplay api module.
- [`epok::DungeonLibrary::item`](#epok-dungeonlibrary-item-1) — A key (lock above 0) or other item the recipe placed.
- [`epok::DungeonLibrary::item_count`](#epok-dungeonlibrary-item-count-1) — Performs `item count` as part of the gameplay api module.
- [`epok::DungeonLibrary::lock`](#epok-dungeonlibrary-lock-1) — A locked door: its doorway, the room it guards and the lock its key opens.
- [`epok::DungeonLibrary::lock_count`](#epok-dungeonlibrary-lock-count-1) — Performs `lock count` as part of the gameplay api module.
- [`epok::DungeonLibrary::minimap_shown`](#epok-dungeonlibrary-minimap-shown-1) — Performs `minimap shown` as part of the gameplay api module.
- [`epok::DungeonLibrary::neighbour`](#epok-dungeonlibrary-neighbour-1) — The `index`-th room linked to `room`, or -1.
- [`epok::DungeonLibrary::next_room`](#epok-dungeonlibrary-next-room-1) — The room after `from` on a shortest way to `to`; `from` when they are the same room, -1 when there is no way.
- [`epok::DungeonLibrary::random`](#epok-dungeonlibrary-random-1) — A number below `below` from the dungeon's own random stream: the same for the same stream, index and seed on every run, so a room can always roll the same enemy.
- [`epok::DungeonLibrary::room`](#epok-dungeonlibrary-room-1) — Performs `room` as part of the gameplay api module.
- [`epok::DungeonLibrary::room_at`](#epok-dungeonlibrary-room-at-1) — The room holding a world point (pixels in 2D), or -1 in a corridor or outside.
- [`epok::DungeonLibrary::room_count`](#epok-dungeonlibrary-room-count-1) — Performs `room count` as part of the gameplay api module.
- [`epok::DungeonLibrary::room_explored`](#epok-dungeonlibrary-room-explored-1) — Performs `room explored` as part of the gameplay api module.
- [`epok::DungeonLibrary::room_of`](#epok-dungeonlibrary-room-of-1) — The room a 3D actor stands in, or -1.
- [`epok::DungeonLibrary::seed`](#epok-dungeonlibrary-seed-1) — Performs `seed` as part of the gameplay api module.
- [`epok::DungeonLibrary::show_full_map`](#epok-dungeonlibrary-show-full-map-1) — Performs `show full map` as part of the gameplay api module.
- [`epok::DungeonLibrary::show_minimap`](#epok-dungeonlibrary-show-minimap-1) — fills the screen (for a pause or map screen) over the box.
- [`epok::DungeonLibrary::spawn_room`](#epok-dungeonlibrary-spawn-room-1) — Performs `spawn room` as part of the gameplay api module.
- [`epok::DungeonLibrary::spot`](#epok-dungeonlibrary-spot-1) — A free floor point of a room, away from its doors, keys and items: somewhere to put an enemy or a prop.
- [`epok::gameplay_actor_data`](#epok-gameplay-actor-data-1) — Performs `gameplay actor data` as part of the gameplay api module.
- [`epok::gameplay_actor_id`](#epok-gameplay-actor-id-1) — Performs `gameplay actor id` as part of the gameplay api module.
- [`epok::gameplay_card_slot_name`](#epok-gameplay-card-slot-name-1) — Performs `gameplay card slot name` as part of the gameplay api module.
- [`epok::gameplay_dungeon_point`](#epok-gameplay-dungeon-point-1) — Performs `gameplay dungeon point` as part of the gameplay api module.
- [`epok::gameplay_dungeon_vector`](#epok-gameplay-dungeon-vector-1) — Performs `gameplay dungeon vector` as part of the gameplay api module.
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
- [`epok::LocalizationLibrary::key_count`](#epok-localizationlibrary-key-count-1) — Performs `key count` as part of the gameplay api module.
- [`epok::LocalizationLibrary::language`](#epok-localizationlibrary-language-1) — Performs `language` as part of the gameplay api module.
- [`epok::LocalizationLibrary::language_count`](#epok-localizationlibrary-language-count-1) — Performs `language count` as part of the gameplay api module.
- [`epok::LocalizationLibrary::set_language`](#epok-localizationlibrary-set-language-1) — Sets language as part of the gameplay api module.
- [`epok::MathLibrary::add`](#epok-mathlibrary-add-1) — Adds add as part of the gameplay api module.
- [`epok::MathLibrary::clamp`](#epok-mathlibrary-clamp-1) — Performs `clamp` as part of the gameplay api module.
- [`epok::MathLibrary::cosine_degrees`](#epok-mathlibrary-cosine-degrees-1) — Performs `cosine degrees` as part of the gameplay api module.
- [`epok::MathLibrary::delta_degrees`](#epok-mathlibrary-delta-degrees-1) — Shortest signed turn from one heading to another, in -180..180.
- [`epok::MathLibrary::heading_degrees`](#epok-mathlibrary-heading-degrees-1) — Heading of a planar vector in degrees, measured from +y towards +x, so a stick pushed forward reads 0 and a stick pushed right reads 90.
- [`epok::MathLibrary::length`](#epok-mathlibrary-length-1) — Performs `length` as part of the gameplay api module.
- [`epok::MathLibrary::length2`](#epok-mathlibrary-length2-1) — Performs `length2` as part of the gameplay api module.
- [`epok::MathLibrary::lerp`](#epok-mathlibrary-lerp-1) — Performs `lerp` as part of the gameplay api module.
- [`epok::MathLibrary::move_toward`](#epok-mathlibrary-move-toward-1) — Performs `move toward` as part of the gameplay api module.
- [`epok::MathLibrary::move_toward_degrees`](#epok-mathlibrary-move-toward-degrees-1) — The same approach on a circle: turns the short way and never overshoots.
- [`epok::MathLibrary::scale`](#epok-mathlibrary-scale-1) — Performs `scale` as part of the gameplay api module.
- [`epok::MathLibrary::sine_degrees`](#epok-mathlibrary-sine-degrees-1) — Trigonometry, roots and angle arithmetic in the engine's own Q12 form.
- [`epok::MathLibrary::smoothstep`](#epok-mathlibrary-smoothstep-1) — Performs `smoothstep` as part of the gameplay api module.
- [`epok::MathLibrary::square_root`](#epok-mathlibrary-square-root-1) — Performs `square root` as part of the gameplay api module.
- [`epok::MathLibrary::stick_dead_zone`](#epok-mathlibrary-stick-dead-zone-1) — The engine's standard analog dead zone, as a share of full deflection.
- [`epok::MathLibrary::stick_intent`](#epok-mathlibrary-stick-intent-1) — Performs `stick intent` as part of the gameplay api module.
- [`epok::MathLibrary::subtract`](#epok-mathlibrary-subtract-1) — Performs `subtract` as part of the gameplay api module.
- [`epok::MathLibrary::vector3`](#epok-mathlibrary-vector3-1) — Performs `vector3` as part of the gameplay api module.
- [`epok::MathLibrary::wrap_degrees`](#epok-mathlibrary-wrap-degrees-1) — 0 <= result < 360.
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
- [`epok::World2DLibrary::overlap_box_2d`](#epok-world2dlibrary-overlap-box-2d-1) — Actors whose Collider2D bounds overlap the box centred on `center` with `half_extents`.
- [`epok::World2DLibrary::raycast_2d`](#epok-world2dlibrary-raycast-2d-1) — Nearest Collider2D along origin + displacement (pixels).
- [`epok::World2DLibrary::screen_to_world`](#epok-world2dlibrary-screen-to-world-1) — Performs `screen to world` as part of the gameplay api module.
- [`epok::World2DLibrary::world_to_screen`](#epok-world2dlibrary-world-to-screen-1) — Performs `world to screen` as part of the gameplay api module.

<a id="epok-collisionlibrary-ground-1"></a>

## `epok::CollisionLibrary::ground`

**Purpose.** Performs `ground` as part of the gameplay api module.

**Exact declaration**

```cpp
static CollisionHitSample ground(ObjectId actor,Fixed distance,uint32_t mask)
```

- **Declared at:** [line 317](../../../runtime/gameplay_api.hpp#L317)
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

- **Declared at:** [line 320](../../../runtime/gameplay_api.hpp#L320)
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

- **Declared at:** [line 314](../../../runtime/gameplay_api.hpp#L314)
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

- **Declared at:** [line 311](../../../runtime/gameplay_api.hpp#L311)
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

<a id="epok-dungeonlibrary-distance-1"></a>

## `epok::DungeonLibrary::distance`

**Purpose.** Rooms between two rooms on a shortest way, or -1 when there is no way.

**Exact declaration**

```cpp
static int32_t distance(int32_t from,int32_t to)
```

- **Declared at:** [line 435](../../../runtime/gameplay_api.hpp#L435)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int32_t` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t from
// int32_t to

auto result = epok::DungeonLibrary::distance(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-explore-everything-1"></a>

## `epok::DungeonLibrary::explore_everything`

**Purpose.** Performs `explore everything` as part of the gameplay api module.

**Exact declaration**

```cpp
static void explore_everything()
```

- **Declared at:** [line 459](../../../runtime/gameplay_api.hpp#L459)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

epok::DungeonLibrary::explore_everything();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-explore-room-1"></a>

## `epok::DungeonLibrary::explore_room`

**Purpose.** Rooms the player has entered show on the minimap, with the corridors they open into.

**Details.** Exploring needs the generator's minimap; a room can also be revealed from game code, for a map item or a save.

**Exact declaration**

```cpp
static void explore_room(int32_t room)
```

- **Declared at:** [line 457](../../../runtime/gameplay_api.hpp#L457)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Exploring needs the generator's minimap; a room can also be revealed from game code, for a map item or a save.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room

epok::DungeonLibrary::explore_room(room);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-forget-explored-1"></a>

## `epok::DungeonLibrary::forget_explored`

**Purpose.** Performs `forget explored` as part of the gameplay api module.

**Exact declaration**

```cpp
static void forget_explored()
```

- **Declared at:** [line 460](../../../runtime/gameplay_api.hpp#L460)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

epok::DungeonLibrary::forget_explored();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-full-map-shown-1"></a>

## `epok::DungeonLibrary::full_map_shown`

**Purpose.** Performs `full map shown` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool full_map_shown()
```

- **Declared at:** [line 453](../../../runtime/gameplay_api.hpp#L453)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::full_map_shown();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeonlibrary-goal-room-1"></a>

## `epok::DungeonLibrary::goal_room`

**Purpose.** Performs `goal room` as part of the gameplay api module.

**Exact declaration**

```cpp
static int32_t goal_room()
```

- **Declared at:** [line 421](../../../runtime/gameplay_api.hpp#L421)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::goal_room();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-item-1"></a>

## `epok::DungeonLibrary::item`

**Purpose.** A key (lock above 0) or other item the recipe placed.

**Exact declaration**

```cpp
static DungeonPoint item(uint32_t index)
```

- **Declared at:** [line 438](../../../runtime/gameplay_api.hpp#L438)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `DungeonPoint`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

auto result = epok::DungeonLibrary::item(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-item-count-1"></a>

## `epok::DungeonLibrary::item_count`

**Purpose.** Performs `item count` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t item_count()
```

- **Declared at:** [line 436](../../../runtime/gameplay_api.hpp#L436)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::item_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-lock-1"></a>

## `epok::DungeonLibrary::lock`

**Purpose.** A locked door: its doorway, the room it guards and the lock its key opens.

**Exact declaration**

```cpp
static DungeonPoint lock(uint32_t index)
```

- **Declared at:** [line 441](../../../runtime/gameplay_api.hpp#L441)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `DungeonPoint`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

auto result = epok::DungeonLibrary::lock(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-lock-count-1"></a>

## `epok::DungeonLibrary::lock_count`

**Purpose.** Performs `lock count` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t lock_count()
```

- **Declared at:** [line 439](../../../runtime/gameplay_api.hpp#L439)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::lock_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-minimap-shown-1"></a>

## `epok::DungeonLibrary::minimap_shown`

**Purpose.** Performs `minimap shown` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool minimap_shown()
```

- **Declared at:** [line 451](../../../runtime/gameplay_api.hpp#L451)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::minimap_shown();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeonlibrary-neighbour-1"></a>

## `epok::DungeonLibrary::neighbour`

**Purpose.** The `index`-th room linked to `room`, or -1.

**Exact declaration**

```cpp
static int32_t neighbour(int32_t room,uint32_t index)
```

- **Declared at:** [line 430](../../../runtime/gameplay_api.hpp#L430)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room
// uint32_t index

auto result = epok::DungeonLibrary::neighbour(room, index);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-next-room-1"></a>

## `epok::DungeonLibrary::next_room`

**Purpose.** The room after `from` on a shortest way to `to`; `from` when they are the same room, -1 when there is no way.

**Exact declaration**

```cpp
static int32_t next_room(int32_t from,int32_t to)
```

- **Declared at:** [line 433](../../../runtime/gameplay_api.hpp#L433)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int32_t` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t from
// int32_t to

auto result = epok::DungeonLibrary::next_room(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-random-1"></a>

## `epok::DungeonLibrary::random`

**Purpose.** A number below `below` from the dungeon's own random stream: the same for the same stream, index and seed on every run, so a room can always roll the same enemy.

**Exact declaration**

```cpp
static uint32_t random(uint32_t stream,uint32_t index,uint32_t below)
```

- **Declared at:** [line 448](../../../runtime/gameplay_api.hpp#L448)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `stream` | `uint32_t` | Input | Value supplied for `stream`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `below` | `uint32_t` | Input | Value supplied for `below`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t stream
// uint32_t index
// uint32_t below

auto result = epok::DungeonLibrary::random(stream, index, below);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-room-1"></a>

## `epok::DungeonLibrary::room`

**Purpose.** Performs `room` as part of the gameplay api module.

**Exact declaration**

```cpp
static DungeonRoom room(int32_t index)
```

- **Declared at:** [line 422](../../../runtime/gameplay_api.hpp#L422)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `DungeonRoom`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t index

auto result = epok::DungeonLibrary::room(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-room-at-1"></a>

## `epok::DungeonLibrary::room_at`

**Purpose.** The room holding a world point (pixels in 2D), or -1 in a corridor or outside.

**Exact declaration**

```cpp
static int32_t room_at(GameplayVector3 point)
```

- **Declared at:** [line 417](../../../runtime/gameplay_api.hpp#L417)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `GameplayVector3` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 point

auto result = epok::DungeonLibrary::room_at(point);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-room-count-1"></a>

## `epok::DungeonLibrary::room_count`

**Purpose.** Performs `room count` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t room_count()
```

- **Declared at:** [line 415](../../../runtime/gameplay_api.hpp#L415)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::room_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-room-explored-1"></a>

## `epok::DungeonLibrary::room_explored`

**Purpose.** Performs `room explored` as part of the gameplay api module.

**Exact declaration**

```cpp
static bool room_explored(int32_t room)
```

- **Declared at:** [line 458](../../../runtime/gameplay_api.hpp#L458)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room

auto result = epok::DungeonLibrary::room_explored(room);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeonlibrary-room-of-1"></a>

## `epok::DungeonLibrary::room_of`

**Purpose.** The room a 3D actor stands in, or -1.

**Details.** In 2D pass the actor's position to room_at.

**Exact declaration**

```cpp
static int32_t room_of(ObjectId actor)
```

- **Declared at:** [line 419](../../../runtime/gameplay_api.hpp#L419)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** In 2D pass the actor's position to room_at.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::DungeonLibrary::room_of(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-seed-1"></a>

## `epok::DungeonLibrary::seed`

**Purpose.** Performs `seed` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t seed()
```

- **Declared at:** [line 445](../../../runtime/gameplay_api.hpp#L445)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::seed();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-show-full-map-1"></a>

## `epok::DungeonLibrary::show_full_map`

**Purpose.** Performs `show full map` as part of the gameplay api module.

**Exact declaration**

```cpp
static void show_full_map(bool full)
```

- **Declared at:** [line 452](../../../runtime/gameplay_api.hpp#L452)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `full` | `bool` | Input | Value supplied for `full`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// bool full

epok::DungeonLibrary::show_full_map(full);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-show-minimap-1"></a>

## `epok::DungeonLibrary::show_minimap`

**Purpose.** fills the screen (for a pause or map screen) over the box.

**Exact declaration**

```cpp
static void show_minimap(bool visible)
```

- **Declared at:** [line 450](../../../runtime/gameplay_api.hpp#L450)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `visible` | `bool` | Input | Value supplied for `visible`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// bool visible

epok::DungeonLibrary::show_minimap(visible);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-spawn-room-1"></a>

## `epok::DungeonLibrary::spawn_room`

**Purpose.** Performs `spawn room` as part of the gameplay api module.

**Exact declaration**

```cpp
static int32_t spawn_room()
```

- **Declared at:** [line 420](../../../runtime/gameplay_api.hpp#L420)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::DungeonLibrary::spawn_room();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeonlibrary-spot-1"></a>

## `epok::DungeonLibrary::spot`

**Purpose.** A free floor point of a room, away from its doors, keys and items: somewhere to put an enemy or a prop.

**Details.** A room has `spots` of them.

**Exact declaration**

```cpp
static DungeonPoint spot(int32_t room,uint32_t index)
```

- **Declared at:** [line 444](../../../runtime/gameplay_api.hpp#L444)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `DungeonPoint`. Check the purpose and failure notes before using the value.

**Use it when.** A room has `spots` of them.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room
// uint32_t index

auto result = epok::DungeonLibrary::spot(room, index);
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

- **Declared at:** [line 149](../../../runtime/gameplay_api.hpp#L149)
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

- **Declared at:** [line 150](../../../runtime/gameplay_api.hpp#L150)
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

- **Declared at:** [line 151](../../../runtime/gameplay_api.hpp#L151)
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

<a id="epok-gameplay-dungeon-point-1"></a>

## `epok::gameplay_dungeon_point`

**Purpose.** Performs `gameplay dungeon point` as part of the gameplay api module.

**Exact declaration**

```cpp
inline DungeonPoint gameplay_dungeon_point(const dungeon::PointData* point)
```

- **Declared at:** [line 405](../../../runtime/gameplay_api.hpp#L405)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `const dungeon::PointData *` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `DungeonPoint`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// const dungeon::PointData * point

auto result = epok::gameplay_dungeon_point(point);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-dungeon-vector-1"></a>

## `epok::gameplay_dungeon_vector`

**Purpose.** Performs `gameplay dungeon vector` as part of the gameplay api module.

**Exact declaration**

```cpp
inline GameplayVector3 gameplay_dungeon_vector(const int32_t* raw)
```

- **Declared at:** [line 404](../../../runtime/gameplay_api.hpp#L404)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw` | `const int32_t *` | Input | Value supplied for `raw`. See the exact type and module contract. |

**Returns.** Returns `GameplayVector3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * raw

auto result = epok::gameplay_dungeon_vector(raw);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-i16-1"></a>

## `epok::gameplay_i16`

**Purpose.** Performs `gameplay i16` as part of the gameplay api module.

**Exact declaration**

```cpp
inline int16_t gameplay_i16(int32_t value)
```

- **Declared at:** [line 152](../../../runtime/gameplay_api.hpp#L152)
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

- **Declared at:** [line 153](../../../runtime/gameplay_api.hpp#L153)
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

- **Declared at:** [line 154](../../../runtime/gameplay_api.hpp#L154)
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

- **Declared at:** [line 155](../../../runtime/gameplay_api.hpp#L155)
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

- **Declared at:** [line 147](../../../runtime/gameplay_api.hpp#L147)
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

- **Declared at:** [line 148](../../../runtime/gameplay_api.hpp#L148)
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

- **Declared at:** [line 156](../../../runtime/gameplay_api.hpp#L156)
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

- **Declared at:** [line 157](../../../runtime/gameplay_api.hpp#L157)
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

- **Declared at:** [line 163](../../../runtime/gameplay_api.hpp#L163)
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

- **Declared at:** [line 175](../../../runtime/gameplay_api.hpp#L175)
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

- **Declared at:** [line 161](../../../runtime/gameplay_api.hpp#L161)
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

- **Declared at:** [line 171](../../../runtime/gameplay_api.hpp#L171)
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

- **Declared at:** [line 173](../../../runtime/gameplay_api.hpp#L173)
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

- **Declared at:** [line 165](../../../runtime/gameplay_api.hpp#L165)
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

- **Declared at:** [line 167](../../../runtime/gameplay_api.hpp#L167)
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

- **Declared at:** [line 169](../../../runtime/gameplay_api.hpp#L169)
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

<a id="epok-localizationlibrary-key-count-1"></a>

## `epok::LocalizationLibrary::key_count`

**Purpose.** Performs `key count` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t key_count()
```

- **Declared at:** [line 214](../../../runtime/gameplay_api.hpp#L214)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::LocalizationLibrary::key_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localizationlibrary-language-1"></a>

## `epok::LocalizationLibrary::language`

**Purpose.** Performs `language` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t language()
```

- **Declared at:** [line 210](../../../runtime/gameplay_api.hpp#L210)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::LocalizationLibrary::language();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localizationlibrary-language-count-1"></a>

## `epok::LocalizationLibrary::language_count`

**Purpose.** Performs `language count` as part of the gameplay api module.

**Exact declaration**

```cpp
static uint32_t language_count()
```

- **Declared at:** [line 212](../../../runtime/gameplay_api.hpp#L212)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::LocalizationLibrary::language_count();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-localizationlibrary-set-language-1"></a>

## `epok::LocalizationLibrary::set_language`

**Purpose.** Sets language as part of the gameplay api module.

**Exact declaration**

```cpp
static void set_language(uint32_t index)
```

- **Declared at:** [line 202](../../../runtime/gameplay_api.hpp#L202)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

epok::LocalizationLibrary::set_language(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-add-1"></a>

## `epok::MathLibrary::add`

**Purpose.** Adds add as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 add(GameplayVector3 a,GameplayVector3 b)
```

- **Declared at:** [line 222](../../../runtime/gameplay_api.hpp#L222)
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

- **Declared at:** [line 218](../../../runtime/gameplay_api.hpp#L218)
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

<a id="epok-mathlibrary-cosine-degrees-1"></a>

## `epok::MathLibrary::cosine_degrees`

**Purpose.** Performs `cosine degrees` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed cosine_degrees(Fixed degrees)
```

- **Declared at:** [line 229](../../../runtime/gameplay_api.hpp#L229)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::MathLibrary::cosine_degrees(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-delta-degrees-1"></a>

## `epok::MathLibrary::delta_degrees`

**Purpose.** Shortest signed turn from one heading to another, in -180..180.

**Exact declaration**

```cpp
static Fixed delta_degrees(Fixed from,Fixed to)
```

- **Declared at:** [line 236](../../../runtime/gameplay_api.hpp#L236)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `Fixed` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `Fixed` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed from
// Fixed to

auto result = epok::MathLibrary::delta_degrees(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-heading-degrees-1"></a>

## `epok::MathLibrary::heading_degrees`

**Purpose.** Heading of a planar vector in degrees, measured from +y towards +x, so a stick pushed forward reads 0 and a stick pushed right reads 90.

**Exact declaration**

```cpp
static Fixed heading_degrees(Fixed x,Fixed y)
```

- **Declared at:** [line 242](../../../runtime/gameplay_api.hpp#L242)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::MathLibrary::heading_degrees(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-length-1"></a>

## `epok::MathLibrary::length`

**Purpose.** Performs `length` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed length(GameplayVector3 value)
```

- **Declared at:** [line 232](../../../runtime/gameplay_api.hpp#L232)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayVector3` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector3 value

auto result = epok::MathLibrary::length(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-length2-1"></a>

## `epok::MathLibrary::length2`

**Purpose.** Performs `length2` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed length2(Fixed x,Fixed y)
```

- **Declared at:** [line 231](../../../runtime/gameplay_api.hpp#L231)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::MathLibrary::length2(x, y);
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

- **Declared at:** [line 219](../../../runtime/gameplay_api.hpp#L219)
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

<a id="epok-mathlibrary-move-toward-1"></a>

## `epok::MathLibrary::move_toward`

**Purpose.** Performs `move toward` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed move_toward(Fixed from,Fixed to,Fixed max_step)
```

- **Declared at:** [line 237](../../../runtime/gameplay_api.hpp#L237)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `Fixed` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `Fixed` | Input | Value supplied for `to`. See the exact type and module contract. |
| `max_step` | `Fixed` | Input | Value supplied for `max_step`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed from
// Fixed to
// Fixed max_step

auto result = epok::MathLibrary::move_toward(from, to, max_step);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-move-toward-degrees-1"></a>

## `epok::MathLibrary::move_toward_degrees`

**Purpose.** The same approach on a circle: turns the short way and never overshoots.

**Exact declaration**

```cpp
static Fixed move_toward_degrees(Fixed from,Fixed to,Fixed max_step)
```

- **Declared at:** [line 239](../../../runtime/gameplay_api.hpp#L239)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `Fixed` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `Fixed` | Input | Value supplied for `to`. See the exact type and module contract. |
| `max_step` | `Fixed` | Input | Value supplied for `max_step`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed from
// Fixed to
// Fixed max_step

auto result = epok::MathLibrary::move_toward_degrees(from, to, max_step);
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

- **Declared at:** [line 223](../../../runtime/gameplay_api.hpp#L223)
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

<a id="epok-mathlibrary-sine-degrees-1"></a>

## `epok::MathLibrary::sine_degrees`

**Purpose.** Trigonometry, roots and angle arithmetic in the engine's own Q12 form.

**Details.** Every one of these is integer-only, so a script reaches the same bits the renderer and the collision solver already agree on, on host and on target.

**Exact declaration**

```cpp
static Fixed sine_degrees(Fixed degrees)
```

- **Declared at:** [line 228](../../../runtime/gameplay_api.hpp#L228)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** Every one of these is integer-only, so a script reaches the same bits the renderer and the collision solver already agree on, on host and on target.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::MathLibrary::sine_degrees(degrees);
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

- **Declared at:** [line 220](../../../runtime/gameplay_api.hpp#L220)
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

<a id="epok-mathlibrary-square-root-1"></a>

## `epok::MathLibrary::square_root`

**Purpose.** Performs `square root` as part of the gameplay api module.

**Exact declaration**

```cpp
static Fixed square_root(Fixed value)
```

- **Declared at:** [line 230](../../../runtime/gameplay_api.hpp#L230)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

auto result = epok::MathLibrary::square_root(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-stick-dead-zone-1"></a>

## `epok::MathLibrary::stick_dead_zone`

**Purpose.** The engine's standard analog dead zone, as a share of full deflection.

**Details.** It is fixed rather than a parameter so every language reaches the same raw threshold without depending on how a literal is rounded.

**Exact declaration**

```cpp
static Fixed stick_dead_zone()
```

- **Declared at:** [line 257](../../../runtime/gameplay_api.hpp#L257)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** It is fixed rather than a parameter so every language reaches the same raw threshold without depending on how a literal is rounded.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

auto result = epok::MathLibrary::stick_dead_zone();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-stick-intent-1"></a>

## `epok::MathLibrary::stick_intent`

**Purpose.** Performs `stick intent` as part of the gameplay api module.

**Exact declaration**

```cpp
static MovementIntent stick_intent(Fixed x,Fixed y)
```

- **Declared at:** [line 258](../../../runtime/gameplay_api.hpp#L258)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `MovementIntent`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::MathLibrary::stick_intent(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-mathlibrary-subtract-1"></a>

## `epok::MathLibrary::subtract`

**Purpose.** Performs `subtract` as part of the gameplay api module.

**Exact declaration**

```cpp
static GameplayVector3 subtract(GameplayVector3 a,GameplayVector3 b)
```

- **Declared at:** [line 224](../../../runtime/gameplay_api.hpp#L224)
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

auto result = epok::MathLibrary::subtract(a, b);
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

- **Declared at:** [line 221](../../../runtime/gameplay_api.hpp#L221)
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

<a id="epok-mathlibrary-wrap-degrees-1"></a>

## `epok::MathLibrary::wrap_degrees`

**Purpose.** 0 <= result < 360.

**Details.** A turn is exact in Q12, so wrapping never drifts.

**Exact declaration**

```cpp
static Fixed wrap_degrees(Fixed degrees)
```

- **Declared at:** [line 234](../../../runtime/gameplay_api.hpp#L234)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** A turn is exact in Q12, so wrapping never drifts.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::MathLibrary::wrap_degrees(degrees);
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

- **Declared at:** [line 397](../../../runtime/gameplay_api.hpp#L397)
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

- **Declared at:** [line 396](../../../runtime/gameplay_api.hpp#L396)
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

- **Declared at:** [line 392](../../../runtime/gameplay_api.hpp#L392)
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

- **Declared at:** [line 401](../../../runtime/gameplay_api.hpp#L401)
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

- **Declared at:** [line 395](../../../runtime/gameplay_api.hpp#L395)
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

- **Declared at:** [line 391](../../../runtime/gameplay_api.hpp#L391)
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

- **Declared at:** [line 393](../../../runtime/gameplay_api.hpp#L393)
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

- **Declared at:** [line 398](../../../runtime/gameplay_api.hpp#L398)
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

- **Declared at:** [line 390](../../../runtime/gameplay_api.hpp#L390)
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

- **Declared at:** [line 399](../../../runtime/gameplay_api.hpp#L399)
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

- **Declared at:** [line 394](../../../runtime/gameplay_api.hpp#L394)
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

- **Declared at:** [line 400](../../../runtime/gameplay_api.hpp#L400)
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

- **Declared at:** [line 384](../../../runtime/gameplay_api.hpp#L384)
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

- **Declared at:** [line 386](../../../runtime/gameplay_api.hpp#L386)
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

- **Declared at:** [line 385](../../../runtime/gameplay_api.hpp#L385)
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

- **Declared at:** [line 382](../../../runtime/gameplay_api.hpp#L382)
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

- **Declared at:** [line 377](../../../runtime/gameplay_api.hpp#L377)
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

- **Declared at:** [line 380](../../../runtime/gameplay_api.hpp#L380)
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

- **Declared at:** [line 375](../../../runtime/gameplay_api.hpp#L375)
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

- **Declared at:** [line 383](../../../runtime/gameplay_api.hpp#L383)
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

- **Declared at:** [line 378](../../../runtime/gameplay_api.hpp#L378)
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

- **Declared at:** [line 379](../../../runtime/gameplay_api.hpp#L379)
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

- **Declared at:** [line 381](../../../runtime/gameplay_api.hpp#L381)
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

- **Declared at:** [line 376](../../../runtime/gameplay_api.hpp#L376)
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

- **Declared at:** [line 371](../../../runtime/gameplay_api.hpp#L371)
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

- **Declared at:** [line 370](../../../runtime/gameplay_api.hpp#L370)
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

- **Declared at:** [line 369](../../../runtime/gameplay_api.hpp#L369)
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

- **Declared at:** [line 363](../../../runtime/gameplay_api.hpp#L363)
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

- **Declared at:** [line 339](../../../runtime/gameplay_api.hpp#L339)
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

- **Declared at:** [line 365](../../../runtime/gameplay_api.hpp#L365)
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

- **Declared at:** [line 328](../../../runtime/gameplay_api.hpp#L328)
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

- **Declared at:** [line 329](../../../runtime/gameplay_api.hpp#L329)
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

- **Declared at:** [line 356](../../../runtime/gameplay_api.hpp#L356)
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

- **Declared at:** [line 364](../../../runtime/gameplay_api.hpp#L364)
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

- **Declared at:** [line 340](../../../runtime/gameplay_api.hpp#L340)
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

- **Declared at:** [line 357](../../../runtime/gameplay_api.hpp#L357)
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

- **Declared at:** [line 326](../../../runtime/gameplay_api.hpp#L326)
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

- **Declared at:** [line 327](../../../runtime/gameplay_api.hpp#L327)
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

- **Declared at:** [line 194](../../../runtime/gameplay_api.hpp#L194)
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

- **Declared at:** [line 196](../../../runtime/gameplay_api.hpp#L196)
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

- **Declared at:** [line 187](../../../runtime/gameplay_api.hpp#L187)
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

- **Declared at:** [line 282](../../../runtime/gameplay_api.hpp#L282)
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

- **Declared at:** [line 283](../../../runtime/gameplay_api.hpp#L283)
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

- **Declared at:** [line 281](../../../runtime/gameplay_api.hpp#L281)
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

- **Declared at:** [line 280](../../../runtime/gameplay_api.hpp#L280)
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

- **Declared at:** [line 284](../../../runtime/gameplay_api.hpp#L284)
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

<a id="epok-world2dlibrary-overlap-box-2d-1"></a>

## `epok::World2DLibrary::overlap_box_2d`

**Purpose.** Actors whose Collider2D bounds overlap the box centred on `center` with `half_extents`.

**Exact declaration**

```cpp
static ObjectBatch8 overlap_box_2d(GameplayVector2 center,GameplayVector2 half_extents,uint32_t mask,bool triggers)
```

- **Declared at:** [line 299](../../../runtime/gameplay_api.hpp#L299)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `center` | `GameplayVector2` | Input | Value supplied for `center`. See the exact type and module contract. |
| `half_extents` | `GameplayVector2` | Input | Value supplied for `half_extents`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `ObjectBatch8`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay api module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector2 center
// GameplayVector2 half_extents
// uint32_t mask
// bool triggers

auto result = epok::World2DLibrary::overlap_box_2d(center, half_extents, mask, triggers);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay api module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2dlibrary-raycast-2d-1"></a>

## `epok::World2DLibrary::raycast_2d`

**Purpose.** Nearest Collider2D along origin + displacement (pixels).

**Details.** Triggers are ignored unless `triggers` is set; one-way platforms only stop a ray travelling down onto their top.

**Exact declaration**

```cpp
static CollisionHit2DSample raycast_2d(GameplayVector2 origin,GameplayVector2 displacement,uint32_t mask,bool triggers)
```

- **Declared at:** [line 290](../../../runtime/gameplay_api.hpp#L290)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `origin` | `GameplayVector2` | Input | Value supplied for `origin`. See the exact type and module contract. |
| `displacement` | `GameplayVector2` | Input | Value supplied for `displacement`. See the exact type and module contract. |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `triggers` | `bool` | Input | Value supplied for `triggers`. See the exact type and module contract. |

**Returns.** Returns `CollisionHit2DSample`. Check the purpose and failure notes before using the value.

**Use it when.** Triggers are ignored unless `triggers` is set; one-way platforms only stop a ray travelling down onto their top.

**Usage pattern**

```cpp
#include "gameplay_api.hpp"

// Assume these named values have been initialized with valid data:
// GameplayVector2 origin
// GameplayVector2 displacement
// uint32_t mask
// bool triggers

auto result = epok::World2DLibrary::raycast_2d(origin, displacement, mask, triggers);
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

- **Declared at:** [line 307](../../../runtime/gameplay_api.hpp#L307)
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

- **Declared at:** [line 306](../../../runtime/gameplay_api.hpp#L306)
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
