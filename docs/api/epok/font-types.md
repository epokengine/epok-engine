# Epok API: Font Types

> **Header:** `"font_types.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/font_types.hpp)

This module covers GPU text rendering with the built-in or uploaded font atlas. It documents 3 public callables declared directly in this header.

## Declared types

`epok::Font`, `epok::GlyphMetric`

## Callable index

- [`epok::builtin_cell`](#epok-builtin-cell-1) — The cell mapping src/bitmap_font.rs bakes into the built-in atlas: ASCII 32..126 in order, then the sixteen Spanish extras, and anything else draws '?'.
- [`epok::find_glyph`](#epok-find-glyph-1) — Metrics are sorted by codepoint at import, so a miss costs log2(count) reads.
- [`epok::utf8_scalar`](#epok-utf8-scalar-1) — One UTF-8 scalar from `text`, advancing `n` past it.

<a id="epok-builtin-cell-1"></a>

## `epok::builtin_cell`

**Purpose.** The cell mapping src/bitmap_font.rs bakes into the built-in atlas: ASCII 32..126 in order, then the sixteen Spanish extras, and anything else draws '?'.

**Details.** Font index -1 is this atlas; only authored fonts go through `Font`.

**Exact declaration**

```cpp
inline int builtin_cell(uint32_t codepoint)
```

- **Declared at:** [line 40](../../../runtime/font_types.hpp#L40)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `codepoint` | `uint32_t` | Input | Value supplied for `codepoint`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Font index -1 is this atlas; only authored fonts go through `Font`.

**Usage pattern**

```cpp
#include "font_types.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t codepoint

auto result = epok::builtin_cell(codepoint);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-find-glyph-1"></a>

## `epok::find_glyph`

**Purpose.** Metrics are sorted by codepoint at import, so a miss costs log2(count) reads.

**Exact declaration**

```cpp
inline const GlyphMetric* find_glyph(const Font& font,uint32_t codepoint)
```

- **Declared at:** [line 15](../../../runtime/font_types.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `font` | `const Font &` | Input | Value supplied for `font`. See the exact type and module contract. |
| `codepoint` | `uint32_t` | Input | Value supplied for `codepoint`. See the exact type and module contract. |

**Returns.** Returns `const GlyphMetric *`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "font_types.hpp"

// Assume these named values have been initialized with valid data:
// const Font & font
// uint32_t codepoint

auto result = epok::find_glyph(font, codepoint);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-utf8-scalar-1"></a>

## `epok::utf8_scalar`

**Purpose.** One UTF-8 scalar from `text`, advancing `n` past it.

**Details.** A malformed or truncated sequence yields '?' and consumes one byte, so a bad string can neither loop nor read past the terminator.

**Exact declaration**

```cpp
inline uint32_t utf8_scalar(const char* text,size_t& n)
```

- **Declared at:** [line 24](../../../runtime/font_types.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |
| `n` | `size_t &` | Input/output; inspect the function contract | Value supplied for `n`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** A malformed or truncated sequence yields '?' and consumes one byte, so a bad string can neither loop nor read past the terminator.

**Usage pattern**

```cpp
#include "font_types.hpp"

// Assume these named values have been initialized with valid data:
// const char * text
// size_t & n

auto result = epok::utf8_scalar(text, n);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
