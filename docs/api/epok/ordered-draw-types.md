# Epok API: Ordered Draw Types

> **Header:** `"ordered_draw_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/ordered_draw_types.hpp)

This module covers the ordered draw types module. It documents 3 public callables declared directly in this header.

## Declared types

`epok::OrderedDrawBlock`, `epok::OrderedDrawData`, `epok::OrderedDrawStats`, `epok::OrderedProjectionBlock`

## Callable index

- [`epok::ordered_face_block`](#epok-ordered-face-block-1) — A face's projection block index sits above its two topology bits.
- [`epok::ordered_projection_block`](#epok-ordered-projection-block-1) — Performs `ordered projection block` as part of the ordered draw types module.
- [`epok::ordered_vertex_world`](#epok-ordered-vertex-world-1) — Exact Q12 world position of a stored vertex, for the CPU and validation paths: (relative + center) * 16 + fraction reproduces the cooked Q12 value.

<a id="epok-ordered-face-block-1"></a>

## `epok::ordered_face_block`

**Purpose.** A face's projection block index sits above its two topology bits.

**Exact declaration**

```cpp
inline uint32_t ordered_face_block(uint16_t topology)
```

- **Declared at:** [line 34](../../../runtime/ordered_draw_types.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `topology` | `uint16_t` | Input | Value supplied for `topology`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t topology

auto result = epok::ordered_face_block(topology);
```

**Why choose it.** It provides direct, allocation-conscious access to the ordered draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-ordered-projection-block-1"></a>

## `epok::ordered_projection_block`

**Purpose.** Performs `ordered projection block` as part of the ordered draw types module.

**Exact declaration**

```cpp
inline const OrderedProjectionBlock* ordered_projection_block(const OrderedDrawData* data,uint32_t token)
```

- **Declared at:** [line 56](../../../runtime/ordered_draw_types.hpp#L56)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const OrderedDrawData *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `token` | `uint32_t` | Input | Value supplied for `token`. See the exact type and module contract. |

**Returns.** Returns `const OrderedProjectionBlock *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const OrderedDrawData * data
// uint32_t token

auto result = epok::ordered_projection_block(data, token);
```

**Why choose it.** It provides direct, allocation-conscious access to the ordered draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordered-vertex-world-1"></a>

## `epok::ordered_vertex_world`

**Purpose.** Exact Q12 world position of a stored vertex, for the CPU and validation paths: (relative + center) * 16 + fraction reproduces the cooked Q12 value.

**Exact declaration**

```cpp
inline void ordered_vertex_world(const OrderedProjectionBlock& block,const uint8_t* entry,int32_t* world)
```

- **Declared at:** [line 37](../../../runtime/ordered_draw_types.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `block` | `const OrderedProjectionBlock &` | Input | Value supplied for `block`. See the exact type and module contract. |
| `entry` | `const uint8_t *` | Input | Value supplied for `entry`. See the exact type and module contract. |
| `world` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `world`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ordered draw types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw_types.hpp"

// Assume these named values have been initialized with valid data:
// const OrderedProjectionBlock & block
// const uint8_t * entry
// int32_t * world

epok::ordered_vertex_world(block, entry, world);
```

**Why choose it.** It provides direct, allocation-conscious access to the ordered draw types module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
