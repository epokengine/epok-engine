# PsyQo API: Primitives / Common

> **Header:** `"psyqo/primitives/common.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh)

This module covers typed PlayStation GPU primitives. It documents 27 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Color::(anonymous struct at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\primitives\common.hh:92:5)`, `psyqo::Prim::TPageAttr::ColorMode`, `psyqo::Prim::TPageAttr::SemiTrans`, `psyqo::Prim::Transparency`, `psyqo::PrimPieces::ClutIndex`, `psyqo::PrimPieces::PageInfo`, `psyqo::PrimPieces::TexInfo`, `psyqo::PrimPieces::TPageAttr`, `psyqo::PrimPieces::TPageLoc`, `psyqo::PrimPieces::UVCoords`, `psyqo::PrimPieces::UVCoordsPadded`, `psyqo::Rect`, `psyqo::Vertex::(anonymous struct at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\primitives\common.hh:48:5)`

## Callable index

- [`psyqo::PrimPieces::ClutIndex::ClutIndex`](#psyqo-primpieces-clutindex-clutindex-1) — Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::ClutIndex::ClutIndex`](#psyqo-primpieces-clutindex-clutindex-2) — Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::ClutIndex::ClutIndex`](#psyqo-primpieces-clutindex-clutindex-3) — Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::copy`](#psyqo-primpieces-tpageattr-copy-1) — Performs `copy` as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::disableDisplayArea`](#psyqo-primpieces-tpageattr-disabledisplayarea-1) — Performs `disable display area` as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::enableDisplayArea`](#psyqo-primpieces-tpageattr-enabledisplayarea-1) — Performs `enable display area` as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::getColorMode`](#psyqo-primpieces-tpageattr-getcolormode-1) — Returns color mode as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::getPageLoc`](#psyqo-primpieces-tpageattr-getpageloc-1) — Returns page loc as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::getPageX`](#psyqo-primpieces-tpageattr-getpagex-1) — Returns page x as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::getPageY`](#psyqo-primpieces-tpageattr-getpagey-1) — Returns page y as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::getSemiTrans`](#psyqo-primpieces-tpageattr-getsemitrans-1) — Returns semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::hasDithering`](#psyqo-primpieces-tpageattr-hasdithering-1) — Reports whether dithering as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::isDisplayAreaEnabled`](#psyqo-primpieces-tpageattr-isdisplayareaenabled-1) — Reports whether display area enabled as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::operator=`](#psyqo-primpieces-tpageattr-operator-1) — Performs `operator =` as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::operator=`](#psyqo-primpieces-tpageattr-operator-2) — Performs `operator =` as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::set`](#psyqo-primpieces-tpageattr-set-1) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::set`](#psyqo-primpieces-tpageattr-set-2) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::setDithering`](#psyqo-primpieces-tpageattr-setdithering-1) — Sets dithering as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::setPageLoc`](#psyqo-primpieces-tpageattr-setpageloc-1) — Sets page loc as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::setPageX`](#psyqo-primpieces-tpageattr-setpagex-1) — Sets page x as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::setPageY`](#psyqo-primpieces-tpageattr-setpagey-1) — Sets page y as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::TPageAttr`](#psyqo-primpieces-tpageattr-tpageattr-1) — Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::TPageAttr`](#psyqo-primpieces-tpageattr-tpageattr-2) — Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageAttr::TPageAttr`](#psyqo-primpieces-tpageattr-tpageattr-3) — Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageLoc::setPageX`](#psyqo-primpieces-tpageloc-setpagex-1) — Sets page x as part of typed PlayStation GPU primitives.
- [`psyqo::PrimPieces::TPageLoc::setPageY`](#psyqo-primpieces-tpageloc-setpagey-1) — Sets page y as part of typed PlayStation GPU primitives.
- [`psyqo::Rect::isEmpty`](#psyqo-rect-isempty-1) — Reports whether empty as part of typed PlayStation GPU primitives.

<a id="psyqo-primpieces-clutindex-clutindex-1"></a>

## `psyqo::PrimPieces::ClutIndex::ClutIndex`

**Purpose.** Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
ClutIndex()
```

- **Declared at:** [line 122](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L122)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::ClutIndex value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-clutindex-clutindex-2"></a>

## `psyqo::PrimPieces::ClutIndex::ClutIndex`

**Purpose.** Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
ClutIndex(Vertex v) : C
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L123)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::PrimPieces::ClutIndex value(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-clutindex-clutindex-3"></a>

## `psyqo::PrimPieces::ClutIndex::ClutIndex`

**Purpose.** Constructs `psyqo::PrimPieces::ClutIndex` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
ClutIndex(uint16_t x, uint16_t y) : i
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L124)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `uint16_t` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `uint16_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// uint16_t x
// uint16_t y

psyqo::PrimPieces::ClutIndex value(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-copy-1"></a>

## `psyqo::PrimPieces::TPageAttr::copy`

**Purpose.** Performs `copy` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& copy(const TPageAttr& other)
```

- **Declared at:** [line 200](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L200)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const TPageAttr &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// const TPageAttr & other

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.copy(other);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-primpieces-tpageattr-disabledisplayarea-1"></a>

## `psyqo::PrimPieces::TPageAttr::disableDisplayArea`

**Purpose.** Performs `disable display area` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& disableDisplayArea()
```

- **Declared at:** [line 241](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L241)
- **Kind:** `cxx method`

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.disableDisplayArea();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-enabledisplayarea-1"></a>

## `psyqo::PrimPieces::TPageAttr::enableDisplayArea`

**Purpose.** Performs `enable display area` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& enableDisplayArea()
```

- **Declared at:** [line 245](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L245)
- **Kind:** `cxx method`

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.enableDisplayArea();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-getcolormode-1"></a>

## `psyqo::PrimPieces::TPageAttr::getColorMode`

**Purpose.** Returns color mode as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Prim::TPageAttr::ColorMode getColorMode() const
```

- **Declared at:** [line 255](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L255)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Prim::TPageAttr::ColorMode`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.getColorMode();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-getpageloc-1"></a>

## `psyqo::PrimPieces::TPageAttr::getPageLoc`

**Purpose.** Returns page loc as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageLoc getPageLoc() const
```

- **Declared at:** [line 251](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L251)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `TPageLoc`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.getPageLoc();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-getpagex-1"></a>

## `psyqo::PrimPieces::TPageAttr::getPageX`

**Purpose.** Returns page x as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
uint8_t getPageX() const
```

- **Declared at:** [line 249](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L249)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.getPageX();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-getpagey-1"></a>

## `psyqo::PrimPieces::TPageAttr::getPageY`

**Purpose.** Returns page y as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
uint8_t getPageY() const
```

- **Declared at:** [line 250](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L250)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.getPageY();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-getsemitrans-1"></a>

## `psyqo::PrimPieces::TPageAttr::getSemiTrans`

**Purpose.** Returns semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Prim::TPageAttr::SemiTrans getSemiTrans() const
```

- **Declared at:** [line 252](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L252)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Prim::TPageAttr::SemiTrans`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.getSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-hasdithering-1"></a>

## `psyqo::PrimPieces::TPageAttr::hasDithering`

**Purpose.** Reports whether dithering as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool hasDithering() const
```

- **Declared at:** [line 258](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L258)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.hasDithering();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-primpieces-tpageattr-isdisplayareaenabled-1"></a>

## `psyqo::PrimPieces::TPageAttr::isDisplayAreaEnabled`

**Purpose.** Reports whether display area enabled as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isDisplayAreaEnabled() const
```

- **Declared at:** [line 259](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L259)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.isDisplayAreaEnabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-primpieces-tpageattr-operator-1"></a>

## `psyqo::PrimPieces::TPageAttr::operator=`

**Purpose.** Performs `operator =` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& operator=(TPageAttr&& other)
```

- **Declared at:** [line 192](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L192)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `TPageAttr &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// TPageAttr && other

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.operator=(other);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-primpieces-tpageattr-operator-2"></a>

## `psyqo::PrimPieces::TPageAttr::operator=`

**Purpose.** Performs `operator =` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& operator=(const TPageAttr& other)
```

- **Declared at:** [line 196](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L196)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const TPageAttr &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// const TPageAttr & other

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.operator=(other);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-primpieces-tpageattr-set-1"></a>

## `psyqo::PrimPieces::TPageAttr::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& set(Prim::TPageAttr::ColorMode mode)
```

- **Declared at:** [line 227](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L227)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mode` | `Prim::TPageAttr::ColorMode` | Input | Value supplied for `mode`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// Prim::TPageAttr::ColorMode mode

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.set(mode);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-set-2"></a>

## `psyqo::PrimPieces::TPageAttr::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& set(Prim::TPageAttr::SemiTrans trans)
```

- **Declared at:** [line 221](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L221)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `trans` | `Prim::TPageAttr::SemiTrans` | Input | Value supplied for `trans`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// Prim::TPageAttr::SemiTrans trans

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.set(trans);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-setdithering-1"></a>

## `psyqo::PrimPieces::TPageAttr::setDithering`

**Purpose.** Sets dithering as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& setDithering(bool dithering)
```

- **Declared at:** [line 233](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L233)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dithering` | `bool` | Input | Value supplied for `dithering`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// bool dithering

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.setDithering(dithering);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-setpageloc-1"></a>

## `psyqo::PrimPieces::TPageAttr::setPageLoc`

**Purpose.** Sets page loc as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& setPageLoc(TPageLoc loc)
```

- **Declared at:** [line 216](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L216)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `loc` | `TPageLoc` | Input | Value supplied for `loc`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// TPageLoc loc

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.setPageLoc(loc);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-setpagex-1"></a>

## `psyqo::PrimPieces::TPageAttr::setPageX`

**Purpose.** Sets page x as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& setPageX(uint8_t x)
```

- **Declared at:** [line 204](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L204)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `uint8_t` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// uint8_t x

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.setPageX(x);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-setpagey-1"></a>

## `psyqo::PrimPieces::TPageAttr::setPageY`

**Purpose.** Sets page y as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr& setPageY(uint8_t y)
```

- **Declared at:** [line 210](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L210)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `y` | `uint8_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `TPageAttr &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// uint8_t y

psyqo::PrimPieces::TPageAttr& object = /* obtain a valid instance */;

auto result = object.setPageY(y);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-tpageattr-1"></a>

## `psyqo::PrimPieces::TPageAttr::TPageAttr`

**Purpose.** Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr()
```

- **Declared at:** [line 189](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L189)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::PrimPieces::TPageAttr value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageattr-tpageattr-2"></a>

## `psyqo::PrimPieces::TPageAttr::TPageAttr`

**Purpose.** Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr(TPageAttr&& other) : i
```

- **Declared at:** [line 190](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L190)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `TPageAttr &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// TPageAttr && other

psyqo::PrimPieces::TPageAttr value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-primpieces-tpageattr-tpageattr-3"></a>

## `psyqo::PrimPieces::TPageAttr::TPageAttr`

**Purpose.** Constructs `psyqo::PrimPieces::TPageAttr` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageAttr(const TPageAttr& other) : i
```

- **Declared at:** [line 191](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L191)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const TPageAttr &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// const TPageAttr & other

psyqo::PrimPieces::TPageAttr value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-primpieces-tpageloc-setpagex-1"></a>

## `psyqo::PrimPieces::TPageLoc::setPageX`

**Purpose.** Sets page x as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageLoc& setPageX(uint8_t x)
```

- **Declared at:** [line 155](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L155)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `uint8_t` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `TPageLoc &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// uint8_t x

psyqo::PrimPieces::TPageLoc& object = /* obtain a valid instance */;

auto result = object.setPageX(x);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-primpieces-tpageloc-setpagey-1"></a>

## `psyqo::PrimPieces::TPageLoc::setPageY`

**Purpose.** Sets page y as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPageLoc& setPageY(uint8_t y)
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L161)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `y` | `uint8_t` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** Returns `TPageLoc &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

// Assume these named values have been initialized with valid data:
// uint8_t y

psyqo::PrimPieces::TPageLoc& object = /* obtain a valid instance */;

auto result = object.setPageY(y);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-rect-isempty-1"></a>

## `psyqo::Rect::isEmpty`

**Purpose.** Reports whether empty as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isEmpty()
```

- **Declared at:** [line 81](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/common.hh#L81)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/common.hh"

psyqo::Rect& object = /* obtain a valid instance */;

auto result = object.isEmpty();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
