# Epok API: Sequence Data

> **Header:** `"sequence_data.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_data.hpp)

This module covers the sequence data module. It documents 12 public callables declared directly in this header.

## Declared types

`epok::psx_audio::Bank`, `epok::psx_audio::Sample`, `epok::psx_audio::Sequence`, `epok::psx_audio::Zone`

## Callable index

- [`epok::psx_audio::Bank::sample`](#epok-psx-audio-bank-sample-1) — Performs `sample` as part of the sequence data module.
- [`epok::psx_audio::Bank::sample_count`](#epok-psx-audio-bank-sample-count-1) — Performs `sample count` as part of the sequence data module.
- [`epok::psx_audio::Bank::valid`](#epok-psx-audio-bank-valid-1) — Performs `valid` as part of the sequence data module.
- [`epok::psx_audio::Bank::zone`](#epok-psx-audio-bank-zone-1) — Performs `zone` as part of the sequence data module.
- [`epok::psx_audio::Bank::zone_count`](#epok-psx-audio-bank-zone-count-1) — Performs `zone count` as part of the sequence data module.
- [`epok::psx_audio::read16`](#epok-psx-audio-read16-1) — Reads 16 as part of the sequence data module.
- [`epok::psx_audio::read32`](#epok-psx-audio-read32-1) — Reads 32 as part of the sequence data module.
- [`epok::psx_audio::Sequence::count`](#epok-psx-audio-sequence-count-1) — Performs `count` as part of the sequence data module.
- [`epok::psx_audio::Sequence::events`](#epok-psx-audio-sequence-events-1) — Performs `events` as part of the sequence data module.
- [`epok::psx_audio::Sequence::ppqn`](#epok-psx-audio-sequence-ppqn-1) — Performs `ppqn` as part of the sequence data module.
- [`epok::psx_audio::Sequence::valid`](#epok-psx-audio-sequence-valid-1) — Performs `valid` as part of the sequence data module.
- [`epok::psx_audio::Sequence::voices`](#epok-psx-audio-sequence-voices-1) — Performs `voices` as part of the sequence data module.

<a id="epok-psx-audio-bank-sample-1"></a>

## `epok::psx_audio::Bank::sample`

**Purpose.** Performs `sample` as part of the sequence data module.

**Exact declaration**

```cpp
const Sample& sample(uint16_t i) const
```

- **Declared at:** [line 30](../../../runtime/sequence_data.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `uint16_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Sample &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t i

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

- **Declared at:** [line 28](../../../runtime/sequence_data.hpp#L28)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 32](../../../runtime/sequence_data.hpp#L32)
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

- **Declared at:** [line 31](../../../runtime/sequence_data.hpp#L31)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `uint16_t` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Zone &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t i

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

- **Declared at:** [line 29](../../../runtime/sequence_data.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 6](../../../runtime/sequence_data.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const uint8_t *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * p

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

- **Declared at:** [line 7](../../../runtime/sequence_data.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const uint8_t *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * p

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

- **Declared at:** [line 65](../../../runtime/sequence_data.hpp#L65)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 66](../../../runtime/sequence_data.hpp#L66)
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

<a id="epok-psx-audio-sequence-ppqn-1"></a>

## `epok::psx_audio::Sequence::ppqn`

**Purpose.** Performs `ppqn` as part of the sequence data module.

**Exact declaration**

```cpp
uint16_t ppqn() const
```

- **Declared at:** [line 63](../../../runtime/sequence_data.hpp#L63)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 67](../../../runtime/sequence_data.hpp#L67)
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

- **Declared at:** [line 64](../../../runtime/sequence_data.hpp#L64)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence data module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_data.hpp"

epok::psx_audio::Sequence& object = /* obtain a valid instance */;

auto result = object.voices();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
