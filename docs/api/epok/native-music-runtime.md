# Epok API: Native Music Runtime

> **Header:** `"native_music_runtime.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/native_music_runtime.hpp)

This module covers XA music streaming and playback state. It documents 21 public callables declared directly in this header.

## Declared types

`epok::psx_audio::Instance`, `epok::psx_audio::Parameters`, `epok::psx_audio::Phase`, `epok::psx_audio::Physical`, `epok::SequenceStats`, `epok::SequenceTimingStats`

## Callable index

- [`epok::psx_audio::counter_ticks`](#epok-psx-audio-counter-ticks-1) — Performs `counter ticks` as part of XA music streaming and playback state.
- [`epok::psx_audio::first_voice`](#epok-psx-audio-first-voice-1) — Performs `first voice` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::cut`](#epok-psx-audio-instance-cut-1) — Performs `cut` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::finish_layer`](#epok-psx-audio-instance-finish-layer-1) — Performs `finish layer` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::native_advance`](#epok-psx-audio-instance-native-advance-1) — Performs `native advance` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::native_begin`](#epok-psx-audio-instance-native-begin-1) — Performs `native begin` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::native_parameters`](#epok-psx-audio-instance-native-parameters-1) — Performs `native parameters` as part of XA music streaming and playback state.
- [`epok::psx_audio::Instance::owns`](#epok-psx-audio-instance-owns-1) — Performs `owns` as part of XA music streaming and playback state.
- [`epok::psx_audio::Physical::reset_metadata`](#epok-psx-audio-physical-reset-metadata-1) — Resets metadata as part of XA music streaming and playback state.
- [`epok::psx_audio::retire`](#epok-psx-audio-retire-1) — Performs `retire` as part of XA music streaming and playback state.
- [`epok::psx_audio::stolen`](#epok-psx-audio-stolen-1) — Performs `stolen` as part of XA music streaming and playback state.
- [`epok::sequence_clock_fault`](#epok-sequence-clock-fault-1) — Performs `sequence clock fault` as part of XA music streaming and playback state.
- [`epok::sequence_is_playing`](#epok-sequence-is-playing-1) — Performs `sequence is playing` as part of XA music streaming and playback state.
- [`epok::sequence_parameters`](#epok-sequence-parameters-1) — Performs `sequence parameters` as part of XA music streaming and playback state.
- [`epok::sequence_play`](#epok-sequence-play-1) — Performs `sequence play` as part of XA music streaming and playback state.
- [`epok::sequence_prepare`](#epok-sequence-prepare-1) — Performs `sequence prepare` as part of XA music streaming and playback state.
- [`epok::sequence_retiring`](#epok-sequence-retiring-1) — Performs `sequence retiring` as part of XA music streaming and playback state.
- [`epok::sequence_service`](#epok-sequence-service-1) — Performs `sequence service` as part of XA music streaming and playback state.
- [`epok::sequence_stop`](#epok-sequence-stop-1) — Performs `sequence stop` as part of XA music streaming and playback state.
- [`epok::sequence_update_sources`](#epok-sequence-update-sources-1) — Performs `sequence update sources` as part of XA music streaming and playback state.
- [`epok::sequence_voice_stolen`](#epok-sequence-voice-stolen-1) — Performs `sequence voice stolen` as part of XA music streaming and playback state.

<a id="epok-psx-audio-counter-ticks-1"></a>

## `epok::psx_audio::counter_ticks`

**Purpose.** Performs `counter ticks` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline uint16_t counter_ticks()
```

- **Declared at:** [line 35](../../../runtime/native_music_runtime.hpp#L35)
- **Kind:** `function decl`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

auto result = epok::psx_audio::counter_ticks();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-first-voice-1"></a>

## `epok::psx_audio::first_voice`

**Purpose.** Performs `first voice` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline int first_voice(uint32_t mask)
```

- **Declared at:** [line 42](../../../runtime/native_music_runtime.hpp#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int mask

auto result = epok::psx_audio::first_voice(mask);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-cut-1"></a>

## `epok::psx_audio::Instance::cut`

**Purpose.** Performs `cut` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void cut(uint16_t note)
```

- **Declared at:** [line 59](../../../runtime/native_music_runtime.hpp#L59)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.cut(note);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-finish-layer-1"></a>

## `epok::psx_audio::Instance::finish_layer`

**Purpose.** Performs `finish layer` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void finish_layer(int p)
```

- **Declared at:** [line 65](../../../runtime/native_music_runtime.hpp#L65)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int p

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.finish_layer(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-native-advance-1"></a>

## `epok::psx_audio::Instance::native_advance`

**Purpose.** Performs `native advance` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void native_advance(uint32_t elapsed)
```

- **Declared at:** [line 56](../../../runtime/native_music_runtime.hpp#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `int` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int elapsed

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.native_advance(elapsed);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-native-begin-1"></a>

## `epok::psx_audio::Instance::native_begin`

**Purpose.** Performs `native begin` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void native_begin()
```

- **Declared at:** [line 55](../../../runtime/native_music_runtime.hpp#L55)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.native_begin();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-native-parameters-1"></a>

## `epok::psx_audio::Instance::native_parameters`

**Purpose.** Performs `native parameters` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void native_parameters(int voice)
```

- **Declared at:** [line 57](../../../runtime/native_music_runtime.hpp#L57)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int voice

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.native_parameters(voice);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-owns-1"></a>

## `epok::psx_audio::Instance::owns`

**Purpose.** Performs `owns` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool owns(int voice,uint16_t note)const
```

- **Declared at:** [line 58](../../../runtime/native_music_runtime.hpp#L58)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int voice
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.owns(voice, note);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-physical-reset-metadata-1"></a>

## `epok::psx_audio::Physical::reset_metadata`

**Purpose.** Resets metadata as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void reset_metadata()
```

- **Declared at:** [line 29](../../../runtime/native_music_runtime.hpp#L29)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

epok::psx_audio::Physical& object = /* obtain a valid instance */;

object.reset_metadata();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-retire-1"></a>

## `epok::psx_audio::retire`

**Purpose.** Performs `retire` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void retire(Instance& i)
```

- **Declared at:** [line 72](../../../runtime/native_music_runtime.hpp#L72)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `Instance &` | Input/output; inspect the function contract | Value supplied for `i`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Instance & i

epok::psx_audio::retire(i);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-stolen-1"></a>

## `epok::psx_audio::stolen`

**Purpose.** Performs `stolen` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void stolen(int p)
```

- **Declared at:** [line 77](../../../runtime/native_music_runtime.hpp#L77)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int p

epok::psx_audio::stolen(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-sequence-clock-fault-1"></a>

## `epok::sequence_clock_fault`

**Purpose.** Performs `sequence clock fault` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_clock_fault()
```

- **Declared at:** [line 143](../../../runtime/native_music_runtime.hpp#L143)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

epok::sequence_clock_fault();
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-is-playing-1"></a>

## `epok::sequence_is_playing`

**Purpose.** Performs `sequence is playing` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline bool sequence_is_playing(const AudioSource* s)
```

- **Declared at:** [line 81](../../../runtime/native_music_runtime.hpp#L81)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `const AudioSource *` | Input | Value supplied for `s`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * s

auto result = epok::sequence_is_playing(s);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-parameters-1"></a>

## `epok::sequence_parameters`

**Purpose.** Performs `sequence parameters` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline psx_audio::Parameters sequence_parameters(const AudioSource* s)
```

- **Declared at:** [line 84](../../../runtime/native_music_runtime.hpp#L84)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `const AudioSource *` | Input | Value supplied for `s`. See the exact type and module contract. |

**Returns.** Returns `psx_audio::Parameters`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * s

auto result = epok::sequence_parameters(s);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-play-1"></a>

## `epok::sequence_play`

**Purpose.** Performs `sequence play` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_play(AudioSource* s)
```

- **Declared at:** [line 90](../../../runtime/native_music_runtime.hpp#L90)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `s`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * s

epok::sequence_play(s);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-prepare-1"></a>

## `epok::sequence_prepare`

**Purpose.** Performs `sequence prepare` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline bool sequence_prepare()
```

- **Declared at:** [line 146](../../../runtime/native_music_runtime.hpp#L146)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

auto result = epok::sequence_prepare();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-retiring-1"></a>

## `epok::sequence_retiring`

**Purpose.** Performs `sequence retiring` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline bool sequence_retiring()
```

- **Declared at:** [line 83](../../../runtime/native_music_runtime.hpp#L83)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

auto result = epok::sequence_retiring();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-service-1"></a>

## `epok::sequence_service`

**Purpose.** Performs `sequence service` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_service(uint32_t elapsed_us)
```

- **Declared at:** [line 111](../../../runtime/native_music_runtime.hpp#L111)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed_us` | `int` | Input | Value supplied for `elapsed_us`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int elapsed_us

epok::sequence_service(elapsed_us);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-stop-1"></a>

## `epok::sequence_stop`

**Purpose.** Performs `sequence stop` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_stop(AudioSource* s)
```

- **Declared at:** [line 82](../../../runtime/native_music_runtime.hpp#L82)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `s`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * s

epok::sequence_stop(s);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-update-sources-1"></a>

## `epok::sequence_update_sources`

**Purpose.** Performs `sequence update sources` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_update_sources()
```

- **Declared at:** [line 101](../../../runtime/native_music_runtime.hpp#L101)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

epok::sequence_update_sources();
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-voice-stolen-1"></a>

## `epok::sequence_voice_stolen`

**Purpose.** Performs `sequence voice stolen` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void sequence_voice_stolen(int p)
```

- **Declared at:** [line 80](../../../runtime/native_music_runtime.hpp#L80)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "native_music_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int p

epok::sequence_voice_stolen(p);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
