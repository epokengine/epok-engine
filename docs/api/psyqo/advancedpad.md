# PsyQo API: Advancedpad

> **Header:** `"psyqo/advancedpad.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh)

This module covers the advancedpad module. It documents 11 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::AdvancedPad`, `psyqo::AdvancedPad::Button`, `psyqo::AdvancedPad::Event`, `psyqo::AdvancedPad::Event::(unnamed enum at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\advancedpad.hh:84:9)`, `psyqo::AdvancedPad::Pad`, `psyqo::AdvancedPad::PadData::(anonymous struct at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\advancedpad.hh:235:9)`, `psyqo::AdvancedPad::PadType`, `psyqo::AdvancedPad::PollingMode`

## Callable index

- [`psyqo::AdvancedPad::getAdc`](#psyqo-advancedpad-getadc-1) — Returns the state of an Analog Input.
- [`psyqo::AdvancedPad::getHalfword`](#psyqo-advancedpad-gethalfword-1) — Returns raw pad data as an unsigned 16-bit value.
- [`psyqo::AdvancedPad::getPadType`](#psyqo-advancedpad-getpadtype-1) — Returns the type of the pad.
- [`psyqo::AdvancedPad::initialize`](#psyqo-advancedpad-initialize-1) — Performs `initialize` as part of the advancedpad module.
- [`psyqo::AdvancedPad::isButtonPressed`](#psyqo-advancedpad-isbuttonpressed-1) — Returns the state of a button.
- [`psyqo::AdvancedPad::isPadConnected`](#psyqo-advancedpad-ispadconnected-1) — Returns the state of a pad.
- [`psyqo::AdvancedPad::setOnEvent`](#psyqo-advancedpad-setonevent-1) — Sets the event callback function.
- [`psyqo::operator++`](#psyqo-operator-1) — postfix increment operator
- [`psyqo::operator++`](#psyqo-operator-2) — prefix increment operator
- [`psyqo::operator--`](#psyqo-operator-3) — postfix decrement operator
- [`psyqo::operator--`](#psyqo-operator-4) — prefix decrement operator

<a id="psyqo-advancedpad-getadc-1"></a>

## `psyqo::AdvancedPad::getAdc`

**Purpose.** Returns the state of an Analog Input.

**Details.** See the specific Analog Input functions for details. Indices greater than 3 will return 0. index 0: For analog pads: RightJoyX (00h=Left, 80h=Center, FFh=Right), mouse: X-axis index 1: For analog pads: RightJoyY (00h=Up, 80h=Center, FFh=Down), mouse: Y-axis index 2: For analog pads: LeftJoyX (00h=Left, 80h=Center, FFh=Right) index 3: For analog pads: LeftJoyY (00h=Up, 80h=Center, FFh=Down)

**Exact declaration**

```cpp
uint8_t getAdc(Pad pad, unsigned int index) const
```

- **Declared at:** [line 164](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L164)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |
| `index` | `unsigned int` | Input | The index of the Analog Input(adc#). |

**Returns.** The state of the Analog Input as an unsigned 8-bit value(0-255).

**Use it when.** See the specific Analog Input functions for details. Indices greater than 3 will return 0. index 0: For analog pads: RightJoyX (00h=Left, 80h=Center, FFh=Right), mouse: X-axis index 1: For analog pads: RightJoyY (00h=Up, 80h=Center, FFh=Down), mouse: Y-axis index 2: For analog pads: LeftJoyX (00h=Left, 80h=Center, FFh=Right) index 3: For analog pads: LeftJoyY (00h=Up, 80h=Center, FFh=Down)

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad
// unsigned int index

psyqo::AdvancedPad& object = /* obtain a valid instance */;

auto result = object.getAdc(pad, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-advancedpad-gethalfword-1"></a>

## `psyqo::AdvancedPad::getHalfword`

**Purpose.** Returns raw pad data as an unsigned 16-bit value.

**Details.** A low level call which returns the halfword value for the requested index of the given pad index. It is recommended to use the higher level functions instead. index 0: pad type << 8 | connected(0 = connected, ffh = disconnected) index 1: button state index 2: analog input 1 << 8 | analog input 0 index 3: analog input 3 << 8 | analog input 2 The index is modulo 4, so it will wrap around if it is greater than 3.

**Exact declaration**

```cpp
uint16_t getHalfword(Pad pad, unsigned int index) const
```

- **Declared at:** [line 187](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L187)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |
| `index` | `unsigned int` | Input | The index of the halfword. |

**Returns.** The value of the halfword.

**Use it when.** A low level call which returns the halfword value for the requested index of the given pad index. It is recommended to use the higher level functions instead. index 0: pad type << 8 | connected(0 = connected, ffh = disconnected) index 1: button state index 2: analog input 1 << 8 | analog input 0 index 3: analog input 3 << 8 | analog input 2 The index is modulo 4, so it will wrap around if it is greater than 3.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad
// unsigned int index

psyqo::AdvancedPad& object = /* obtain a valid instance */;

auto result = object.getHalfword(pad, index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-advancedpad-getpadtype-1"></a>

## `psyqo::AdvancedPad::getPadType`

**Purpose.** Returns the type of the pad.

**Details.** Known pad types are defined in the PadType enum, returns 0xff if no pad is connected. PadType::Multitap is for internal use only, and should not be returned. Pad connection status should be checked with isPadConnected.

**Exact declaration**

```cpp
uint8_t getPadType(Pad pad) const
```

- **Declared at:** [line 199](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L199)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |

**Returns.** The type of the pad.

**Use it when.** Known pad types are defined in the PadType enum, returns 0xff if no pad is connected. PadType::Multitap is for internal use only, and should not be returned. Pad connection status should be checked with isPadConnected.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad

psyqo::AdvancedPad& object = /* obtain a valid instance */;

auto result = object.getPadType(pad);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-advancedpad-initialize-1"></a>

## `psyqo::AdvancedPad::initialize`

**Purpose.** Performs `initialize` as part of the advancedpad module.

**Exact declaration**

```cpp
void initialize(PollingMode mode = PollingMode::Normal)
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `mode` | `PollingMode` | Input | Value supplied for `mode`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the advancedpad module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// PollingMode mode

psyqo::AdvancedPad& object = /* obtain a valid instance */;

object.initialize(mode);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-advancedpad-isbuttonpressed-1"></a>

## `psyqo::AdvancedPad::isButtonPressed`

**Purpose.** Returns the state of a button.

**Details.** Returns the state of a button. The state is a boolean value that is `true` if the button is pressed, and `false` otherwise.

**Exact declaration**

```cpp
bool isButtonPressed(Pad pad, Button button) const
```

- **Declared at:** [line 146](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L146)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |
| `button` | `Button` | Input | The button to query. |

**Returns.** A boolean value indicating whether the button is pressed.

**Use it when.** Returns the state of a button. The state is a boolean value that is `true` if the button is pressed, and `false` otherwise.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad
// Button button

psyqo::AdvancedPad& object = /* obtain a valid instance */;

auto result = object.isButtonPressed(pad, button);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-advancedpad-ispadconnected-1"></a>

## `psyqo::AdvancedPad::isPadConnected`

**Purpose.** Returns the state of a pad.

**Details.** Returns the state of a pad. The state is a boolean value that is `true` if the pad is connected, and `false` otherwise.

**Exact declaration**

```cpp
bool isPadConnected(Pad pad) const
```

- **Declared at:** [line 134](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L134)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |

**Returns.** A boolean value indicating whether the pad is connected.

**Use it when.** Returns the state of a pad. The state is a boolean value that is `true` if the pad is connected, and `false` otherwise.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad

psyqo::AdvancedPad& object = /* obtain a valid instance */;

auto result = object.isPadConnected(pad);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-advancedpad-setonevent-1"></a>

## `psyqo::AdvancedPad::setOnEvent`

**Purpose.** Sets the event callback function.

**Details.** The event callback will be called for each pad-related event, such as pad connection / disconnection, or button press / release. The callback will only be called between frames. Scenes that are calling `setOnEvent` during their `start` method should call `setOnEvent` again in their `teardown` method with the `nullptr` value in order to unregister the event callback cleanly. Only one callback can be registered at a time, so setting a new callback will simply remove the previous one. Careful about what is called from the callback: pushing or popping scenes might call into `setOnEvent` as a result, and could end up corrupting memory as a result of the callback being deleted while being executed.

**Exact declaration**

```cpp
void setOnEvent(eastl::function<void(Event)>&& callback)
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L123)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (Event)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The event callback will be called for each pad-related event, such as pad connection / disconnection, or button press / release. The callback will only be called between frames. Scenes that are calling `setOnEvent` during their `start` method should call `setOnEvent` again in their `teardown` method with the `nullptr` value in order to unregister the event callback cleanly. Only one callback can be registered at a time, so setting a new callback will simply remove the previous one. Careful about what is called from the callback: pushing or popping scenes might call into `setOnEvent` as a result, and could end up corrupting memory as a result of the callback being deleted while being executed.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Event)> && callback

psyqo::AdvancedPad& object = /* obtain a valid instance */;

object.setOnEvent(callback);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-operator-1"></a>

## `psyqo::operator++`

**Purpose.** postfix increment operator

**Exact declaration**

```cpp
inline psyqo::AdvancedPad::Pad operator++(psyqo::AdvancedPad::Pad& pad, int)
```

- **Declared at:** [line 260](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L260)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `psyqo::AdvancedPad::Pad &` | Input/output; inspect the function contract | Value supplied for `pad`. See the exact type and module contract. |
| `arg2` | `int` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** Returns `psyqo::AdvancedPad::Pad`. Check the purpose and failure notes before using the value.

**Use it when.** You need the advancedpad module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// psyqo::AdvancedPad::Pad & pad
// int arg2

auto result = psyqo::operator++(pad, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-operator-2"></a>

## `psyqo::operator++`

**Purpose.** prefix increment operator

**Exact declaration**

```cpp
inline psyqo::AdvancedPad::Pad& operator++(psyqo::AdvancedPad::Pad& pad)
```

- **Declared at:** [line 255](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L255)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `psyqo::AdvancedPad::Pad &` | Input/output; inspect the function contract | Value supplied for `pad`. See the exact type and module contract. |

**Returns.** Returns `psyqo::AdvancedPad::Pad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the advancedpad module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// psyqo::AdvancedPad::Pad & pad

auto result = psyqo::operator++(pad);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-operator-3"></a>

## `psyqo::operator--`

**Purpose.** postfix decrement operator

**Exact declaration**

```cpp
inline psyqo::AdvancedPad::Pad operator--(psyqo::AdvancedPad::Pad& pad, int)
```

- **Declared at:** [line 272](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L272)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `psyqo::AdvancedPad::Pad &` | Input/output; inspect the function contract | Value supplied for `pad`. See the exact type and module contract. |
| `arg2` | `int` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** Returns `psyqo::AdvancedPad::Pad`. Check the purpose and failure notes before using the value.

**Use it when.** You need the advancedpad module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// psyqo::AdvancedPad::Pad & pad
// int arg2

auto result = psyqo::operator--(pad, arg2);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-operator-4"></a>

## `psyqo::operator--`

**Purpose.** prefix decrement operator

**Exact declaration**

```cpp
inline psyqo::AdvancedPad::Pad& operator--(psyqo::AdvancedPad::Pad& pad)
```

- **Declared at:** [line 267](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/advancedpad.hh#L267)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `psyqo::AdvancedPad::Pad &` | Input/output; inspect the function contract | Value supplied for `pad`. See the exact type and module contract. |

**Returns.** Returns `psyqo::AdvancedPad::Pad &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the advancedpad module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/advancedpad.hh"

// Assume these named values have been initialized with valid data:
// psyqo::AdvancedPad::Pad & pad

auto result = psyqo::operator--(pad);
```

**Why choose it.** It provides direct, allocation-conscious access to the advancedpad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
