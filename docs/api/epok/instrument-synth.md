# Epok API: Instrument Synth

> **Header:** `"instrument_synth.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/instrument_synth.hpp)

This module covers the instrument synth module. It documents 37 public callables declared directly in this header.

## Declared types

`epok::instrument::synth::Controls`, `epok::instrument::synth::detail::AttenuationTable`, `epok::instrument::synth::detail::OctaveTable`, `epok::instrument::synth::Error`, `epok::instrument::synth::Output`, `epok::instrument::synth::State`, `epok::instrument::synth::State::HardwareParameters`

## Callable index

- [`epok::instrument::synth::Controls::Controls`](#epok-instrument-synth-controls-controls-1) — Constructs `epok::instrument::synth::Controls` for the instrument synth module.
- [`epok::instrument::synth::Controls::valid`](#epok-instrument-synth-controls-valid-1) — Performs `valid` as part of the instrument synth module.
- [`epok::instrument::synth::detail::advance_phase`](#epok-instrument-synth-detail-advance-phase-1) — Performs `advance phase` as part of the instrument synth module.
- [`epok::instrument::synth::detail::attenuation_for_gain_q16`](#epok-instrument-synth-detail-attenuation-for-gain-q16-1) — Performs `attenuation for gain q16` as part of the instrument synth module.
- [`epok::instrument::synth::detail::attenuation_gain_q15`](#epok-instrument-synth-detail-attenuation-gain-q15-1) — Performs `attenuation gain q15` as part of the instrument synth module.
- [`epok::instrument::synth::detail::AttenuationTable::AttenuationTable`](#epok-instrument-synth-detail-attenuationtable-attenuationtable-1) — Constructs `epok::instrument::synth::detail::AttenuationTable` for the instrument synth module.
- [`epok::instrument::synth::detail::clamp`](#epok-instrument-synth-detail-clamp-1) — Performs `clamp` as part of the instrument synth module.
- [`epok::instrument::synth::detail::concave`](#epok-instrument-synth-detail-concave-1) — Performs `concave` as part of the instrument synth module.
- [`epok::instrument::synth::detail::convex`](#epok-instrument-synth-detail-convex-1) — Performs `convex` as part of the instrument synth module.
- [`epok::instrument::synth::detail::copy_block`](#epok-instrument-synth-detail-copy-block-1) — Performs `copy block` as part of the instrument synth module.
- [`epok::instrument::synth::detail::copy_value`](#epok-instrument-synth-detail-copy-value-1) — Performs `copy value` as part of the instrument synth module.
- [`epok::instrument::synth::detail::fast_pow2_cents_q30`](#epok-instrument-synth-detail-fast-pow2-cents-q30-1) — Performs `fast pow2 cents q30` as part of the instrument synth module.
- [`epok::instrument::synth::detail::gain_at_whole_centibel`](#epok-instrument-synth-detail-gain-at-whole-centibel-1) — Performs `gain at whole centibel` as part of the instrument synth module.
- [`epok::instrument::synth::detail::lfo_rate_q48`](#epok-instrument-synth-detail-lfo-rate-q48-1) — Performs `lfo rate q48` as part of the instrument synth module.
- [`epok::instrument::synth::detail::local_bank_span`](#epok-instrument-synth-detail-local-bank-span-1) — Performs `local bank span` as part of the instrument synth module.
- [`epok::instrument::synth::detail::OctaveTable::OctaveTable`](#epok-instrument-synth-detail-octavetable-octavetable-1) — Constructs `epok::instrument::synth::detail::OctaveTable` for the instrument synth module.
- [`epok::instrument::synth::detail::pow2_cents_q30`](#epok-instrument-synth-detail-pow2-cents-q30-1) — Tenth-order integer exp2 on [0,1), followed by an integral power-of-two.
- [`epok::instrument::synth::detail::same_controls`](#epok-instrument-synth-detail-same-controls-1) — Performs `same controls` as part of the instrument synth module.
- [`epok::instrument::synth::detail::scale_q15`](#epok-instrument-synth-detail-scale-q15-1) — Performs `scale q15` as part of the instrument synth module.
- [`epok::instrument::synth::detail::source_changed`](#epok-instrument-synth-detail-source-changed-1) — Performs `source changed` as part of the instrument synth module.
- [`epok::instrument::synth::detail::source_q30`](#epok-instrument-synth-detail-source-q30-1) — Performs `source q30` as part of the instrument synth module.
- [`epok::instrument::synth::detail::time_microseconds`](#epok-instrument-synth-detail-time-microseconds-1) — Performs `time microseconds` as part of the instrument synth module.
- [`epok::instrument::synth::detail::triangle_q15`](#epok-instrument-synth-detail-triangle-q15-1) — Performs `triangle q15` as part of the instrument synth module.
- [`epok::instrument::synth::State::active`](#epok-instrument-synth-state-active-1) — Performs `active` as part of the instrument synth module.
- [`epok::instrument::synth::State::advance`](#epok-instrument-synth-state-advance-1) — Performs `advance` as part of the instrument synth module.
- [`epok::instrument::synth::State::copy_initial`](#epok-instrument-synth-state-copy-initial-1) — The immutable prepared source must outlive this voice, like its bank.
- [`epok::instrument::synth::State::error`](#epok-instrument-synth-state-error-1) — Performs `error` as part of the instrument synth module.
- [`epok::instrument::synth::State::hardware_output`](#epok-instrument-synth-state-hardware-output-1) — Performs `hardware output` as part of the instrument synth module.
- [`epok::instrument::synth::State::hardware_parameters`](#epok-instrument-synth-state-hardware-parameters-1) — Performs `hardware parameters` as part of the instrument synth module.
- [`epok::instrument::synth::State::output`](#epok-instrument-synth-state-output-1) — Performs `output` as part of the instrument synth module.
- [`epok::instrument::synth::State::release`](#epok-instrument-synth-state-release-1) — Performs `release` as part of the instrument synth module.
- [`epok::instrument::synth::State::reset`](#epok-instrument-synth-state-reset-1) — Resets reset as part of the instrument synth module.
- [`epok::instrument::synth::State::start`](#epok-instrument-synth-state-start-1) — `bank.data` must outlive this state.
- [`epok::instrument::synth::State::start_validated`](#epok-instrument-synth-state-start-validated-1) — PSX loader has validated the immutable bank and the kernel has validated controls/note data.
- [`epok::instrument::synth::State::start_voice`](#epok-instrument-synth-state-start-voice-1) — Host source-preview adapters may construct the same neutral wire structs without fabricating an EPSB container.
- [`epok::instrument::synth::State::State`](#epok-instrument-synth-state-state-1) — Constructs `epok::instrument::synth::State` for the instrument synth module.
- [`epok::instrument::synth::State::update_controls`](#epok-instrument-synth-state-update-controls-1) — Updates controls as part of the instrument synth module.

<a id="epok-instrument-synth-controls-controls-1"></a>

## `epok::instrument::synth::Controls::Controls`

**Purpose.** Constructs `epok::instrument::synth::Controls` for the instrument synth module.

**Exact declaration**

```cpp
Controls()
```

- **Declared at:** [line 22](../../../runtime/instrument_synth.hpp#L22)
- **Kind:** `constructor`

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::Controls value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-controls-valid-1"></a>

## `epok::instrument::synth::Controls::valid`

**Purpose.** Performs `valid` as part of the instrument synth module.

**Exact declaration**

```cpp
bool valid() const
```

- **Declared at:** [line 30](../../../runtime/instrument_synth.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::Controls& object = /* obtain a valid instance */;

auto result = object.valid();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-synth-detail-advance-phase-1"></a>

## `epok::instrument::synth::detail::advance_phase`

**Purpose.** Performs `advance phase` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint64_t advance_phase(uint64_t phase_q48, uint64_t rate_q48, uint64_t microseconds)
```

- **Declared at:** [line 307](../../../runtime/instrument_synth.hpp#L307)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `phase_q48` | `uint64_t` | Input | Value supplied for `phase_q48`. See the exact type and module contract. |
| `rate_q48` | `uint64_t` | Input | Value supplied for `rate_q48`. See the exact type and module contract. |
| `microseconds` | `uint64_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t phase_q48
// uint64_t rate_q48
// uint64_t microseconds

auto result = epok::instrument::synth::detail::advance_phase(phase_q48, rate_q48, microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-attenuation-for-gain-q16-1"></a>

## `epok::instrument::synth::detail::attenuation_for_gain_q16`

**Purpose.** Performs `attenuation for gain q16` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint32_t attenuation_for_gain_q16(uint16_t gain)
```

- **Declared at:** [line 326](../../../runtime/instrument_synth.hpp#L326)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gain` | `uint16_t` | Input | Value supplied for `gain`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t gain

auto result = epok::instrument::synth::detail::attenuation_for_gain_q16(gain);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-attenuation-gain-q15-1"></a>

## `epok::instrument::synth::detail::attenuation_gain_q15`

**Purpose.** Performs `attenuation gain q15` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint16_t attenuation_gain_q15(int64_t centibels_q16)
```

- **Declared at:** [line 288](../../../runtime/instrument_synth.hpp#L288)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `centibels_q16` | `int64_t` | Input | Value supplied for `centibels_q16`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t centibels_q16

auto result = epok::instrument::synth::detail::attenuation_gain_q15(centibels_q16);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-attenuationtable-attenuationtable-1"></a>

## `epok::instrument::synth::detail::AttenuationTable::AttenuationTable`

**Purpose.** Constructs `epok::instrument::synth::detail::AttenuationTable` for the instrument synth module.

**Exact declaration**

```cpp
constexpr AttenuationTable()
```

- **Declared at:** [line 273](../../../runtime/instrument_synth.hpp#L273)
- **Kind:** `constructor`

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::detail::AttenuationTable value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-clamp-1"></a>

## `epok::instrument::synth::detail::clamp`

**Purpose.** Performs `clamp` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int64_t clamp(int64_t value, int64_t low, int64_t high)
```

- **Declared at:** [line 177](../../../runtime/instrument_synth.hpp#L177)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `low` | `int64_t` | Input | Value supplied for `low`. See the exact type and module contract. |
| `high` | `int64_t` | Input | Value supplied for `high`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value
// int64_t low
// int64_t high

auto result = epok::instrument::synth::detail::clamp(value, low, high);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-concave-1"></a>

## `epok::instrument::synth::detail::concave`

**Purpose.** Performs `concave` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int64_t concave(int64_t normalized)
```

- **Declared at:** [line 202](../../../runtime/instrument_synth.hpp#L202)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `normalized` | `int64_t` | Input | Value supplied for `normalized`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t normalized

auto result = epok::instrument::synth::detail::concave(normalized);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-convex-1"></a>

## `epok::instrument::synth::detail::convex`

**Purpose.** Performs `convex` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int64_t convex(int64_t normalized)
```

- **Declared at:** [line 211](../../../runtime/instrument_synth.hpp#L211)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `normalized` | `int64_t` | Input | Value supplied for `normalized`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t normalized

auto result = epok::instrument::synth::detail::convex(normalized);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-copy-block-1"></a>

## `epok::instrument::synth::detail::copy_block`

**Purpose.** Performs `copy block` as part of the instrument synth module.

**Exact declaration**

```cpp
template<size_t Bytes> inline void copy_block(void* destination,const void* source)
```

- **Declared at:** [line 147](../../../runtime/instrument_synth.hpp#L147)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `destination` | `void *` | Input/output; inspect the function contract | Value supplied for `destination`. See the exact type and module contract. |
| `source` | `const void *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Bytes

// Assume these named values have been initialized with valid data:
// void * destination
// const void * source

epok::instrument::synth::detail& object = /* obtain a valid instance */;

object.copy_block<Bytes>(destination, source);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-copy-value-1"></a>

## `epok::instrument::synth::detail::copy_value`

**Purpose.** Performs `copy value` as part of the instrument synth module.

**Exact declaration**

```cpp
template<class T> inline void copy_value(T& destination,const T& source)
```

- **Declared at:** [line 161](../../../runtime/instrument_synth.hpp#L161)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `destination` | `T &` | Input/output; inspect the function contract | Value supplied for `destination`. See the exact type and module contract. |
| `source` | `const T &` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// T & destination
// const T & source

epok::instrument::synth::detail& object = /* obtain a valid instance */;

object.copy_value<T>(destination, source);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-fast-pow2-cents-q30-1"></a>

## `epok::instrument::synth::detail::fast_pow2_cents_q30`

**Purpose.** Performs `fast pow2 cents q30` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint64_t fast_pow2_cents_q30(int64_t cents_q16)
```

- **Declared at:** [line 248](../../../runtime/instrument_synth.hpp#L248)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cents_q16` | `int64_t` | Input | Value supplied for `cents_q16`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t cents_q16

auto result = epok::instrument::synth::detail::fast_pow2_cents_q30(cents_q16);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-gain-at-whole-centibel-1"></a>

## `epok::instrument::synth::detail::gain_at_whole_centibel`

**Purpose.** Performs `gain at whole centibel` as part of the instrument synth module.

**Exact declaration**

```cpp
inline constexpr uint16_t gain_at_whole_centibel(uint32_t centibels)
```

- **Declared at:** [line 262](../../../runtime/instrument_synth.hpp#L262)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `centibels` | `uint32_t` | Input | Value supplied for `centibels`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t centibels

auto result = epok::instrument::synth::detail::gain_at_whole_centibel(centibels);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-lfo-rate-q48-1"></a>

## `epok::instrument::synth::detail::lfo_rate_q48`

**Purpose.** Performs `lfo rate q48` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint64_t lfo_rate_q48(int64_t frequency_cents_q16)
```

- **Declared at:** [line 298](../../../runtime/instrument_synth.hpp#L298)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frequency_cents_q16` | `int64_t` | Input | Value supplied for `frequency_cents_q16`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t frequency_cents_q16

auto result = epok::instrument::synth::detail::lfo_rate_q48(frequency_cents_q16);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-local-bank-span-1"></a>

## `epok::instrument::synth::detail::local_bank_span`

**Purpose.** Performs `local bank span` as part of the instrument synth module.

**Exact declaration**

```cpp
inline bool local_bank_span(const BankView& bank, uint16_t zone_index, const Zone*& zone, const Modulation*& modulations)
```

- **Declared at:** [line 341](../../../runtime/instrument_synth.hpp#L341)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `zone_index` | `uint16_t` | Input | Value supplied for `zone_index`. See the exact type and module contract. |
| `zone` | `const Zone *&` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `modulations` | `const Modulation *&` | Input | Value supplied for `modulations`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// uint16_t zone_index
// const Zone *& zone
// const Modulation *& modulations

auto result = epok::instrument::synth::detail::local_bank_span(bank, zone_index, zone, modulations);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-octavetable-octavetable-1"></a>

## `epok::instrument::synth::detail::OctaveTable::OctaveTable`

**Purpose.** Constructs `epok::instrument::synth::detail::OctaveTable` for the instrument synth module.

**Exact declaration**

```cpp
constexpr OctaveTable()
```

- **Declared at:** [line 242](../../../runtime/instrument_synth.hpp#L242)
- **Kind:** `constructor`

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::detail::OctaveTable value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-pow2-cents-q30-1"></a>

## `epok::instrument::synth::detail::pow2_cents_q30`

**Purpose.** Tenth-order integer exp2 on [0,1), followed by an integral power-of-two.

**Details.** Across the admitted SF2 ranges its error is below one 0.01-cent output unit.

**Exact declaration**

```cpp
inline constexpr uint64_t pow2_cents_q30(int64_t cents_q16)
```

- **Declared at:** [line 219](../../../runtime/instrument_synth.hpp#L219)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cents_q16` | `int64_t` | Input | Value supplied for `cents_q16`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** Across the admitted SF2 ranges its error is below one 0.01-cent output unit.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t cents_q16

auto result = epok::instrument::synth::detail::pow2_cents_q30(cents_q16);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-same-controls-1"></a>

## `epok::instrument::synth::detail::same_controls`

**Purpose.** Performs `same controls` as part of the instrument synth module.

**Exact declaration**

```cpp
inline bool same_controls(const Controls& left, const Controls& right)
```

- **Declared at:** [line 421](../../../runtime/instrument_synth.hpp#L421)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `left` | `const Controls &` | Input | Value supplied for `left`. See the exact type and module contract. |
| `right` | `const Controls &` | Input | Value supplied for `right`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const Controls & left
// const Controls & right

auto result = epok::instrument::synth::detail::same_controls(left, right);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-scale-q15-1"></a>

## `epok::instrument::synth::detail::scale_q15`

**Purpose.** Performs `scale q15` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int64_t scale_q15(int64_t value, int32_t factor)
```

- **Declared at:** [line 321](../../../runtime/instrument_synth.hpp#L321)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `factor` | `int32_t` | Input | Value supplied for `factor`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value
// int32_t factor

auto result = epok::instrument::synth::detail::scale_q15(value, factor);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-source-changed-1"></a>

## `epok::instrument::synth::detail::source_changed`

**Purpose.** Performs `source changed` as part of the instrument synth module.

**Exact declaration**

```cpp
inline bool source_changed(uint16_t source,const Controls& a,const Controls& b)
```

- **Declared at:** [line 432](../../../runtime/instrument_synth.hpp#L432)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `uint16_t` | Input | Value supplied for `source`. See the exact type and module contract. |
| `a` | `const Controls &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Controls &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t source
// const Controls & a
// const Controls & b

auto result = epok::instrument::synth::detail::source_changed(source, a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-source-q30-1"></a>

## `epok::instrument::synth::detail::source_q30`

**Purpose.** Performs `source q30` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int64_t source_q30(uint16_t bits, uint8_t key, uint8_t velocity, const Controls& controls, bool& valid)
```

- **Declared at:** [line 357](../../../runtime/instrument_synth.hpp#L357)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bits` | `uint16_t` | Input | Value supplied for `bits`. See the exact type and module contract. |
| `key` | `uint8_t` | Input | Value supplied for `key`. See the exact type and module contract. |
| `velocity` | `uint8_t` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `controls` | `const Controls &` | Input | Value supplied for `controls`. See the exact type and module contract. |
| `valid` | `bool &` | Input/output; inspect the function contract | Value supplied for `valid`. See the exact type and module contract. |

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t bits
// uint8_t key
// uint8_t velocity
// const Controls & controls
// bool & valid

auto result = epok::instrument::synth::detail::source_q30(bits, key, velocity, controls, valid);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-detail-time-microseconds-1"></a>

## `epok::instrument::synth::detail::time_microseconds`

**Purpose.** Performs `time microseconds` as part of the instrument synth module.

**Exact declaration**

```cpp
inline uint64_t time_microseconds(int64_t timecents_q16, int32_t maximum, bool zero_sentinel)
```

- **Declared at:** [line 280](../../../runtime/instrument_synth.hpp#L280)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `timecents_q16` | `int64_t` | Input | Value supplied for `timecents_q16`. See the exact type and module contract. |
| `maximum` | `int32_t` | Input | Value supplied for `maximum`. See the exact type and module contract. |
| `zero_sentinel` | `bool` | Input | Value supplied for `zero_sentinel`. See the exact type and module contract. |

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// int64_t timecents_q16
// int32_t maximum
// bool zero_sentinel

auto result = epok::instrument::synth::detail::time_microseconds(timecents_q16, maximum, zero_sentinel);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-detail-triangle-q15-1"></a>

## `epok::instrument::synth::detail::triangle_q15`

**Purpose.** Performs `triangle q15` as part of the instrument synth module.

**Exact declaration**

```cpp
inline int32_t triangle_q15(uint64_t phase_q48)
```

- **Declared at:** [line 313](../../../runtime/instrument_synth.hpp#L313)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `phase_q48` | `uint64_t` | Input | Value supplied for `phase_q48`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t phase_q48

auto result = epok::instrument::synth::detail::triangle_q15(phase_q48);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-active-1"></a>

## `epok::instrument::synth::State::active`

**Purpose.** Performs `active` as part of the instrument synth module.

**Exact declaration**

```cpp
bool active() const
```

- **Declared at:** [line 80](../../../runtime/instrument_synth.hpp#L80)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-synth-state-advance-1"></a>

## `epok::instrument::synth::State::advance`

**Purpose.** Performs `advance` as part of the instrument synth module.

**Exact declaration**

```cpp
const Output& advance(uint32_t microseconds)
```

- **Declared at:** [line 78](../../../runtime/instrument_synth.hpp#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `microseconds` | `uint32_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |

**Returns.** Returns `const Output &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t microseconds

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.advance(microseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-copy-initial-1"></a>

## `epok::instrument::synth::State::copy_initial`

**Purpose.** The immutable prepared source must outlive this voice, like its bank.

**Exact declaration**

```cpp
void copy_initial(const State& source)
```

- **Declared at:** [line 84](../../../runtime/instrument_synth.hpp#L84)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const State &` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const State & source

epok::instrument::synth::State& object = /* obtain a valid instance */;

object.copy_initial(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-state-error-1"></a>

## `epok::instrument::synth::State::error`

**Purpose.** Performs `error` as part of the instrument synth module.

**Exact declaration**

```cpp
Error error() const
```

- **Declared at:** [line 81](../../../runtime/instrument_synth.hpp#L81)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.error();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-hardware-output-1"></a>

## `epok::instrument::synth::State::hardware_output`

**Purpose.** Performs `hardware output` as part of the instrument synth module.

**Exact declaration**

```cpp
Output hardware_output()
```

- **Declared at:** [line 89](../../../runtime/instrument_synth.hpp#L89)
- **Kind:** `cxx method`

**Returns.** Returns `Output`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.hardware_output();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-hardware-parameters-1"></a>

## `epok::instrument::synth::State::hardware_parameters`

**Purpose.** Performs `hardware parameters` as part of the instrument synth module.

**Exact declaration**

```cpp
HardwareParameters hardware_parameters() const
```

- **Declared at:** [line 88](../../../runtime/instrument_synth.hpp#L88)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `HardwareParameters`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.hardware_parameters();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-output-1"></a>

## `epok::instrument::synth::State::output`

**Purpose.** Performs `output` as part of the instrument synth module.

**Exact declaration**

```cpp
const Output& output() const
```

- **Declared at:** [line 79](../../../runtime/instrument_synth.hpp#L79)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const Output &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.output();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-release-1"></a>

## `epok::instrument::synth::State::release`

**Purpose.** Performs `release` as part of the instrument synth module.

**Exact declaration**

```cpp
void release()
```

- **Declared at:** [line 76](../../../runtime/instrument_synth.hpp#L76)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

object.release();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-reset-1"></a>

## `epok::instrument::synth::State::reset`

**Purpose.** Resets reset as part of the instrument synth module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 82](../../../runtime/instrument_synth.hpp#L82)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-start-1"></a>

## `epok::instrument::synth::State::start`

**Purpose.** `bank.data` must outlive this state.

**Details.** The loader validates BankView once; start performs bounded local table/span checks and never scans sample bytes.

**Exact declaration**

```cpp
Error start(const BankView& bank, uint16_t zone_index, uint8_t key, uint8_t velocity, const Controls& controls)
```

- **Declared at:** [line 63](../../../runtime/instrument_synth.hpp#L63)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `zone_index` | `uint16_t` | Input | Value supplied for `zone_index`. See the exact type and module contract. |
| `key` | `uint8_t` | Input | Value supplied for `key`. See the exact type and module contract. |
| `velocity` | `uint8_t` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `controls` | `const Controls &` | Input | Value supplied for `controls`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** The loader validates BankView once; start performs bounded local table/span checks and never scans sample bytes.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// uint16_t zone_index
// uint8_t key
// uint8_t velocity
// const Controls & controls

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.start(bank, zone_index, key, velocity, controls);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-state-start-validated-1"></a>

## `epok::instrument::synth::State::start_validated`

**Purpose.** PSX loader has validated the immutable bank and the kernel has validated controls/note data.

**Details.** Caller must select an in-range, matching zone first. Source-only filter control signals may be omitted when the cooked voice proves they can never contribute to target pitch or gain.

**Exact declaration**

```cpp
Error start_validated(const BankView& bank, uint16_t zone_index, uint8_t key, uint8_t velocity, const Controls& controls)
```

- **Declared at:** [line 69](../../../runtime/instrument_synth.hpp#L69)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `zone_index` | `uint16_t` | Input | Value supplied for `zone_index`. See the exact type and module contract. |
| `key` | `uint8_t` | Input | Value supplied for `key`. See the exact type and module contract. |
| `velocity` | `uint8_t` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `controls` | `const Controls &` | Input | Value supplied for `controls`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** Caller must select an in-range, matching zone first. Source-only filter control signals may be omitted when the cooked voice proves they can never contribute to target pitch or gain.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// uint16_t zone_index
// uint8_t key
// uint8_t velocity
// const Controls & controls

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.start_validated(bank, zone_index, key, velocity, controls);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-state-start-voice-1"></a>

## `epok::instrument::synth::State::start_voice`

**Purpose.** Host source-preview adapters may construct the same neutral wire structs without fabricating an EPSB container.

**Details.** Both referenced spans must outlive State.

**Exact declaration**

```cpp
Error start_voice(const Zone& zone, const Modulation* modulations, uint16_t modulation_count, uint8_t key, uint8_t velocity, const Controls& controls)
```

- **Declared at:** [line 73](../../../runtime/instrument_synth.hpp#L73)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `zone` | `const Zone &` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `modulations` | `const Modulation *` | Input | Value supplied for `modulations`. See the exact type and module contract. |
| `modulation_count` | `uint16_t` | Input | Value supplied for `modulation_count`. See the exact type and module contract. |
| `key` | `uint8_t` | Input | Value supplied for `key`. See the exact type and module contract. |
| `velocity` | `uint8_t` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `controls` | `const Controls &` | Input | Value supplied for `controls`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** Both referenced spans must outlive State.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const Zone & zone
// const Modulation * modulations
// uint16_t modulation_count
// uint8_t key
// uint8_t velocity
// const Controls & controls

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.start_voice(zone, modulations, modulation_count, key, velocity, controls);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-synth-state-state-1"></a>

## `epok::instrument::synth::State::State`

**Purpose.** Constructs `epok::instrument::synth::State` for the instrument synth module.

**Exact declaration**

```cpp
State() = default
```

- **Declared at:** [line 60](../../../runtime/instrument_synth.hpp#L60)
- **Kind:** `constructor`

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

epok::instrument::synth::State value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-synth-state-update-controls-1"></a>

## `epok::instrument::synth::State::update_controls`

**Purpose.** Updates controls as part of the instrument synth module.

**Exact declaration**

```cpp
Error update_controls(const Controls& controls)
```

- **Declared at:** [line 77](../../../runtime/instrument_synth.hpp#L77)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `controls` | `const Controls &` | Input | Value supplied for `controls`. See the exact type and module contract. |

**Returns.** Returns `Error`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument synth module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_synth.hpp"

// Assume these named values have been initialized with valid data:
// const Controls & controls

epok::instrument::synth::State& object = /* obtain a valid instance */;

auto result = object.update_controls(controls);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument synth module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
