# Epok API: Sequence Service

> **Header:** `"sequence_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_service.hpp)

This module covers the sequence service module. It documents 28 public callables declared directly in this header.

## Declared types

`epok::psx_audio::Instance`, `epok::psx_audio::Parameters`, `epok::psx_audio::Phase`, `epok::psx_audio::Physical`, `epok::SequenceStats`, `epok::SequenceTimingStats`

## Callable index

- [`epok::psx_audio::counter_ticks`](#epok-psx-audio-counter-ticks-1) — Performs `counter ticks` as part of the sequence service module.
- [`epok::psx_audio::first_voice`](#epok-psx-audio-first-voice-1) — Performs `first voice` as part of the sequence service module.
- [`epok::psx_audio::Instance::advance_library`](#epok-psx-audio-instance-advance-library-1) — Performs `advance library` as part of the sequence service module.
- [`epok::psx_audio::Instance::cut`](#epok-psx-audio-instance-cut-1) — Performs `cut` as part of the sequence service module.
- [`epok::psx_audio::Instance::defer_note`](#epok-psx-audio-instance-defer-note-1) — Performs `defer note` as part of the sequence service module.
- [`epok::psx_audio::Instance::finish_layer`](#epok-psx-audio-instance-finish-layer-1) — Performs `finish layer` as part of the sequence service module.
- [`epok::psx_audio::Instance::owns`](#epok-psx-audio-instance-owns-1) — Performs `owns` as part of the sequence service module.
- [`epok::psx_audio::Instance::release`](#epok-psx-audio-instance-release-1) — Performs `release` as part of the sequence service module.
- [`epok::psx_audio::Instance::release_library`](#epok-psx-audio-instance-release-library-1) — Performs `release library` as part of the sequence service module.
- [`epok::psx_audio::Instance::start`](#epok-psx-audio-instance-start-1) — Starts start as part of the sequence service module.
- [`epok::psx_audio::Instance::start_library`](#epok-psx-audio-instance-start-library-1) — Starts library as part of the sequence service module.
- [`epok::psx_audio::Instance::update`](#epok-psx-audio-instance-update-1) — Updates update as part of the sequence service module.
- [`epok::psx_audio::Instance::update_library`](#epok-psx-audio-instance-update-library-1) — Updates library as part of the sequence service module.
- [`epok::psx_audio::Instance::voice`](#epok-psx-audio-instance-voice-1) — Performs `voice` as part of the sequence service module.
- [`epok::psx_audio::next_envelope`](#epok-psx-audio-next-envelope-1) — Performs `next envelope` as part of the sequence service module.
- [`epok::psx_audio::Physical::reset_metadata`](#epok-psx-audio-physical-reset-metadata-1) — Resets metadata as part of the sequence service module.
- [`epok::psx_audio::retire`](#epok-psx-audio-retire-1) — Performs `retire` as part of the sequence service module.
- [`epok::psx_audio::stolen`](#epok-psx-audio-stolen-1) — Performs `stolen` as part of the sequence service module.
- [`epok::sequence_clock_fault`](#epok-sequence-clock-fault-1) — Performs `sequence clock fault` as part of the sequence service module.
- [`epok::sequence_is_playing`](#epok-sequence-is-playing-1) — Performs `sequence is playing` as part of the sequence service module.
- [`epok::sequence_parameters`](#epok-sequence-parameters-1) — Performs `sequence parameters` as part of the sequence service module.
- [`epok::sequence_play`](#epok-sequence-play-1) — Performs `sequence play` as part of the sequence service module.
- [`epok::sequence_prepare`](#epok-sequence-prepare-1) — Performs `sequence prepare` as part of the sequence service module.
- [`epok::sequence_retiring`](#epok-sequence-retiring-1) — Performs `sequence retiring` as part of the sequence service module.
- [`epok::sequence_service`](#epok-sequence-service-1) — Performs `sequence service` as part of the sequence service module.
- [`epok::sequence_stop`](#epok-sequence-stop-1) — Performs `sequence stop` as part of the sequence service module.
- [`epok::sequence_update_sources`](#epok-sequence-update-sources-1) — Performs `sequence update sources` as part of the sequence service module.
- [`epok::sequence_voice_stolen`](#epok-sequence-voice-stolen-1) — Performs `sequence voice stolen` as part of the sequence service module.

<a id="epok-psx-audio-counter-ticks-1"></a>

## `epok::psx_audio::counter_ticks`

**Purpose.** Performs `counter ticks` as part of the sequence service module.

**Exact declaration**

```cpp
inline uint16_t counter_ticks()
```

- **Declared at:** [line 56](../../../runtime/sequence_service.hpp#L56)
- **Kind:** `function decl`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

auto result = epok::psx_audio::counter_ticks();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-first-voice-1"></a>

## `epok::psx_audio::first_voice`

**Purpose.** Performs `first voice` as part of the sequence service module.

**Exact declaration**

```cpp
inline int first_voice(uint32_t mask)
```

- **Declared at:** [line 64](../../../runtime/sequence_service.hpp#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `int` | Input | Value supplied for `mask`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int mask

auto result = epok::psx_audio::first_voice(mask);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-advance-library-1"></a>

## `epok::psx_audio::Instance::advance_library`

**Purpose.** Performs `advance library` as part of the sequence service module.

**Exact declaration**

```cpp
bool advance_library(int number)
```

- **Declared at:** [line 114](../../../runtime/sequence_service.hpp#L114)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `number` | `int` | Input | Value supplied for `number`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int number

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.advance_library(number);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-cut-1"></a>

## `epok::psx_audio::Instance::cut`

**Purpose.** Performs `cut` as part of the sequence service module.

**Exact declaration**

```cpp
void cut(uint16_t note)
```

- **Declared at:** [line 99](../../../runtime/sequence_service.hpp#L99)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.cut(note);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-defer-note-1"></a>

## `epok::psx_audio::Instance::defer_note`

**Purpose.** Performs `defer note` as part of the sequence service module.

**Exact declaration**

```cpp
bool defer_note(const sequence::Event&,uint32_t cursor)
```

- **Declared at:** [line 83](../../../runtime/sequence_service.hpp#L83)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const sequence::Event &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `cursor` | `int` | Input | Value supplied for `cursor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// const sequence::Event & arg1
// int cursor

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.defer_note(arg1, cursor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-instance-finish-layer-1"></a>

## `epok::psx_audio::Instance::finish_layer`

**Purpose.** Performs `finish layer` as part of the sequence service module.

**Exact declaration**

```cpp
void finish_layer(int i)
```

- **Declared at:** [line 107](../../../runtime/sequence_service.hpp#L107)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.finish_layer(i);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-owns-1"></a>

## `epok::psx_audio::Instance::owns`

**Purpose.** Performs `owns` as part of the sequence service module.

**Exact declaration**

```cpp
bool owns(int voice,uint16_t note)const
```

- **Declared at:** [line 92](../../../runtime/sequence_service.hpp#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int voice
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.owns(voice, note);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-release-1"></a>

## `epok::psx_audio::Instance::release`

**Purpose.** Performs `release` as part of the sequence service module.

**Exact declaration**

```cpp
void release(uint16_t note)
```

- **Declared at:** [line 117](../../../runtime/sequence_service.hpp#L117)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.release(note);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-release-library-1"></a>

## `epok::psx_audio::Instance::release_library`

**Purpose.** Performs `release library` as part of the sequence service module.

**Exact declaration**

```cpp
void release_library(uint16_t note)
```

- **Declared at:** [line 115](../../../runtime/sequence_service.hpp#L115)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.release_library(note);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-instance-start-1"></a>

## `epok::psx_audio::Instance::start`

**Purpose.** Starts start as part of the sequence service module.

**Exact declaration**

```cpp
bool start(uint16_t note,const sequence::Note& n,const sequence::Channel& channel)
```

- **Declared at:** [line 164](../../../runtime/sequence_service.hpp#L164)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |
| `n` | `const sequence::Note &` | Input | Value supplied for `n`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note
// const sequence::Note & n
// const sequence::Channel & channel

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.start(note, n, channel);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-instance-start-library-1"></a>

## `epok::psx_audio::Instance::start_library`

**Purpose.** Starts library as part of the sequence service module.

**Exact declaration**

```cpp
bool start_library(uint16_t note,const sequence::Note& n,const sequence::Channel& channel)
```

- **Declared at:** [line 165](../../../runtime/sequence_service.hpp#L165)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |
| `n` | `const sequence::Note &` | Input | Value supplied for `n`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note
// const sequence::Note & n
// const sequence::Channel & channel

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.start_library(note, n, channel);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-instance-update-1"></a>

## `epok::psx_audio::Instance::update`

**Purpose.** Updates update as part of the sequence service module.

**Exact declaration**

```cpp
void update(uint16_t note,const sequence::Channel& channel)
```

- **Declared at:** [line 125](../../../runtime/sequence_service.hpp#L125)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note
// const sequence::Channel & channel

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.update(note, channel);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-instance-update-library-1"></a>

## `epok::psx_audio::Instance::update_library`

**Purpose.** Updates library as part of the sequence service module.

**Exact declaration**

```cpp
void update_library(int physical_voice,const sequence::Channel& channel,bool controls_changed)
```

- **Declared at:** [line 116](../../../runtime/sequence_service.hpp#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `physical_voice` | `int` | Input | Value supplied for `physical_voice`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |
| `controls_changed` | `bool` | Input | Value supplied for `controls_changed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int physical_voice
// const sequence::Channel & channel
// bool controls_changed

epok::psx_audio::Instance& object = /* obtain a valid instance */;

object.update_library(physical_voice, channel, controls_changed);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-instance-voice-1"></a>

## `epok::psx_audio::Instance::voice`

**Purpose.** Performs `voice` as part of the sequence service module.

**Exact declaration**

```cpp
int voice(uint16_t note) const
```

- **Declared at:** [line 95](../../../runtime/sequence_service.hpp#L95)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int note

epok::psx_audio::Instance& object = /* obtain a valid instance */;

auto result = object.voice(note);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-next-envelope-1"></a>

## `epok::psx_audio::next_envelope`

**Purpose.** Performs `next envelope` as part of the sequence service module.

**Exact declaration**

```cpp
inline void next_envelope(Physical& v)
```

- **Declared at:** [line 182](../../../runtime/sequence_service.hpp#L182)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Physical &` | Input/output; inspect the function contract | Value supplied for `v`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// Physical & v

epok::psx_audio::next_envelope(v);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-physical-reset-metadata-1"></a>

## `epok::psx_audio::Physical::reset_metadata`

**Purpose.** Resets metadata as part of the sequence service module.

**Exact declaration**

```cpp
void reset_metadata()
```

- **Declared at:** [line 41](../../../runtime/sequence_service.hpp#L41)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

epok::psx_audio::Physical& object = /* obtain a valid instance */;

object.reset_metadata();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-retire-1"></a>

## `epok::psx_audio::retire`

**Purpose.** Performs `retire` as part of the sequence service module.

**Exact declaration**

```cpp
inline void retire(Instance& instance)
```

- **Declared at:** [line 169](../../../runtime/sequence_service.hpp#L169)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Instance &` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// Instance & instance

epok::psx_audio::retire(instance);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-stolen-1"></a>

## `epok::psx_audio::stolen`

**Purpose.** Performs `stolen` as part of the sequence service module.

**Exact declaration**

```cpp
inline void stolen(int i)
```

- **Declared at:** [line 175](../../../runtime/sequence_service.hpp#L175)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::psx_audio::stolen(i);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-sequence-clock-fault-1"></a>

## `epok::sequence_clock_fault`

**Purpose.** Performs `sequence clock fault` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_clock_fault()
```

- **Declared at:** [line 347](../../../runtime/sequence_service.hpp#L347)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

epok::sequence_clock_fault();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-is-playing-1"></a>

## `epok::sequence_is_playing`

**Purpose.** Performs `sequence is playing` as part of the sequence service module.

**Exact declaration**

```cpp
inline bool sequence_is_playing(const AudioSource* source)
```

- **Declared at:** [line 231](../../../runtime/sequence_service.hpp#L231)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const AudioSource *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * source

auto result = epok::sequence_is_playing(source);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-parameters-1"></a>

## `epok::sequence_parameters`

**Purpose.** Performs `sequence parameters` as part of the sequence service module.

**Exact declaration**

```cpp
inline psx_audio::Parameters sequence_parameters(const AudioSource* source)
```

- **Declared at:** [line 243](../../../runtime/sequence_service.hpp#L243)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const AudioSource *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `psx_audio::Parameters`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * source

auto result = epok::sequence_parameters(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-play-1"></a>

## `epok::sequence_play`

**Purpose.** Performs `sequence play` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_play(AudioSource* source)
```

- **Declared at:** [line 249](../../../runtime/sequence_service.hpp#L249)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * source

epok::sequence_play(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-prepare-1"></a>

## `epok::sequence_prepare`

**Purpose.** Performs `sequence prepare` as part of the sequence service module.

**Exact declaration**

```cpp
inline bool sequence_prepare()
```

- **Declared at:** [line 351](../../../runtime/sequence_service.hpp#L351)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

auto result = epok::sequence_prepare();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-retiring-1"></a>

## `epok::sequence_retiring`

**Purpose.** Performs `sequence retiring` as part of the sequence service module.

**Exact declaration**

```cpp
inline bool sequence_retiring()
```

- **Declared at:** [line 238](../../../runtime/sequence_service.hpp#L238)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

auto result = epok::sequence_retiring();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-service-1"></a>

## `epok::sequence_service`

**Purpose.** Performs `sequence service` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_service(uint32_t elapsed_us)
```

- **Declared at:** [line 283](../../../runtime/sequence_service.hpp#L283)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed_us` | `int` | Input | Value supplied for `elapsed_us`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int elapsed_us

epok::sequence_service(elapsed_us);
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-stop-1"></a>

## `epok::sequence_stop`

**Purpose.** Performs `sequence stop` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_stop(AudioSource* source)
```

- **Declared at:** [line 235](../../../runtime/sequence_service.hpp#L235)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `AudioSource *` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// AudioSource * source

epok::sequence_stop(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-update-sources-1"></a>

## `epok::sequence_update_sources`

**Purpose.** Performs `sequence update sources` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_update_sources()
```

- **Declared at:** [line 266](../../../runtime/sequence_service.hpp#L266)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

epok::sequence_update_sources();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-voice-stolen-1"></a>

## `epok::sequence_voice_stolen`

**Purpose.** Performs `sequence voice stolen` as part of the sequence service module.

**Exact declaration**

```cpp
inline void sequence_voice_stolen(int voice)
```

- **Declared at:** [line 230](../../../runtime/sequence_service.hpp#L230)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `voice` | `int` | Input | Value supplied for `voice`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// int voice

epok::sequence_voice_stolen(voice);
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
