# Epok API: World2D Content

> **Header:** `"world2d_content.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_content.hpp)

This module covers the world2d content module. It documents 79 public callables declared directly in this header.

## Declared types

`epok::AudioPlayer2DComponent`, `epok::EmissionShape2D`, `epok::Line2DComponent`, `epok::Particles2DComponent`, `epok::Path2DComponent`, `epok::PathFollow2DComponent`, `epok::Polygon2DComponent`, `epok::RemoteTransform2DComponent`, `epok::VisibleOnScreenNotifier2DComponent`

## Callable index

- [`epok::AudioPlayer2DComponent::begin_play`](#epok-audioplayer2dcomponent-begin-play-1) — Begins play as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::class_id`](#epok-audioplayer2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::end_play`](#epok-audioplayer2dcomponent-end-play-1) — Ends play as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::is_playing`](#epok-audioplayer2dcomponent-is-playing-1) — Reports whether playing as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::level_attached`](#epok-audioplayer2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::on_disable`](#epok-audioplayer2dcomponent-on-disable-1) — Performs `on disable` as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::play`](#epok-audioplayer2dcomponent-play-1) — Starts play as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::releasable`](#epok-audioplayer2dcomponent-releasable-1) — Performs `releasable` as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::set_clip`](#epok-audioplayer2dcomponent-set-clip-1) — Sets clip as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::set_pitch`](#epok-audioplayer2dcomponent-set-pitch-1) — Sets pitch as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::set_volume`](#epok-audioplayer2dcomponent-set-volume-1) — Sets volume as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::spatialize`](#epok-audioplayer2dcomponent-spatialize-1) — Loudness and pan for a listener at `listener` (world pixels).
- [`epok::AudioPlayer2DComponent::stop`](#epok-audioplayer2dcomponent-stop-1) — Stops stop as part of the world2d content module.
- [`epok::AudioPlayer2DComponent::~AudioPlayer2DComponent`](#epok-audioplayer2dcomponent-audioplayer2dcomponent-1) — Releases the resources owned by `epok::AudioPlayer2DComponent`.
- [`epok::Line2DComponent::add_point`](#epok-line2dcomponent-add-point-1) — Adds point as part of the world2d content module.
- [`epok::Line2DComponent::class_id`](#epok-line2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::Line2DComponent::clear_points`](#epok-line2dcomponent-clear-points-1) — Clears points as part of the world2d content module.
- [`epok::Line2DComponent::follow_nodes`](#epok-line2dcomponent-follow-nodes-1) — Moves the points onto the followed nodes.
- [`epok::Line2DComponent::get_point_count`](#epok-line2dcomponent-get-point-count-1) — Returns point count as part of the world2d content module.
- [`epok::Line2DComponent::get_point_x`](#epok-line2dcomponent-get-point-x-1) — Returns point x as part of the world2d content module.
- [`epok::Line2DComponent::get_point_y`](#epok-line2dcomponent-get-point-y-1) — Returns point y as part of the world2d content module.
- [`epok::Line2DComponent::level_attached`](#epok-line2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::Line2DComponent::remove_point`](#epok-line2dcomponent-remove-point-1) — Removes point as part of the world2d content module.
- [`epok::Line2DComponent::set_colors`](#epok-line2dcomponent-set-colors-1) — Sets colors as part of the world2d content module.
- [`epok::Line2DComponent::set_point`](#epok-line2dcomponent-set-point-1) — Sets point as part of the world2d content module.
- [`epok::Line2DComponent::set_visible`](#epok-line2dcomponent-set-visible-1) — Sets visible as part of the world2d content module.
- [`epok::Line2DComponent::set_width`](#epok-line2dcomponent-set-width-1) — Sets width as part of the world2d content module.
- [`epok::Line2DComponent::stop_following`](#epok-line2dcomponent-stop-following-1) — Stops following nodes: the points stay where they are and are the line's own again.
- [`epok::Line2DComponent::~Line2DComponent`](#epok-line2dcomponent-line2dcomponent-1) — Releases the resources owned by `epok::Line2DComponent`.
- [`epok::Particles2DComponent::class_id`](#epok-particles2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::Particles2DComponent::get_alive_count`](#epok-particles2dcomponent-get-alive-count-1) — Returns alive count as part of the world2d content module.
- [`epok::Particles2DComponent::is_emitting`](#epok-particles2dcomponent-is-emitting-1) — Reports whether emitting as part of the world2d content module.
- [`epok::Particles2DComponent::level_attached`](#epok-particles2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::Particles2DComponent::restart`](#epok-particles2dcomponent-restart-1) — Starts a one-shot burst again (or restarts a stream).
- [`epok::Particles2DComponent::set_emitting`](#epok-particles2dcomponent-set-emitting-1) — Sets emitting as part of the world2d content module.
- [`epok::Particles2DComponent::set_visible`](#epok-particles2dcomponent-set-visible-1) — Sets visible as part of the world2d content module.
- [`epok::Particles2DComponent::take_finished`](#epok-particles2dcomponent-take-finished-1) — True once after a one-shot burst's last particle died.
- [`epok::Particles2DComponent::~Particles2DComponent`](#epok-particles2dcomponent-particles2dcomponent-1) — Releases the resources owned by `epok::Particles2DComponent`.
- [`epok::Path2DComponent::class_id`](#epok-path2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::Path2DComponent::get_length`](#epok-path2dcomponent-get-length-1) — Returns length as part of the world2d content module.
- [`epok::Path2DComponent::level_attached`](#epok-path2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::Path2DComponent::sample_local`](#epok-path2dcomponent-sample-local-1) — The local point and unit direction at `distance` along the path, clamped (or wrapped, for a closed path) to its length.
- [`epok::Path2DComponent::sample_x`](#epok-path2dcomponent-sample-x-1) — World position at `distance` along the path.
- [`epok::Path2DComponent::sample_y`](#epok-path2dcomponent-sample-y-1) — Performs `sample y` as part of the world2d content module.
- [`epok::Path2DComponent::~Path2DComponent`](#epok-path2dcomponent-path2dcomponent-1) — Releases the resources owned by `epok::Path2DComponent`.
- [`epok::PathFollow2DComponent::advance`](#epok-pathfollow2dcomponent-advance-1) — One simulation step: advance, then place the actor.
- [`epok::PathFollow2DComponent::class_id`](#epok-pathfollow2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::PathFollow2DComponent::get_progress`](#epok-pathfollow2dcomponent-get-progress-1) — Returns progress as part of the world2d content module.
- [`epok::PathFollow2DComponent::get_progress_ratio`](#epok-pathfollow2dcomponent-get-progress-ratio-1) — Returns progress ratio as part of the world2d content module.
- [`epok::PathFollow2DComponent::get_speed`](#epok-pathfollow2dcomponent-get-speed-1) — Returns speed as part of the world2d content module.
- [`epok::PathFollow2DComponent::is_moving`](#epok-pathfollow2dcomponent-is-moving-1) — Reports whether moving as part of the world2d content module.
- [`epok::PathFollow2DComponent::level_attached`](#epok-pathfollow2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::PathFollow2DComponent::resolve_path`](#epok-pathfollow2dcomponent-resolve-path-1) — The Path2D this follower uses.
- [`epok::PathFollow2DComponent::set_moving`](#epok-pathfollow2dcomponent-set-moving-1) — Sets moving as part of the world2d content module.
- [`epok::PathFollow2DComponent::set_progress`](#epok-pathfollow2dcomponent-set-progress-1) — Sets progress as part of the world2d content module.
- [`epok::PathFollow2DComponent::set_progress_ratio`](#epok-pathfollow2dcomponent-set-progress-ratio-1) — Progress as a fraction (0..1) of the path's length.
- [`epok::PathFollow2DComponent::set_speed`](#epok-pathfollow2dcomponent-set-speed-1) — Sets speed as part of the world2d content module.
- [`epok::PathFollow2DComponent::~PathFollow2DComponent`](#epok-pathfollow2dcomponent-pathfollow2dcomponent-1) — Releases the resources owned by `epok::PathFollow2DComponent`.
- [`epok::Polygon2DComponent::class_id`](#epok-polygon2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::Polygon2DComponent::level_attached`](#epok-polygon2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::Polygon2DComponent::set_color`](#epok-polygon2dcomponent-set-color-1) — Sets color as part of the world2d content module.
- [`epok::Polygon2DComponent::set_visible`](#epok-polygon2dcomponent-set-visible-1) — Sets visible as part of the world2d content module.
- [`epok::Polygon2DComponent::~Polygon2DComponent`](#epok-polygon2dcomponent-polygon2dcomponent-1) — Releases the resources owned by `epok::Polygon2DComponent`.
- [`epok::RemoteTransform2DComponent::class_id`](#epok-remotetransform2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::RemoteTransform2DComponent::get_target`](#epok-remotetransform2dcomponent-get-target-1) — Returns target as part of the world2d content module.
- [`epok::RemoteTransform2DComponent::level_attached`](#epok-remotetransform2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::RemoteTransform2DComponent::push`](#epok-remotetransform2dcomponent-push-1) — Pushes the transform once.
- [`epok::RemoteTransform2DComponent::set_target`](#epok-remotetransform2dcomponent-set-target-1) — Sets target as part of the world2d content module.
- [`epok::RemoteTransform2DComponent::~RemoteTransform2DComponent`](#epok-remotetransform2dcomponent-remotetransform2dcomponent-1) — Releases the resources owned by `epok::RemoteTransform2DComponent`.
- [`epok::VisibleOnScreenNotifier2DComponent::class_id`](#epok-visibleonscreennotifier2dcomponent-class-id-1) — Performs `class id` as part of the world2d content module.
- [`epok::VisibleOnScreenNotifier2DComponent::is_on_screen`](#epok-visibleonscreennotifier2dcomponent-is-on-screen-1) — Reports whether on screen as part of the world2d content module.
- [`epok::VisibleOnScreenNotifier2DComponent::level_attached`](#epok-visibleonscreennotifier2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d content module.
- [`epok::VisibleOnScreenNotifier2DComponent::take_screen_entered`](#epok-visibleonscreennotifier2dcomponent-take-screen-entered-1) — Performs `take screen entered` as part of the world2d content module.
- [`epok::VisibleOnScreenNotifier2DComponent::take_screen_exited`](#epok-visibleonscreennotifier2dcomponent-take-screen-exited-1) — Performs `take screen exited` as part of the world2d content module.
- [`epok::VisibleOnScreenNotifier2DComponent::~VisibleOnScreenNotifier2DComponent`](#epok-visibleonscreennotifier2dcomponent-visibleonscreennotifier2dcomponent-1) — Releases the resources owned by `epok::VisibleOnScreenNotifier2DComponent`.
- [`epok::world2d::angle_degrees`](#epok-world2d-angle-degrees-1) — Angle of a vector in Q12 degrees, from +X towards +Y (clockwise on screen), in (-180, 180].
- [`epok::world2d::isqrt64`](#epok-world2d-isqrt64-1) — Integer square root of a non-negative 64-bit value.
- [`epok::world2d::length_2d`](#epok-world2d-length-2d-1) — Length of a Q12 vector, in Q12.
- [`epok::world2d::set_world_position`](#epok-world2d-set-world-position-1) — Moves the owner's root so its world position becomes `world`, through the inverse of the parent chain's linear part when the root is attached.

<a id="epok-audioplayer2dcomponent-begin-play-1"></a>

## `epok::AudioPlayer2DComponent::begin_play`

**Purpose.** Begins play as part of the world2d content module.

**Exact declaration**

```cpp
void begin_play() override
```

- **Declared at:** [line 463](../../../runtime/world2d_content.hpp#L463)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.begin_play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-class-id-1"></a>

## `epok::AudioPlayer2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 443](../../../runtime/world2d_content.hpp#L443)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-end-play-1"></a>

## `epok::AudioPlayer2DComponent::end_play`

**Purpose.** Ends play as part of the world2d content module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 467](../../../runtime/world2d_content.hpp#L467)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-is-playing-1"></a>

## `epok::AudioPlayer2DComponent::is_playing`

**Purpose.** Reports whether playing as part of the world2d content module.

**Exact declaration**

```cpp
bool is_playing() const
```

- **Declared at:** [line 457](../../../runtime/world2d_content.hpp#L457)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

auto result = object.is_playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-level-attached-1"></a>

## `epok::AudioPlayer2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 444](../../../runtime/world2d_content.hpp#L444)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-on-disable-1"></a>

## `epok::AudioPlayer2DComponent::on_disable`

**Purpose.** Performs `on disable` as part of the world2d content module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 466](../../../runtime/world2d_content.hpp#L466)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-play-1"></a>

## `epok::AudioPlayer2DComponent::play`

**Purpose.** Starts play as part of the world2d content module.

**Exact declaration**

```cpp
void play()
```

- **Declared at:** [line 455](../../../runtime/world2d_content.hpp#L455)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.play();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-releasable-1"></a>

## `epok::AudioPlayer2DComponent::releasable`

**Purpose.** Performs `releasable` as part of the world2d content module.

**Exact declaration**

```cpp
bool releasable() const override
```

- **Declared at:** [line 468](../../../runtime/world2d_content.hpp#L468)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

auto result = object.releasable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-set-clip-1"></a>

## `epok::AudioPlayer2DComponent::set_clip`

**Purpose.** Sets clip as part of the world2d content module.

**Exact declaration**

```cpp
void set_clip(int32_t value)
```

- **Declared at:** [line 460](../../../runtime/world2d_content.hpp#L460)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.set_clip(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-set-pitch-1"></a>

## `epok::AudioPlayer2DComponent::set_pitch`

**Purpose.** Sets pitch as part of the world2d content module.

**Exact declaration**

```cpp
void set_pitch(Fixed value)
```

- **Declared at:** [line 459](../../../runtime/world2d_content.hpp#L459)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.set_pitch(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-set-volume-1"></a>

## `epok::AudioPlayer2DComponent::set_volume`

**Purpose.** Sets volume as part of the world2d content module.

**Exact declaration**

```cpp
void set_volume(Fixed value)
```

- **Declared at:** [line 458](../../../runtime/world2d_content.hpp#L458)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.set_volume(value);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-spatialize-1"></a>

## `epok::AudioPlayer2DComponent::spatialize`

**Purpose.** Loudness and pan for a listener at `listener` (world pixels).

**Exact declaration**

```cpp
void spatialize(const Fixed* here, const Fixed* listener, Fixed half_width)
```

- **Declared at:** [line 470](../../../runtime/world2d_content.hpp#L470)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `here` | `const Fixed *` | Input | Value supplied for `here`. See the exact type and module contract. |
| `listener` | `const Fixed *` | Input | Value supplied for `listener`. See the exact type and module contract. |
| `half_width` | `Fixed` | Input | Value supplied for `half_width`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * here
// const Fixed * listener
// Fixed half_width

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.spatialize(here, listener, half_width);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-audioplayer2dcomponent-stop-1"></a>

## `epok::AudioPlayer2DComponent::stop`

**Purpose.** Stops stop as part of the world2d content module.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 456](../../../runtime/world2d_content.hpp#L456)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::AudioPlayer2DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audioplayer2dcomponent-audioplayer2dcomponent-1"></a>

## `epok::AudioPlayer2DComponent::~AudioPlayer2DComponent`

**Purpose.** Releases the resources owned by `epok::AudioPlayer2DComponent`.

**Exact declaration**

```cpp
~AudioPlayer2DComponent() override
```

- **Declared at:** [line 445](../../../runtime/world2d_content.hpp#L445)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::AudioPlayer2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-line2dcomponent-add-point-1"></a>

## `epok::Line2DComponent::add_point`

**Purpose.** Adds point as part of the world2d content module.

**Exact declaration**

```cpp
bool add_point(Fixed x, Fixed y)
```

- **Declared at:** [line 287](../../../runtime/world2d_content.hpp#L287)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Line2DComponent& object = /* obtain a valid instance */;

auto result = object.add_point(x, y);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-line2dcomponent-class-id-1"></a>

## `epok::Line2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 269](../../../runtime/world2d_content.hpp#L269)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-clear-points-1"></a>

## `epok::Line2DComponent::clear_points`

**Purpose.** Clears points as part of the world2d content module.

**Exact declaration**

```cpp
void clear_points()
```

- **Declared at:** [line 304](../../../runtime/world2d_content.hpp#L304)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

object.clear_points();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-follow-nodes-1"></a>

## `epok::Line2DComponent::follow_nodes`

**Purpose.** Moves the points onto the followed nodes.

**Details.** Called by the service every step.

**Exact declaration**

```cpp
void follow_nodes()
```

- **Declared at:** [line 308](../../../runtime/world2d_content.hpp#L308)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Called by the service every step.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

object.follow_nodes();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-get-point-count-1"></a>

## `epok::Line2DComponent::get_point_count`

**Purpose.** Returns point count as part of the world2d content module.

**Exact declaration**

```cpp
uint32_t get_point_count() const
```

- **Declared at:** [line 286](../../../runtime/world2d_content.hpp#L286)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

auto result = object.get_point_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-get-point-x-1"></a>

## `epok::Line2DComponent::get_point_x`

**Purpose.** Returns point x as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_point_x(uint32_t index) const
```

- **Declared at:** [line 297](../../../runtime/world2d_content.hpp#L297)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

epok::Line2DComponent& object = /* obtain a valid instance */;

auto result = object.get_point_x(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-get-point-y-1"></a>

## `epok::Line2DComponent::get_point_y`

**Purpose.** Returns point y as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_point_y(uint32_t index) const
```

- **Declared at:** [line 298](../../../runtime/world2d_content.hpp#L298)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

epok::Line2DComponent& object = /* obtain a valid instance */;

auto result = object.get_point_y(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-level-attached-1"></a>

## `epok::Line2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 270](../../../runtime/world2d_content.hpp#L270)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-remove-point-1"></a>

## `epok::Line2DComponent::remove_point`

**Purpose.** Removes point as part of the world2d content module.

**Exact declaration**

```cpp
void remove_point(uint32_t index)
```

- **Declared at:** [line 299](../../../runtime/world2d_content.hpp#L299)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

epok::Line2DComponent& object = /* obtain a valid instance */;

object.remove_point(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-set-colors-1"></a>

## `epok::Line2DComponent::set_colors`

**Purpose.** Sets colors as part of the world2d content module.

**Exact declaration**

```cpp
void set_colors(uint32_t red, uint32_t green, uint32_t blue, uint32_t end_red, uint32_t end_green, uint32_t end_blue)
```

- **Declared at:** [line 331](../../../runtime/world2d_content.hpp#L331)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |
| `end_red` | `uint32_t` | Input | Value supplied for `end_red`. See the exact type and module contract. |
| `end_green` | `uint32_t` | Input | Value supplied for `end_green`. See the exact type and module contract. |
| `end_blue` | `uint32_t` | Input | Value supplied for `end_blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue
// uint32_t end_red
// uint32_t end_green
// uint32_t end_blue

epok::Line2DComponent& object = /* obtain a valid instance */;

object.set_colors(red, green, blue, end_red, end_green, end_blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-set-point-1"></a>

## `epok::Line2DComponent::set_point`

**Purpose.** Sets point as part of the world2d content module.

**Exact declaration**

```cpp
void set_point(uint32_t index, Fixed x, Fixed y)
```

- **Declared at:** [line 294](../../../runtime/world2d_content.hpp#L294)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index
// Fixed x
// Fixed y

epok::Line2DComponent& object = /* obtain a valid instance */;

object.set_point(index, x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-set-visible-1"></a>

## `epok::Line2DComponent::set_visible`

**Purpose.** Sets visible as part of the world2d content module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 338](../../../runtime/world2d_content.hpp#L338)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Line2DComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-set-width-1"></a>

## `epok::Line2DComponent::set_width`

**Purpose.** Sets width as part of the world2d content module.

**Exact declaration**

```cpp
void set_width(Fixed value)
```

- **Declared at:** [line 330](../../../runtime/world2d_content.hpp#L330)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Line2DComponent& object = /* obtain a valid instance */;

object.set_width(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-stop-following-1"></a>

## `epok::Line2DComponent::stop_following`

**Purpose.** Stops following nodes: the points stay where they are and are the line's own again.

**Exact declaration**

```cpp
void stop_following()
```

- **Declared at:** [line 306](../../../runtime/world2d_content.hpp#L306)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Line2DComponent& object = /* obtain a valid instance */;

object.stop_following();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-line2dcomponent-line2dcomponent-1"></a>

## `epok::Line2DComponent::~Line2DComponent`

**Purpose.** Releases the resources owned by `epok::Line2DComponent`.

**Exact declaration**

```cpp
~Line2DComponent() override
```

- **Declared at:** [line 271](../../../runtime/world2d_content.hpp#L271)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::Line2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-class-id-1"></a>

## `epok::Particles2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 382](../../../runtime/world2d_content.hpp#L382)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-get-alive-count-1"></a>

## `epok::Particles2DComponent::get_alive_count`

**Purpose.** Returns alive count as part of the world2d content module.

**Exact declaration**

```cpp
uint32_t get_alive_count() const
```

- **Declared at:** [line 425](../../../runtime/world2d_content.hpp#L425)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

auto result = object.get_alive_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-is-emitting-1"></a>

## `epok::Particles2DComponent::is_emitting`

**Purpose.** Reports whether emitting as part of the world2d content module.

**Exact declaration**

```cpp
bool is_emitting() const
```

- **Declared at:** [line 417](../../../runtime/world2d_content.hpp#L417)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

auto result = object.is_emitting();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-particles2dcomponent-level-attached-1"></a>

## `epok::Particles2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 383](../../../runtime/world2d_content.hpp#L383)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-restart-1"></a>

## `epok::Particles2DComponent::restart`

**Purpose.** Starts a one-shot burst again (or restarts a stream).

**Exact declaration**

```cpp
void restart()
```

- **Declared at:** [line 419](../../../runtime/world2d_content.hpp#L419)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

object.restart();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-set-emitting-1"></a>

## `epok::Particles2DComponent::set_emitting`

**Purpose.** Sets emitting as part of the world2d content module.

**Exact declaration**

```cpp
void set_emitting(bool value)
```

- **Declared at:** [line 413](../../../runtime/world2d_content.hpp#L413)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Particles2DComponent& object = /* obtain a valid instance */;

object.set_emitting(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-set-visible-1"></a>

## `epok::Particles2DComponent::set_visible`

**Purpose.** Sets visible as part of the world2d content module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 432](../../../runtime/world2d_content.hpp#L432)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Particles2DComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-particles2dcomponent-take-finished-1"></a>

## `epok::Particles2DComponent::take_finished`

**Purpose.** True once after a one-shot burst's last particle died.

**Exact declaration**

```cpp
bool take_finished()
```

- **Declared at:** [line 427](../../../runtime/world2d_content.hpp#L427)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Particles2DComponent& object = /* obtain a valid instance */;

auto result = object.take_finished();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-particles2dcomponent-particles2dcomponent-1"></a>

## `epok::Particles2DComponent::~Particles2DComponent`

**Purpose.** Releases the resources owned by `epok::Particles2DComponent`.

**Exact declaration**

```cpp
~Particles2DComponent() override
```

- **Declared at:** [line 384](../../../runtime/world2d_content.hpp#L384)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::Particles2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-class-id-1"></a>

## `epok::Path2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 101](../../../runtime/world2d_content.hpp#L101)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Path2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-get-length-1"></a>

## `epok::Path2DComponent::get_length`

**Purpose.** Returns length as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_length() const
```

- **Declared at:** [line 111](../../../runtime/world2d_content.hpp#L111)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Path2DComponent& object = /* obtain a valid instance */;

auto result = object.get_length();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-level-attached-1"></a>

## `epok::Path2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 102](../../../runtime/world2d_content.hpp#L102)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Path2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-sample-local-1"></a>

## `epok::Path2DComponent::sample_local`

**Purpose.** The local point and unit direction at `distance` along the path, clamped (or wrapped, for a closed path) to its length.

**Exact declaration**

```cpp
bool sample_local(Fixed distance, Fixed* point, Fixed* direction) const
```

- **Declared at:** [line 116](../../../runtime/world2d_content.hpp#L116)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |
| `point` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `point`. See the exact type and module contract. |
| `direction` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `direction`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed distance
// Fixed * point
// Fixed * direction

epok::Path2DComponent& object = /* obtain a valid instance */;

auto result = object.sample_local(distance, point, direction);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-path2dcomponent-sample-x-1"></a>

## `epok::Path2DComponent::sample_x`

**Purpose.** World position at `distance` along the path.

**Exact declaration**

```cpp
Fixed sample_x(Fixed distance) const
```

- **Declared at:** [line 154](../../../runtime/world2d_content.hpp#L154)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed distance

epok::Path2DComponent& object = /* obtain a valid instance */;

auto result = object.sample_x(distance);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-sample-y-1"></a>

## `epok::Path2DComponent::sample_y`

**Purpose.** Performs `sample y` as part of the world2d content module.

**Exact declaration**

```cpp
Fixed sample_y(Fixed distance) const
```

- **Declared at:** [line 155](../../../runtime/world2d_content.hpp#L155)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `distance` | `Fixed` | Input | Value supplied for `distance`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed distance

epok::Path2DComponent& object = /* obtain a valid instance */;

auto result = object.sample_y(distance);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-path2dcomponent-path2dcomponent-1"></a>

## `epok::Path2DComponent::~Path2DComponent`

**Purpose.** Releases the resources owned by `epok::Path2DComponent`.

**Exact declaration**

```cpp
~Path2DComponent() override
```

- **Declared at:** [line 103](../../../runtime/world2d_content.hpp#L103)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::Path2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-advance-1"></a>

## `epok::PathFollow2DComponent::advance`

**Purpose.** One simulation step: advance, then place the actor.

**Details.** Writes the world displacement of the actor into `moved`. False when there is no path.

**Exact declaration**

```cpp
bool advance(Fixed dt, Fixed* moved)
```

- **Declared at:** [line 219](../../../runtime/world2d_content.hpp#L219)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `moved` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `moved`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Writes the world displacement of the actor into `moved`. False when there is no path.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// Fixed * moved

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.advance(dt, moved);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-pathfollow2dcomponent-class-id-1"></a>

## `epok::PathFollow2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 176](../../../runtime/world2d_content.hpp#L176)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-get-progress-1"></a>

## `epok::PathFollow2DComponent::get_progress`

**Purpose.** Returns progress as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_progress() const
```

- **Declared at:** [line 190](../../../runtime/world2d_content.hpp#L190)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.get_progress();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-get-progress-ratio-1"></a>

## `epok::PathFollow2DComponent::get_progress_ratio`

**Purpose.** Returns progress ratio as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_progress_ratio() const
```

- **Declared at:** [line 195](../../../runtime/world2d_content.hpp#L195)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.get_progress_ratio();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-get-speed-1"></a>

## `epok::PathFollow2DComponent::get_speed`

**Purpose.** Returns speed as part of the world2d content module.

**Exact declaration**

```cpp
Fixed get_speed() const
```

- **Declared at:** [line 201](../../../runtime/world2d_content.hpp#L201)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.get_speed();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-is-moving-1"></a>

## `epok::PathFollow2DComponent::is_moving`

**Purpose.** Reports whether moving as part of the world2d content module.

**Exact declaration**

```cpp
bool is_moving() const
```

- **Declared at:** [line 203](../../../runtime/world2d_content.hpp#L203)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.is_moving();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-pathfollow2dcomponent-level-attached-1"></a>

## `epok::PathFollow2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 177](../../../runtime/world2d_content.hpp#L177)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-resolve-path-1"></a>

## `epok::PathFollow2DComponent::resolve_path`

**Purpose.** The Path2D this follower uses.

**Exact declaration**

```cpp
Path2DComponent* resolve_path() const
```

- **Declared at:** [line 206](../../../runtime/world2d_content.hpp#L206)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Path2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

auto result = object.resolve_path();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-set-moving-1"></a>

## `epok::PathFollow2DComponent::set_moving`

**Purpose.** Sets moving as part of the world2d content module.

**Exact declaration**

```cpp
void set_moving(bool value)
```

- **Declared at:** [line 202](../../../runtime/world2d_content.hpp#L202)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

object.set_moving(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-set-progress-1"></a>

## `epok::PathFollow2DComponent::set_progress`

**Purpose.** Sets progress as part of the world2d content module.

**Exact declaration**

```cpp
void set_progress(Fixed value)
```

- **Declared at:** [line 189](../../../runtime/world2d_content.hpp#L189)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

object.set_progress(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-set-progress-ratio-1"></a>

## `epok::PathFollow2DComponent::set_progress_ratio`

**Purpose.** Progress as a fraction (0..1) of the path's length.

**Exact declaration**

```cpp
void set_progress_ratio(Fixed value)
```

- **Declared at:** [line 192](../../../runtime/world2d_content.hpp#L192)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

object.set_progress_ratio(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-set-speed-1"></a>

## `epok::PathFollow2DComponent::set_speed`

**Purpose.** Sets speed as part of the world2d content module.

**Exact declaration**

```cpp
void set_speed(Fixed value)
```

- **Declared at:** [line 200](../../../runtime/world2d_content.hpp#L200)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::PathFollow2DComponent& object = /* obtain a valid instance */;

object.set_speed(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-pathfollow2dcomponent-pathfollow2dcomponent-1"></a>

## `epok::PathFollow2DComponent::~PathFollow2DComponent`

**Purpose.** Releases the resources owned by `epok::PathFollow2DComponent`.

**Exact declaration**

```cpp
~PathFollow2DComponent() override
```

- **Declared at:** [line 178](../../../runtime/world2d_content.hpp#L178)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::PathFollow2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-polygon2dcomponent-class-id-1"></a>

## `epok::Polygon2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 348](../../../runtime/world2d_content.hpp#L348)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Polygon2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-polygon2dcomponent-level-attached-1"></a>

## `epok::Polygon2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 349](../../../runtime/world2d_content.hpp#L349)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::Polygon2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-polygon2dcomponent-set-color-1"></a>

## `epok::Polygon2DComponent::set_color`

**Purpose.** Sets color as part of the world2d content module.

**Exact declaration**

```cpp
void set_color(uint32_t red, uint32_t green, uint32_t blue)
```

- **Declared at:** [line 363](../../../runtime/world2d_content.hpp#L363)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::Polygon2DComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-polygon2dcomponent-set-visible-1"></a>

## `epok::Polygon2DComponent::set_visible`

**Purpose.** Sets visible as part of the world2d content module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 368](../../../runtime/world2d_content.hpp#L368)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Polygon2DComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-polygon2dcomponent-polygon2dcomponent-1"></a>

## `epok::Polygon2DComponent::~Polygon2DComponent`

**Purpose.** Releases the resources owned by `epok::Polygon2DComponent`.

**Exact declaration**

```cpp
~Polygon2DComponent() override
```

- **Declared at:** [line 350](../../../runtime/world2d_content.hpp#L350)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::Polygon2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-class-id-1"></a>

## `epok::RemoteTransform2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 519](../../../runtime/world2d_content.hpp#L519)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::RemoteTransform2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-get-target-1"></a>

## `epok::RemoteTransform2DComponent::get_target`

**Purpose.** Returns target as part of the world2d content module.

**Exact declaration**

```cpp
ObjectId get_target() const
```

- **Declared at:** [line 528](../../../runtime/world2d_content.hpp#L528)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::RemoteTransform2DComponent& object = /* obtain a valid instance */;

auto result = object.get_target();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-level-attached-1"></a>

## `epok::RemoteTransform2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 520](../../../runtime/world2d_content.hpp#L520)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::RemoteTransform2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-push-1"></a>

## `epok::RemoteTransform2DComponent::push`

**Purpose.** Pushes the transform once.

**Details.** Called by the service every simulation step.

**Exact declaration**

```cpp
void push()
```

- **Declared at:** [line 530](../../../runtime/world2d_content.hpp#L530)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Called by the service every simulation step.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::RemoteTransform2DComponent& object = /* obtain a valid instance */;

object.push();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-set-target-1"></a>

## `epok::RemoteTransform2DComponent::set_target`

**Purpose.** Sets target as part of the world2d content module.

**Exact declaration**

```cpp
void set_target(ObjectId value)
```

- **Declared at:** [line 527](../../../runtime/world2d_content.hpp#L527)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectId` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId value

epok::RemoteTransform2DComponent& object = /* obtain a valid instance */;

object.set_target(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-remotetransform2dcomponent-remotetransform2dcomponent-1"></a>

## `epok::RemoteTransform2DComponent::~RemoteTransform2DComponent`

**Purpose.** Releases the resources owned by `epok::RemoteTransform2DComponent`.

**Exact declaration**

```cpp
~RemoteTransform2DComponent() override
```

- **Declared at:** [line 521](../../../runtime/world2d_content.hpp#L521)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::RemoteTransform2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-visibleonscreennotifier2dcomponent-class-id-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d content module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 498](../../../runtime/world2d_content.hpp#L498)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::VisibleOnScreenNotifier2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-visibleonscreennotifier2dcomponent-is-on-screen-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::is_on_screen`

**Purpose.** Reports whether on screen as part of the world2d content module.

**Exact declaration**

```cpp
bool is_on_screen() const
```

- **Declared at:** [line 507](../../../runtime/world2d_content.hpp#L507)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::VisibleOnScreenNotifier2DComponent& object = /* obtain a valid instance */;

auto result = object.is_on_screen();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibleonscreennotifier2dcomponent-level-attached-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d content module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 499](../../../runtime/world2d_content.hpp#L499)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::VisibleOnScreenNotifier2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-visibleonscreennotifier2dcomponent-take-screen-entered-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::take_screen_entered`

**Purpose.** Performs `take screen entered` as part of the world2d content module.

**Exact declaration**

```cpp
bool take_screen_entered()
```

- **Declared at:** [line 508](../../../runtime/world2d_content.hpp#L508)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::VisibleOnScreenNotifier2DComponent& object = /* obtain a valid instance */;

auto result = object.take_screen_entered();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibleonscreennotifier2dcomponent-take-screen-exited-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::take_screen_exited`

**Purpose.** Performs `take screen exited` as part of the world2d content module.

**Exact declaration**

```cpp
bool take_screen_exited()
```

- **Declared at:** [line 509](../../../runtime/world2d_content.hpp#L509)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

epok::VisibleOnScreenNotifier2DComponent& object = /* obtain a valid instance */;

auto result = object.take_screen_exited();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibleonscreennotifier2dcomponent-visibleonscreennotifier2dcomponent-1"></a>

## `epok::VisibleOnScreenNotifier2DComponent::~VisibleOnScreenNotifier2DComponent`

**Purpose.** Releases the resources owned by `epok::VisibleOnScreenNotifier2DComponent`.

**Exact declaration**

```cpp
~VisibleOnScreenNotifier2DComponent() override
```

- **Declared at:** [line 500](../../../runtime/world2d_content.hpp#L500)
- **Kind:** `destructor`

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// `epok::VisibleOnScreenNotifier2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-angle-degrees-1"></a>

## `epok::world2d::angle_degrees`

**Purpose.** Angle of a vector in Q12 degrees, from +X towards +Y (clockwise on screen), in (-180, 180].

**Details.** A rational approximation, within a third of a degree.

**Exact declaration**

```cpp
inline Fixed angle_degrees(Fixed x, Fixed y)
```

- **Declared at:** [line 54](../../../runtime/world2d_content.hpp#L54)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** A rational approximation, within a third of a degree.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::world2d::angle_degrees(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-isqrt64-1"></a>

## `epok::world2d::isqrt64`

**Purpose.** Integer square root of a non-negative 64-bit value.

**Exact declaration**

```cpp
inline int64_t isqrt64(int64_t value)
```

- **Declared at:** [line 39](../../../runtime/world2d_content.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value

auto result = epok::world2d::isqrt64(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-length-2d-1"></a>

## `epok::world2d::length_2d`

**Purpose.** Length of a Q12 vector, in Q12.

**Exact declaration**

```cpp
inline Fixed length_2d(Fixed x, Fixed y)
```

- **Declared at:** [line 68](../../../runtime/world2d_content.hpp#L68)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

auto result = epok::world2d::length_2d(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d content module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-set-world-position-1"></a>

## `epok::world2d::set_world_position`

**Purpose.** Moves the owner's root so its world position becomes `world`, through the inverse of the parent chain's linear part when the root is attached.

**Exact declaration**

```cpp
inline bool set_world_position(SceneComponent2D& root, const Fixed* world)
```

- **Declared at:** [line 73](../../../runtime/world2d_content.hpp#L73)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `SceneComponent2D &` | Input/output; inspect the function contract | Value supplied for `root`. See the exact type and module contract. |
| `world` | `const Fixed *` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d content module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_content.hpp"

// Assume these named values have been initialized with valid data:
// SceneComponent2D & root
// const Fixed * world

auto result = epok::world2d::set_world_position(root, world);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
