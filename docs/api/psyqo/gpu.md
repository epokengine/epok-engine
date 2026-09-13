# PsyQo API: Gpu

> **Header:** `"psyqo/gpu.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh)

This module covers GPU setup, command submission and frame synchronization. It documents 46 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::GPU::ColorMode`, `psyqo::GPU::Configuration`, `psyqo::GPU::Interlace`, `psyqo::GPU::MiscSetting`, `psyqo::GPU::Resolution`, `psyqo::GPU::VideoMode`

## Callable index

- [`psyqo::GPU::armPeriodicTimer`](#psyqo-gpu-armperiodictimer-1) — Creates a periodic timer.
- [`psyqo::GPU::armTimer`](#psyqo-gpu-armtimer-1) — Creates a single-use timer.
- [`psyqo::GPU::cancelTimer`](#psyqo-gpu-canceltimer-1) — Cancels a timer.
- [`psyqo::GPU::chain`](#psyqo-gpu-chain-1) — Chains an already constructed DMA chain to the next DMA chain transfer.
- [`psyqo::GPU::chain`](#psyqo-gpu-chain-2) — Chains a fragment to the next DMA chain transfer.
- [`psyqo::GPU::chain`](#psyqo-gpu-chain-3) — Chains an ordering table to the next DMA chain transfer.
- [`psyqo::GPU::changeTimerPeriod`](#psyqo-gpu-changetimerperiod-1) — Changes the period of a periodic timer.
- [`psyqo::GPU::clear`](#psyqo-gpu-clear-1) — Immediately clears the drawing buffer.
- [`psyqo::GPU::delay`](#psyqo-gpu-delay-1) — Delays the coroutine for a specified amount of time.
- [`psyqo::GPU::disableScissor`](#psyqo-gpu-disablescissor-1) — Immediately disables the scissoring of the VRAM.
- [`psyqo::GPU::enableScissor`](#psyqo-gpu-enablescissor-1) — Enables the scissoring of the VRAM.
- [`psyqo::GPU::getClear`](#psyqo-gpu-getclear-1) — Sets a `FastFill` primitive to clear the current drawing buffer.
- [`psyqo::GPU::getFrameCount`](#psyqo-gpu-getframecount-1) — Returns the number of frames rendered by the GPU so far.
- [`psyqo::GPU::getNextClear`](#psyqo-gpu-getnextclear-1) — Sets a `FastFill` primitive to clear the next drawing buffer.
- [`psyqo::GPU::getNextScissor`](#psyqo-gpu-getnextscissor-1) — Gets the next scissoring region.
- [`psyqo::GPU::getParity`](#psyqo-gpu-getparity-1) — Get the index of the current display buffer.
- [`psyqo::GPU::getRefreshRate`](#psyqo-gpu-getrefreshrate-1) — Returns the refresh rate of the GPU.
- [`psyqo::GPU::getScissor`](#psyqo-gpu-getscissor-1) — Gets the current scissoring region.
- [`psyqo::GPU::initialize`](#psyqo-gpu-initialize-1) — Performs `initialize` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::isChainIdle`](#psyqo-gpu-ischainidle-1) — Gets the status of the background DMA transfer operation when initiated by a frame flip.
- [`psyqo::GPU::isChainTransferred`](#psyqo-gpu-ischaintransferred-1) — Gets the status of the background DMA transfer operation when initiated by a frame flip.
- [`psyqo::GPU::isChainTransferring`](#psyqo-gpu-ischaintransferring-1) — Gets the status of the background DMA transfer operation when initiated by a frame flip.
- [`psyqo::GPU::now`](#psyqo-gpu-now-1) — Gets the current timestamp in microseconds.
- [`psyqo::GPU::pauseTimer`](#psyqo-gpu-pausetimer-1) — Pauses a timer.
- [`psyqo::GPU::pumpCallbacks`](#psyqo-gpu-pumpcallbacks-1) — Runs one round of event processing.
- [`psyqo::GPU::reinitialize`](#psyqo-gpu-reinitialize-1) — Performs `reinitialize` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::resumeTimer`](#psyqo-gpu-resumetimer-1) — Resumes a paused timer.
- [`psyqo::GPU::sendChain`](#psyqo-gpu-sendchain-1) — Immediately sends the current DMA chain
- [`psyqo::GPU::sendChain`](#psyqo-gpu-sendchain-2) — Initiates the transfer of the current DMA chain.
- [`psyqo::GPU::sendFragment`](#psyqo-gpu-sendfragment-1) — Immediately sends a fragment to the GPU. This is a blocking operation. See the fragments.hh file for more information.
- [`psyqo::GPU::sendFragment`](#psyqo-gpu-sendfragment-2) — Sends a fragment to the GPU as a non-blocking call.
- [`psyqo::GPU::sendPrimitive`](#psyqo-gpu-sendprimitive-1) — Sends a primitive to the GPU. This is a blocking call.
- [`psyqo::GPU::sendRaw`](#psyqo-gpu-sendraw-1) — Sends a raw 32 bits value to the Data register of the GPU.
- [`psyqo::GPU::uploadToVRAM`](#psyqo-gpu-uploadtovram-1) — Uploads a buffer to the VRAM as a blocking call.
- [`psyqo::GPU::uploadToVRAM`](#psyqo-gpu-uploadtovram-2) — Uploads a buffer to the VRAM as a non-blocking call.
- [`psyqo::GPU::waitChainIdle`](#psyqo-gpu-waitchainidle-1) — Waits until the background DMA transfer operation initiated by a frame flip is complete.
- [`psyqo::GPU::waitFifo`](#psyqo-gpu-waitfifo-1) — Waits until the GPU's FIFO is ready to receive data.
- [`psyqo::GPU::waitReady`](#psyqo-gpu-waitready-1) — Waits until the GPU is ready to send a command.
- [`psyqo::timer_literals::operator""_ms`](#psyqo-timer-literals-operator-ms-1) — Performs `operator "" ms` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_ms`](#psyqo-timer-literals-operator-ms-2) — Performs `operator "" ms` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_ns`](#psyqo-timer-literals-operator-ns-1) — Performs `operator "" ns` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_ns`](#psyqo-timer-literals-operator-ns-2) — Literal operators for time units.
- [`psyqo::timer_literals::operator""_s`](#psyqo-timer-literals-operator-s-1) — Performs `operator "" s` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_s`](#psyqo-timer-literals-operator-s-2) — Performs `operator "" s` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_us`](#psyqo-timer-literals-operator-us-1) — Performs `operator "" us` as part of GPU setup, command submission and frame synchronization.
- [`psyqo::timer_literals::operator""_us`](#psyqo-timer-literals-operator-us-2) — Performs `operator "" us` as part of GPU setup, command submission and frame synchronization.

<a id="psyqo-gpu-armperiodictimer-1"></a>

## `psyqo::GPU::armPeriodicTimer`

**Purpose.** Creates a periodic timer.

**Details.** This method will create a periodic timer. The timer will fire every `period` microseconds. See the `armTimer` method for more information about timers in general. Periodic timers will first fire at `now() + period` microseconds, and then every `period` microseconds thereafter. They will never be canceled automatically.

**Exact declaration**

```cpp
unsigned armPeriodicTimer(uint32_t period, eastl::function<void(uint32_t)> &&callback)
```

- **Declared at:** [line 461](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L461)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `period` | `uint32_t` | Input | The period of the timer in microseconds. |
| `callback` | `eastl::function<void (uint32_t)> &&` | Consumed or moved input | The callback function to be called when the timer expires. |

**Returns.** The id of the created timer.

**Use it when.** This method will create a periodic timer. The timer will fire every `period` microseconds. See the `armTimer` method for more information about timers in general. Periodic timers will first fire at `now() + period` microseconds, and then every `period` microseconds thereafter. They will never be canceled automatically.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t period
// eastl::function<void (uint32_t)> && callback

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.armPeriodicTimer(period, callback);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-armtimer-1"></a>

## `psyqo::GPU::armTimer`

**Purpose.** Creates a single-use timer.

**Details.** This method will create a single-use timer. The timer will fire after the specified deadline has passed. Timers will only fire during the idle period of the CPU, for example during calls to `sendFragment`, or between calls to the `frame` method of the current scene. If the scene takes too long to compute, timers may significantly be delayed past their set deadline. The deadline can be computed based on the return value of the `now()` method. It is okay if the deadline rolls over their 32 bits span. Simply doing `gpu().now() + DELAY_IN_MICROSECONDS` will still work, as long as the delay isn't greater than 30 minutes. Single-use timers will automatically be disabled upon being fired, and their id will no longer be valid. The returned id is guaranteed to be unique across active timers, but may collision with the id of timers that got canceled or got disabled on their own.

**Exact declaration**

```cpp
uintptr_t armTimer(uint32_t deadline, eastl::function<void(uint32_t)> &&callback)
```

- **Declared at:** [line 435](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L435)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `deadline` | `uint32_t` | Input | The deadline of the timer in microseconds. |
| `callback` | `eastl::function<void (uint32_t)> &&` | Consumed or moved input | The callback function to be called when the timer expires. |

**Returns.** The id of the created timer.

**Use it when.** This method will create a single-use timer. The timer will fire after the specified deadline has passed. Timers will only fire during the idle period of the CPU, for example during calls to `sendFragment`, or between calls to the `frame` method of the current scene. If the scene takes too long to compute, timers may significantly be delayed past their set deadline. The deadline can be computed based on the return value of the `now()` method. It is okay if the deadline rolls over their 32 bits span. Simply doing `gpu().now() + DELAY_IN_MICROSECONDS` will still work, as long as the delay isn't greater than 30 minutes. Single-use timers will automatically be disabled upon being fired, and their id will no longer be valid. The returned id is guaranteed to be unique across active timers, but may collision with the id of timers that got canceled or got disabled on their own.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t deadline
// eastl::function<void (uint32_t)> && callback

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.armTimer(deadline, callback);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-canceltimer-1"></a>

## `psyqo::GPU::cancelTimer`

**Purpose.** Cancels a timer.

**Details.** This method will cancel an active timer. The timer will no longer fire. Its id will no longer be valid.

**Exact declaration**

```cpp
void cancelTimer(uintptr_t id)
```

- **Declared at:** [line 508](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L508)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uintptr_t` | Input | The id of the timer to cancel. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will cancel an active timer. The timer will no longer fire. Its id will no longer be valid.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uintptr_t id

psyqo::GPU& object = /* obtain a valid instance */;

object.cancelTimer(id);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-chain-1"></a>

## `psyqo::GPU::chain`

**Purpose.** Chains an already constructed DMA chain to the next DMA chain transfer.

**Details.** This method will chain an already constructed DMA chain to the next DMA chain transfer. This is an even more complex operation than the previous `chain` method, as it requires the user to construct the DMA chain manually. Some helpers are provided in the `Fragments` namespace.

**Exact declaration**

```cpp
template <Fragment Frag1, Fragment Frag2> void chain(Frag1 *first, Frag2 *last)
```

- **Declared at:** [line 334](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L334)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `Frag1 *` | Input/output; inspect the function contract | The pointer to the first fragment of the chain. |
| `last` | `Frag2 *` | Input/output; inspect the function contract | The pointer to the last fragment of the chain. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will chain an already constructed DMA chain to the next DMA chain transfer. This is an even more complex operation than the previous `chain` method, as it requires the user to construct the DMA chain manually. Some helpers are provided in the `Fragments` namespace.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Frag1, Frag2

// Assume these named values have been initialized with valid data:
// Frag1 * first
// Frag2 * last

psyqo::GPU& object = /* obtain a valid instance */;

object.chain<Frag1, Frag2>(first, last);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-chain-2"></a>

## `psyqo::GPU::chain`

**Purpose.** Chains a fragment to the next DMA chain transfer.

**Details.** This method will chain a fragment to the next DMA chain transfer. DMA Chaining is a complex operation, and it is recommended that you use the `sendFragment` method instead if you are unsure. This can be used while a DMA chain is being sent. Use the `sendChain` method to transfer the DMA chain during the current frame, or simply return from the current scene's `frame` method to transfer the DMA chain automatically during the frame flip operation. Note that the latter means the DMA chain will render on the _next_ rendered frame, thus creating a sort of triple buffering system. The constructed DMA chain will thus need to be using the `Next` variants of the primitive constructors, if applicable.

**Exact declaration**

```cpp
template <Fragment Frag> void chain(Frag &fragment)
```

- **Declared at:** [line 320](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L320)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fragment` | `Frag &` | Input/output; inspect the function contract | The fragment to chain. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will chain a fragment to the next DMA chain transfer. DMA Chaining is a complex operation, and it is recommended that you use the `sendFragment` method instead if you are unsure. This can be used while a DMA chain is being sent. Use the `sendChain` method to transfer the DMA chain during the current frame, or simply return from the current scene's `frame` method to transfer the DMA chain automatically during the frame flip operation. Note that the latter means the DMA chain will render on the _next_ rendered frame, thus creating a sort of triple buffering system. The constructed DMA chain will thus need to be using the `Next` variants of the primitive constructors, if applicable.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Frag

// Assume these named values have been initialized with valid data:
// Frag & fragment

psyqo::GPU& object = /* obtain a valid instance */;

object.chain<Frag>(fragment);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-chain-3"></a>

## `psyqo::GPU::chain`

**Purpose.** Chains an ordering table to the next DMA chain transfer.

**Details.** This method will chain an ordering table to the next DMA chain transfer. The ordering table table will be cleared automatically after the transfer is complete.

**Exact declaration**

```cpp
template <size_t N, Safe safety = Safe::Yes> void chain(OrderingTable<N, safety> &table)
```

- **Declared at:** [line 349](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L349)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `OrderingTable<N, safety> &` | Input/output; inspect the function contract | The ordering table to chain. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will chain an ordering table to the next DMA chain transfer. The ordering table table will be cleared automatically after the transfer is complete.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// N, safety

// Assume these named values have been initialized with valid data:
// OrderingTable<N, safety> & table

psyqo::GPU& object = /* obtain a valid instance */;

object.chain<N, safety>(table);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-changetimerperiod-1"></a>

## `psyqo::GPU::changeTimerPeriod`

**Purpose.** Changes the period of a periodic timer.

**Details.** This method will change the period of a periodic timer. The timer now will fire every `period` microseconds instead of its previous period. When the `reset` argument is false, the next deadline for the timer will be adjusted according to the difference between the new period and the previous one. If the new period is shorter, and the deadline goes in the past, the timer will fire as soon as possible. When the `reset` argument is true, the new deadline will simply be set to the new `period`. This method has no effect if the timer is not periodic.

**Exact declaration**

```cpp
void changeTimerPeriod(uintptr_t id, uint32_t period, bool reset = false)
```

- **Declared at:** [line 477](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L477)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uintptr_t` | Input | The id of the timer to change. |
| `period` | `uint32_t` | Input | The new period of the timer. |
| `reset` | `bool` | Input | The timer's deadline will be adjusted to the new period if false. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will change the period of a periodic timer. The timer now will fire every `period` microseconds instead of its previous period. When the `reset` argument is false, the next deadline for the timer will be adjusted according to the difference between the new period and the previous one. If the new period is shorter, and the deadline goes in the past, the timer will fire as soon as possible. When the `reset` argument is true, the new deadline will simply be set to the new `period`. This method has no effect if the timer is not periodic.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uintptr_t id
// uint32_t period
// bool reset

psyqo::GPU& object = /* obtain a valid instance */;

object.changeTimerPeriod(id, period, reset);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-clear-1"></a>

## `psyqo::GPU::clear`

**Purpose.** Immediately clears the drawing buffer.

**Details.** This method will immediately clear the drawing buffer with the specified color.

**Exact declaration**

```cpp
void clear(Color bg = {{0, 0, 0}})
```

- **Declared at:** [line 149](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L149)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bg` | `Color` | Input | The color to fill the drawing buffer with. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will immediately clear the drawing buffer with the specified color.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// Color bg

psyqo::GPU& object = /* obtain a valid instance */;

object.clear(bg);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-delay-1"></a>

## `psyqo::GPU::delay`

**Purpose.** Delays the coroutine for a specified amount of time.

**Details.** This method will delay the coroutine for a specified amount of time. This is a coroutine-friendly version of the `armTimer` method. The coroutine will be suspended until the delay has passed. The delay is specified in microseconds, and the timer literals can be used to specify the delay. The function can only be called from within a coroutine, and is meant to be used with the `co_await` keyword.

**Exact declaration**

```cpp
TimerAwaiter delay(uint32_t microseconds)
```

- **Declared at:** [line 448](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L448)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `microseconds` | `uint32_t` | Input | Value supplied for `microseconds`. See the exact type and module contract. |

**Returns.** TimerAwaiter The awaitable object to be used with the `co_await` keyword.

**Use it when.** This method will delay the coroutine for a specified amount of time. This is a coroutine-friendly version of the `armTimer` method. The coroutine will be suspended until the delay has passed. The delay is specified in microseconds, and the timer literals can be used to specify the delay. The function can only be called from within a coroutine, and is meant to be used with the `co_await` keyword.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t microseconds

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.delay(microseconds);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-disablescissor-1"></a>

## `psyqo::GPU::disableScissor`

**Purpose.** Immediately disables the scissoring of the VRAM.

**Exact declaration**

```cpp
void disableScissor()
```

- **Declared at:** [line 245](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L245)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.disableScissor();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-enablescissor-1"></a>

## `psyqo::GPU::enableScissor`

**Purpose.** Enables the scissoring of the VRAM.

**Details.** This method will enable the scissoring of the VRAM, and will clip the drawing to the currently active buffer.

**Exact declaration**

```cpp
void enableScissor()
```

- **Declared at:** [line 253](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L253)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will enable the scissoring of the VRAM, and will clip the drawing to the currently active buffer.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.enableScissor();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-getclear-1"></a>

## `psyqo::GPU::getClear`

**Purpose.** Sets a `FastFill` primitive to clear the current drawing buffer.

**Details.** This method will set the `FastFill` primitive passed as an argument in a way to completely clear the current drawing buffer with the specified color. This will be done in accordance to the current drawing buffer settings.

**Exact declaration**

```cpp
void getClear(Prim::FastFill &ff, Color bg = {{0, 0, 0}}) const
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L161)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ff` | `Prim::FastFill &` | Input/output; inspect the function contract | The `FastFill` primitive to set. |
| `bg` | `Color` | Input | Value supplied for `bg`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will set the `FastFill` primitive passed as an argument in a way to completely clear the current drawing buffer with the specified color. This will be done in accordance to the current drawing buffer settings.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// Prim::FastFill & ff
// Color bg

psyqo::GPU& object = /* obtain a valid instance */;

object.getClear(ff, bg);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-getframecount-1"></a>

## `psyqo::GPU::getFrameCount`

**Purpose.** Returns the number of frames rendered by the GPU so far.

**Details.** This returns the internal frame counter being kept by the GPU class. The 32 bits value will wrap around when it reaches 2^32 frames, which is 2 years, 3 months, 7 days, 6 hours, 6 minutes and 28.27 seconds when running constantly at a 60Hz refresh rate. This counter will be incremented during the frame flip operation by the appropriate number of hardware frames which have passed since the last frame flip. In other words, this counter monotonically increases by one for each vsync event that occurred during the last rendering.

**Exact declaration**

```cpp
uint32_t getFrameCount() const
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L125)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** This returns the internal frame counter being kept by the GPU class. The 32 bits value will wrap around when it reaches 2^32 frames, which is 2 years, 3 months, 7 days, 6 hours, 6 minutes and 28.27 seconds when running constantly at a 60Hz refresh rate. This counter will be incremented during the frame flip operation by the appropriate number of hardware frames which have passed since the last frame flip. In other words, this counter monotonically increases by one for each vsync event that occurred during the last rendering.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.getFrameCount();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-getnextclear-1"></a>

## `psyqo::GPU::getNextClear`

**Purpose.** Sets a `FastFill` primitive to clear the next drawing buffer.

**Details.** This method will set the `FastFill` primitive passed as an argument in a way to completely clear the next drawing buffer with the specified color. This will be done in accordance to the next drawing buffer settings, after a flip. This is useful for clearing the buffer within a dma chain to be sent during the frame flip.

**Exact declaration**

```cpp
void getNextClear(Prim::FastFill &ff, Color bg = {{0, 0, 0}}) const
```

- **Declared at:** [line 174](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L174)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ff` | `Prim::FastFill &` | Input/output; inspect the function contract | The `FastFill` primitive to set. |
| `bg` | `Color` | Input | Value supplied for `bg`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will set the `FastFill` primitive passed as an argument in a way to completely clear the next drawing buffer with the specified color. This will be done in accordance to the next drawing buffer settings, after a flip. This is useful for clearing the buffer within a dma chain to be sent during the frame flip.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// Prim::FastFill & ff
// Color bg

psyqo::GPU& object = /* obtain a valid instance */;

object.getNextClear(ff, bg);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-getnextscissor-1"></a>

## `psyqo::GPU::getNextScissor`

**Purpose.** Gets the next scissoring region.

**Details.** This method will set the scissor primitive to the next active drawing buffer. This is useful for setting the scissor within a dma chain to be sent during the frame flip.

**Exact declaration**

```cpp
void getNextScissor(Prim::Scissor &scissor)
```

- **Declared at:** [line 272](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L272)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `scissor` | `Prim::Scissor &` | Input/output; inspect the function contract | The scissor primitive to set. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will set the scissor primitive to the next active drawing buffer. This is useful for setting the scissor within a dma chain to be sent during the frame flip.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// Prim::Scissor & scissor

psyqo::GPU& object = /* obtain a valid instance */;

object.getNextScissor(scissor);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-getparity-1"></a>

## `psyqo::GPU::getParity`

**Purpose.** Get the index of the current display buffer.

**Details.** This method will return the index of the current display buffer. The index will be either 0 or 1, and will be updated during the frame flip operation. This is useful for double buffering: when designing an application which uses double buffering, the application should keep two sets of data, one for each display buffer. The application should then use the `getParity` method to determine which if its two sets of data should be used for the current frame.

**Exact declaration**

```cpp
unsigned getParity() const
```

- **Declared at:** [line 140](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L140)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** unsigned The index of the current display buffer, either 0 or 1.

**Use it when.** This method will return the index of the current display buffer. The index will be either 0 or 1, and will be updated during the frame flip operation. This is useful for double buffering: when designing an application which uses double buffering, the application should keep two sets of data, one for each display buffer. The application should then use the `getParity` method to determine which if its two sets of data should be used for the current frame.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.getParity();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-getrefreshrate-1"></a>

## `psyqo::GPU::getRefreshRate`

**Purpose.** Returns the refresh rate of the GPU.

**Details.** This method will return either 60 or 50, depending on the current video mode.

**Exact declaration**

```cpp
unsigned getRefreshRate() const
```

- **Declared at:** [line 111](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L111)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `unsigned int`. Check the purpose and failure notes before using the value.

**Use it when.** This method will return either 60 or 50, depending on the current video mode.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.getRefreshRate();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-getscissor-1"></a>

## `psyqo::GPU::getScissor`

**Purpose.** Gets the current scissoring region.

**Details.** This method will set the scissor primitive to the currently active drawing buffer.

**Exact declaration**

```cpp
void getScissor(Prim::Scissor &scissor)
```

- **Declared at:** [line 262](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L262)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `scissor` | `Prim::Scissor &` | Input/output; inspect the function contract | The scissor primitive to set. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will set the scissor primitive to the currently active drawing buffer.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// Prim::Scissor & scissor

psyqo::GPU& object = /* obtain a valid instance */;

object.getScissor(scissor);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-initialize-1"></a>

## `psyqo::GPU::initialize`

**Purpose.** Performs `initialize` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
void initialize(const Configuration &config)
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L99)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `config` | `const Configuration &` | Input | Value supplied for `config`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// const Configuration & config

psyqo::GPU& object = /* obtain a valid instance */;

object.initialize(config);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-ischainidle-1"></a>

## `psyqo::GPU::isChainIdle`

**Purpose.** Gets the status of the background DMA transfer operation when initiated by a frame flip.

**Exact declaration**

```cpp
bool isChainIdle() const
```

- **Declared at:** [line 375](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L375)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** true if no background DMA transfer is in progress nor completed.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.isChainIdle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-ischaintransferred-1"></a>

## `psyqo::GPU::isChainTransferred`

**Purpose.** Gets the status of the background DMA transfer operation when initiated by a frame flip.

**Exact declaration**

```cpp
bool isChainTransferred() const
```

- **Declared at:** [line 389](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L389)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** true if a background DMA transfer has completed, and is now waiting for a frame flip.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.isChainTransferred();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-ischaintransferring-1"></a>

## `psyqo::GPU::isChainTransferring`

**Purpose.** Gets the status of the background DMA transfer operation when initiated by a frame flip.

**Exact declaration**

```cpp
bool isChainTransferring() const
```

- **Declared at:** [line 382](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L382)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** true if a background DMA transfer is in progress.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.isChainTransferring();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-now-1"></a>

## `psyqo::GPU::now`

**Purpose.** Gets the current timestamp in microseconds.

**Details.** The current timestamp is in microseconds. It will wrap around after a bit more than an hour, so it shouldn't be used for deadlines that are more than 30 minutes away. This relies on root counter 1 set in hsync mode without any target value. The value will be updated during the idle moments of the page flip, without relying on interrupts. The precision isn't really good, as it assumes one scanline runs at 64us, but it should be good enough for most purposes. Creating a stopwatch out of it should show that it's running a bit too fast, approximately 1 second too fast every minute or so. Its monotonicity should be proper however. The method `now()` should be reserved for interacting with timers. If longer span is required, with more accuracy but less precision, for something that's not related with timers, then the current amount of time in seconds since the application started can simply be obtained using `getFrameCount() / getRefreshRate()`.

**Exact declaration**

```cpp
uint32_t now() const
```

- **Declared at:** [line 416](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L416)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** The current timestamp in microseconds.

**Use it when.** The current timestamp is in microseconds. It will wrap around after a bit more than an hour, so it shouldn't be used for deadlines that are more than 30 minutes away. This relies on root counter 1 set in hsync mode without any target value. The value will be updated during the idle moments of the page flip, without relying on interrupts. The precision isn't really good, as it assumes one scanline runs at 64us, but it should be good enough for most purposes. Creating a stopwatch out of it should show that it's running a bit too fast, approximately 1 second too fast every minute or so. Its monotonicity should be proper however. The method `now()` should be reserved for interacting with timers. If longer span is required, with more accuracy but less precision, for something that's not related with timers, then the current amount of time in seconds since the application started can simply be obtained using `getFrameCount() / getRefreshRate()`.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

auto result = object.now();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-pausetimer-1"></a>

## `psyqo::GPU::pauseTimer`

**Purpose.** Pauses a timer.

**Details.** This method will pause a timer. It will not fire anymore, even if its deadline had already passed at the moment of this call, but it will remain active, and its id will remain valid. The remainder of the deadline will be remembered, for when the timer is resumed. This method has no effect if the timer is already paused.

**Exact declaration**

```cpp
void pauseTimer(uintptr_t id)
```

- **Declared at:** [line 488](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L488)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uintptr_t` | Input | The id of the timer to pause. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will pause a timer. It will not fire anymore, even if its deadline had already passed at the moment of this call, but it will remain active, and its id will remain valid. The remainder of the deadline will be remembered, for when the timer is resumed. This method has no effect if the timer is already paused.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uintptr_t id

psyqo::GPU& object = /* obtain a valid instance */;

object.pauseTimer(id);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-pumpcallbacks-1"></a>

## `psyqo::GPU::pumpCallbacks`

**Purpose.** Runs one round of event processing.

**Details.** While this method is technically for internal use, it is exposed here for convenience. It will run one round of event processing, including the processing of timers. This method should be called in a loop when waiting for other events to be processed.

**Exact declaration**

```cpp
void pumpCallbacks()
```

- **Declared at:** [line 518](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L518)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** While this method is technically for internal use, it is exposed here for convenience. It will run one round of event processing, including the processing of timers. This method should be called in a loop when waiting for other events to be processed.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.pumpCallbacks();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-reinitialize-1"></a>

## `psyqo::GPU::reinitialize`

**Purpose.** Performs `reinitialize` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
void reinitialize(const Configuration &config)
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L100)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `config` | `const Configuration &` | Input | Value supplied for `config`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// const Configuration & config

psyqo::GPU& object = /* obtain a valid instance */;

object.reinitialize(config);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-resumetimer-1"></a>

## `psyqo::GPU::resumeTimer`

**Purpose.** Resumes a paused timer.

**Details.** This method will resume a paused timer. The timer will be able to fire again, according to its original settings. The new deadline will be calculated from the remainder of the time left when it was paused. This method will have no effect if the timer is not paused.

**Exact declaration**

```cpp
void resumeTimer(uintptr_t id)
```

- **Declared at:** [line 499](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L499)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `uintptr_t` | Input | The id of the timer to resume. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will resume a paused timer. The timer will be able to fire again, according to its original settings. The new deadline will be calculated from the remainder of the time left when it was paused. This method will have no effect if the timer is not paused.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uintptr_t id

psyqo::GPU& object = /* obtain a valid instance */;

object.resumeTimer(id);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-sendchain-1"></a>

## `psyqo::GPU::sendChain`

**Purpose.** Immediately sends the current DMA chain

**Details.** This method will immediately send the current DMA chain to the GPU, and block until completion.

**Exact declaration**

```cpp
void sendChain()
```

- **Declared at:** [line 359](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L359)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will immediately send the current DMA chain to the GPU, and block until completion.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.sendChain();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-sendchain-2"></a>

## `psyqo::GPU::sendChain`

**Purpose.** Initiates the transfer of the current DMA chain.

**Details.** See the non-blocking variant of `uploadToVRAM` for more information about asynchronous transfers.

**Exact declaration**

```cpp
void sendChain(eastl::function<void()> &&callback, DMA::DmaCallback dmaCallback = DMA::FROM_MAIN_LOOP)
```

- **Declared at:** [line 368](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L368)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | The callback to call upon completion. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | `DMA::FROM_MAIN_LOOP` or `DMA::FROM_ISR`. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** See the non-blocking variant of `uploadToVRAM` for more information about asynchronous transfers.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::GPU& object = /* obtain a valid instance */;

object.sendChain(callback, dmaCallback);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-sendfragment-1"></a>

## `psyqo::GPU::sendFragment`

**Purpose.** Immediately sends a fragment to the GPU. This is a blocking operation. See the fragments.hh file for more information.

**Exact declaration**

```cpp
template <Fragment Frag> void sendFragment(const Frag &fragment)
```

- **Declared at:** [line 222](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L222)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fragment` | `const Frag &` | Input | The fragment to send to the GPU. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Frag

// Assume these named values have been initialized with valid data:
// const Frag & fragment

psyqo::GPU& object = /* obtain a valid instance */;

object.sendFragment<Frag>(fragment);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-sendfragment-2"></a>

## `psyqo::GPU::sendFragment`

**Purpose.** Sends a fragment to the GPU as a non-blocking call.

**Details.** See the non-blocking variant of `uploadToVRAM` for more information about asynchronous transfers.

**Exact declaration**

```cpp
template <Fragment Frag> void sendFragment(const Frag &fragment, eastl::function<void()> &&callback, DMA::DmaCallback dmaCallback = DMA::FROM_MAIN_LOOP)
```

- **Declared at:** [line 236](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L236)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fragment` | `const Frag &` | Input | The fragment to send to the GPU. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | The callback to call upon completion. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | `DMA::FROM_MAIN_LOOP` or `DMA::FROM_ISR`. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** See the non-blocking variant of `uploadToVRAM` for more information about asynchronous transfers.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Frag

// Assume these named values have been initialized with valid data:
// const Frag & fragment
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::GPU& object = /* obtain a valid instance */;

object.sendFragment<Frag>(fragment, callback, dmaCallback);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-sendprimitive-1"></a>

## `psyqo::GPU::sendPrimitive`

**Purpose.** Sends a primitive to the GPU. This is a blocking call.

**Details.** This method will immediately send the specified primitive to the GPU.

**Exact declaration**

```cpp
template <Primitive Prim> void sendPrimitive(const Prim &primitive)
```

- **Declared at:** [line 296](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L296)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `primitive` | `const Prim &` | Input | The primitive to send to the GPU. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will immediately send the specified primitive to the GPU.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Replace these template arguments with types or values accepted by the declaration:
// Prim

// Assume these named values have been initialized with valid data:
// const Prim & primitive

psyqo::GPU& object = /* obtain a valid instance */;

object.sendPrimitive<Prim>(primitive);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-sendraw-1"></a>

## `psyqo::GPU::sendRaw`

**Purpose.** Sends a raw 32 bits value to the Data register of the GPU.

**Exact declaration**

```cpp
static void sendRaw(uint32_t data)
```

- **Declared at:** [line 287](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L287)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `uint32_t` | Input | Value supplied for `data`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t data

psyqo::GPU::sendRaw(data);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-uploadtovram-1"></a>

## `psyqo::GPU::uploadToVRAM`

**Purpose.** Uploads a buffer to the VRAM as a blocking call.

**Details.** This method will immediately upload the specified set of pixels to the VRAM, at the specified location and size. The GPU cache will be flushed. It will block until completion of the upload.

**Exact declaration**

```cpp
void uploadToVRAM(const uint16_t *data, Rect region)
```

- **Declared at:** [line 187](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L187)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const uint16_t *` | Input | The pixels to upload. Must be a contiguous array of 16-bpp pixels, with the number of pixels being equal to the area specified by the `region` parameter. |
| `region` | `Rect` | Input | The region in VRAM to upload the pixels to. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will immediately upload the specified set of pixels to the VRAM, at the specified location and size. The GPU cache will be flushed. It will block until completion of the upload.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// const uint16_t * data
// Rect region

psyqo::GPU& object = /* obtain a valid instance */;

object.uploadToVRAM(data, region);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-uploadtovram-2"></a>

## `psyqo::GPU::uploadToVRAM`

**Purpose.** Uploads a buffer to the VRAM as a non-blocking call.

**Details.** This method will initiate an upload of the specified set of pixels to the VRAM, at the specified location and size. The GPU cache will be flushed. It will return immediately, and the upload will be performed in the background. Upon completion, the specified callback will be called. If `dmaCallback` is set to `FROM_ISR`, the callback will be called from the interrupt handler, and care must be taken to properly synchronize variable changes. Please use the EASTL's `atomic_signal_fence` function for this purpose. If `dmaCallback` is set to `FROM_MAIN_LOOP`, the callback will be called in the same execution context as the main loop, and it is therefore safe to access variables there. The callback will thus be called between calls to the current scene's `frame` method, or during `Kernel::pumpCallbacks()`. Note that during the upload, no GPU operation should be performed.

**Exact declaration**

```cpp
void uploadToVRAM(const uint16_t *data, Rect region, eastl::function<void()> &&callback, DMA::DmaCallback dmaCallback = DMA::FROM_MAIN_LOOP)
```

- **Declared at:** [line 212](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L212)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const uint16_t *` | Input | The pixels to upload. Must be a contiguous array of 16-bpp pixels, with the number of pixels being equal to the area specified by the `region` parameter. |
| `region` | `Rect` | Input | The region in VRAM to upload the pixels to. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | The callback to call upon completion. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | `DMA::FROM_MAIN_LOOP` or `DMA::FROM_ISR`. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will initiate an upload of the specified set of pixels to the VRAM, at the specified location and size. The GPU cache will be flushed. It will return immediately, and the upload will be performed in the background. Upon completion, the specified callback will be called. If `dmaCallback` is set to `FROM_ISR`, the callback will be called from the interrupt handler, and care must be taken to properly synchronize variable changes. Please use the EASTL's `atomic_signal_fence` function for this purpose. If `dmaCallback` is set to `FROM_MAIN_LOOP`, the callback will be called in the same execution context as the main loop, and it is therefore safe to access variables there. The callback will thus be called between calls to the current scene's `frame` method, or during `Kernel::pumpCallbacks()`. Note that during the upload, no GPU operation should be performed.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// const uint16_t * data
// Rect region
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::GPU& object = /* obtain a valid instance */;

object.uploadToVRAM(data, region, callback, dmaCallback);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gpu-waitchainidle-1"></a>

## `psyqo::GPU::waitChainIdle`

**Purpose.** Waits until the background DMA transfer operation initiated by a frame flip is complete.

**Exact declaration**

```cpp
void waitChainIdle()
```

- **Declared at:** [line 395](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L395)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.waitChainIdle();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-waitfifo-1"></a>

## `psyqo::GPU::waitFifo`

**Purpose.** Waits until the GPU's FIFO is ready to receive data.

**Exact declaration**

```cpp
void waitFifo()
```

- **Declared at:** [line 282](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L282)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.waitFifo();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gpu-waitready-1"></a>

## `psyqo::GPU::waitReady`

**Purpose.** Waits until the GPU is ready to send a command.

**Exact declaration**

```cpp
void waitReady()
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L277)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

psyqo::GPU& object = /* obtain a valid instance */;

object.waitReady();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-timer-literals-operator-ms-1"></a>

## `psyqo::timer_literals::operator""_ms`

**Purpose.** Performs `operator "" ms` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_ms(long double value)
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L66)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `long double` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// long double value

auto result = psyqo::timer_literals::operator""_ms(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-ms-2"></a>

## `psyqo::timer_literals::operator""_ms`

**Purpose.** Performs `operator "" ms` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_ms(unsigned long long int value)
```

- **Declared at:** [line 62](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `unsigned long long` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// unsigned long long value

auto result = psyqo::timer_literals::operator""_ms(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-ns-1"></a>

## `psyqo::timer_literals::operator""_ns`

**Purpose.** Performs `operator "" ns` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_ns(long double value)
```

- **Declared at:** [line 64](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `long double` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// long double value

auto result = psyqo::timer_literals::operator""_ns(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-ns-2"></a>

## `psyqo::timer_literals::operator""_ns`

**Purpose.** Literal operators for time units.

**Details.** These operators can be used to specify time units suitable for the GPU's `armTimer` and `armPeriodicTimer` methods. For example, `gpu().armPeriodicTimer(1_s, callback)` will create a timer that fires every second.

**Exact declaration**

```cpp
consteval uint32_t operator""_ns(unsigned long long int value)
```

- **Declared at:** [line 60](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `unsigned long long` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** These operators can be used to specify time units suitable for the GPU's `armTimer` and `armPeriodicTimer` methods. For example, `gpu().armPeriodicTimer(1_s, callback)` will create a timer that fires every second.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// unsigned long long value

auto result = psyqo::timer_literals::operator""_ns(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-s-1"></a>

## `psyqo::timer_literals::operator""_s`

**Purpose.** Performs `operator "" s` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_s(long double value)
```

- **Declared at:** [line 67](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L67)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `long double` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// long double value

auto result = psyqo::timer_literals::operator""_s(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-s-2"></a>

## `psyqo::timer_literals::operator""_s`

**Purpose.** Performs `operator "" s` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_s(unsigned long long int value)
```

- **Declared at:** [line 63](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L63)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `unsigned long long` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// unsigned long long value

auto result = psyqo::timer_literals::operator""_s(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-us-1"></a>

## `psyqo::timer_literals::operator""_us`

**Purpose.** Performs `operator "" us` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_us(long double value)
```

- **Declared at:** [line 65](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L65)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `long double` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// long double value

auto result = psyqo::timer_literals::operator""_us(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-timer-literals-operator-us-2"></a>

## `psyqo::timer_literals::operator""_us`

**Purpose.** Performs `operator "" us` as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
consteval uint32_t operator""_us(unsigned long long int value)
```

- **Declared at:** [line 61](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gpu.hh#L61)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `unsigned long long` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gpu.hh"

// Assume these named values have been initialized with valid data:
// unsigned long long value

auto result = psyqo::timer_literals::operator""_us(value);
```

**Why choose it.** It provides direct, allocation-conscious access to GPU setup, command submission and frame synchronization. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
