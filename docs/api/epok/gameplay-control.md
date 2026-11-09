# Epok API: Gameplay Control

> **Header:** `"gameplay_control.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/gameplay_control.hpp)

This module covers the gameplay control module. It documents 47 public callables declared directly in this header.

## Declared types

`epok::GameplayAction`, `epok::GameplayActionSample`, `epok::GameplayControls`, `epok::GameplayInputLibrary`, `epok::InputCapture`

## Callable index

- [`epok::gameplay_controls`](#epok-gameplay-controls-1) — Performs `gameplay controls` as part of the gameplay control module.
- [`epok::gameplay_controls_activation_changed`](#epok-gameplay-controls-activation-changed-1) — Observe the inactive state before a nested callback can enable the owner again.
- [`epok::GameplayControls::all_actions`](#epok-gameplaycontrols-all-actions-1) — Performs `all actions` as part of the gameplay control module.
- [`epok::GameplayControls::axis_raw`](#epok-gameplaycontrols-axis-raw-1) — Performs `axis raw` as part of the gameplay control module.
- [`epok::GameplayControls::begin_frame`](#epok-gameplaycontrols-begin-frame-1) — Begins frame as part of the gameplay control module.
- [`epok::GameplayControls::begin_tick`](#epok-gameplaycontrols-begin-tick-1) — Begins tick as part of the gameplay control module.
- [`epok::GameplayControls::bind_axis`](#epok-gameplaycontrols-bind-axis-1) — Performs `bind axis` as part of the gameplay control module.
- [`epok::GameplayControls::bind_button`](#epok-gameplaycontrols-bind-button-1) — Performs `bind button` as part of the gameplay control module.
- [`epok::GameplayControls::button_held`](#epok-gameplaycontrols-button-held-1) — Temporary physical-source adapters for the existing controller templates.
- [`epok::GameplayControls::button_pressed`](#epok-gameplaycontrols-button-pressed-1) — Performs `button pressed` as part of the gameplay control module.
- [`epok::GameplayControls::capture`](#epok-gameplaycontrols-capture-1) — Performs `capture` as part of the gameplay control module.
- [`epok::GameplayControls::captured`](#epok-gameplaycontrols-captured-1) — Performs `captured` as part of the gameplay control module.
- [`epok::GameplayControls::context_sample`](#epok-gameplaycontrols-context-sample-1) — Performs `context sample` as part of the gameplay control module.
- [`epok::GameplayControls::context_take_pressed`](#epok-gameplaycontrols-context-take-pressed-1) — Performs `context take pressed` as part of the gameplay control module.
- [`epok::GameplayControls::discard_tick_edges`](#epok-gameplaycontrols-discard-tick-edges-1) — Performs `discard tick edges` as part of the gameplay control module.
- [`epok::GameplayControls::end_tick`](#epok-gameplaycontrols-end-tick-1) — Ends tick as part of the gameplay control module.
- [`epok::GameplayControls::GameplayControls`](#epok-gameplaycontrols-gameplaycontrols-1) — Constructs `epok::GameplayControls` for the gameplay control module.
- [`epok::GameplayControls::mask`](#epok-gameplaycontrols-mask-1) — Performs `mask` as part of the gameplay control module.
- [`epok::GameplayControls::owner_capture`](#epok-gameplaycontrols-owner-capture-1) — Performs `owner capture` as part of the gameplay control module.
- [`epok::GameplayControls::release`](#epok-gameplaycontrols-release-1) — Performs `release` as part of the gameplay control module.
- [`epok::GameplayControls::reset`](#epok-gameplaycontrols-reset-1) — Resets reset as part of the gameplay control module.
- [`epok::GameplayControls::sample`](#epok-gameplaycontrols-sample-1) — Performs `sample` as part of the gameplay control module.
- [`epok::GameplayControls::take_pressed`](#epok-gameplaycontrols-take-pressed-1) — Performs `take pressed` as part of the gameplay control module.
- [`epok::GameplayControls::valid`](#epok-gameplaycontrols-valid-1) — Performs `valid` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::all_actions`](#epok-gameplayinputlibrary-all-actions-1) — Performs `all actions` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::axis`](#epok-gameplayinputlibrary-axis-1) — Performs `axis` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::axis_raw`](#epok-gameplayinputlibrary-axis-raw-1) — Performs `axis raw` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::axis_value`](#epok-gameplayinputlibrary-axis-value-1) — Performs `axis value` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::bind_axis`](#epok-gameplayinputlibrary-bind-axis-1) — Performs `bind axis` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::bind_button`](#epok-gameplayinputlibrary-bind-button-1) — Performs `bind button` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::button_frame_pressed`](#epok-gameplayinputlibrary-button-frame-pressed-1) — Performs `button frame pressed` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::button_held`](#epok-gameplayinputlibrary-button-held-1) — Performs `button held` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::button_pressed`](#epok-gameplayinputlibrary-button-pressed-1) — Performs `button pressed` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::capture`](#epok-gameplayinputlibrary-capture-1) — Performs `capture` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::captured`](#epok-gameplayinputlibrary-captured-1) — Performs `captured` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::context_sample`](#epok-gameplayinputlibrary-context-sample-1) — Performs `context sample` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::context_take_pressed`](#epok-gameplayinputlibrary-context-take-pressed-1) — Performs `context take pressed` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::frame_sample`](#epok-gameplayinputlibrary-frame-sample-1) — Performs `frame sample` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::look_actions`](#epok-gameplayinputlibrary-look-actions-1) — Performs `look actions` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::mask`](#epok-gameplayinputlibrary-mask-1) — Performs `mask` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::movement_actions`](#epok-gameplayinputlibrary-movement-actions-1) — Performs `movement actions` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::owner_capture`](#epok-gameplayinputlibrary-owner-capture-1) — Performs `owner capture` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::release`](#epok-gameplayinputlibrary-release-1) — Performs `release` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::sample`](#epok-gameplayinputlibrary-sample-1) — Performs `sample` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::take_frame_pressed`](#epok-gameplayinputlibrary-take-frame-pressed-1) — Performs `take frame pressed` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::take_pressed`](#epok-gameplayinputlibrary-take-pressed-1) — Performs `take pressed` as part of the gameplay control module.
- [`epok::GameplayInputLibrary::valid`](#epok-gameplayinputlibrary-valid-1) — Performs `valid` as part of the gameplay control module.

<a id="epok-gameplay-controls-1"></a>

## `epok::gameplay_controls`

**Purpose.** Performs `gameplay controls` as part of the gameplay control module.

**Exact declaration**

```cpp
inline GameplayControls& gameplay_controls()
```

- **Declared at:** [line 263](../../../runtime/gameplay_control.hpp#L263)
- **Kind:** `function decl`

**Returns.** Returns `GameplayControls &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

auto result = epok::gameplay_controls();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-controls-activation-changed-1"></a>

## `epok::gameplay_controls_activation_changed`

**Purpose.** Observe the inactive state before a nested callback can enable the owner again.

**Exact declaration**

```cpp
inline void gameplay_controls_activation_changed()
```

- **Declared at:** [line 265](../../../runtime/gameplay_control.hpp#L265)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::gameplay_controls_activation_changed();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-all-actions-1"></a>

## `epok::GameplayControls::all_actions`

**Purpose.** Performs `all actions` as part of the gameplay control module.

**Exact declaration**

```cpp
static constexpr uint32_t all_actions()
```

- **Declared at:** [line 167](../../../runtime/gameplay_control.hpp#L167)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

auto result = epok::GameplayControls::all_actions();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-axis-raw-1"></a>

## `epok::GameplayControls::axis_raw`

**Purpose.** Performs `axis raw` as part of the gameplay control module.

**Exact declaration**

```cpp
int32_t axis_raw(Axis axis,unsigned port)
```

- **Declared at:** [line 257](../../../runtime/gameplay_control.hpp#L257)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.axis_raw(axis, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-begin-frame-1"></a>

## `epok::GameplayControls::begin_frame`

**Purpose.** Begins frame as part of the gameplay control module.

**Exact declaration**

```cpp
void begin_frame()
```

- **Declared at:** [line 177](../../../runtime/gameplay_control.hpp#L177)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::GameplayControls& object = /* obtain a valid instance */;

object.begin_frame();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-begin-tick-1"></a>

## `epok::GameplayControls::begin_tick`

**Purpose.** Begins tick as part of the gameplay control module.

**Exact declaration**

```cpp
void begin_tick()
```

- **Declared at:** [line 188](../../../runtime/gameplay_control.hpp#L188)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::GameplayControls& object = /* obtain a valid instance */;

object.begin_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-bind-axis-1"></a>

## `epok::GameplayControls::bind_axis`

**Purpose.** Performs `bind axis` as part of the gameplay control module.

**Exact declaration**

```cpp
bool bind_axis(GameplayAction action,Axis axis,Button negative,Button positive,unsigned port)
```

- **Declared at:** [line 231](../../../runtime/gameplay_control.hpp#L231)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `negative` | `Button` | Input | Value supplied for `negative`. See the exact type and module contract. |
| `positive` | `Button` | Input | Value supplied for `positive`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// Axis axis
// Button negative
// Button positive
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.bind_axis(action, axis, negative, positive, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-bind-button-1"></a>

## `epok::GameplayControls::bind_button`

**Purpose.** Performs `bind button` as part of the gameplay control module.

**Exact declaration**

```cpp
bool bind_button(GameplayAction action,Button button,unsigned port)
```

- **Declared at:** [line 227](../../../runtime/gameplay_control.hpp#L227)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// Button button
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.bind_button(action, button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-button-held-1"></a>

## `epok::GameplayControls::button_held`

**Purpose.** Temporary physical-source adapters for the existing controller templates.

**Exact declaration**

```cpp
bool button_held(Button button,unsigned port)
```

- **Declared at:** [line 249](../../../runtime/gameplay_control.hpp#L249)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.button_held(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-button-pressed-1"></a>

## `epok::GameplayControls::button_pressed`

**Purpose.** Performs `button pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
bool button_pressed(Button button,unsigned port,bool frame=false)
```

- **Declared at:** [line 253](../../../runtime/gameplay_control.hpp#L253)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// unsigned int port
// bool frame

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.button_pressed(button, port, frame);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-capture-1"></a>

## `epok::GameplayControls::capture`

**Purpose.** Performs `capture` as part of the gameplay control module.

**Exact declaration**

```cpp
InputCapture capture(ObjectId owner,uint32_t actions,int32_t priority,unsigned port)
```

- **Declared at:** [line 199](../../../runtime/gameplay_control.hpp#L199)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `ObjectId` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `actions` | `uint32_t` | Input | Value supplied for `actions`. See the exact type and module contract. |
| `priority` | `int32_t` | Input | Value supplied for `priority`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputCapture`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId owner
// uint32_t actions
// int32_t priority
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.capture(owner, actions, priority, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-captured-1"></a>

## `epok::GameplayControls::captured`

**Purpose.** Performs `captured` as part of the gameplay control module.

**Exact declaration**

```cpp
bool captured(GameplayAction action,unsigned port)
```

- **Declared at:** [line 168](../../../runtime/gameplay_control.hpp#L168)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.captured(action, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-context-sample-1"></a>

## `epok::GameplayControls::context_sample`

**Purpose.** Performs `context sample` as part of the gameplay control module.

**Exact declaration**

```cpp
GameplayActionSample context_sample(InputCapture token,GameplayAction action,bool frame)
```

- **Declared at:** [line 239](../../../runtime/gameplay_control.hpp#L239)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token
// GameplayAction action
// bool frame

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.context_sample(token, action, frame);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-context-take-pressed-1"></a>

## `epok::GameplayControls::context_take_pressed`

**Purpose.** Performs `context take pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
bool context_take_pressed(InputCapture token,GameplayAction action,bool frame)
```

- **Declared at:** [line 245](../../../runtime/gameplay_control.hpp#L245)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token
// GameplayAction action
// bool frame

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.context_take_pressed(token, action, frame);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-discard-tick-edges-1"></a>

## `epok::GameplayControls::discard_tick_edges`

**Purpose.** Performs `discard tick edges` as part of the gameplay control module.

**Exact declaration**

```cpp
void discard_tick_edges()
```

- **Declared at:** [line 195](../../../runtime/gameplay_control.hpp#L195)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::GameplayControls& object = /* obtain a valid instance */;

object.discard_tick_edges();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-end-tick-1"></a>

## `epok::GameplayControls::end_tick`

**Purpose.** Ends tick as part of the gameplay control module.

**Exact declaration**

```cpp
void end_tick()
```

- **Declared at:** [line 198](../../../runtime/gameplay_control.hpp#L198)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::GameplayControls& object = /* obtain a valid instance */;

object.end_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-gameplaycontrols-1"></a>

## `epok::GameplayControls::GameplayControls`

**Purpose.** Constructs `epok::GameplayControls` for the gameplay control module.

**Exact declaration**

```cpp
constexpr explicit GameplayControls(Input& source=input):m_input(&source)
```

- **Declared at:** [line 163](../../../runtime/gameplay_control.hpp#L163)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `Input &` | Input/output; inspect the function contract | Value supplied for `source`. See the exact type and module contract. |

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Input & source

epok::GameplayControls value(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplaycontrols-mask-1"></a>

## `epok::GameplayControls::mask`

**Purpose.** Performs `mask` as part of the gameplay control module.

**Exact declaration**

```cpp
static constexpr uint32_t mask(GameplayAction action)
```

- **Declared at:** [line 164](../../../runtime/gameplay_control.hpp#L164)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action

auto result = epok::GameplayControls::mask(action);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-owner-capture-1"></a>

## `epok::GameplayControls::owner_capture`

**Purpose.** Performs `owner capture` as part of the gameplay control module.

**Exact declaration**

```cpp
InputCapture owner_capture(ObjectId owner,uint32_t actions,unsigned port)
```

- **Declared at:** [line 214](../../../runtime/gameplay_control.hpp#L214)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `ObjectId` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `actions` | `uint32_t` | Input | Value supplied for `actions`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputCapture`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId owner
// uint32_t actions
// unsigned int port

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.owner_capture(owner, actions, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-release-1"></a>

## `epok::GameplayControls::release`

**Purpose.** Performs `release` as part of the gameplay control module.

**Exact declaration**

```cpp
bool release(InputCapture token)
```

- **Declared at:** [line 224](../../../runtime/gameplay_control.hpp#L224)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.release(token);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-reset-1"></a>

## `epok::GameplayControls::reset`

**Purpose.** Resets reset as part of the gameplay control module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 172](../../../runtime/gameplay_control.hpp#L172)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

epok::GameplayControls& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-sample-1"></a>

## `epok::GameplayControls::sample`

**Purpose.** Performs `sample` as part of the gameplay control module.

**Exact declaration**

```cpp
GameplayActionSample sample(GameplayAction action,unsigned port,bool frame=false)
```

- **Declared at:** [line 236](../../../runtime/gameplay_control.hpp#L236)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// unsigned int port
// bool frame

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.sample(action, port, frame);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplaycontrols-take-pressed-1"></a>

## `epok::GameplayControls::take_pressed`

**Purpose.** Performs `take pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
bool take_pressed(GameplayAction action,unsigned port,bool frame=false)
```

- **Declared at:** [line 242](../../../runtime/gameplay_control.hpp#L242)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// unsigned int port
// bool frame

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.take_pressed(action, port, frame);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplaycontrols-valid-1"></a>

## `epok::GameplayControls::valid`

**Purpose.** Performs `valid` as part of the gameplay control module.

**Exact declaration**

```cpp
bool valid(InputCapture token)
```

- **Declared at:** [line 213](../../../runtime/gameplay_control.hpp#L213)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token

epok::GameplayControls& object = /* obtain a valid instance */;

auto result = object.valid(token);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-all-actions-1"></a>

## `epok::GameplayInputLibrary::all_actions`

**Purpose.** Performs `all actions` as part of the gameplay control module.

**Exact declaration**

```cpp
static uint32_t all_actions()
```

- **Declared at:** [line 271](../../../runtime/gameplay_control.hpp#L271)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

auto result = epok::GameplayInputLibrary::all_actions();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-axis-1"></a>

## `epok::GameplayInputLibrary::axis`

**Purpose.** Performs `axis` as part of the gameplay control module.

**Exact declaration**

```cpp
static GameplayActionSample axis(Axis axis,uint32_t port)
```

- **Declared at:** [line 290](../../../runtime/gameplay_control.hpp#L290)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// uint32_t port

auto result = epok::GameplayInputLibrary::axis(axis, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-axis-raw-1"></a>

## `epok::GameplayInputLibrary::axis_raw`

**Purpose.** Performs `axis raw` as part of the gameplay control module.

**Exact declaration**

```cpp
static int32_t axis_raw(Axis axis,uint32_t port=0)
```

- **Declared at:** [line 288](../../../runtime/gameplay_control.hpp#L288)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// uint32_t port

auto result = epok::GameplayInputLibrary::axis_raw(axis, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-axis-value-1"></a>

## `epok::GameplayInputLibrary::axis_value`

**Purpose.** Performs `axis value` as part of the gameplay control module.

**Exact declaration**

```cpp
static Fixed axis_value(Axis axis,uint32_t port)
```

- **Declared at:** [line 289](../../../runtime/gameplay_control.hpp#L289)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Axis axis
// uint32_t port

auto result = epok::GameplayInputLibrary::axis_value(axis, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-bind-axis-1"></a>

## `epok::GameplayInputLibrary::bind_axis`

**Purpose.** Performs `bind axis` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool bind_axis(GameplayAction action,Axis axis,Button negative,Button positive,uint32_t port)
```

- **Declared at:** [line 284](../../../runtime/gameplay_control.hpp#L284)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `axis` | `Axis` | Input | Value supplied for `axis`. See the exact type and module contract. |
| `negative` | `Button` | Input | Value supplied for `negative`. See the exact type and module contract. |
| `positive` | `Button` | Input | Value supplied for `positive`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// Axis axis
// Button negative
// Button positive
// uint32_t port

auto result = epok::GameplayInputLibrary::bind_axis(action, axis, negative, positive, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-bind-button-1"></a>

## `epok::GameplayInputLibrary::bind_button`

**Purpose.** Performs `bind button` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool bind_button(GameplayAction action,Button button,uint32_t port)
```

- **Declared at:** [line 283](../../../runtime/gameplay_control.hpp#L283)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// Button button
// uint32_t port

auto result = epok::GameplayInputLibrary::bind_button(action, button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-button-frame-pressed-1"></a>

## `epok::GameplayInputLibrary::button_frame_pressed`

**Purpose.** Performs `button frame pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool button_frame_pressed(Button button,uint32_t port=0)
```

- **Declared at:** [line 287](../../../runtime/gameplay_control.hpp#L287)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

auto result = epok::GameplayInputLibrary::button_frame_pressed(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-button-held-1"></a>

## `epok::GameplayInputLibrary::button_held`

**Purpose.** Performs `button held` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool button_held(Button button,uint32_t port=0)
```

- **Declared at:** [line 285](../../../runtime/gameplay_control.hpp#L285)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

auto result = epok::GameplayInputLibrary::button_held(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-button-pressed-1"></a>

## `epok::GameplayInputLibrary::button_pressed`

**Purpose.** Performs `button pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool button_pressed(Button button,uint32_t port=0)
```

- **Declared at:** [line 286](../../../runtime/gameplay_control.hpp#L286)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// Button button
// uint32_t port

auto result = epok::GameplayInputLibrary::button_pressed(button, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-capture-1"></a>

## `epok::GameplayInputLibrary::capture`

**Purpose.** Performs `capture` as part of the gameplay control module.

**Exact declaration**

```cpp
static InputCapture capture(ObjectId owner,uint32_t actions,int32_t priority,uint32_t port)
```

- **Declared at:** [line 272](../../../runtime/gameplay_control.hpp#L272)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `ObjectId` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `actions` | `uint32_t` | Input | Value supplied for `actions`. See the exact type and module contract. |
| `priority` | `int32_t` | Input | Value supplied for `priority`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputCapture`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId owner
// uint32_t actions
// int32_t priority
// uint32_t port

auto result = epok::GameplayInputLibrary::capture(owner, actions, priority, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-captured-1"></a>

## `epok::GameplayInputLibrary::captured`

**Purpose.** Performs `captured` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool captured(GameplayAction action,uint32_t port)
```

- **Declared at:** [line 277](../../../runtime/gameplay_control.hpp#L277)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// uint32_t port

auto result = epok::GameplayInputLibrary::captured(action, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-context-sample-1"></a>

## `epok::GameplayInputLibrary::context_sample`

**Purpose.** Performs `context sample` as part of the gameplay control module.

**Exact declaration**

```cpp
static GameplayActionSample context_sample(InputCapture token,GameplayAction action,bool frame)
```

- **Declared at:** [line 279](../../../runtime/gameplay_control.hpp#L279)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token
// GameplayAction action
// bool frame

auto result = epok::GameplayInputLibrary::context_sample(token, action, frame);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-context-take-pressed-1"></a>

## `epok::GameplayInputLibrary::context_take_pressed`

**Purpose.** Performs `context take pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool context_take_pressed(InputCapture token,GameplayAction action,bool frame)
```

- **Declared at:** [line 282](../../../runtime/gameplay_control.hpp#L282)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `frame` | `bool` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token
// GameplayAction action
// bool frame

auto result = epok::GameplayInputLibrary::context_take_pressed(token, action, frame);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-frame-sample-1"></a>

## `epok::GameplayInputLibrary::frame_sample`

**Purpose.** Performs `frame sample` as part of the gameplay control module.

**Exact declaration**

```cpp
static GameplayActionSample frame_sample(GameplayAction action,uint32_t port)
```

- **Declared at:** [line 278](../../../runtime/gameplay_control.hpp#L278)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// uint32_t port

auto result = epok::GameplayInputLibrary::frame_sample(action, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-look-actions-1"></a>

## `epok::GameplayInputLibrary::look_actions`

**Purpose.** Performs `look actions` as part of the gameplay control module.

**Exact declaration**

```cpp
static uint32_t look_actions()
```

- **Declared at:** [line 270](../../../runtime/gameplay_control.hpp#L270)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

auto result = epok::GameplayInputLibrary::look_actions();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-mask-1"></a>

## `epok::GameplayInputLibrary::mask`

**Purpose.** Performs `mask` as part of the gameplay control module.

**Exact declaration**

```cpp
static uint32_t mask(GameplayAction action)
```

- **Declared at:** [line 268](../../../runtime/gameplay_control.hpp#L268)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action

auto result = epok::GameplayInputLibrary::mask(action);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-movement-actions-1"></a>

## `epok::GameplayInputLibrary::movement_actions`

**Purpose.** Performs `movement actions` as part of the gameplay control module.

**Exact declaration**

```cpp
static uint32_t movement_actions()
```

- **Declared at:** [line 269](../../../runtime/gameplay_control.hpp#L269)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

auto result = epok::GameplayInputLibrary::movement_actions();
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-owner-capture-1"></a>

## `epok::GameplayInputLibrary::owner_capture`

**Purpose.** Performs `owner capture` as part of the gameplay control module.

**Exact declaration**

```cpp
static InputCapture owner_capture(ObjectId owner,uint32_t actions,uint32_t port)
```

- **Declared at:** [line 275](../../../runtime/gameplay_control.hpp#L275)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `ObjectId` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `actions` | `uint32_t` | Input | Value supplied for `actions`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `InputCapture`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId owner
// uint32_t actions
// uint32_t port

auto result = epok::GameplayInputLibrary::owner_capture(owner, actions, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-release-1"></a>

## `epok::GameplayInputLibrary::release`

**Purpose.** Performs `release` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool release(InputCapture token)
```

- **Declared at:** [line 273](../../../runtime/gameplay_control.hpp#L273)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token

auto result = epok::GameplayInputLibrary::release(token);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-sample-1"></a>

## `epok::GameplayInputLibrary::sample`

**Purpose.** Performs `sample` as part of the gameplay control module.

**Exact declaration**

```cpp
static GameplayActionSample sample(GameplayAction action,uint32_t port)
```

- **Declared at:** [line 276](../../../runtime/gameplay_control.hpp#L276)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `GameplayActionSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// uint32_t port

auto result = epok::GameplayInputLibrary::sample(action, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the gameplay control module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplayinputlibrary-take-frame-pressed-1"></a>

## `epok::GameplayInputLibrary::take_frame_pressed`

**Purpose.** Performs `take frame pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool take_frame_pressed(GameplayAction action,uint32_t port)
```

- **Declared at:** [line 281](../../../runtime/gameplay_control.hpp#L281)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// uint32_t port

auto result = epok::GameplayInputLibrary::take_frame_pressed(action, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-take-pressed-1"></a>

## `epok::GameplayInputLibrary::take_pressed`

**Purpose.** Performs `take pressed` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool take_pressed(GameplayAction action,uint32_t port)
```

- **Declared at:** [line 280](../../../runtime/gameplay_control.hpp#L280)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `action` | `GameplayAction` | Input | Value supplied for `action`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// GameplayAction action
// uint32_t port

auto result = epok::GameplayInputLibrary::take_pressed(action, port);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-gameplayinputlibrary-valid-1"></a>

## `epok::GameplayInputLibrary::valid`

**Purpose.** Performs `valid` as part of the gameplay control module.

**Exact declaration**

```cpp
static bool valid(InputCapture token)
```

- **Declared at:** [line 274](../../../runtime/gameplay_control.hpp#L274)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `token` | `InputCapture` | Input | Value supplied for `token`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the gameplay control module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "gameplay_control.hpp"

// Assume these named values have been initialized with valid data:
// InputCapture token

auto result = epok::GameplayInputLibrary::valid(token);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.
