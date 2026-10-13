# Epok API: World2D Viewports

> **Header:** `"world2d_viewports.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_viewports.hpp)

This module covers the world2d viewports module. It documents 23 public callables declared directly in this header.

## Declared types

`epok::SplitScreen2DComponent`, `epok::SubViewport2DComponent`, `epok::world2d::RenderPass`

## Callable index

- [`epok::SplitScreen2DComponent::class_id`](#epok-splitscreen2dcomponent-class-id-1) — Performs `class id` as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::half`](#epok-splitscreen2dcomponent-half-1) — Screen rectangle (x, y, w, h) of half `index`, either side of the divider.
- [`epok::SplitScreen2DComponent::is_enabled`](#epok-splitscreen2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::is_side_by_side`](#epok-splitscreen2dcomponent-is-side-by-side-1) — Reports whether side by side as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::level_attached`](#epok-splitscreen2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::set_enabled`](#epok-splitscreen2dcomponent-set-enabled-1) — Sets enabled as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::set_side_by_side`](#epok-splitscreen2dcomponent-set-side-by-side-1) — Sets side by side as part of the world2d viewports module.
- [`epok::SplitScreen2DComponent::view`](#epok-splitscreen2dcomponent-view-1) — The view through camera `index`, its projection centred on its half.
- [`epok::SplitScreen2DComponent::~SplitScreen2DComponent`](#epok-splitscreen2dcomponent-splitscreen2dcomponent-1) — Releases the resources owned by `epok::SplitScreen2DComponent`.
- [`epok::SubViewport2DComponent::class_id`](#epok-subviewport2dcomponent-class-id-1) — Performs `class id` as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::get_camera`](#epok-subviewport2dcomponent-get-camera-1) — Returns camera as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::is_enabled`](#epok-subviewport2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::level_attached`](#epok-subviewport2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::set_camera`](#epok-subviewport2dcomponent-set-camera-1) — Sets camera as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::set_enabled`](#epok-subviewport2dcomponent-set-enabled-1) — Sets enabled as part of the world2d viewports module.
- [`epok::SubViewport2DComponent::set_interval`](#epok-subviewport2dcomponent-set-interval-1) — Redraw every `frames` frames (1: every frame); a still picture costs nothing between.
- [`epok::SubViewport2DComponent::view`](#epok-subviewport2dcomponent-view-1) — The view this picture is drawn through: the camera's, recentred on the picture.
- [`epok::SubViewport2DComponent::~SubViewport2DComponent`](#epok-subviewport2dcomponent-subviewport2dcomponent-1) — Releases the resources owned by `epok::SubViewport2DComponent`.
- [`epok::world2d::active_split`](#epok-world2d-active-split-1) — The active split screen, when both its cameras exist.
- [`epok::world2d::camera_on`](#epok-world2d-camera-on-1) — Performs `camera on` as part of the world2d viewports module.
- [`epok::world2d::RenderPass::accepts`](#epok-world2d-renderpass-accepts-1) — Performs `accepts` as part of the world2d viewports module.
- [`epok::world2d::RenderPass::hide`](#epok-world2d-renderpass-hide-1) — Performs `hide` as part of the world2d viewports module.
- [`epok::world2d::subviewport_on`](#epok-world2d-subviewport-on-1) — Performs `subviewport on` as part of the world2d viewports module.

<a id="epok-splitscreen2dcomponent-class-id-1"></a>

## `epok::SplitScreen2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d viewports module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 92](../../../runtime/world2d_viewports.hpp#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-splitscreen2dcomponent-half-1"></a>

## `epok::SplitScreen2DComponent::half`

**Purpose.** Screen rectangle (x, y, w, h) of half `index`, either side of the divider.

**Exact declaration**

```cpp
void half(int index, int16_t* rect) const
```

- **Declared at:** [line 107](../../../runtime/world2d_viewports.hpp#L107)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `rect` | `int16_t *` | Input/output; inspect the function contract | Value supplied for `rect`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// int index
// int16_t * rect

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

object.half(index, rect);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-splitscreen2dcomponent-is-enabled-1"></a>

## `epok::SplitScreen2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d viewports module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 103](../../../runtime/world2d_viewports.hpp#L103)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-splitscreen2dcomponent-is-side-by-side-1"></a>

## `epok::SplitScreen2DComponent::is_side_by_side`

**Purpose.** Reports whether side by side as part of the world2d viewports module.

**Exact declaration**

```cpp
bool is_side_by_side() const
```

- **Declared at:** [line 105](../../../runtime/world2d_viewports.hpp#L105)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

auto result = object.is_side_by_side();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-splitscreen2dcomponent-level-attached-1"></a>

## `epok::SplitScreen2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d viewports module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 93](../../../runtime/world2d_viewports.hpp#L93)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-splitscreen2dcomponent-set-enabled-1"></a>

## `epok::SplitScreen2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d viewports module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 102](../../../runtime/world2d_viewports.hpp#L102)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-splitscreen2dcomponent-set-side-by-side-1"></a>

## `epok::SplitScreen2DComponent::set_side_by_side`

**Purpose.** Sets side by side as part of the world2d viewports module.

**Exact declaration**

```cpp
void set_side_by_side(bool value)
```

- **Declared at:** [line 104](../../../runtime/world2d_viewports.hpp#L104)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

object.set_side_by_side(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-splitscreen2dcomponent-view-1"></a>

## `epok::SplitScreen2DComponent::view`

**Purpose.** The view through camera `index`, its projection centred on its half.

**Exact declaration**

```cpp
bool view(int index, Camera2D& out) const
```

- **Declared at:** [line 124](../../../runtime/world2d_viewports.hpp#L124)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |
| `out` | `Camera2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// int index
// Camera2D & out

epok::SplitScreen2DComponent& object = /* obtain a valid instance */;

auto result = object.view(index, out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-splitscreen2dcomponent-splitscreen2dcomponent-1"></a>

## `epok::SplitScreen2DComponent::~SplitScreen2DComponent`

**Purpose.** Releases the resources owned by `epok::SplitScreen2DComponent`.

**Exact declaration**

```cpp
~SplitScreen2DComponent() override
```

- **Declared at:** [line 94](../../../runtime/world2d_viewports.hpp#L94)
- **Kind:** `destructor`

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// `epok::SplitScreen2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-class-id-1"></a>

## `epok::SubViewport2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d viewports module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 56](../../../runtime/world2d_viewports.hpp#L56)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-get-camera-1"></a>

## `epok::SubViewport2DComponent::get_camera`

**Purpose.** Returns camera as part of the world2d viewports module.

**Exact declaration**

```cpp
ObjectId get_camera() const
```

- **Declared at:** [line 79](../../../runtime/world2d_viewports.hpp#L79)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

auto result = object.get_camera();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-is-enabled-1"></a>

## `epok::SubViewport2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d viewports module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 77](../../../runtime/world2d_viewports.hpp#L77)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-subviewport2dcomponent-level-attached-1"></a>

## `epok::SubViewport2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d viewports module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 57](../../../runtime/world2d_viewports.hpp#L57)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-set-camera-1"></a>

## `epok::SubViewport2DComponent::set_camera`

**Purpose.** Sets camera as part of the world2d viewports module.

**Exact declaration**

```cpp
void set_camera(ObjectId actor)
```

- **Declared at:** [line 78](../../../runtime/world2d_viewports.hpp#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

object.set_camera(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-set-enabled-1"></a>

## `epok::SubViewport2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d viewports module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 76](../../../runtime/world2d_viewports.hpp#L76)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-set-interval-1"></a>

## `epok::SubViewport2DComponent::set_interval`

**Purpose.** Redraw every `frames` frames (1: every frame); a still picture costs nothing between.

**Exact declaration**

```cpp
void set_interval(uint32_t frames)
```

- **Declared at:** [line 81](../../../runtime/world2d_viewports.hpp#L81)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frames` | `uint32_t` | Input | Value supplied for `frames`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t frames

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

object.set_interval(frames);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-subviewport2dcomponent-view-1"></a>

## `epok::SubViewport2DComponent::view`

**Purpose.** The view this picture is drawn through: the camera's, recentred on the picture.

**Exact declaration**

```cpp
bool view(Camera2D& out) const
```

- **Declared at:** [line 85](../../../runtime/world2d_viewports.hpp#L85)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `out` | `Camera2D &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// Camera2D & out

epok::SubViewport2DComponent& object = /* obtain a valid instance */;

auto result = object.view(out);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-subviewport2dcomponent-subviewport2dcomponent-1"></a>

## `epok::SubViewport2DComponent::~SubViewport2DComponent`

**Purpose.** Releases the resources owned by `epok::SubViewport2DComponent`.

**Exact declaration**

```cpp
~SubViewport2DComponent() override
```

- **Declared at:** [line 58](../../../runtime/world2d_viewports.hpp#L58)
- **Kind:** `destructor`

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// `epok::SubViewport2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-active-split-1"></a>

## `epok::world2d::active_split`

**Purpose.** The active split screen, when both its cameras exist.

**Exact declaration**

```cpp
inline SplitScreen2DComponent* active_split()
```

- **Declared at:** [line 139](../../../runtime/world2d_viewports.hpp#L139)
- **Kind:** `function decl`

**Returns.** Returns `SplitScreen2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

auto result = epok::world2d::active_split();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-camera-on-1"></a>

## `epok::world2d::camera_on`

**Purpose.** Performs `camera on` as part of the world2d viewports module.

**Exact declaration**

```cpp
inline Camera2DComponent* camera_on(ObjectId actor)
```

- **Declared at:** [line 128](../../../runtime/world2d_viewports.hpp#L128)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `Camera2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::world2d::camera_on(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-world2d-renderpass-accepts-1"></a>

## `epok::world2d::RenderPass::accepts`

**Purpose.** Performs `accepts` as part of the world2d viewports module.

**Exact declaration**

```cpp
bool accepts(const Canvas& canvas) const
```

- **Declared at:** [line 38](../../../runtime/world2d_viewports.hpp#L38)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `canvas` | `const Canvas &` | Input | Value supplied for `canvas`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// const Canvas & canvas

epok::world2d::RenderPass& object = /* obtain a valid instance */;

auto result = object.accepts(canvas);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-world2d-renderpass-hide-1"></a>

## `epok::world2d::RenderPass::hide`

**Purpose.** Performs `hide` as part of the world2d viewports module.

**Exact declaration**

```cpp
void hide(int8_t layer)
```

- **Declared at:** [line 45](../../../runtime/world2d_viewports.hpp#L45)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `layer` | `int8_t` | Input | Value supplied for `layer`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// int8_t layer

epok::world2d::RenderPass& object = /* obtain a valid instance */;

object.hide(layer);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-world2d-subviewport-on-1"></a>

## `epok::world2d::subviewport_on`

**Purpose.** Performs `subviewport on` as part of the world2d viewports module.

**Exact declaration**

```cpp
inline SubViewport2DComponent* subviewport_on(ObjectId actor)
```

- **Declared at:** [line 133](../../../runtime/world2d_viewports.hpp#L133)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `SubViewport2DComponent *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d viewports module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_viewports.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::world2d::subviewport_on(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d viewports module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
