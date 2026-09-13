# PsyQo API: Cdrom Device

> **Header:** `"psyqo/cdrom-device.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh)

This module covers the cdrom device module. It documents 64 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::CDRomDevice`, `psyqo::CDRomDevice::Action`, `psyqo::CDRomDevice::GetPlaybackLocationAwaiter`, `psyqo::CDRomDevice::GetTOCSizeAwaiter`, `psyqo::CDRomDevice::MuteAwaiter`, `psyqo::CDRomDevice::PlaybackLocation`, `psyqo::CDRomDevice::ReadTOCAwaiter`, `psyqo::CDRomDevice::ResetAwaiter`, `psyqo::CDRomDevice::UnmuteAwaiter`, `psyqo::Concepts::CDRomDeviceStateEnumHasIdle`, `psyqo::GPU`

## Callable index

- [`psyqo::CDRomDevice::getPlaybackLocation`](#psyqo-cdromdevice-getplaybacklocation-1) — Returns playback location as part of the cdrom device module.
- [`psyqo::CDRomDevice::getPlaybackLocation`](#psyqo-cdromdevice-getplaybacklocation-2) — Get the Playback location of the CDDA audio.
- [`psyqo::CDRomDevice::getPlaybackLocation`](#psyqo-cdromdevice-getplaybacklocation-3) — Returns playback location as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_ready`](#psyqo-cdromdevice-getplaybacklocationawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_resume`](#psyqo-cdromdevice-getplaybacklocationawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_suspend`](#psyqo-cdromdevice-getplaybacklocationawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetPlaybackLocationAwaiter::GetPlaybackLocationAwaiter`](#psyqo-cdromdevice-getplaybacklocationawaiter-getplaybacklocationawaiter-1) — Constructs `psyqo::CDRomDevice::GetPlaybackLocationAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::getTOCSize`](#psyqo-cdromdevice-gettocsize-1) — Returns tocsize as part of the cdrom device module.
- [`psyqo::CDRomDevice::getTOCSize`](#psyqo-cdromdevice-gettocsize-2) — Gets the size of the Table of Contents from the CDRom. Note that while the blocking variant is available because it is a fairly short operation with the CDRom controller, it can still block the system for roughly 2ms, which is a long time in the context of a 33MHz CPU.
- [`psyqo::CDRomDevice::GetTOCSizeAwaiter::await_ready`](#psyqo-cdromdevice-gettocsizeawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetTOCSizeAwaiter::await_resume`](#psyqo-cdromdevice-gettocsizeawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetTOCSizeAwaiter::await_suspend`](#psyqo-cdromdevice-gettocsizeawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::GetTOCSizeAwaiter::GetTOCSizeAwaiter`](#psyqo-cdromdevice-gettocsizeawaiter-gettocsizeawaiter-1) — Constructs `psyqo::CDRomDevice::GetTOCSizeAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::getTOCSizeBlocking`](#psyqo-cdromdevice-gettocsizeblocking-1) — Returns tocsize blocking as part of the cdrom device module.
- [`psyqo::CDRomDevice::isIdle`](#psyqo-cdromdevice-isidle-1) — Checks if the CDROM device is in idle state.
- [`psyqo::CDRomDevice::mute`](#psyqo-cdromdevice-mute-1) — Performs `mute` as part of the cdrom device module.
- [`psyqo::CDRomDevice::mute`](#psyqo-cdromdevice-mute-2) — Mutes the CD audio for both CDDA and CDXA.
- [`psyqo::CDRomDevice::MuteAwaiter::await_ready`](#psyqo-cdromdevice-muteawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::MuteAwaiter::await_resume`](#psyqo-cdromdevice-muteawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::MuteAwaiter::await_suspend`](#psyqo-cdromdevice-muteawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::MuteAwaiter::MuteAwaiter`](#psyqo-cdromdevice-muteawaiter-muteawaiter-1) — Constructs `psyqo::CDRomDevice::MuteAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::muteBlocking`](#psyqo-cdromdevice-muteblocking-1) — Performs `mute blocking` as part of the cdrom device module.
- [`psyqo::CDRomDevice::pauseCDDA`](#psyqo-cdromdevice-pausecdda-1) — Pauses CDDA playback.
- [`psyqo::CDRomDevice::playCDDADisc`](#psyqo-cdromdevice-playcddadisc-1) — Starts cddadisc as part of the cdrom device module.
- [`psyqo::CDRomDevice::playCDDADisc`](#psyqo-cdromdevice-playcddadisc-2) — Starts cddadisc as part of the cdrom device module.
- [`psyqo::CDRomDevice::playCDDATrack`](#psyqo-cdromdevice-playcddatrack-1) — Begins playing CDDA audio from a given starting point.
- [`psyqo::CDRomDevice::playCDDATrack`](#psyqo-cdromdevice-playcddatrack-2) — Starts cddatrack as part of the cdrom device module.
- [`psyqo::CDRomDevice::prepare`](#psyqo-cdromdevice-prepare-1) — Prepares the CDRom subsystem.
- [`psyqo::CDRomDevice::readSectors`](#psyqo-cdromdevice-readsectors-1) — Reads sectors from the CDRom.
- [`psyqo::CDRomDevice::readSectorsBlocking`](#psyqo-cdromdevice-readsectorsblocking-1) — Reads sectors blocking as part of the cdrom device module.
- [`psyqo::CDRomDevice::readTOC`](#psyqo-cdromdevice-readtoc-1) — Reads toc as part of the cdrom device module.
- [`psyqo::CDRomDevice::readTOC`](#psyqo-cdromdevice-readtoc-2) — Reads the Table of Contents from the CDRom.
- [`psyqo::CDRomDevice::ReadTOCAwaiter::await_ready`](#psyqo-cdromdevice-readtocawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ReadTOCAwaiter::await_resume`](#psyqo-cdromdevice-readtocawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ReadTOCAwaiter::await_suspend`](#psyqo-cdromdevice-readtocawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ReadTOCAwaiter::ReadTOCAwaiter`](#psyqo-cdromdevice-readtocawaiter-readtocawaiter-1) — Constructs `psyqo::CDRomDevice::ReadTOCAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::readTOCBlocking`](#psyqo-cdromdevice-readtocblocking-1) — Reads tocblocking as part of the cdrom device module.
- [`psyqo::CDRomDevice::reset`](#psyqo-cdromdevice-reset-1) — Resets reset as part of the cdrom device module.
- [`psyqo::CDRomDevice::reset`](#psyqo-cdromdevice-reset-2) — Resets the CDRom controller.
- [`psyqo::CDRomDevice::ResetAwaiter::await_ready`](#psyqo-cdromdevice-resetawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ResetAwaiter::await_resume`](#psyqo-cdromdevice-resetawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ResetAwaiter::await_suspend`](#psyqo-cdromdevice-resetawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::ResetAwaiter::ResetAwaiter`](#psyqo-cdromdevice-resetawaiter-resetawaiter-1) — Constructs `psyqo::CDRomDevice::ResetAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::resetBlocking`](#psyqo-cdromdevice-resetblocking-1) — Resets blocking as part of the cdrom device module.
- [`psyqo::CDRomDevice::resumeCDDA`](#psyqo-cdromdevice-resumecdda-1) — Resumes cdda as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleGetPlaybackLocation`](#psyqo-cdromdevice-schedulegetplaybacklocation-1) — Performs `schedule get playback location` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleGetTOCSize`](#psyqo-cdromdevice-schedulegettocsize-1) — Performs `schedule get tocsize` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleMute`](#psyqo-cdromdevice-schedulemute-1) — Performs `schedule mute` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleReadTOC`](#psyqo-cdromdevice-schedulereadtoc-1) — Performs `schedule read toc` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleReset`](#psyqo-cdromdevice-schedulereset-1) — Performs `schedule reset` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleTest`](#psyqo-cdromdevice-scheduletest-1) — Performs `schedule test` as part of the cdrom device module.
- [`psyqo::CDRomDevice::scheduleUnmute`](#psyqo-cdromdevice-scheduleunmute-1) — Performs `schedule unmute` as part of the cdrom device module.
- [`psyqo::CDRomDevice::setVolume`](#psyqo-cdromdevice-setvolume-1) — Set the Volume of the CDDA audio.
- [`psyqo::CDRomDevice::stopCDDA`](#psyqo-cdromdevice-stopcdda-1) — Stops CDDA playback.
- [`psyqo::CDRomDevice::test`](#psyqo-cdromdevice-test-1) — Sends a test command to the CDRom mech
- [`psyqo::CDRomDevice::testBlocking`](#psyqo-cdromdevice-testblocking-1) — Performs `test blocking` as part of the cdrom device module.
- [`psyqo::CDRomDevice::unmute`](#psyqo-cdromdevice-unmute-1) — Performs `unmute` as part of the cdrom device module.
- [`psyqo::CDRomDevice::unmute`](#psyqo-cdromdevice-unmute-2) — Unmutes the CD audio for both CDDA and CDXA.
- [`psyqo::CDRomDevice::UnmuteAwaiter::await_ready`](#psyqo-cdromdevice-unmuteawaiter-await-ready-1) — Performs `await ready` as part of the cdrom device module.
- [`psyqo::CDRomDevice::UnmuteAwaiter::await_resume`](#psyqo-cdromdevice-unmuteawaiter-await-resume-1) — Performs `await resume` as part of the cdrom device module.
- [`psyqo::CDRomDevice::UnmuteAwaiter::await_suspend`](#psyqo-cdromdevice-unmuteawaiter-await-suspend-1) — Performs `await suspend` as part of the cdrom device module.
- [`psyqo::CDRomDevice::UnmuteAwaiter::UnmuteAwaiter`](#psyqo-cdromdevice-unmuteawaiter-unmuteawaiter-1) — Constructs `psyqo::CDRomDevice::UnmuteAwaiter` for the cdrom device module.
- [`psyqo::CDRomDevice::unmuteBlocking`](#psyqo-cdromdevice-unmuteblocking-1) — Performs `unmute blocking` as part of the cdrom device module.
- [`psyqo::CDRomDevice::~CDRomDevice`](#psyqo-cdromdevice-cdromdevice-1) — Releases the resources owned by `psyqo::CDRomDevice`.

<a id="psyqo-cdromdevice-getplaybacklocation-1"></a>

## `psyqo::CDRomDevice::getPlaybackLocation`

**Purpose.** Returns playback location as part of the cdrom device module.

**Exact declaration**

```cpp
GetPlaybackLocationAwaiter getPlaybackLocation()
```

- **Declared at:** [line 392](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L392)
- **Kind:** `cxx method`

**Returns.** Returns `GetPlaybackLocationAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.getPlaybackLocation();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-getplaybacklocation-2"></a>

## `psyqo::CDRomDevice::getPlaybackLocation`

**Purpose.** Get the Playback location of the CDDA audio.

**Details.** This method will request the current playback location of the CDDA audio. The callback will be called with a pointer to a `PlaybackLocation` structure, which will contain the relative and absolute MSF values, the current track number, and the current index. The callback will be called with a null pointer if the location could not be retrieved.

**Exact declaration**

```cpp
void getPlaybackLocation(PlaybackLocation *location, eastl::function<void(PlaybackLocation *)> &&callback)
```

- **Declared at:** [line 389](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L389)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `location` | `PlaybackLocation *` | Input/output; inspect the function contract | If provided, the location will be stored here. |
| `callback` | `eastl::function<void (PlaybackLocation *)> &&` | Consumed or moved input | The callback to call when the location is retrieved. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will request the current playback location of the CDDA audio. The callback will be called with a pointer to a `PlaybackLocation` structure, which will contain the relative and absolute MSF values, the current track number, and the current index. The callback will be called with a null pointer if the location could not be retrieved.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// PlaybackLocation * location
// eastl::function<void (PlaybackLocation *)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.getPlaybackLocation(location, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-getplaybacklocation-3"></a>

## `psyqo::CDRomDevice::getPlaybackLocation`

**Purpose.** Returns playback location as part of the cdrom device module.

**Exact declaration**

```cpp
void getPlaybackLocation(eastl::function<void(PlaybackLocation *)> &&callback)
```

- **Declared at:** [line 390](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L390)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (PlaybackLocation *)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (PlaybackLocation *)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.getPlaybackLocation(callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-getplaybacklocationawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 180](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L180)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::GetPlaybackLocationAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-getplaybacklocationawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
PlaybackLocation *await_resume()
```

- **Declared at:** [line 188](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L188)
- **Kind:** `cxx method`

**Returns.** Returns `PlaybackLocation *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::GetPlaybackLocationAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-getplaybacklocationawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::GetPlaybackLocationAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 182](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L182)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::GetPlaybackLocationAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-getplaybacklocationawaiter-getplaybacklocationawaiter-1"></a>

## `psyqo::CDRomDevice::GetPlaybackLocationAwaiter::GetPlaybackLocationAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::GetPlaybackLocationAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
GetPlaybackLocationAwaiter(CDRomDevice &device) : m_dev
```

- **Declared at:** [line 179](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L179)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device

psyqo::CDRomDevice::GetPlaybackLocationAwaiter value(device);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-gettocsize-1"></a>

## `psyqo::CDRomDevice::getTOCSize`

**Purpose.** Returns tocsize as part of the cdrom device module.

**Exact declaration**

```cpp
GetTOCSizeAwaiter getTOCSize()
```

- **Declared at:** [line 279](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L279)
- **Kind:** `cxx method`

**Returns.** Returns `GetTOCSizeAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.getTOCSize();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-gettocsize-2"></a>

## `psyqo::CDRomDevice::getTOCSize`

**Purpose.** Gets the size of the Table of Contents from the CDRom. Note that while the blocking variant is available because it is a fairly short operation with the CDRom controller, it can still block the system for roughly 2ms, which is a long time in the context of a 33MHz CPU.

**Exact declaration**

```cpp
void getTOCSize(unsigned *size, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 276](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L276)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `size` | `unsigned int *` | Input/output; inspect the function contract | The pointer to store the size of the TOC. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the size is retrieved. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// unsigned int * size
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.getTOCSize(size, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-gettocsizeawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::GetTOCSizeAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L109)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::GetTOCSizeAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-gettocsizeawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::GetTOCSizeAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
unsigned await_resume()
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L117)
- **Kind:** `cxx method`

**Returns.** Returns `unsigned int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::GetTOCSizeAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-gettocsizeawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::GetTOCSizeAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 111](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L111)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::GetTOCSizeAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-gettocsizeawaiter-gettocsizeawaiter-1"></a>

## `psyqo::CDRomDevice::GetTOCSizeAwaiter::GetTOCSizeAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::GetTOCSizeAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
GetTOCSizeAwaiter(CDRomDevice &device) : m_dev
```

- **Declared at:** [line 108](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L108)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device

psyqo::CDRomDevice::GetTOCSizeAwaiter value(device);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-gettocsizeblocking-1"></a>

## `psyqo::CDRomDevice::getTOCSizeBlocking`

**Purpose.** Returns tocsize blocking as part of the cdrom device module.

**Exact declaration**

```cpp
unsigned getTOCSizeBlocking(GPU &)
```

- **Declared at:** [line 278](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L278)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `unsigned int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// GPU & arg1

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.getTOCSizeBlocking(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-isidle-1"></a>

## `psyqo::CDRomDevice::isIdle`

**Purpose.** Checks if the CDROM device is in idle state.

**Exact declaration**

```cpp
bool isIdle() const
```

- **Declared at:** [line 452](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L452)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.isIdle();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-mute-1"></a>

## `psyqo::CDRomDevice::mute`

**Purpose.** Performs `mute` as part of the cdrom device module.

**Exact declaration**

```cpp
MuteAwaiter mute()
```

- **Declared at:** [line 311](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L311)
- **Kind:** `cxx method`

**Returns.** Returns `MuteAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.mute();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-mute-2"></a>

## `psyqo::CDRomDevice::mute`

**Purpose.** Mutes the CD audio for both CDDA and CDXA.

**Exact declaration**

```cpp
void mute(eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 308](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L308)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the mute operation is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.mute(callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-muteawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::MuteAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 146](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L146)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::MuteAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-muteawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::MuteAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 154](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L154)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::MuteAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-muteawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::MuteAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 148](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L148)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::MuteAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-muteawaiter-muteawaiter-1"></a>

## `psyqo::CDRomDevice::MuteAwaiter::MuteAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::MuteAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
MuteAwaiter(CDRomDevice &device) : m_dev
```

- **Declared at:** [line 145](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L145)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device

psyqo::CDRomDevice::MuteAwaiter value(device);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-muteblocking-1"></a>

## `psyqo::CDRomDevice::muteBlocking`

**Purpose.** Performs `mute blocking` as part of the cdrom device module.

**Exact declaration**

```cpp
void muteBlocking(GPU &)
```

- **Declared at:** [line 310](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L310)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// GPU & arg1

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.muteBlocking(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-pausecdda-1"></a>

## `psyqo::CDRomDevice::pauseCDDA`

**Purpose.** Pauses CDDA playback.

**Details.** This method will request a pause of the CDDA playback. The callback which was provided to the `playCDDA` method will be called with `true` when the playback is paused successfully. This method can only be called when the CDDA playback is in progress, as indicated by a first successful call of the `playCDDA` callback, and will fail if not. Pausing the playback will not stop the CDRom drive motor, which means that another `playCDDA` call start playing faster than if the motor was stopped.

**Exact declaration**

```cpp
void pauseCDDA()
```

- **Declared at:** [line 364](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L364)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will request a pause of the CDDA playback. The callback which was provided to the `playCDDA` method will be called with `true` when the playback is paused successfully. This method can only be called when the CDDA playback is in progress, as indicated by a first successful call of the `playCDDA` callback, and will fail if not. Pausing the playback will not stop the CDRom drive motor, which means that another `playCDDA` call start playing faster than if the motor was stopped.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.pauseCDDA();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-playcddadisc-1"></a>

## `psyqo::CDRomDevice::playCDDADisc`

**Purpose.** Starts cddadisc as part of the cdrom device module.

**Exact declaration**

```cpp
void playCDDADisc(MSF start, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 348](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L348)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `start` | `MSF` | Input | Value supplied for `start`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF start
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.playCDDADisc(start, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-playcddadisc-2"></a>

## `psyqo::CDRomDevice::playCDDADisc`

**Purpose.** Starts cddadisc as part of the cdrom device module.

**Exact declaration**

```cpp
void playCDDADisc(unsigned track, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 349](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L349)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `track` | `unsigned int` | Input | Value supplied for `track`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// unsigned int track
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.playCDDADisc(track, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-playcddatrack-1"></a>

## `psyqo::CDRomDevice::playCDDATrack`

**Purpose.** Begins playing CDDA audio from a given starting point.

**Details.** This method will begin playing CDDA audio from a given starting point. The starting point is either a track number or an MSF value. Unlike other APIs here, upon success, the callback will be called *twice*: once when the playback actually started, and once when the playback is complete or paused, which can be after the end of the track, at the end of the disc if the last track is reached, or if the playback is paused or stopped using the `pauseCDDA` or `stopCDDA` methods. The first callback will be called with `true` if the playback started successfully, and `false` if it failed. In the case of failure, the second callback will not be called. The Track variant will stop playback at the end of the track, while the Disc variant will stop playback at the end of the disc. The resume method can be used to resume playback after a pause. Its callback argument will function similarly to the callback argument of the `playCDDA` methods.

**Exact declaration**

```cpp
void playCDDATrack(MSF start, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 346](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L346)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `start` | `MSF` | Input | The starting point for playback. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when playback is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will begin playing CDDA audio from a given starting point. The starting point is either a track number or an MSF value. Unlike other APIs here, upon success, the callback will be called *twice*: once when the playback actually started, and once when the playback is complete or paused, which can be after the end of the track, at the end of the disc if the last track is reached, or if the playback is paused or stopped using the `pauseCDDA` or `stopCDDA` methods. The first callback will be called with `true` if the playback started successfully, and `false` if it failed. In the case of failure, the second callback will not be called. The Track variant will stop playback at the end of the track, while the Disc variant will stop playback at the end of the disc. The resume method can be used to resume playback after a pause. Its callback argument will function similarly to the callback argument of the `playCDDA` methods.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF start
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.playCDDATrack(start, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-playcddatrack-2"></a>

## `psyqo::CDRomDevice::playCDDATrack`

**Purpose.** Starts cddatrack as part of the cdrom device module.

**Exact declaration**

```cpp
void playCDDATrack(unsigned track, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 347](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L347)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `track` | `unsigned int` | Input | Value supplied for `track`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// unsigned int track
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.playCDDATrack(track, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-prepare-1"></a>

## `psyqo::CDRomDevice::prepare`

**Purpose.** Prepares the CDRom subsystem.

**Details.** This method prepares the kernel and the system for the CDRom subsystem. It should be called once from the application's `prepare` method.

**Exact declaration**

```cpp
void prepare()
```

- **Declared at:** [line 232](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L232)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method prepares the kernel and the system for the CDRom subsystem. It should be called once from the application's `prepare` method.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.prepare();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-readsectors-1"></a>

## `psyqo::CDRomDevice::readSectors`

**Purpose.** Reads sectors from the CDRom.

**Details.** This method will read a number of sectors from the CDRom drive. The sectors will be read into the provided buffer. Note that only one read operation can be active at a time, and that the `ISO9660Parser` class will call this method to read the filesystem structure, so care must be taken to ensure no other read operation is active when the parser is used.

**Exact declaration**

```cpp
void readSectors(uint32_t sector, uint32_t count, void *buffer, eastl::function<void(bool)> &&callback) override
```

- **Declared at:** [line 264](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L264)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | The sector to start reading from. |
| `count` | `uint32_t` | Input | The number of sectors to read. |
| `buffer` | `void *` | Input/output; inspect the function contract | The buffer to read the sectors into. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the read is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will read a number of sectors from the CDRom drive. The sectors will be read into the provided buffer. Note that only one read operation can be active at a time, and that the `ISO9660Parser` class will call this method to read the filesystem structure, so care must be taken to ensure no other read operation is active when the parser is used.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.readSectors(sector, count, buffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-readsectorsblocking-1"></a>

## `psyqo::CDRomDevice::readSectorsBlocking`

**Purpose.** Reads sectors blocking as part of the cdrom device module.

**Exact declaration**

```cpp
bool readSectorsBlocking(uint32_t sector, uint32_t count, void *buffer, GPU &)
```

- **Declared at:** [line 265](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L265)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `count` | `uint32_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `buffer` | `void *` | Input/output; inspect the function contract | Value supplied for `buffer`. See the exact type and module contract. |
| `arg4` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg4`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// uint32_t count
// void * buffer
// GPU & arg4

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.readSectorsBlocking(sector, count, buffer, arg4);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-readtoc-1"></a>

## `psyqo::CDRomDevice::readTOC`

**Purpose.** Reads toc as part of the cdrom device module.

**Exact declaration**

```cpp
ReadTOCAwaiter readTOC(MSF *toc, unsigned size)
```

- **Declared at:** [line 301](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L301)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `toc` | `MSF *` | Input/output; inspect the function contract | Value supplied for `toc`. See the exact type and module contract. |
| `size` | `unsigned int` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `ReadTOCAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF * toc
// unsigned int size

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.readTOC(toc, size);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-readtoc-2"></a>

## `psyqo::CDRomDevice::readTOC`

**Purpose.** Reads the Table of Contents from the CDRom.

**Details.** This method will read the Table of Contents from the CDRom drive. The TOC will be read into the provided buffer. Note that a CD-Rom can have up to 99 tracks, and the TOC will be read into the provided buffer starting at index 1 for the first track. Any tracks that are not present on the CD will not have their MSF structure filled in, so the application should ensure that the buffer is initialized to zero before calling this method. The blocking variant may take a total of 200ms to complete, depending on the number of tracks on the CD.

**Exact declaration**

```cpp
void readTOC(MSF *toc, unsigned size, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 298](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L298)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `toc` | `MSF *` | Input/output; inspect the function contract | The buffer to read the TOC into. |
| `size` | `unsigned int` | Input | The size of the buffer. Should be 100 to hold all possible tracks. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the read is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will read the Table of Contents from the CDRom drive. The TOC will be read into the provided buffer. Note that a CD-Rom can have up to 99 tracks, and the TOC will be read into the provided buffer starting at index 1 for the first track. Any tracks that are not present on the CD will not have their MSF structure filled in, so the application should ensure that the buffer is initialized to zero before calling this method. The blocking variant may take a total of 200ms to complete, depending on the number of tracks on the CD.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF * toc
// unsigned int size
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.readTOC(toc, size, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-readtocawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::ReadTOCAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 127](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L127)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::ReadTOCAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-readtocawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::ReadTOCAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 135](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L135)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::ReadTOCAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-readtocawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::ReadTOCAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 129](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L129)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::ReadTOCAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-readtocawaiter-readtocawaiter-1"></a>

## `psyqo::CDRomDevice::ReadTOCAwaiter::ReadTOCAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::ReadTOCAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
ReadTOCAwaiter(CDRomDevice &device, MSF *toc, unsigned size) : m_dev
```

- **Declared at:** [line 126](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L126)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |
| `toc` | `MSF *` | Input/output; inspect the function contract | Value supplied for `toc`. See the exact type and module contract. |
| `size` | `unsigned int` | Input | Value supplied for `size`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device
// MSF * toc
// unsigned int size

psyqo::CDRomDevice::ReadTOCAwaiter value(device, toc, size);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-readtocblocking-1"></a>

## `psyqo::CDRomDevice::readTOCBlocking`

**Purpose.** Reads tocblocking as part of the cdrom device module.

**Exact declaration**

```cpp
bool readTOCBlocking(MSF *toc, unsigned size, GPU &)
```

- **Declared at:** [line 300](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L300)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `toc` | `MSF *` | Input/output; inspect the function contract | Value supplied for `toc`. See the exact type and module contract. |
| `size` | `unsigned int` | Input | Value supplied for `size`. See the exact type and module contract. |
| `arg3` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg3`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF * toc
// unsigned int size
// GPU & arg3

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.readTOCBlocking(toc, size, arg3);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-reset-1"></a>

## `psyqo::CDRomDevice::reset`

**Purpose.** Resets reset as part of the cdrom device module.

**Exact declaration**

```cpp
ResetAwaiter reset()
```

- **Declared at:** [line 246](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L246)
- **Kind:** `cxx method`

**Returns.** Returns `ResetAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.reset();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-reset-2"></a>

## `psyqo::CDRomDevice::reset`

**Purpose.** Resets the CDRom controller.

**Details.** This method will reset the CDRom controller. It technically does not need to be called, but it is a good idea to call it when the application starts, in order to ensure that the controller is in a known state.

**Exact declaration**

```cpp
void reset(eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 243](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L243)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will reset the CDRom controller. It technically does not need to be called, but it is a good idea to call it when the application starts, in order to ensure that the controller is in a known state.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.reset(callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-resetawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::ResetAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 92](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::ResetAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-resetawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::ResetAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L100)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::ResetAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-resetawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::ResetAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 94](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L94)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::ResetAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-resetawaiter-resetawaiter-1"></a>

## `psyqo::CDRomDevice::ResetAwaiter::ResetAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::ResetAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
ResetAwaiter(CDRomDevice &device) : m_dev
```

- **Declared at:** [line 91](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L91)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device

psyqo::CDRomDevice::ResetAwaiter value(device);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-resetblocking-1"></a>

## `psyqo::CDRomDevice::resetBlocking`

**Purpose.** Resets blocking as part of the cdrom device module.

**Exact declaration**

```cpp
bool resetBlocking(GPU &)
```

- **Declared at:** [line 245](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L245)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// GPU & arg1

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.resetBlocking(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-resumecdda-1"></a>

## `psyqo::CDRomDevice::resumeCDDA`

**Purpose.** Resumes cdda as part of the cdrom device module.

**Exact declaration**

```cpp
void resumeCDDA(eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 350](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L350)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.resumeCDDA(callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-schedulegetplaybacklocation-1"></a>

## `psyqo::CDRomDevice::scheduleGetPlaybackLocation`

**Purpose.** Performs `schedule get playback location` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleGetPlaybackLocation(PlaybackLocation *location)
```

- **Declared at:** [line 391](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L391)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `location` | `PlaybackLocation *` | Input/output; inspect the function contract | Value supplied for `location`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// PlaybackLocation * location

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleGetPlaybackLocation(location);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-schedulegettocsize-1"></a>

## `psyqo::CDRomDevice::scheduleGetTOCSize`

**Purpose.** Performs `schedule get tocsize` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleGetTOCSize(unsigned *size)
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L277)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `size` | `unsigned int *` | Input/output; inspect the function contract | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// unsigned int * size

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleGetTOCSize(size);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-schedulemute-1"></a>

## `psyqo::CDRomDevice::scheduleMute`

**Purpose.** Performs `schedule mute` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleMute()
```

- **Declared at:** [line 309](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L309)
- **Kind:** `cxx method`

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleMute();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-schedulereadtoc-1"></a>

## `psyqo::CDRomDevice::scheduleReadTOC`

**Purpose.** Performs `schedule read toc` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleReadTOC(MSF *toc, unsigned size)
```

- **Declared at:** [line 299](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L299)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `toc` | `MSF *` | Input/output; inspect the function contract | Value supplied for `toc`. See the exact type and module contract. |
| `size` | `unsigned int` | Input | Value supplied for `size`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// MSF * toc
// unsigned int size

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleReadTOC(toc, size);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-schedulereset-1"></a>

## `psyqo::CDRomDevice::scheduleReset`

**Purpose.** Performs `schedule reset` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleReset()
```

- **Declared at:** [line 244](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L244)
- **Kind:** `cxx method`

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleReset();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-scheduletest-1"></a>

## `psyqo::CDRomDevice::scheduleTest`

**Purpose.** Performs `schedule test` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleTest(const psyqo::Hardware::CDRom::CDRomCommandBuffer &commandBuffer)
```

- **Declared at:** [line 422](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L422)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `commandBuffer` | `const psyqo::Hardware::CDRom::CDRomCommandBuffer &` | Input | Value supplied for `commandBuffer`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// const psyqo::Hardware::CDRom::CDRomCommandBuffer & commandBuffer

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleTest(commandBuffer);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-scheduleunmute-1"></a>

## `psyqo::CDRomDevice::scheduleUnmute`

**Purpose.** Performs `schedule unmute` as part of the cdrom device module.

**Exact declaration**

```cpp
TaskQueue::Task scheduleUnmute()
```

- **Declared at:** [line 319](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L319)
- **Kind:** `cxx method`

**Returns.** Returns `TaskQueue::Task`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.scheduleUnmute();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-setvolume-1"></a>

## `psyqo::CDRomDevice::setVolume`

**Purpose.** Set the Volume of the CDDA audio.

**Details.** This method will set the volume of the CDDA audio. The volume is set using four values, which represent the volume of the left channel to the left speaker, the right channel to the left speaker, the left channel to the right speaker, and the right channel to the right speaker. The given output value should be in the range of 0 to 128, where 0 is silence and 128 is full volume. The values for a given output speaker will be added together, so clipping can occur if the sum of the values is greater than 128. The method can be used at any time, unlike the mute/unmute methods, which can only be used when the drive is idle. The normal volume setting is 0x80, 0x00, 0x00, 0x80.

**Exact declaration**

```cpp
void setVolume(uint8_t leftToLeft, uint8_t rightToLeft, uint8_t leftToRight, uint8_t rightToRight)
```

- **Declared at:** [line 414](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L414)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `leftToLeft` | `uint8_t` | Input | The volume of the left channel to the left speaker. |
| `rightToLeft` | `uint8_t` | Input | The volume of the right channel to the left speaker. |
| `leftToRight` | `uint8_t` | Input | The volume of the left channel to the right speaker. |
| `rightToRight` | `uint8_t` | Input | The volume of the right channel to the right speaker. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will set the volume of the CDDA audio. The volume is set using four values, which represent the volume of the left channel to the left speaker, the right channel to the left speaker, the left channel to the right speaker, and the right channel to the right speaker. The given output value should be in the range of 0 to 128, where 0 is silence and 128 is full volume. The values for a given output speaker will be added together, so clipping can occur if the sum of the values is greater than 128. The method can be used at any time, unlike the mute/unmute methods, which can only be used when the drive is idle. The normal volume setting is 0x80, 0x00, 0x00, 0x80.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// uint8_t leftToLeft
// uint8_t rightToLeft
// uint8_t leftToRight
// uint8_t rightToRight

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.setVolume(leftToLeft, rightToLeft, leftToRight, rightToRight);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-stopcdda-1"></a>

## `psyqo::CDRomDevice::stopCDDA`

**Purpose.** Stops CDDA playback.

**Details.** This method will request a stop of the CDDA playback. It functions similarly to the `pauseCDDA` method, but will stop the CDRom drive motor, which means that another `playCDDA` call will take longer to start playing than if the motor was not stopped.

**Exact declaration**

```cpp
void stopCDDA()
```

- **Declared at:** [line 374](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L374)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will request a stop of the CDDA playback. It functions similarly to the `pauseCDDA` method, but will stop the CDRom drive motor, which means that another `playCDDA` call will take longer to start playing than if the motor was not stopped.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.stopCDDA();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-test-1"></a>

## `psyqo::CDRomDevice::test`

**Purpose.** Sends a test command to the CDRom mech

**Exact declaration**

```cpp
void test(const psyqo::Hardware::CDRom::CDRomCommandBuffer &commandBuffer, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 421](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L421)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `commandBuffer` | `const psyqo::Hardware::CDRom::CDRomCommandBuffer &` | Input | Value supplied for `commandBuffer`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the command operation is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// const psyqo::Hardware::CDRom::CDRomCommandBuffer & commandBuffer
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.test(commandBuffer, callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-testblocking-1"></a>

## `psyqo::CDRomDevice::testBlocking`

**Purpose.** Performs `test blocking` as part of the cdrom device module.

**Exact declaration**

```cpp
void testBlocking(GPU &, const psyqo::Hardware::CDRom::CDRomCommandBuffer &commandBuffer)
```

- **Declared at:** [line 423](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L423)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |
| `commandBuffer` | `const psyqo::Hardware::CDRom::CDRomCommandBuffer &` | Input | Value supplied for `commandBuffer`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// GPU & arg1
// const psyqo::Hardware::CDRom::CDRomCommandBuffer & commandBuffer

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.testBlocking(arg1, commandBuffer);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-unmute-1"></a>

## `psyqo::CDRomDevice::unmute`

**Purpose.** Performs `unmute` as part of the cdrom device module.

**Exact declaration**

```cpp
UnmuteAwaiter unmute()
```

- **Declared at:** [line 321](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L321)
- **Kind:** `cxx method`

**Returns.** Returns `UnmuteAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice& object = /* obtain a valid instance */;

auto result = object.unmute();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-unmute-2"></a>

## `psyqo::CDRomDevice::unmute`

**Purpose.** Unmutes the CD audio for both CDDA and CDXA.

**Exact declaration**

```cpp
void unmute(eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 318](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L318)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | The callback to call when the unmute operation is complete. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (bool)> && callback

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.unmute(callback);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-unmuteawaiter-await-ready-1"></a>

## `psyqo::CDRomDevice::UnmuteAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_ready() const
```

- **Declared at:** [line 163](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L163)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::UnmuteAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-unmuteawaiter-await-resume-1"></a>

## `psyqo::CDRomDevice::UnmuteAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the cdrom device module.

**Exact declaration**

```cpp
bool await_resume()
```

- **Declared at:** [line 171](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L171)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

psyqo::CDRomDevice::UnmuteAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-unmuteawaiter-await-suspend-1"></a>

## `psyqo::CDRomDevice::UnmuteAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the cdrom device module.

**Exact declaration**

```cpp
template <typename U> void await_suspend(std::coroutine_handle<U> handle)
```

- **Declared at:** [line 165](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L165)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `std::coroutine_handle<U>` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Replace these template arguments with types or values accepted by the declaration:
// U

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<U> handle

psyqo::CDRomDevice::UnmuteAwaiter& object = /* obtain a valid instance */;

object.await_suspend<U>(handle);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="psyqo-cdromdevice-unmuteawaiter-unmuteawaiter-1"></a>

## `psyqo::CDRomDevice::UnmuteAwaiter::UnmuteAwaiter`

**Purpose.** Constructs `psyqo::CDRomDevice::UnmuteAwaiter` for the cdrom device module.

**Exact declaration**

```cpp
UnmuteAwaiter(CDRomDevice &device) : m_dev
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L162)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `device` | `CDRomDevice &` | Input/output; inspect the function contract | Value supplied for `device`. See the exact type and module contract. |

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// CDRomDevice & device

psyqo::CDRomDevice::UnmuteAwaiter value(device);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-unmuteblocking-1"></a>

## `psyqo::CDRomDevice::unmuteBlocking`

**Purpose.** Performs `unmute blocking` as part of the cdrom device module.

**Exact declaration**

```cpp
void unmuteBlocking(GPU &)
```

- **Declared at:** [line 320](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L320)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// Assume these named values have been initialized with valid data:
// GPU & arg1

psyqo::CDRomDevice& object = /* obtain a valid instance */;

object.unmuteBlocking(arg1);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-cdromdevice-cdromdevice-1"></a>

## `psyqo::CDRomDevice::~CDRomDevice`

**Purpose.** Releases the resources owned by `psyqo::CDRomDevice`.

**Exact declaration**

```cpp
virtual ~CDRomDevice()
```

- **Declared at:** [line 222](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/cdrom-device.hh#L222)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need the cdrom device module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/cdrom-device.hh"

// `psyqo::CDRomDevice` cleans up when its owning scope ends.
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.
