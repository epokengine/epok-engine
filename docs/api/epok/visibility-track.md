# Epok API: Visibility Track

> **Header:** `"visibility_track.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/visibility_track.hpp)

This module covers frustum tests and chunk visibility. It documents 15 public callables declared directly in this header.

## Declared types

`epok::PackedVisibilityTrackSample`, `epok::VisibilityPageSelection`, `epok::VisibilityPageSet`, `epok::VisibilityTrackData`, `epok::VisibilityTrackSample`, `epok::VisibilityTrackStats`

## Callable index

- [`epok::advance_visibility_track`](#epok-advance-visibility-track-1) — Readiness is supplied by the transport.
- [`epok::set_visibility_track`](#epok-set-visibility-track-1) — Sets visibility track as part of frustum tests and chunk visibility.
- [`epok::visibility_chunk`](#epok-visibility-chunk-1) — Performs `visibility chunk` as part of frustum tests and chunk visibility.
- [`epok::visibility_face`](#epok-visibility-face-1) — Performs `visibility face` as part of frustum tests and chunk visibility.
- [`epok::visibility_has_page_sets`](#epok-visibility-has-page-sets-1) — Performs `visibility has page sets` as part of frustum tests and chunk visibility.
- [`epok::visibility_next_mask`](#epok-visibility-next-mask-1) — Performs `visibility next mask` as part of frustum tests and chunk visibility.
- [`epok::visibility_prefetch_next`](#epok-visibility-prefetch-next-1) — Performs `visibility prefetch next` as part of frustum tests and chunk visibility.
- [`epok::visibility_track_apply`](#epok-visibility-track-apply-1) — Performs `visibility track apply` as part of frustum tests and chunk visibility.
- [`epok::visibility_varint`](#epok-visibility-varint-1) — Performs `visibility varint` as part of frustum tests and chunk visibility.
- [`epok::visibility_vertices`](#epok-visibility-vertices-1) — Performs `visibility vertices` as part of frustum tests and chunk visibility.
- [`epok::visibility_visit_lookahead_pages`](#epok-visibility-visit-lookahead-pages-1) — Optional read-ahead for a deterministic playback path.
- [`epok::visibility_visit_pages`](#epok-visibility-visit-pages-1) — Performs `visibility visit pages` as part of frustum tests and chunk visibility.
- [`epok::visibility_xor_run`](#epok-visibility-xor-run-1) — Toggle a cooked contiguous run a word at a time.
- [`epok::VisibilityTrackData::is_cut`](#epok-visibilitytrackdata-is-cut-1) — Reports whether cut as part of frustum tests and chunk visibility.
- [`epok::VisibilityTrackData::sample`](#epok-visibilitytrackdata-sample-1) — Performs `sample` as part of frustum tests and chunk visibility.

<a id="epok-advance-visibility-track-1"></a>

## `epok::advance_visibility_track`

**Purpose.** Readiness is supplied by the transport.

**Details.** No camera advance can expose a page before its verified read completes. Looping rebuilds the first full mask.

**Exact declaration**

```cpp
template<class Ready,class Camera> inline void advance_visibility_track(uint32_t elapsed,Ready&& ready,Camera&& camera)
```

- **Declared at:** [line 167](../../../runtime/visibility_track.hpp#L167)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |
| `ready` | `Ready &&` | Consumed or moved input | Value supplied for `ready`. See the exact type and module contract. |
| `camera` | `Camera &&` | Consumed or moved input | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** No camera advance can expose a page before its verified read completes. Looping rebuilds the first full mask.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Ready, Camera

// Assume these named values have been initialized with valid data:
// uint32_t elapsed
// Ready && ready
// Camera && camera

epok::advance_visibility_track<Ready, Camera>(elapsed, ready, camera);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-set-visibility-track-1"></a>

## `epok::set_visibility_track`

**Purpose.** Sets visibility track as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline void set_visibility_track(const VisibilityTrackData* data)
```

- **Declared at:** [line 117](../../../runtime/visibility_track.hpp#L117)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const VisibilityTrackData *` | Input | Value supplied for `data`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// const VisibilityTrackData * data

epok::set_visibility_track(data);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-chunk-1"></a>

## `epok::visibility_chunk`

**Purpose.** Performs `visibility chunk` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline bool visibility_chunk(const uint32_t* mask,size_t faces)
```

- **Declared at:** [line 151](../../../runtime/visibility_track.hpp#L151)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `const uint32_t *` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `faces` | `size_t` | Input | Value supplied for `faces`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// const uint32_t * mask
// size_t faces

auto result = epok::visibility_chunk(mask, faces);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-face-1"></a>

## `epok::visibility_face`

**Purpose.** Performs `visibility face` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline bool visibility_face(const uint32_t* mask,size_t face)
```

- **Declared at:** [line 140](../../../runtime/visibility_track.hpp#L140)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `const uint32_t *` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `face` | `size_t` | Input | Value supplied for `face`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// const uint32_t * mask
// size_t face

auto result = epok::visibility_face(mask, face);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-has-page-sets-1"></a>

## `epok::visibility_has_page_sets`

**Purpose.** Performs `visibility has page sets` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline bool visibility_has_page_sets()
```

- **Declared at:** [line 59](../../../runtime/visibility_track.hpp#L59)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

auto result = epok::visibility_has_page_sets();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibility-next-mask-1"></a>

## `epok::visibility_next_mask`

**Purpose.** Performs `visibility next mask` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline const uint32_t* visibility_next_mask(const uint32_t* current)
```

- **Declared at:** [line 156](../../../runtime/visibility_track.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `current` | `const uint32_t *` | Input | Value supplied for `current`. See the exact type and module contract. |

**Returns.** Returns `const uint32_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// const uint32_t * current

auto result = epok::visibility_next_mask(current);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-prefetch-next-1"></a>

## `epok::visibility_prefetch_next`

**Purpose.** Performs `visibility prefetch next` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline bool visibility_prefetch_next()
```

- **Declared at:** [line 159](../../../runtime/visibility_track.hpp#L159)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

auto result = epok::visibility_prefetch_next();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibility-track-apply-1"></a>

## `epok::visibility_track_apply`

**Purpose.** Performs `visibility track apply` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline void visibility_track_apply(uint32_t* words,const VisibilityTrackData& data,size_t sample)
```

- **Declared at:** [line 107](../../../runtime/visibility_track.hpp#L107)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `words` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `words`. See the exact type and module contract. |
| `data` | `const VisibilityTrackData &` | Input | Value supplied for `data`. See the exact type and module contract. |
| `sample` | `size_t` | Input | Value supplied for `sample`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * words
// const VisibilityTrackData & data
// size_t sample

epok::visibility_track_apply(words, data, sample);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-varint-1"></a>

## `epok::visibility_varint`

**Purpose.** Performs `visibility varint` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
inline uint32_t visibility_varint(const uint8_t*& p)
```

- **Declared at:** [line 91](../../../runtime/visibility_track.hpp#L91)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const uint8_t *&` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t *& p

auto result = epok::visibility_varint(p);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-vertices-1"></a>

## `epok::visibility_vertices`

**Purpose.** Performs `visibility vertices` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
template<class Quad> inline size_t visibility_vertices(const uint32_t* faces,const Quad* quads,size_t count,uint32_t (&vertices)[16])
```

- **Declared at:** [line 143](../../../runtime/visibility_track.hpp#L143)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `faces` | `const uint32_t *` | Input | Value supplied for `faces`. See the exact type and module contract. |
| `quads` | `const Quad *` | Input | Value supplied for `quads`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `vertices` | `uint32_t (&)[16]` | Input/output; inspect the function contract | Value supplied for `vertices`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Quad

// Assume these named values have been initialized with valid data:
// const uint32_t * faces
// const Quad * quads
// size_t count
// uint32_t (&)[16] vertices

auto result = epok::visibility_vertices<Quad>(faces, quads, count, vertices);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-visit-lookahead-pages-1"></a>

## `epok::visibility_visit_lookahead_pages`

**Purpose.** Optional read-ahead for a deterministic playback path.

**Details.** Current/next pages are selected separately and always retain priority. Do not guess a gameplay rail branch or read through a camera cut; cap traversal even for tiny samples.

**Exact declaration**

```cpp
template<class Visit> inline void visibility_visit_lookahead_pages(uint32_t horizon_us, Visit&& visit)
```

- **Declared at:** [line 76](../../../runtime/visibility_track.hpp#L76)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `horizon_us` | `uint32_t` | Input | Value supplied for `horizon_us`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Current/next pages are selected separately and always retain priority. Do not guess a gameplay rail branch or read through a camera cut; cap traversal even for tiny samples.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// uint32_t horizon_us
// Visit && visit

epok::visibility_visit_lookahead_pages<Visit>(horizon_us, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-visit-pages-1"></a>

## `epok::visibility_visit_pages`

**Purpose.** Performs `visibility visit pages` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
template<class Visit> inline void visibility_visit_pages(bool next,Visit&& visit)
```

- **Declared at:** [line 62](../../../runtime/visibility_track.hpp#L62)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `next` | `bool` | Input | Value supplied for `next`. See the exact type and module contract. |
| `visit` | `Visit &&` | Consumed or moved input | Value supplied for `visit`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Visit

// Assume these named values have been initialized with valid data:
// bool next
// Visit && visit

epok::visibility_visit_pages<Visit>(next, visit);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibility-xor-run-1"></a>

## `epok::visibility_xor_run`

**Purpose.** Toggle a cooked contiguous run a word at a time.

**Details.** width is always 1..32: neither shift can be 32, including a full aligned word or its last bit.

**Exact declaration**

```cpp
inline void visibility_xor_run(uint32_t* words,uint32_t first,uint32_t count)
```

- **Declared at:** [line 98](../../../runtime/visibility_track.hpp#L98)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `words` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `words`. See the exact type and module contract. |
| `first` | `uint32_t` | Input | Value supplied for `first`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** width is always 1..32: neither shift can be 32, including a full aligned word or its last bit.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * words
// uint32_t first
// uint32_t count

epok::visibility_xor_run(words, first, count);
```

**Why choose it.** It provides direct, allocation-conscious access to frustum tests and chunk visibility. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-visibilitytrackdata-is-cut-1"></a>

## `epok::VisibilityTrackData::is_cut`

**Purpose.** Reports whether cut as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
bool is_cut(size_t index) const
```

- **Declared at:** [line 48](../../../runtime/visibility_track.hpp#L48)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::VisibilityTrackData& object = /* obtain a valid instance */;

auto result = object.is_cut(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-visibilitytrackdata-sample-1"></a>

## `epok::VisibilityTrackData::sample`

**Purpose.** Performs `sample` as part of frustum tests and chunk visibility.

**Exact declaration**

```cpp
VisibilityTrackSample sample(size_t index) const
```

- **Declared at:** [line 42](../../../runtime/visibility_track.hpp#L42)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `VisibilityTrackSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need frustum tests and chunk visibility and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "visibility_track.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::VisibilityTrackData& object = /* obtain a valid instance */;

auto result = object.sample(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
