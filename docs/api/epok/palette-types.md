# Epok API: Palette Types

> **Header:** `"palette_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/palette_types.hpp)

This module covers CLUT animation and palette ownership. It documents 4 public callables declared directly in this header.

## Declared types

`epok::Fixed`, `epok::PaletteAnimator`, `epok::PaletteStats`

## Callable index

- [`epok::PaletteAnimator::advance`](#epok-paletteanimator-advance-1) — Performs `advance` as part of CLUT animation and palette ownership.
- [`epok::PaletteAnimator::apply`](#epok-paletteanimator-apply-1) — Performs `apply` as part of CLUT animation and palette ownership.
- [`epok::PaletteAnimator::reset`](#epok-paletteanimator-reset-1) — Resets reset as part of CLUT animation and palette ownership.
- [`epok::PaletteAnimator::source_index`](#epok-paletteanimator-source-index-1) — Performs `source index` as part of CLUT animation and palette ownership.

<a id="epok-paletteanimator-advance-1"></a>

## `epok::PaletteAnimator::advance`

**Purpose.** Performs `advance` as part of CLUT animation and palette ownership.

**Exact declaration**

```cpp
void advance(Fixed dt)
```

- **Declared at:** [line 11](../../../runtime/palette_types.hpp#L11)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need CLUT animation and palette ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "palette_types.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::PaletteAnimator& object = /* obtain a valid instance */;

object.advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to CLUT animation and palette ownership. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimator-apply-1"></a>

## `epok::PaletteAnimator::apply`

**Purpose.** Performs `apply` as part of CLUT animation and palette ownership.

**Exact declaration**

```cpp
void apply(const uint16_t* source,uint16_t* destination)const
```

- **Declared at:** [line 24](../../../runtime/palette_types.hpp#L24)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const uint16_t *` | Input | Value supplied for `source`. See the exact type and module contract. |
| `destination` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `destination`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need CLUT animation and palette ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "palette_types.hpp"

// Assume these named values have been initialized with valid data:
// const uint16_t * source
// uint16_t * destination

epok::PaletteAnimator& object = /* obtain a valid instance */;

object.apply(source, destination);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-paletteanimator-reset-1"></a>

## `epok::PaletteAnimator::reset`

**Purpose.** Resets reset as part of CLUT animation and palette ownership.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 10](../../../runtime/palette_types.hpp#L10)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need CLUT animation and palette ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "palette_types.hpp"

epok::PaletteAnimator& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to CLUT animation and palette ownership. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-paletteanimator-source-index-1"></a>

## `epok::PaletteAnimator::source_index`

**Purpose.** Performs `source index` as part of CLUT animation and palette ownership.

**Exact declaration**

```cpp
uint16_t source_index(uint16_t index)const
```

- **Declared at:** [line 18](../../../runtime/palette_types.hpp#L18)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint16_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need CLUT animation and palette ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "palette_types.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t index

epok::PaletteAnimator& object = /* obtain a valid instance */;

auto result = object.source_index(index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
