# PsyQo API: Primitives / Lines

> **Header:** `"psyqo/primitives/lines.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh)

This module covers typed PlayStation GPU primitives. It documents 23 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::GouraudLine`, `psyqo::Prim::Line`, `psyqo::Prim::PolyLine`, `psyqo::Prim::PolyLineBegin`, `psyqo::Prim::PolyLineEnd`

## Callable index

- [`psyqo::Prim::GouraudLine::getCommandWord`](#psyqo-prim-gouraudline-getcommandword-1) — The GP0 command word, minus any colour.
- [`psyqo::Prim::GouraudLine::GouraudLine`](#psyqo-prim-gouraudline-gouraudline-1) — Constructs `psyqo::Prim::GouraudLine` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudLine::GouraudLine`](#psyqo-prim-gouraudline-gouraudline-2) — Constructs `psyqo::Prim::GouraudLine` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudLine::setColorA`](#psyqo-prim-gouraudline-setcolora-1) — Sets color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudLine::setColorAPacked`](#psyqo-prim-gouraudline-setcolorapacked-1) — Sets the command word and vertex A's colour in one go.
- [`psyqo::Prim::GouraudLine::setColorB`](#psyqo-prim-gouraudline-setcolorb-1) — Sets color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudLine::setOpaque`](#psyqo-prim-gouraudline-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudLine::setSemiTrans`](#psyqo-prim-gouraudline-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Line::Line`](#psyqo-prim-line-line-1) — Constructs `psyqo::Prim::Line` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Line::Line`](#psyqo-prim-line-line-2) — Constructs `psyqo::Prim::Line` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Line::setColor`](#psyqo-prim-line-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Line::setOpaque`](#psyqo-prim-line-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Line::setSemiTrans`](#psyqo-prim-line-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLine::PolyLine<N>`](#psyqo-prim-polyline-polyline-n-1) — Constructs `psyqo::Prim::PolyLine` for typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLine::PolyLine<N>`](#psyqo-prim-polyline-polyline-n-2) — Constructs `psyqo::Prim::PolyLine` for typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLine::setColor`](#psyqo-prim-polyline-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLine::setOpaque`](#psyqo-prim-polyline-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLine::setSemiTrans`](#psyqo-prim-polyline-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLineBegin::PolyLineBegin`](#psyqo-prim-polylinebegin-polylinebegin-1) — Constructs `psyqo::Prim::PolyLineBegin` for typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLineBegin::PolyLineBegin`](#psyqo-prim-polylinebegin-polylinebegin-2) — Constructs `psyqo::Prim::PolyLineBegin` for typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLineBegin::setColor`](#psyqo-prim-polylinebegin-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLineBegin::setOpaque`](#psyqo-prim-polylinebegin-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::PolyLineBegin::setSemiTrans`](#psyqo-prim-polylinebegin-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.

<a id="psyqo-prim-gouraudline-getcommandword-1"></a>

## `psyqo::Prim::GouraudLine::getCommandWord`

**Purpose.** The GP0 command word, minus any colour.

**Details.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Exact declaration**

```cpp
uint32_t getCommandWord() const
```

- **Declared at:** [line 102](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L102)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.getCommandWord();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-gouraudline-1"></a>

## `psyqo::Prim::GouraudLine::GouraudLine`

**Purpose.** Constructs `psyqo::Prim::GouraudLine` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine() : c
```

- **Declared at:** [line 80](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L80)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::GouraudLine value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-gouraudline-2"></a>

## `psyqo::Prim::GouraudLine::GouraudLine`

**Purpose.** Constructs `psyqo::Prim::GouraudLine` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine(Color c) : c
```

- **Declared at:** [line 81](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L81)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudLine value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-setcolora-1"></a>

## `psyqo::Prim::GouraudLine::setColorA`

**Purpose.** Sets color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine& setColorA(Color c)
```

- **Declared at:** [line 82](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L82)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudLine &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.setColorA(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-setcolorapacked-1"></a>

## `psyqo::Prim::GouraudLine::setColorAPacked`

**Purpose.** Sets the command word and vertex A's colour in one go.

**Details.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Exact declaration**

```cpp
GouraudLine& setColorAPacked(uint32_t packed)
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L110)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `GouraudLine &`. Check the purpose and failure notes before using the value.

**Use it when.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// uint32_t packed

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.setColorAPacked(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-setcolorb-1"></a>

## `psyqo::Prim::GouraudLine::setColorB`

**Purpose.** Sets color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine& setColorB(Color c)
```

- **Declared at:** [line 87](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L87)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudLine &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.setColorB(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-setopaque-1"></a>

## `psyqo::Prim::GouraudLine::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine& setOpaque()
```

- **Declared at:** [line 91](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L91)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudLine &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudline-setsemitrans-1"></a>

## `psyqo::Prim::GouraudLine::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudLine& setSemiTrans()
```

- **Declared at:** [line 114](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L114)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudLine &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::GouraudLine& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-line-line-1"></a>

## `psyqo::Prim::Line::Line`

**Purpose.** Constructs `psyqo::Prim::Line` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Line() : c
```

- **Declared at:** [line 45](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L45)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::Line value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-line-line-2"></a>

## `psyqo::Prim::Line::Line`

**Purpose.** Constructs `psyqo::Prim::Line` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Line(Color c) : c
```

- **Declared at:** [line 46](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L46)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Line value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-line-setcolor-1"></a>

## `psyqo::Prim::Line::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Line& setColor(Color c)
```

- **Declared at:** [line 47](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Line &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Line& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-line-setopaque-1"></a>

## `psyqo::Prim::Line::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Line& setOpaque()
```

- **Declared at:** [line 52](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L52)
- **Kind:** `cxx method`

**Returns.** Returns `Line &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::Line& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-line-setsemitrans-1"></a>

## `psyqo::Prim::Line::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Line& setSemiTrans()
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L56)
- **Kind:** `cxx method`

**Returns.** Returns `Line &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::Line& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polyline-polyline-n-1"></a>

## `psyqo::Prim::PolyLine::PolyLine<N>`

**Purpose.** Constructs `psyqo::Prim::PolyLine` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLine() : c
```

- **Declared at:** [line 184](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L184)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLine value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polyline-polyline-n-2"></a>

## `psyqo::Prim::PolyLine::PolyLine<N>`

**Purpose.** Constructs `psyqo::Prim::PolyLine` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLine(Color c) : c
```

- **Declared at:** [line 185](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L185)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::PolyLine value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polyline-setcolor-1"></a>

## `psyqo::Prim::PolyLine::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLine& setColor(Color c)
```

- **Declared at:** [line 186](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L186)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `PolyLine<N> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::PolyLine& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polyline-setopaque-1"></a>

## `psyqo::Prim::PolyLine::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLine& setOpaque()
```

- **Declared at:** [line 191](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L191)
- **Kind:** `cxx method`

**Returns.** Returns `PolyLine<N> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLine& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polyline-setsemitrans-1"></a>

## `psyqo::Prim::PolyLine::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLine& setSemiTrans()
```

- **Declared at:** [line 195](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L195)
- **Kind:** `cxx method`

**Returns.** Returns `PolyLine<N> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLine& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polylinebegin-polylinebegin-1"></a>

## `psyqo::Prim::PolyLineBegin::PolyLineBegin`

**Purpose.** Constructs `psyqo::Prim::PolyLineBegin` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLineBegin() : c
```

- **Declared at:** [line 143](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L143)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLineBegin value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polylinebegin-polylinebegin-2"></a>

## `psyqo::Prim::PolyLineBegin::PolyLineBegin`

**Purpose.** Constructs `psyqo::Prim::PolyLineBegin` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLineBegin(Color c) : c
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L144)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::PolyLineBegin value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polylinebegin-setcolor-1"></a>

## `psyqo::Prim::PolyLineBegin::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLineBegin& setColor(Color c)
```

- **Declared at:** [line 145](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L145)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `PolyLineBegin &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::PolyLineBegin& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polylinebegin-setopaque-1"></a>

## `psyqo::Prim::PolyLineBegin::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLineBegin& setOpaque()
```

- **Declared at:** [line 150](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L150)
- **Kind:** `cxx method`

**Returns.** Returns `PolyLineBegin &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLineBegin& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-polylinebegin-setsemitrans-1"></a>

## `psyqo::Prim::PolyLineBegin::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
PolyLineBegin& setSemiTrans()
```

- **Declared at:** [line 154](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/lines.hh#L154)
- **Kind:** `cxx method`

**Returns.** Returns `PolyLineBegin &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/lines.hh"

psyqo::Prim::PolyLineBegin& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
