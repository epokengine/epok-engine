# PsyQo API: Hardware / Cdrom

> **Header:** `"psyqo/hardware/cdrom.hh"` · **Tier:** PsyQo low-level API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cdrom.hh)

This module covers the hardware/cdrom module. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Hardware::CDRom::Access`, `psyqo::Hardware::CDRom::CDL`, `psyqo::Hardware::CDRom::CommandFifo`

## Callable index

- [`psyqo::Hardware::CDRom::Access::access`](#psyqo-hardware-cdrom-access-access-1) — Performs `access` as part of the hardware/cdrom module.
- [`psyqo::Hardware::CDRom::CommandFifo::send`](#psyqo-hardware-cdrom-commandfifo-send-1) — Performs `send` as part of the hardware/cdrom module.
- [`psyqo::Hardware::CDRom::CommandFifo::send`](#psyqo-hardware-cdrom-commandfifo-send-2) — Performs `send` as part of the hardware/cdrom module.
- [`psyqo::Hardware::CDRom::CommandFifo::send`](#psyqo-hardware-cdrom-commandfifo-send-3) — Performs `send` as part of the hardware/cdrom module.

<a id="psyqo-hardware-cdrom-access-access-1"></a>

## `psyqo::Hardware::CDRom::Access::access`

**Purpose.** Performs `access` as part of the hardware/cdrom module.

**Exact declaration**

```cpp
static volatile uint8_t& access(int index = 0)
```

- **Declared at:** [line 80](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cdrom.hh#L80)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `int` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `volatile uint8_t &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/cdrom module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cdrom.hh"

// Assume these named values have been initialized with valid data:
// int index

auto result = psyqo::Hardware::CDRom::Access::access(index);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cdrom-commandfifo-send-1"></a>

## `psyqo::Hardware::CDRom::CommandFifo::send`

**Purpose.** Performs `send` as part of the hardware/cdrom module.

**Exact declaration**

```cpp
template <typename... Args> void send(CDL cmd, Args... args)
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cdrom.hh#L99)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cmd` | `CDL` | Input | Value supplied for `cmd`. See the exact type and module contract. |
| `args` | `Args...` | Input | Value supplied for `args`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cdrom module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cdrom.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Args

// Assume these named values have been initialized with valid data:
// CDL cmd
// Args... args

psyqo::Hardware::CDRom::CommandFifo& object = /* obtain a valid instance */;

object.send<Args>(cmd, args);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cdrom-commandfifo-send-2"></a>

## `psyqo::Hardware::CDRom::CommandFifo::send`

**Purpose.** Performs `send` as part of the hardware/cdrom module.

**Exact declaration**

```cpp
void send(CDL cmd)
```

- **Declared at:** [line 94](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cdrom.hh#L94)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cmd` | `CDL` | Input | Value supplied for `cmd`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cdrom module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cdrom.hh"

// Assume these named values have been initialized with valid data:
// CDL cmd

psyqo::Hardware::CDRom::CommandFifo& object = /* obtain a valid instance */;

object.send(cmd);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cdrom-commandfifo-send-3"></a>

## `psyqo::Hardware::CDRom::CommandFifo::send`

**Purpose.** Performs `send` as part of the hardware/cdrom module.

**Exact declaration**

```cpp
void send(CDL cmd, const CDRomCommandBuffer& commandBuffer)
```

- **Declared at:** [line 87](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cdrom.hh#L87)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `cmd` | `CDL` | Input | Value supplied for `cmd`. See the exact type and module contract. |
| `commandBuffer` | `const CDRomCommandBuffer &` | Input | Value supplied for `commandBuffer`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cdrom module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cdrom.hh"

// Assume these named values have been initialized with valid data:
// CDL cmd
// const CDRomCommandBuffer & commandBuffer

psyqo::Hardware::CDRom::CommandFifo& object = /* obtain a valid instance */;

object.send(cmd, commandBuffer);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
