# PsyQo API: Hardware / Cpu

> **Header:** `"psyqo/hardware/cpu.hh"` · **Tier:** PsyQo low-level API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh)

This module covers the hardware/cpu module. It documents 5 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Hardware::CPU::IRQ`, `psyqo::Hardware::CPU::IRQReg`

## Callable index

- [`psyqo::Hardware::CPU::flushWriteQueue`](#psyqo-hardware-cpu-flushwritequeue-1) — Performs `flush write queue` as part of the hardware/cpu module.
- [`psyqo::Hardware::CPU::IRQReg::clear`](#psyqo-hardware-cpu-irqreg-clear-1) — Clears clear as part of the hardware/cpu module.
- [`psyqo::Hardware::CPU::IRQReg::clear`](#psyqo-hardware-cpu-irqreg-clear-2) — Clears clear as part of the hardware/cpu module.
- [`psyqo::Hardware::CPU::IRQReg::isSet`](#psyqo-hardware-cpu-irqreg-isset-1) — Reports whether set as part of the hardware/cpu module.
- [`psyqo::Hardware::CPU::IRQReg::set`](#psyqo-hardware-cpu-irqreg-set-1) — Sets set as part of the hardware/cpu module.

<a id="psyqo-hardware-cpu-flushwritequeue-1"></a>

## `psyqo::Hardware::CPU::flushWriteQueue`

**Purpose.** Performs `flush write queue` as part of the hardware/cpu module.

**Exact declaration**

```cpp
static inline void flushWriteQueue()
```

- **Declared at:** [line 62](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh#L62)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cpu module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cpu.hh"

psyqo::Hardware::CPU::flushWriteQueue();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cpu-irqreg-clear-1"></a>

## `psyqo::Hardware::CPU::IRQReg::clear`

**Purpose.** Clears clear as part of the hardware/cpu module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 51](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh#L51)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cpu module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cpu.hh"

psyqo::Hardware::CPU::IRQReg& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the hardware/cpu module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cpu-irqreg-clear-2"></a>

## `psyqo::Hardware::CPU::IRQReg::clear`

**Purpose.** Clears clear as part of the hardware/cpu module.

**Exact declaration**

```cpp
void clear(IRQ irq)
```

- **Declared at:** [line 50](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh#L50)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `irq` | `IRQ` | Input | Value supplied for `irq`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cpu module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cpu.hh"

// Assume these named values have been initialized with valid data:
// IRQ irq

psyqo::Hardware::CPU::IRQReg& object = /* obtain a valid instance */;

object.clear(irq);
```

**Why choose it.** It provides direct, allocation-conscious access to the hardware/cpu module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cpu-irqreg-isset-1"></a>

## `psyqo::Hardware::CPU::IRQReg::isSet`

**Purpose.** Reports whether set as part of the hardware/cpu module.

**Exact declaration**

```cpp
bool isSet(IRQ irq) const
```

- **Declared at:** [line 52](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh#L52)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `irq` | `IRQ` | Input | Value supplied for `irq`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the hardware/cpu module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cpu.hh"

// Assume these named values have been initialized with valid data:
// IRQ irq

psyqo::Hardware::CPU::IRQReg& object = /* obtain a valid instance */;

auto result = object.isSet(irq);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-hardware-cpu-irqreg-set-1"></a>

## `psyqo::Hardware::CPU::IRQReg::set`

**Purpose.** Sets set as part of the hardware/cpu module.

**Exact declaration**

```cpp
void set(IRQ irq)
```

- **Declared at:** [line 49](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/hardware/cpu.hh#L49)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `irq` | `IRQ` | Input | Value supplied for `irq`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the hardware/cpu module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/hardware/cpu.hh"

// Assume these named values have been initialized with valid data:
// IRQ irq

psyqo::Hardware::CPU::IRQReg& object = /* obtain a valid instance */;

object.set(irq);
```

**Why choose it.** It provides direct, allocation-conscious access to the hardware/cpu module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.
