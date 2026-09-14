# Epok API: Instrument Reverb

> **Header:** `"instrument_reverb.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/instrument_reverb.hpp)

This module covers the instrument reverb module. It documents 18 public callables declared directly in this header.

## Declared types

`epok::instrument::reverb::Error`, `epok::instrument::reverb::Preset`, `epok::instrument::reverb::PsxHardware`, `epok::instrument::reverb::Resource`

## Callable index

- [`epok::instrument::reverb::PsxHardware::read16`](#epok-instrument-reverb-psxhardware-read16-1) — Reads 16 as part of the instrument reverb module.
- [`epok::instrument::reverb::PsxHardware::write16`](#epok-instrument-reverb-psxhardware-write16-1) — Writes 16 as part of the instrument reverb module.
- [`epok::instrument::reverb::PsxHardware::zero_spu`](#epok-instrument-reverb-psxhardware-zero-spu-1) — Performs `zero spu` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::acquire`](#epok-instrument-reverb-resource-acquire-1) — Performs `acquire` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::active`](#epok-instrument-reverb-resource-active-1) — Performs `active` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::clear_voice`](#epok-instrument-reverb-resource-clear-voice-1) — Clears voice as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::generation`](#epok-instrument-reverb-resource-generation-1) — Performs `generation` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::owner`](#epok-instrument-reverb-resource-owner-1) — Performs `owner` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::prepare`](#epok-instrument-reverb-resource-prepare-1) — Performs `prepare` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::prepared`](#epok-instrument-reverb-resource-prepared-1) — Performs `prepared` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::preset`](#epok-instrument-reverb-resource-preset-1) — Performs `preset` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::reap`](#epok-instrument-reverb-resource-reap-1) — Performs `reap` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::release`](#epok-instrument-reverb-resource-release-1) — Performs `release` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::reserved_begin`](#epok-instrument-reverb-resource-reserved-begin-1) — Performs `reserved begin` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::reserved_bytes`](#epok-instrument-reverb-resource-reserved-bytes-1) — Performs `reserved bytes` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::send`](#epok-instrument-reverb-resource-send-1) — Performs `send` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::send_mask`](#epok-instrument-reverb-resource-send-mask-1) — Performs `send mask` as part of the instrument reverb module.
- [`epok::instrument::reverb::Resource::teardown`](#epok-instrument-reverb-resource-teardown-1) — Performs `teardown` as part of the instrument reverb module.

<a id="epok-instrument-reverb-psxhardware-read16-1"></a>

## `epok::instrument::reverb::PsxHardware::read16`

**Purpose.** Reads 16 as part of the instrument reverb module.

**Exact declaration**

```cpp
static uint16_t read16(uintptr_t address)
```

- **Declared at:** [line 43](../../../runtime/instrument_reverb.hpp#L43)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uintptr_t` | Input | Value supplied for `address`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uintptr_t address

auto result = epok::instrument::reverb::PsxHardware::read16(address);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-psxhardware-write16-1"></a>

## `epok::instrument::reverb::PsxHardware::write16`

**Purpose.** Writes 16 as part of the instrument reverb module.

**Exact declaration**

```cpp
static void write16(uintptr_t address, uint16_t value)
```

- **Declared at:** [line 44](../../../runtime/instrument_reverb.hpp#L44)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uintptr_t` | Input | Value supplied for `address`. See the exact type and module contract. |
| `value` | `uint16_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uintptr_t address
// uint16_t value

epok::instrument::reverb::PsxHardware::write16(address, value);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-psxhardware-zero-spu-1"></a>

## `epok::instrument::reverb::PsxHardware::zero_spu`

**Purpose.** Performs `zero spu` as part of the instrument reverb module.

**Exact declaration**

```cpp
static bool zero_spu(uint32_t address, uint32_t bytes)
```

- **Declared at:** [line 46](../../../runtime/instrument_reverb.hpp#L46)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `address` | `uint32_t` | Input | Value supplied for `address`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t address
// uint32_t bytes

auto result = epok::instrument::reverb::PsxHardware::zero_spu(address, bytes);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-instrument-reverb-resource-acquire-1"></a>

## `epok::instrument::reverb::Resource::acquire`

**Purpose.** Performs `acquire` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error acquire(uint16_t owner, uint32_t generation)
```

- **Declared at:** [line 107](../../../runtime/instrument_reverb.hpp#L107)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `uint16_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `generation` | `uint32_t` | Input | Value supplied for `generation`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t owner
// uint32_t generation

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.acquire(owner, generation);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-active-1"></a>

## `epok::instrument::reverb::Resource::active`

**Purpose.** Performs `active` as part of the instrument reverb module.

**Exact declaration**

```cpp
bool active() const
```

- **Declared at:** [line 157](../../../runtime/instrument_reverb.hpp#L157)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-reverb-resource-clear-voice-1"></a>

## `epok::instrument::reverb::Resource::clear_voice`

**Purpose.** Clears voice as part of the instrument reverb module.

**Exact declaration**

```cpp
void clear_voice(uint8_t physical_voice)
```

- **Declared at:** [line 141](../../../runtime/instrument_reverb.hpp#L141)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `physical_voice` | `uint8_t` | Input | Value supplied for `physical_voice`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t physical_voice

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

object.clear_voice(physical_voice);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-generation-1"></a>

## `epok::instrument::reverb::Resource::generation`

**Purpose.** Performs `generation` as part of the instrument reverb module.

**Exact declaration**

```cpp
uint32_t generation() const
```

- **Declared at:** [line 160](../../../runtime/instrument_reverb.hpp#L160)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.generation();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-owner-1"></a>

## `epok::instrument::reverb::Resource::owner`

**Purpose.** Performs `owner` as part of the instrument reverb module.

**Exact declaration**

```cpp
uint16_t owner() const
```

- **Declared at:** [line 159](../../../runtime/instrument_reverb.hpp#L159)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.owner();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-prepare-1"></a>

## `epok::instrument::reverb::Resource::prepare`

**Purpose.** Performs `prepare` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error prepare(uint32_t sample_upload_end, Preset preset, uint16_t depth_q15)
```

- **Declared at:** [line 85](../../../runtime/instrument_reverb.hpp#L85)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sample_upload_end` | `uint32_t` | Input | Value supplied for `sample_upload_end`. See the exact type and module contract. |
| `preset` | `Preset` | Input | Value supplied for `preset`. See the exact type and module contract. |
| `depth_q15` | `uint16_t` | Input | Value supplied for `depth_q15`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t sample_upload_end
// Preset preset
// uint16_t depth_q15

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.prepare(sample_upload_end, preset, depth_q15);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-prepared-1"></a>

## `epok::instrument::reverb::Resource::prepared`

**Purpose.** Performs `prepared` as part of the instrument reverb module.

**Exact declaration**

```cpp
bool prepared() const
```

- **Declared at:** [line 156](../../../runtime/instrument_reverb.hpp#L156)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.prepared();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-reverb-resource-preset-1"></a>

## `epok::instrument::reverb::Resource::preset`

**Purpose.** Performs `preset` as part of the instrument reverb module.

**Exact declaration**

```cpp
Preset preset() const
```

- **Declared at:** [line 158](../../../runtime/instrument_reverb.hpp#L158)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Preset`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.preset();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-reap-1"></a>

## `epok::instrument::reverb::Resource::reap`

**Purpose.** Performs `reap` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error reap(uint16_t owner, uint32_t generation)
```

- **Declared at:** [line 128](../../../runtime/instrument_reverb.hpp#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `uint16_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `generation` | `uint32_t` | Input | Value supplied for `generation`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t owner
// uint32_t generation

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.reap(owner, generation);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-release-1"></a>

## `epok::instrument::reverb::Resource::release`

**Purpose.** Performs `release` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error release(uint16_t owner, uint32_t generation)
```

- **Declared at:** [line 123](../../../runtime/instrument_reverb.hpp#L123)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `uint16_t` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `generation` | `uint32_t` | Input | Value supplied for `generation`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t owner
// uint32_t generation

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.release(owner, generation);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-reserved-begin-1"></a>

## `epok::instrument::reverb::Resource::reserved_begin`

**Purpose.** Performs `reserved begin` as part of the instrument reverb module.

**Exact declaration**

```cpp
uint32_t reserved_begin() const
```

- **Declared at:** [line 162](../../../runtime/instrument_reverb.hpp#L162)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.reserved_begin();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-reserved-bytes-1"></a>

## `epok::instrument::reverb::Resource::reserved_bytes`

**Purpose.** Performs `reserved bytes` as part of the instrument reverb module.

**Exact declaration**

```cpp
uint32_t reserved_bytes() const
```

- **Declared at:** [line 163](../../../runtime/instrument_reverb.hpp#L163)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.reserved_bytes();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-reverb-resource-send-1"></a>

## `epok::instrument::reverb::Resource::send`

**Purpose.** Performs `send` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error send(uint8_t physical_voice, bool enabled)
```

- **Declared at:** [line 130](../../../runtime/instrument_reverb.hpp#L130)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `physical_voice` | `uint8_t` | Input | Value supplied for `physical_voice`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

// Assume these named values have been initialized with valid data:
// uint8_t physical_voice
// bool enabled

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.send(physical_voice, enabled);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-instrument-reverb-resource-send-mask-1"></a>

## `epok::instrument::reverb::Resource::send_mask`

**Purpose.** Performs `send mask` as part of the instrument reverb module.

**Exact declaration**

```cpp
uint32_t send_mask() const
```

- **Declared at:** [line 161](../../../runtime/instrument_reverb.hpp#L161)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.send_mask();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-instrument-reverb-resource-teardown-1"></a>

## `epok::instrument::reverb::Resource::teardown`

**Purpose.** Performs `teardown` as part of the instrument reverb module.

**Exact declaration**

```cpp
Error teardown()
```

- **Declared at:** [line 148](../../../runtime/instrument_reverb.hpp#L148)
- **Kind:** `cxx method`

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument reverb module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_reverb.hpp"

epok::instrument::reverb::Resource& object = /* obtain a valid instance */;

auto result = object.teardown();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument reverb module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
