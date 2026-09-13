# PsyQo API: Primitives / Rectangles

> **Header:** `"psyqo/primitives/rectangles.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh)

This module covers typed PlayStation GPU primitives. It documents 20 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::Pixel`, `psyqo::Prim::Rectangle`, `psyqo::Prim::Rectangle16x16`, `psyqo::Prim::Rectangle8x8`

## Callable index

- [`psyqo::Prim::Pixel::Pixel`](#psyqo-prim-pixel-pixel-1) — Constructs `psyqo::Prim::Pixel` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Pixel::Pixel`](#psyqo-prim-pixel-pixel-2) — Constructs `psyqo::Prim::Pixel` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Pixel::setColor`](#psyqo-prim-pixel-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Pixel::setOpaque`](#psyqo-prim-pixel-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Pixel::setSemiTrans`](#psyqo-prim-pixel-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle16x16::Rectangle16x16`](#psyqo-prim-rectangle16x16-rectangle16x16-1) — Constructs `psyqo::Prim::Rectangle16x16` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle16x16::Rectangle16x16`](#psyqo-prim-rectangle16x16-rectangle16x16-2) — Constructs `psyqo::Prim::Rectangle16x16` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle16x16::setColor`](#psyqo-prim-rectangle16x16-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle16x16::setOpaque`](#psyqo-prim-rectangle16x16-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle16x16::setSemiTrans`](#psyqo-prim-rectangle16x16-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle8x8::Rectangle8x8`](#psyqo-prim-rectangle8x8-rectangle8x8-1) — Constructs `psyqo::Prim::Rectangle8x8` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle8x8::Rectangle8x8`](#psyqo-prim-rectangle8x8-rectangle8x8-2) — Constructs `psyqo::Prim::Rectangle8x8` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle8x8::setColor`](#psyqo-prim-rectangle8x8-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle8x8::setOpaque`](#psyqo-prim-rectangle8x8-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle8x8::setSemiTrans`](#psyqo-prim-rectangle8x8-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle::Rectangle`](#psyqo-prim-rectangle-rectangle-1) — Constructs `psyqo::Prim::Rectangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle::Rectangle`](#psyqo-prim-rectangle-rectangle-2) — Constructs `psyqo::Prim::Rectangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle::setColor`](#psyqo-prim-rectangle-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle::setOpaque`](#psyqo-prim-rectangle-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Rectangle::setSemiTrans`](#psyqo-prim-rectangle-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.

<a id="psyqo-prim-pixel-pixel-1"></a>

## `psyqo::Prim::Pixel::Pixel`

**Purpose.** Constructs `psyqo::Prim::Pixel` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Pixel() : c
```

- **Declared at:** [line 83](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L83)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Pixel value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-pixel-pixel-2"></a>

## `psyqo::Prim::Pixel::Pixel`

**Purpose.** Constructs `psyqo::Prim::Pixel` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Pixel(Color c) : c
```

- **Declared at:** [line 84](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L84)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Pixel value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-pixel-setcolor-1"></a>

## `psyqo::Prim::Pixel::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Pixel& setColor(Color c)
```

- **Declared at:** [line 85](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L85)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Pixel &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Pixel& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-pixel-setopaque-1"></a>

## `psyqo::Prim::Pixel::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Pixel& setOpaque()
```

- **Declared at:** [line 90](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L90)
- **Kind:** `cxx method`

**Returns.** Returns `Pixel &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Pixel& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-pixel-setsemitrans-1"></a>

## `psyqo::Prim::Pixel::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Pixel& setSemiTrans()
```

- **Declared at:** [line 94](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L94)
- **Kind:** `cxx method`

**Returns.** Returns `Pixel &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Pixel& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle16x16-rectangle16x16-1"></a>

## `psyqo::Prim::Rectangle16x16::Rectangle16x16`

**Purpose.** Constructs `psyqo::Prim::Rectangle16x16` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle16x16() : c
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L151)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle16x16 value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle16x16-rectangle16x16-2"></a>

## `psyqo::Prim::Rectangle16x16::Rectangle16x16`

**Purpose.** Constructs `psyqo::Prim::Rectangle16x16` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle16x16(Color c) : c
```

- **Declared at:** [line 152](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L152)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle16x16 value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle16x16-setcolor-1"></a>

## `psyqo::Prim::Rectangle16x16::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle16x16& setColor(Color c)
```

- **Declared at:** [line 153](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L153)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Rectangle16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle16x16& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle16x16-setopaque-1"></a>

## `psyqo::Prim::Rectangle16x16::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle16x16& setOpaque()
```

- **Declared at:** [line 158](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L158)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle16x16& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle16x16-setsemitrans-1"></a>

## `psyqo::Prim::Rectangle16x16::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle16x16& setSemiTrans()
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L162)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle16x16& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle8x8-rectangle8x8-1"></a>

## `psyqo::Prim::Rectangle8x8::Rectangle8x8`

**Purpose.** Constructs `psyqo::Prim::Rectangle8x8` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle8x8() : c
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L117)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle8x8 value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle8x8-rectangle8x8-2"></a>

## `psyqo::Prim::Rectangle8x8::Rectangle8x8`

**Purpose.** Constructs `psyqo::Prim::Rectangle8x8` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle8x8(Color c) : c
```

- **Declared at:** [line 118](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L118)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle8x8 value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle8x8-setcolor-1"></a>

## `psyqo::Prim::Rectangle8x8::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle8x8& setColor(Color c)
```

- **Declared at:** [line 119](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L119)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Rectangle8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle8x8& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle8x8-setopaque-1"></a>

## `psyqo::Prim::Rectangle8x8::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle8x8& setOpaque()
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L124)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle8x8& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle8x8-setsemitrans-1"></a>

## `psyqo::Prim::Rectangle8x8::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle8x8& setSemiTrans()
```

- **Declared at:** [line 128](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L128)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle8x8& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle-rectangle-1"></a>

## `psyqo::Prim::Rectangle::Rectangle`

**Purpose.** Constructs `psyqo::Prim::Rectangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle() : c
```

- **Declared at:** [line 48](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L48)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle-rectangle-2"></a>

## `psyqo::Prim::Rectangle::Rectangle`

**Purpose.** Constructs `psyqo::Prim::Rectangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle(Color c) : c
```

- **Declared at:** [line 49](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L49)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle-setcolor-1"></a>

## `psyqo::Prim::Rectangle::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle& setColor(Color c)
```

- **Declared at:** [line 50](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L50)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Rectangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Rectangle& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle-setopaque-1"></a>

## `psyqo::Prim::Rectangle::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle& setOpaque()
```

- **Declared at:** [line 55](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L55)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-rectangle-setsemitrans-1"></a>

## `psyqo::Prim::Rectangle::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Rectangle& setSemiTrans()
```

- **Declared at:** [line 59](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/rectangles.hh#L59)
- **Kind:** `cxx method`

**Returns.** Returns `Rectangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/rectangles.hh"

psyqo::Prim::Rectangle& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
