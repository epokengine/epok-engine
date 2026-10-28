# Epok API: Fragment Slot

> **Header:** `"fragment_slot.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/fragment_slot.hpp)

This module covers the fragment slot module. It documents 5 public callables declared directly in this header.

## Declared types

`epok::FragmentSlot`

## Callable index

- [`epok::FragmentSlot::FragmentSlot<Plain, Textured>`](#epok-fragmentslot-fragmentslot-plain-textured-1) — Constructs `epok::FragmentSlot` for the fragment slot module.
- [`epok::FragmentSlot::make_plain`](#epok-fragmentslot-make-plain-1) — Performs `make plain` as part of the fragment slot module.
- [`epok::FragmentSlot::make_textured`](#epok-fragmentslot-make-textured-1) — Performs `make textured` as part of the fragment slot module.
- [`epok::update_retained_triangle`](#epok-update-retained-triangle-1) — The retained packet's command, colors and UVs already exist.
- [`epok::update_retained_uniform_color`](#epok-update-retained-uniform-color-1) — Updates retained uniform color as part of the fragment slot module.

<a id="epok-fragmentslot-fragmentslot-plain-textured-1"></a>

## `epok::FragmentSlot::FragmentSlot<Plain, Textured>`

**Purpose.** Constructs `epok::FragmentSlot` for the fragment slot module.

**Exact declaration**

```cpp
FragmentSlot()
```

- **Declared at:** [line 12](../../../runtime/fragment_slot.hpp#L12)
- **Kind:** `constructor`

**Use it when.** You need the fragment slot module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fragment_slot.hpp"

epok::FragmentSlot value();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragment slot module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fragmentslot-make-plain-1"></a>

## `epok::FragmentSlot::make_plain`

**Purpose.** Performs `make plain` as part of the fragment slot module.

**Exact declaration**

```cpp
Plain& make_plain()
```

- **Declared at:** [line 13](../../../runtime/fragment_slot.hpp#L13)
- **Kind:** `cxx method`

**Returns.** Returns `Plain &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragment slot module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fragment_slot.hpp"

epok::FragmentSlot& object = /* obtain a valid instance */;

auto result = object.make_plain();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragment slot module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-fragmentslot-make-textured-1"></a>

## `epok::FragmentSlot::make_textured`

**Purpose.** Performs `make textured` as part of the fragment slot module.

**Exact declaration**

```cpp
Textured& make_textured()
```

- **Declared at:** [line 14](../../../runtime/fragment_slot.hpp#L14)
- **Kind:** `cxx method`

**Returns.** Returns `Textured &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the fragment slot module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fragment_slot.hpp"

epok::FragmentSlot& object = /* obtain a valid instance */;

auto result = object.make_textured();
```

**Why choose it.** It provides direct, allocation-conscious access to the fragment slot module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-update-retained-triangle-1"></a>

## `epok::update_retained_triangle`

**Purpose.** The retained packet's command, colors and UVs already exist.

**Details.** Update only XY and DMA links, using the same GT3/G3 word layout as PsyQo's typed fragments.

**Exact declaration**

```cpp
template<bool Textured,class Head> inline void update_retained_triangle(uint32_t* packet,uint32_t a,uint32_t b,uint32_t c,Head& bucket)
```

- **Declared at:** [line 18](../../../runtime/fragment_slot.hpp#L18)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packet` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `packet`. See the exact type and module contract. |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `uint32_t` | Input | Value supplied for `c`. See the exact type and module contract. |
| `bucket` | `Head &` | Input/output; inspect the function contract | Value supplied for `bucket`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Update only XY and DMA links, using the same GT3/G3 word layout as PsyQo's typed fragments.

**Usage pattern**

```cpp
#include "fragment_slot.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Textured, Head

// Assume these named values have been initialized with valid data:
// uint32_t * packet
// uint32_t a
// uint32_t b
// uint32_t c
// Head & bucket

epok::update_retained_triangle<Textured, Head>(packet, a, b, c, bucket);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-update-retained-uniform-color-1"></a>

## `epok::update_retained_uniform_color`

**Purpose.** Updates retained uniform color as part of the fragment slot module.

**Exact declaration**

```cpp
template<bool Textured> inline void update_retained_uniform_color(uint32_t* packet,uint32_t command,uint32_t color)
```

- **Declared at:** [line 23](../../../runtime/fragment_slot.hpp#L23)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packet` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `packet`. See the exact type and module contract. |
| `command` | `uint32_t` | Input | Value supplied for `command`. See the exact type and module contract. |
| `color` | `uint32_t` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the fragment slot module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fragment_slot.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Textured

// Assume these named values have been initialized with valid data:
// uint32_t * packet
// uint32_t command
// uint32_t color

epok::update_retained_uniform_color<Textured>(packet, command, color);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
