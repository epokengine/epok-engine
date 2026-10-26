# Epok API: Sequence Clock

> **Header:** `"sequence_clock.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_clock.hpp)

This module covers the sequence clock module. It documents 6 public callables declared directly in this header.

## Declared types

`epok::SequenceClockDiagnostics`, `epok::SequenceStoragePause`

## Callable index

- [`epok::sequence_clock_irq`](#epok-sequence-clock-irq-1) — Performs `sequence clock irq` as part of the sequence clock module.
- [`epok::sequence_clock_start`](#epok-sequence-clock-start-1) — Performs `sequence clock start` as part of the sequence clock module.
- [`epok::sequence_elapsed_ticks`](#epok-sequence-elapsed-ticks-1) — Timer 2 wraps every 15.48 ms.
- [`epok::sequence_hsync`](#epok-sequence-hsync-1) — Performs `sequence hsync` as part of the sequence clock module.
- [`epok::SequenceStoragePause::SequenceStoragePause`](#epok-sequencestoragepause-sequencestoragepause-1) — Constructs `epok::SequenceStoragePause` for the sequence clock module.
- [`epok::SequenceStoragePause::~SequenceStoragePause`](#epok-sequencestoragepause-sequencestoragepause-2) — Releases the resources owned by `epok::SequenceStoragePause`.

<a id="epok-sequence-clock-irq-1"></a>

## `epok::sequence_clock_irq`

**Purpose.** Performs `sequence clock irq` as part of the sequence clock module.

**Exact declaration**

```cpp
inline void sequence_clock_irq()
```

- **Declared at:** [line 107](../../../runtime/sequence_clock.hpp#L107)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence clock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

epok::sequence_clock_irq();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence clock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-clock-start-1"></a>

## `epok::sequence_clock_start`

**Purpose.** Performs `sequence clock start` as part of the sequence clock module.

**Exact declaration**

```cpp
inline bool sequence_clock_start()
```

- **Declared at:** [line 151](../../../runtime/sequence_clock.hpp#L151)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence clock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

auto result = epok::sequence_clock_start();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-elapsed-ticks-1"></a>

## `epok::sequence_elapsed_ticks`

**Purpose.** Timer 2 wraps every 15.48 ms.

**Details.** For a short delayed IRQ, the independent HSync counter identifies the wrap count without estimating musical time. 267..273 CPU/8 ticks per line covers NTSC/PAL clocks and either output mode; two extra lines cover the non-atomic counter reads. Even at the 100 ms bound the admissible interval is narrower than one Timer 2 wrap, so it is unique.

**Exact declaration**

```cpp
inline bool sequence_elapsed_ticks(uint16_t ticks,uint16_t lines,uint32_t& elapsed)
```

- **Declared at:** [line 99](../../../runtime/sequence_clock.hpp#L99)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ticks` | `uint16_t` | Input | Value supplied for `ticks`. See the exact type and module contract. |
| `lines` | `uint16_t` | Input | Value supplied for `lines`. See the exact type and module contract. |
| `elapsed` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** For a short delayed IRQ, the independent HSync counter identifies the wrap count without estimating musical time. 267..273 CPU/8 ticks per line covers NTSC/PAL clocks and either output mode; two extra lines cover the non-atomic counter reads. Even at the 100 ms bound the admissible interval is narrower than one Timer 2 wrap, so it is unique.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t ticks
// uint16_t lines
// uint32_t & elapsed

auto result = epok::sequence_elapsed_ticks(ticks, lines, elapsed);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-hsync-1"></a>

## `epok::sequence_hsync`

**Purpose.** Performs `sequence hsync` as part of the sequence clock module.

**Exact declaration**

```cpp
inline uint16_t sequence_hsync()
```

- **Declared at:** [line 65](../../../runtime/sequence_clock.hpp#L65)
- **Kind:** `function decl`

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sequence clock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

auto result = epok::sequence_hsync();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence clock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequencestoragepause-sequencestoragepause-1"></a>

## `epok::SequenceStoragePause::SequenceStoragePause`

**Purpose.** Constructs `epok::SequenceStoragePause` for the sequence clock module.

**Exact declaration**

```cpp
SequenceStoragePause()
```

- **Declared at:** [line 80](../../../runtime/sequence_clock.hpp#L80)
- **Kind:** `constructor`

**Use it when.** You need the sequence clock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

epok::SequenceStoragePause value();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence clock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequencestoragepause-sequencestoragepause-2"></a>

## `epok::SequenceStoragePause::~SequenceStoragePause`

**Purpose.** Releases the resources owned by `epok::SequenceStoragePause`.

**Exact declaration**

```cpp
~SequenceStoragePause()
```

- **Declared at:** [line 81](../../../runtime/sequence_clock.hpp#L81)
- **Kind:** `destructor`

**Use it when.** You need the sequence clock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_clock.hpp"

// `epok::SequenceStoragePause` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence clock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
