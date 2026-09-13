# Epok API: Blueprint Api

> **Header:** `"blueprint_api.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/blueprint_api.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 34 public callables declared directly in this header.

## Callable index

- [`epok::bp::api::burst_effect`](#epok-bp-api-burst-effect-1) — Performs `burst effect` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::cast`](#epok-bp-api-cast-1) — Performs `cast` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::destroy`](#epok-bp-api-destroy-1) — Destroys destroy as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::effect_sequence`](#epok-bp-api-effect-sequence-1) — Performs `effect sequence` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::held`](#epok-bp-api-held-1) — Performs `held` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::make_transform`](#epok-bp-api-make-transform-1) — Performs `make transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::pause_effect`](#epok-bp-api-pause-effect-1) — Pauses effect as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::pause_sequence`](#epok-bp-api-pause-sequence-1) — Pauses sequence as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::play_audio`](#epok-bp-api-play-audio-1) — Starts audio as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::play_effect_component`](#epok-bp-api-play-effect-component-1) — Starts effect component as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::play_sequence_component`](#epok-bp-api-play-sequence-component-1) — Starts sequence component as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::playback_snapshot`](#epok-bp-api-playback-snapshot-1) — Starts back snapshot as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::position`](#epok-bp-api-position-1) — Performs `position` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::pressed`](#epok-bp-api-pressed-1) — Performs `pressed` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::released`](#epok-bp-api-released-1) — Performs `released` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::request_scene`](#epok-bp-api-request-scene-1) — Requests scene as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::resume_effect`](#epok-bp-api-resume-effect-1) — Resumes effect as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::resume_sequence`](#epok-bp-api-resume-sequence-1) — Resumes sequence as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::rotation`](#epok-bp-api-rotation-1) — Performs `rotation` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::scale`](#epok-bp-api-scale-1) — Performs `scale` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_active`](#epok-bp-api-set-active-1) — Sets active as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_audio_clip`](#epok-bp-api-set-audio-clip-1) — Sets audio clip as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_position`](#epok-bp-api-set-position-1) — Sets position as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_rotation`](#epok-bp-api-set-rotation-1) — Sets rotation as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_scale`](#epok-bp-api-set-scale-1) — Sets scale as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::set_texture`](#epok-bp-api-set-texture-1) — Sets texture as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::spawn`](#epok-bp-api-spawn-1) — Performs `spawn` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::spawn_class`](#epok-bp-api-spawn-class-1) — Performs `spawn class` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::stop_audio`](#epok-bp-api-stop-audio-1) — Stops audio as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::stop_effect`](#epok-bp-api-stop-effect-1) — Stops effect as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::stop_sequence`](#epok-bp-api-stop-sequence-1) — Stops sequence as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::transform`](#epok-bp-api-transform-1) — Performs `transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::api::valid`](#epok-bp-api-valid-1) — Performs `valid` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::asset_index`](#epok-bp-asset-index-1) — Performs `asset index` as part of compiled Blueprint execution and object interaction.

<a id="epok-bp-api-burst-effect-1"></a>

## `epok::bp::api::burst_effect`

**Purpose.** Performs `burst effect` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool burst_effect(effects::Handle,uint32_t)
```

- **Declared at:** [line 15](../../../runtime/blueprint_api.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `effects::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `uint32_t` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle arg1
// uint32_t arg2

auto result = epok::bp::api::burst_effect(arg1, arg2);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-cast-1"></a>

## `epok::bp::api::cast`

**Purpose.** Performs `cast` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline EntityHandle cast(ClassId type,EntityHandle target)
```

- **Declared at:** [line 55](../../../runtime/blueprint_api.hpp#L55)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `ClassId` | Input | Value supplied for `type`. See the exact type and module contract. |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// ClassId type
// EntityHandle target

auto result = epok::bp::api::cast(type, target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-destroy-1"></a>

## `epok::bp::api::destroy`

**Purpose.** Destroys destroy as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void destroy(EntityHandle target)
```

- **Declared at:** [line 46](../../../runtime/blueprint_api.hpp#L46)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

epok::bp::api::destroy(target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-effect-sequence-1"></a>

## `epok::bp::api::effect_sequence`

**Purpose.** Performs `effect sequence` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
timeline::Handle effect_sequence(effects::Handle)
```

- **Declared at:** [line 18](../../../runtime/blueprint_api.hpp#L18)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `effects::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `timeline::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle arg1

auto result = epok::bp::api::effect_sequence(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-held-1"></a>

## `epok::bp::api::held`

**Purpose.** Performs `held` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool held(uint32_t button,uint32_t port)
```

- **Declared at:** [line 41](../../../runtime/blueprint_api.hpp#L41)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `uint32_t` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t button
// uint32_t port

auto result = epok::bp::api::held(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-make-transform-1"></a>

## `epok::bp::api::make_transform`

**Purpose.** Performs `make transform` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Transform make_transform(const Vector<3>& position,const Vector<3>& rotation,const Vector<3>& scale)
```

- **Declared at:** [line 22](../../../runtime/blueprint_api.hpp#L22)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `position` | `const Vector<3> &` | Input | Value supplied for `position`. See the exact type and module contract. |
| `rotation` | `const Vector<3> &` | Input | Value supplied for `rotation`. See the exact type and module contract. |
| `scale` | `const Vector<3> &` | Input | Value supplied for `scale`. See the exact type and module contract. |

**Returns.** Returns `Transform`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// const Vector<3> & position
// const Vector<3> & rotation
// const Vector<3> & scale

auto result = epok::bp::api::make_transform(position, rotation, scale);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-api-pause-effect-1"></a>

## `epok::bp::api::pause_effect`

**Purpose.** Pauses effect as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool pause_effect(effects::Handle)
```

- **Declared at:** [line 16](../../../runtime/blueprint_api.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `effects::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle arg1

auto result = epok::bp::api::pause_effect(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-pause-sequence-1"></a>

## `epok::bp::api::pause_sequence`

**Purpose.** Pauses sequence as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool pause_sequence(timeline::Handle)
```

- **Declared at:** [line 11](../../../runtime/blueprint_api.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `timeline::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle arg1

auto result = epok::bp::api::pause_sequence(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-play-audio-1"></a>

## `epok::bp::api::play_audio`

**Purpose.** Starts audio as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void play_audio(EntityHandle target)
```

- **Declared at:** [line 47](../../../runtime/blueprint_api.hpp#L47)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

epok::bp::api::play_audio(target);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-bp-api-play-effect-component-1"></a>

## `epok::bp::api::play_effect_component`

**Purpose.** Starts effect component as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
effects::Handle play_effect_component(EntityHandle)
```

- **Declared at:** [line 13](../../../runtime/blueprint_api.hpp#L13)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EntityHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `effects::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle arg1

auto result = epok::bp::api::play_effect_component(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-play-sequence-component-1"></a>

## `epok::bp::api::play_sequence_component`

**Purpose.** Starts sequence component as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
timeline::Handle play_sequence_component(EntityHandle)
```

- **Declared at:** [line 9](../../../runtime/blueprint_api.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EntityHandle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `timeline::Handle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle arg1

auto result = epok::bp::api::play_sequence_component(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-playback-snapshot-1"></a>

## `epok::bp::api::playback_snapshot`

**Purpose.** Starts back snapshot as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
PlaybackSnapshot playback_snapshot(const PlaybackWait&)
```

- **Declared at:** [line 19](../../../runtime/blueprint_api.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const PlaybackWait &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `PlaybackSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// const PlaybackWait & arg1

auto result = epok::bp::api::playback_snapshot(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-api-position-1"></a>

## `epok::bp::api::position`

**Purpose.** Performs `position` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Vector<3> position(EntityHandle target)
```

- **Declared at:** [line 25](../../../runtime/blueprint_api.hpp#L25)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `Vector<3>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

auto result = epok::bp::api::position(target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-pressed-1"></a>

## `epok::bp::api::pressed`

**Purpose.** Performs `pressed` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool pressed(uint32_t button,uint32_t port)
```

- **Declared at:** [line 42](../../../runtime/blueprint_api.hpp#L42)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `uint32_t` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t button
// uint32_t port

auto result = epok::bp::api::pressed(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-released-1"></a>

## `epok::bp::api::released`

**Purpose.** Performs `released` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool released(uint32_t button,uint32_t port)
```

- **Declared at:** [line 43](../../../runtime/blueprint_api.hpp#L43)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `uint32_t` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t button
// uint32_t port

auto result = epok::bp::api::released(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-request-scene-1"></a>

## `epok::bp::api::request_scene`

**Purpose.** Requests scene as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool request_scene(uint32_t index)
```

- **Declared at:** [line 44](../../../runtime/blueprint_api.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `uint32_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t index

auto result = epok::bp::api::request_scene(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-resume-effect-1"></a>

## `epok::bp::api::resume_effect`

**Purpose.** Resumes effect as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool resume_effect(effects::Handle)
```

- **Declared at:** [line 17](../../../runtime/blueprint_api.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `effects::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle arg1

auto result = epok::bp::api::resume_effect(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-resume-sequence-1"></a>

## `epok::bp::api::resume_sequence`

**Purpose.** Resumes sequence as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool resume_sequence(timeline::Handle)
```

- **Declared at:** [line 12](../../../runtime/blueprint_api.hpp#L12)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `timeline::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle arg1

auto result = epok::bp::api::resume_sequence(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-rotation-1"></a>

## `epok::bp::api::rotation`

**Purpose.** Performs `rotation` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Vector<3> rotation(EntityHandle target)
```

- **Declared at:** [line 28](../../../runtime/blueprint_api.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `Vector<3>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

auto result = epok::bp::api::rotation(target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-scale-1"></a>

## `epok::bp::api::scale`

**Purpose.** Performs `scale` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Vector<3> scale(EntityHandle target)
```

- **Declared at:** [line 31](../../../runtime/blueprint_api.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `Vector<3>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

auto result = epok::bp::api::scale(target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-set-active-1"></a>

## `epok::bp::api::set_active`

**Purpose.** Sets active as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_active(EntityHandle target,bool active)
```

- **Declared at:** [line 45](../../../runtime/blueprint_api.hpp#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// bool active

epok::bp::api::set_active(target, active);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-set-audio-clip-1"></a>

## `epok::bp::api::set_audio_clip`

**Purpose.** Sets audio clip as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_audio_clip(EntityHandle target,uint64_t clip)
```

- **Declared at:** [line 63](../../../runtime/blueprint_api.hpp#L63)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `clip` | `uint64_t` | Input | Value supplied for `clip`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// uint64_t clip

epok::bp::api::set_audio_clip(target, clip);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-bp-api-set-position-1"></a>

## `epok::bp::api::set_position`

**Purpose.** Sets position as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_position(EntityHandle target,const Vector<3>& value)
```

- **Declared at:** [line 34](../../../runtime/blueprint_api.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `value` | `const Vector<3> &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// const Vector<3> & value

epok::bp::api::set_position(target, value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-api-set-rotation-1"></a>

## `epok::bp::api::set_rotation`

**Purpose.** Sets rotation as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_rotation(EntityHandle target,const Vector<3>& value)
```

- **Declared at:** [line 35](../../../runtime/blueprint_api.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `value` | `const Vector<3> &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// const Vector<3> & value

epok::bp::api::set_rotation(target, value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-api-set-scale-1"></a>

## `epok::bp::api::set_scale`

**Purpose.** Sets scale as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_scale(EntityHandle target,const Vector<3>& value)
```

- **Declared at:** [line 36](../../../runtime/blueprint_api.hpp#L36)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `value` | `const Vector<3> &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// const Vector<3> & value

epok::bp::api::set_scale(target, value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-api-set-texture-1"></a>

## `epok::bp::api::set_texture`

**Purpose.** Sets texture as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void set_texture(EntityHandle target,uint64_t texture)
```

- **Declared at:** [line 56](../../../runtime/blueprint_api.hpp#L56)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |
| `texture` | `uint64_t` | Input | Value supplied for `texture`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target
// uint64_t texture

epok::bp::api::set_texture(target, texture);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-spawn-1"></a>

## `epok::bp::api::spawn`

**Purpose.** Performs `spawn` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline EntityHandle spawn(ClassId type,EntityHandle parent)
```

- **Declared at:** [line 49](../../../runtime/blueprint_api.hpp#L49)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `ClassId` | Input | Value supplied for `type`. See the exact type and module contract. |
| `parent` | `EntityHandle` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// ClassId type
// EntityHandle parent

auto result = epok::bp::api::spawn(type, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-spawn-class-1"></a>

## `epok::bp::api::spawn_class`

**Purpose.** Performs `spawn class` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline EntityHandle spawn_class(ClassId base,ClassId type,EntityHandle parent)
```

- **Declared at:** [line 50](../../../runtime/blueprint_api.hpp#L50)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `base` | `ClassId` | Input | Value supplied for `base`. See the exact type and module contract. |
| `type` | `ClassId` | Input | Value supplied for `type`. See the exact type and module contract. |
| `parent` | `EntityHandle` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// ClassId base
// ClassId type
// EntityHandle parent

auto result = epok::bp::api::spawn_class(base, type, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-stop-audio-1"></a>

## `epok::bp::api::stop_audio`

**Purpose.** Stops audio as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void stop_audio(EntityHandle target)
```

- **Declared at:** [line 48](../../../runtime/blueprint_api.hpp#L48)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

epok::bp::api::stop_audio(target);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-bp-api-stop-effect-1"></a>

## `epok::bp::api::stop_effect`

**Purpose.** Stops effect as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool stop_effect(effects::Handle)
```

- **Declared at:** [line 14](../../../runtime/blueprint_api.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `effects::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// effects::Handle arg1

auto result = epok::bp::api::stop_effect(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-stop-sequence-1"></a>

## `epok::bp::api::stop_sequence`

**Purpose.** Stops sequence as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool stop_sequence(timeline::Handle)
```

- **Declared at:** [line 10](../../../runtime/blueprint_api.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `timeline::Handle` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// timeline::Handle arg1

auto result = epok::bp::api::stop_sequence(arg1);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-api-transform-1"></a>

## `epok::bp::api::transform`

**Purpose.** Performs `transform` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Transform transform(EntityHandle target)
```

- **Declared at:** [line 21](../../../runtime/blueprint_api.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `Transform`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

auto result = epok::bp::api::transform(target);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-api-valid-1"></a>

## `epok::bp::api::valid`

**Purpose.** Performs `valid` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool valid(EntityHandle target)
```

- **Declared at:** [line 20](../../../runtime/blueprint_api.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `target` | `EntityHandle` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle target

auto result = epok::bp::api::valid(target);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-asset-index-1"></a>

## `epok::bp::asset_index`

**Purpose.** Performs `asset index` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
int asset_index(uint64_t asset,uint32_t kind)
```

- **Declared at:** [line 4](../../../runtime/blueprint_api.hpp#L4)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `asset` | `uint64_t` | Input | Value supplied for `asset`. See the exact type and module contract. |
| `kind` | `uint32_t` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_api.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t asset
// uint32_t kind

auto result = epok::bp::asset_index(asset, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
