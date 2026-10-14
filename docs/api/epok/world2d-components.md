# Epok API: World2D Components

> **Header:** `"world2d_components.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_components.hpp)

This module covers the world2d components module. It documents 82 public callables declared directly in this header.

## Declared types

`epok::Camera2DComponent`, `epok::CharacterBody2DComponent`, `epok::Collider2DComponent`, `epok::Sprite2DComponent`, `epok::TileMapLayerComponent`, `epok::world2d::Link`, `epok::world2d::List`

## Callable index

- [`epok::Camera2DComponent::class_id`](#epok-camera2dcomponent-class-id-1) — Performs `class id` as part of the world2d components module.
- [`epok::Camera2DComponent::clear_limits`](#epok-camera2dcomponent-clear-limits-1) — Clears limits as part of the world2d components module.
- [`epok::Camera2DComponent::follow`](#epok-camera2dcomponent-follow-1) — One simulation step towards `target` (the owner's world position).
- [`epok::Camera2DComponent::get_zoom`](#epok-camera2dcomponent-get-zoom-1) — Returns zoom as part of the world2d components module.
- [`epok::Camera2DComponent::is_current`](#epok-camera2dcomponent-is-current-1) — Reports whether current as part of the world2d components module.
- [`epok::Camera2DComponent::level_attached`](#epok-camera2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d components module.
- [`epok::Camera2DComponent::make_current`](#epok-camera2dcomponent-make-current-1) — Performs `make current` as part of the world2d components module.
- [`epok::Camera2DComponent::reset_smoothing`](#epok-camera2dcomponent-reset-smoothing-1) — Jump to the target this step instead of smoothing towards it.
- [`epok::Camera2DComponent::screen_center_x`](#epok-camera2dcomponent-screen-center-x-1) — Performs `screen center x` as part of the world2d components module.
- [`epok::Camera2DComponent::screen_center_y`](#epok-camera2dcomponent-screen-center-y-1) — Performs `screen center y` as part of the world2d components module.
- [`epok::Camera2DComponent::set_limits`](#epok-camera2dcomponent-set-limits-1) — Sets limits as part of the world2d components module.
- [`epok::Camera2DComponent::set_offset`](#epok-camera2dcomponent-set-offset-1) — Sets offset as part of the world2d components module.
- [`epok::Camera2DComponent::set_zoom`](#epok-camera2dcomponent-set-zoom-1) — Sets zoom as part of the world2d components module.
- [`epok::Camera2DComponent::shake`](#epok-camera2dcomponent-shake-1) — Shakes the view by up to `amplitude` pixels, fading out over `seconds`.
- [`epok::Camera2DComponent::view`](#epok-camera2dcomponent-view-1) — The Camera2D the renderer projects through.
- [`epok::Camera2DComponent::~Camera2DComponent`](#epok-camera2dcomponent-camera2dcomponent-1) — Releases the resources owned by `epok::Camera2DComponent`.
- [`epok::CharacterBody2DComponent::class_id`](#epok-characterbody2dcomponent-class-id-1) — Performs `class id` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::is_on_ceiling`](#epok-characterbody2dcomponent-is-on-ceiling-1) — Reports whether on ceiling as part of the world2d components module.
- [`epok::CharacterBody2DComponent::is_on_floor`](#epok-characterbody2dcomponent-is-on-floor-1) — Reports whether on floor as part of the world2d components module.
- [`epok::CharacterBody2DComponent::is_on_wall`](#epok-characterbody2dcomponent-is-on-wall-1) — Reports whether on wall as part of the world2d components module.
- [`epok::CharacterBody2DComponent::last_collision_actor`](#epok-characterbody2dcomponent-last-collision-actor-1) — Performs `last collision actor` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::last_collision_is_tile`](#epok-characterbody2dcomponent-last-collision-is-tile-1) — Whether the last collision was a tile of a TileMapLayer (last_collision_actor is its actor).
- [`epok::CharacterBody2DComponent::last_motion_x`](#epok-characterbody2dcomponent-last-motion-x-1) — Performs `last motion x` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::last_motion_y`](#epok-characterbody2dcomponent-last-motion-y-1) — Performs `last motion y` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::last_tile_x`](#epok-characterbody2dcomponent-last-tile-x-1) — Cell column / row of that tile.
- [`epok::CharacterBody2DComponent::last_tile_y`](#epok-characterbody2dcomponent-last-tile-y-1) — Performs `last tile y` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::level_attached`](#epok-characterbody2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::move_and_collide`](#epok-characterbody2dcomponent-move-and-collide-1) — Moves by an explicit displacement (pixels) and stops at the first obstacle.
- [`epok::CharacterBody2DComponent::move_and_slide`](#epok-characterbody2dcomponent-move-and-slide-1) — Moves by velocity * step, slides along what it hits, zeroes the velocity components pushing into the obstacle and refreshes the floor/wall/ceiling state.
- [`epok::CharacterBody2DComponent::set_velocity`](#epok-characterbody2dcomponent-set-velocity-1) — Sets velocity as part of the world2d components module.
- [`epok::CharacterBody2DComponent::velocity_x`](#epok-characterbody2dcomponent-velocity-x-1) — Performs `velocity x` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::velocity_y`](#epok-characterbody2dcomponent-velocity-y-1) — Performs `velocity y` as part of the world2d components module.
- [`epok::CharacterBody2DComponent::~CharacterBody2DComponent`](#epok-characterbody2dcomponent-characterbody2dcomponent-1) — Releases the resources owned by `epok::CharacterBody2DComponent`.
- [`epok::Collider2DComponent::class_id`](#epok-collider2dcomponent-class-id-1) — Performs `class id` as part of the world2d components module.
- [`epok::Collider2DComponent::collision_layer`](#epok-collider2dcomponent-collision-layer-1) — Performs `collision layer` as part of the world2d components module.
- [`epok::Collider2DComponent::collision_mask`](#epok-collider2dcomponent-collision-mask-1) — Performs `collision mask` as part of the world2d components module.
- [`epok::Collider2DComponent::is_enabled`](#epok-collider2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d components module.
- [`epok::Collider2DComponent::is_trigger`](#epok-collider2dcomponent-is-trigger-1) — Reports whether trigger as part of the world2d components module.
- [`epok::Collider2DComponent::level_attached`](#epok-collider2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d components module.
- [`epok::Collider2DComponent::set_box_size`](#epok-collider2dcomponent-set-box-size-1) — Full box size in pixels (width, height), centred on the shape offset.
- [`epok::Collider2DComponent::set_circle_radius`](#epok-collider2dcomponent-set-circle-radius-1) — Sets circle radius as part of the world2d components module.
- [`epok::Collider2DComponent::set_collision_layer`](#epok-collider2dcomponent-set-collision-layer-1) — Sets collision layer as part of the world2d components module.
- [`epok::Collider2DComponent::set_collision_mask`](#epok-collider2dcomponent-set-collision-mask-1) — Sets collision mask as part of the world2d components module.
- [`epok::Collider2DComponent::set_enabled`](#epok-collider2dcomponent-set-enabled-1) — Sets enabled as part of the world2d components module.
- [`epok::Collider2DComponent::~Collider2DComponent`](#epok-collider2dcomponent-collider2dcomponent-1) — Releases the resources owned by `epok::Collider2DComponent`.
- [`epok::Sprite2DComponent::advance`](#epok-sprite2dcomponent-advance-1) — Advances the playing clip by one simulation step.
- [`epok::Sprite2DComponent::animation_count`](#epok-sprite2dcomponent-animation-count-1) — Performs `animation count` as part of the world2d components module.
- [`epok::Sprite2DComponent::class_id`](#epok-sprite2dcomponent-class-id-1) — Performs `class id` as part of the world2d components module.
- [`epok::Sprite2DComponent::current_animation`](#epok-sprite2dcomponent-current-animation-1) — Performs `current animation` as part of the world2d components module.
- [`epok::Sprite2DComponent::current_frame`](#epok-sprite2dcomponent-current-frame-1) — Performs `current frame` as part of the world2d components module.
- [`epok::Sprite2DComponent::is_flipped_h`](#epok-sprite2dcomponent-is-flipped-h-1) — Reports whether flipped h as part of the world2d components module.
- [`epok::Sprite2DComponent::is_playing`](#epok-sprite2dcomponent-is-playing-1) — Reports whether playing as part of the world2d components module.
- [`epok::Sprite2DComponent::is_visible`](#epok-sprite2dcomponent-is-visible-1) — Reports whether visible as part of the world2d components module.
- [`epok::Sprite2DComponent::level_attached`](#epok-sprite2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d components module.
- [`epok::Sprite2DComponent::pause`](#epok-sprite2dcomponent-pause-1) — Pauses pause as part of the world2d components module.
- [`epok::Sprite2DComponent::play`](#epok-sprite2dcomponent-play-1) — Starts play as part of the world2d components module.
- [`epok::Sprite2DComponent::play_named`](#epok-sprite2dcomponent-play-named-1) — C++ convenience: play a clip by its authored name.
- [`epok::Sprite2DComponent::set_flip_h`](#epok-sprite2dcomponent-set-flip-h-1) — Sets flip h as part of the world2d components module.
- [`epok::Sprite2DComponent::set_flip_v`](#epok-sprite2dcomponent-set-flip-v-1) — Sets flip v as part of the world2d components module.
- [`epok::Sprite2DComponent::set_frame`](#epok-sprite2dcomponent-set-frame-1) — Sets frame as part of the world2d components module.
- [`epok::Sprite2DComponent::set_modulate`](#epok-sprite2dcomponent-set-modulate-1) — Sets modulate as part of the world2d components module.
- [`epok::Sprite2DComponent::set_speed_scale`](#epok-sprite2dcomponent-set-speed-scale-1) — Sets speed scale as part of the world2d components module.
- [`epok::Sprite2DComponent::set_texture_region`](#epok-sprite2dcomponent-set-texture-region-1) — Sets texture region as part of the world2d components module.
- [`epok::Sprite2DComponent::set_viewport`](#epok-sprite2dcomponent-set-viewport-1) — Shows the picture of the SubViewport2D on `actor` (an empty id: the texture again).
- [`epok::Sprite2DComponent::set_visible`](#epok-sprite2dcomponent-set-visible-1) — Sets visible as part of the world2d components module.
- [`epok::Sprite2DComponent::set_z_index`](#epok-sprite2dcomponent-set-z-index-1) — Sets z index as part of the world2d components module.
- [`epok::Sprite2DComponent::source_rect`](#epok-sprite2dcomponent-source-rect-1) — The texel rectangle drawn this frame, given the texture size.
- [`epok::Sprite2DComponent::stop`](#epok-sprite2dcomponent-stop-1) — Stops stop as part of the world2d components module.
- [`epok::Sprite2DComponent::take_animation_finished`](#epok-sprite2dcomponent-take-animation-finished-1) — True once after a non-looping clip reached its last frame.
- [`epok::Sprite2DComponent::take_frame_event`](#epok-sprite2dcomponent-take-frame-event-1) — The event id of the last frame entered (0 = none), reported once.
- [`epok::Sprite2DComponent::~Sprite2DComponent`](#epok-sprite2dcomponent-sprite2dcomponent-1) — Releases the resources owned by `epok::Sprite2DComponent`.
- [`epok::world2d::find`](#epok-world2d-find-1) — The first component of type T on `actor`, or null.
- [`epok::world2d::Link::Link<T>`](#epok-world2d-link-link-t-1) — Constructs `epok::world2d::Link` for the world2d components module.
- [`epok::world2d::Link::Link<T>`](#epok-world2d-link-link-t-2) — Constructs `epok::world2d::Link` for the world2d components module.
- [`epok::world2d::Link::operator=`](#epok-world2d-link-operator-1) — Performs `operator =` as part of the world2d components module.
- [`epok::world2d::List::add`](#epok-world2d-list-add-1) — Adds add as part of the world2d components module.
- [`epok::world2d::List::each`](#epok-world2d-list-each-1) — Safe against the visited item unlinking itself.
- [`epok::world2d::List::remove`](#epok-world2d-list-remove-1) — Removes remove as part of the world2d components module.
- [`epok::world2d::resolve_scene2d`](#epok-world2d-resolve-scene2d-1) — Performs `resolve scene2d` as part of the world2d components module.
- [`epok::world2d::root_of`](#epok-world2d-root-of-1) — The root SceneComponent2D of a component's owner, or null.
- [`epok::world2d::step_seconds`](#epok-world2d-step-seconds-1) — Performs `step seconds` as part of the world2d components module.
- [`epok::world2d::world_of`](#epok-world2d-world-of-1) — World matrix of the owner's root, following SceneComponent2D attachments.

<a id="epok-camera2dcomponent-class-id-1"></a>

## `epok::Camera2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 252](../../../runtime/world2d_components.hpp#L252)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-clear-limits-1"></a>

## `epok::Camera2DComponent::clear_limits`

**Purpose.** Clears limits as part of the world2d components module.

**Exact declaration**

```cpp
void clear_limits()
```

- **Declared at:** [line 292](../../../runtime/world2d_components.hpp#L292)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.clear_limits();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-follow-1"></a>

## `epok::Camera2DComponent::follow`

**Purpose.** One simulation step towards `target` (the owner's world position).

**Exact declaration**

```cpp
void follow(const Fixed* target, Fixed dt)
```

- **Declared at:** [line 304](../../../runtime/world2d_components.hpp#L304)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `const Fixed *` | Input | Value supplied for `target`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * target
// Fixed dt

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.follow(target, dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-camera2dcomponent-get-zoom-1"></a>

## `epok::Camera2DComponent::get_zoom`

**Purpose.** Returns zoom as part of the world2d components module.

**Exact declaration**

```cpp
Fixed get_zoom() const
```

- **Declared at:** [line 286](../../../runtime/world2d_components.hpp#L286)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.get_zoom();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-is-current-1"></a>

## `epok::Camera2DComponent::is_current`

**Purpose.** Reports whether current as part of the world2d components module.

**Exact declaration**

```cpp
bool is_current() const
```

- **Declared at:** [line 282](../../../runtime/world2d_components.hpp#L282)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.is_current();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-camera2dcomponent-level-attached-1"></a>

## `epok::Camera2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d components module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 253](../../../runtime/world2d_components.hpp#L253)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-make-current-1"></a>

## `epok::Camera2DComponent::make_current`

**Purpose.** Performs `make current` as part of the world2d components module.

**Exact declaration**

```cpp
void make_current()
```

- **Declared at:** [line 279](../../../runtime/world2d_components.hpp#L279)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.make_current();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-reset-smoothing-1"></a>

## `epok::Camera2DComponent::reset_smoothing`

**Purpose.** Jump to the target this step instead of smoothing towards it.

**Exact declaration**

```cpp
void reset_smoothing()
```

- **Declared at:** [line 299](../../../runtime/world2d_components.hpp#L299)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.reset_smoothing();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-screen-center-x-1"></a>

## `epok::Camera2DComponent::screen_center_x`

**Purpose.** Performs `screen center x` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed screen_center_x() const
```

- **Declared at:** [line 300](../../../runtime/world2d_components.hpp#L300)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.screen_center_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-screen-center-y-1"></a>

## `epok::Camera2DComponent::screen_center_y`

**Purpose.** Performs `screen center y` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed screen_center_y() const
```

- **Declared at:** [line 301](../../../runtime/world2d_components.hpp#L301)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.screen_center_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-set-limits-1"></a>

## `epok::Camera2DComponent::set_limits`

**Purpose.** Sets limits as part of the world2d components module.

**Exact declaration**

```cpp
void set_limits(Fixed left, Fixed top, Fixed right, Fixed bottom)
```

- **Declared at:** [line 288](../../../runtime/world2d_components.hpp#L288)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `left` | `Fixed` | Input | Value supplied for `left`. See the exact type and module contract. |
| `top` | `Fixed` | Input | Value supplied for `top`. See the exact type and module contract. |
| `right` | `Fixed` | Input | Value supplied for `right`. See the exact type and module contract. |
| `bottom` | `Fixed` | Input | Value supplied for `bottom`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed left
// Fixed top
// Fixed right
// Fixed bottom

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.set_limits(left, top, right, bottom);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-set-offset-1"></a>

## `epok::Camera2DComponent::set_offset`

**Purpose.** Sets offset as part of the world2d components module.

**Exact declaration**

```cpp
void set_offset(Fixed x, Fixed y)
```

- **Declared at:** [line 287](../../../runtime/world2d_components.hpp#L287)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.set_offset(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-set-zoom-1"></a>

## `epok::Camera2DComponent::set_zoom`

**Purpose.** Sets zoom as part of the world2d components module.

**Exact declaration**

```cpp
void set_zoom(Fixed value)
```

- **Declared at:** [line 283](../../../runtime/world2d_components.hpp#L283)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.set_zoom(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-shake-1"></a>

## `epok::Camera2DComponent::shake`

**Purpose.** Shakes the view by up to `amplitude` pixels, fading out over `seconds`.

**Exact declaration**

```cpp
void shake(Fixed amplitude, Fixed seconds)
```

- **Declared at:** [line 294](../../../runtime/world2d_components.hpp#L294)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `amplitude` | `Fixed` | Input | Value supplied for `amplitude`. See the exact type and module contract. |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed amplitude
// Fixed seconds

epok::Camera2DComponent& object = /* obtain a valid instance */;

object.shake(amplitude, seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-view-1"></a>

## `epok::Camera2DComponent::view`

**Purpose.** The Camera2D the renderer projects through.

**Exact declaration**

```cpp
Camera2D view()
```

- **Declared at:** [line 337](../../../runtime/world2d_components.hpp#L337)
- **Kind:** `cxx method`

**Returns.** Returns `Camera2D`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Camera2DComponent& object = /* obtain a valid instance */;

auto result = object.view();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-camera2dcomponent-camera2dcomponent-1"></a>

## `epok::Camera2DComponent::~Camera2DComponent`

**Purpose.** Releases the resources owned by `epok::Camera2DComponent`.

**Exact declaration**

```cpp
~Camera2DComponent() override
```

- **Declared at:** [line 254](../../../runtime/world2d_components.hpp#L254)
- **Kind:** `destructor`

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// `epok::Camera2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-class-id-1"></a>

## `epok::CharacterBody2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 402](../../../runtime/world2d_components.hpp#L402)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-is-on-ceiling-1"></a>

## `epok::CharacterBody2DComponent::is_on_ceiling`

**Purpose.** Reports whether on ceiling as part of the world2d components module.

**Exact declaration**

```cpp
bool is_on_ceiling() const
```

- **Declared at:** [line 431](../../../runtime/world2d_components.hpp#L431)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_on_ceiling();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-is-on-floor-1"></a>

## `epok::CharacterBody2DComponent::is_on_floor`

**Purpose.** Reports whether on floor as part of the world2d components module.

**Exact declaration**

```cpp
bool is_on_floor() const
```

- **Declared at:** [line 429](../../../runtime/world2d_components.hpp#L429)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_on_floor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-is-on-wall-1"></a>

## `epok::CharacterBody2DComponent::is_on_wall`

**Purpose.** Reports whether on wall as part of the world2d components module.

**Exact declaration**

```cpp
bool is_on_wall() const
```

- **Declared at:** [line 430](../../../runtime/world2d_components.hpp#L430)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_on_wall();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-last-collision-actor-1"></a>

## `epok::CharacterBody2DComponent::last_collision_actor`

**Purpose.** Performs `last collision actor` as part of the world2d components module.

**Exact declaration**

```cpp
ObjectId last_collision_actor() const
```

- **Declared at:** [line 432](../../../runtime/world2d_components.hpp#L432)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_collision_actor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-last-collision-is-tile-1"></a>

## `epok::CharacterBody2DComponent::last_collision_is_tile`

**Purpose.** Whether the last collision was a tile of a TileMapLayer (last_collision_actor is its actor).

**Exact declaration**

```cpp
bool last_collision_is_tile() const
```

- **Declared at:** [line 436](../../../runtime/world2d_components.hpp#L436)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_collision_is_tile();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-last-motion-x-1"></a>

## `epok::CharacterBody2DComponent::last_motion_x`

**Purpose.** Performs `last motion x` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed last_motion_x() const
```

- **Declared at:** [line 433](../../../runtime/world2d_components.hpp#L433)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_motion_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-last-motion-y-1"></a>

## `epok::CharacterBody2DComponent::last_motion_y`

**Purpose.** Performs `last motion y` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed last_motion_y() const
```

- **Declared at:** [line 434](../../../runtime/world2d_components.hpp#L434)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_motion_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-last-tile-x-1"></a>

## `epok::CharacterBody2DComponent::last_tile_x`

**Purpose.** Cell column / row of that tile.

**Exact declaration**

```cpp
int32_t last_tile_x() const
```

- **Declared at:** [line 438](../../../runtime/world2d_components.hpp#L438)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_tile_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-last-tile-y-1"></a>

## `epok::CharacterBody2DComponent::last_tile_y`

**Purpose.** Performs `last tile y` as part of the world2d components module.

**Exact declaration**

```cpp
int32_t last_tile_y() const
```

- **Declared at:** [line 439](../../../runtime/world2d_components.hpp#L439)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.last_tile_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-level-attached-1"></a>

## `epok::CharacterBody2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d components module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 403](../../../runtime/world2d_components.hpp#L403)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-move-and-collide-1"></a>

## `epok::CharacterBody2DComponent::move_and_collide`

**Purpose.** Moves by an explicit displacement (pixels) and stops at the first obstacle.

**Exact declaration**

```cpp
bool move_and_collide(Fixed dx, Fixed dy)
```

- **Declared at:** [line 473](../../../runtime/world2d_components.hpp#L473)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dx` | `Fixed` | Input | Value supplied for `dx`. See the exact type and module contract. |
| `dy` | `Fixed` | Input | Value supplied for `dy`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dx
// Fixed dy

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.move_and_collide(dx, dy);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-move-and-slide-1"></a>

## `epok::CharacterBody2DComponent::move_and_slide`

**Purpose.** Moves by velocity * step, slides along what it hits, zeroes the velocity components pushing into the obstacle and refreshes the floor/wall/ceiling state.

**Details.** Returns whether anything was hit. Call it from tick: it uses the simulation step.

**Exact declaration**

```cpp
bool move_and_slide()
```

- **Declared at:** [line 444](../../../runtime/world2d_components.hpp#L444)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Returns whether anything was hit. Call it from tick: it uses the simulation step.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.move_and_slide();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-characterbody2dcomponent-set-velocity-1"></a>

## `epok::CharacterBody2DComponent::set_velocity`

**Purpose.** Sets velocity as part of the world2d components module.

**Exact declaration**

```cpp
void set_velocity(Fixed x, Fixed y)
```

- **Declared at:** [line 426](../../../runtime/world2d_components.hpp#L426)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

object.set_velocity(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-velocity-x-1"></a>

## `epok::CharacterBody2DComponent::velocity_x`

**Purpose.** Performs `velocity x` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed velocity_x() const
```

- **Declared at:** [line 427](../../../runtime/world2d_components.hpp#L427)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.velocity_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-velocity-y-1"></a>

## `epok::CharacterBody2DComponent::velocity_y`

**Purpose.** Performs `velocity y` as part of the world2d components module.

**Exact declaration**

```cpp
Fixed velocity_y() const
```

- **Declared at:** [line 428](../../../runtime/world2d_components.hpp#L428)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::CharacterBody2DComponent& object = /* obtain a valid instance */;

auto result = object.velocity_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-characterbody2dcomponent-characterbody2dcomponent-1"></a>

## `epok::CharacterBody2DComponent::~CharacterBody2DComponent`

**Purpose.** Releases the resources owned by `epok::CharacterBody2DComponent`.

**Exact declaration**

```cpp
~CharacterBody2DComponent() override
```

- **Declared at:** [line 404](../../../runtime/world2d_components.hpp#L404)
- **Kind:** `destructor`

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// `epok::CharacterBody2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-class-id-1"></a>

## `epok::Collider2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 364](../../../runtime/world2d_components.hpp#L364)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-collision-layer-1"></a>

## `epok::Collider2DComponent::collision_layer`

**Purpose.** Performs `collision layer` as part of the world2d components module.

**Exact declaration**

```cpp
uint32_t collision_layer() const
```

- **Declared at:** [line 378](../../../runtime/world2d_components.hpp#L378)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

auto result = object.collision_layer();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-collision-mask-1"></a>

## `epok::Collider2DComponent::collision_mask`

**Purpose.** Performs `collision mask` as part of the world2d components module.

**Exact declaration**

```cpp
uint32_t collision_mask() const
```

- **Declared at:** [line 379](../../../runtime/world2d_components.hpp#L379)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

auto result = object.collision_mask();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-is-enabled-1"></a>

## `epok::Collider2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d components module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 374](../../../runtime/world2d_components.hpp#L374)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-collider2dcomponent-is-trigger-1"></a>

## `epok::Collider2DComponent::is_trigger`

**Purpose.** Reports whether trigger as part of the world2d components module.

**Exact declaration**

```cpp
bool is_trigger() const
```

- **Declared at:** [line 375](../../../runtime/world2d_components.hpp#L375)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

auto result = object.is_trigger();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-collider2dcomponent-level-attached-1"></a>

## `epok::Collider2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d components module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 365](../../../runtime/world2d_components.hpp#L365)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-set-box-size-1"></a>

## `epok::Collider2DComponent::set_box_size`

**Purpose.** Full box size in pixels (width, height), centred on the shape offset.

**Exact declaration**

```cpp
void set_box_size(Fixed width, Fixed height)
```

- **Declared at:** [line 381](../../../runtime/world2d_components.hpp#L381)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `width` | `Fixed` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `Fixed` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed width
// Fixed height

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.set_box_size(width, height);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-set-circle-radius-1"></a>

## `epok::Collider2DComponent::set_circle_radius`

**Purpose.** Sets circle radius as part of the world2d components module.

**Exact declaration**

```cpp
void set_circle_radius(Fixed radius)
```

- **Declared at:** [line 386](../../../runtime/world2d_components.hpp#L386)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `radius` | `Fixed` | Input | Value supplied for `radius`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed radius

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.set_circle_radius(radius);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-set-collision-layer-1"></a>

## `epok::Collider2DComponent::set_collision_layer`

**Purpose.** Sets collision layer as part of the world2d components module.

**Exact declaration**

```cpp
void set_collision_layer(uint32_t value)
```

- **Declared at:** [line 376](../../../runtime/world2d_components.hpp#L376)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.set_collision_layer(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-set-collision-mask-1"></a>

## `epok::Collider2DComponent::set_collision_mask`

**Purpose.** Sets collision mask as part of the world2d components module.

**Exact declaration**

```cpp
void set_collision_mask(uint32_t value)
```

- **Declared at:** [line 377](../../../runtime/world2d_components.hpp#L377)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.set_collision_mask(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-set-enabled-1"></a>

## `epok::Collider2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d components module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 373](../../../runtime/world2d_components.hpp#L373)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Collider2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-collider2dcomponent-collider2dcomponent-1"></a>

## `epok::Collider2DComponent::~Collider2DComponent`

**Purpose.** Releases the resources owned by `epok::Collider2DComponent`.

**Exact declaration**

```cpp
~Collider2DComponent() override
```

- **Declared at:** [line 366](../../../runtime/world2d_components.hpp#L366)
- **Kind:** `destructor`

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// `epok::Collider2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-advance-1"></a>

## `epok::Sprite2DComponent::advance`

**Purpose.** Advances the playing clip by one simulation step.

**Details.** Called by the World2D service.

**Exact declaration**

```cpp
void advance(Fixed dt)
```

- **Declared at:** [line 200](../../../runtime/world2d_components.hpp#L200)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Called by the World2D service.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-animation-count-1"></a>

## `epok::Sprite2DComponent::animation_count`

**Purpose.** Performs `animation count` as part of the world2d components module.

**Exact declaration**

```cpp
uint32_t animation_count() const
```

- **Declared at:** [line 161](../../../runtime/world2d_components.hpp#L161)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.animation_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-class-id-1"></a>

## `epok::Sprite2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d components module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 125](../../../runtime/world2d_components.hpp#L125)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-current-animation-1"></a>

## `epok::Sprite2DComponent::current_animation`

**Purpose.** Performs `current animation` as part of the world2d components module.

**Exact declaration**

```cpp
uint32_t current_animation() const
```

- **Declared at:** [line 160](../../../runtime/world2d_components.hpp#L160)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.current_animation();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-current-frame-1"></a>

## `epok::Sprite2DComponent::current_frame`

**Purpose.** Performs `current frame` as part of the world2d components module.

**Exact declaration**

```cpp
uint32_t current_frame() const
```

- **Declared at:** [line 162](../../../runtime/world2d_components.hpp#L162)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.current_frame();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-is-flipped-h-1"></a>

## `epok::Sprite2DComponent::is_flipped_h`

**Purpose.** Reports whether flipped h as part of the world2d components module.

**Exact declaration**

```cpp
bool is_flipped_h() const
```

- **Declared at:** [line 175](../../../runtime/world2d_components.hpp#L175)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.is_flipped_h();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite2dcomponent-is-playing-1"></a>

## `epok::Sprite2DComponent::is_playing`

**Purpose.** Reports whether playing as part of the world2d components module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 159](../../../runtime/world2d_components.hpp#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite2dcomponent-is-visible-1"></a>

## `epok::Sprite2DComponent::is_visible`

**Purpose.** Reports whether visible as part of the world2d components module.

**Exact declaration**

```cpp
bool is_visible() const
```

- **Declared at:** [line 177](../../../runtime/world2d_components.hpp#L177)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.is_visible();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite2dcomponent-level-attached-1"></a>

## `epok::Sprite2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d components module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 126](../../../runtime/world2d_components.hpp#L126)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-pause-1"></a>

## `epok::Sprite2DComponent::pause`

**Purpose.** Pauses pause as part of the world2d components module.

**Exact declaration**

```cpp
void pause()
```

- **Declared at:** [line 158](../../../runtime/world2d_components.hpp#L158)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.pause();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-play-1"></a>

## `epok::Sprite2DComponent::play`

**Purpose.** Starts play as part of the world2d components module.

**Exact declaration**

```cpp
void play(uint32_t clip)
```

- **Declared at:** [line 150](../../../runtime/world2d_components.hpp#L150)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `clip` | `uint32_t` | Input | Value supplied for `clip`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t clip

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.play(clip);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-play-named-1"></a>

## `epok::Sprite2DComponent::play_named`

**Purpose.** C++ convenience: play a clip by its authored name.

**Exact declaration**

```cpp
bool play_named(const char* name)
```

- **Declared at:** [line 190](../../../runtime/world2d_components.hpp#L190)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// const char * name

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.play_named(name);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sprite2dcomponent-set-flip-h-1"></a>

## `epok::Sprite2DComponent::set_flip_h`

**Purpose.** Sets flip h as part of the world2d components module.

**Exact declaration**

```cpp
void set_flip_h(bool value)
```

- **Declared at:** [line 173](../../../runtime/world2d_components.hpp#L173)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_flip_h(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-flip-v-1"></a>

## `epok::Sprite2DComponent::set_flip_v`

**Purpose.** Sets flip v as part of the world2d components module.

**Exact declaration**

```cpp
void set_flip_v(bool value)
```

- **Declared at:** [line 174](../../../runtime/world2d_components.hpp#L174)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_flip_v(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-frame-1"></a>

## `epok::Sprite2DComponent::set_frame`

**Purpose.** Sets frame as part of the world2d components module.

**Exact declaration**

```cpp
void set_frame(uint32_t value)
```

- **Declared at:** [line 163](../../../runtime/world2d_components.hpp#L163)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_frame(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-modulate-1"></a>

## `epok::Sprite2DComponent::set_modulate`

**Purpose.** Sets modulate as part of the world2d components module.

**Exact declaration**

```cpp
void set_modulate(uint32_t red, uint32_t green, uint32_t blue)
```

- **Declared at:** [line 178](../../../runtime/world2d_components.hpp#L178)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_modulate(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-speed-scale-1"></a>

## `epok::Sprite2DComponent::set_speed_scale`

**Purpose.** Sets speed scale as part of the world2d components module.

**Exact declaration**

```cpp
void set_speed_scale(Fixed value)
```

- **Declared at:** [line 172](../../../runtime/world2d_components.hpp#L172)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_speed_scale(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-texture-region-1"></a>

## `epok::Sprite2DComponent::set_texture_region`

**Purpose.** Sets texture region as part of the world2d components module.

**Exact declaration**

```cpp
void set_texture_region(uint32_t x, uint32_t y, uint32_t width, uint32_t height)
```

- **Declared at:** [line 186](../../../runtime/world2d_components.hpp#L186)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `uint32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `uint32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `width` | `uint32_t` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `uint32_t` | Input | Value supplied for `height`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t x
// uint32_t y
// uint32_t width
// uint32_t height

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_texture_region(x, y, width, height);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-viewport-1"></a>

## `epok::Sprite2DComponent::set_viewport`

**Purpose.** Shows the picture of the SubViewport2D on `actor` (an empty id: the texture again).

**Exact declaration**

```cpp
void set_viewport(ObjectId actor)
```

- **Declared at:** [line 149](../../../runtime/world2d_components.hpp#L149)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_viewport(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-visible-1"></a>

## `epok::Sprite2DComponent::set_visible`

**Purpose.** Sets visible as part of the world2d components module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 176](../../../runtime/world2d_components.hpp#L176)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-set-z-index-1"></a>

## `epok::Sprite2DComponent::set_z_index`

**Purpose.** Sets z index as part of the world2d components module.

**Exact declaration**

```cpp
void set_z_index(int32_t value)
```

- **Declared at:** [line 183](../../../runtime/world2d_components.hpp#L183)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.set_z_index(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-source-rect-1"></a>

## `epok::Sprite2DComponent::source_rect`

**Purpose.** The texel rectangle drawn this frame, given the texture size.

**Details.** False when empty.

**Exact declaration**

```cpp
bool source_rect(uint16_t texture_width, uint16_t texture_height, uint16_t* out) const
```

- **Declared at:** [line 219](../../../runtime/world2d_components.hpp#L219)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `texture_width` | `uint16_t` | Input | Value supplied for `texture_width`. See the exact type and module contract. |
| `texture_height` | `uint16_t` | Input | Value supplied for `texture_height`. See the exact type and module contract. |
| `out` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** False when empty.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t texture_width
// uint16_t texture_height
// uint16_t * out

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.source_rect(texture_width, texture_height, out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sprite2dcomponent-stop-1"></a>

## `epok::Sprite2DComponent::stop`

**Purpose.** Stops stop as part of the world2d components module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 157](../../../runtime/world2d_components.hpp#L157)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-take-animation-finished-1"></a>

## `epok::Sprite2DComponent::take_animation_finished`

**Purpose.** True once after a non-looping clip reached its last frame.

**Exact declaration**

```cpp
bool take_animation_finished()
```

- **Declared at:** [line 169](../../../runtime/world2d_components.hpp#L169)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.take_animation_finished();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sprite2dcomponent-take-frame-event-1"></a>

## `epok::Sprite2DComponent::take_frame_event`

**Purpose.** The event id of the last frame entered (0 = none), reported once.

**Exact declaration**

```cpp
uint32_t take_frame_event()
```

- **Declared at:** [line 171](../../../runtime/world2d_components.hpp#L171)
- **Kind:** `cxx method`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::Sprite2DComponent& object = /* obtain a valid instance */;

auto result = object.take_frame_event();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sprite2dcomponent-sprite2dcomponent-1"></a>

## `epok::Sprite2DComponent::~Sprite2DComponent`

**Purpose.** Releases the resources owned by `epok::Sprite2DComponent`.

**Exact declaration**

```cpp
~Sprite2DComponent() override
```

- **Declared at:** [line 127](../../../runtime/world2d_components.hpp#L127)
- **Kind:** `destructor`

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// `epok::Sprite2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-find-1"></a>

## `epok::world2d::find`

**Purpose.** The first component of type T on `actor`, or null.

**Details.** C++ convenience for gameplay code: `auto* body = world2d::find<CharacterBody2DComponent>(get_owner());`

**Exact declaration**

```cpp
template<class T> T* find(Actor* actor)
```

- **Declared at:** [line 104](../../../runtime/world2d_components.hpp#L104)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `T *`. Check the purpose and failure notes before using the value.

**Use it when.** C++ convenience for gameplay code: `auto* body = world2d::find<CharacterBody2DComponent>(get_owner());`

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// Actor * actor

epok::world2d& object = /* obtain a valid instance */;

auto result = object.find<T>(actor);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-link-link-t-1"></a>

## `epok::world2d::Link::Link<T>`

**Purpose.** Constructs `epok::world2d::Link` for the world2d components module.

**Exact declaration**

```cpp
Link() = default
```

- **Declared at:** [line 39](../../../runtime/world2d_components.hpp#L39)
- **Kind:** `constructor`

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

epok::world2d::Link value();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-link-link-t-2"></a>

## `epok::world2d::Link::Link<T>`

**Purpose.** Constructs `epok::world2d::Link` for the world2d components module.

**Exact declaration**

```cpp
Link(const Link&)
```

- **Declared at:** [line 40](../../../runtime/world2d_components.hpp#L40)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Link<T> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// const Link<T> & arg1

epok::world2d::Link value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-link-operator-1"></a>

## `epok::world2d::Link::operator=`

**Purpose.** Performs `operator =` as part of the world2d components module.

**Exact declaration**

```cpp
Link& operator=(const Link&)
```

- **Declared at:** [line 41](../../../runtime/world2d_components.hpp#L41)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Link<T> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Link<T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// const Link<T> & arg1

epok::world2d::Link& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-list-add-1"></a>

## `epok::world2d::List::add`

**Purpose.** Adds add as part of the world2d components module.

**Exact declaration**

```cpp
void add(T* item)
```

- **Declared at:** [line 47](../../../runtime/world2d_components.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `item` | `T *` | Input/output; inspect the function contract | Value supplied for `item`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// T * item

epok::world2d::List& object = /* obtain a valid instance */;

object.add(item);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-list-each-1"></a>

## `epok::world2d::List::each`

**Purpose.** Safe against the visited item unlinking itself.

**Exact declaration**

```cpp
template<class F> void each(F&& visit) const
```

- **Declared at:** [line 70](../../../runtime/world2d_components.hpp#L70)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `visit` | `F &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// F

// Assume these named values have been initialized with valid data:
// F && visit

epok::world2d::List& object = /* obtain a valid instance */;

object.each<F>(visit);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-list-remove-1"></a>

## `epok::world2d::List::remove`

**Purpose.** Removes remove as part of the world2d components module.

**Exact declaration**

```cpp
void remove(T* item)
```

- **Declared at:** [line 58](../../../runtime/world2d_components.hpp#L58)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `item` | `T *` | Input/output; inspect the function contract | Value supplied for `item`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// T * item

epok::world2d::List& object = /* obtain a valid instance */;

object.remove(item);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-resolve-scene2d-1"></a>

## `epok::world2d::resolve_scene2d`

**Purpose.** Performs `resolve scene2d` as part of the world2d components module.

**Exact declaration**

```cpp
inline const SceneComponent2D* resolve_scene2d(ObjectId id)
```

- **Declared at:** [line 93](../../../runtime/world2d_components.hpp#L93)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const SceneComponent2D *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::world2d::resolve_scene2d(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-root-of-1"></a>

## `epok::world2d::root_of`

**Purpose.** The root SceneComponent2D of a component's owner, or null.

**Exact declaration**

```cpp
inline SceneComponent2D* root_of(ActorComponent& component)
```

- **Declared at:** [line 88](../../../runtime/world2d_components.hpp#L88)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `SceneComponent2D *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & component

auto result = epok::world2d::root_of(component);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-step-seconds-1"></a>

## `epok::world2d::step_seconds`

**Purpose.** Performs `step seconds` as part of the world2d components module.

**Exact declaration**

```cpp
inline Fixed step_seconds()
```

- **Declared at:** [line 101](../../../runtime/world2d_components.hpp#L101)
- **Kind:** `function decl`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

auto result = epok::world2d::step_seconds();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d components module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-world-of-1"></a>

## `epok::world2d::world_of`

**Purpose.** World matrix of the owner's root, following SceneComponent2D attachments.

**Exact declaration**

```cpp
inline bool world_of(ActorComponent& component, Affine2D& out)
```

- **Declared at:** [line 97](../../../runtime/world2d_components.hpp#L97)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |
| `out` | `Affine2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d components module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_components.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & component
// Affine2D & out

auto result = epok::world2d::world_of(component, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
