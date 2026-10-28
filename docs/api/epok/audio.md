# Epok API: Audio

> **Header:** `"audio.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/audio.hpp)

This module covers SPU sound playback and voice ownership. It documents 10 public callables declared directly in this header.

## Declared types

`epok::AudioClip`, `epok::AudioReuseStats`, `epok::AudioVoice`, `epok::psx_audio::Sequence`

## Callable index

- [`epok::audio_initialize`](#epok-audio-initialize-1) — Performs `audio initialize` as part of SPU sound playback and voice ownership.
- [`epok::audio_keyoff`](#epok-audio-keyoff-1) — Performs `audio keyoff` as part of SPU sound playback and voice ownership.
- [`epok::audio_parameters`](#epok-audio-parameters-1) — Performs `audio parameters` as part of SPU sound playback and voice ownership.
- [`epok::audio_record_keyoffs`](#epok-audio-record-keyoffs-1) — Performs `audio record keyoffs` as part of SPU sound playback and voice ownership.
- [`epok::audio_reuse_ready`](#epok-audio-reuse-ready-1) — Two complete HSync ticks exceed the SPU key-off/key-on recovery interval.
- [`epok::audio_scanline`](#epok-audio-scanline-1) — Performs `audio scanline` as part of SPU sound playback and voice ownership.
- [`epok::audio_tick`](#epok-audio-tick-1) — Performs `audio tick` as part of SPU sound playback and voice ownership.
- [`epok::music_is_playing`](#epok-music-is-playing-1) — Performs `music is playing` as part of SPU sound playback and voice ownership.
- [`epok::music_play`](#epok-music-play-1) — Performs `music play` as part of SPU sound playback and voice ownership.
- [`epok::music_stop`](#epok-music-stop-1) — Performs `music stop` as part of SPU sound playback and voice ownership.

<a id="epok-audio-initialize-1"></a>

## `epok::audio_initialize`

**Purpose.** Performs `audio initialize` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline bool audio_initialize(const AudioClip* clips,size_t count)
```

- **Declared at:** [line 49](../../../runtime/audio.hpp#L49)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `clips` | `const AudioClip *` | Input | Value supplied for `clips`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// const AudioClip * clips
// size_t count

auto result = epok::audio_initialize(clips, count);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-audio-keyoff-1"></a>

## `epok::audio_keyoff`

**Purpose.** Performs `audio keyoff` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline void audio_keyoff(int voice)
```

- **Declared at:** [line 83](../../../runtime/audio.hpp#L83)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// int voice

epok::audio_keyoff(voice);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audio-parameters-1"></a>

## `epok::audio_parameters`

**Purpose.** Performs `audio parameters` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline void audio_parameters(int i)
```

- **Declared at:** [line 102](../../../runtime/audio.hpp#L102)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::audio_parameters(i);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audio-record-keyoffs-1"></a>

## `epok::audio_record_keyoffs`

**Purpose.** Performs `audio record keyoffs` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline void audio_record_keyoffs(uint32_t mask)
```

- **Declared at:** [line 44](../../../runtime/audio.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `uint32_t` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t mask

epok::audio_record_keyoffs(mask);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audio-reuse-ready-1"></a>

## `epok::audio_reuse_ready`

**Purpose.** Two complete HSync ticks exceed the SPU key-off/key-on recovery interval.

**Details.** Prefer another free channel instead of spinning on a just-stopped channel.

**Exact declaration**

```cpp
inline bool audio_reuse_ready(int voice,uint16_t now)
```

- **Declared at:** [line 43](../../../runtime/audio.hpp#L43)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |
| `now` | `uint16_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Prefer another free channel instead of spinning on a just-stopped channel.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// int voice
// uint16_t now

auto result = epok::audio_reuse_ready(voice, now);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audio-scanline-1"></a>

## `epok::audio_scanline`

**Purpose.** Performs `audio scanline` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline uint16_t audio_scanline()
```

- **Declared at:** [line 36](../../../runtime/audio.hpp#L36)
- **Kind:** `function decl`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

auto result = epok::audio_scanline();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-audio-tick-1"></a>

## `epok::audio_tick`

**Purpose.** Performs `audio tick` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
inline void audio_tick()
```

- **Declared at:** [line 155](../../../runtime/audio.hpp#L155)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

epok::audio_tick();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-music-is-playing-1"></a>

## `epok::music_is_playing`

**Purpose.** Performs `music is playing` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
bool music_is_playing(const AudioSource*)
```

- **Declared at:** [line 14](../../../runtime/audio.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const AudioSource *` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * arg1

auto result = epok::music_is_playing(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-music-play-1"></a>

## `epok::music_play`

**Purpose.** Performs `music play` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
void music_play(AudioSource*)
```

- **Declared at:** [line 14](../../../runtime/audio.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * arg1

epok::music_play(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-music-stop-1"></a>

## `epok::music_stop`

**Purpose.** Performs `music stop` as part of SPU sound playback and voice ownership.

**Exact declaration**

```cpp
void music_stop(AudioSource*)
```

- **Declared at:** [line 14](../../../runtime/audio.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU sound playback and voice ownership and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "audio.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * arg1

epok::music_stop(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
