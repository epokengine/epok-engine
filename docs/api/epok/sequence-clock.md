# Epok API: Sequence Clock

> **Header:** `"sequence_clock.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_clock.hpp)

This module covers the sequence clock module. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::sequence_clock_irq`](#epok-sequence-clock-irq-1) — Performs `sequence clock irq` as part of the sequence clock module.
- [`epok::sequence_clock_start`](#epok-sequence-clock-start-1) — Performs `sequence clock start` as part of the sequence clock module.
- [`epok::sequence_hsync`](#epok-sequence-hsync-1) — Performs `sequence hsync` as part of the sequence clock module.

<a id="epok-sequence-clock-irq-1"></a>

## `epok::sequence_clock_irq`

**Purpose.** Performs `sequence clock irq` as part of the sequence clock module.

**Exact declaration**

```cpp
inline void sequence_clock_irq()
```

- **Declared at:** [line 25](../../../runtime/sequence_clock.hpp#L25)
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

- **Declared at:** [line 41](../../../runtime/sequence_clock.hpp#L41)
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

<a id="epok-sequence-hsync-1"></a>

## `epok::sequence_hsync`

**Purpose.** Performs `sequence hsync` as part of the sequence clock module.

**Exact declaration**

```cpp
inline uint16_t sequence_hsync()
```

- **Declared at:** [line 14](../../../runtime/sequence_clock.hpp#L14)
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
