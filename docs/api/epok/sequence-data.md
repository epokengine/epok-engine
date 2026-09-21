# Epok API: Sequence Data

> **Header:** `"sequence_data.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_data.hpp)

This module covers the sequence data module. It documents 16 public callables declared directly in this header.

## Declared types

`epok::psx_audio::Bank`, `epok::psx_audio::Sample`, `epok::psx_audio::Sequence`, `epok::psx_audio::Zone`

## Callable index

- [`epok::psx_audio::Bank::is_library`](#epok-psx-audio-bank-is-library-1) — Reports whether library as part of the sequence data module.
- [`epok::psx_audio::Bank::library`](#epok-psx-audio-bank-library-1) — Performs `library` as part of the sequence data module.
- [`epok::psx_audio::Bank::sample`](#epok-psx-audio-bank-sample-1) — Performs `sample` as part of the sequence data module.
- [`epok::psx_audio::Bank::sample_count`](#epok-psx-audio-bank-sample-count-1) — Performs `sample count` as part of the sequence data module.
- [`epok::psx_audio::Bank::valid`](#epok-psx-audio-bank-valid-1) — Performs `valid` as part of the sequence data module.
- [`epok::psx_audio::Bank::zone`](#epok-psx-audio-bank-zone-1) — Performs `zone` as part of the sequence data module.
- [`epok::psx_audio::Bank::zone_count`](#epok-psx-audio-bank-zone-count-1) — Performs `zone count` as part of the sequence data module.
- [`epok::psx_audio::read16`](#epok-psx-audio-read16-1) — Reads 16 as part of the sequence data module.
- [`epok::psx_audio::read32`](#epok-psx-audio-read32-1) — Reads 32 as part of the sequence data module.
- [`epok::psx_audio::Sequence::count`](#epok-psx-audio-sequence-count-1) — Performs `count` as part of the sequence data module.
- [`epok::psx_audio::Sequence::events`](#epok-psx-audio-sequence-events-1) — Performs `events` as part of the sequence data module.
- [`epok::psx_audio::Sequence::is_native`](#epok-psx-audio-sequence-is-native-1) — Reports whether native as part of the sequence data module.
- [`epok::psx_audio::Sequence::native`](#epok-psx-audio-sequence-native-1) — Performs `native` as part of the sequence data module.
- [`epok::psx_audio::Sequence::ppqn`](#epok-psx-audio-sequence-ppqn-1) — Performs `ppqn` as part of the sequence data module.
- [`epok::psx_audio::Sequence::valid`](#epok-psx-audio-sequence-valid-1) — Performs `valid` as part of the sequence data module.
- [`epok::psx_audio::Sequence::voices`](#epok-psx-audio-sequence-voices-1) — Performs `voices` as part of the sequence data module.

<a id="epok-psx-audio-bank-is-library-1"></a>

## `epok::psx_audio::Bank::is_library`

**Purpose.** Reports whether library as part of the sequence data module.

**Exact declaration**

```cpp
bool is_library() const
```

- **Declared at:** [line 31](../../../runtime/sequence_data.hpp#L31)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.is_library();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-library-1"></a>

## `epok::psx_audio::Bank::library`

**Purpose.** Performs `library` as part of the sequence data module.

**Exact declaration**

```cpp
instrument::BankView library() const
```

- **Declared at:** [line 32](../../../runtime/sequence_data.hpp#L32)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `instrument::BankView`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.library();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-sample-1"></a>

## `epok::psx_audio::Bank::sample`

**Purpose.** Performs `sample` as part of the sequence data module.

**Exact declaration**

```cpp
const Sample& sample(uint16_t i) const
```

- **Declared at:** [line 35](../../../runtime/sequence_data.hpp#L35)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Sample &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.sample(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-sample-count-1"></a>

## `epok::psx_audio::Bank::sample_count`

**Purpose.** Performs `sample count` as part of the sequence data module.

**Exact declaration**

```cpp
uint16_t sample_count() const
```

- **Declared at:** [line 33](../../../runtime/sequence_data.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.sample_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-valid-1"></a>

## `epok::psx_audio::Bank::valid`

**Purpose.** Performs `valid` as part of the sequence data module.

**Exact declaration**

```cpp
bool valid() const
```

- **Declared at:** [line 37](../../../runtime/sequence_data.hpp#L37)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-zone-1"></a>

## `epok::psx_audio::Bank::zone`

**Purpose.** Performs `zone` as part of the sequence data module.

**Exact declaration**

```cpp
const Zone& zone(uint16_t i) const
```

- **Declared at:** [line 36](../../../runtime/sequence_data.hpp#L36)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Zone &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.zone(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-bank-zone-count-1"></a>

## `epok::psx_audio::Bank::zone_count`

**Purpose.** Performs `zone count` as part of the sequence data module.

**Exact declaration**

```cpp
uint16_t zone_count() const
```

- **Declared at:** [line 34](../../../runtime/sequence_data.hpp#L34)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Bank& object = /* obtain a valid instance */;

auto result = object.zone_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-read16-1"></a>

## `epok::psx_audio::read16`

**Purpose.** Reads 16 as part of the sequence data module.

**Exact declaration**

```cpp
inline uint16_t read16(const uint8_t* p)
```

- **Declared at:** [line 9](../../../runtime/sequence_data.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// const int * p

auto result = epok::psx_audio::read16(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-read32-1"></a>

## `epok::psx_audio::read32`

**Purpose.** Reads 32 as part of the sequence data module.

**Exact declaration**

```cpp
inline uint32_t read32(const uint8_t* p)
```

- **Declared at:** [line 10](../../../runtime/sequence_data.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// const int * p

auto result = epok::psx_audio::read32(p);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-sequence-count-1"></a>

## `epok::psx_audio::Sequence::count`

**Purpose.** Performs `count` as part of the sequence data module.

**Exact declaration**

```cpp
uint32_t count() const
```

- **Declared at:** [line 74](../../../runtime/sequence_data.hpp#L74)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-events-1"></a>

## `epok::psx_audio::Sequence::events`

**Purpose.** Performs `events` as part of the sequence data module.

**Exact declaration**

```cpp
const sequence::Event* events() const
```

- **Declared at:** [line 75](../../../runtime/sequence_data.hpp#L75)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const sequence::Event *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.events();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-is-native-1"></a>

## `epok::psx_audio::Sequence::is_native`

**Purpose.** Reports whether native as part of the sequence data module.

**Exact declaration**

```cpp
bool is_native()const
```

- **Declared at:** [line 70](../../../runtime/sequence_data.hpp#L70)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.is_native();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-native-1"></a>

## `epok::psx_audio::Sequence::native`

**Purpose.** Performs `native` as part of the sequence data module.

**Exact declaration**

```cpp
native_music::View native()const
```

- **Declared at:** [line 71](../../../runtime/sequence_data.hpp#L71)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `native_music::View`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.native();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-ppqn-1"></a>

## `epok::psx_audio::Sequence::ppqn`

**Purpose.** Performs `ppqn` as part of the sequence data module.

**Exact declaration**

```cpp
uint16_t ppqn() const
```

- **Declared at:** [line 72](../../../runtime/sequence_data.hpp#L72)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.ppqn();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-valid-1"></a>

## `epok::psx_audio::Sequence::valid`

**Purpose.** Performs `valid` as part of the sequence data module.

**Exact declaration**

```cpp
bool valid() const
```

- **Declared at:** [line 76](../../../runtime/sequence_data.hpp#L76)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-psx-audio-sequence-voices-1"></a>

## `epok::psx_audio::Sequence::voices`

**Purpose.** Performs `voices` as part of the sequence data module.

**Exact declaration**

```cpp
uint16_t voices() const
```

- **Declared at:** [line 73](../../../runtime/sequence_data.hpp#L73)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.voices();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
