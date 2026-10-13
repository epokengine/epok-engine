# Epok API: Texture Upload

> **Header:** `"texture_upload.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/texture_upload.hpp)

This module covers native text data and HUD text components. It documents 1 public callable declared directly in this header.

## Declared types

`epok::TextureLoadError`, `epok::TextureLoadStats`

## Callable index

- [`epok::upload_texture_bank`](#epok-upload-texture-bank-1) — Acquire returns a pinned, checksum-verified page; upload MUST be blocking.

<a id="epok-upload-texture-bank-1"></a>

## `epok::upload_texture_bank`

**Purpose.** Acquire returns a pinned, checksum-verified page; upload MUST be blocking.

**Details.** The lease ends only after the pixel DMA completes. Nothing retains CPU pixel pointers after this function returns. Palettes remain resident for animation.

**Exact declaration**

```cpp
template<class Acquire,class Release,class Upload> bool upload_texture_bank(const Texture* bank,size_t count,uint32_t pages,uint32_t page_bytes, Acquire acquire,Release release,Upload upload)
```

- **Declared at:** [line 13](../../../runtime/texture_upload.hpp#L13)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const Texture *` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `pages` | `uint32_t` | Input | Value supplied for `pages`. See the exact type and module contract. |
| `page_bytes` | `uint32_t` | Input | Value supplied for `page_bytes`. See the exact type and module contract. |
| `acquire` | `Acquire` | Input | Value supplied for `acquire`. See the exact type and module contract. |
| `release` | `Release` | Input | Value supplied for `release`. See the exact type and module contract. |
| `upload` | `Upload` | Input | Value supplied for `upload`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The lease ends only after the pixel DMA completes. Nothing retains CPU pixel pointers after this function returns. Palettes remain resident for animation.

**Usage pattern**

```cpp
#include "texture_upload.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Acquire, Release, Upload

// Assume these named values have been initialized with valid data:
// const Texture * bank
// size_t count
// uint32_t pages
// uint32_t page_bytes
// Acquire acquire
// Release release
// Upload upload

auto result = epok::upload_texture_bank<Acquire, Release, Upload>(bank, count, pages, page_bytes, acquire, release, upload);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
