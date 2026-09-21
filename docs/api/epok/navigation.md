# Epok API: Navigation

> **Header:** `"navigation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/navigation.hpp)

This module covers the navigation module. It documents 17 public callables declared directly in this header.

## Declared types

`epok::nav::Graph`, `epok::nav::Handle`, `epok::nav::Node`, `epok::nav::Request`, `epok::nav::Stats`, `epok::nav::Status`, `epok::nav::Surface`, `epok::nav::Traversal`, `epok::nav::World`, `epok::nav::World::Obstacle`

## Callable index

- [`epok::nav::World::add_obstacle`](#epok-nav-world-add-obstacle-1) — Adds obstacle as part of the navigation module.
- [`epok::nav::World::cancel`](#epok-nav-world-cancel-1) — Performs `cancel` as part of the navigation module.
- [`epok::nav::World::edge_open`](#epok-nav-world-edge-open-1) — Performs `edge open` as part of the navigation module.
- [`epok::nav::World::floor`](#epok-nav-world-floor-1) — Barycentric floor query in Q8; the closest reachable layer wins.
- [`epok::nav::World::get`](#epok-nav-world-get-1) — Returns get as part of the navigation module.
- [`epok::nav::World::locate`](#epok-nav-world-locate-1) — Performs `locate` as part of the navigation module.
- [`epok::nav::World::occupy`](#epok-nav-world-occupy-1) — Performs `occupy` as part of the navigation module.
- [`epok::nav::World::rebuild`](#epok-nav-world-rebuild-1) — Replace the current topology (procedural levels/streaming).
- [`epok::nav::World::remove_obstacle`](#epok-nav-world-remove-obstacle-1) — Removes obstacle as part of the navigation module.
- [`epok::nav::World::replan`](#epok-nav-world-replan-1) — Performs `replan` as part of the navigation module.
- [`epok::nav::World::request`](#epok-nav-world-request-1) — Requests request as part of the navigation module.
- [`epok::nav::World::reserve`](#epok-nav-world-reserve-1) — Performs `reserve` as part of the navigation module.
- [`epok::nav::World::reset`](#epok-nav-world-reset-1) — Resets reset as part of the navigation module.
- [`epok::nav::World::set_obstacle`](#epok-nav-world-set-obstacle-1) — Sets obstacle as part of the navigation module.
- [`epok::nav::World::tick`](#epok-nav-world-tick-1) — Performs `tick` as part of the navigation module.
- [`epok::nav::World::traversal`](#epok-nav-world-traversal-1) — Performs `traversal` as part of the navigation module.
- [`epok::nav::World::updating`](#epok-nav-world-updating-1) — Performs `updating` as part of the navigation module.

<a id="epok-nav-world-add-obstacle-1"></a>

## `epok::nav::World::add_obstacle`

**Purpose.** Adds obstacle as part of the navigation module.

**Exact declaration**

```cpp
uint16_t add_obstacle()
```

- **Declared at:** [line 52](../../../runtime/navigation.hpp#L52)
- **Kind:** `cxx method`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.add_obstacle();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-cancel-1"></a>

## `epok::nav::World::cancel`

**Purpose.** Performs `cancel` as part of the navigation module.

**Exact declaration**

```cpp
void cancel(Handle h)
```

- **Declared at:** [line 104](../../../runtime/navigation.hpp#L104)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::nav::World& object = /* obtain a valid instance */;

object.cancel(h);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-edge-open-1"></a>

## `epok::nav::World::edge_open`

**Purpose.** Performs `edge open` as part of the navigation module.

**Exact declaration**

```cpp
bool edge_open(uint16_t from,uint16_t to)const
```

- **Declared at:** [line 60](../../../runtime/navigation.hpp#L60)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// int from
// int to

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.edge_open(from, to);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-nav-world-floor-1"></a>

## `epok::nav::World::floor`

**Purpose.** Barycentric floor query in Q8; the closest reachable layer wins.

**Details.** Geometry is bounded at bake time. Caller can restrict to a local surface below.

**Exact declaration**

```cpp
bool floor(int32_t x,int32_t z,int32_t near,int32_t reach,int32_t& y)const
```

- **Declared at:** [line 84](../../../runtime/navigation.hpp#L84)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `int` | Input | Value supplied for `x`. See the exact type and module contract. |
| `z` | `int` | Input | Value supplied for `z`. See the exact type and module contract. |
| `near` | `int` | Input | Value supplied for `near`. See the exact type and module contract. |
| `reach` | `int` | Input | Value supplied for `reach`. See the exact type and module contract. |
| `y` | `int &` | Input/output; inspect the function contract | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Geometry is bounded at bake time. Caller can restrict to a local surface below.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// int x
// int z
// int near
// int reach
// int & y

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.floor(x, z, near, reach, y);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-nav-world-get-1"></a>

## `epok::nav::World::get`

**Purpose.** Returns get as part of the navigation module.

**Exact declaration**

```cpp
Request* get(Handle h)
```

- **Declared at:** [line 100](../../../runtime/navigation.hpp#L100)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** Returns `Request *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.get(h);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-locate-1"></a>

## `epok::nav::World::locate`

**Purpose.** Performs `locate` as part of the navigation module.

**Exact declaration**

```cpp
void locate(Handle h,const int32_t* p)
```

- **Declared at:** [line 67](../../../runtime/navigation.hpp#L67)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `p` | `const int *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// const int * p

epok::nav::World& object = /* obtain a valid instance */;

object.locate(h, p);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-nav-world-occupy-1"></a>

## `epok::nav::World::occupy`

**Purpose.** Performs `occupy` as part of the navigation module.

**Exact declaration**

```cpp
void occupy(Handle h,uint16_t node)
```

- **Declared at:** [line 81](../../../runtime/navigation.hpp#L81)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `node` | `int` | Input | Value supplied for `node`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// int node

epok::nav::World& object = /* obtain a valid instance */;

object.occupy(h, node);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-rebuild-1"></a>

## `epok::nav::World::rebuild`

**Purpose.** Replace the current topology (procedural levels/streaming).

**Details.** No old paths survive; goals and generation-checked request handles remain valid.

**Exact declaration**

```cpp
bool rebuild(Graph value)
```

- **Declared at:** [line 43](../../../runtime/navigation.hpp#L43)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Graph` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** No old paths survive; goals and generation-checked request handles remain valid.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Graph value

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.rebuild(value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-nav-world-remove-obstacle-1"></a>

## `epok::nav::World::remove_obstacle`

**Purpose.** Removes obstacle as part of the navigation module.

**Exact declaration**

```cpp
void remove_obstacle(uint16_t id)
```

- **Declared at:** [line 58](../../../runtime/navigation.hpp#L58)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `int` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// int id

epok::nav::World& object = /* obtain a valid instance */;

object.remove_obstacle(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-replan-1"></a>

## `epok::nav::World::replan`

**Purpose.** Performs `replan` as part of the navigation module.

**Exact declaration**

```cpp
void replan(Handle h,uint16_t avoid=invalid)
```

- **Declared at:** [line 68](../../../runtime/navigation.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `avoid` | `int` | Input | Value supplied for `avoid`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// int avoid

epok::nav::World& object = /* obtain a valid instance */;

object.replan(h, avoid);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-request-1"></a>

## `epok::nav::World::request`

**Purpose.** Requests request as part of the navigation module.

**Exact declaration**

```cpp
Handle request(const int32_t* from,const int32_t* to)
```

- **Declared at:** [line 105](../../../runtime/navigation.hpp#L105)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `const int *` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `const int *` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// const int * from
// const int * to

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.request(from, to);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-nav-world-reserve-1"></a>

## `epok::nav::World::reserve`

**Purpose.** Performs `reserve` as part of the navigation module.

**Exact declaration**

```cpp
bool reserve(Handle h,uint16_t from,uint16_t to)
```

- **Declared at:** [line 69](../../../runtime/navigation.hpp#L69)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `Handle` | Input | Value supplied for `h`. See the exact type and module contract. |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Handle h
// int from
// int to

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.reserve(h, from, to);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-nav-world-reset-1"></a>

## `epok::nav::World::reset`

**Purpose.** Resets reset as part of the navigation module.

**Exact declaration**

```cpp
void reset(Graph value)
```

- **Declared at:** [line 35](../../../runtime/navigation.hpp#L35)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Graph` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// Graph value

epok::nav::World& object = /* obtain a valid instance */;

object.reset(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-set-obstacle-1"></a>

## `epok::nav::World::set_obstacle`

**Purpose.** Sets obstacle as part of the navigation module.

**Exact declaration**

```cpp
bool set_obstacle(uint16_t id,const int32_t* lo,const int32_t* hi)
```

- **Declared at:** [line 53](../../../runtime/navigation.hpp#L53)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `int` | Input | Value supplied for `id`. See the exact type and module contract. |
| `lo` | `const int *` | Input | Value supplied for `lo`. See the exact type and module contract. |
| `hi` | `const int *` | Input | Value supplied for `hi`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// int id
// const int * lo
// const int * hi

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.set_obstacle(id, lo, hi);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-nav-world-tick-1"></a>

## `epok::nav::World::tick`

**Purpose.** Performs `tick` as part of the navigation module.

**Exact declaration**

```cpp
void tick()
```

- **Declared at:** [line 113](../../../runtime/navigation.hpp#L113)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

epok::nav::World& object = /* obtain a valid instance */;

object.tick();
```

**Why choose it.** It provides direct, allocation-conscious access to the navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-traversal-1"></a>

## `epok::nav::World::traversal`

**Purpose.** Performs `traversal` as part of the navigation module.

**Exact declaration**

```cpp
const Traversal* traversal(uint16_t from,uint16_t to)const
```

- **Declared at:** [line 64](../../../runtime/navigation.hpp#L64)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |

**Returns.** Returns `const Traversal *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

// Assume these named values have been initialized with valid data:
// int from
// int to

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.traversal(from, to);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-nav-world-updating-1"></a>

## `epok::nav::World::updating`

**Purpose.** Performs `updating` as part of the navigation module.

**Exact declaration**

```cpp
bool updating()const
```

- **Declared at:** [line 59](../../../runtime/navigation.hpp#L59)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "navigation.hpp"

epok::nav::World& object = /* obtain a valid instance */;

auto result = object.updating();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
