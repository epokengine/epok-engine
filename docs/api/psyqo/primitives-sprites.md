# PsyQo API: Primitives / Sprites

> **Header:** `"psyqo/primitives/sprites.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh)

This module covers typed PlayStation GPU primitives. It documents 20 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::Sprite`, `psyqo::Prim::Sprite16x16`, `psyqo::Prim::Sprite1x1`, `psyqo::Prim::Sprite8x8`

## Callable index

- [`psyqo::Prim::Sprite16x16::setColor`](#psyqo-prim-sprite16x16-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite16x16::setOpaque`](#psyqo-prim-sprite16x16-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite16x16::setSemiTrans`](#psyqo-prim-sprite16x16-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite16x16::Sprite16x16`](#psyqo-prim-sprite16x16-sprite16x16-1) — Constructs `psyqo::Prim::Sprite16x16` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite16x16::Sprite16x16`](#psyqo-prim-sprite16x16-sprite16x16-2) — Constructs `psyqo::Prim::Sprite16x16` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite1x1::setColor`](#psyqo-prim-sprite1x1-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite1x1::setOpaque`](#psyqo-prim-sprite1x1-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite1x1::setSemiTrans`](#psyqo-prim-sprite1x1-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite1x1::Sprite1x1`](#psyqo-prim-sprite1x1-sprite1x1-1) — Constructs `psyqo::Prim::Sprite1x1` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite1x1::Sprite1x1`](#psyqo-prim-sprite1x1-sprite1x1-2) — Constructs `psyqo::Prim::Sprite1x1` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite8x8::setColor`](#psyqo-prim-sprite8x8-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite8x8::setOpaque`](#psyqo-prim-sprite8x8-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite8x8::setSemiTrans`](#psyqo-prim-sprite8x8-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite8x8::Sprite8x8`](#psyqo-prim-sprite8x8-sprite8x8-1) — Constructs `psyqo::Prim::Sprite8x8` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite8x8::Sprite8x8`](#psyqo-prim-sprite8x8-sprite8x8-2) — Constructs `psyqo::Prim::Sprite8x8` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite::setColor`](#psyqo-prim-sprite-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite::setOpaque`](#psyqo-prim-sprite-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite::setSemiTrans`](#psyqo-prim-sprite-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite::Sprite`](#psyqo-prim-sprite-sprite-1) — Constructs `psyqo::Prim::Sprite` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Sprite::Sprite`](#psyqo-prim-sprite-sprite-2) — Constructs `psyqo::Prim::Sprite` for typed PlayStation GPU primitives.

<a id="psyqo-prim-sprite16x16-setcolor-1"></a>

## `psyqo::Prim::Sprite16x16::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite16x16& setColor(Color c)
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L160)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Sprite16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite16x16& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite16x16-setopaque-1"></a>

## `psyqo::Prim::Sprite16x16::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite16x16& setOpaque()
```

- **Declared at:** [line 165](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L165)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite16x16& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite16x16-setsemitrans-1"></a>

## `psyqo::Prim::Sprite16x16::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite16x16& setSemiTrans()
```

- **Declared at:** [line 169](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L169)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite16x16 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite16x16& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite16x16-sprite16x16-1"></a>

## `psyqo::Prim::Sprite16x16::Sprite16x16`

**Purpose.** Constructs `psyqo::Prim::Sprite16x16` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite16x16() : c
```

- **Declared at:** [line 158](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L158)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite16x16 value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite16x16-sprite16x16-2"></a>

## `psyqo::Prim::Sprite16x16::Sprite16x16`

**Purpose.** Constructs `psyqo::Prim::Sprite16x16` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite16x16(Color c) : c
```

- **Declared at:** [line 159](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L159)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite16x16 value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite1x1-setcolor-1"></a>

## `psyqo::Prim::Sprite1x1::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite1x1& setColor(Color c)
```

- **Declared at:** [line 88](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L88)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Sprite1x1 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite1x1& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite1x1-setopaque-1"></a>

## `psyqo::Prim::Sprite1x1::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite1x1& setOpaque()
```

- **Declared at:** [line 93](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L93)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite1x1 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite1x1& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite1x1-setsemitrans-1"></a>

## `psyqo::Prim::Sprite1x1::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite1x1& setSemiTrans()
```

- **Declared at:** [line 97](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L97)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite1x1 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite1x1& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite1x1-sprite1x1-1"></a>

## `psyqo::Prim::Sprite1x1::Sprite1x1`

**Purpose.** Constructs `psyqo::Prim::Sprite1x1` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite1x1() : c
```

- **Declared at:** [line 86](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L86)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite1x1 value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite1x1-sprite1x1-2"></a>

## `psyqo::Prim::Sprite1x1::Sprite1x1`

**Purpose.** Constructs `psyqo::Prim::Sprite1x1` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite1x1(Color c) : c
```

- **Declared at:** [line 87](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L87)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite1x1 value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite8x8-setcolor-1"></a>

## `psyqo::Prim::Sprite8x8::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite8x8& setColor(Color c)
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L124)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Sprite8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite8x8& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite8x8-setopaque-1"></a>

## `psyqo::Prim::Sprite8x8::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite8x8& setOpaque()
```

- **Declared at:** [line 129](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L129)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite8x8& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite8x8-setsemitrans-1"></a>

## `psyqo::Prim::Sprite8x8::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite8x8& setSemiTrans()
```

- **Declared at:** [line 133](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L133)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite8x8 &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite8x8& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite8x8-sprite8x8-1"></a>

## `psyqo::Prim::Sprite8x8::Sprite8x8`

**Purpose.** Constructs `psyqo::Prim::Sprite8x8` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite8x8() : c
```

- **Declared at:** [line 122](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L122)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite8x8 value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite8x8-sprite8x8-2"></a>

## `psyqo::Prim::Sprite8x8::Sprite8x8`

**Purpose.** Constructs `psyqo::Prim::Sprite8x8` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite8x8(Color c) : c
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L123)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite8x8 value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite-setcolor-1"></a>

## `psyqo::Prim::Sprite::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite& setColor(Color c)
```

- **Declared at:** [line 51](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L51)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Sprite &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite-setopaque-1"></a>

## `psyqo::Prim::Sprite::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite& setOpaque()
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L56)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite-setsemitrans-1"></a>

## `psyqo::Prim::Sprite::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite& setSemiTrans()
```

- **Declared at:** [line 60](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L60)
- **Kind:** `cxx method`

**Returns.** Returns `Sprite &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite-sprite-1"></a>

## `psyqo::Prim::Sprite::Sprite`

**Purpose.** Constructs `psyqo::Prim::Sprite` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite() : c
```

- **Declared at:** [line 49](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L49)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

psyqo::Prim::Sprite value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-sprite-sprite-2"></a>

## `psyqo::Prim::Sprite::Sprite`

**Purpose.** Constructs `psyqo::Prim::Sprite` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Sprite(Color c) : c
```

- **Declared at:** [line 50](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/sprites.hh#L50)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/sprites.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Sprite value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
