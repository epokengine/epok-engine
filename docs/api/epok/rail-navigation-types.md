# Epok API: Rail Navigation Types

> **Header:** `"rail_navigation_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/rail_navigation_types.hpp)

This module covers the rail navigation types module. It documents 1 public callable declared directly in this header.

## Declared types

`epok::PackedRailParameter`, `epok::RailLink`, `epok::RailNavigationData`, `epok::RailNavigationState`, `epok::RailParameter`, `epok::RailRoute`

## Callable index

- [`epok::RailNavigationData::parameter`](#epok-railnavigationdata-parameter-1) — Performs `parameter` as part of the rail navigation types module.

<a id="epok-railnavigationdata-parameter-1"></a>

## `epok::RailNavigationData::parameter`

**Purpose.** Performs `parameter` as part of the rail navigation types module.

**Exact declaration**

```cpp
RailParameter parameter(size_t sample) const
```

- **Declared at:** [line 17](../../../runtime/rail_navigation_types.hpp#L17)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sample` | `size_t` | Input | Value supplied for `sample`. See the exact type and module contract. |

**Returns.** Returns `RailParameter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the rail navigation types module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "rail_navigation_types.hpp"

// Assume these named values have been initialized with valid data:
// size_t sample

epok::RailNavigationData& object = /* obtain a valid instance */;

auto result = object.parameter(sample);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
