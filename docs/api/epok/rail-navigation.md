# Epok API: Rail Navigation

> **Header:** `"rail_navigation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/rail_navigation.hpp)

This module covers the rail navigation module. It documents 6 public callables declared directly in this header.

## Callable index

- [`epok::advance_rail_navigation`](#epok-advance-rail-navigation-1) — Performs `advance rail navigation` as part of the rail navigation module.
- [`epok::rail_error`](#epok-rail-error-1) — Performs `rail error` as part of the rail navigation module.
- [`epok::rail_pending_cut`](#epok-rail-pending-cut-1) — Performs `rail pending cut` as part of the rail navigation module.
- [`epok::rail_prepare`](#epok-rail-prepare-1) — Performs `rail prepare` as part of the rail navigation module.
- [`epok::rail_projection`](#epok-rail-projection-1) — Performs `rail projection` as part of the rail navigation module.
- [`epok::rail_set_link_enabled`](#epok-rail-set-link-enabled-1) — This controller consumes world-space player position, never pad input.

<a id="epok-advance-rail-navigation-1"></a>

## `epok::advance_rail_navigation`

**Purpose.** Performs `advance rail navigation` as part of the rail navigation module.

**Exact declaration**

```cpp
template<class Ready,class Camera> inline void advance_rail_navigation(uint32_t elapsed,const int32_t* player,Ready&& ready,Camera&& camera)
```

- **Declared at:** [line 39](../../../runtime/rail_navigation.hpp#L39)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |
| `player` | `const int32_t *` | Input | Value supplied for `player`. See the exact type and module contract. |
| `ready` | `Ready &&` | Consumed or moved input | Value supplied for `ready`. See the exact type and module contract. |
| `camera` | `Camera &&` | Consumed or moved input | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the rail navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Ready, Camera

// Assume these named values have been initialized with valid data:
// uint32_t elapsed
// const int32_t * player
// Ready && ready
// Camera && camera

epok::advance_rail_navigation<Ready, Camera>(elapsed, player, ready, camera);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-rail-error-1"></a>

## `epok::rail_error`

**Purpose.** Performs `rail error` as part of the rail navigation module.

**Exact declaration**

```cpp
inline int64_t rail_error(int64_t value,const RailParameter& p)
```

- **Declared at:** [line 21](../../../runtime/rail_navigation.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `p` | `const RailParameter &` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the rail navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value
// const RailParameter & p

auto result = epok::rail_error(value, p);
```

**Why choose it.** It provides direct, allocation-conscious access to the rail navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-rail-pending-cut-1"></a>

## `epok::rail_pending_cut`

**Purpose.** Performs `rail pending cut` as part of the rail navigation module.

**Exact declaration**

```cpp
inline bool rail_pending_cut()
```

- **Declared at:** [line 4](../../../runtime/rail_navigation.hpp#L4)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the rail navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

auto result = epok::rail_pending_cut();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rail-prepare-1"></a>

## `epok::rail_prepare`

**Purpose.** Performs `rail prepare` as part of the rail navigation module.

**Exact declaration**

```cpp
inline void rail_prepare(uint16_t sample,uint16_t route,bool crossing,bool forward)
```

- **Declared at:** [line 24](../../../runtime/rail_navigation.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sample` | `uint16_t` | Input | Value supplied for `sample`. See the exact type and module contract. |
| `route` | `uint16_t` | Input | Value supplied for `route`. See the exact type and module contract. |
| `crossing` | `bool` | Input | Value supplied for `crossing`. See the exact type and module contract. |
| `forward` | `bool` | Input | Value supplied for `forward`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the rail navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t sample
// uint16_t route
// bool crossing
// bool forward

epok::rail_prepare(sample, route, crossing, forward);
```

**Why choose it.** It provides direct, allocation-conscious access to the rail navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rail-projection-1"></a>

## `epok::rail_projection`

**Purpose.** Performs `rail projection` as part of the rail navigation module.

**Exact declaration**

```cpp
inline int64_t rail_projection(const int32_t* player,const VisibilityTrackSample& sample,const RailParameter& p)
```

- **Declared at:** [line 17](../../../runtime/rail_navigation.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `player` | `const int32_t *` | Input | Value supplied for `player`. See the exact type and module contract. |
| `sample` | `const VisibilityTrackSample &` | Input | Value supplied for `sample`. See the exact type and module contract. |
| `p` | `const RailParameter &` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the rail navigation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * player
// const VisibilityTrackSample & sample
// const RailParameter & p

auto result = epok::rail_projection(player, sample, p);
```

**Why choose it.** It provides direct, allocation-conscious access to the rail navigation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-rail-set-link-enabled-1"></a>

## `epok::rail_set_link_enabled`

**Purpose.** This controller consumes world-space player position, never pad input.

**Details.** It is reusable with native/Blueprint actors and independent of the page transport.

**Exact declaration**

```cpp
inline bool rail_set_link_enabled(size_t index,bool enabled)
```

- **Declared at:** [line 11](../../../runtime/rail_navigation.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** It is reusable with native/Blueprint actors and independent of the page transport.

**Usage pattern**

```cpp
#include "rail_navigation.hpp"

// Assume these named values have been initialized with valid data:
// size_t index
// bool enabled

auto result = epok::rail_set_link_enabled(index, enabled);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
