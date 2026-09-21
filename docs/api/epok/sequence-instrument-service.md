# Epok API: Sequence Instrument Service

> **Header:** `"sequence_instrument_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_instrument_service.hpp)

This module covers the sequence instrument service module. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::psx_audio::instrument_controls`](#epok-psx-audio-instrument-controls-1) — Performs `instrument controls` as part of the sequence instrument service module.
- [`epok::psx_audio::prepare_allocation`](#epok-psx-audio-prepare-allocation-1) — Performs `prepare allocation` as part of the sequence instrument service module.
- [`epok::sequence_flush_starts`](#epok-sequence-flush-starts-1) — Performs `sequence flush starts` as part of the sequence instrument service module.

<a id="epok-psx-audio-instrument-controls-1"></a>

## `epok::psx_audio::instrument_controls`

**Purpose.** Performs `instrument controls` as part of the sequence instrument service module.

**Exact declaration**

```cpp
inline instrument::synth::Controls instrument_controls(const sequence::Channel& c)
```

- **Declared at:** [line 19](../../../runtime/sequence_instrument_service.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `const sequence::Channel &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `instrument::synth::Controls`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence instrument service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_instrument_service.hpp"

// Assume these named values have been initialized with valid data:
// const sequence::Channel & c

auto result = epok::psx_audio::instrument_controls(c);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-psx-audio-prepare-allocation-1"></a>

## `epok::psx_audio::prepare_allocation`

**Purpose.** Performs `prepare allocation` as part of the sequence instrument service module.

**Exact declaration**

```cpp
inline void prepare_allocation()
```

- **Declared at:** [line 5](../../../runtime/sequence_instrument_service.hpp#L5)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence instrument service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_instrument_service.hpp"

epok::psx_audio::prepare_allocation();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-sequence-flush-starts-1"></a>

## `epok::sequence_flush_starts`

**Purpose.** Performs `sequence flush starts` as part of the sequence instrument service module.

**Exact declaration**

```cpp
inline uint32_t sequence_flush_starts()
```

- **Declared at:** [line 221](../../../runtime/sequence_instrument_service.hpp#L221)
- **Kind:** `function decl`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence instrument service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_instrument_service.hpp"

auto result = epok::sequence_flush_starts();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence instrument service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
