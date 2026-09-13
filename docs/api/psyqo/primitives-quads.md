# PsyQo API: Primitives / Quads

> **Header:** `"psyqo/primitives/quads.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh)

This module covers typed PlayStation GPU primitives. It documents 60 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::GouraudQuad`, `psyqo::Prim::GouraudTexturedQuad`, `psyqo::Prim::Quad`, `psyqo::Prim::TexturedQuad`

## Callable index

- [`psyqo::Prim::GouraudQuad::getColorA`](#psyqo-prim-gouraudquad-getcolora-1) — Returns color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::getColorB`](#psyqo-prim-gouraudquad-getcolorb-1) — Returns color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::getColorC`](#psyqo-prim-gouraudquad-getcolorc-1) — Returns color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::getColorD`](#psyqo-prim-gouraudquad-getcolord-1) — Returns color d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::getCommandWord`](#psyqo-prim-gouraudquad-getcommandword-1) — The GP0 command word, minus any colour.
- [`psyqo::Prim::GouraudQuad::GouraudQuad`](#psyqo-prim-gouraudquad-gouraudquad-1) — Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::GouraudQuad`](#psyqo-prim-gouraudquad-gouraudquad-2) — Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::GouraudQuad`](#psyqo-prim-gouraudquad-gouraudquad-3) — Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::interpolateColors`](#psyqo-prim-gouraudquad-interpolatecolors-1) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::interpolateColors`](#psyqo-prim-gouraudquad-interpolatecolors-2) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::isSemiTrans`](#psyqo-prim-gouraudquad-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setColorA`](#psyqo-prim-gouraudquad-setcolora-1) — Sets color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setColorAPacked`](#psyqo-prim-gouraudquad-setcolorapacked-1) — Sets the command word and vertex A's colour in one go.
- [`psyqo::Prim::GouraudQuad::setColorB`](#psyqo-prim-gouraudquad-setcolorb-1) — Sets color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setColorC`](#psyqo-prim-gouraudquad-setcolorc-1) — Sets color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setColorD`](#psyqo-prim-gouraudquad-setcolord-1) — Sets color d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setOpaque`](#psyqo-prim-gouraudquad-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setPointA`](#psyqo-prim-gouraudquad-setpointa-1) — Sets point a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setPointB`](#psyqo-prim-gouraudquad-setpointb-1) — Sets point b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setPointC`](#psyqo-prim-gouraudquad-setpointc-1) — Sets point c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setPointD`](#psyqo-prim-gouraudquad-setpointd-1) — Sets point d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudQuad::setSemiTrans`](#psyqo-prim-gouraudquad-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::getColorA`](#psyqo-prim-gouraudtexturedquad-getcolora-1) — Returns color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::getColorB`](#psyqo-prim-gouraudtexturedquad-getcolorb-1) — Returns color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::getColorC`](#psyqo-prim-gouraudtexturedquad-getcolorc-1) — Returns color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::getColorD`](#psyqo-prim-gouraudtexturedquad-getcolord-1) — Returns color d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::getCommandWord`](#psyqo-prim-gouraudtexturedquad-getcommandword-1) — The GP0 command word, minus any colour.
- [`psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`](#psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-1) — Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`](#psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-2) — Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`](#psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-3) — Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::interpolateColors`](#psyqo-prim-gouraudtexturedquad-interpolatecolors-1) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::interpolateColors`](#psyqo-prim-gouraudtexturedquad-interpolatecolors-2) — Performs `interpolate colors` as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::isSemiTrans`](#psyqo-prim-gouraudtexturedquad-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setColorA`](#psyqo-prim-gouraudtexturedquad-setcolora-1) — Sets color a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setColorAPacked`](#psyqo-prim-gouraudtexturedquad-setcolorapacked-1) — Sets the command word and vertex A's colour in one go.
- [`psyqo::Prim::GouraudTexturedQuad::setColorB`](#psyqo-prim-gouraudtexturedquad-setcolorb-1) — Sets color b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setColorC`](#psyqo-prim-gouraudtexturedquad-setcolorc-1) — Sets color c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setColorD`](#psyqo-prim-gouraudtexturedquad-setcolord-1) — Sets color d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setOpaque`](#psyqo-prim-gouraudtexturedquad-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::GouraudTexturedQuad::setSemiTrans`](#psyqo-prim-gouraudtexturedquad-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::getColor`](#psyqo-prim-quad-getcolor-1) — Returns color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::isSemiTrans`](#psyqo-prim-quad-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::Quad`](#psyqo-prim-quad-quad-1) — Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::Quad`](#psyqo-prim-quad-quad-2) — Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::Quad`](#psyqo-prim-quad-quad-3) — Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setColor`](#psyqo-prim-quad-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setOpaque`](#psyqo-prim-quad-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setPointA`](#psyqo-prim-quad-setpointa-1) — Sets point a as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setPointB`](#psyqo-prim-quad-setpointb-1) — Sets point b as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setPointC`](#psyqo-prim-quad-setpointc-1) — Sets point c as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setPointD`](#psyqo-prim-quad-setpointd-1) — Sets point d as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::Quad::setSemiTrans`](#psyqo-prim-quad-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::getColor`](#psyqo-prim-texturedquad-getcolor-1) — Returns color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::isSemiTrans`](#psyqo-prim-texturedquad-issemitrans-1) — Reports whether semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::setColor`](#psyqo-prim-texturedquad-setcolor-1) — Sets color as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::setOpaque`](#psyqo-prim-texturedquad-setopaque-1) — Sets opaque as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::setSemiTrans`](#psyqo-prim-texturedquad-setsemitrans-1) — Sets semi trans as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::TexturedQuad`](#psyqo-prim-texturedquad-texturedquad-1) — Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::TexturedQuad`](#psyqo-prim-texturedquad-texturedquad-2) — Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.
- [`psyqo::Prim::TexturedQuad::TexturedQuad`](#psyqo-prim-texturedquad-texturedquad-3) — Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.

<a id="psyqo-prim-gouraudquad-getcolora-1"></a>

## `psyqo::Prim::GouraudQuad::getColorA`

**Purpose.** Returns color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorA() const
```

- **Declared at:** [line 176](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L176)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.getColorA();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-getcolorb-1"></a>

## `psyqo::Prim::GouraudQuad::getColorB`

**Purpose.** Returns color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorB() const
```

- **Declared at:** [line 177](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L177)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.getColorB();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-getcolorc-1"></a>

## `psyqo::Prim::GouraudQuad::getColorC`

**Purpose.** Returns color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorC() const
```

- **Declared at:** [line 178](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L178)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.getColorC();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-getcolord-1"></a>

## `psyqo::Prim::GouraudQuad::getColorD`

**Purpose.** Returns color d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorD() const
```

- **Declared at:** [line 179](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L179)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.getColorD();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-getcommandword-1"></a>

## `psyqo::Prim::GouraudQuad::getCommandWord`

**Purpose.** The GP0 command word, minus any colour.

**Details.** A GP0 polygon's first word is the command in bits 31-24 and vertex A's colour in 23-0, and the GTE's RGBC register has an 8 bit CODE field that it fuses into every colour it produces - which is exactly what that field is for. Load this into CODE before an nc* command and the colour FIFO hands back finished first words; then store them with setColorAPacked and the whole thing is a word store instead of a read-modify-write per vertex.

**Exact declaration**

```cpp
uint32_t getCommandWord() const
```

- **Declared at:** [line 195](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L195)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** A GP0 polygon's first word is the command in bits 31-24 and vertex A's colour in 23-0, and the GTE's RGBC register has an 8 bit CODE field that it fuses into every colour it produces - which is exactly what that field is for. Load this into CODE before an nc* command and the colour FIFO hands back finished first words; then store them with setColorAPacked and the whole thing is a word store instead of a read-modify-write per vertex.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.getCommandWord();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-gouraudquad-1"></a>

## `psyqo::Prim::GouraudQuad::GouraudQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad() : c
```

- **Declared at:** [line 156](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L156)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-gouraudquad-2"></a>

## `psyqo::Prim::GouraudQuad::GouraudQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad(Color c) : c
```

- **Declared at:** [line 157](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L157)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudQuad value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-gouraudquad-3"></a>

## `psyqo::Prim::GouraudQuad::GouraudQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad(const GouraudQuad& other, Color c) : c
```

- **Declared at:** [line 158](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L158)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const GouraudQuad &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// const GouraudQuad & other
// Color c

psyqo::Prim::GouraudQuad value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudquad-interpolatecolors-1"></a>

## `psyqo::Prim::GouraudQuad::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(Color a, Color b, Color c, Color d)
```

- **Declared at:** [line 250](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L250)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Color` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Color` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `Color` | Input | Value supplied for `d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// Color a
// Color b
// Color c
// Color d

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c, d);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-prim-gouraudquad-interpolatecolors-2"></a>

## `psyqo::Prim::GouraudQuad::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(const Color* a, const Color* b, const Color* c, const Color* d)
```

- **Declared at:** [line 229](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L229)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Color *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Color *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Color *` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `const Color *` | Input | Value supplied for `d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// const Color * a
// const Color * b
// const Color * c
// const Color * d

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c, d);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudquad-issemitrans-1"></a>

## `psyqo::Prim::GouraudQuad::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 211](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L211)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-gouraudquad-setcolora-1"></a>

## `psyqo::Prim::GouraudQuad::setColorA`

**Purpose.** Sets color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setColorA(Color c)
```

- **Declared at:** [line 159](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L159)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setColorA(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setcolorapacked-1"></a>

## `psyqo::Prim::GouraudQuad::setColorAPacked`

**Purpose.** Sets the command word and vertex A's colour in one go.

**Details.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Exact declaration**

```cpp
GouraudQuad& setColorAPacked(uint32_t packed)
```

- **Declared at:** [line 203](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L203)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// uint32_t packed

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setColorAPacked(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setcolorb-1"></a>

## `psyqo::Prim::GouraudQuad::setColorB`

**Purpose.** Sets color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setColorB(Color c)
```

- **Declared at:** [line 164](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L164)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setColorB(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setcolorc-1"></a>

## `psyqo::Prim::GouraudQuad::setColorC`

**Purpose.** Sets color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setColorC(Color c)
```

- **Declared at:** [line 168](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L168)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setColorC(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setcolord-1"></a>

## `psyqo::Prim::GouraudQuad::setColorD`

**Purpose.** Sets color d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setColorD(Color c)
```

- **Declared at:** [line 172](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L172)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setColorD(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setopaque-1"></a>

## `psyqo::Prim::GouraudQuad::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setOpaque()
```

- **Declared at:** [line 180](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L180)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setpointa-1"></a>

## `psyqo::Prim::GouraudQuad::setPointA`

**Purpose.** Sets point a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setPointA(Vertex v)
```

- **Declared at:** [line 212](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L212)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setPointA(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setpointb-1"></a>

## `psyqo::Prim::GouraudQuad::setPointB`

**Purpose.** Sets point b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setPointB(Vertex v)
```

- **Declared at:** [line 216](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L216)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setPointB(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setpointc-1"></a>

## `psyqo::Prim::GouraudQuad::setPointC`

**Purpose.** Sets point c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setPointC(Vertex v)
```

- **Declared at:** [line 220](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L220)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setPointC(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setpointd-1"></a>

## `psyqo::Prim::GouraudQuad::setPointD`

**Purpose.** Sets point d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setPointD(Vertex v)
```

- **Declared at:** [line 224](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L224)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setPointD(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudquad-setsemitrans-1"></a>

## `psyqo::Prim::GouraudQuad::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudQuad& setSemiTrans()
```

- **Declared at:** [line 207](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L207)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudQuad& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-getcolora-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::getColorA`

**Purpose.** Returns color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorA() const
```

- **Declared at:** [line 321](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L321)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.getColorA();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-getcolorb-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::getColorB`

**Purpose.** Returns color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorB() const
```

- **Declared at:** [line 322](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L322)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.getColorB();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-getcolorc-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::getColorC`

**Purpose.** Returns color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorC() const
```

- **Declared at:** [line 323](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L323)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.getColorC();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-getcolord-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::getColorD`

**Purpose.** Returns color d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColorD() const
```

- **Declared at:** [line 324](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L324)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.getColorD();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-getcommandword-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::getCommandWord`

**Purpose.** The GP0 command word, minus any colour.

**Details.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Exact declaration**

```cpp
uint32_t getCommandWord() const
```

- **Declared at:** [line 336](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L336)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Meant for the GTE's RGBC CODE field, which gets fused into every colour the GTE emits. Preload it and the colour FIFO hands back finished first words. See `GouraudQuad::getCommandWord` for the full round trip.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.getCommandWord();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad() : c
```

- **Declared at:** [line 301](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L301)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-2"></a>

## `psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad(Color c) : c
```

- **Declared at:** [line 302](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L302)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedQuad value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-gouraudtexturedquad-3"></a>

## `psyqo::Prim::GouraudTexturedQuad::GouraudTexturedQuad`

**Purpose.** Constructs `psyqo::Prim::GouraudTexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad(const GouraudTexturedQuad& other, Color c) : c
```

- **Declared at:** [line 303](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L303)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const GouraudTexturedQuad &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// const GouraudTexturedQuad & other
// Color c

psyqo::Prim::GouraudTexturedQuad value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtexturedquad-interpolatecolors-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(Color a, Color b, Color c, Color d)
```

- **Declared at:** [line 375](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L375)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Color` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Color` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `Color` | Input | Value supplied for `d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// Color a
// Color b
// Color c
// Color d

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c, d);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-prim-gouraudtexturedquad-interpolatecolors-2"></a>

## `psyqo::Prim::GouraudTexturedQuad::interpolateColors`

**Purpose.** Performs `interpolate colors` as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
template <Transparency transparency = Transparency::Auto> void interpolateColors(const Color* a, const Color* b, const Color* c, const Color* d)
```

- **Declared at:** [line 354](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L354)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Color *` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Color *` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Color *` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `const Color *` | Input | Value supplied for `d`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Replace these template arguments with types or values accepted by the declaration:
// transparency

// Assume these named values have been initialized with valid data:
// const Color * a
// const Color * b
// const Color * c
// const Color * d

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

object.interpolateColors<transparency>(a, b, c, d);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-gouraudtexturedquad-issemitrans-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 352](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L352)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-gouraudtexturedquad-setcolora-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setColorA`

**Purpose.** Sets color a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setColorA(Color c)
```

- **Declared at:** [line 304](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L304)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColorA(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setcolorapacked-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setColorAPacked`

**Purpose.** Sets the command word and vertex A's colour in one go.

**Details.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Exact declaration**

```cpp
GouraudTexturedQuad& setColorAPacked(uint32_t packed)
```

- **Declared at:** [line 344](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L344)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `packed` | `uint32_t` | Input | Value supplied for `packed`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** For a value that came out of the GTE with CODE preloaded from getCommandWord. Unlike setColorA this does not preserve the transparency bit, because the value being stored already carries it.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// uint32_t packed

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColorAPacked(packed);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setcolorb-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setColorB`

**Purpose.** Sets color b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setColorB(Color c)
```

- **Declared at:** [line 309](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L309)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColorB(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setcolorc-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setColorC`

**Purpose.** Sets color c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setColorC(Color c)
```

- **Declared at:** [line 313](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L313)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColorC(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setcolord-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setColorD`

**Purpose.** Sets color d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setColorD(Color c)
```

- **Declared at:** [line 317](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L317)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColorD(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setopaque-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setOpaque()
```

- **Declared at:** [line 325](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L325)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-gouraudtexturedquad-setsemitrans-1"></a>

## `psyqo::Prim::GouraudTexturedQuad::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
GouraudTexturedQuad& setSemiTrans()
```

- **Declared at:** [line 348](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L348)
- **Kind:** `cxx method`

**Returns.** Returns `GouraudTexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::GouraudTexturedQuad& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-getcolor-1"></a>

## `psyqo::Prim::Quad::getColor`

**Purpose.** Returns color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColor() const
```

- **Declared at:** [line 61](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L61)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.getColor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-issemitrans-1"></a>

## `psyqo::Prim::Quad::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 70](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L70)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-quad-quad-1"></a>

## `psyqo::Prim::Quad::Quad`

**Purpose.** Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad() : c
```

- **Declared at:** [line 53](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L53)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::Quad value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-quad-2"></a>

## `psyqo::Prim::Quad::Quad`

**Purpose.** Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad(Color c) : c
```

- **Declared at:** [line 54](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L54)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Quad value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-quad-3"></a>

## `psyqo::Prim::Quad::Quad`

**Purpose.** Constructs `psyqo::Prim::Quad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad(const Quad& other, Color c) : c
```

- **Declared at:** [line 55](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L55)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const Quad &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// const Quad & other
// Color c

psyqo::Prim::Quad value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-prim-quad-setcolor-1"></a>

## `psyqo::Prim::Quad::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setColor(Color c)
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setopaque-1"></a>

## `psyqo::Prim::Quad::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setOpaque()
```

- **Declared at:** [line 62](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L62)
- **Kind:** `cxx method`

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setpointa-1"></a>

## `psyqo::Prim::Quad::setPointA`

**Purpose.** Sets point a as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setPointA(Vertex v)
```

- **Declared at:** [line 71](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L71)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setPointA(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setpointb-1"></a>

## `psyqo::Prim::Quad::setPointB`

**Purpose.** Sets point b as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setPointB(Vertex v)
```

- **Declared at:** [line 75](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L75)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setPointB(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setpointc-1"></a>

## `psyqo::Prim::Quad::setPointC`

**Purpose.** Sets point c as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setPointC(Vertex v)
```

- **Declared at:** [line 79](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L79)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setPointC(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setpointd-1"></a>

## `psyqo::Prim::Quad::setPointD`

**Purpose.** Sets point d as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setPointD(Vertex v)
```

- **Declared at:** [line 83](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L83)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vertex` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Vertex v

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setPointD(v);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-quad-setsemitrans-1"></a>

## `psyqo::Prim::Quad::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Quad& setSemiTrans()
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L66)
- **Kind:** `cxx method`

**Returns.** Returns `Quad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::Quad& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-getcolor-1"></a>

## `psyqo::Prim::TexturedQuad::getColor`

**Purpose.** Returns color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
Color getColor() const
```

- **Declared at:** [line 119](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L119)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::TexturedQuad& object = /* obtain a valid instance */;

auto result = object.getColor();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-issemitrans-1"></a>

## `psyqo::Prim::TexturedQuad::isSemiTrans`

**Purpose.** Reports whether semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
bool isSemiTrans() const
```

- **Declared at:** [line 128](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L128)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::TexturedQuad& object = /* obtain a valid instance */;

auto result = object.isSemiTrans();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-prim-texturedquad-setcolor-1"></a>

## `psyqo::Prim::TexturedQuad::setColor`

**Purpose.** Sets color as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad& setColor(Color c)
```

- **Declared at:** [line 114](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L114)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `TexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::TexturedQuad& object = /* obtain a valid instance */;

auto result = object.setColor(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-setopaque-1"></a>

## `psyqo::Prim::TexturedQuad::setOpaque`

**Purpose.** Sets opaque as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad& setOpaque()
```

- **Declared at:** [line 120](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L120)
- **Kind:** `cxx method`

**Returns.** Returns `TexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::TexturedQuad& object = /* obtain a valid instance */;

auto result = object.setOpaque();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-setsemitrans-1"></a>

## `psyqo::Prim::TexturedQuad::setSemiTrans`

**Purpose.** Sets semi trans as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad& setSemiTrans()
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L124)
- **Kind:** `cxx method`

**Returns.** Returns `TexturedQuad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::TexturedQuad& object = /* obtain a valid instance */;

auto result = object.setSemiTrans();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-texturedquad-1"></a>

## `psyqo::Prim::TexturedQuad::TexturedQuad`

**Purpose.** Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad() : c
```

- **Declared at:** [line 111](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L111)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

psyqo::Prim::TexturedQuad value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-texturedquad-2"></a>

## `psyqo::Prim::TexturedQuad::TexturedQuad`

**Purpose.** Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad(Color c) : c
```

- **Declared at:** [line 112](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L112)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// Color c

psyqo::Prim::TexturedQuad value(c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-texturedquad-texturedquad-3"></a>

## `psyqo::Prim::TexturedQuad::TexturedQuad`

**Purpose.** Constructs `psyqo::Prim::TexturedQuad` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TexturedQuad(const TexturedQuad& other, Color c) : c
```

- **Declared at:** [line 113](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/quads.hh#L113)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const TexturedQuad &` | Input | Value supplied for `other`. See the exact type and module contract. |
| `c` | `Color` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/quads.hh"

// Assume these named values have been initialized with valid data:
// const TexturedQuad & other
// Color c

psyqo::Prim::TexturedQuad value(other, c);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
