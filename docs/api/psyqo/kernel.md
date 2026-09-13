# PsyQo API: Kernel

> **Header:** `"psyqo/kernel.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh)

This module covers PSX kernel ownership, interrupts and low-level services. It documents 29 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Application`, `psyqo::DMA::DmaCallback`, `psyqo::Kernel::DMA`, `psyqo::Kernel::IRQ`

## Callable index

- [`psyqo::Kernel::abort`](#psyqo-kernel-abort-1) — Stops the execution of the application.
- [`psyqo::Kernel::assert`](#psyqo-kernel-assert-1) — A simple `assert` macro.
- [`psyqo::Kernel::disableDma`](#psyqo-kernel-disabledma-1) — Disables the given DMA channel.
- [`psyqo::Kernel::enableDma`](#psyqo-kernel-enabledma-1) — Enables the given DMA channel.
- [`psyqo::Kernel::fastEnterCriticalSection`](#psyqo-kernel-fastentercriticalsection-1) — A faster version of `enterCriticalSection`.
- [`psyqo::Kernel::fastLeaveCriticalSection`](#psyqo-kernel-fastleavecriticalsection-1) — A faster version of `leaveCriticalSection`.
- [`psyqo::Kernel::flushCache`](#psyqo-kernel-flushcache-1) — Flushes the i-cache.
- [`psyqo::Kernel::installCrashHandler`](#psyqo-kernel-installcrashhandler-1) — Installs a crash handler for the application.
- [`psyqo::Kernel::Internal::abort`](#psyqo-kernel-internal-abort-1) — Performs `abort` as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::abort`](#psyqo-kernel-internal-abort-2) — Performs `abort` as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::addInitializer`](#psyqo-kernel-internal-addinitializer-1) — Adds initializer as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::addOnFrame`](#psyqo-kernel-internal-addonframe-1) — Adds on frame as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::beginFrame`](#psyqo-kernel-internal-beginframe-1) — Begins frame as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::crashHandler`](#psyqo-kernel-internal-crashhandler-1) — Performs `crash handler` as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::getCop0Status`](#psyqo-kernel-internal-getcop0status-1) — Returns cop0 status as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::prepare`](#psyqo-kernel-internal-prepare-1) — Performs `prepare` as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::pumpCallbacks`](#psyqo-kernel-internal-pumpcallbacks-1) — Performs `pump callbacks` as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::Internal::setCop0Status`](#psyqo-kernel-internal-setcop0status-1) — Sets cop0 status as part of PSX kernel ownership, interrupts and low-level services.
- [`psyqo::Kernel::isKernelTakenOver`](#psyqo-kernel-iskerneltakenover-1) — Returns whether the kernel has been taken over.
- [`psyqo::Kernel::openEvent`](#psyqo-kernel-openevent-1) — A C++ wrapper around the `openEvent` syscall.
- [`psyqo::Kernel::queueCallback`](#psyqo-kernel-queuecallback-1) — Queues a callback to be called from the main thead.
- [`psyqo::Kernel::queueCallbackFromISR`](#psyqo-kernel-queuecallbackfromisr-1) — Queues a callback to be called from the main thead.
- [`psyqo::Kernel::queueIRQHandler`](#psyqo-kernel-queueirqhandler-1) — Queues an IRQ handler to be called from the exception handler.
- [`psyqo::Kernel::queuePsyqoBreakHandler`](#psyqo-kernel-queuepsyqobreakhandler-1) — Queues a break handler for psyqo's reserved category.
- [`psyqo::Kernel::registerDmaEvent`](#psyqo-kernel-registerdmaevent-1) — Sets an ISR callback for a given DMA channel.
- [`psyqo::Kernel::setBreakHandler`](#psyqo-kernel-setbreakhandler-1) — Sets a break handler for a given category.
- [`psyqo::Kernel::takeOverKernel`](#psyqo-kernel-takeoverkernel-1) — Takes over the kernel. Can only be called once inside the main function.
- [`psyqo::Kernel::unregisterDmaEvent`](#psyqo-kernel-unregisterdmaevent-1) — Frees the given DMA callback slot.
- [`psyqo::Kernel::waitForStatus`](#psyqo-kernel-waitforstatus-1) — Waits for a specific status to be set in a register.

<a id="psyqo-kernel-abort-1"></a>

## `psyqo::Kernel::abort`

**Purpose.** Stops the execution of the application.

**Exact declaration**

```cpp
static inline void abort(const char* msg, std::source_location location = std::source_location::current())
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L151)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `msg` | `const char *` | Input | Value supplied for `msg`. See the exact type and module contract. |
| `location` | `std::source_location` | Input | Value supplied for `location`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// const char * msg
// std::source_location location

psyqo::Kernel::abort(msg, location);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-assert-1"></a>

## `psyqo::Kernel::assert`

**Purpose.** A simple `assert` macro.

**Exact declaration**

```cpp
inline void assert(bool condition, const char* message, std::source_location location = std::source_location::current())
```

- **Declared at:** [line 357](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L357)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `condition` | `bool` | Input | Value supplied for `condition`. See the exact type and module contract. |
| `message` | `const char *` | Input | Value supplied for `message`. See the exact type and module contract. |
| `location` | `std::source_location` | Input | Value supplied for `location`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// bool condition
// const char * message
// std::source_location location

psyqo::Kernel::assert(condition, message, location);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-disabledma-1"></a>

## `psyqo::Kernel::disableDma`

**Purpose.** Disables the given DMA channel.

**Exact declaration**

```cpp
void disableDma(DMA channel)
```

- **Declared at:** [line 284](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L284)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channel` | `DMA` | Input | the DMA channel to disable. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// DMA channel

psyqo::Kernel::disableDma(channel);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-enabledma-1"></a>

## `psyqo::Kernel::enableDma`

**Purpose.** Enables the given DMA channel.

**Exact declaration**

```cpp
void enableDma(DMA channel, unsigned priority = 7)
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L277)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channel` | `DMA` | Input | the DMA channel to enable. |
| `priority` | `unsigned int` | Input | the priority of the channel. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// DMA channel
// unsigned int priority

psyqo::Kernel::enableDma(channel, priority);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-fastentercriticalsection-1"></a>

## `psyqo::Kernel::fastEnterCriticalSection`

**Purpose.** A faster version of `enterCriticalSection`.

**Details.** This function is technically equivalent to `enterCriticalSection`.

**Exact declaration**

```cpp
static inline void fastEnterCriticalSection()
```

- **Declared at:** [line 92](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L92)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** false if the critical section was already entered, true otherwise.

**Use it when.** This function is technically equivalent to `enterCriticalSection`.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::fastEnterCriticalSection();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-fastleavecriticalsection-1"></a>

## `psyqo::Kernel::fastLeaveCriticalSection`

**Purpose.** A faster version of `leaveCriticalSection`.

**Details.** This function is technically equivalent to `leaveCriticalSection`.

**Exact declaration**

```cpp
static inline void fastLeaveCriticalSection()
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L103)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is technically equivalent to `leaveCriticalSection`.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::fastLeaveCriticalSection();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-flushcache-1"></a>

## `psyqo::Kernel::flushCache`

**Purpose.** Flushes the i-cache.

**Details.** This function is used to flush the i-cache. This is required when the application has written some code to memory.

**Exact declaration**

```cpp
void flushCache()
```

- **Declared at:** [line 269](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L269)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is used to flush the i-cache. This is required when the application has written some code to memory.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::flushCache();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-installcrashhandler-1"></a>

## `psyqo::Kernel::installCrashHandler`

**Purpose.** Installs a crash handler for the application.

**Details.** This function installs a crash handler for the application. The crash handler will be called when the application crashes, such when an unhandled exception occurs. It will display a message on the screen with the crash information, including the exception type, the exception address, and the value of all the registers at the time of the crash. The crash handler requires the system font to be uploaded to VRAM, at the default location (960, 464). If the system font is not available, the crash handler will not be able to display the message properly. As usual, this function should be called from `main`, before handing over control to the application, it should only be called once, and its associated cost will only be added to the binary if it is called.

**Exact declaration**

```cpp
void installCrashHandler()
```

- **Declared at:** [line 216](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L216)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function installs a crash handler for the application. The crash handler will be called when the application crashes, such when an unhandled exception occurs. It will display a message on the screen with the crash information, including the exception type, the exception address, and the value of all the registers at the time of the crash. The crash handler requires the system font to be uploaded to VRAM, at the default location (960, 464). If the system font is not available, the crash handler will not be able to display the message properly. As usual, this function should be called from `main`, before handing over control to the application, it should only be called once, and its associated cost will only be added to the binary if it is called.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::installCrashHandler();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-internal-abort-1"></a>

## `psyqo::Kernel::Internal::abort`

**Purpose.** Performs `abort` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void abort()
```

- **Declared at:** [line 82](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L82)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::Internal::abort();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-internal-abort-2"></a>

## `psyqo::Kernel::Internal::abort`

**Purpose.** Performs `abort` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void abort(const char* msg, std::source_location location = std::source_location::current())
```

- **Declared at:** [line 81](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L81)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `msg` | `const char *` | Input | Value supplied for `msg`. See the exact type and module contract. |
| `location` | `std::source_location` | Input | Value supplied for `location`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// const char * msg
// std::source_location location

psyqo::Kernel::Internal::abort(msg, location);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-internal-addinitializer-1"></a>

## `psyqo::Kernel::Internal::addInitializer`

**Purpose.** Adds initializer as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void addInitializer(eastl::function<void(Application&)>&& lambda)
```

- **Declared at:** [line 348](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L348)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lambda` | `eastl::function<void (Application &)> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Application &)> && lambda

psyqo::Kernel::Internal::addInitializer(lambda);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-internal-addonframe-1"></a>

## `psyqo::Kernel::Internal::addOnFrame`

**Purpose.** Adds on frame as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void addOnFrame(eastl::function<void()>&& lambda)
```

- **Declared at:** [line 349](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L349)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void ()> && lambda

psyqo::Kernel::Internal::addOnFrame(lambda);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-internal-beginframe-1"></a>

## `psyqo::Kernel::Internal::beginFrame`

**Purpose.** Begins frame as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void beginFrame()
```

- **Declared at:** [line 350](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L350)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::Internal::beginFrame();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-internal-crashhandler-1"></a>

## `psyqo::Kernel::Internal::crashHandler`

**Purpose.** Performs `crash handler` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void crashHandler(uint32_t exceptionCode, uint32_t* kernelRegisters)
```

- **Declared at:** [line 351](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L351)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `exceptionCode` | `uint32_t` | Input | Value supplied for `exceptionCode`. See the exact type and module contract. |
| `kernelRegisters` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `kernelRegisters`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// uint32_t exceptionCode
// uint32_t * kernelRegisters

psyqo::Kernel::Internal::crashHandler(exceptionCode, kernelRegisters);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-internal-getcop0status-1"></a>

## `psyqo::Kernel::Internal::getCop0Status`

**Purpose.** Returns cop0 status as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static inline uint32_t getCop0Status()
```

- **Declared at:** [line 65](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L65)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

auto result = psyqo::Kernel::Internal::getCop0Status();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-internal-prepare-1"></a>

## `psyqo::Kernel::Internal::prepare`

**Purpose.** Performs `prepare` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void prepare(Application&)
```

- **Declared at:** [line 347](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L347)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Application &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// Application & arg1

psyqo::Kernel::Internal::prepare(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-internal-pumpcallbacks-1"></a>

## `psyqo::Kernel::Internal::pumpCallbacks`

**Purpose.** Performs `pump callbacks` as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
void pumpCallbacks()
```

- **Declared at:** [line 346](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L346)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::Internal::pumpCallbacks();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-kernel-internal-setcop0status-1"></a>

## `psyqo::Kernel::Internal::setCop0Status`

**Purpose.** Sets cop0 status as part of PSX kernel ownership, interrupts and low-level services.

**Exact declaration**

```cpp
static inline void setCop0Status(uint32_t r)
```

- **Declared at:** [line 75](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L75)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `r` | `uint32_t` | Input | Value supplied for `r`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// uint32_t r

psyqo::Kernel::Internal::setCop0Status(r);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-iskerneltakenover-1"></a>

## `psyqo::Kernel::isKernelTakenOver`

**Purpose.** Returns whether the kernel has been taken over.

**Exact declaration**

```cpp
bool isKernelTakenOver()
```

- **Declared at:** [line 198](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L198)
- **Kind:** `function decl`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

auto result = psyqo::Kernel::isKernelTakenOver();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-kernel-openevent-1"></a>

## `psyqo::Kernel::openEvent`

**Purpose.** A C++ wrapper around the `openEvent` syscall.

**Details.** This enables the application to register a C++ lambda for the kernel's OpenEvent call. This will allocate an internal slot, with currently no mechanism to free it. This means that calling `closeEvent` on the resulting event will leak resources. If psyqo took over the kernel, this function will no longer work.

**Exact declaration**

```cpp
uint32_t openEvent(uint32_t classId, uint32_t spec, uint32_t mode, eastl::function<void()>&& lambda)
```

- **Declared at:** [line 246](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L246)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `classId` | `uint32_t` | Input | Value supplied for `classId`. See the exact type and module contract. |
| `spec` | `uint32_t` | Input | Value supplied for `spec`. See the exact type and module contract. |
| `mode` | `uint32_t` | Input | Value supplied for `mode`. See the exact type and module contract. |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** This enables the application to register a C++ lambda for the kernel's OpenEvent call. This will allocate an internal slot, with currently no mechanism to free it. This means that calling `closeEvent` on the resulting event will leak resources. If psyqo took over the kernel, this function will no longer work.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// uint32_t classId
// uint32_t spec
// uint32_t mode
// eastl::function<void ()> && lambda

auto result = psyqo::Kernel::openEvent(classId, spec, mode, lambda);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-queuecallback-1"></a>

## `psyqo::Kernel::queueCallback`

**Purpose.** Queues a callback to be called from the main thead.

**Details.** This function is used to queue a callback to be called from the main thread, during idle moments like various blocking operations. This variant is safe to call from the main thread only. Its usefulness from the main thread is limited, and could be considered the same as JavaScript's `process.nextTick()`, meaning it's a great way to avoid get out of a deep callstack.

**Exact declaration**

```cpp
void queueCallback(eastl::function<void()>&& lambda)
```

- **Declared at:** [line 303](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L303)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is used to queue a callback to be called from the main thread, during idle moments like various blocking operations. This variant is safe to call from the main thread only. Its usefulness from the main thread is limited, and could be considered the same as JavaScript's `process.nextTick()`, meaning it's a great way to avoid get out of a deep callstack.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void ()> && lambda

psyqo::Kernel::queueCallback(lambda);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-queuecallbackfromisr-1"></a>

## `psyqo::Kernel::queueCallbackFromISR`

**Purpose.** Queues a callback to be called from the main thead.

**Details.** This function is used to queue a callback to be called from the main thead, during idle moments like various blocking operations. This variant is safe to call from an interrupt handler. This is how to idiomatically execute something safely from an interrupt handler.

**Exact declaration**

```cpp
void queueCallbackFromISR(eastl::function<void()>&& lambda)
```

- **Declared at:** [line 314](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L314)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is used to queue a callback to be called from the main thead, during idle moments like various blocking operations. This variant is safe to call from an interrupt handler. This is how to idiomatically execute something safely from an interrupt handler.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void ()> && lambda

psyqo::Kernel::queueCallbackFromISR(lambda);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-queueirqhandler-1"></a>

## `psyqo::Kernel::queueIRQHandler`

**Purpose.** Queues an IRQ handler to be called from the exception handler.

**Details.** This function is used to queue an IRQ handler to be called from the exception handler when the kernel has been taken over. While it is technically possible to queue VBlank, it should solely be reserved for the GPU object instead. Also, note that the kernel has its own DMA IRQ handler, and that the `registerDmaEvent` function should be used instead of trying to queue a handler for the DMA IRQ. The specified handler will be called from the exception handler, with the same restrictions as for any other interrupt handler. The queued handlers will be called in the order they were queued, but it is recommended to only queue one handler per IRQ.

**Exact declaration**

```cpp
void queueIRQHandler(IRQ irq, eastl::function<void()>&& lambda)
```

- **Declared at:** [line 235](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L235)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `irq` | `IRQ` | Input | The IRQ to handle. |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | The function to call when the IRQ is triggered. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is used to queue an IRQ handler to be called from the exception handler when the kernel has been taken over. While it is technically possible to queue VBlank, it should solely be reserved for the GPU object instead. Also, note that the kernel has its own DMA IRQ handler, and that the `registerDmaEvent` function should be used instead of trying to queue a handler for the DMA IRQ. The specified handler will be called from the exception handler, with the same restrictions as for any other interrupt handler. The queued handlers will be called in the order they were queued, but it is recommended to only queue one handler per IRQ.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// IRQ irq
// eastl::function<void ()> && lambda

psyqo::Kernel::queueIRQHandler(irq, lambda);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-queuepsyqobreakhandler-1"></a>

## `psyqo::Kernel::queuePsyqoBreakHandler`

**Purpose.** Queues a break handler for psyqo's reserved category.

**Exact declaration**

```cpp
void queuePsyqoBreakHandler(eastl::function<bool(uint32_t)>&& handler)
```

- **Declared at:** [line 343](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L343)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handler` | `eastl::function<bool (uint32_t)> &&` | Consumed or moved input | The handler to call when a break occurs. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<bool (uint32_t)> && handler

psyqo::Kernel::queuePsyqoBreakHandler(handler);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-registerdmaevent-1"></a>

## `psyqo::Kernel::registerDmaEvent`

**Purpose.** Sets an ISR callback for a given DMA channel.

**Details.** The PSYQo kernel registers a dispatcher interrupt handler for DMA interrupts, and this function registers a callback function for a given DMA channel. Multiple callbacks can be registered for a given channel. All the callbacks registered will be called sequentially during the dispatcher interrupt handler. Note this means the callbacks will be called from the interrupt handler, with the same restrictions as for any other interrupt handler.

**Exact declaration**

```cpp
unsigned registerDmaEvent(DMA channel, eastl::function<void()>&& lambda)
```

- **Declared at:** [line 261](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L261)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channel` | `DMA` | Input | Value supplied for `channel`. See the exact type and module contract. |
| `lambda` | `eastl::function<void ()> &&` | Consumed or moved input | Value supplied for `lambda`. See the exact type and module contract. |

**Returns.** unsigned A slot id for the given callback.

**Use it when.** The PSYQo kernel registers a dispatcher interrupt handler for DMA interrupts, and this function registers a callback function for a given DMA channel. Multiple callbacks can be registered for a given channel. All the callbacks registered will be called sequentially during the dispatcher interrupt handler. Note this means the callbacks will be called from the interrupt handler, with the same restrictions as for any other interrupt handler.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// DMA channel
// eastl::function<void ()> && lambda

auto result = psyqo::Kernel::registerDmaEvent(channel, lambda);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-setbreakhandler-1"></a>

## `psyqo::Kernel::setBreakHandler`

**Purpose.** Sets a break handler for a given category.

**Details.** This function is used to set a break handler for a given category. The category is technically the upper 10 bits of the break code, and the handler is a function that takes the lower 10 bits of the break code. The handler should return true if it handled the break, and false otherwise. The handler will be called from the exception handler, with the same restrictions as for any other interrupt handler. Note that the category is actually limited to 16 categories by psyqo, from 0 to 15. It is also worth noting that category 0 is usually reserved for pcdrv, category 7 is reserved by the compiler to emit division by zero checks, and psyqo uses category 14 for its own purposes. Only one handler can be set per category, and trying to set a handler for a category that already has a handler will cause an assertion failure.

**Exact declaration**

```cpp
void setBreakHandler(unsigned category, eastl::function<bool(uint32_t)>&& handler)
```

- **Declared at:** [line 336](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L336)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `category` | `unsigned int` | Input | The category to handle. |
| `handler` | `eastl::function<bool (uint32_t)> &&` | Consumed or moved input | Value supplied for `handler`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function is used to set a break handler for a given category. The category is technically the upper 10 bits of the break code, and the handler is a function that takes the lower 10 bits of the break code. The handler should return true if it handled the break, and false otherwise. The handler will be called from the exception handler, with the same restrictions as for any other interrupt handler. Note that the category is actually limited to 16 categories by psyqo, from 0 to 15. It is also worth noting that category 0 is usually reserved for pcdrv, category 7 is reserved by the compiler to emit division by zero checks, and psyqo uses category 14 for its own purposes. Only one handler can be set per category, and trying to set a handler for a category that already has a handler will cause an assertion failure.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// unsigned int category
// eastl::function<bool (uint32_t)> && handler

psyqo::Kernel::setBreakHandler(category, handler);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-kernel-takeoverkernel-1"></a>

## `psyqo::Kernel::takeOverKernel`

**Purpose.** Takes over the kernel. Can only be called once inside the main function.

**Details.** This function will make psyqo take over the retail kernel. This means the application will no longer be able to call any of the kernel functions, and will have to rely on the psyqo kernel instead. Debugging features from third party addons which hook into the kernel will no longer work. Most calls to the kernel will either be no-ops or will crash the application. Most notably, only the `printf` call will be redirected to psyqo's printf, but will not be printing anywhere, so only emulators hooking into A0 calls will be able to see the output. Disabling the kernel is a one-way operation, and cannot be undone. The kernel will be taken over before the first call to `prepare`. The exception handler that psyqo installs will not be able to catch problems, but is much more lightweight and faster than the retail one. Also, 60kB of memory can be reclaimed, and linking the binary with -Xlinker --defsym=TLOAD_ADDR=0x80001000 will allow the application to do just that. This requires a loader able to write into the kernel while disabling interrupts. The ps1-packer tool can achieve that. The first 4kB of memory is reserved for the psyqo kernel. It is noteworthy that while the pros of taking over the kernel are significant, the cons are also significant. The loss of debugging, flexibility, and retail kernel features may not be worth it for most application cases, and should be considered carefully. Last but not least, like with most psyqo features, the added payload to the binary to support the feature will only occur if this function is called.

**Exact declaration**

```cpp
void takeOverKernel()
```

- **Declared at:** [line 193](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L193)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This function will make psyqo take over the retail kernel. This means the application will no longer be able to call any of the kernel functions, and will have to rely on the psyqo kernel instead. Debugging features from third party addons which hook into the kernel will no longer work. Most calls to the kernel will either be no-ops or will crash the application. Most notably, only the `printf` call will be redirected to psyqo's printf, but will not be printing anywhere, so only emulators hooking into A0 calls will be able to see the output. Disabling the kernel is a one-way operation, and cannot be undone. The kernel will be taken over before the first call to `prepare`. The exception handler that psyqo installs will not be able to catch problems, but is much more lightweight and faster than the retail one. Also, 60kB of memory can be reclaimed, and linking the binary with -Xlinker --defsym=TLOAD_ADDR=0x80001000 will allow the application to do just that. This requires a loader able to write into the kernel while disabling interrupts. The ps1-packer tool can achieve that. The first 4kB of memory is reserved for the psyqo kernel. It is noteworthy that while the pros of taking over the kernel are significant, the cons are also significant. The loss of debugging, flexibility, and retail kernel features may not be worth it for most application cases, and should be considered carefully. Last but not least, like with most psyqo features, the added payload to the binary to support the feature will only occur if this function is called.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

psyqo::Kernel::takeOverKernel();
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-unregisterdmaevent-1"></a>

## `psyqo::Kernel::unregisterDmaEvent`

**Purpose.** Frees the given DMA callback slot.

**Exact declaration**

```cpp
void unregisterDmaEvent(unsigned slot)
```

- **Declared at:** [line 291](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L291)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `unsigned int` | Input | The slot to free, as returned by `registerDmaEvent`. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Assume these named values have been initialized with valid data:
// unsigned int slot

psyqo::Kernel::unregisterDmaEvent(slot);
```

**Why choose it.** It provides direct, allocation-conscious access to PSX kernel ownership, interrupts and low-level services. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-kernel-waitforstatus-1"></a>

## `psyqo::Kernel::waitForStatus`

**Purpose.** Waits for a specific status to be set in a register.

**Exact declaration**

```cpp
template <typename T> static void waitForStatus(T mask, T expected, const volatile T* value)
```

- **Declared at:** [line 118](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/kernel.hh#L118)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mask` | `T` | Input | The mask to apply to the register. |
| `expected` | `T` | Input | The expected value of the register after applying the mask. |
| `value` | `const volatile T *` | Input | The address of the register to wait for. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need PSX kernel ownership, interrupts and low-level services and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/kernel.hh"

// Replace these template arguments with types or values accepted by the declaration:
// T

// Assume these named values have been initialized with valid data:
// T mask
// T expected
// const volatile T * value

psyqo::Kernel::waitForStatus<T>(mask, expected, value);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
