# Epok API: Retained

> **Header:** `"retained.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/retained.hpp)

This module covers the retained module. It documents 19 public callables declared directly in this header.

## Declared types

`epok::FramePacketSlots`, `epok::MeshMaterialCache`, `epok::RetainedGeometry`, `epok::RetainedGeometry::State`, `epok::RetainedKey`, `epok::RetainedQuad`, `epok::RetainedStats`

## Callable index

- [`epok::FramePacketSlots::allocate_dynamic`](#epok-framepacketslots-allocate-dynamic-1) — Performs `allocate dynamic` as part of the retained module.
- [`epok::FramePacketSlots::allocate_dynamic_pair`](#epok-framepacketslots-allocate-dynamic-pair-1) — GT4/G4 need 52/36 bytes including DMA, versus 40/28 for GT3/G3.
- [`epok::FramePacketSlots::begin_frame`](#epok-framepacketslots-begin-frame-1) — Begins frame as part of the retained module.
- [`epok::FramePacketSlots::claim`](#epok-framepacketslots-claim-1) — Performs `claim` as part of the retained module.
- [`epok::FramePacketSlots::dynamic_low`](#epok-framepacketslots-dynamic-low-1) — Performs `dynamic low` as part of the retained module.
- [`epok::FramePacketSlots::run_floor`](#epok-framepacketslots-run-floor-1) — A pass emitting many packets may bump through the current free run [run_floor(), dynamic_low()) itself, exactly as allocate_dynamic(_pair) would, and store its new low end before any other allocation.
- [`epok::FramePacketSlots::set_dynamic_low`](#epok-framepacketslots-set-dynamic-low-1) — Sets dynamic low as part of the retained module.
- [`epok::mesh_faces_unlit`](#epok-mesh-faces-unlit-1) — An imported editable mesh owns per-face materials.
- [`epok::MeshMaterialCache::faces_unlit`](#epok-meshmaterialcache-faces-unlit-1) — Performs `faces unlit` as part of the retained module.
- [`epok::retain_mesh_packets`](#epok-retain-mesh-packets-1) — Performs `retain mesh packets` as part of the retained module.
- [`epok::RetainedGeometry::allocate`](#epok-retainedgeometry-allocate-1) — `limit_slot` is the packet-buffer capacity.
- [`epok::RetainedGeometry::first_slot`](#epok-retainedgeometry-first-slot-1) — Performs `first slot` as part of the retained module.
- [`epok::RetainedGeometry::forget`](#epok-retainedgeometry-forget-1) — A reused entity slot keeps its quads; a different geometry reallocates.
- [`epok::RetainedGeometry::invalidate_borrowed`](#epok-retainedgeometry-invalidate-borrowed-1) — Dynamic writes can replace dormant cached packets.
- [`epok::RetainedGeometry::quads`](#epok-retainedgeometry-quads-1) — Performs `quads` as part of the retained module.
- [`epok::RetainedGeometry::reset`](#epok-retainedgeometry-reset-1) — Resets reset as part of the retained module.
- [`epok::RetainedGeometry::slot_top`](#epok-retainedgeometry-slot-top-1) — First preferred fragment slot beyond the cache (not a per-frame reservation).
- [`epok::RetainedGeometry::state`](#epok-retainedgeometry-state-1) — Performs `state` as part of the retained module.
- [`epok::RetainedKey::operator==`](#epok-retainedkey-operator-1) — Performs `operator ==` as part of the retained module.

<a id="epok-framepacketslots-allocate-dynamic-1"></a>

## `epok::FramePacketSlots::allocate_dynamic`

**Purpose.** Performs `allocate dynamic` as part of the retained module.

**Exact declaration**

```cpp
uint32_t allocate_dynamic()
```

- **Declared at:** [line 102](../../../runtime/retained.hpp#L102)
- **Kind:** `cxx method`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::FramePacketSlots& object = /* obtain a valid instance */;

auto result = object.allocate_dynamic();
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-allocate-dynamic-pair-1"></a>

## `epok::FramePacketSlots::allocate_dynamic_pair`

**Purpose.** GT4/G4 need 52/36 bytes including DMA, versus 40/28 for GT3/G3.

**Details.** Reserve two ADJACENT slots atomically before writing a larger packet. Two independent allocate_dynamic() calls could straddle a retained range.

**Exact declaration**

```cpp
uint32_t allocate_dynamic_pair()
```

- **Declared at:** [line 111](../../../runtime/retained.hpp#L111)
- **Kind:** `cxx method`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Reserve two ADJACENT slots atomically before writing a larger packet. Two independent allocate_dynamic() calls could straddle a retained range.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::FramePacketSlots& object = /* obtain a valid instance */;

auto result = object.allocate_dynamic_pair();
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-begin-frame-1"></a>

## `epok::FramePacketSlots::begin_frame`

**Purpose.** Begins frame as part of the retained module.

**Exact declaration**

```cpp
void begin_frame()
```

- **Declared at:** [line 86](../../../runtime/retained.hpp#L86)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::FramePacketSlots& object = /* obtain a valid instance */;

object.begin_frame();
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-claim-1"></a>

## `epok::FramePacketSlots::claim`

**Purpose.** Performs `claim` as part of the retained module.

**Exact declaration**

```cpp
bool claim(uint32_t first,uint32_t count)
```

- **Declared at:** [line 93](../../../runtime/retained.hpp#L93)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t first
// uint32_t count

epok::FramePacketSlots& object = /* obtain a valid instance */;

auto result = object.claim(first, count);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-dynamic-low-1"></a>

## `epok::FramePacketSlots::dynamic_low`

**Purpose.** Performs `dynamic low` as part of the retained module.

**Exact declaration**

```cpp
uint32_t dynamic_low() const
```

- **Declared at:** [line 87](../../../runtime/retained.hpp#L87)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::FramePacketSlots& object = /* obtain a valid instance */;

auto result = object.dynamic_low();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-run-floor-1"></a>

## `epok::FramePacketSlots::run_floor`

**Purpose.** A pass emitting many packets may bump through the current free run [run_floor(), dynamic_low()) itself, exactly as allocate_dynamic(_pair) would, and store its new low end before any other allocation.

**Exact declaration**

```cpp
uint32_t run_floor() const
```

- **Declared at:** [line 91](../../../runtime/retained.hpp#L91)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::FramePacketSlots& object = /* obtain a valid instance */;

auto result = object.run_floor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-framepacketslots-set-dynamic-low-1"></a>

## `epok::FramePacketSlots::set_dynamic_low`

**Purpose.** Sets dynamic low as part of the retained module.

**Exact declaration**

```cpp
void set_dynamic_low(uint32_t low)
```

- **Declared at:** [line 92](../../../runtime/retained.hpp#L92)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `low` | `uint32_t` | Input | Value supplied for `low`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t low

epok::FramePacketSlots& object = /* obtain a valid instance */;

object.set_dynamic_low(low);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-mesh-faces-unlit-1"></a>

## `epok::mesh_faces_unlit`

**Purpose.** An imported editable mesh owns per-face materials.

**Details.** The actor tint can still have its default lit flag even when every face explicitly opts out.

**Exact declaration**

```cpp
inline bool mesh_faces_unlit(const MeshGeometry* geometry)
```

- **Declared at:** [line 20](../../../runtime/retained.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The actor tint can still have its default lit flag even when every face explicitly opts out.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * geometry

auto result = epok::mesh_faces_unlit(geometry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshmaterialcache-faces-unlit-1"></a>

## `epok::MeshMaterialCache::faces_unlit`

**Purpose.** Performs `faces unlit` as part of the retained module.

**Exact declaration**

```cpp
bool faces_unlit(const MeshGeometry* source)
```

- **Declared at:** [line 33](../../../runtime/retained.hpp#L33)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const MeshGeometry *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * source

epok::MeshMaterialCache& object = /* obtain a valid instance */;

auto result = object.faces_unlit(source);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-retain-mesh-packets-1"></a>

## `epok::retain_mesh_packets`

**Purpose.** Performs `retain mesh packets` as part of the retained module.

**Exact declaration**

```cpp
inline bool retain_mesh_packets(bool skeletal,bool object_lit,bool baked_colors)
```

- **Declared at:** [line 38](../../../runtime/retained.hpp#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `skeletal` | `bool` | Input | Value supplied for `skeletal`. See the exact type and module contract. |
| `object_lit` | `bool` | Input | Value supplied for `object_lit`. See the exact type and module contract. |
| `baked_colors` | `bool` | Input | Value supplied for `baked_colors`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// bool skeletal
// bool object_lit
// bool baked_colors

auto result = epok::retain_mesh_packets(skeletal, object_lit, baked_colors);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-allocate-1"></a>

## `epok::RetainedGeometry::allocate`

**Purpose.** `limit_slot` is the packet-buffer capacity.

**Details.** Ranges describe cache addresses; FramePacketSlots separately arbitrates their use in the current frame. Freed metadata ranges are not reclaimed until reset().

**Exact declaration**

```cpp
bool allocate(size_t index,const MeshGeometry* geometry,size_t quads,uint32_t limit_slot)
```

- **Declared at:** [line 163](../../../runtime/retained.hpp#L163)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `geometry` | `const MeshGeometry *` | Input | Value supplied for `geometry`. See the exact type and module contract. |
| `quads` | `size_t` | Input | Value supplied for `quads`. See the exact type and module contract. |
| `limit_slot` | `uint32_t` | Input | Value supplied for `limit_slot`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Ranges describe cache addresses; FramePacketSlots separately arbitrates their use in the current frame. Freed metadata ranges are not reclaimed until reset().

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// const MeshGeometry * geometry
// size_t quads
// uint32_t limit_slot

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.allocate(index, geometry, quads, limit_slot);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-retainedgeometry-first-slot-1"></a>

## `epok::RetainedGeometry::first_slot`

**Purpose.** Performs `first slot` as part of the retained module.

**Exact declaration**

```cpp
uint32_t first_slot(size_t index) const
```

- **Declared at:** [line 182](../../../runtime/retained.hpp#L182)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.first_slot(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-forget-1"></a>

## `epok::RetainedGeometry::forget`

**Purpose.** A reused entity slot keeps its quads; a different geometry reallocates.

**Exact declaration**

```cpp
void forget(size_t index)
```

- **Declared at:** [line 157](../../../runtime/retained.hpp#L157)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::RetainedGeometry& object = /* obtain a valid instance */;

object.forget(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-invalidate-borrowed-1"></a>

## `epok::RetainedGeometry::invalidate_borrowed`

**Purpose.** Dynamic writes can replace dormant cached packets.

**Details.** Invalidate their keys after rendering, even for invisible objects; leave the other parity alone. The descending allocator's low watermark is conservative across claimed gaps.

**Exact declaration**

```cpp
void invalidate_borrowed(unsigned parity,uint32_t first_slot)
```

- **Declared at:** [line 175](../../../runtime/retained.hpp#L175)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `parity` | `unsigned int` | Input | Value supplied for `parity`. See the exact type and module contract. |
| `first_slot` | `uint32_t` | Input | Value supplied for `first_slot`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Invalidate their keys after rendering, even for invisible objects; leave the other parity alone. The descending allocator's low watermark is conservative across claimed gaps.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int parity
// uint32_t first_slot

epok::RetainedGeometry& object = /* obtain a valid instance */;

object.invalidate_borrowed(parity, first_slot);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-quads-1"></a>

## `epok::RetainedGeometry::quads`

**Purpose.** Performs `quads` as part of the retained module.

**Exact declaration**

```cpp
RetainedQuad* quads(size_t index)
```

- **Declared at:** [line 181](../../../runtime/retained.hpp#L181)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `RetainedQuad *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.quads(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-reset-1"></a>

## `epok::RetainedGeometry::reset`

**Purpose.** Resets reset as part of the retained module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 155](../../../runtime/retained.hpp#L155)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::RetainedGeometry& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-slot-top-1"></a>

## `epok::RetainedGeometry::slot_top`

**Purpose.** First preferred fragment slot beyond the cache (not a per-frame reservation).

**Exact declaration**

```cpp
uint32_t slot_top() const
```

- **Declared at:** [line 159](../../../runtime/retained.hpp#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.slot_top();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedgeometry-state-1"></a>

## `epok::RetainedGeometry::state`

**Purpose.** Performs `state` as part of the retained module.

**Exact declaration**

```cpp
State& state(size_t index)
```

- **Declared at:** [line 180](../../../runtime/retained.hpp#L180)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `State &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::RetainedGeometry& object = /* obtain a valid instance */;

auto result = object.state(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the retained module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-retainedkey-operator-1"></a>

## `epok::RetainedKey::operator==`

**Purpose.** Performs `operator ==` as part of the retained module.

**Exact declaration**

```cpp
bool operator==(const RetainedKey& o) const
```

- **Declared at:** [line 47](../../../runtime/retained.hpp#L47)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `o` | `const RetainedKey &` | Input | Value supplied for `o`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the retained module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "retained.hpp"

// Assume these named values have been initialized with valid data:
// const RetainedKey & o

epok::RetainedKey& object = /* obtain a valid instance */;

auto result = object.operator==(o);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
