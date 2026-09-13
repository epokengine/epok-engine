# PsyQo API: Simplepad

> **Header:** `"psyqo/simplepad.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/simplepad.hh)

This module covers the simplepad module. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::SimplePad`, `psyqo::SimplePad::Button`, `psyqo::SimplePad::Event`, `psyqo::SimplePad::Event::(unnamed enum at D:\GitProjects\GameEngines\Epok\EpokEngine\third_party\nugget\psyqo\simplepad.hh:71:9)`, `psyqo::SimplePad::Pad`

## Callable index

- [`psyqo::SimplePad::initialize`](#psyqo-simplepad-initialize-1) — Initializes the pads.
- [`psyqo::SimplePad::isButtonPressed`](#psyqo-simplepad-isbuttonpressed-1) — Returns the state of a button.
- [`psyqo::SimplePad::isPadConnected`](#psyqo-simplepad-ispadconnected-1) — Returns the state of a pad.
- [`psyqo::SimplePad::setOnEvent`](#psyqo-simplepad-setonevent-1) — Sets the event callback function.

<a id="psyqo-simplepad-initialize-1"></a>

## `psyqo::SimplePad::initialize`

**Purpose.** Initializes the pads.

**Details.** This will initialize the pads polling by calling the BIOS' interface. This means this method cannot be called from the `prepare` method of the `Application` class, but rather from the `start` method of the root `Scene` object. Also, there can be interference with the BIOS' memory card functions, so this method is explicit to be called in the right order.

**Exact declaration**

```cpp
void initialize()
```

- **Declared at:** [line 86](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/simplepad.hh#L86)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will initialize the pads polling by calling the BIOS' interface. This means this method cannot be called from the `prepare` method of the `Application` class, but rather from the `start` method of the root `Scene` object. Also, there can be interference with the BIOS' memory card functions, so this method is explicit to be called in the right order.

**Usage pattern**

```cpp
#include "psyqo/simplepad.hh"

psyqo::SimplePad& object = /* obtain a valid instance */;

object.initialize();
```

**Why choose it.** It provides direct, allocation-conscious access to the simplepad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-simplepad-isbuttonpressed-1"></a>

## `psyqo::SimplePad::isButtonPressed`

**Purpose.** Returns the state of a button.

**Details.** Returns the state of a button. The state is a boolean value that is `true` if the button is pressed, and `false` otherwise.

**Exact declaration**

```cpp
bool isButtonPressed(Pad pad, Button button) const
```

- **Declared at:** [line 129](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/simplepad.hh#L129)
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
#include "psyqo/simplepad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad
// Button button

psyqo::SimplePad& object = /* obtain a valid instance */;

auto result = object.isButtonPressed(pad, button);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-simplepad-ispadconnected-1"></a>

## `psyqo::SimplePad::isPadConnected`

**Purpose.** Returns the state of a pad.

**Details.** Returns the state of a pad. The state is a boolean value that is `true` if the pad is connected, and `false` otherwise.

**Exact declaration**

```cpp
bool isPadConnected(Pad pad) const
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/simplepad.hh#L117)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | The pad to query. |

**Returns.** A boolean value indicating whether the pad is connected.

**Use it when.** Returns the state of a pad. The state is a boolean value that is `true` if the pad is connected, and `false` otherwise.

**Usage pattern**

```cpp
#include "psyqo/simplepad.hh"

// Assume these named values have been initialized with valid data:
// Pad pad

psyqo::SimplePad& object = /* obtain a valid instance */;

auto result = object.isPadConnected(pad);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-simplepad-setonevent-1"></a>

## `psyqo::SimplePad::setOnEvent`

**Purpose.** Sets the event callback function.

**Details.** The event callback will be called for each pad-related event, such as pad connection / disconnection, or button press / release. The callback will only be called between frames. Scenes that are calling `setOnEvent` during their `start` method should call `setOnEvent` again in their `teardown` method with the `nullptr` value in order to unregister the event callback cleanly. Only one callback can be registered at a time, so setting a new callback will simply remove the previous one. Careful about what is called from the callback: pushing or popping scenes might call into `setOnEvent` as a result, and could end up corrupting memory as a result of the callback being deleted while being executed.

**Exact declaration**

```cpp
void setOnEvent(eastl::function<void(Event)>&& callback)
```

- **Declared at:** [line 106](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/simplepad.hh#L106)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `eastl::function<void (Event)> &&` | Consumed or moved input | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The event callback will be called for each pad-related event, such as pad connection / disconnection, or button press / release. The callback will only be called between frames. Scenes that are calling `setOnEvent` during their `start` method should call `setOnEvent` again in their `teardown` method with the `nullptr` value in order to unregister the event callback cleanly. Only one callback can be registered at a time, so setting a new callback will simply remove the previous one. Careful about what is called from the callback: pushing or popping scenes might call into `setOnEvent` as a result, and could end up corrupting memory as a result of the callback being deleted while being executed.

**Usage pattern**

```cpp
#include "psyqo/simplepad.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Event)> && callback

psyqo::SimplePad& object = /* obtain a valid instance */;

object.setOnEvent(callback);
```

**Why choose it.** It provides direct, allocation-conscious access to the simplepad module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
