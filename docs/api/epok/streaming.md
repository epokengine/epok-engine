# Epok API: Streaming

> **Header:** `"streaming.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/streaming.hpp)

This module covers bounded CD or PC geometry-page streaming. It documents 34 public callables declared directly in this header.

## Declared types

`epok::StreamingStats`, `epok::StreamingWarmupStats`, `epok::StreamMeshLease`, `epok::StreamMeshView`, `epok::StreamObjectBinding`, `epok::StreamPageCursor`

## Callable index

- [`epok::streaming_acquire`](#epok-streaming-acquire-1) — Performs `streaming acquire` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_acquire_slot`](#epok-streaming-acquire-slot-1) — Performs `streaming acquire slot` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_acquire_slot_slow`](#epok-streaming-acquire-slot-slow-1) — Performs `streaming acquire slot slow` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_bind_chain`](#epok-streaming-bind-chain-1) — Performs `streaming bind chain` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_bind_object`](#epok-streaming-bind-object-1) — Performs `streaming bind object` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_descriptor_valid`](#epok-streaming-descriptor-valid-1) — Exported descriptors are immutable.
- [`epok::streaming_lookup`](#epok-streaming-lookup-1) — Performs `streaming lookup` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_prefetch`](#epok-streaming-prefetch-1) — Opportunistic only: XA playback and a pending XA request retain priority.
- [`epok::streaming_prefetch_needed`](#epok-streaming-prefetch-needed-1) — Performs `streaming prefetch needed` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_prepare`](#epok-streaming-prepare-1) — Performs `streaming prepare` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_release`](#epok-streaming-release-1) — Performs `streaming release` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_resolve_archive_mesh`](#epok-streaming-resolve-archive-mesh-1) — Performs `streaming resolve archive mesh` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_resolve_stable`](#epok-streaming-resolve-stable-1) — Only whole-archive pools permit unpinned payload views.
- [`epok::streaming_scene_changed`](#epok-streaming-scene-changed-1) — Performs `streaming scene changed` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_start_read`](#epok-streaming-start-read-1) — Performs `streaming start read` as part of bounded CD or PC geometry-page streaming.
- [`epok::streaming_tick`](#epok-streaming-tick-1) — Release the controller only after every parser/read/XA callback has drained.
- [`epok::streaming_warmup`](#epok-streaming-warmup-1) — The caller selects a startup working set from resident metadata.
- [`epok::streaming_warmup_scene`](#epok-streaming-warmup-scene-1) — A bounded small-scene startup policy.
- [`epok::StreamMeshLease::geometry`](#epok-streammeshlease-geometry-1) — Compatibility accessor for users needing a full rebound descriptor.
- [`epok::StreamMeshLease::operator=`](#epok-streammeshlease-operator-1) — Performs `operator =` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::quads`](#epok-streammeshlease-quads-1) — Performs `quads` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::StreamMeshLease`](#epok-streammeshlease-streammeshlease-1) — Constructs `epok::StreamMeshLease` for bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::StreamMeshLease`](#epok-streammeshlease-streammeshlease-2) — Constructs `epok::StreamMeshLease` for bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::valid`](#epok-streammeshlease-valid-1) — Performs `valid` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::vertices`](#epok-streammeshlease-vertices-1) — Performs `vertices` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamMeshLease::~StreamMeshLease`](#epok-streammeshlease-streammeshlease-3) — Releases the resources owned by `epok::StreamMeshLease`.
- [`epok::StreamPageCursor::borrow`](#epok-streampagecursor-borrow-1) — Performs `borrow` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::operator=`](#epok-streampagecursor-operator-1) — Performs `operator =` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::release`](#epok-streampagecursor-release-1) — Performs `release` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::release_borrow`](#epok-streampagecursor-release-borrow-1) — Performs `release borrow` as part of bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::resolve`](#epok-streampagecursor-resolve-1) — Renderer-only sequential view: consume all payload data before the next resolve/advance/release.
- [`epok::StreamPageCursor::StreamPageCursor`](#epok-streampagecursor-streampagecursor-1) — Constructs `epok::StreamPageCursor` for bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::StreamPageCursor`](#epok-streampagecursor-streampagecursor-2) — Constructs `epok::StreamPageCursor` for bounded CD or PC geometry-page streaming.
- [`epok::StreamPageCursor::~StreamPageCursor`](#epok-streampagecursor-streampagecursor-3) — Releases the resources owned by `epok::StreamPageCursor`.

<a id="epok-streaming-acquire-1"></a>

## `epok::streaming_acquire`

**Purpose.** Performs `streaming acquire` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline const uint8_t *streaming_acquire(uint32_t page, psyqo::GPU &gpu)
```

- **Declared at:** [line 191](../../../runtime/streaming.hpp#L191)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// psyqo::GPU & gpu

auto result = epok::streaming_acquire(page, gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-acquire-slot-1"></a>

## `epok::streaming_acquire_slot`

**Purpose.** Performs `streaming acquire slot` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline int streaming_acquire_slot(uint32_t page, psyqo::GPU &gpu)
```

- **Declared at:** [line 185](../../../runtime/streaming.hpp#L185)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// psyqo::GPU & gpu

auto result = epok::streaming_acquire_slot(page, gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-acquire-slot-slow-1"></a>

## `epok::streaming_acquire_slot_slow`

**Purpose.** Performs `streaming acquire slot slow` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline int streaming_acquire_slot_slow(uint32_t page, psyqo::GPU &gpu)
```

- **Declared at:** [line 146](../../../runtime/streaming.hpp#L146)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// psyqo::GPU & gpu

auto result = epok::streaming_acquire_slot_slow(page, gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-bind-chain-1"></a>

## `epok::streaming_bind_chain`

**Purpose.** Performs `streaming bind chain` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline bool streaming_bind_chain(const MeshGeometry *root, StreamObjectBinding &binding, psyqo::GPU &gpu)
```

- **Declared at:** [line 339](../../../runtime/streaming.hpp#L339)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `const MeshGeometry *` | Input | Value supplied for `root`. See the exact type and module contract. |
| `binding` | `StreamObjectBinding &` | Input/output; inspect the function contract | Value supplied for `binding`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * root
// StreamObjectBinding & binding
// psyqo::GPU & gpu

auto result = epok::streaming_bind_chain(root, binding, gpu);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-bind-object-1"></a>

## `epok::streaming_bind_object`

**Purpose.** Performs `streaming bind object` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline bool streaming_bind_object(const MeshGeometry *root, StreamObjectBinding &binding, psyqo::GPU &gpu)
```

- **Declared at:** [line 357](../../../runtime/streaming.hpp#L357)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `const MeshGeometry *` | Input | Value supplied for `root`. See the exact type and module contract. |
| `binding` | `StreamObjectBinding &` | Input/output; inspect the function contract | Value supplied for `binding`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry * root
// StreamObjectBinding & binding
// psyqo::GPU & gpu

auto result = epok::streaming_bind_object(root, binding, gpu);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-descriptor-valid-1"></a>

## `epok::streaming_descriptor_valid`

**Purpose.** Exported descriptors are immutable.

**Details.** A renderer may validate their whole geometry chain once, then retain that result until the chain changes. Arbitrary callers still get these checks from the default lease constructor.

**Exact declaration**

```cpp
constexpr bool streaming_descriptor_valid(const MeshGeometry &mesh)
```

- **Declared at:** [line 288](../../../runtime/streaming.hpp#L288)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mesh` | `const MeshGeometry &` | Input | Value supplied for `mesh`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** A renderer may validate their whole geometry chain once, then retain that result until the chain changes. Arbitrary callers still get these checks from the default lease constructor.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry & mesh

auto result = epok::streaming_descriptor_valid(mesh);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-lookup-1"></a>

## `epok::streaming_lookup`

**Purpose.** Performs `streaming lookup` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline void streaming_lookup()
```

- **Declared at:** [line 58](../../../runtime/streaming.hpp#L58)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::streaming_lookup();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streaming-prefetch-1"></a>

## `epok::streaming_prefetch`

**Purpose.** Opportunistic only: XA playback and a pending XA request retain priority.

**Details.** Call after acquisition while its page is pinned to protect current geometry.

**Exact declaration**

```cpp
inline bool streaming_prefetch(uint32_t page)
```

- **Declared at:** [line 124](../../../runtime/streaming.hpp#L124)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Call after acquisition while its page is pinned to protect current geometry.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

auto result = epok::streaming_prefetch(page);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streaming-prefetch-needed-1"></a>

## `epok::streaming_prefetch_needed`

**Purpose.** Performs `streaming prefetch needed` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline bool streaming_prefetch_needed()
```

- **Declared at:** [line 39](../../../runtime/streaming.hpp#L39)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

auto result = epok::streaming_prefetch_needed();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streaming-prepare-1"></a>

## `epok::streaming_prepare`

**Purpose.** Performs `streaming prepare` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline void streaming_prepare()
```

- **Declared at:** [line 43](../../../runtime/streaming.hpp#L43)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::streaming_prepare();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streaming-release-1"></a>

## `epok::streaming_release`

**Purpose.** Performs `streaming release` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline void streaming_release(uint32_t page)
```

- **Declared at:** [line 194](../../../runtime/streaming.hpp#L194)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::streaming_release(page);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streaming-resolve-archive-mesh-1"></a>

## `epok::streaming_resolve_archive_mesh`

**Purpose.** Performs `streaming resolve archive mesh` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline StreamMeshView streaming_resolve_archive_mesh(const MeshGeometry &mesh, psyqo::GPU &gpu, bool metadata_validated)
```

- **Declared at:** [line 312](../../../runtime/streaming.hpp#L312)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mesh` | `const MeshGeometry &` | Input | Value supplied for `mesh`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `metadata_validated` | `bool` | Input | Value supplied for `metadata_validated`. See the exact type and module contract. |

**Returns.** Returns `StreamMeshView`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry & mesh
// psyqo::GPU & gpu
// bool metadata_validated

auto result = epok::streaming_resolve_archive_mesh(mesh, gpu, metadata_validated);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-resolve-stable-1"></a>

## `epok::streaming_resolve_stable`

**Purpose.** Only whole-archive pools permit unpinned payload views.

**Details.** FixedSlots enforces immutable page identity even for reservations made outside this backend. Failures remain global: a cached pointer must not bypass a later read error.

**Exact declaration**

```cpp
inline const uint8_t *streaming_resolve_stable(uint32_t page, psyqo::GPU &gpu)
```

- **Declared at:** [line 199](../../../runtime/streaming.hpp#L199)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** FixedSlots enforces immutable page identity even for reservations made outside this backend. Failures remain global: a cached pointer must not bypass a later read error.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// psyqo::GPU & gpu

auto result = epok::streaming_resolve_stable(page, gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-scene-changed-1"></a>

## `epok::streaming_scene_changed`

**Purpose.** Performs `streaming scene changed` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline void streaming_scene_changed()
```

- **Declared at:** [line 38](../../../runtime/streaming.hpp#L38)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::streaming_scene_changed();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streaming-start-read-1"></a>

## `epok::streaming_start_read`

**Purpose.** Performs `streaming start read` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
inline bool streaming_start_read(uint32_t page)
```

- **Declared at:** [line 82](../../../runtime/streaming.hpp#L82)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

auto result = epok::streaming_start_read(page);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streaming-tick-1"></a>

## `epok::streaming_tick`

**Purpose.** Release the controller only after every parser/read/XA callback has drained.

**Details.** In particular a timeout must never release or reuse an active DMA buffer.

**Exact declaration**

```cpp
inline void streaming_tick()
```

- **Declared at:** [line 51](../../../runtime/streaming.hpp#L51)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** In particular a timeout must never release or reuse an active DMA buffer.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::streaming_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streaming-warmup-1"></a>

## `epok::streaming_warmup`

**Purpose.** The caller selects a startup working set from resident metadata.

**Details.** Validate the entire set before any I/O. A set that cannot fit is rejected, rather than repeatedly evicting pages and calling that work a warmup. All required pages remain pinned until the batch completes. Demand counters retain the real I/O cost; these separate counters identify startup work.

**Exact declaration**

```cpp
inline bool streaming_warmup(const uint32_t *pages, size_t count, psyqo::GPU &gpu)
```

- **Declared at:** [line 219](../../../runtime/streaming.hpp#L219)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pages` | `const uint32_t *` | Input | Value supplied for `pages`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Validate the entire set before any I/O. A set that cannot fit is rejected, rather than repeatedly evicting pages and calling that work a warmup. All required pages remain pinned until the batch completes. Demand counters retain the real I/O cost; these separate counters identify startup work.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const uint32_t * pages
// size_t count
// psyqo::GPU & gpu

auto result = epok::streaming_warmup(pages, count, gpu);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streaming-warmup-scene-1"></a>

## `epok::streaming_warmup_scene`

**Purpose.** A bounded small-scene startup policy.

**Details.** Include every active object's page so camera/transform changes in the first script frame cannot invalidate a purely camera-based warmup. Large active sets are rejected before reading anything. The caller supplies its normal hierarchy-aware activity predicate.

**Exact declaration**

```cpp
template <class Entity, class Active> inline bool streaming_warmup_scene(const Entity *objects, size_t count, Active &&active, psyqo::GPU &gpu)
```

- **Declared at:** [line 261](../../../runtime/streaming.hpp#L261)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Entity *` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `active` | `Active &&` | Consumed or moved input | Value supplied for `active`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Include every active object's page so camera/transform changes in the first script frame cannot invalidate a purely camera-based warmup. Large active sets are rejected before reading anything. The caller supplies its normal hierarchy-aware activity predicate.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Entity, Active

// Assume these named values have been initialized with valid data:
// const Entity * objects
// size_t count
// Active && active
// psyqo::GPU & gpu

auto result = epok::streaming_warmup_scene<Entity, Active>(objects, count, active, gpu);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streammeshlease-geometry-1"></a>

## `epok::StreamMeshLease::geometry`

**Purpose.** Compatibility accessor for users needing a full rebound descriptor.

**Details.** The renderer can use valid()/vertices()/quads() and retain its original metadata reference, so the normal resident-page path copies no MeshGeometry at all.

**Exact declaration**

```cpp
const MeshGeometry *geometry() const
```

- **Declared at:** [line 488](../../../runtime/streaming.hpp#L488)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const MeshGeometry *`. Check the purpose and failure notes before using the value.

**Use it when.** The renderer can use valid()/vertices()/quads() and retain its original metadata reference, so the normal resident-page path copies no MeshGeometry at all.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamMeshLease& object = /* obtain a valid instance */;

auto result = object.geometry();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streammeshlease-operator-1"></a>

## `epok::StreamMeshLease::operator=`

**Purpose.** Performs `operator =` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamMeshLease &operator=(const StreamMeshLease &) = delete
```

- **Declared at:** [line 477](../../../runtime/streaming.hpp#L477)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const StreamMeshLease &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `StreamMeshLease &`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const StreamMeshLease & arg1

epok::StreamMeshLease& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streammeshlease-quads-1"></a>

## `epok::StreamMeshLease::quads`

**Purpose.** Performs `quads` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
const MeshQuad *quads() const
```

- **Declared at:** [line 482](../../../runtime/streaming.hpp#L482)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const MeshQuad *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamMeshLease& object = /* obtain a valid instance */;

auto result = object.quads();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streammeshlease-streammeshlease-1"></a>

## `epok::StreamMeshLease::StreamMeshLease`

**Purpose.** Constructs `epok::StreamMeshLease` for bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamMeshLease(const MeshGeometry &mesh, psyqo::GPU &gpu, StreamPageCursor *cursor = nullptr, bool metadata_validated = false)
```

- **Declared at:** [line 441](../../../runtime/streaming.hpp#L441)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mesh` | `const MeshGeometry &` | Input | Value supplied for `mesh`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `cursor` | `StreamPageCursor *` | Input/output; inspect the function contract | Value supplied for `cursor`. See the exact type and module contract. |
| `metadata_validated` | `bool` | Input | Value supplied for `metadata_validated`. See the exact type and module contract. |

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry & mesh
// psyqo::GPU & gpu
// StreamPageCursor * cursor
// bool metadata_validated

epok::StreamMeshLease value(mesh, gpu, cursor, metadata_validated);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streammeshlease-streammeshlease-2"></a>

## `epok::StreamMeshLease::StreamMeshLease`

**Purpose.** Constructs `epok::StreamMeshLease` for bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamMeshLease(const StreamMeshLease &) = delete
```

- **Declared at:** [line 476](../../../runtime/streaming.hpp#L476)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const StreamMeshLease &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const StreamMeshLease & arg1

epok::StreamMeshLease value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streammeshlease-valid-1"></a>

## `epok::StreamMeshLease::valid`

**Purpose.** Performs `valid` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool valid() const
```

- **Declared at:** [line 478](../../../runtime/streaming.hpp#L478)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamMeshLease& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streammeshlease-vertices-1"></a>

## `epok::StreamMeshLease::vertices`

**Purpose.** Performs `vertices` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
auto vertices() const -> const int16_t (*)[3]
```

- **Declared at:** [line 479](../../../runtime/streaming.hpp#L479)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const int16_t (*)[3]`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamMeshLease& object = /* obtain a valid instance */;

auto result = object.vertices();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streammeshlease-streammeshlease-3"></a>

## `epok::StreamMeshLease::~StreamMeshLease`

**Purpose.** Releases the resources owned by `epok::StreamMeshLease`.

**Exact declaration**

```cpp
~StreamMeshLease()
```

- **Declared at:** [line 471](../../../runtime/streaming.hpp#L471)
- **Kind:** `destructor`

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// `epok::StreamMeshLease` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streampagecursor-borrow-1"></a>

## `epok::StreamPageCursor::borrow`

**Purpose.** Performs `borrow` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
const uint8_t *borrow(uint32_t page, psyqo::GPU &gpu)
```

- **Declared at:** [line 395](../../../runtime/streaming.hpp#L395)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `const uint8_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page
// psyqo::GPU & gpu

epok::StreamPageCursor& object = /* obtain a valid instance */;

auto result = object.borrow(page, gpu);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streampagecursor-operator-1"></a>

## `epok::StreamPageCursor::operator=`

**Purpose.** Performs `operator =` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamPageCursor &operator=(const StreamPageCursor &) = delete
```

- **Declared at:** [line 384](../../../runtime/streaming.hpp#L384)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const StreamPageCursor &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `StreamPageCursor &`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const StreamPageCursor & arg1

epok::StreamPageCursor& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streampagecursor-release-1"></a>

## `epok::StreamPageCursor::release`

**Purpose.** Performs `release` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool release()
```

- **Declared at:** [line 389](../../../runtime/streaming.hpp#L389)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamPageCursor& object = /* obtain a valid instance */;

auto result = object.release();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streampagecursor-release-borrow-1"></a>

## `epok::StreamPageCursor::release_borrow`

**Purpose.** Performs `release borrow` as part of bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
bool release_borrow(uint32_t page)
```

- **Declared at:** [line 400](../../../runtime/streaming.hpp#L400)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `page` | `uint32_t` | Input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t page

epok::StreamPageCursor& object = /* obtain a valid instance */;

auto result = object.release_borrow(page);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-streampagecursor-resolve-1"></a>

## `epok::StreamPageCursor::resolve`

**Purpose.** Renderer-only sequential view: consume all payload data before the next resolve/advance/release.

**Details.** Its pointers expire when the held page changes. Unlike a public lease this raw view owns no borrow counter or destructor. Outstanding public leases still prevent advancement to another page.

**Exact declaration**

```cpp
StreamMeshView resolve(const MeshGeometry &mesh, psyqo::GPU &gpu, bool metadata_validated = false)
```

- **Declared at:** [line 409](../../../runtime/streaming.hpp#L409)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mesh` | `const MeshGeometry &` | Input | Value supplied for `mesh`. See the exact type and module contract. |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `metadata_validated` | `bool` | Input | Value supplied for `metadata_validated`. See the exact type and module contract. |

**Returns.** Returns `StreamMeshView`. Check the purpose and failure notes before using the value.

**Use it when.** Its pointers expire when the held page changes. Unlike a public lease this raw view owns no borrow counter or destructor. Outstanding public leases still prevent advancement to another page.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const MeshGeometry & mesh
// psyqo::GPU & gpu
// bool metadata_validated

epok::StreamPageCursor& object = /* obtain a valid instance */;

auto result = object.resolve(mesh, gpu, metadata_validated);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streampagecursor-streampagecursor-1"></a>

## `epok::StreamPageCursor::StreamPageCursor`

**Purpose.** Constructs `epok::StreamPageCursor` for bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamPageCursor() = default
```

- **Declared at:** [line 382](../../../runtime/streaming.hpp#L382)
- **Kind:** `constructor`

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

epok::StreamPageCursor value();
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-streampagecursor-streampagecursor-2"></a>

## `epok::StreamPageCursor::StreamPageCursor`

**Purpose.** Constructs `epok::StreamPageCursor` for bounded CD or PC geometry-page streaming.

**Exact declaration**

```cpp
StreamPageCursor(const StreamPageCursor &) = delete
```

- **Declared at:** [line 383](../../../runtime/streaming.hpp#L383)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const StreamPageCursor &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// Assume these named values have been initialized with valid data:
// const StreamPageCursor & arg1

epok::StreamPageCursor value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-streampagecursor-streampagecursor-3"></a>

## `epok::StreamPageCursor::~StreamPageCursor`

**Purpose.** Releases the resources owned by `epok::StreamPageCursor`.

**Exact declaration**

```cpp
~StreamPageCursor()
```

- **Declared at:** [line 385](../../../runtime/streaming.hpp#L385)
- **Kind:** `destructor`

**Use it when.** You need bounded CD or PC geometry-page streaming and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "streaming.hpp"

// `epok::StreamPageCursor` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to bounded CD or PC geometry-page streaming. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
