# Epok API: Ordered Draw

> **Header:** `"ordered_draw.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/ordered_draw.hpp)

This module covers the ordered draw module. It documents 7 public callables declared directly in this header.

## Declared types

`epok::OrderedDrawCursor`

## Callable index

- [`epok::OrderedDrawCursor::count`](#epok-ordereddrawcursor-count-1) — Performs `count` as part of the ordered draw module.
- [`epok::OrderedDrawCursor::owner`](#epok-ordereddrawcursor-owner-1) — Performs `owner` as part of the ordered draw module.
- [`epok::OrderedDrawCursor::prepare`](#epok-ordereddrawcursor-prepare-1) — page() only returns verified, already resident data.
- [`epok::OrderedDrawCursor::publish`](#epok-ordereddrawcursor-publish-1) — Performs `publish` as part of the ordered draw module.
- [`epok::OrderedDrawCursor::reset`](#epok-ordereddrawcursor-reset-1) — Resets reset as part of the ordered draw module.
- [`epok::OrderedDrawCursor::sample`](#epok-ordereddrawcursor-sample-1) — Performs `sample` as part of the ordered draw module.
- [`epok::OrderedDrawCursor::tokens`](#epok-ordereddrawcursor-tokens-1) — Performs `tokens` as part of the ordered draw module.

<a id="epok-ordereddrawcursor-count-1"></a>

## `epok::OrderedDrawCursor::count`

**Purpose.** Performs `count` as part of the ordered draw module.

**Exact declaration**

```cpp
size_t count() const
```

- **Declared at:** [line 22](../../../runtime/ordered_draw.hpp#L22)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-ordereddrawcursor-owner-1"></a>

## `epok::OrderedDrawCursor::owner`

**Purpose.** Performs `owner` as part of the ordered draw module.

**Exact declaration**

```cpp
const OrderedDrawData* owner() const
```

- **Declared at:** [line 24](../../../runtime/ordered_draw.hpp#L24)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const OrderedDrawData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.owner();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-ordereddrawcursor-prepare-1"></a>

## `epok::OrderedDrawCursor::prepare`

**Purpose.** page() only returns verified, already resident data.

**Details.** The caller separately checks/pins the sample's ENTIRE page set before publishing its camera.

**Exact declaration**

```cpp
template<class Page> bool prepare(const OrderedDrawData* data,uint16_t sample,Page&& page)
```

- **Declared at:** [line 34](../../../runtime/ordered_draw.hpp#L34)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const OrderedDrawData *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `sample` | `uint16_t` | Input | Value supplied for `sample`. See the exact type and module contract. |
| `page` | `Page &&` | Consumed or moved input | Value supplied for `page`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The caller separately checks/pins the sample's ENTIRE page set before publishing its camera.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Page

// Assume these named values have been initialized with valid data:
// const OrderedDrawData * data
// uint16_t sample
// Page && page

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.prepare<Page>(data, sample, page);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordereddrawcursor-publish-1"></a>

## `epok::OrderedDrawCursor::publish`

**Purpose.** Performs `publish` as part of the ordered draw module.

**Exact declaration**

```cpp
bool publish(const OrderedDrawData* data,uint16_t sample)
```

- **Declared at:** [line 25](../../../runtime/ordered_draw.hpp#L25)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const OrderedDrawData *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `sample` | `uint16_t` | Input | Value supplied for `sample`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

// Assume these named values have been initialized with valid data:
// const OrderedDrawData * data
// uint16_t sample

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.publish(data, sample);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordereddrawcursor-reset-1"></a>

## `epok::OrderedDrawCursor::reset`

**Purpose.** Resets reset as part of the ordered draw module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 20](../../../runtime/ordered_draw.hpp#L20)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-ordereddrawcursor-sample-1"></a>

## `epok::OrderedDrawCursor::sample`

**Purpose.** Performs `sample` as part of the ordered draw module.

**Exact declaration**

```cpp
uint16_t sample() const
```

- **Declared at:** [line 23](../../../runtime/ordered_draw.hpp#L23)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.sample();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-ordereddrawcursor-tokens-1"></a>

## `epok::OrderedDrawCursor::tokens`

**Purpose.** Performs `tokens` as part of the ordered draw module.

**Exact declaration**

```cpp
const uint32_t* tokens() const
```

- **Declared at:** [line 21](../../../runtime/ordered_draw.hpp#L21)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const uint32_t *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered draw module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_draw.hpp"

epok::OrderedDrawCursor& object = /* obtain a valid instance */;

auto result = object.tokens();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.
