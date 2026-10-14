# Epok API: World2D Navigation

> **Header:** `"world2d_navigation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_navigation.hpp)

This module covers the world2d navigation module. It documents 44 public callables declared directly in this header.

## Declared types

`epok::NavigationAgent2DComponent`, `epok::NavigationRegion2DComponent`, `epok::world2d::NavGrid2D`, `epok::world2d::NavMesh2D`, `epok::world2d::NavScratch`

## Callable index

- [`epok::NavigationAgent2DComponent::class_id`](#epok-navigationagent2dcomponent-class-id-1) — Performs `class id` as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_max_speed`](#epok-navigationagent2dcomponent-get-max-speed-1) — Returns max speed as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_next_x`](#epok-navigationagent2dcomponent-get-next-x-1) — Returns next x as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_next_y`](#epok-navigationagent2dcomponent-get-next-y-1) — Returns next y as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_path_point_count`](#epok-navigationagent2dcomponent-get-path-point-count-1) — Returns path point count as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_target_actor`](#epok-navigationagent2dcomponent-get-target-actor-1) — Returns target actor as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_target_x`](#epok-navigationagent2dcomponent-get-target-x-1) — Returns target x as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_target_y`](#epok-navigationagent2dcomponent-get-target-y-1) — Returns target y as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_velocity_x`](#epok-navigationagent2dcomponent-get-velocity-x-1) — Returns velocity x as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::get_velocity_y`](#epok-navigationagent2dcomponent-get-velocity-y-1) — Returns velocity y as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::is_navigation_finished`](#epok-navigationagent2dcomponent-is-navigation-finished-1) — Reports whether navigation finished as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::is_target_reachable`](#epok-navigationagent2dcomponent-is-target-reachable-1) — Reports whether target reachable as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::level_attached`](#epok-navigationagent2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::set_max_speed`](#epok-navigationagent2dcomponent-set-max-speed-1) — Sets max speed as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::set_target_actor`](#epok-navigationagent2dcomponent-set-target-actor-1) — Sets target actor as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::set_target_position`](#epok-navigationagent2dcomponent-set-target-position-1) — Sets target position as part of the world2d navigation module.
- [`epok::NavigationAgent2DComponent::stop`](#epok-navigationagent2dcomponent-stop-1) — Stops following: no target, no path, no velocity.
- [`epok::NavigationAgent2DComponent::take_arrived`](#epok-navigationagent2dcomponent-take-arrived-1) — True once when the agent reaches a reachable target.
- [`epok::NavigationAgent2DComponent::~NavigationAgent2DComponent`](#epok-navigationagent2dcomponent-navigationagent2dcomponent-1) — Releases the resources owned by `epok::NavigationAgent2DComponent`.
- [`epok::NavigationRegion2DComponent::class_id`](#epok-navigationregion2dcomponent-class-id-1) — Performs `class id` as part of the world2d navigation module.
- [`epok::NavigationRegion2DComponent::covers`](#epok-navigationregion2dcomponent-covers-1) — Whether the region is the one to plan from `p`: inside the grid, or on the mesh.
- [`epok::NavigationRegion2DComponent::find_path`](#epok-navigationregion2dcomponent-find-path-1) — Finds path as part of the world2d navigation module.
- [`epok::NavigationRegion2DComponent::is_enabled`](#epok-navigationregion2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d navigation module.
- [`epok::NavigationRegion2DComponent::is_walkable`](#epok-navigationregion2dcomponent-is-walkable-1) — Whether a world point is inside the region and free to walk on.
- [`epok::NavigationRegion2DComponent::level_attached`](#epok-navigationregion2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d navigation module.
- [`epok::NavigationRegion2DComponent::set_enabled`](#epok-navigationregion2dcomponent-set-enabled-1) — Sets enabled as part of the world2d navigation module.
- [`epok::NavigationRegion2DComponent::~NavigationRegion2DComponent`](#epok-navigationregion2dcomponent-navigationregion2dcomponent-1) — Releases the resources owned by `epok::NavigationRegion2DComponent`.
- [`epok::world2d::grid_line_of_sight`](#epok-world2d-grid-line-of-sight-1) — Whether the straight line between two cell centres crosses only walkable cells and squeezes past no blocked corner.
- [`epok::world2d::grid_path`](#epok-world2d-grid-path-1) — Waypoints from `from` to `to` on a grid (the start excluded), at most `capacity`.
- [`epok::world2d::mesh_path`](#epok-world2d-mesh-path-1) — Waypoints from `from` to `to` on a navmesh (the start excluded), at most `capacity`.
- [`epok::world2d::nav_detail::area2`](#epok-world2d-nav-detail-area2-1) — Area of (b - a) x (c - a), in raw Q12 squared.
- [`epok::world2d::nav_detail::cell_of`](#epok-world2d-nav-detail-cell-of-1) — Performs `cell of` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::centre`](#epok-world2d-nav-detail-centre-1) — Performs `centre` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::centroid`](#epok-world2d-nav-detail-centroid-1) — Performs `centroid` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::closest_on_segment`](#epok-world2d-nav-detail-closest-on-segment-1) — Performs `closest on segment` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::contains`](#epok-world2d-nav-detail-contains-1) — Performs `contains` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::corner`](#epok-world2d-nav-detail-corner-1) — Performs `corner` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::floor_div`](#epok-world2d-nav-detail-floor-div-1) — Performs `floor div` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::heap_pop`](#epok-world2d-nav-detail-heap-pop-1) — Performs `heap pop` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::heap_push`](#epok-world2d-nav-detail-heap-push-1) — Performs `heap push` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::locate`](#epok-world2d-nav-detail-locate-1) — The triangle holding `p` (true), or the closest triangle and point on it (false).
- [`epok::world2d::nav_detail::octile`](#epok-world2d-nav-detail-octile-1) — Performs `octile` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::same`](#epok-world2d-nav-detail-same-1) — Performs `same` as part of the world2d navigation module.
- [`epok::world2d::nav_detail::walkable`](#epok-world2d-nav-detail-walkable-1) — Performs `walkable` as part of the world2d navigation module.

<a id="epok-navigationagent2dcomponent-class-id-1"></a>

## `epok::NavigationAgent2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d navigation module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 468](../../../runtime/world2d_navigation.hpp#L468)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-max-speed-1"></a>

## `epok::NavigationAgent2DComponent::get_max_speed`

**Purpose.** Returns max speed as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_max_speed() const
```

- **Declared at:** [line 522](../../../runtime/world2d_navigation.hpp#L522)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_max_speed();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-next-x-1"></a>

## `epok::NavigationAgent2DComponent::get_next_x`

**Purpose.** Returns next x as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_next_x() const
```

- **Declared at:** [line 516](../../../runtime/world2d_navigation.hpp#L516)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_next_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-next-y-1"></a>

## `epok::NavigationAgent2DComponent::get_next_y`

**Purpose.** Returns next y as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_next_y() const
```

- **Declared at:** [line 517](../../../runtime/world2d_navigation.hpp#L517)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_next_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-path-point-count-1"></a>

## `epok::NavigationAgent2DComponent::get_path_point_count`

**Purpose.** Returns path point count as part of the world2d navigation module.

**Exact declaration**

```cpp
int32_t get_path_point_count() const
```

- **Declared at:** [line 520](../../../runtime/world2d_navigation.hpp#L520)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_path_point_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-target-actor-1"></a>

## `epok::NavigationAgent2DComponent::get_target_actor`

**Purpose.** Returns target actor as part of the world2d navigation module.

**Exact declaration**

```cpp
ObjectId get_target_actor() const
```

- **Declared at:** [line 497](../../../runtime/world2d_navigation.hpp#L497)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_target_actor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-target-x-1"></a>

## `epok::NavigationAgent2DComponent::get_target_x`

**Purpose.** Returns target x as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_target_x() const
```

- **Declared at:** [line 498](../../../runtime/world2d_navigation.hpp#L498)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_target_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-target-y-1"></a>

## `epok::NavigationAgent2DComponent::get_target_y`

**Purpose.** Returns target y as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_target_y() const
```

- **Declared at:** [line 499](../../../runtime/world2d_navigation.hpp#L499)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_target_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-velocity-x-1"></a>

## `epok::NavigationAgent2DComponent::get_velocity_x`

**Purpose.** Returns velocity x as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_velocity_x() const
```

- **Declared at:** [line 518](../../../runtime/world2d_navigation.hpp#L518)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_velocity_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-get-velocity-y-1"></a>

## `epok::NavigationAgent2DComponent::get_velocity_y`

**Purpose.** Returns velocity y as part of the world2d navigation module.

**Exact declaration**

```cpp
Fixed get_velocity_y() const
```

- **Declared at:** [line 519](../../../runtime/world2d_navigation.hpp#L519)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.get_velocity_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-is-navigation-finished-1"></a>

## `epok::NavigationAgent2DComponent::is_navigation_finished`

**Purpose.** Reports whether navigation finished as part of the world2d navigation module.

**Exact declaration**

```cpp
bool is_navigation_finished() const
```

- **Declared at:** [line 508](../../../runtime/world2d_navigation.hpp#L508)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.is_navigation_finished();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagent2dcomponent-is-target-reachable-1"></a>

## `epok::NavigationAgent2DComponent::is_target_reachable`

**Purpose.** Reports whether target reachable as part of the world2d navigation module.

**Exact declaration**

```cpp
bool is_target_reachable() const
```

- **Declared at:** [line 509](../../../runtime/world2d_navigation.hpp#L509)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.is_target_reachable();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagent2dcomponent-level-attached-1"></a>

## `epok::NavigationAgent2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d navigation module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 469](../../../runtime/world2d_navigation.hpp#L469)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-set-max-speed-1"></a>

## `epok::NavigationAgent2DComponent::set_max_speed`

**Purpose.** Sets max speed as part of the world2d navigation module.

**Exact declaration**

```cpp
void set_max_speed(Fixed value)
```

- **Declared at:** [line 521](../../../runtime/world2d_navigation.hpp#L521)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

object.set_max_speed(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-set-target-actor-1"></a>

## `epok::NavigationAgent2DComponent::set_target_actor`

**Purpose.** Sets target actor as part of the world2d navigation module.

**Exact declaration**

```cpp
void set_target_actor(ObjectId actor)
```

- **Declared at:** [line 491](../../../runtime/world2d_navigation.hpp#L491)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

object.set_target_actor(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-set-target-position-1"></a>

## `epok::NavigationAgent2DComponent::set_target_position`

**Purpose.** Sets target position as part of the world2d navigation module.

**Exact declaration**

```cpp
void set_target_position(Fixed x, Fixed y)
```

- **Declared at:** [line 483](../../../runtime/world2d_navigation.hpp#L483)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

object.set_target_position(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-stop-1"></a>

## `epok::NavigationAgent2DComponent::stop`

**Purpose.** Stops following: no target, no path, no velocity.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 501](../../../runtime/world2d_navigation.hpp#L501)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationagent2dcomponent-take-arrived-1"></a>

## `epok::NavigationAgent2DComponent::take_arrived`

**Purpose.** True once when the agent reaches a reachable target.

**Exact declaration**

```cpp
bool take_arrived()
```

- **Declared at:** [line 511](../../../runtime/world2d_navigation.hpp#L511)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationAgent2DComponent& object = /* obtain a valid instance */;

auto result = object.take_arrived();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationagent2dcomponent-navigationagent2dcomponent-1"></a>

## `epok::NavigationAgent2DComponent::~NavigationAgent2DComponent`

**Purpose.** Releases the resources owned by `epok::NavigationAgent2DComponent`.

**Exact declaration**

```cpp
~NavigationAgent2DComponent() override
```

- **Declared at:** [line 470](../../../runtime/world2d_navigation.hpp#L470)
- **Kind:** `destructor`

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// `epok::NavigationAgent2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationregion2dcomponent-class-id-1"></a>

## `epok::NavigationRegion2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d navigation module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 425](../../../runtime/world2d_navigation.hpp#L425)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationregion2dcomponent-covers-1"></a>

## `epok::NavigationRegion2DComponent::covers`

**Purpose.** Whether the region is the one to plan from `p`: inside the grid, or on the mesh.

**Exact declaration**

```cpp
bool covers(const Fixed* p) const
```

- **Declared at:** [line 450](../../../runtime/world2d_navigation.hpp#L450)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const Fixed *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * p

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

auto result = object.covers(p);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-navigationregion2dcomponent-find-path-1"></a>

## `epok::NavigationRegion2DComponent::find_path`

**Purpose.** Finds path as part of the world2d navigation module.

**Exact declaration**

```cpp
size_t find_path(const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached) const
```

- **Declared at:** [line 458](../../../runtime/world2d_navigation.hpp#L458)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `const Fixed *` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `const Fixed *` | Input | Value supplied for `to`. See the exact type and module contract. |
| `out` | `Fixed (*)[2]` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `reached` | `bool &` | Input/output; inspect the function contract | Value supplied for `reached`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * from
// const Fixed * to
// Fixed (*)[2] out
// size_t capacity
// bool & reached

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

auto result = object.find_path(from, to, out, capacity, reached);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-navigationregion2dcomponent-is-enabled-1"></a>

## `epok::NavigationRegion2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d navigation module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 435](../../../runtime/world2d_navigation.hpp#L435)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationregion2dcomponent-is-walkable-1"></a>

## `epok::NavigationRegion2DComponent::is_walkable`

**Purpose.** Whether a world point is inside the region and free to walk on.

**Exact declaration**

```cpp
bool is_walkable(Fixed x, Fixed y) const
```

- **Declared at:** [line 437](../../../runtime/world2d_navigation.hpp#L437)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

auto result = object.is_walkable(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-navigationregion2dcomponent-level-attached-1"></a>

## `epok::NavigationRegion2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d navigation module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 426](../../../runtime/world2d_navigation.hpp#L426)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationregion2dcomponent-set-enabled-1"></a>

## `epok::NavigationRegion2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d navigation module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 434](../../../runtime/world2d_navigation.hpp#L434)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::NavigationRegion2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-navigationregion2dcomponent-navigationregion2dcomponent-1"></a>

## `epok::NavigationRegion2DComponent::~NavigationRegion2DComponent`

**Purpose.** Releases the resources owned by `epok::NavigationRegion2DComponent`.

**Exact declaration**

```cpp
~NavigationRegion2DComponent() override
```

- **Declared at:** [line 427](../../../runtime/world2d_navigation.hpp#L427)
- **Kind:** `destructor`

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// `epok::NavigationRegion2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-grid-line-of-sight-1"></a>

## `epok::world2d::grid_line_of_sight`

**Purpose.** Whether the straight line between two cell centres crosses only walkable cells and squeezes past no blocked corner.

**Exact declaration**

```cpp
inline bool grid_line_of_sight(const NavGrid2D& g, int32_t ax, int32_t ay, int32_t bx, int32_t by)
```

- **Declared at:** [line 105](../../../runtime/world2d_navigation.hpp#L105)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `g` | `const NavGrid2D &` | Input | Value supplied for `g`. See the exact type and module contract. |
| `ax` | `int32_t` | Input | Value supplied for `ax`. See the exact type and module contract. |
| `ay` | `int32_t` | Input | Value supplied for `ay`. See the exact type and module contract. |
| `bx` | `int32_t` | Input | Value supplied for `bx`. See the exact type and module contract. |
| `by` | `int32_t` | Input | Value supplied for `by`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavGrid2D & g
// int32_t ax
// int32_t ay
// int32_t bx
// int32_t by

auto result = epok::world2d::grid_line_of_sight(g, ax, ay, bx, by);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-grid-path-1"></a>

## `epok::world2d::grid_path`

**Purpose.** Waypoints from `from` to `to` on a grid (the start excluded), at most `capacity`.

**Exact declaration**

```cpp
inline size_t grid_path(const NavGrid2D& g, const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached)
```

- **Declared at:** [line 130](../../../runtime/world2d_navigation.hpp#L130)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `g` | `const NavGrid2D &` | Input | Value supplied for `g`. See the exact type and module contract. |
| `from` | `const Fixed *` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `const Fixed *` | Input | Value supplied for `to`. See the exact type and module contract. |
| `out` | `Fixed (*)[2]` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `reached` | `bool &` | Input/output; inspect the function contract | Value supplied for `reached`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavGrid2D & g
// const Fixed * from
// const Fixed * to
// Fixed (*)[2] out
// size_t capacity
// bool & reached

auto result = epok::world2d::grid_path(g, from, to, out, capacity, reached);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-mesh-path-1"></a>

## `epok::world2d::mesh_path`

**Purpose.** Waypoints from `from` to `to` on a navmesh (the start excluded), at most `capacity`.

**Exact declaration**

```cpp
inline size_t mesh_path(const NavMesh2D& m, const Fixed* from, const Fixed* to, Fixed (*out)[2], size_t capacity, bool& reached)
```

- **Declared at:** [line 290](../../../runtime/world2d_navigation.hpp#L290)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const NavMesh2D &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `from` | `const Fixed *` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `const Fixed *` | Input | Value supplied for `to`. See the exact type and module contract. |
| `out` | `Fixed (*)[2]` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `reached` | `bool &` | Input/output; inspect the function contract | Value supplied for `reached`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavMesh2D & m
// const Fixed * from
// const Fixed * to
// Fixed (*)[2] out
// size_t capacity
// bool & reached

auto result = epok::world2d::mesh_path(m, from, to, out, capacity, reached);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-area2-1"></a>

## `epok::world2d::nav_detail::area2`

**Purpose.** Area of (b - a) x (c - a), in raw Q12 squared.

**Exact declaration**

```cpp
inline int64_t area2(const Fixed* a, const Fixed* b, const Fixed* c)
```

- **Declared at:** [line 97](../../../runtime/world2d_navigation.hpp#L97)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Fixed *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Fixed *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Fixed *` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * a
// const Fixed * b
// const Fixed * c

auto result = epok::world2d::nav_detail::area2(a, b, c);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-cell-of-1"></a>

## `epok::world2d::nav_detail::cell_of`

**Purpose.** Performs `cell of` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void cell_of(const NavGrid2D& g, const Fixed* p, int32_t* c)
```

- **Declared at:** [line 57](../../../runtime/world2d_navigation.hpp#L57)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `g` | `const NavGrid2D &` | Input | Value supplied for `g`. See the exact type and module contract. |
| `p` | `const Fixed *` | Input | Value supplied for `p`. See the exact type and module contract. |
| `c` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `c`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavGrid2D & g
// const Fixed * p
// int32_t * c

epok::world2d::nav_detail::cell_of(g, p, c);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-centre-1"></a>

## `epok::world2d::nav_detail::centre`

**Purpose.** Performs `centre` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void centre(const NavGrid2D& g, int32_t x, int32_t y, Fixed* out)
```

- **Declared at:** [line 60](../../../runtime/world2d_navigation.hpp#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `g` | `const NavGrid2D &` | Input | Value supplied for `g`. See the exact type and module contract. |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavGrid2D & g
// int32_t x
// int32_t y
// Fixed * out

epok::world2d::nav_detail::centre(g, x, y, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-centroid-1"></a>

## `epok::world2d::nav_detail::centroid`

**Purpose.** Performs `centroid` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void centroid(const NavMesh2D& m, uint8_t t, Fixed* out)
```

- **Declared at:** [line 246](../../../runtime/world2d_navigation.hpp#L246)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const NavMesh2D &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `t` | `uint8_t` | Input | Value supplied for `t`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavMesh2D & m
// uint8_t t
// Fixed * out

epok::world2d::nav_detail::centroid(m, t, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-closest-on-segment-1"></a>

## `epok::world2d::nav_detail::closest_on_segment`

**Purpose.** Performs `closest on segment` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void closest_on_segment(const Fixed* p, const Fixed* a, const Fixed* b, Fixed* out)
```

- **Declared at:** [line 251](../../../runtime/world2d_navigation.hpp#L251)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const Fixed *` | Input | Value supplied for `p`. See the exact type and module contract. |
| `a` | `const Fixed *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Fixed *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * p
// const Fixed * a
// const Fixed * b
// Fixed * out

epok::world2d::nav_detail::closest_on_segment(p, a, b, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-contains-1"></a>

## `epok::world2d::nav_detail::contains`

**Purpose.** Performs `contains` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline bool contains(const NavMesh2D& m, uint8_t t, const Fixed* p)
```

- **Declared at:** [line 238](../../../runtime/world2d_navigation.hpp#L238)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const NavMesh2D &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `t` | `uint8_t` | Input | Value supplied for `t`. See the exact type and module contract. |
| `p` | `const Fixed *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavMesh2D & m
// uint8_t t
// const Fixed * p

auto result = epok::world2d::nav_detail::contains(m, t, p);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-corner-1"></a>

## `epok::world2d::nav_detail::corner`

**Purpose.** Performs `corner` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void corner(const NavMesh2D& m, uint8_t t, int k, Fixed* out)
```

- **Declared at:** [line 233](../../../runtime/world2d_navigation.hpp#L233)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const NavMesh2D &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `t` | `uint8_t` | Input | Value supplied for `t`. See the exact type and module contract. |
| `k` | `int` | Input | Value supplied for `k`. See the exact type and module contract. |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavMesh2D & m
// uint8_t t
// int k
// Fixed * out

epok::world2d::nav_detail::corner(m, t, k, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-floor-div-1"></a>

## `epok::world2d::nav_detail::floor_div`

**Purpose.** Performs `floor div` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline int32_t floor_div(int32_t a, int32_t b)
```

- **Declared at:** [line 51](../../../runtime/world2d_navigation.hpp#L51)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::world2d::nav_detail::floor_div(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-nav-detail-heap-pop-1"></a>

## `epok::world2d::nav_detail::heap_pop`

**Purpose.** Performs `heap pop` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline uint32_t heap_pop(NavScratch& s, size_t& count)
```

- **Declared at:** [line 80](../../../runtime/world2d_navigation.hpp#L80)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `NavScratch &` | Input/output; inspect the function contract | Value supplied for `s`. See the exact type and module contract. |
| `count` | `size_t &` | Input/output; inspect the function contract | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// NavScratch & s
// size_t & count

auto result = epok::world2d::nav_detail::heap_pop(s, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-heap-push-1"></a>

## `epok::world2d::nav_detail::heap_push`

**Purpose.** Performs `heap push` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline void heap_push(NavScratch& s, size_t& count, uint32_t key)
```

- **Declared at:** [line 69](../../../runtime/world2d_navigation.hpp#L69)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `NavScratch &` | Input/output; inspect the function contract | Value supplied for `s`. See the exact type and module contract. |
| `count` | `size_t &` | Input/output; inspect the function contract | Value supplied for `count`. See the exact type and module contract. |
| `key` | `uint32_t` | Input | Value supplied for `key`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// NavScratch & s
// size_t & count
// uint32_t key

epok::world2d::nav_detail::heap_push(s, count, key);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-locate-1"></a>

## `epok::world2d::nav_detail::locate`

**Purpose.** The triangle holding `p` (true), or the closest triangle and point on it (false).

**Exact declaration**

```cpp
inline bool locate(const NavMesh2D& m, const Fixed* p, uint8_t& triangle, Fixed* point)
```

- **Declared at:** [line 260](../../../runtime/world2d_navigation.hpp#L260)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const NavMesh2D &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `p` | `const Fixed *` | Input | Value supplied for `p`. See the exact type and module contract. |
| `triangle` | `uint8_t &` | Input/output; inspect the function contract | Value supplied for `triangle`. See the exact type and module contract. |
| `point` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavMesh2D & m
// const Fixed * p
// uint8_t & triangle
// Fixed * point

auto result = epok::world2d::nav_detail::locate(m, p, triangle, point);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-octile-1"></a>

## `epok::world2d::nav_detail::octile`

**Purpose.** Performs `octile` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline uint16_t octile(int32_t ax, int32_t ay, int32_t bx, int32_t by)
```

- **Declared at:** [line 64](../../../runtime/world2d_navigation.hpp#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ax` | `int32_t` | Input | Value supplied for `ax`. See the exact type and module contract. |
| `ay` | `int32_t` | Input | Value supplied for `ay`. See the exact type and module contract. |
| `bx` | `int32_t` | Input | Value supplied for `bx`. See the exact type and module contract. |
| `by` | `int32_t` | Input | Value supplied for `by`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// int32_t ax
// int32_t ay
// int32_t bx
// int32_t by

auto result = epok::world2d::nav_detail::octile(ax, ay, bx, by);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-nav-detail-same-1"></a>

## `epok::world2d::nav_detail::same`

**Purpose.** Performs `same` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline bool same(const Fixed* a, const Fixed* b)
```

- **Declared at:** [line 100](../../../runtime/world2d_navigation.hpp#L100)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Fixed *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Fixed *` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * a
// const Fixed * b

auto result = epok::world2d::nav_detail::same(a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-nav-detail-walkable-1"></a>

## `epok::world2d::nav_detail::walkable`

**Purpose.** Performs `walkable` as part of the world2d navigation module.

**Exact declaration**

```cpp
inline bool walkable(const NavGrid2D& g, int32_t x, int32_t y)
```

- **Declared at:** [line 52](../../../runtime/world2d_navigation.hpp#L52)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `g` | `const NavGrid2D &` | Input | Value supplied for `g`. See the exact type and module contract. |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const NavGrid2D & g
// int32_t x
// int32_t y

auto result = epok::world2d::nav_detail::walkable(g, x, y);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
