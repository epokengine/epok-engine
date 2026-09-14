# Epok API: Scene Service

> **Header:** `"scene_service.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/scene_service.hpp)

This module covers scene lifecycle and scene-stack control. It documents 3 public callables declared directly in this header.

## Declared types

`epok::SceneStats`

## Callable index

- [`epok::install_actor_service_hooks`](#epok-install-actor-service-hooks-1) — Performs `install actor service hooks` as part of scene lifecycle and scene-stack control.
- [`epok::music_retains_audio_source`](#epok-music-retains-audio-source-1) — Component-owned AudioSource storage stays quarantined while the XA consumer still points at it -- exactly the rule allocate_actor_data (runtime/lifecycle.hpp) applies to a legacy slot's `audio`.
- [`epok::scene_tick`](#epok-scene-tick-1) — Performs `scene tick` as part of scene lifecycle and scene-stack control.

<a id="epok-install-actor-service-hooks-1"></a>

## `epok::install_actor_service_hooks`

**Purpose.** Performs `install actor service hooks` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline void install_actor_service_hooks()
```

- **Declared at:** [line 25](../../../runtime/scene_service.hpp#L25)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_service.hpp"

epok::install_actor_service_hooks();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-music-retains-audio-source-1"></a>

## `epok::music_retains_audio_source`

**Purpose.** Component-owned AudioSource storage stays quarantined while the XA consumer still points at it -- exactly the rule allocate_actor_data (runtime/lifecycle.hpp) applies to a legacy slot's `audio`.

**Details.** `music_lookup` means a CD lookup for `music_active` is still in flight, so that source is retained even between requests.

**Exact declaration**

```cpp
inline bool music_retains_audio_source(const AudioSource* source)
```

- **Declared at:** [line 21](../../../runtime/scene_service.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `const AudioSource *` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** `music_lookup` means a CD lookup for `music_active` is still in flight, so that source is retained even between requests.

**Usage pattern**

```cpp
#include "scene_service.hpp"

// Assume these named values have been initialized with valid data:
// const AudioSource * source

auto result = epok::music_retains_audio_source(source);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scene-tick-1"></a>

## `epok::scene_tick`

**Purpose.** Performs `scene tick` as part of scene lifecycle and scene-stack control.

**Exact declaration**

```cpp
inline bool scene_tick(psyqo::GPU& gpu)
```

- **Declared at:** [line 56](../../../runtime/scene_service.hpp#L56)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `gpu` | `psyqo::GPU &` | Input/output; inspect the function contract | Value supplied for `gpu`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "scene_service.hpp"

// Assume these named values have been initialized with valid data:
// psyqo::GPU & gpu

auto result = epok::scene_tick(gpu);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
