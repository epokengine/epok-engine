# PsyQo API: Font

> **Header:** `"psyqo/font.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh)

This module covers GPU text rendering with the built-in or uploaded font atlas. It documents 18 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Font`, `psyqo::FontBase`

## Callable index

- [`psyqo::Font::~Font<Fragments>`](#psyqo-font-font-fragments-1) — Releases the resources owned by `psyqo::Font`.
- [`psyqo::FontBase::chainprint`](#psyqo-fontbase-chainprint-1) — Performs `chainprint` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::chainprint`](#psyqo-fontbase-chainprint-2) — These methods use the DMA chaining system to print text to the screen.
- [`psyqo::FontBase::chainprintf`](#psyqo-fontbase-chainprintf-1) — Performs `chainprintf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::chainvprintf`](#psyqo-fontbase-chainvprintf-1) — Performs `chainvprintf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::initialize`](#psyqo-fontbase-initialize-1) — Initializes the object for use.
- [`psyqo::FontBase::print`](#psyqo-fontbase-print-1) — Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::print`](#psyqo-fontbase-print-2) — Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::print`](#psyqo-fontbase-print-3) — These method immediately print text to the screen.
- [`psyqo::FontBase::print`](#psyqo-fontbase-print-4) — Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::printf`](#psyqo-fontbase-printf-1) — Performs `printf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::printf`](#psyqo-fontbase-printf-2) — Performs `printf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::unpackFont`](#psyqo-fontbase-unpackfont-1) — Unpacks and uploads a font to VRAM.
- [`psyqo::FontBase::uploadKromFont`](#psyqo-fontbase-uploadkromfont-1) — Uploads the Kernel rom font to VRAM, and initializes the object.
- [`psyqo::FontBase::uploadSystemFont`](#psyqo-fontbase-uploadsystemfont-1) — Uploads the system font to VRAM, and initializes the object.
- [`psyqo::FontBase::vprintf`](#psyqo-fontbase-vprintf-1) — Performs `vprintf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::vprintf`](#psyqo-fontbase-vprintf-2) — Performs `vprintf` as part of GPU text rendering with the built-in or uploaded font atlas.
- [`psyqo::FontBase::~FontBase`](#psyqo-fontbase-fontbase-1) — Releases the resources owned by `psyqo::FontBase`.

<a id="psyqo-font-font-fragments-1"></a>

## `psyqo::Font::~Font<Fragments>`

**Purpose.** Releases the resources owned by `psyqo::Font`.

**Exact declaration**

```cpp
virtual ~Font()
```

- **Declared at:** [line 180](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L180)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// `psyqo::Font` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-fontbase-chainprint-1"></a>

## `psyqo::FontBase::chainprint`

**Purpose.** Performs `chainprint` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void chainprint(GPU& gpu, const char* text, Vertex pos, Color color)
```

- **Declared at:** [line 144](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L144)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// const char * text
// Vertex pos
// Color color

psyqo::FontBase& object = /* obtain a valid instance */;

object.chainprint(gpu, text, pos, color);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-chainprint-2"></a>

## `psyqo::FontBase::chainprint`

**Purpose.** These methods use the DMA chaining system to print text to the screen.

**Details.** These methods use the DMA chaining system to print text to the screen. They are meant to be used when constructing a frame using DMA chaining. When not using DMA chaining, you should use the `print` method family instead. See the `GPU` class for more details on DMA chaining.

**Exact declaration**

```cpp
void chainprint(GPU& gpu, eastl::string_view text, Vertex pos, Color color)
```

- **Declared at:** [line 143](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L143)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `eastl::string_view` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** These methods use the DMA chaining system to print text to the screen. They are meant to be used when constructing a frame using DMA chaining. When not using DMA chaining, you should use the `print` method family instead. See the `GPU` class for more details on DMA chaining.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// eastl::string_view text
// Vertex pos
// Color color

psyqo::FontBase& object = /* obtain a valid instance */;

object.chainprint(gpu, text, pos, color);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-chainprintf-1"></a>

## `psyqo::FontBase::chainprintf`

**Purpose.** Performs `chainprintf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void chainprintf(GPU& gpu, Vertex pos, Color color, const char* format, ...)
```

- **Declared at:** [line 145](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L145)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// const char * format

psyqo::FontBase& object = /* obtain a valid instance */;

object.chainprintf(gpu, pos, color, format);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-chainvprintf-1"></a>

## `psyqo::FontBase::chainvprintf`

**Purpose.** Performs `chainvprintf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void chainvprintf(GPU& gpu, Vertex pos, Color color, const char* format, va_list ap)
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L151)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |
| `ap` | `va_list` | Input | Value supplied for `ap`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// const char * format
// va_list ap

psyqo::FontBase& object = /* obtain a valid instance */;

object.chainvprintf(gpu, pos, color, format, ap);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-initialize-1"></a>

## `psyqo::FontBase::initialize`

**Purpose.** Initializes the object for use.

**Details.** When using your own font, you should call this method to initialize the Font object. This includes when using the `unpackFont` method. This method should not be called if you are using the `uploadSystemFont` method.

**Exact declaration**

```cpp
void initialize(GPU& gpu, Vertex location, Vertex glyphSize)
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `location` | `Vertex` | Input | The location of the font in VRAM. |
| `glyphSize` | `Vertex` | Input | The size of each glyph in the font. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** When using your own font, you should call this method to initialize the Font object. This includes when using the `unpackFont` method. This method should not be called if you are using the `uploadSystemFont` method.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex location
// Vertex glyphSize

psyqo::FontBase& object = /* obtain a valid instance */;

object.initialize(gpu, location, glyphSize);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-print-1"></a>

## `psyqo::FontBase::print`

**Purpose.** Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void print(GPU& gpu, const char* text, Vertex pos, Color color)
```

- **Declared at:** [line 116](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// const char * text
// Vertex pos
// Color color

psyqo::FontBase& object = /* obtain a valid instance */;

object.print(gpu, text, pos, color);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-print-2"></a>

## `psyqo::FontBase::print`

**Purpose.** Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void print(GPU& gpu, const char* text, Vertex pos, Color color, eastl::function<void()>&& callback, DMA::DmaCallback dmaCallback)
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L117)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | Value supplied for `dmaCallback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// const char * text
// Vertex pos
// Color color
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::FontBase& object = /* obtain a valid instance */;

object.print(gpu, text, pos, color, callback, dmaCallback);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-print-3"></a>

## `psyqo::FontBase::print`

**Purpose.** These method immediately print text to the screen.

**Details.** These methods immediately print text to the screen. They are meant to be used when not using DMA chaining. When using DMA chaining, you should use the `chainprint` method family instead. When a callback is provided, it will be called when the text has been printed, while the method will return immediately. See the `GPU` class for more details on DMA callbacks.

**Exact declaration**

```cpp
void print(GPU& gpu, eastl::string_view text, Vertex pos, Color color)
```

- **Declared at:** [line 113](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L113)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `eastl::string_view` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** These methods immediately print text to the screen. They are meant to be used when not using DMA chaining. When using DMA chaining, you should use the `chainprint` method family instead. When a callback is provided, it will be called when the text has been printed, while the method will return immediately. See the `GPU` class for more details on DMA callbacks.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// eastl::string_view text
// Vertex pos
// Color color

psyqo::FontBase& object = /* obtain a valid instance */;

object.print(gpu, text, pos, color);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-print-4"></a>

## `psyqo::FontBase::print`

**Purpose.** Performs `print` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void print(GPU& gpu, eastl::string_view text, Vertex pos, Color color, eastl::function<void()>&& callback, DMA::DmaCallback dmaCallback)
```

- **Declared at:** [line 114](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L114)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `text` | `eastl::string_view` | Input | Value supplied for `text`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | Value supplied for `dmaCallback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// eastl::string_view text
// Vertex pos
// Color color
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::FontBase& object = /* obtain a valid instance */;

object.print(gpu, text, pos, color, callback, dmaCallback);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-printf-1"></a>

## `psyqo::FontBase::printf`

**Purpose.** Performs `printf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void printf(GPU& gpu, Vertex pos, Color color, const char* format, ...)
```

- **Declared at:** [line 119](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L119)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// const char * format

psyqo::FontBase& object = /* obtain a valid instance */;

object.printf(gpu, pos, color, format);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-printf-2"></a>

## `psyqo::FontBase::printf`

**Purpose.** Performs `printf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void printf(GPU& gpu, Vertex pos, Color color, eastl::function<void()>&& callback, DMA::DmaCallback dmaCallback, const char* format, ...)
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L125)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | Value supplied for `dmaCallback`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback
// const char * format

psyqo::FontBase& object = /* obtain a valid instance */;

object.printf(gpu, pos, color, callback, dmaCallback, format);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-unpackfont-1"></a>

## `psyqo::FontBase::unpackFont`

**Purpose.** Unpacks and uploads a font to VRAM.

**Details.** This method unpacks and uploads a font to VRAM. The compressed font data is expected to be in the format generated by the `font-compress.lua` script. See this script for more details.

**Exact declaration**

```cpp
static void unpackFont(GPU& gpu, const uint8_t* data, Vertex location, Vertex textureSize)
```

- **Declared at:** [line 92](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L92)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `location` | `Vertex` | Input | Value supplied for `location`. See the exact type and module contract. |
| `textureSize` | `Vertex` | Input | Value supplied for `textureSize`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method unpacks and uploads a font to VRAM. The compressed font data is expected to be in the format generated by the `font-compress.lua` script. See this script for more details.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// const uint8_t * data
// Vertex location
// Vertex textureSize

psyqo::FontBase::unpackFont(gpu, data, location, textureSize);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-uploadkromfont-1"></a>

## `psyqo::FontBase::uploadKromFont`

**Purpose.** Uploads the Kernel rom font to VRAM, and initializes the object.

**Details.** This method uploads the built-in Kernel rom font to VRAM, and initializes the object. There is no need to call this method if you are using your own font. Also, when using this method, you should not call initialize() afterwards. The Kernel rom font is a 16x15 font created by Sony, and built into the PSX rom chip. Its appearance is variable, depending on the version of the PSX bios. It may not be available on all PSX models. The footprint for this font is 192 bytes of read-only data, and a 256x90x4bpp texture. This font isn't going to work if psyqo took over the kernel. See the `Kernel` namespace for more information.

**Exact declaration**

```cpp
void uploadKromFont(GPU& gpu, Vertex location = {{.x = 960, .y = 422}})
```

- **Declared at:** [line 84](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L84)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `location` | `Vertex` | Input | Value supplied for `location`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method uploads the built-in Kernel rom font to VRAM, and initializes the object. There is no need to call this method if you are using your own font. Also, when using this method, you should not call initialize() afterwards. The Kernel rom font is a 16x15 font created by Sony, and built into the PSX rom chip. Its appearance is variable, depending on the version of the PSX bios. It may not be available on all PSX models. The footprint for this font is 192 bytes of read-only data, and a 256x90x4bpp texture. This font isn't going to work if psyqo took over the kernel. See the `Kernel` namespace for more information.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex location

psyqo::FontBase& object = /* obtain a valid instance */;

object.uploadKromFont(gpu, location);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-uploadsystemfont-1"></a>

## `psyqo::FontBase::uploadSystemFont`

**Purpose.** Uploads the system font to VRAM, and initializes the object.

**Details.** This method uploads the built-in system font to VRAM, and initializes the object. There is no need to call this method if you are using your own font. Also, when using this method, you should not call initialize() afterwards. The footprint for this font is 877 bytes of read-only data, and a 256x48x4bpp texture. It is a 8x16 clean and simple ASCII font called mig68000, available for free with attribution, made by Zingot Games. See https://www.zingot.com/ and https://zingot.itch.io/fontpack

**Exact declaration**

```cpp
void uploadSystemFont(GPU& gpu, Vertex location = {{.x = 960, .y = 464}})
```

- **Declared at:** [line 71](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L71)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `location` | `Vertex` | Input | Value supplied for `location`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method uploads the built-in system font to VRAM, and initializes the object. There is no need to call this method if you are using your own font. Also, when using this method, you should not call initialize() afterwards. The footprint for this font is 877 bytes of read-only data, and a 256x48x4bpp texture. It is a 8x16 clean and simple ASCII font called mig68000, available for free with attribution, made by Zingot Games. See https://www.zingot.com/ and https://zingot.itch.io/fontpack

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex location

psyqo::FontBase& object = /* obtain a valid instance */;

object.uploadSystemFont(gpu, location);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-vprintf-1"></a>

## `psyqo::FontBase::vprintf`

**Purpose.** Performs `vprintf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void vprintf(GPU& gpu, Vertex pos, Color color, const char* format, va_list ap)
```

- **Declared at:** [line 132](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L132)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |
| `ap` | `va_list` | Input | Value supplied for `ap`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// const char * format
// va_list ap

psyqo::FontBase& object = /* obtain a valid instance */;

object.vprintf(gpu, pos, color, format, ap);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-vprintf-2"></a>

## `psyqo::FontBase::vprintf`

**Purpose.** Performs `vprintf` as part of GPU text rendering with the built-in or uploaded font atlas.

**Exact declaration**

```cpp
void vprintf(GPU& gpu, Vertex pos, Color color, eastl::function<void()>&& callback, DMA::DmaCallback dmaCallback, const char* format, va_list ap)
```

- **Declared at:** [line 133](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L133)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |
| `pos` | `Vertex` | Input | Value supplied for `pos`. See the exact type and module contract. |
| `color` | `Color` | Input | Value supplied for `color`. See the exact type and module contract. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | Value supplied for `dmaCallback`. See the exact type and module contract. |
| `format` | `const char *` | Input | Value supplied for `format`. See the exact type and module contract. |
| `ap` | `va_list` | Input | Value supplied for `ap`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// Assume these named values have been initialized with valid data:
// GPU & gpu
// Vertex pos
// Color color
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback
// const char * format
// va_list ap

psyqo::FontBase& object = /* obtain a valid instance */;

object.vprintf(gpu, pos, color, callback, dmaCallback, format, ap);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-fontbase-fontbase-1"></a>

## `psyqo::FontBase::~FontBase`

**Purpose.** Releases the resources owned by `psyqo::FontBase`.

**Exact declaration**

```cpp
virtual ~FontBase()
```

- **Declared at:** [line 59](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/font.hh#L59)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need GPU text rendering with the built-in or uploaded font atlas and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/font.hh"

// `psyqo::FontBase` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to GPU text rendering with the built-in or uploaded font atlas. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
