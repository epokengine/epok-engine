# Epok API: Dungeon Map

> **Header:** `"dungeon_map.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/dungeon_map.hpp)

This module covers the dungeon map module. It documents 6 public callables declared directly in this header.

## Declared types

`epok::dungeon::Map`, `epok::dungeon::PointData`, `epok::dungeon::RoomData`, `epok::dungeon::RoomFlag`

## Callable index

- [`epok::dungeon::next_room`](#epok-dungeon-next-room-1) — The neighbour of `from` on a shortest way to `to`: `to` itself when they are linked, `from` when they are the same room, -1 when there is no way.
- [`epok::dungeon::random`](#epok-dungeon-random-1) — A gameplay random number below `below`, the same for the same stream, index and dungeon seed on every run.
- [`epok::dungeon::reset`](#epok-dungeon-reset-1) — Resets reset as part of the dungeon map module.
- [`epok::dungeon::room`](#epok-dungeon-room-1) — Performs `room` as part of the dungeon map module.
- [`epok::dungeon::room_at`](#epok-dungeon-room-at-1) — The room whose bounds hold the point, in the dungeon's plane; -1 in corridors and outside.
- [`epok::dungeon::walk`](#epok-dungeon-walk-1) — Breadth-first over the room links.

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
