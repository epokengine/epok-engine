# PsyQo API: Spu

> **Header:** `"psyqo/spu.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh)

This module covers SPU RAM, voices, ADSR and sound transfer. It documents 7 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::SPU`, `psyqo::SPU::ChannelPlaybackConfig`

## Callable index

- [`psyqo::SPU::dmaWrite`](#psyqo-spu-dmawrite-1) — Synchronously uploads data to the SPU's sound RAM.
- [`psyqo::SPU::dmaWrite`](#psyqo-spu-dmawrite-2) — Asynchronously uploads data to the SPU's sound RAM.
- [`psyqo::SPU::getNextFreeChannel`](#psyqo-spu-getnextfreechannel-1) — Returns next free channel as part of SPU RAM, voices, ADSR and sound transfer.
- [`psyqo::SPU::initAsync`](#psyqo-spu-initasync-1) — Enables asynchronous DMA transfers for this SPU object.
- [`psyqo::SPU::initialize`](#psyqo-spu-initialize-1) — Resets the SPU to a known, silent state.
- [`psyqo::SPU::playADPCM`](#psyqo-spu-playadpcm-1) — Starts playing an ADPCM sample on the given channel.
- [`psyqo::SPU::silenceChannels`](#psyqo-spu-silencechannels-1) — Silences the given channels.

<a id="psyqo-spu-dmawrite-1"></a>

## `psyqo::SPU::dmaWrite`

**Purpose.** Synchronously uploads data to the SPU's sound RAM.

**Details.** Blocks until the transfer completes. All sizes are in bytes.

**Exact declaration**

```cpp
static void dmaWrite(uint32_t spuAddress, const void* ramAddress, size_t dataSize, size_t blockSize = 0)
```

- **Declared at:** [line 101](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L101)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `spuAddress` | `uint32_t` | Input | The destination address in sound RAM. Must be 8 byte aligned, and the transfer has to fit within `SOUND_RAM_SIZE`. |
| `ramAddress` | `const void *` | Input | The source address in main RAM. Must be 4 byte aligned. |
| `dataSize` | `size_t` | Input | The number of bytes to transfer. Must be a non-zero multiple of `ADPCM_BLOCK_SIZE`. |
| `blockSize` | `size_t` | Input | The DMA block size in bytes, or 0 to pick the largest one dividing `dataSize`. Must be a multiple of 4 and no larger than `MAX_DMA_BLOCK_SIZE`. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Blocks until the transfer completes. All sizes are in bytes.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t spuAddress
// const void * ramAddress
// size_t dataSize
// size_t blockSize

psyqo::SPU::dmaWrite(spuAddress, ramAddress, dataSize, blockSize);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-spu-dmawrite-2"></a>

## `psyqo::SPU::dmaWrite`

**Purpose.** Asynchronously uploads data to the SPU's sound RAM.

**Details.** Returns as soon as the transfer has started, and requires `initAsync` to have been called. Only one transfer may be in flight at a time, but `callback` is free to start the next one.

**Exact declaration**

```cpp
void dmaWrite(uint32_t spuAddress, const void* ramAddress, size_t dataSize, eastl::function<void()>&& callback, DMA::DmaCallback dmaCallback = DMA::FROM_MAIN_LOOP)
```

- **Declared at:** [line 120](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L120)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `spuAddress` | `uint32_t` | Input | The destination address in sound RAM. Must be 8 byte aligned, and the transfer has to fit within `SOUND_RAM_SIZE`. |
| `ramAddress` | `const void *` | Input | The source address in main RAM. Must be 4 byte aligned, and has to stay alive until the callback fires. |
| `dataSize` | `size_t` | Input | The number of bytes to transfer. Must be a non-zero multiple of `ADPCM_BLOCK_SIZE`. |
| `callback` | `eastl::function<void ()> &&` | Consumed or moved input | The function to call once the transfer has completed. |
| `dmaCallback` | `DMA::DmaCallback` | Callback | Whether to call it from the interrupt handler, or from the main loop. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Returns as soon as the transfer has started, and requires `initAsync` to have been called. Only one transfer may be in flight at a time, but `callback` is free to start the next one.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t spuAddress
// const void * ramAddress
// size_t dataSize
// eastl::function<void ()> && callback
// DMA::DmaCallback dmaCallback

psyqo::SPU& object = /* obtain a valid instance */;

object.dmaWrite(spuAddress, ramAddress, dataSize, callback, dmaCallback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-spu-getnextfreechannel-1"></a>

## `psyqo::SPU::getNextFreeChannel`

**Purpose.** Returns next free channel as part of SPU RAM, voices, ADSR and sound transfer.

**Exact declaration**

```cpp
static uint32_t getNextFreeChannel()
```

- **Declared at:** [line 139](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L139)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

auto result = psyqo::SPU::getNextFreeChannel();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-spu-initasync-1"></a>

## `psyqo::SPU::initAsync`

**Purpose.** Enables asynchronous DMA transfers for this SPU object.

**Details.** Registers the SPU's DMA completion handler and arms the corresponding interrupt, which is what lets the callback taking overload of `dmaWrite` ever complete. Call it once, during application start.

**Exact declaration**

```cpp
void initAsync()
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L78)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Registers the SPU's DMA completion handler and arms the corresponding interrupt, which is what lets the callback taking overload of `dmaWrite` ever complete. Call it once, during application start.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

psyqo::SPU& object = /* obtain a valid instance */;

object.initAsync();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-spu-initialize-1"></a>

## `psyqo::SPU::initialize`

**Purpose.** Resets the SPU to a known, silent state.

**Details.** The kernel calls this during application startup, so there is normally no reason to call it again.

**Exact declaration**

```cpp
static void initialize()
```

- **Declared at:** [line 69](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L69)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The kernel calls this during application startup, so there is normally no reason to call it again.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

psyqo::SPU::initialize();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-spu-playadpcm-1"></a>

## `psyqo::SPU::playADPCM`

**Purpose.** Starts playing an ADPCM sample on the given channel.

**Exact declaration**

```cpp
static void playADPCM(uint8_t channelId, uint32_t spuRamAddress, const ChannelPlaybackConfig& config, bool hardCut)
```

- **Declared at:** [line 138](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L138)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channelId` | `uint8_t` | Input | The channel to play on, 0 to 23. |
| `spuRamAddress` | `uint32_t` | Input | The address of the sample in sound RAM. Must be 8 byte aligned and within `SOUND_RAM_SIZE`. |
| `config` | `const ChannelPlaybackConfig &` | Input | The volume, sample rate and ADSR settings to use. |
| `hardCut` | `bool` | Input | Whether to key the channel off before keying it back on. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

// Assume these named values have been initialized with valid data:
// uint8_t channelId
// uint32_t spuRamAddress
// const ChannelPlaybackConfig & config
// bool hardCut

psyqo::SPU::playADPCM(channelId, spuRamAddress, config, hardCut);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-spu-silencechannels-1"></a>

## `psyqo::SPU::silenceChannels`

**Purpose.** Silences the given channels.

**Exact declaration**

```cpp
static void silenceChannels(uint32_t channelMask)
```

- **Declared at:** [line 85](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/spu.hh#L85)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `channelMask` | `uint32_t` | Input | A bitmask of the channels to silence. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SPU RAM, voices, ADSR and sound transfer and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/spu.hh"

// Assume these named values have been initialized with valid data:
// uint32_t channelMask

psyqo::SPU::silenceChannels(channelMask);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
