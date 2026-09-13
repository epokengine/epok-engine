# PsyQo API: Primitives / Triangles

> **Header:** `"psyqo/primitives/triangles.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh)

This module covers typed PlayStation GPU primitives. It documents 54 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::GouraudTexturedTriangle`, `psyqo::Prim::GouraudTriangle`, `psyqo::Prim::TexturedTriangle`, `psyqo::Prim::Triangle`

## Callable index

- [`psyqo::Prim::GouraudTexturedTriangle::getColorA`](#psyqo-prim-gouraudtexturedtriangle-getcolora-1) — Returns color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::getColorB`](#psyqo-prim-gouraudtexturedtriangle-getcolorb-1) — Returns color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::getColorC`](#psyqo-prim-gouraudtexturedtriangle-getcolorc-1) — Returns color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::getCommandWord`](#psyqo-prim-gouraudtexturedtriangle-getcommandword-1) — The GP0 command word, minus any colour.
- [`psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`](#psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-1) — Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`](#psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-2) — Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`](#psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-3) — Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::interpolateColors`](#psyqo-prim-gouraudtexturedtriangle-interpolatecolors-1) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::interpolateColors`](#psyqo-prim-gouraudtexturedtriangle-interpolatecolors-2) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::isSemiTrans`](#psyqo-prim-gouraudtexturedtriangle-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::setColorA`](#psyqo-prim-gouraudtexturedtriangle-setcolora-1) — Sets color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::setColorAPacked`](#psyqo-prim-gouraudtexturedtriangle-setcolorapacked-1) — Sets the command word and vertex A's colour in one go.
- [`psyqo::Prim::GouraudTexturedTriangle::setColorB`](#psyqo-prim-gouraudtexturedtriangle-setcolorb-1) — Sets color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::setColorC`](#psyqo-prim-gouraudtexturedtriangle-setcolorc-1) — Sets color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::setOpaque`](#psyqo-prim-gouraudtexturedtriangle-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedTriangle::setSemiTrans`](#psyqo-prim-gouraudtexturedtriangle-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::getColorA`](#psyqo-prim-gouraudtriangle-getcolora-1) — Returns color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::getColorB`](#psyqo-prim-gouraudtriangle-getcolorb-1) — Returns color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::getColorC`](#psyqo-prim-gouraudtriangle-getcolorc-1) — Returns color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::getCommandWord`](#psyqo-prim-gouraudtriangle-getcommandword-1) — The GP0 command word, minus any colour.
- [`psyqo::Prim::GouraudTriangle::GouraudTriangle`](#psyqo-prim-gouraudtriangle-gouraudtriangle-1) — Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::GouraudTriangle`](#psyqo-prim-gouraudtriangle-gouraudtriangle-2) — Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::GouraudTriangle`](#psyqo-prim-gouraudtriangle-gouraudtriangle-3) — Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::interpolateColors`](#psyqo-prim-gouraudtriangle-interpolatecolors-1) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::interpolateColors`](#psyqo-prim-gouraudtriangle-interpolatecolors-2) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::isSemiTrans`](#psyqo-prim-gouraudtriangle-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setColorA`](#psyqo-prim-gouraudtriangle-setcolora-1) — Sets color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setColorAPacked`](#psyqo-prim-gouraudtriangle-setcolorapacked-1) — Sets the command word and vertex A's colour in one go.
- [`psyqo::Prim::GouraudTriangle::setColorB`](#psyqo-prim-gouraudtriangle-setcolorb-1) — Sets color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setColorC`](#psyqo-prim-gouraudtriangle-setcolorc-1) — Sets color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setOpaque`](#psyqo-prim-gouraudtriangle-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setPointA`](#psyqo-prim-gouraudtriangle-setpointa-1) — Sets point a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setPointB`](#psyqo-prim-gouraudtriangle-setpointb-1) — Sets point b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setPointC`](#psyqo-prim-gouraudtriangle-setpointc-1) — Sets point c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTriangle::setSemiTrans`](#psyqo-prim-gouraudtriangle-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::getColor`](#psyqo-prim-texturedtriangle-getcolor-1) — Returns color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::isSemiTrans`](#psyqo-prim-texturedtriangle-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::setColor`](#psyqo-prim-texturedtriangle-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::setOpaque`](#psyqo-prim-texturedtriangle-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::setSemiTrans`](#psyqo-prim-texturedtriangle-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::TexturedTriangle`](#psyqo-prim-texturedtriangle-texturedtriangle-1) — Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::TexturedTriangle`](#psyqo-prim-texturedtriangle-texturedtriangle-2) — Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedTriangle::TexturedTriangle`](#psyqo-prim-texturedtriangle-texturedtriangle-3) — Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::getColor`](#psyqo-prim-triangle-getcolor-1) — Returns color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::isSemiTrans`](#psyqo-prim-triangle-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setColor`](#psyqo-prim-triangle-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setOpaque`](#psyqo-prim-triangle-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setPointA`](#psyqo-prim-triangle-setpointa-1) — Sets point a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setPointB`](#psyqo-prim-triangle-setpointb-1) — Sets point b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setPointC`](#psyqo-prim-triangle-setpointc-1) — Sets point c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::setSemiTrans`](#psyqo-prim-triangle-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::Triangle`](#psyqo-prim-triangle-triangle-1) — Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::Triangle`](#psyqo-prim-triangle-triangle-2) — Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Triangle::Triangle`](#psyqo-prim-triangle-triangle-3) — Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.

<a id="psyqo-prim-gouraudtexturedtriangle-getcolora-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::getColorA`

**Purpose.** Returns color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorA() const
```

- **Declared at:** [line 281](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L281)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.getColorA();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-getcolorb-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::getColorB`

**Purpose.** Returns color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorB() const
```

- **Declared at:** [line 282](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L282)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.getColorB();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-getcolorc-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::getColorC`

**Purpose.** Returns color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorC() const
```

- **Declared at:** [line 283](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L283)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.getColorC();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-getcommandword-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::getCommandWord`

**Purpose.** The GP0 command word, minus any colour.

**Details.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Exact declaration**

```cpp
uint32_t getCommandWord() const
```

- **Declared at:** [line 295](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L295)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.getCommandWord();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle() : c
```

- **Declared at:** [line 264](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L264)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-2"></a>

## `psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle(Color c) : c
```

- **Declared at:** [line 265](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L265)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedTriangle value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-gouraudtexturedtriangle-3"></a>

## `psyqo::Prim::GouraudTexturedTriangle::GouraudTexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle(const GouraudTexturedTriangle& other, Color c)
```

- **Declared at:** [line 266](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L266)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const GouraudTexturedTriangle &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// const GouraudTexturedTriangle & other
// Color c

psyqo::Prim::GouraudTexturedTriangle value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtexturedtriangle-interpolatecolors-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(Color a, Color b, Color c)
```

- **Declared at:** [line 330](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L330)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Color` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Color` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// Color a
// Color b
// Color c

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-prim-gouraudtexturedtriangle-interpolatecolors-2"></a>

## `psyqo::Prim::GouraudTexturedTriangle::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(const Color* a, const Color* b, const Color* c)
```

- **Declared at:** [line 313](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L313)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Color *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Color *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Color *` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// const Color * a
// const Color * b
// const Color * c

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtexturedtriangle-issemitrans-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 311](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L311)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-gouraudtexturedtriangle-setcolora-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setColorA`

**Purpose.** Sets color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setColorA(Color c)
```

- **Declared at:** [line 268](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L268)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setColorA(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-setcolorapacked-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setColorAPacked`

**Purpose.** Sets the command word and vertex A's colour in one go.

**Details.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setColorAPacked(uint32_t packed)
```

- **Declared at:** [line 303](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L303)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// uint32_t packed

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setColorAPacked(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-setcolorb-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setColorB`

**Purpose.** Sets color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setColorB(Color c)
```

- **Declared at:** [line 273](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L273)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setColorB(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-setcolorc-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setColorC`

**Purpose.** Sets color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setColorC(Color c)
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L277)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setColorC(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-setopaque-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setOpaque()
```

- **Declared at:** [line 284](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L284)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedtriangle-setsemitrans-1"></a>

## `psyqo::Prim::GouraudTexturedTriangle::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedTriangle& setSemiTrans()
```

- **Declared at:** [line 307](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L307)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-getcolora-1"></a>

## `psyqo::Prim::GouraudTriangle::getColorA`

**Purpose.** Returns color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorA() const
```

- **Declared at:** [line 159](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.getColorA();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-getcolorb-1"></a>

## `psyqo::Prim::GouraudTriangle::getColorB`

**Purpose.** Returns color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorB() const
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L160)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.getColorB();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-getcolorc-1"></a>

## `psyqo::Prim::GouraudTriangle::getColorC`

**Purpose.** Returns color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorC() const
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L161)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.getColorC();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-getcommandword-1"></a>

## `psyqo::Prim::GouraudTriangle::getCommandWord`

**Purpose.** The GP0 command word, minus any colour.

**Details.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Exact declaration**

```cpp
uint32_t getCommandWord() const
```

- **Declared at:** [line 173](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L173)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.getCommandWord();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-gouraudtriangle-1"></a>

## `psyqo::Prim::GouraudTriangle::GouraudTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle() : c
```

- **Declared at:** [line 143](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L143)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-gouraudtriangle-2"></a>

## `psyqo::Prim::GouraudTriangle::GouraudTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle(Color c) : c
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L144)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTriangle value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-gouraudtriangle-3"></a>

## `psyqo::Prim::GouraudTriangle::GouraudTriangle`

**Purpose.** Constructs `psyqo::Prim::GouraudTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle(const GouraudTriangle& other, Color c) : c
```

- **Declared at:** [line 145](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L145)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const GouraudTriangle &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// const GouraudTriangle & other
// Color c

psyqo::Prim::GouraudTriangle value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtriangle-interpolatecolors-1"></a>

## `psyqo::Prim::GouraudTriangle::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(Color a, Color b, Color c)
```

- **Declared at:** [line 220](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L220)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Color` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Color` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// Color a
// Color b
// Color c

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-prim-gouraudtriangle-interpolatecolors-2"></a>

## `psyqo::Prim::GouraudTriangle::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(const Color* a, const Color* b, const Color* c)
```

- **Declared at:** [line 203](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L203)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Color *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Color *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Color *` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// const Color * a
// const Color * b
// const Color * c

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtriangle-issemitrans-1"></a>

## `psyqo::Prim::GouraudTriangle::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 189](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L189)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-gouraudtriangle-setcolora-1"></a>

## `psyqo::Prim::GouraudTriangle::setColorA`

**Purpose.** Sets color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setColorA(Color c)
```

- **Declared at:** [line 146](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L146)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setColorA(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setcolorapacked-1"></a>

## `psyqo::Prim::GouraudTriangle::setColorAPacked`

**Purpose.** Sets the command word and vertex A's colour in one go.

**Details.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Exact declaration**

```cpp
GouraudTriangle& setColorAPacked(uint32_t packed)
```

- **Declared at:** [line 181](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L181)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// uint32_t packed

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setColorAPacked(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setcolorb-1"></a>

## `psyqo::Prim::GouraudTriangle::setColorB`

**Purpose.** Sets color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setColorB(Color c)
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L151)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setColorB(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setcolorc-1"></a>

## `psyqo::Prim::GouraudTriangle::setColorC`

**Purpose.** Sets color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setColorC(Color c)
```

- **Declared at:** [line 155](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L155)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setColorC(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setopaque-1"></a>

## `psyqo::Prim::GouraudTriangle::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setOpaque()
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L162)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setpointa-1"></a>

## `psyqo::Prim::GouraudTriangle::setPointA`

**Purpose.** Sets point a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setPointA(Vertex v)
```

- **Declared at:** [line 190](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L190)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setPointA(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setpointb-1"></a>

## `psyqo::Prim::GouraudTriangle::setPointB`

**Purpose.** Sets point b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setPointB(Vertex v)
```

- **Declared at:** [line 194](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L194)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setPointB(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setpointc-1"></a>

## `psyqo::Prim::GouraudTriangle::setPointC`

**Purpose.** Sets point c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setPointC(Vertex v)
```

- **Declared at:** [line 198](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L198)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setPointC(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtriangle-setsemitrans-1"></a>

## `psyqo::Prim::GouraudTriangle::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTriangle& setSemiTrans()
```

- **Declared at:** [line 185](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L185)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::GouraudTriangle& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-getcolor-1"></a>

## `psyqo::Prim::TexturedTriangle::getColor`

**Purpose.** Returns color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColor() const
```

- **Declared at:** [line 108](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L108)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::TexturedTriangle& object = /* obtain a valid instance */;

auto result = object.getColor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-issemitrans-1"></a>

## `psyqo::Prim::TexturedTriangle::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L117)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::TexturedTriangle& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-texturedtriangle-setcolor-1"></a>

## `psyqo::Prim::TexturedTriangle::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle& setColor(Color c)
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `TexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::TexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-setopaque-1"></a>

## `psyqo::Prim::TexturedTriangle::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle& setOpaque()
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L109)
- **Kind:** `cxx method`

**Returns.** Returns `TexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::TexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-setsemitrans-1"></a>

## `psyqo::Prim::TexturedTriangle::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle& setSemiTrans()
```

- **Declared at:** [line 113](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L113)
- **Kind:** `cxx method`

**Returns.** Returns `TexturedTriangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::TexturedTriangle& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-texturedtriangle-1"></a>

## `psyqo::Prim::TexturedTriangle::TexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle() : c
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L100)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::TexturedTriangle value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-texturedtriangle-2"></a>

## `psyqo::Prim::TexturedTriangle::TexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle(Color c) : c
```

- **Declared at:** [line 101](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L101)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::TexturedTriangle value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedtriangle-texturedtriangle-3"></a>

## `psyqo::Prim::TexturedTriangle::TexturedTriangle`

**Purpose.** Constructs `psyqo::Prim::TexturedTriangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedTriangle(const TexturedTriangle& other, Color c) : c
```

- **Declared at:** [line 102](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L102)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const TexturedTriangle &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// const TexturedTriangle & other
// Color c

psyqo::Prim::TexturedTriangle value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-triangle-getcolor-1"></a>

## `psyqo::Prim::Triangle::getColor`

**Purpose.** Returns color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColor() const
```

- **Declared at:** [line 54](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L54)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.getColor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-issemitrans-1"></a>

## `psyqo::Prim::Triangle::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 63](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L63)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-triangle-setcolor-1"></a>

## `psyqo::Prim::Triangle::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setColor(Color c)
```

- **Declared at:** [line 49](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L49)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-setopaque-1"></a>

## `psyqo::Prim::Triangle::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setOpaque()
```

- **Declared at:** [line 55](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L55)
- **Kind:** `cxx method`

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-setpointa-1"></a>

## `psyqo::Prim::Triangle::setPointA`

**Purpose.** Sets point a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setPointA(Vertex v)
```

- **Declared at:** [line 64](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L64)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setPointA(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-setpointb-1"></a>

## `psyqo::Prim::Triangle::setPointB`

**Purpose.** Sets point b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setPointB(Vertex v)
```

- **Declared at:** [line 68](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setPointB(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-setpointc-1"></a>

## `psyqo::Prim::Triangle::setPointC`

**Purpose.** Sets point c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setPointC(Vertex v)
```

- **Declared at:** [line 72](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L72)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setPointC(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-setsemitrans-1"></a>

## `psyqo::Prim::Triangle::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle& setSemiTrans()
```

- **Declared at:** [line 59](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L59)
- **Kind:** `cxx method`

**Returns.** Returns `Triangle &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::Triangle& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-triangle-1"></a>

## `psyqo::Prim::Triangle::Triangle`

**Purpose.** Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle() : c
```

- **Declared at:** [line 46](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L46)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

psyqo::Prim::Triangle value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-triangle-2"></a>

## `psyqo::Prim::Triangle::Triangle`

**Purpose.** Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle(Color c) : c
```

- **Declared at:** [line 47](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L47)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Triangle value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-triangle-triangle-3"></a>

## `psyqo::Prim::Triangle::Triangle`

**Purpose.** Constructs `psyqo::Prim::Triangle` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Triangle(const Triangle& other, Color c) : c
```

- **Declared at:** [line 48](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/triangles.hh#L48)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const Triangle &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/triangles.hh"

// Assume these named values have been initialized with valid data:
// const Triangle & other
// Color c

psyqo::Prim::Triangle value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
