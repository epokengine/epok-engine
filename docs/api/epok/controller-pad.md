# Epok API: Controller Pad

> **Header:** `"controller_pad.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/controller_pad.hpp)

This module covers SIO0 controller polling and DualShock configuration. It documents 11 public callables declared directly in this header.

## Declared types

`epok::ControllerPad`, `epok::ControllerPad::Button`, `epok::ControllerPad::Pad`, `epok::ControllerPad::TransferTrace`

## Callable index

- [`epok::ControllerPad::getAdc`](#epok-controllerpad-getadc-1) — Returns adc as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::getPadType`](#epok-controllerpad-getpadtype-1) — Returns pad type as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::initialize`](#epok-controllerpad-initialize-1) — Performs `initialize` as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::isButtonPressed`](#epok-controllerpad-isbuttonpressed-1) — Reports whether button pressed as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::isPadConnected`](#epok-controllerpad-ispadconnected-1) — Reports whether pad connected as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::rumbleProbing`](#epok-controllerpad-rumbleprobing-1) — Performs `rumble probing` as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::rumbleSupported`](#epok-controllerpad-rumblesupported-1) — Performs `rumble supported` as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::setRumble`](#epok-controllerpad-setrumble-1) — Sets rumble as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::stopRumble`](#epok-controllerpad-stoprumble-1) — Stops rumble as part of SIO0 controller polling and DualShock configuration.
- [`epok::ControllerPad::synchronize`](#epok-controllerpad-synchronize-1) — A long card transaction or blocked game frame may let the controller's own watchdog forget its configuration.
- [`epok::ControllerPad::trace`](#epok-controllerpad-trace-1) — Performs `trace` as part of SIO0 controller polling and DualShock configuration.

<a id="epok-controllerpad-getadc-1"></a>

## `epok::ControllerPad::getAdc`

**Purpose.** Returns adc as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
uint8_t getAdc(Pad pad,unsigned axis)const
```

- **Declared at:** [line 207](../../../runtime/controller_pad.hpp#L207)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | Value supplied for `pad`. See the exact type and module contract. |
| `axis` | `unsigned int` | Input | Value supplied for `axis`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// Pad pad
// unsigned int axis

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.getAdc(pad, axis);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-getpadtype-1"></a>

## `epok::ControllerPad::getPadType`

**Purpose.** Returns pad type as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
uint8_t getPadType(Pad pad)const
```

- **Declared at:** [line 206](../../../runtime/controller_pad.hpp#L206)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | Value supplied for `pad`. See the exact type and module contract. |

**Returns.** Returns `uint8_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// Pad pad

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.getPadType(pad);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-initialize-1"></a>

## `epok::ControllerPad::initialize`

**Purpose.** Performs `initialize` as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
void initialize(bool enable_rumble=false)
```

- **Declared at:** [line 197](../../../runtime/controller_pad.hpp#L197)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `enable_rumble` | `bool` | Input | Value supplied for `enable_rumble`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// bool enable_rumble

epok::ControllerPad& object = /* obtain a valid instance */;

object.initialize(enable_rumble);
```

**Why choose it.** It provides direct, allocation-conscious access to SIO0 controller polling and DualShock configuration. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-isbuttonpressed-1"></a>

## `epok::ControllerPad::isButtonPressed`

**Purpose.** Reports whether button pressed as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
bool isButtonPressed(Pad pad,Button button)const
```

- **Declared at:** [line 205](../../../runtime/controller_pad.hpp#L205)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | Value supplied for `pad`. See the exact type and module contract. |
| `button` | `Button` | Input | Value supplied for `button`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// Pad pad
// Button button

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.isButtonPressed(pad, button);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-controllerpad-ispadconnected-1"></a>

## `epok::ControllerPad::isPadConnected`

**Purpose.** Reports whether pad connected as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
bool isPadConnected(Pad pad)const
```

- **Declared at:** [line 204](../../../runtime/controller_pad.hpp#L204)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `Pad` | Input | Value supplied for `pad`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// Pad pad

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.isPadConnected(pad);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-controllerpad-rumbleprobing-1"></a>

## `epok::ControllerPad::rumbleProbing`

**Purpose.** Performs `rumble probing` as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
bool rumbleProbing(unsigned p)const
```

- **Declared at:** [line 209](../../../runtime/controller_pad.hpp#L209)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.rumbleProbing(p);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-controllerpad-rumblesupported-1"></a>

## `epok::ControllerPad::rumbleSupported`

**Purpose.** Performs `rumble supported` as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
bool rumbleSupported(unsigned p)const
```

- **Declared at:** [line 208](../../../runtime/controller_pad.hpp#L208)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.rumbleSupported(p);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-controllerpad-setrumble-1"></a>

## `epok::ControllerPad::setRumble`

**Purpose.** Sets rumble as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
void setRumble(unsigned p,uint8_t large,bool small,uint32_t remaining_us)
```

- **Declared at:** [line 211](../../../runtime/controller_pad.hpp#L211)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |
| `large` | `uint8_t` | Input | Value supplied for `large`. See the exact type and module contract. |
| `small` | `bool` | Input | Value supplied for `small`. See the exact type and module contract. |
| `remaining_us` | `uint32_t` | Input | Value supplied for `remaining_us`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p
// uint8_t large
// bool small
// uint32_t remaining_us

epok::ControllerPad& object = /* obtain a valid instance */;

object.setRumble(p, large, small, remaining_us);
```

**Why choose it.** It provides direct, allocation-conscious access to SIO0 controller polling and DualShock configuration. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-stoprumble-1"></a>

## `epok::ControllerPad::stopRumble`

**Purpose.** Stops rumble as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
void stopRumble()
```

- **Declared at:** [line 218](../../../runtime/controller_pad.hpp#L218)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

epok::ControllerPad& object = /* obtain a valid instance */;

object.stopRumble();
```

**Why choose it.** It provides direct, allocation-conscious access to SIO0 controller polling and DualShock configuration. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-synchronize-1"></a>

## `epok::ControllerPad::synchronize`

**Purpose.** A long card transaction or blocked game frame may let the controller's own watchdog forget its configuration.

**Details.** Re-probe rather than keeping a stale "ready" flag when polling resumes. The clock is GPU microseconds.

**Exact declaration**

```cpp
void synchronize(uint32_t now)
```

- **Declared at:** [line 190](../../../runtime/controller_pad.hpp#L190)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `now` | `uint32_t` | Input | Value supplied for `now`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Re-probe rather than keeping a stale "ready" flag when polling resumes. The clock is GPU microseconds.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t now

epok::ControllerPad& object = /* obtain a valid instance */;

object.synchronize(now);
```

**Why choose it.** It provides direct, allocation-conscious access to SIO0 controller polling and DualShock configuration. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-controllerpad-trace-1"></a>

## `epok::ControllerPad::trace`

**Purpose.** Performs `trace` as part of SIO0 controller polling and DualShock configuration.

**Exact declaration**

```cpp
const TransferTrace& trace(unsigned p=0)const
```

- **Declared at:** [line 210](../../../runtime/controller_pad.hpp#L210)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `unsigned int` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `const TransferTrace &`. Check the purpose and failure notes before using the value.

**Use it when.** You need SIO0 controller polling and DualShock configuration and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "controller_pad.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int p

epok::ControllerPad& object = /* obtain a valid instance */;

auto result = object.trace(p);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
