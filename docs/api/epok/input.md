# Epok API: Input

> **Header:** `"input.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/input.hpp)

This module covers controller sampling and simulation-tick input edges. It documents 12 public callables declared directly in this header.

## Declared types

`epok::Button`, `epok::Input`

## Callable index

- [`epok::Input::begin_tick`](#epok-input-begin-tick-1) — Begins tick as part of controller sampling and simulation-tick input edges.
- [`epok::Input::connected`](#epok-input-connected-1) — Performs `connected` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::discard_edges`](#epok-input-discard-edges-1) — Performs `discard edges` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::end_tick`](#epok-input-end-tick-1) — Ends tick as part of controller sampling and simulation-tick input edges.
- [`epok::Input::frame_pressed`](#epok-input-frame-pressed-1) — Performs `frame pressed` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::frame_released`](#epok-input-frame-released-1) — Performs `frame released` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::held`](#epok-input-held-1) — Performs `held` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::poll`](#epok-input-poll-1) — Polls poll as part of controller sampling and simulation-tick input edges.
- [`epok::Input::pressed`](#epok-input-pressed-1) — Performs `pressed` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::released`](#epok-input-released-1) — Performs `released` as part of controller sampling and simulation-tick input edges.
- [`epok::Input::reset`](#epok-input-reset-1) — Resets reset as part of controller sampling and simulation-tick input edges.
- [`epok::Input::sample`](#epok-input-sample-1) — Sampling accumulates edges until a simulation tick consumes them.

<a id="epok-input-begin-tick-1"></a>

## `epok::Input::begin_tick`

**Purpose.** Begins tick as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
void begin_tick()
```

- **Declared at:** [line 40](../../../runtime/input.hpp#L40)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

epok::Input& object = /* obtain a valid instance */;

object.begin_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to controller sampling and simulation-tick input edges. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-input-connected-1"></a>

## `epok::Input::connected`

**Purpose.** Performs `connected` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool connected(unsigned port=0) const
```

- **Declared at:** [line 15](../../../runtime/input.hpp#L15)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.connected(port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-discard-edges-1"></a>

## `epok::Input::discard_edges`

**Purpose.** Performs `discard edges` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
void discard_edges()
```

- **Declared at:** [line 44](../../../runtime/input.hpp#L44)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

epok::Input& object = /* obtain a valid instance */;

object.discard_edges();
```

**Why choose it.** It provides direct, allocation-conscious access to controller sampling and simulation-tick input edges. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-input-end-tick-1"></a>

## `epok::Input::end_tick`

**Purpose.** Ends tick as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
void end_tick()
```

- **Declared at:** [line 43](../../../runtime/input.hpp#L43)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

epok::Input& object = /* obtain a valid instance */;

object.end_tick();
```

**Why choose it.** It provides direct, allocation-conscious access to controller sampling and simulation-tick input edges. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-input-frame-pressed-1"></a>

## `epok::Input::frame_pressed`

**Purpose.** Performs `frame pressed` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool frame_pressed(Button b,unsigned port=0) const
```

- **Declared at:** [line 19](../../../runtime/input.hpp#L19)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `Button` | Input | Value supplied for `b`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// Button b
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.frame_pressed(b, port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-frame-released-1"></a>

## `epok::Input::frame_released`

**Purpose.** Performs `frame released` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool frame_released(Button b,unsigned port=0) const
```

- **Declared at:** [line 20](../../../runtime/input.hpp#L20)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `Button` | Input | Value supplied for `b`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// Button b
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.frame_released(b, port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-held-1"></a>

## `epok::Input::held`

**Purpose.** Performs `held` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool held(Button b,unsigned port=0) const
```

- **Declared at:** [line 16](../../../runtime/input.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `Button` | Input | Value supplied for `b`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// Button b
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.held(b, port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-poll-1"></a>

## `epok::Input::poll`

**Purpose.** Polls poll as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
template<class PadReader> void poll(const PadReader& pad,unsigned second_port=1)
```

- **Declared at:** [line 31](../../../runtime/input.hpp#L31)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pad` | `const PadReader &` | Input | Value supplied for `pad`. See the exact type and module contract. |
| `second_port` | `unsigned int` | Input | Value supplied for `second_port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// PadReader

// Assume these named values have been initialized with valid data:
// const PadReader & pad
// unsigned int second_port

epok::Input& object = /* obtain a valid instance */;

object.poll<PadReader>(pad, second_port);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-input-pressed-1"></a>

## `epok::Input::pressed`

**Purpose.** Performs `pressed` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool pressed(Button b,unsigned port=0) const
```

- **Declared at:** [line 17](../../../runtime/input.hpp#L17)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `Button` | Input | Value supplied for `b`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// Button b
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.pressed(b, port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-released-1"></a>

## `epok::Input::released`

**Purpose.** Performs `released` as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
bool released(Button b,unsigned port=0) const
```

- **Declared at:** [line 18](../../../runtime/input.hpp#L18)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `b` | `Button` | Input | Value supplied for `b`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// Button b
// unsigned int port

epok::Input& object = /* obtain a valid instance */;

auto result = object.released(b, port);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-input-reset-1"></a>

## `epok::Input::reset`

**Purpose.** Resets reset as part of controller sampling and simulation-tick input edges.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 45](../../../runtime/input.hpp#L45)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need controller sampling and simulation-tick input edges and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "input.hpp"

epok::Input& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to controller sampling and simulation-tick input edges. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-input-sample-1"></a>

## `epok::Input::sample`

**Purpose.** Sampling accumulates edges until a simulation tick consumes them.

**Details.** A render with no tick cannot lose a quick press/release; catch-up ticks don't repeat it.

**Exact declaration**

```cpp
void sample(unsigned port,bool connected,uint16_t held)
```

- **Declared at:** [line 23](../../../runtime/input.hpp#L23)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |
| `connected` | `bool` | Input | Value supplied for `connected`. See the exact type and module contract. |
| `held` | `uint16_t` | Input | Value supplied for `held`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** A render with no tick cannot lose a quick press/release; catch-up ticks don't repeat it.

**Usage pattern**

```cpp
#include "input.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int port
// bool connected
// uint16_t held

epok::Input& object = /* obtain a valid instance */;

object.sample(port, connected, held);
```

**Why choose it.** It provides direct, allocation-conscious access to controller sampling and simulation-tick input edges. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
