# Epok API: Music

> **Header:** `"music.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/music.hpp)

This module covers XA music streaming and playback state. It documents 10 public callables declared directly in this header.

## Declared types

`epok::XAAction`, `epok::XAState`

## Callable index

- [`epok::music_error`](#epok-music-error-1) — Performs `music error` as part of XA music streaming and playback state.
- [`epok::music_prepare`](#epok-music-prepare-1) — Performs `music prepare` as part of XA music streaming and playback state.
- [`epok::music_tick`](#epok-music-tick-1) — Performs `music tick` as part of XA music streaming and playback state.
- [`epok::XAAction::acknowledge`](#epok-xaaction-acknowledge-1) — Performs `acknowledge` as part of XA music streaming and playback state.
- [`epok::XAAction::complete`](#epok-xaaction-complete-1) — Performs `complete` as part of XA music streaming and playback state.
- [`epok::XAAction::dataReady`](#epok-xaaction-dataready-1) — Performs `data ready` as part of XA music streaming and playback state.
- [`epok::XAAction::end`](#epok-xaaction-end-1) — Ends end as part of XA music streaming and playback state.
- [`epok::XAAction::start`](#epok-xaaction-start-1) — Starts start as part of XA music streaming and playback state.
- [`epok::XAAction::stop`](#epok-xaaction-stop-1) — Stops stop as part of XA music streaming and playback state.
- [`epok::XAAction::XAAction`](#epok-xaaction-xaaction-1) — Constructs `epok::XAAction` for XA music streaming and playback state.

<a id="epok-music-error-1"></a>

## `epok::music_error`

**Purpose.** Performs `music error` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void music_error(uint32_t code)
```

- **Declared at:** [line 180](../../../runtime/music.hpp#L180)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `code` | `uint32_t` | Input | Value supplied for `code`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t code

epok::music_error(code);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-music-prepare-1"></a>

## `epok::music_prepare`

**Purpose.** Performs `music prepare` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void music_prepare(bool require_data = false)
```

- **Declared at:** [line 143](../../../runtime/music.hpp#L143)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `require_data` | `bool` | Input | Value supplied for `require_data`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// bool require_data

epok::music_prepare(require_data);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-music-tick-1"></a>

## `epok::music_tick`

**Purpose.** Performs `music tick` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
inline void music_tick()
```

- **Declared at:** [line 189](../../../runtime/music.hpp#L189)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

epok::music_tick();
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-xaaction-acknowledge-1"></a>

## `epok::XAAction::acknowledge`

**Purpose.** Performs `acknowledge` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool acknowledge(const psyqo::CDRomDevice::Response &) override
```

- **Declared at:** [line 65](../../../runtime/music.hpp#L65)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::XAAction& object = /* obtain a valid instance */;

auto result = object.acknowledge(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-xaaction-complete-1"></a>

## `epok::XAAction::complete`

**Purpose.** Performs `complete` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool complete(const psyqo::CDRomDevice::Response &) override
```

- **Declared at:** [line 132](../../../runtime/music.hpp#L132)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::XAAction& object = /* obtain a valid instance */;

auto result = object.complete(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-xaaction-dataready-1"></a>

## `epok::XAAction::dataReady`

**Purpose.** Performs `data ready` as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool dataReady(const psyqo::CDRomDevice::Response &) override
```

- **Declared at:** [line 103](../../../runtime/music.hpp#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::XAAction& object = /* obtain a valid instance */;

auto result = object.dataReady(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-xaaction-end-1"></a>

## `epok::XAAction::end`

**Purpose.** Ends end as part of XA music streaming and playback state.

**Exact declaration**

```cpp
bool end(const psyqo::CDRomDevice::Response &) override
```

- **Declared at:** [line 136](../../../runtime/music.hpp#L136)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const psyqo::CDRomDevice::Response &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// const psyqo::CDRomDevice::Response & arg1

epok::XAAction& object = /* obtain a valid instance */;

auto result = object.end(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-xaaction-start-1"></a>

## `epok::XAAction::start`

**Purpose.** Starts start as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void start(uint32_t sector, eastl::function<void(bool)> &&callback)
```

- **Declared at:** [line 49](../../../runtime/music.hpp#L49)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sector` | `uint32_t` | Input | Value supplied for `sector`. See the exact type and module contract. |
| `callback` | `eastl::function<void (bool)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t sector
// eastl::function<void (bool)> && callback

epok::XAAction& object = /* obtain a valid instance */;

object.start(sector, callback);
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-xaaction-stop-1"></a>

## `epok::XAAction::stop`

**Purpose.** Stops stop as part of XA music streaming and playback state.

**Exact declaration**

```cpp
void stop()
```

- **Declared at:** [line 60](../../../runtime/music.hpp#L60)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

epok::XAAction& object = /* obtain a valid instance */;

object.stop();
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-xaaction-xaaction-1"></a>

## `epok::XAAction::XAAction`

**Purpose.** Constructs `epok::XAAction` for XA music streaming and playback state.

**Exact declaration**

```cpp
XAAction() :
```

- **Declared at:** [line 48](../../../runtime/music.hpp#L48)
- **Kind:** `constructor`

**Use it when.** You need XA music streaming and playback state and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "music.hpp"

epok::XAAction value();
```

**Why choose it.** It provides direct, allocation-conscious access to XA music streaming and playback state. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
