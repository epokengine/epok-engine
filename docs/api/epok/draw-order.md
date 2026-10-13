# Epok API: Draw Order

> **Header:** `"draw_order.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/draw_order.hpp)

This module covers the draw order module. It documents 2 public callables declared directly in this header.

## Declared types

`epok::DrawOrderSpan`

## Callable index

- [`epok::decode_draw_order`](#epok-decode-draw-order-1) — Performs `decode draw order` as part of the draw order module.
- [`epok::draw_order_varint`](#epok-draw-order-varint-1) — Draws order varint as part of the draw order module.

<a id="epok-decode-draw-order-1"></a>

## `epok::decode_draw_order`

**Purpose.** Performs `decode draw order` as part of the draw order module.

**Exact declaration**

```cpp
inline bool decode_draw_order(const uint8_t* data,size_t bytes, const uint32_t* previous,size_t previous_count,uint32_t* output,size_t capacity,size_t expected)
```

- **Declared at:** [line 22](../../../runtime/draw_order.hpp#L22)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `bytes` | `size_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `previous` | `const uint32_t *` | Input | Value supplied for `previous`. See the exact type and module contract. |
| `previous_count` | `size_t` | Input | Value supplied for `previous_count`. See the exact type and module contract. |
| `output` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `capacity` | `size_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `expected` | `size_t` | Input | Value supplied for `expected`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the draw order module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "draw_order.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * data
// size_t bytes
// const uint32_t * previous
// size_t previous_count
// uint32_t * output
// size_t capacity
// size_t expected

auto result = epok::decode_draw_order(data, bytes, previous, previous_count, output, capacity, expected);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-draw-order-varint-1"></a>

## `epok::draw_order_varint`

**Purpose.** Draws order varint as part of the draw order module.

**Exact declaration**

```cpp
inline bool draw_order_varint(const uint8_t*& cursor,const uint8_t* end,uint32_t& value)
```

- **Declared at:** [line 11](../../../runtime/draw_order.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cursor` | `const uint8_t *&` | Input | Value supplied for `cursor`. See the exact type and module contract. |
| `end` | `const uint8_t *` | Input | Value supplied for `end`. See the exact type and module contract. |
| `value` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the draw order module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "draw_order.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t *& cursor
// const uint8_t * end
// uint32_t & value

auto result = epok::draw_order_varint(cursor, end, value);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
