# Epok API: Native Music Data

> **Header:** `"native_music_data.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/native_music_data.hpp)

This module covers XA music streaming and playback state. It documents 7 public callables declared directly in this header.

## Declared types

`epok::native_music::Command`, `epok::native_music::Op`, `epok::native_music::Tone`, `epok::native_music::View`

## Callable index

- [`epok::native_music::u16`](#epok-native-music-u16-1) — Performs `u16` as part of XA music streaming and playback state.
- [`epok::native_music::u32`](#epok-native-music-u32-1) — Performs `u32` as part of XA music streaming and playback state.
- [`epok::native_music::View::commands`](#epok-native-music-view-commands-1) — Performs `commands` as part of XA music streaming and playback state.
- [`epok::native_music::View::count`](#epok-native-music-view-count-1) — Performs `count` as part of XA music streaming and playback state.
- [`epok::native_music::View::patches`](#epok-native-music-view-patches-1) — Performs `patches` as part of XA music streaming and playback state.
- [`epok::native_music::View::tones`](#epok-native-music-view-tones-1) — Performs `tones` as part of XA music streaming and playback state.
- [`epok::native_music::View::valid`](#epok-native-music-view-valid-1) — Performs `valid` as part of XA music streaming and playback state.

<a id="epok-native-music-u16-1"></a>

## `epok::native_music::u16`

**Purpose.** Performs `u16` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline uint16_t u16(const uint8_t* p)
```

- **Declared at:** [line 11](../../../runtime/native_music_data.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const uint8_t *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * p

auto result = epok::native_music::u16(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-native-music-u32-1"></a>

## `epok::native_music::u32`

**Purpose.** Performs `u32` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline uint32_t u32(const uint8_t* p)
```

- **Declared at:** [line 12](../../../runtime/native_music_data.hpp#L12)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const uint8_t *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * p

auto result = epok::native_music::u32(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-native-music-view-commands-1"></a>

## `epok::native_music::View::commands`

**Purpose.** Performs `commands` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
const Command* commands()const
```

- **Declared at:** [line 17](../../../runtime/native_music_data.hpp#L17)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const Command *`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

epok::native_music::View& object = /* obtain a valid instance */;

auto result = object.commands();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-native-music-view-count-1"></a>

## `epok::native_music::View::count`

**Purpose.** Performs `count` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
uint32_t count()const
```

- **Declared at:** [line 15](../../../runtime/native_music_data.hpp#L15)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

epok::native_music::View& object = /* obtain a valid instance */;

auto result = object.count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-native-music-view-patches-1"></a>

## `epok::native_music::View::patches`

**Purpose.** Performs `patches` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
const Tone* patches()const
```

- **Declared at:** [line 18](../../../runtime/native_music_data.hpp#L18)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const Tone *`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

epok::native_music::View& object = /* obtain a valid instance */;

auto result = object.patches();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-native-music-view-tones-1"></a>

## `epok::native_music::View::tones`

**Purpose.** Performs `tones` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
uint16_t tones()const
```

- **Declared at:** [line 16](../../../runtime/native_music_data.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

epok::native_music::View& object = /* obtain a valid instance */;

auto result = object.tones();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-native-music-view-valid-1"></a>

## `epok::native_music::View::valid`

**Purpose.** Performs `valid` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool valid(uint16_t samples)const
```

- **Declared at:** [line 19](../../../runtime/native_music_data.hpp#L19)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `samples` | `uint16_t` | Input | Value supplied for `samples`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_data.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t samples

epok::native_music::View& object = /* obtain a valid instance */;

auto result = object.valid(samples);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
