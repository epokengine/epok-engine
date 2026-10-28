# Epok API: Instrument Allocator

> **Header:** `"instrument_allocator.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/instrument_allocator.hpp)

This module covers the instrument allocator module. It documents 3 public callables declared directly in this header.

## Declared types

`epok::instrument::allocation::Plan`, `epok::instrument::allocation::Request`, `epok::instrument::allocation::Slot`

## Callable index

- [`epok::instrument::allocation::reserve`](#epok-instrument-allocation-reserve-1) — Performs `reserve` as part of the instrument allocator module.
- [`epok::instrument::allocation::reserve_free`](#epok-instrument-allocation-reserve-free-1) — The common native case needs no stealing or note-group bookkeeping.
- [`epok::instrument::allocation::same_group`](#epok-instrument-allocation-same-group-1) — Performs `same group` as part of the instrument allocator module.

<a id="epok-instrument-allocation-reserve-1"></a>

## `epok::instrument::allocation::reserve`

**Purpose.** Performs `reserve` as part of the instrument allocator module.

**Exact declaration**

```cpp
inline Plan reserve(const Slot (&slots)[24], const Request& request)
```

- **Declared at:** [line 35](../../../runtime/instrument_allocator.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slots` | `const Slot (&)[24]` | Input | Value supplied for `slots`. See the exact type and module contract. |
| `request` | `const Request &` | Input | Value supplied for `request`. See the exact type and module contract. |

**Returns.** Returns `Plan`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_allocator.hpp"

// Assume these named values have been initialized with valid data:
// const Slot (&)[24] slots
// const Request & request

auto result = epok::instrument::allocation::reserve(slots, request);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument allocator module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-allocation-reserve-free-1"></a>

## `epok::instrument::allocation::reserve_free`

**Purpose.** The common native case needs no stealing or note-group bookkeeping.

**Details.** Callers can count live voices directly without constructing the full Slot array.

**Exact declaration**

```cpp
inline Plan reserve_free(uint32_t free_mask,unsigned own,unsigned music,const Request& request)
```

- **Declared at:** [line 19](../../../runtime/instrument_allocator.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `free_mask` | `uint32_t` | Input | Value supplied for `free_mask`. See the exact type and module contract. |
| `own` | `unsigned int` | Input | Value supplied for `own`. See the exact type and module contract. |
| `music` | `unsigned int` | Input | Value supplied for `music`. See the exact type and module contract. |
| `request` | `const Request &` | Input | Value supplied for `request`. See the exact type and module contract. |

**Returns.** Returns `Plan`. Check the purpose and failure notes before using the value.

**Use it when.** Callers can count live voices directly without constructing the full Slot array.

**Usage pattern**

```cpp
#include "instrument_allocator.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t free_mask
// unsigned int own
// unsigned int music
// const Request & request

auto result = epok::instrument::allocation::reserve_free(free_mask, own, music, request);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument allocator module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-allocation-same-group-1"></a>

## `epok::instrument::allocation::same_group`

**Purpose.** Performs `same group` as part of the instrument allocator module.

**Exact declaration**

```cpp
inline bool same_group(const Slot& a, const Slot& b)
```

- **Declared at:** [line 32](../../../runtime/instrument_allocator.hpp#L32)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Slot &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Slot &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument allocator module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_allocator.hpp"

// Assume these named values have been initialized with valid data:
// const Slot & a
// const Slot & b

auto result = epok::instrument::allocation::same_group(a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
