# Epok API: Native Music Service

> **Header:** `"native_music_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/native_music_service.hpp)

This module covers XA music streaming and playback state. It documents 1 public callable declared directly in this header.

## Callable index

- [`epok::psx_audio::native_flush_keyoffs`](#epok-psx-audio-native-flush-keyoffs-1) — Performs `native flush keyoffs` as part of XA music streaming and playback state.

<a id="epok-psx-audio-native-flush-keyoffs-1"></a>

## `epok::psx_audio::native_flush_keyoffs`

**Purpose.** Performs `native flush keyoffs` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void native_flush_keyoffs()
```

- **Declared at:** [line 5](../../../runtime/native_music_service.hpp#L5)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_service.hpp"

epok::psx_audio::native_flush_keyoffs();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
