# PsyQo API: Cdrom Commandbuffer

> **Header:** `"psyqo/cdrom-commandbuffer.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-commandbuffer.hh)

This module covers the cdrom commandbuffer module. It documents 1 public callable declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Hardware::CDRom::CDRomCommandBuffer`

## Callable index

- [`psyqo::Hardware::CDRom::CDRomCommandBuffer::set`](#psyqo-hardware-cdrom-cdromcommandbuffer-set-1) — Sets set as part of the cdrom commandbuffer module.

<a id="psyqo-hardware-cdrom-cdromcommandbuffer-set-1"></a>

## `psyqo::Hardware::CDRom::CDRomCommandBuffer::set`

**Purpose.** Sets set as part of the cdrom commandbuffer module.

**Exact declaration**

```cpp
template <CDRomArgumentType... T> void set(T... values)
```

- **Declared at:** [line 41](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-commandbuffer.hh#L41)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `values` | `T...` | Input | Value supplied for `values`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom commandbuffer module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-commandbuffer.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// T... values

psyqo::Hardware::CDRom::CDRomCommandBuffer& object = /* obtain a valid instance */;

object.set<T>(values);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
