# Epok API: Instrument Bank

> **Header:** `"instrument_bank.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/instrument_bank.hpp)

This module covers the instrument bank module. It documents 18 public callables declared directly in this header.

## Declared types

`epok::instrument::BankView`, `epok::instrument::Destination`, `epok::instrument::Envelope`, `epok::instrument::Lfo`, `epok::instrument::Modulation`, `epok::instrument::Sample`, `epok::instrument::Zone`

## Callable index

- [`epok::instrument::BankView::modulation`](#epok-instrument-bankview-modulation-1) — Performs `modulation` as part of the instrument bank module.
- [`epok::instrument::BankView::modulation_count`](#epok-instrument-bankview-modulation-count-1) — Performs `modulation count` as part of the instrument bank module.
- [`epok::instrument::BankView::reverb_bytes`](#epok-instrument-bankview-reverb-bytes-1) — Performs `reverb bytes` as part of the instrument bank module.
- [`epok::instrument::BankView::reverb_depth`](#epok-instrument-bankview-reverb-depth-1) — Performs `reverb depth` as part of the instrument bank module.
- [`epok::instrument::BankView::reverb_preset`](#epok-instrument-bankview-reverb-preset-1) — Performs `reverb preset` as part of the instrument bank module.
- [`epok::instrument::BankView::sample`](#epok-instrument-bankview-sample-1) — Performs `sample` as part of the instrument bank module.
- [`epok::instrument::BankView::sample_bytes`](#epok-instrument-bankview-sample-bytes-1) — Performs `sample bytes` as part of the instrument bank module.
- [`epok::instrument::BankView::sample_count`](#epok-instrument-bankview-sample-count-1) — Performs `sample count` as part of the instrument bank module.
- [`epok::instrument::BankView::valid`](#epok-instrument-bankview-valid-1) — Analysis may inspect an over-budget payload; playback always enforces it.
- [`epok::instrument::BankView::zone`](#epok-instrument-bankview-zone-1) — Performs `zone` as part of the instrument bank module.
- [`epok::instrument::BankView::zone_count`](#epok-instrument-bankview-zone-count-1) — Performs `zone count` as part of the instrument bank module.
- [`epok::instrument::range`](#epok-instrument-range-1) — Performs `range` as part of the instrument bank module.
- [`epok::instrument::time`](#epok-instrument-time-1) — Performs `time` as part of the instrument bank module.
- [`epok::instrument::u16`](#epok-instrument-u16-1) — Performs `u16` as part of the instrument bank module.
- [`epok::instrument::u32`](#epok-instrument-u32-1) — Performs `u32` as part of the instrument bank module.
- [`epok::instrument::valid_envelope`](#epok-instrument-valid-envelope-1) — Performs `valid envelope` as part of the instrument bank module.
- [`epok::instrument::valid_lfo`](#epok-instrument-valid-lfo-1) — Performs `valid lfo` as part of the instrument bank module.
- [`epok::instrument::valid_source`](#epok-instrument-valid-source-1) — Performs `valid source` as part of the instrument bank module.

<a id="epok-instrument-bankview-modulation-1"></a>

## `epok::instrument::BankView::modulation`

**Purpose.** Performs `modulation` as part of the instrument bank module.

**Exact declaration**

```cpp
const Modulation& modulation(uint16_t i) const
```

- **Declared at:** [line 69](../../../runtime/instrument_bank.hpp#L69)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Modulation &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.modulation(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-modulation-count-1"></a>

## `epok::instrument::BankView::modulation_count`

**Purpose.** Performs `modulation count` as part of the instrument bank module.

**Exact declaration**

```cpp
uint32_t modulation_count() const
```

- **Declared at:** [line 62](../../../runtime/instrument_bank.hpp#L62)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.modulation_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-reverb-bytes-1"></a>

## `epok::instrument::BankView::reverb_bytes`

**Purpose.** Performs `reverb bytes` as part of the instrument bank module.

**Exact declaration**

```cpp
uint32_t reverb_bytes() const
```

- **Declared at:** [line 66](../../../runtime/instrument_bank.hpp#L66)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.reverb_bytes();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-reverb-depth-1"></a>

## `epok::instrument::BankView::reverb_depth`

**Purpose.** Performs `reverb depth` as part of the instrument bank module.

**Exact declaration**

```cpp
uint16_t reverb_depth() const
```

- **Declared at:** [line 65](../../../runtime/instrument_bank.hpp#L65)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.reverb_depth();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-reverb-preset-1"></a>

## `epok::instrument::BankView::reverb_preset`

**Purpose.** Performs `reverb preset` as part of the instrument bank module.

**Exact declaration**

```cpp
uint32_t reverb_preset() const
```

- **Declared at:** [line 64](../../../runtime/instrument_bank.hpp#L64)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.reverb_preset();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-sample-1"></a>

## `epok::instrument::BankView::sample`

**Purpose.** Performs `sample` as part of the instrument bank module.

**Exact declaration**

```cpp
const Sample& sample(uint16_t i) const
```

- **Declared at:** [line 67](../../../runtime/instrument_bank.hpp#L67)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Sample &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.sample(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-sample-bytes-1"></a>

## `epok::instrument::BankView::sample_bytes`

**Purpose.** Performs `sample bytes` as part of the instrument bank module.

**Exact declaration**

```cpp
uint32_t sample_bytes() const
```

- **Declared at:** [line 63](../../../runtime/instrument_bank.hpp#L63)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.sample_bytes();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-sample-count-1"></a>

## `epok::instrument::BankView::sample_count`

**Purpose.** Performs `sample count` as part of the instrument bank module.

**Exact declaration**

```cpp
uint16_t sample_count() const
```

- **Declared at:** [line 60](../../../runtime/instrument_bank.hpp#L60)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.sample_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-valid-1"></a>

## `epok::instrument::BankView::valid`

**Purpose.** Analysis may inspect an over-budget payload; playback always enforces it.

**Exact declaration**

```cpp
bool valid(bool enforce_spu_budget = true) const
```

- **Declared at:** [line 71](../../../runtime/instrument_bank.hpp#L71)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `enforce_spu_budget` | `bool` | Input | Value supplied for `enforce_spu_budget`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// bool enforce_spu_budget

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.valid(enforce_spu_budget);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-bankview-zone-1"></a>

## `epok::instrument::BankView::zone`

**Purpose.** Performs `zone` as part of the instrument bank module.

**Exact declaration**

```cpp
const Zone& zone(uint16_t i) const
```

- **Declared at:** [line 68](../../../runtime/instrument_bank.hpp#L68)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |

**Returns.** Returns `const Zone &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int i

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.zone(i);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-bankview-zone-count-1"></a>

## `epok::instrument::BankView::zone_count`

**Purpose.** Performs `zone count` as part of the instrument bank module.

**Exact declaration**

```cpp
uint16_t zone_count() const
```

- **Declared at:** [line 61](../../../runtime/instrument_bank.hpp#L61)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

epok::instrument::BankView& object = /* obtain a valid instance */;

auto result = object.zone_count();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-range-1"></a>

## `epok::instrument::range`

**Purpose.** Performs `range` as part of the instrument bank module.

**Exact declaration**

```cpp
inline bool range(int32_t value, int32_t lo, int32_t hi)
```

- **Declared at:** [line 38](../../../runtime/instrument_bank.hpp#L38)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |
| `lo` | `int` | Input | Value supplied for `lo`. See the exact type and module contract. |
| `hi` | `int` | Input | Value supplied for `hi`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int value
// int lo
// int hi

auto result = epok::instrument::range(value, lo, hi);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-time-1"></a>

## `epok::instrument::time`

**Purpose.** Performs `time` as part of the instrument bank module.

**Exact declaration**

```cpp
inline bool time(int32_t value, int32_t maximum, bool zero)
```

- **Declared at:** [line 39](../../../runtime/instrument_bank.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |
| `maximum` | `int` | Input | Value supplied for `maximum`. See the exact type and module contract. |
| `zero` | `bool` | Input | Value supplied for `zero`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int value
// int maximum
// bool zero

auto result = epok::instrument::time(value, maximum, zero);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-instrument-u16-1"></a>

## `epok::instrument::u16`

**Purpose.** Performs `u16` as part of the instrument bank module.

**Exact declaration**

```cpp
inline uint16_t u16(const uint8_t* p)
```

- **Declared at:** [line 8](../../../runtime/instrument_bank.hpp#L8)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// const int * p

auto result = epok::instrument::u16(p);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument bank module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-u32-1"></a>

## `epok::instrument::u32`

**Purpose.** Performs `u32` as part of the instrument bank module.

**Exact declaration**

```cpp
inline uint32_t u32(const uint8_t* p)
```

- **Declared at:** [line 9](../../../runtime/instrument_bank.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// const int * p

auto result = epok::instrument::u32(p);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument bank module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-valid-envelope-1"></a>

## `epok::instrument::valid_envelope`

**Purpose.** Performs `valid envelope` as part of the instrument bank module.

**Exact declaration**

```cpp
inline bool valid_envelope(const Envelope& e, bool volume)
```

- **Declared at:** [line 40](../../../runtime/instrument_bank.hpp#L40)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `e` | `const Envelope &` | Input | Value supplied for `e`. See the exact type and module contract. |
| `volume` | `bool` | Input | Value supplied for `volume`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// const Envelope & e
// bool volume

auto result = epok::instrument::valid_envelope(e, volume);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-valid-lfo-1"></a>

## `epok::instrument::valid_lfo`

**Purpose.** Performs `valid lfo` as part of the instrument bank module.

**Exact declaration**

```cpp
inline bool valid_lfo(const Lfo& lfo, bool vibrato)
```

- **Declared at:** [line 45](../../../runtime/instrument_bank.hpp#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lfo` | `const Lfo &` | Input | Value supplied for `lfo`. See the exact type and module contract. |
| `vibrato` | `bool` | Input | Value supplied for `vibrato`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// const Lfo & lfo
// bool vibrato

auto result = epok::instrument::valid_lfo(lfo, vibrato);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-valid-source-1"></a>

## `epok::instrument::valid_source`

**Purpose.** Performs `valid source` as part of the instrument bank module.

**Exact declaration**

```cpp
inline bool valid_source(uint16_t bits)
```

- **Declared at:** [line 49](../../../runtime/instrument_bank.hpp#L49)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bits` | `int` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument bank module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_bank.hpp"

// Assume these named values have been initialized with valid data:
// int bits

auto result = epok::instrument::valid_source(bits);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
