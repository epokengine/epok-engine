# Epok API: Dungeon Map

> **Header:** `"dungeon_map.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/dungeon_map.hpp)

This module covers the dungeon map module. It documents 16 public callables declared directly in this header.

## Declared types

`epok::dungeon::Map`, `epok::dungeon::MapRect`, `epok::dungeon::Minimap`, `epok::dungeon::MinimapColor`, `epok::dungeon::MinimapFlag`, `epok::dungeon::PointData`, `epok::dungeon::RoomData`, `epok::dungeon::RoomFlag`

## Callable index

- [`epok::dungeon::explore_at`](#epok-dungeon-explore-at-1) — Explores where a point stands: its rectangle, and its whole room the first time.
- [`epok::dungeon::explore_everything`](#epok-dungeon-explore-everything-1) — Performs `explore everything` as part of the dungeon map module.
- [`epok::dungeon::explore_room`](#epok-dungeon-explore-room-1) — Marks a room and every rectangle of it explored.
- [`epok::dungeon::floor_divide`](#epok-dungeon-floor-divide-1) — Performs `floor divide` as part of the dungeon map module.
- [`epok::dungeon::forget_everything`](#epok-dungeon-forget-everything-1) — Performs `forget everything` as part of the dungeon map module.
- [`epok::dungeon::map_cell`](#epok-dungeon-map-cell-1) — The minimap cell holding a world point; false outside the grid.
- [`epok::dungeon::next_room`](#epok-dungeon-next-room-1) — The neighbour of `from` on a shortest way to `to`: `to` itself when they are linked, `from` when they are the same room, -1 when there is no way.
- [`epok::dungeon::random`](#epok-dungeon-random-1) — A gameplay random number below `below`, the same for the same stream, index and dungeon seed on every run.
- [`epok::dungeon::rect_at`](#epok-dungeon-rect-at-1) — The rectangle holding a minimap cell, or -1.
- [`epok::dungeon::rect_explored`](#epok-dungeon-rect-explored-1) — Performs `rect explored` as part of the dungeon map module.
- [`epok::dungeon::reset`](#epok-dungeon-reset-1) — Resets reset as part of the dungeon map module.
- [`epok::dungeon::reset_minimap`](#epok-dungeon-reset-minimap-1) — Resets minimap as part of the dungeon map module.
- [`epok::dungeon::room`](#epok-dungeon-room-1) — Performs `room` as part of the dungeon map module.
- [`epok::dungeon::room_at`](#epok-dungeon-room-at-1) — The room whose bounds hold the point, in the dungeon's plane; -1 in corridors and outside.
- [`epok::dungeon::room_explored`](#epok-dungeon-room-explored-1) — Performs `room explored` as part of the dungeon map module.
- [`epok::dungeon::walk`](#epok-dungeon-walk-1) — Breadth-first over the room links.

<a id="epok-dungeon-explore-at-1"></a>

## `epok::dungeon::explore_at`

**Purpose.** Explores where a point stands: its rectangle, and its whole room the first time.

**Exact declaration**

```cpp
inline void explore_at(const int32_t* point)
```

- **Declared at:** [line 163](../../../runtime/dungeon_map.hpp#L163)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `const int32_t *` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * point

epok::dungeon::explore_at(point);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dungeon-explore-everything-1"></a>

## `epok::dungeon::explore_everything`

**Purpose.** Performs `explore everything` as part of the dungeon map module.

**Exact declaration**

```cpp
inline void explore_everything()
```

- **Declared at:** [line 137](../../../runtime/dungeon_map.hpp#L137)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

epok::dungeon::explore_everything();
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-explore-room-1"></a>

## `epok::dungeon::explore_room`

**Purpose.** Marks a room and every rectangle of it explored.

**Exact declaration**

```cpp
inline void explore_room(int32_t room)
```

- **Declared at:** [line 132](../../../runtime/dungeon_map.hpp#L132)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room

epok::dungeon::explore_room(room);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-floor-divide-1"></a>

## `epok::dungeon::floor_divide`

**Purpose.** Performs `floor divide` as part of the dungeon map module.

**Exact declaration**

```cpp
inline int32_t floor_divide(int32_t a,int32_t b)
```

- **Declared at:** [line 145](../../../runtime/dungeon_map.hpp#L145)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::dungeon::floor_divide(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-forget-everything-1"></a>

## `epok::dungeon::forget_everything`

**Purpose.** Performs `forget everything` as part of the dungeon map module.

**Exact declaration**

```cpp
inline void forget_everything()
```

- **Declared at:** [line 141](../../../runtime/dungeon_map.hpp#L141)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

epok::dungeon::forget_everything();
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-map-cell-1"></a>

## `epok::dungeon::map_cell`

**Purpose.** The minimap cell holding a world point; false outside the grid.

**Exact declaration**

```cpp
inline bool map_cell(const int32_t* point,int32_t& cx,int32_t& cy)
```

- **Declared at:** [line 147](../../../runtime/dungeon_map.hpp#L147)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `const int32_t *` | Input | Value supplied for `point`. See the exact type and module contract. |
| `cx` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `cx`. See the exact type and module contract. |
| `cy` | `int32_t &` | Input/output; inspect the function contract | Value supplied for `cy`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * point
// int32_t & cx
// int32_t & cy

auto result = epok::dungeon::map_cell(point, cx, cy);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dungeon-next-room-1"></a>

## `epok::dungeon::next_room`

**Purpose.** The neighbour of `from` on a shortest way to `to`: `to` itself when they are linked, `from` when they are the same room, -1 when there is no way.

**Exact declaration**

```cpp
inline int32_t next_room(int32_t from,int32_t to)
```

- **Declared at:** [line 74](../../../runtime/dungeon_map.hpp#L74)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int32_t` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t from
// int32_t to

auto result = epok::dungeon::next_room(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-random-1"></a>

## `epok::dungeon::random`

**Purpose.** A gameplay random number below `below`, the same for the same stream, index and dungeon seed on every run.

**Exact declaration**

```cpp
inline uint32_t random(uint32_t stream,uint32_t index,uint32_t below)
```

- **Declared at:** [line 85](../../../runtime/dungeon_map.hpp#L85)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `stream` | `uint32_t` | Input | Value supplied for `stream`. See the exact type and module contract. |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `below` | `uint32_t` | Input | Value supplied for `below`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t stream
// uint32_t index
// uint32_t below

auto result = epok::dungeon::random(stream, index, below);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-rect-at-1"></a>

## `epok::dungeon::rect_at`

**Purpose.** The rectangle holding a minimap cell, or -1.

**Exact declaration**

```cpp
inline int32_t rect_at(int32_t cx,int32_t cy)
```

- **Declared at:** [line 155](../../../runtime/dungeon_map.hpp#L155)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cx` | `int32_t` | Input | Value supplied for `cx`. See the exact type and module contract. |
| `cy` | `int32_t` | Input | Value supplied for `cy`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t cx
// int32_t cy

auto result = epok::dungeon::rect_at(cx, cy);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-rect-explored-1"></a>

## `epok::dungeon::rect_explored`

**Purpose.** Performs `rect explored` as part of the dungeon map module.

**Exact declaration**

```cpp
inline bool rect_explored(uint16_t i)
```

- **Declared at:** [line 129](../../../runtime/dungeon_map.hpp#L129)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `uint16_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t i

auto result = epok::dungeon::rect_explored(i);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeon-reset-1"></a>

## `epok::dungeon::reset`

**Purpose.** Resets reset as part of the dungeon map module.

**Exact declaration**

```cpp
inline void reset(const Map& value)
```

- **Declared at:** [line 37](../../../runtime/dungeon_map.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const Map &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// const Map & value

epok::dungeon::reset(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dungeon-reset-minimap-1"></a>

## `epok::dungeon::reset_minimap`

**Purpose.** Resets minimap as part of the dungeon map module.

**Exact declaration**

```cpp
inline void reset_minimap(const Minimap& value)
```

- **Declared at:** [line 125](../../../runtime/dungeon_map.hpp#L125)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const Minimap &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// const Minimap & value

epok::dungeon::reset_minimap(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dungeon-room-1"></a>

## `epok::dungeon::room`

**Purpose.** Performs `room` as part of the dungeon map module.

**Exact declaration**

```cpp
inline const RoomData* room(int32_t index)
```

- **Declared at:** [line 39](../../../runtime/dungeon_map.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const RoomData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t index

auto result = epok::dungeon::room(index);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dungeon-room-at-1"></a>

## `epok::dungeon::room_at`

**Purpose.** The room whose bounds hold the point, in the dungeon's plane; -1 in corridors and outside.

**Details.** Rooms never overlap on that plane, whatever their level.

**Exact declaration**

```cpp
inline int32_t room_at(const int32_t* point)
```

- **Declared at:** [line 44](../../../runtime/dungeon_map.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `const int32_t *` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Rooms never overlap on that plane, whatever their level.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * point

auto result = epok::dungeon::room_at(point);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dungeon-room-explored-1"></a>

## `epok::dungeon::room_explored`

**Purpose.** Performs `room explored` as part of the dungeon map module.

**Exact declaration**

```cpp
inline bool room_explored(int32_t room)
```

- **Declared at:** [line 130](../../../runtime/dungeon_map.hpp#L130)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `room` | `int32_t` | Input | Value supplied for `room`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dungeon map module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t room

auto result = epok::dungeon::room_explored(room);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dungeon-walk-1"></a>

## `epok::dungeon::walk`

**Purpose.** Breadth-first over the room links.

**Details.** Writes each reached room's predecessor towards `from` and returns the number of steps to `to`, or -1 when it cannot be reached.

**Exact declaration**

```cpp
inline int32_t walk(int32_t from,int32_t to,int8_t* previous)
```

- **Declared at:** [line 54](../../../runtime/dungeon_map.hpp#L54)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int32_t` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int32_t` | Input | Value supplied for `to`. See the exact type and module contract. |
| `previous` | `int8_t *` | Input/output; inspect the function contract | Value supplied for `previous`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Writes each reached room's predecessor towards `from` and returns the number of steps to `to`, or -1 when it cannot be reached.

**Usage pattern**

```cpp
#include "dungeon_map.hpp"

// Assume these named values have been initialized with valid data:
// int32_t from
// int32_t to
// int8_t * previous

auto result = epok::dungeon::walk(from, to, previous);
```

**Why choose it.** It provides direct, allocation-conscious access to the dungeon map module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
