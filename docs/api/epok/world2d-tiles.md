# Epok API: World2D Tiles

> **Header:** `"world2d_tiles.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_tiles.hpp)

This module covers the world2d tiles module. It documents 63 public callables declared directly in this header.

## Declared types

`epok::CanvasLayer2DComponent`, `epok::CanvasModulate2DComponent`, `epok::Depth2DComponent`, `epok::Parallax2DComponent`, `epok::TileCollision`, `epok::TileInfo`, `epok::TileMapLayerComponent`, `epok::TileSetData`, `epok::world2d::Canvas`, `epok::world2d::TileHooks`

## Callable index

- [`epok::CanvasLayer2DComponent::class_id`](#epok-canvaslayer2dcomponent-class-id-1) — Performs `class id` as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::get_layer`](#epok-canvaslayer2dcomponent-get-layer-1) — Returns layer as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::is_under_3d`](#epok-canvaslayer2dcomponent-is-under-3d-1) — Reports whether under 3d as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::is_visible`](#epok-canvaslayer2dcomponent-is-visible-1) — Reports whether visible as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::level_attached`](#epok-canvaslayer2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::set_follow_viewport`](#epok-canvaslayer2dcomponent-set-follow-viewport-1) — Sets follow viewport as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::set_layer`](#epok-canvaslayer2dcomponent-set-layer-1) — Sets layer as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::set_offset`](#epok-canvaslayer2dcomponent-set-offset-1) — Sets offset as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::set_under_3d`](#epok-canvaslayer2dcomponent-set-under-3d-1) — Sets under 3d as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::set_visible`](#epok-canvaslayer2dcomponent-set-visible-1) — Sets visible as part of the world2d tiles module.
- [`epok::CanvasLayer2DComponent::~CanvasLayer2DComponent`](#epok-canvaslayer2dcomponent-canvaslayer2dcomponent-1) — Releases the resources owned by `epok::CanvasLayer2DComponent`.
- [`epok::CanvasModulate2DComponent::class_id`](#epok-canvasmodulate2dcomponent-class-id-1) — Performs `class id` as part of the world2d tiles module.
- [`epok::CanvasModulate2DComponent::level_attached`](#epok-canvasmodulate2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d tiles module.
- [`epok::CanvasModulate2DComponent::set_color`](#epok-canvasmodulate2dcomponent-set-color-1) — Sets color as part of the world2d tiles module.
- [`epok::CanvasModulate2DComponent::~CanvasModulate2DComponent`](#epok-canvasmodulate2dcomponent-canvasmodulate2dcomponent-1) — Releases the resources owned by `epok::CanvasModulate2DComponent`.
- [`epok::Depth2DComponent::class_id`](#epok-depth2dcomponent-class-id-1) — Performs `class id` as part of the world2d tiles module.
- [`epok::Depth2DComponent::get_anchor`](#epok-depth2dcomponent-get-anchor-1) — Returns anchor as part of the world2d tiles module.
- [`epok::Depth2DComponent::get_bias`](#epok-depth2dcomponent-get-bias-1) — Returns bias as part of the world2d tiles module.
- [`epok::Depth2DComponent::get_distance`](#epok-depth2dcomponent-get-distance-1) — Returns distance as part of the world2d tiles module.
- [`epok::Depth2DComponent::is_enabled`](#epok-depth2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d tiles module.
- [`epok::Depth2DComponent::level_attached`](#epok-depth2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d tiles module.
- [`epok::Depth2DComponent::set_anchor`](#epok-depth2dcomponent-set-anchor-1) — A 3D actor whose position gives the depth; an empty id returns to `distance`.
- [`epok::Depth2DComponent::set_bias`](#epok-depth2dcomponent-set-bias-1) — Sets bias as part of the world2d tiles module.
- [`epok::Depth2DComponent::set_distance`](#epok-depth2dcomponent-set-distance-1) — World units in front of the 3D camera, 0 to 128 (the far end of the ordering table).
- [`epok::Depth2DComponent::set_enabled`](#epok-depth2dcomponent-set-enabled-1) — Sets enabled as part of the world2d tiles module.
- [`epok::Depth2DComponent::~Depth2DComponent`](#epok-depth2dcomponent-depth2dcomponent-1) — Releases the resources owned by `epok::Depth2DComponent`.
- [`epok::Parallax2DComponent::class_id`](#epok-parallax2dcomponent-class-id-1) — Performs `class id` as part of the world2d tiles module.
- [`epok::Parallax2DComponent::follow`](#epok-parallax2dcomponent-follow-1) — One simulation step: `camera` is the active camera's centre (or 0, 0 without one).
- [`epok::Parallax2DComponent::level_attached`](#epok-parallax2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d tiles module.
- [`epok::Parallax2DComponent::set_autoscroll`](#epok-parallax2dcomponent-set-autoscroll-1) — Sets autoscroll as part of the world2d tiles module.
- [`epok::Parallax2DComponent::set_repeat_size`](#epok-parallax2dcomponent-set-repeat-size-1) — Sets repeat size as part of the world2d tiles module.
- [`epok::Parallax2DComponent::set_scroll_scale`](#epok-parallax2dcomponent-set-scroll-scale-1) — Sets scroll scale as part of the world2d tiles module.
- [`epok::Parallax2DComponent::~Parallax2DComponent`](#epok-parallax2dcomponent-parallax2dcomponent-1) — Releases the resources owned by `epok::Parallax2DComponent`.
- [`epok::TileMapLayerComponent::cell`](#epok-tilemaplayercomponent-cell-1) — The value of a cell, runtime edits included.
- [`epok::TileMapLayerComponent::class_id`](#epok-tilemaplayercomponent-class-id-1) — Performs `class id` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::cooked_cell`](#epok-tilemaplayercomponent-cooked-cell-1) — The cooked value of a cell, ignoring runtime edits.
- [`epok::TileMapLayerComponent::erase_cell`](#epok-tilemaplayercomponent-erase-cell-1) — Performs `erase cell` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::get_cell_atlas_x`](#epok-tilemaplayercomponent-get-cell-atlas-x-1) — Atlas column of the cell's tile, or -1 when the cell is empty.
- [`epok::TileMapLayerComponent::get_cell_atlas_y`](#epok-tilemaplayercomponent-get-cell-atlas-y-1) — Atlas row of the cell's tile, or -1 when the cell is empty.
- [`epok::TileMapLayerComponent::get_cell_custom_data`](#epok-tilemaplayercomponent-get-cell-custom-data-1) — Custom data layer `layer` of the cell's tile; 0 for an empty cell or unknown layer.
- [`epok::TileMapLayerComponent::is_cell_empty`](#epok-tilemaplayercomponent-is-cell-empty-1) — Reports whether cell empty as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::is_cell_solid`](#epok-tilemaplayercomponent-is-cell-solid-1) — Whether the cell's tile blocks bodies (full, rectangle or one-way collision).
- [`epok::TileMapLayerComponent::is_visible`](#epok-tilemaplayercomponent-is-visible-1) — Reports whether visible as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::level_attached`](#epok-tilemaplayercomponent-level-attached-1) — Performs `level attached` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::map_to_world_x`](#epok-tilemaplayercomponent-map-to-world-x-1) — World X / Y of a cell's centre.
- [`epok::TileMapLayerComponent::map_to_world_y`](#epok-tilemaplayercomponent-map-to-world-y-1) — Performs `map to world y` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::origin`](#epok-tilemaplayercomponent-origin-1) — World pixels of the grid origin (cell 0, 0's top-left corner).
- [`epok::TileMapLayerComponent::set_cell`](#epok-tilemaplayercomponent-set-cell-1) — Places the atlas tile (atlas_x, atlas_y) in a cell.
- [`epok::TileMapLayerComponent::set_cell_flipped`](#epok-tilemaplayercomponent-set-cell-flipped-1) — Same as set_cell, keeping the flip flags: flip_h, flip_v and transpose.
- [`epok::TileMapLayerComponent::set_collision_enabled`](#epok-tilemaplayercomponent-set-collision-enabled-1) — Sets collision enabled as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::set_modulate`](#epok-tilemaplayercomponent-set-modulate-1) — Sets modulate as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::set_visible`](#epok-tilemaplayercomponent-set-visible-1) — Sets visible as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::set_z_index`](#epok-tilemaplayercomponent-set-z-index-1) — Sets z index as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::tile_height`](#epok-tilemaplayercomponent-tile-height-1) — Performs `tile height` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::tile_of`](#epok-tilemaplayercomponent-tile-of-1) — Performs `tile of` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::tile_width`](#epok-tilemaplayercomponent-tile-width-1) — Performs `tile width` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::world_to_map_x`](#epok-tilemaplayercomponent-world-to-map-x-1) — The cell column containing world X / the cell row containing world Y.
- [`epok::TileMapLayerComponent::world_to_map_y`](#epok-tilemaplayercomponent-world-to-map-y-1) — Performs `world to map y` as part of the world2d tiles module.
- [`epok::TileMapLayerComponent::~TileMapLayerComponent`](#epok-tilemaplayercomponent-tilemaplayercomponent-1) — Releases the resources owned by `epok::TileMapLayerComponent`.
- [`epok::world2d::canvas_of`](#epok-world2d-canvas-of-1) — Walks from `component`'s owner up its attachments.
- [`epok::world2d::depth_bucket`](#epok-world2d-depth-bucket-1) — The bucket of camera-space depth `z` (Q12), moved by `bias` buckets and clamped to the table: the bucket a 3D triangle averaging that depth falls in.
- [`epok::world2d::depth_of`](#epok-world2d-depth-of-1) — The nearest enabled, active Depth2D on `component`'s owner or up its attachments.
- [`epok::world2d::floor_cells`](#epok-world2d-floor-cells-1) — Floor division of a Q12 distance by a whole number of pixels.

<a id="epok-canvaslayer2dcomponent-class-id-1"></a>

## `epok::CanvasLayer2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 306](../../../runtime/world2d_tiles.hpp#L306)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-get-layer-1"></a>

## `epok::CanvasLayer2DComponent::get_layer`

**Purpose.** Returns layer as part of the world2d tiles module.

**Exact declaration**

```cpp
int32_t get_layer() const
```

- **Declared at:** [line 318](../../../runtime/world2d_tiles.hpp#L318)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

auto result = object.get_layer();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-is-under-3d-1"></a>

## `epok::CanvasLayer2DComponent::is_under_3d`

**Purpose.** Reports whether under 3d as part of the world2d tiles module.

**Exact declaration**

```cpp
bool is_under_3d() const
```

- **Declared at:** [line 324](../../../runtime/world2d_tiles.hpp#L324)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

auto result = object.is_under_3d();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-canvaslayer2dcomponent-is-visible-1"></a>

## `epok::CanvasLayer2DComponent::is_visible`

**Purpose.** Reports whether visible as part of the world2d tiles module.

**Exact declaration**

```cpp
bool is_visible() const
```

- **Declared at:** [line 321](../../../runtime/world2d_tiles.hpp#L321)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

auto result = object.is_visible();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-canvaslayer2dcomponent-level-attached-1"></a>

## `epok::CanvasLayer2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d tiles module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 307](../../../runtime/world2d_tiles.hpp#L307)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-set-follow-viewport-1"></a>

## `epok::CanvasLayer2DComponent::set_follow_viewport`

**Purpose.** Sets follow viewport as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_follow_viewport(bool value)
```

- **Declared at:** [line 322](../../../runtime/world2d_tiles.hpp#L322)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.set_follow_viewport(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-set-layer-1"></a>

## `epok::CanvasLayer2DComponent::set_layer`

**Purpose.** Sets layer as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_layer(int32_t value)
```

- **Declared at:** [line 317](../../../runtime/world2d_tiles.hpp#L317)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.set_layer(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-set-offset-1"></a>

## `epok::CanvasLayer2DComponent::set_offset`

**Purpose.** Sets offset as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_offset(Fixed x, Fixed y)
```

- **Declared at:** [line 319](../../../runtime/world2d_tiles.hpp#L319)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.set_offset(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-set-under-3d-1"></a>

## `epok::CanvasLayer2DComponent::set_under_3d`

**Purpose.** Sets under 3d as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_under_3d(bool value)
```

- **Declared at:** [line 323](../../../runtime/world2d_tiles.hpp#L323)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.set_under_3d(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-set-visible-1"></a>

## `epok::CanvasLayer2DComponent::set_visible`

**Purpose.** Sets visible as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 320](../../../runtime/world2d_tiles.hpp#L320)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::CanvasLayer2DComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvaslayer2dcomponent-canvaslayer2dcomponent-1"></a>

## `epok::CanvasLayer2DComponent::~CanvasLayer2DComponent`

**Purpose.** Releases the resources owned by `epok::CanvasLayer2DComponent`.

**Exact declaration**

```cpp
~CanvasLayer2DComponent() override
```

- **Declared at:** [line 308](../../../runtime/world2d_tiles.hpp#L308)
- **Kind:** `destructor`

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// `epok::CanvasLayer2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvasmodulate2dcomponent-class-id-1"></a>

## `epok::CanvasModulate2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 334](../../../runtime/world2d_tiles.hpp#L334)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasModulate2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvasmodulate2dcomponent-level-attached-1"></a>

## `epok::CanvasModulate2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d tiles module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 335](../../../runtime/world2d_tiles.hpp#L335)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::CanvasModulate2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvasmodulate2dcomponent-set-color-1"></a>

## `epok::CanvasModulate2DComponent::set_color`

**Purpose.** Sets color as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_color(uint32_t red, uint32_t green, uint32_t blue)
```

- **Declared at:** [line 341](../../../runtime/world2d_tiles.hpp#L341)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::CanvasModulate2DComponent& object = /* obtain a valid instance */;

object.set_color(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-canvasmodulate2dcomponent-canvasmodulate2dcomponent-1"></a>

## `epok::CanvasModulate2DComponent::~CanvasModulate2DComponent`

**Purpose.** Releases the resources owned by `epok::CanvasModulate2DComponent`.

**Exact declaration**

```cpp
~CanvasModulate2DComponent() override
```

- **Declared at:** [line 336](../../../runtime/world2d_tiles.hpp#L336)
- **Kind:** `destructor`

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// `epok::CanvasModulate2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-class-id-1"></a>

## `epok::Depth2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 364](../../../runtime/world2d_tiles.hpp#L364)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-get-anchor-1"></a>

## `epok::Depth2DComponent::get_anchor`

**Purpose.** Returns anchor as part of the world2d tiles module.

**Exact declaration**

```cpp
ObjectId get_anchor() const
```

- **Declared at:** [line 383](../../../runtime/world2d_tiles.hpp#L383)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

auto result = object.get_anchor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-get-bias-1"></a>

## `epok::Depth2DComponent::get_bias`

**Purpose.** Returns bias as part of the world2d tiles module.

**Exact declaration**

```cpp
int32_t get_bias() const
```

- **Declared at:** [line 387](../../../runtime/world2d_tiles.hpp#L387)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

auto result = object.get_bias();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-get-distance-1"></a>

## `epok::Depth2DComponent::get_distance`

**Purpose.** Returns distance as part of the world2d tiles module.

**Exact declaration**

```cpp
Fixed get_distance() const
```

- **Declared at:** [line 380](../../../runtime/world2d_tiles.hpp#L380)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

auto result = object.get_distance();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-is-enabled-1"></a>

## `epok::Depth2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d tiles module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 375](../../../runtime/world2d_tiles.hpp#L375)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-depth2dcomponent-level-attached-1"></a>

## `epok::Depth2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d tiles module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 365](../../../runtime/world2d_tiles.hpp#L365)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Depth2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-set-anchor-1"></a>

## `epok::Depth2DComponent::set_anchor`

**Purpose.** A 3D actor whose position gives the depth; an empty id returns to `distance`.

**Exact declaration**

```cpp
void set_anchor(ObjectId actor)
```

- **Declared at:** [line 382](../../../runtime/world2d_tiles.hpp#L382)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

epok::Depth2DComponent& object = /* obtain a valid instance */;

object.set_anchor(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-set-bias-1"></a>

## `epok::Depth2DComponent::set_bias`

**Purpose.** Sets bias as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_bias(int32_t buckets)
```

- **Declared at:** [line 384](../../../runtime/world2d_tiles.hpp#L384)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buckets` | `int32_t` | Input | Value supplied for `buckets`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t buckets

epok::Depth2DComponent& object = /* obtain a valid instance */;

object.set_bias(buckets);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-set-distance-1"></a>

## `epok::Depth2DComponent::set_distance`

**Purpose.** World units in front of the 3D camera, 0 to 128 (the far end of the ordering table).

**Exact declaration**

```cpp
void set_distance(Fixed value)
```

- **Declared at:** [line 377](../../../runtime/world2d_tiles.hpp#L377)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Depth2DComponent& object = /* obtain a valid instance */;

object.set_distance(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-set-enabled-1"></a>

## `epok::Depth2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 374](../../../runtime/world2d_tiles.hpp#L374)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Depth2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-depth2dcomponent-depth2dcomponent-1"></a>

## `epok::Depth2DComponent::~Depth2DComponent`

**Purpose.** Releases the resources owned by `epok::Depth2DComponent`.

**Exact declaration**

```cpp
~Depth2DComponent() override
```

- **Declared at:** [line 366](../../../runtime/world2d_tiles.hpp#L366)
- **Kind:** `destructor`

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// `epok::Depth2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-class-id-1"></a>

## `epok::Parallax2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 250](../../../runtime/world2d_tiles.hpp#L250)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Parallax2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-follow-1"></a>

## `epok::Parallax2DComponent::follow`

**Purpose.** One simulation step: `camera` is the active camera's centre (or 0, 0 without one).

**Exact declaration**

```cpp
void follow(const Fixed* camera, Fixed dt)
```

- **Declared at:** [line 270](../../../runtime/world2d_tiles.hpp#L270)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const Fixed *` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// const Fixed * camera
// Fixed dt

epok::Parallax2DComponent& object = /* obtain a valid instance */;

object.follow(camera, dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-parallax2dcomponent-level-attached-1"></a>

## `epok::Parallax2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d tiles module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 251](../../../runtime/world2d_tiles.hpp#L251)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::Parallax2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-set-autoscroll-1"></a>

## `epok::Parallax2DComponent::set_autoscroll`

**Purpose.** Sets autoscroll as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_autoscroll(Fixed x, Fixed y)
```

- **Declared at:** [line 264](../../../runtime/world2d_tiles.hpp#L264)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Parallax2DComponent& object = /* obtain a valid instance */;

object.set_autoscroll(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-set-repeat-size-1"></a>

## `epok::Parallax2DComponent::set_repeat_size`

**Purpose.** Sets repeat size as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_repeat_size(Fixed x, Fixed y)
```

- **Declared at:** [line 265](../../../runtime/world2d_tiles.hpp#L265)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Parallax2DComponent& object = /* obtain a valid instance */;

object.set_repeat_size(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-set-scroll-scale-1"></a>

## `epok::Parallax2DComponent::set_scroll_scale`

**Purpose.** Sets scroll scale as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_scroll_scale(Fixed x, Fixed y)
```

- **Declared at:** [line 263](../../../runtime/world2d_tiles.hpp#L263)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::Parallax2DComponent& object = /* obtain a valid instance */;

object.set_scroll_scale(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-parallax2dcomponent-parallax2dcomponent-1"></a>

## `epok::Parallax2DComponent::~Parallax2DComponent`

**Purpose.** Releases the resources owned by `epok::Parallax2DComponent`.

**Exact declaration**

```cpp
~Parallax2DComponent() override
```

- **Declared at:** [line 252](../../../runtime/world2d_tiles.hpp#L252)
- **Kind:** `destructor`

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// `epok::Parallax2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-cell-1"></a>

## `epok::TileMapLayerComponent::cell`

**Purpose.** The value of a cell, runtime edits included.

**Exact declaration**

```cpp
uint16_t cell(int32_t x, int32_t y) const
```

- **Declared at:** [line 140](../../../runtime/world2d_tiles.hpp#L140)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.cell(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-class-id-1"></a>

## `epok::TileMapLayerComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 100](../../../runtime/world2d_tiles.hpp#L100)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-cooked-cell-1"></a>

## `epok::TileMapLayerComponent::cooked_cell`

**Purpose.** The cooked value of a cell, ignoring runtime edits.

**Exact declaration**

```cpp
uint16_t cooked_cell(int32_t x, int32_t y) const
```

- **Declared at:** [line 130](../../../runtime/world2d_tiles.hpp#L130)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.cooked_cell(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-erase-cell-1"></a>

## `epok::TileMapLayerComponent::erase_cell`

**Purpose.** Performs `erase cell` as part of the world2d tiles module.

**Exact declaration**

```cpp
bool erase_cell(int32_t x, int32_t y)
```

- **Declared at:** [line 198](../../../runtime/world2d_tiles.hpp#L198)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.erase_cell(x, y);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-get-cell-atlas-x-1"></a>

## `epok::TileMapLayerComponent::get_cell_atlas_x`

**Purpose.** Atlas column of the cell's tile, or -1 when the cell is empty.

**Exact declaration**

```cpp
int32_t get_cell_atlas_x(int32_t x, int32_t y) const
```

- **Declared at:** [line 162](../../../runtime/world2d_tiles.hpp#L162)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.get_cell_atlas_x(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-get-cell-atlas-y-1"></a>

## `epok::TileMapLayerComponent::get_cell_atlas_y`

**Purpose.** Atlas row of the cell's tile, or -1 when the cell is empty.

**Exact declaration**

```cpp
int32_t get_cell_atlas_y(int32_t x, int32_t y) const
```

- **Declared at:** [line 167](../../../runtime/world2d_tiles.hpp#L167)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.get_cell_atlas_y(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-get-cell-custom-data-1"></a>

## `epok::TileMapLayerComponent::get_cell_custom_data`

**Purpose.** Custom data layer `layer` of the cell's tile; 0 for an empty cell or unknown layer.

**Exact declaration**

```cpp
int32_t get_cell_custom_data(int32_t x, int32_t y, uint32_t layer) const
```

- **Declared at:** [line 177](../../../runtime/world2d_tiles.hpp#L177)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `layer` | `uint32_t` | Input | Value supplied for `layer`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// uint32_t layer

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.get_cell_custom_data(x, y, layer);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-is-cell-empty-1"></a>

## `epok::TileMapLayerComponent::is_cell_empty`

**Purpose.** Reports whether cell empty as part of the world2d tiles module.

**Exact declaration**

```cpp
bool is_cell_empty(int32_t x, int32_t y) const
```

- **Declared at:** [line 160](../../../runtime/world2d_tiles.hpp#L160)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.is_cell_empty(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-is-cell-solid-1"></a>

## `epok::TileMapLayerComponent::is_cell_solid`

**Purpose.** Whether the cell's tile blocks bodies (full, rectangle or one-way collision).

**Exact declaration**

```cpp
bool is_cell_solid(int32_t x, int32_t y) const
```

- **Declared at:** [line 172](../../../runtime/world2d_tiles.hpp#L172)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.is_cell_solid(x, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-is-visible-1"></a>

## `epok::TileMapLayerComponent::is_visible`

**Purpose.** Reports whether visible as part of the world2d tiles module.

**Exact declaration**

```cpp
bool is_visible() const
```

- **Declared at:** [line 222](../../../runtime/world2d_tiles.hpp#L222)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.is_visible();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-level-attached-1"></a>

## `epok::TileMapLayerComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d tiles module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 101](../../../runtime/world2d_tiles.hpp#L101)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-map-to-world-x-1"></a>

## `epok::TileMapLayerComponent::map_to_world_x`

**Purpose.** World X / Y of a cell's centre.

**Exact declaration**

```cpp
Fixed map_to_world_x(int32_t x) const
```

- **Declared at:** [line 211](../../../runtime/world2d_tiles.hpp#L211)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.map_to_world_x(x);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-map-to-world-y-1"></a>

## `epok::TileMapLayerComponent::map_to_world_y`

**Purpose.** Performs `map to world y` as part of the world2d tiles module.

**Exact declaration**

```cpp
Fixed map_to_world_y(int32_t y) const
```

- **Declared at:** [line 216](../../../runtime/world2d_tiles.hpp#L216)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.map_to_world_y(y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-origin-1"></a>

## `epok::TileMapLayerComponent::origin`

**Purpose.** World pixels of the grid origin (cell 0, 0's top-left corner).

**Details.** False without a root.

**Exact declaration**

```cpp
bool origin(Fixed* out) const
```

- **Declared at:** [line 150](../../../runtime/world2d_tiles.hpp#L150)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** False without a root.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed * out

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.origin(out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-tilemaplayercomponent-set-cell-1"></a>

## `epok::TileMapLayerComponent::set_cell`

**Purpose.** Places the atlas tile (atlas_x, atlas_y) in a cell.

**Details.** False when the tile set has no such tile or the map's runtime edit table is full.

**Exact declaration**

```cpp
bool set_cell(int32_t x, int32_t y, int32_t atlas_x, int32_t atlas_y)
```

- **Declared at:** [line 184](../../../runtime/world2d_tiles.hpp#L184)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `atlas_x` | `int32_t` | Input | Value supplied for `atlas_x`. See the exact type and module contract. |
| `atlas_y` | `int32_t` | Input | Value supplied for `atlas_y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** False when the tile set has no such tile or the map's runtime edit table is full.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t atlas_x
// int32_t atlas_y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.set_cell(x, y, atlas_x, atlas_y);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-set-cell-flipped-1"></a>

## `epok::TileMapLayerComponent::set_cell_flipped`

**Purpose.** Same as set_cell, keeping the flip flags: flip_h, flip_v and transpose.

**Exact declaration**

```cpp
bool set_cell_flipped(int32_t x, int32_t y, int32_t atlas_x, int32_t atlas_y, bool flip_h, bool flip_v, bool transpose)
```

- **Declared at:** [line 191](../../../runtime/world2d_tiles.hpp#L191)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int32_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `int32_t` | Input | Value supplied for `y`. See the exact type and module contract. |
| `atlas_x` | `int32_t` | Input | Value supplied for `atlas_x`. See the exact type and module contract. |
| `atlas_y` | `int32_t` | Input | Value supplied for `atlas_y`. See the exact type and module contract. |
| `flip_h` | `bool` | Input | Value supplied for `flip_h`. See the exact type and module contract. |
| `flip_v` | `bool` | Input | Value supplied for `flip_v`. See the exact type and module contract. |
| `transpose` | `bool` | Input | Value supplied for `transpose`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t x
// int32_t y
// int32_t atlas_x
// int32_t atlas_y
// bool flip_h
// bool flip_v
// bool transpose

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.set_cell_flipped(x, y, atlas_x, atlas_y, flip_h, flip_v, transpose);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tilemaplayercomponent-set-collision-enabled-1"></a>

## `epok::TileMapLayerComponent::set_collision_enabled`

**Purpose.** Sets collision enabled as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_collision_enabled(bool value)
```

- **Declared at:** [line 223](../../../runtime/world2d_tiles.hpp#L223)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

object.set_collision_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-set-modulate-1"></a>

## `epok::TileMapLayerComponent::set_modulate`

**Purpose.** Sets modulate as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_modulate(uint32_t red, uint32_t green, uint32_t blue)
```

- **Declared at:** [line 224](../../../runtime/world2d_tiles.hpp#L224)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `red` | `uint32_t` | Input | Value supplied for `red`. See the exact type and module contract. |
| `green` | `uint32_t` | Input | Value supplied for `green`. See the exact type and module contract. |
| `blue` | `uint32_t` | Input | Value supplied for `blue`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t red
// uint32_t green
// uint32_t blue

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

object.set_modulate(red, green, blue);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-set-visible-1"></a>

## `epok::TileMapLayerComponent::set_visible`

**Purpose.** Sets visible as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_visible(bool value)
```

- **Declared at:** [line 221](../../../runtime/world2d_tiles.hpp#L221)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

object.set_visible(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-set-z-index-1"></a>

## `epok::TileMapLayerComponent::set_z_index`

**Purpose.** Sets z index as part of the world2d tiles module.

**Exact declaration**

```cpp
void set_z_index(int32_t value)
```

- **Declared at:** [line 229](../../../runtime/world2d_tiles.hpp#L229)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

object.set_z_index(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-tile-height-1"></a>

## `epok::TileMapLayerComponent::tile_height`

**Purpose.** Performs `tile height` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint16_t tile_height() const
```

- **Declared at:** [line 158](../../../runtime/world2d_tiles.hpp#L158)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.tile_height();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-tile-of-1"></a>

## `epok::TileMapLayerComponent::tile_of`

**Purpose.** Performs `tile of` as part of the world2d tiles module.

**Exact declaration**

```cpp
const TileInfo* tile_of(uint16_t value) const
```

- **Declared at:** [line 145](../../../runtime/world2d_tiles.hpp#L145)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `const TileInfo *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t value

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.tile_of(value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-tile-width-1"></a>

## `epok::TileMapLayerComponent::tile_width`

**Purpose.** Performs `tile width` as part of the world2d tiles module.

**Exact declaration**

```cpp
uint16_t tile_width() const
```

- **Declared at:** [line 157](../../../runtime/world2d_tiles.hpp#L157)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.tile_width();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-world-to-map-x-1"></a>

## `epok::TileMapLayerComponent::world_to_map_x`

**Purpose.** The cell column containing world X / the cell row containing world Y.

**Exact declaration**

```cpp
int32_t world_to_map_x(Fixed world_x) const
```

- **Declared at:** [line 200](../../../runtime/world2d_tiles.hpp#L200)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world_x` | `Fixed` | Input | Value supplied for `world_x`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed world_x

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.world_to_map_x(world_x);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-world-to-map-y-1"></a>

## `epok::TileMapLayerComponent::world_to_map_y`

**Purpose.** Performs `world to map y` as part of the world2d tiles module.

**Exact declaration**

```cpp
int32_t world_to_map_y(Fixed world_y) const
```

- **Declared at:** [line 205](../../../runtime/world2d_tiles.hpp#L205)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world_y` | `Fixed` | Input | Value supplied for `world_y`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// Fixed world_y

epok::TileMapLayerComponent& object = /* obtain a valid instance */;

auto result = object.world_to_map_y(world_y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tilemaplayercomponent-tilemaplayercomponent-1"></a>

## `epok::TileMapLayerComponent::~TileMapLayerComponent`

**Purpose.** Releases the resources owned by `epok::TileMapLayerComponent`.

**Exact declaration**

```cpp
~TileMapLayerComponent() override
```

- **Declared at:** [line 108](../../../runtime/world2d_tiles.hpp#L108)
- **Kind:** `destructor`

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// `epok::TileMapLayerComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-canvas-of-1"></a>

## `epok::world2d::canvas_of`

**Purpose.** Walks from `component`'s owner up its attachments.

**Details.** The nearest CanvasLayer2D decides the canvas; the nearest Parallax2D below it decides the repetition.

**Exact declaration**

```cpp
inline Canvas canvas_of(ActorComponent& component)
```

- **Declared at:** [line 411](../../../runtime/world2d_tiles.hpp#L411)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `Canvas`. Check the purpose and failure notes before using the value.

**Use it when.** The nearest CanvasLayer2D decides the canvas; the nearest Parallax2D below it decides the repetition.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & component

auto result = epok::world2d::canvas_of(component);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-depth-bucket-1"></a>

## `epok::world2d::depth_bucket`

**Purpose.** The bucket of camera-space depth `z` (Q12), moved by `bias` buckets and clamped to the table: the bucket a 3D triangle averaging that depth falls in.

**Exact declaration**

```cpp
inline int depth_bucket(int32_t z, int32_t bias)
```

- **Declared at:** [line 396](../../../runtime/world2d_tiles.hpp#L396)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `z` | `int32_t` | Input | Value supplied for `z`. See the exact type and module contract. |
| `bias` | `int32_t` | Input | Value supplied for `bias`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t z
// int32_t bias

auto result = epok::world2d::depth_bucket(z, bias);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-depth-of-1"></a>

## `epok::world2d::depth_of`

**Purpose.** The nearest enabled, active Depth2D on `component`'s owner or up its attachments.

**Exact declaration**

```cpp
inline const Depth2DComponent* depth_of(ActorComponent& component)
```

- **Declared at:** [line 439](../../../runtime/world2d_tiles.hpp#L439)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent &` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `const Depth2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent & component

auto result = epok::world2d::depth_of(component);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-floor-cells-1"></a>

## `epok::world2d::floor_cells`

**Purpose.** Floor division of a Q12 distance by a whole number of pixels.

**Exact declaration**

```cpp
inline int32_t floor_cells(int32_t raw, int32_t pixels)
```

- **Declared at:** [line 89](../../../runtime/world2d_tiles.hpp#L89)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `raw` | `int32_t` | Input | Value supplied for `raw`. See the exact type and module contract. |
| `pixels` | `int32_t` | Input | Value supplied for `pixels`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d tiles module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_tiles.hpp"

// Assume these named values have been initialized with valid data:
// int32_t raw
// int32_t pixels

auto result = epok::world2d::floor_cells(raw, pixels);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d tiles module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
