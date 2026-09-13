# Epok API: Blueprint Runtime

> **Header:** `"blueprint_runtime.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/blueprint_runtime.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 63 public callables declared directly in this header.

## Declared types

`epok::bp::Breakpoint`, `epok::bp::Continuation`, `epok::bp::Continuations`, `epok::bp::Debugger`, `epok::bp::Timeline`, `epok::bp::TimelineKey`, `epok::bp::TimelineSample`, `epok::bp::Trace`, `epok::bp::TraceKind`, `epok::bp::TraceRing`, `epok::bp::Vector`

## Callable index

- [`epok::bp::add`](#epok-bp-add-1) — Adds add as part of compiled Blueprint execution and object interaction.
- [`epok::bp::add`](#epok-bp-add-2) — Adds add as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::advance`](#epok-bp-continuations-advance-1) — Performs `advance` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::cancel`](#epok-bp-continuations-cancel-1) — Performs `cancel` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::cancel_all`](#epok-bp-continuations-cancel-all-1) — Performs `cancel all` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::cancel_owner`](#epok-bp-continuations-cancel-owner-1) — Performs `cancel owner` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::clear`](#epok-bp-continuations-clear-1) — Clears clear as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::delay`](#epok-bp-continuations-delay-1) — Performs `delay` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::poll`](#epok-bp-continuations-poll-1) — Polls poll as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::reset`](#epok-bp-continuations-reset-1) — Resets reset as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::signal`](#epok-bp-continuations-signal-1) — Performs `signal` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::size`](#epok-bp-continuations-size-1) — Performs `size` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Continuations::wait_external`](#epok-bp-continuations-wait-external-1) — External producers only mark an existing frame ready.
- [`epok::bp::Continuations::waiting`](#epok-bp-continuations-waiting-1) — Performs `waiting` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::add`](#epok-bp-debugger-add-1) — Adds add as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::checkpoint`](#epok-bp-debugger-checkpoint-1) — Performs `checkpoint` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::clear_breakpoints`](#epok-bp-debugger-clear-breakpoints-1) — Clears breakpoints as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::location`](#epok-bp-debugger-location-1) — Performs `location` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::pause`](#epok-bp-debugger-pause-1) — Pauses pause as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::paused`](#epok-bp-debugger-paused-1) — Pauses d as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::reset`](#epok-bp-debugger-reset-1) — Resets reset as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::resume`](#epok-bp-debugger-resume-1) — Resumes resume as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Debugger::step`](#epok-bp-debugger-step-1) — Performs `step` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::div`](#epok-bp-div-1) — Performs `div` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::div`](#epok-bp-div-2) — Performs `div` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::div`](#epok-bp-div-3) — Performs `div` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::from_int`](#epok-bp-from-int-1) — Performs `from int` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::iadd`](#epok-bp-iadd-1) — Performs `iadd` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::idiv`](#epok-bp-idiv-1) — Performs `idiv` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::imod`](#epok-bp-imod-1) — Performs `imod` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::imul`](#epok-bp-imul-1) — Performs `imul` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::increment`](#epok-bp-increment-1) — Performs `increment` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::ineg`](#epok-bp-ineg-1) — Performs `ineg` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::interpolate`](#epok-bp-interpolate-1) — Performs `interpolate` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::isub`](#epok-bp-isub-1) — Reports whether ub as part of compiled Blueprint execution and object interaction.
- [`epok::bp::mul`](#epok-bp-mul-1) — Performs `mul` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::mul`](#epok-bp-mul-2) — Performs `mul` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::mul`](#epok-bp-mul-3) — Performs `mul` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::neg`](#epok-bp-neg-1) — Performs `neg` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::same_owner`](#epok-bp-same-owner-1) — Performs `same owner` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::saturate`](#epok-bp-saturate-1) — Blueprint arithmetic saturates, divides by zero to zero, and rounds division toward zero.
- [`epok::bp::sub`](#epok-bp-sub-1) — Performs `sub` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::sub`](#epok-bp-sub-2) — Performs `sub` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::advance`](#epok-bp-timeline-advance-1) — Performs `advance` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::cancel`](#epok-bp-timeline-cancel-1) — Performs `cancel` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::configure`](#epok-bp-timeline-configure-1) — Performs `configure` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::play`](#epok-bp-timeline-play-1) — Starts play as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::playing`](#epok-bp-timeline-playing-1) — Starts ing as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::reset`](#epok-bp-timeline-reset-1) — Resets reset as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Timeline::value`](#epok-bp-timeline-value-1) — Performs `value` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::to_int`](#epok-bp-to-int-1) — Performs `to int` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::trace`](#epok-bp-trace-1) — Performs `trace` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TraceRing::clear`](#epok-bp-tracering-clear-1) — Clears clear as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TraceRing::poll`](#epok-bp-tracering-poll-1) — Polls poll as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TraceRing::push`](#epok-bp-tracering-push-1) — Pushes push as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TraceRing::size`](#epok-bp-tracering-size-1) — Performs `size` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::uadd`](#epok-bp-uadd-1) — Performs `uadd` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::udiv`](#epok-bp-udiv-1) — Performs `udiv` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::umod`](#epok-bp-umod-1) — Performs `umod` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::umul`](#epok-bp-umul-1) — Performs `umul` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::usub`](#epok-bp-usub-1) — Performs `usub` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Vector::operator[]`](#epok-bp-vector-operator-1) — Performs `operator []` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::Vector::operator[]`](#epok-bp-vector-operator-2) — Performs `operator []` as part of compiled Blueprint execution and object interaction.

<a id="epok-bp-add-1"></a>

## `epok::bp::add`

**Purpose.** Adds add as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed add(Fixed a, Fixed b)
```

- **Declared at:** [line 28](../../../runtime/blueprint_runtime.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b

auto result = epok::bp::add(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-add-2"></a>

## `epok::bp::add`

**Purpose.** Adds add as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> add(const Vector<Size>& a, const Vector<Size>& b)
```

- **Declared at:** [line 47](../../../runtime/blueprint_runtime.hpp#L47)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vector<Size> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// const Vector<Size> & b

epok::bp& object = /* obtain a valid instance */;

auto result = object.add<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-continuations-advance-1"></a>

## `epok::bp::Continuations::advance`

**Purpose.** Performs `advance` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void advance(Fixed dt, uint32_t scene_generation = 0, bool paused = false)
```

- **Declared at:** [line 127](../../../runtime/blueprint_runtime.hpp#L127)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// uint32_t scene_generation
// bool paused

epok::bp::Continuations& object = /* obtain a valid instance */;

object.advance(dt, scene_generation, paused);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-cancel-1"></a>

## `epok::bp::Continuations::cancel`

**Purpose.** Performs `cancel` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void cancel(uint32_t node)
```

- **Declared at:** [line 159](../../../runtime/blueprint_runtime.hpp#L159)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node

epok::bp::Continuations& object = /* obtain a valid instance */;

object.cancel(node);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-cancel-all-1"></a>

## `epok::bp::Continuations::cancel_all`

**Purpose.** Performs `cancel all` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void cancel_all()
```

- **Declared at:** [line 169](../../../runtime/blueprint_runtime.hpp#L169)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Continuations& object = /* obtain a valid instance */;

object.cancel_all();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-cancel-owner-1"></a>

## `epok::bp::Continuations::cancel_owner`

**Purpose.** Performs `cancel owner` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void cancel_owner(EntityHandle owner)
```

- **Declared at:** [line 154](../../../runtime/blueprint_runtime.hpp#L154)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

epok::bp::Continuations& object = /* obtain a valid instance */;

object.cancel_owner(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-clear-1"></a>

## `epok::bp::Continuations::clear`

**Purpose.** Clears clear as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 170](../../../runtime/blueprint_runtime.hpp#L170)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Continuations& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-delay-1"></a>

## `epok::bp::Continuations::delay`

**Purpose.** Performs `delay` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool delay(uint32_t node, Fixed seconds, EntityHandle owner, uint32_t scene_generation = 0)
```

- **Declared at:** [line 92](../../../runtime/blueprint_runtime.hpp#L92)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node
// Fixed seconds
// EntityHandle owner
// uint32_t scene_generation

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.delay(node, seconds, owner, scene_generation);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-poll-1"></a>

## `epok::bp::Continuations::poll`

**Purpose.** Polls poll as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool poll(Continuation& result)
```

- **Declared at:** [line 142](../../../runtime/blueprint_runtime.hpp#L142)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `Continuation &` | Input/output; inspect the function contract | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Continuation & result

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.poll(result);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-continuations-reset-1"></a>

## `epok::bp::Continuations::reset`

**Purpose.** Resets reset as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 164](../../../runtime/blueprint_runtime.hpp#L164)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Continuations& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-signal-1"></a>

## `epok::bp::Continuations::signal`

**Purpose.** Performs `signal` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool signal(uint32_t node, uint32_t scene_generation = 0)
```

- **Declared at:** [line 116](../../../runtime/blueprint_runtime.hpp#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node
// uint32_t scene_generation

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.signal(node, scene_generation);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-size-1"></a>

## `epok::bp::Continuations::size`

**Purpose.** Performs `size` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
size_t size() const
```

- **Declared at:** [line 171](../../../runtime/blueprint_runtime.hpp#L171)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-wait-external-1"></a>

## `epok::bp::Continuations::wait_external`

**Purpose.** External producers only mark an existing frame ready.

**Details.** They never invoke callbacks or allocate another listener/continuation. Negative remaining time is internal state and cannot be authored through Delay.

**Exact declaration**

```cpp
bool wait_external(uint32_t node, EntityHandle owner, uint32_t scene_generation = 0)
```

- **Declared at:** [line 104](../../../runtime/blueprint_runtime.hpp#L104)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** They never invoke callbacks or allocate another listener/continuation. Negative remaining time is internal state and cannot be authored through Delay.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node
// EntityHandle owner
// uint32_t scene_generation

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.wait_external(node, owner, scene_generation);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-continuations-waiting-1"></a>

## `epok::bp::Continuations::waiting`

**Purpose.** Performs `waiting` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool waiting(uint32_t node) const
```

- **Declared at:** [line 112](../../../runtime/blueprint_runtime.hpp#L112)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node

epok::bp::Continuations& object = /* obtain a valid instance */;

auto result = object.waiting(node);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-add-1"></a>

## `epok::bp::Debugger::add`

**Purpose.** Adds add as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool add(Breakpoint point)
```

- **Declared at:** [line 281](../../../runtime/blueprint_runtime.hpp#L281)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `point` | `Breakpoint` | Input | Value supplied for `point`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Breakpoint point

epok::bp::Debugger& object = /* obtain a valid instance */;

auto result = object.add(point);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-checkpoint-1"></a>

## `epok::bp::Debugger::checkpoint`

**Purpose.** Performs `checkpoint` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool checkpoint(uint32_t class_id, uint32_t node_id, EntityHandle owner)
```

- **Declared at:** [line 288](../../../runtime/blueprint_runtime.hpp#L288)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `class_id` | `uint32_t` | Input | Value supplied for `class_id`. See the exact type and module contract. |
| `node_id` | `uint32_t` | Input | Value supplied for `node_id`. See the exact type and module contract. |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t class_id
// uint32_t node_id
// EntityHandle owner

epok::bp::Debugger& object = /* obtain a valid instance */;

auto result = object.checkpoint(class_id, node_id, owner);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-clear-breakpoints-1"></a>

## `epok::bp::Debugger::clear_breakpoints`

**Purpose.** Clears breakpoints as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void clear_breakpoints()
```

- **Declared at:** [line 287](../../../runtime/blueprint_runtime.hpp#L287)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

object.clear_breakpoints();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-location-1"></a>

## `epok::bp::Debugger::location`

**Purpose.** Performs `location` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
const Trace* location() const
```

- **Declared at:** [line 311](../../../runtime/blueprint_runtime.hpp#L311)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const Trace *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

auto result = object.location();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-pause-1"></a>

## `epok::bp::Debugger::pause`

**Purpose.** Pauses pause as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void pause()
```

- **Declared at:** [line 307](../../../runtime/blueprint_runtime.hpp#L307)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

object.pause();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-paused-1"></a>

## `epok::bp::Debugger::paused`

**Purpose.** Pauses d as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool paused() const
```

- **Declared at:** [line 310](../../../runtime/blueprint_runtime.hpp#L310)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

auto result = object.paused();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-reset-1"></a>

## `epok::bp::Debugger::reset`

**Purpose.** Resets reset as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 312](../../../runtime/blueprint_runtime.hpp#L312)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-resume-1"></a>

## `epok::bp::Debugger::resume`

**Purpose.** Resumes resume as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void resume()
```

- **Declared at:** [line 308](../../../runtime/blueprint_runtime.hpp#L308)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

object.resume();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-debugger-step-1"></a>

## `epok::bp::Debugger::step`

**Purpose.** Performs `step` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void step()
```

- **Declared at:** [line 309](../../../runtime/blueprint_runtime.hpp#L309)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Debugger& object = /* obtain a valid instance */;

object.step();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-div-1"></a>

## `epok::bp::div`

**Purpose.** Performs `div` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed div(Fixed a, Fixed b)
```

- **Declared at:** [line 31](../../../runtime/blueprint_runtime.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b

auto result = epok::bp::div(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-div-2"></a>

## `epok::bp::div`

**Purpose.** Performs `div` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> div(const Vector<Size>& a, Fixed b)
```

- **Declared at:** [line 62](../../../runtime/blueprint_runtime.hpp#L62)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// Fixed b

epok::bp& object = /* obtain a valid instance */;

auto result = object.div<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-div-3"></a>

## `epok::bp::div`

**Purpose.** Performs `div` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> div(const Vector<Size>& a, const Vector<Size>& b)
```

- **Declared at:** [line 56](../../../runtime/blueprint_runtime.hpp#L56)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vector<Size> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// const Vector<Size> & b

epok::bp& object = /* obtain a valid instance */;

auto result = object.div<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-from-int-1"></a>

## `epok::bp::from_int`

**Purpose.** Performs `from int` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed from_int(int32_t value)
```

- **Declared at:** [line 35](../../../runtime/blueprint_runtime.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value

auto result = epok::bp::from_int(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-iadd-1"></a>

## `epok::bp::iadd`

**Purpose.** Performs `iadd` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t iadd(int32_t a, int32_t b)
```

- **Declared at:** [line 14](../../../runtime/blueprint_runtime.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::bp::iadd(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-idiv-1"></a>

## `epok::bp::idiv`

**Purpose.** Performs `idiv` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t idiv(int32_t a, int32_t b)
```

- **Declared at:** [line 17](../../../runtime/blueprint_runtime.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::bp::idiv(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-imod-1"></a>

## `epok::bp::imod`

**Purpose.** Performs `imod` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t imod(int32_t a, int32_t b)
```

- **Declared at:** [line 18](../../../runtime/blueprint_runtime.hpp#L18)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::bp::imod(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-imul-1"></a>

## `epok::bp::imul`

**Purpose.** Performs `imul` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t imul(int32_t a, int32_t b)
```

- **Declared at:** [line 16](../../../runtime/blueprint_runtime.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::bp::imul(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-increment-1"></a>

## `epok::bp::increment`

**Purpose.** Performs `increment` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void increment(uint32_t& value)
```

- **Declared at:** [line 68](../../../runtime/blueprint_runtime.hpp#L68)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t & value

epok::bp::increment(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-ineg-1"></a>

## `epok::bp::ineg`

**Purpose.** Performs `ineg` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t ineg(int32_t a)
```

- **Declared at:** [line 19](../../../runtime/blueprint_runtime.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a

auto result = epok::bp::ineg(a);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-interpolate-1"></a>

## `epok::bp::interpolate`

**Purpose.** Performs `interpolate` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed interpolate(Fixed a, Fixed b, Fixed alpha)
```

- **Declared at:** [line 37](../../../runtime/blueprint_runtime.hpp#L37)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |
| `alpha` | `Fixed` | Input | Value supplied for `alpha`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b
// Fixed alpha

auto result = epok::bp::interpolate(a, b, alpha);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-isub-1"></a>

## `epok::bp::isub`

**Purpose.** Reports whether ub as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr int32_t isub(int32_t a, int32_t b)
```

- **Declared at:** [line 15](../../../runtime/blueprint_runtime.hpp#L15)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::bp::isub(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-mul-1"></a>

## `epok::bp::mul`

**Purpose.** Performs `mul` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed mul(Fixed a, Fixed b)
```

- **Declared at:** [line 30](../../../runtime/blueprint_runtime.hpp#L30)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b

auto result = epok::bp::mul(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-mul-2"></a>

## `epok::bp::mul`

**Purpose.** Performs `mul` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> mul(const Vector<Size>& a, Fixed b)
```

- **Declared at:** [line 59](../../../runtime/blueprint_runtime.hpp#L59)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// Fixed b

epok::bp& object = /* obtain a valid instance */;

auto result = object.mul<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-mul-3"></a>

## `epok::bp::mul`

**Purpose.** Performs `mul` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> mul(const Vector<Size>& a, const Vector<Size>& b)
```

- **Declared at:** [line 53](../../../runtime/blueprint_runtime.hpp#L53)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vector<Size> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// const Vector<Size> & b

epok::bp& object = /* obtain a valid instance */;

auto result = object.mul<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-neg-1"></a>

## `epok::bp::neg`

**Purpose.** Performs `neg` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed neg(Fixed a)
```

- **Declared at:** [line 34](../../../runtime/blueprint_runtime.hpp#L34)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a

auto result = epok::bp::neg(a);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-same-owner-1"></a>

## `epok::bp::same_owner`

**Purpose.** Performs `same owner` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr bool same_owner(EntityHandle a, EntityHandle b)
```

- **Declared at:** [line 65](../../../runtime/blueprint_runtime.hpp#L65)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `EntityHandle` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `EntityHandle` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle a
// EntityHandle b

auto result = epok::bp::same_owner(a, b);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-saturate-1"></a>

## `epok::bp::saturate`

**Purpose.** Blueprint arithmetic saturates, divides by zero to zero, and rounds division toward zero.

**Details.** Intermediates are wide enough for every signed 32-bit operand; neither signed overflow nor negative shifts are part of the language contract.

**Exact declaration**

```cpp
constexpr int32_t saturate(int64_t value)
```

- **Declared at:** [line 11](../../../runtime/blueprint_runtime.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Intermediates are wide enough for every signed 32-bit operand; neither signed overflow nor negative shifts are part of the language contract.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value

auto result = epok::bp::saturate(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-sub-1"></a>

## `epok::bp::sub`

**Purpose.** Performs `sub` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Fixed sub(Fixed a, Fixed b)
```

- **Declared at:** [line 29](../../../runtime/blueprint_runtime.hpp#L29)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b

auto result = epok::bp::sub(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-sub-2"></a>

## `epok::bp::sub`

**Purpose.** Performs `sub` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<size_t Size> inline Vector<Size> sub(const Vector<Size>& a, const Vector<Size>& b)
```

- **Declared at:** [line 50](../../../runtime/blueprint_runtime.hpp#L50)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vector<Size> &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vector<Size> &` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `Vector<Size>`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Size

// Assume these named values have been initialized with valid data:
// const Vector<Size> & a
// const Vector<Size> & b

epok::bp& object = /* obtain a valid instance */;

auto result = object.sub<Size>(a, b);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-timeline-advance-1"></a>

## `epok::bp::Timeline::advance`

**Purpose.** Performs `advance` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
TimelineSample advance(Fixed dt, uint32_t scene_generation = 0, bool paused = false)
```

- **Declared at:** [line 218](../../../runtime/blueprint_runtime.hpp#L218)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |
| `paused` | `bool` | Input | Value supplied for `paused`. See the exact type and module contract. |

**Returns.** Returns `TimelineSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt
// uint32_t scene_generation
// bool paused

epok::bp::Timeline& object = /* obtain a valid instance */;

auto result = object.advance(dt, scene_generation, paused);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-timeline-cancel-1"></a>

## `epok::bp::Timeline::cancel`

**Purpose.** Performs `cancel` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void cancel()
```

- **Declared at:** [line 233](../../../runtime/blueprint_runtime.hpp#L233)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Timeline& object = /* obtain a valid instance */;

object.cancel();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-timeline-configure-1"></a>

## `epok::bp::Timeline::configure`

**Purpose.** Performs `configure` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool configure(const TimelineKey* data, size_t length)
```

- **Declared at:** [line 193](../../../runtime/blueprint_runtime.hpp#L193)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const TimelineKey *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `length` | `size_t` | Input | Value supplied for `length`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// const TimelineKey * data
// size_t length

epok::bp::Timeline& object = /* obtain a valid instance */;

auto result = object.configure(data, length);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-timeline-play-1"></a>

## `epok::bp::Timeline::play`

**Purpose.** Starts play as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool play(EntityHandle source, uint32_t scene_generation = 0, bool loop = false)
```

- **Declared at:** [line 202](../../../runtime/blueprint_runtime.hpp#L202)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `EntityHandle` | Input | Value supplied for `source`. See the exact type and module contract. |
| `scene_generation` | `uint32_t` | Input | Value supplied for `scene_generation`. See the exact type and module contract. |
| `loop` | `bool` | Input | Value supplied for `loop`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle source
// uint32_t scene_generation
// bool loop

epok::bp::Timeline& object = /* obtain a valid instance */;

auto result = object.play(source, scene_generation, loop);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-timeline-playing-1"></a>

## `epok::bp::Timeline::playing`

**Purpose.** Starts ing as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool playing() const
```

- **Declared at:** [line 235](../../../runtime/blueprint_runtime.hpp#L235)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Timeline& object = /* obtain a valid instance */;

auto result = object.playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-timeline-reset-1"></a>

## `epok::bp::Timeline::reset`

**Purpose.** Resets reset as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 234](../../../runtime/blueprint_runtime.hpp#L234)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Timeline& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-timeline-value-1"></a>

## `epok::bp::Timeline::value`

**Purpose.** Performs `value` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
Fixed value() const
```

- **Declared at:** [line 207](../../../runtime/blueprint_runtime.hpp#L207)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::Timeline& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-to-int-1"></a>

## `epok::bp::to_int`

**Purpose.** Performs `to int` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline int32_t to_int(Fixed value)
```

- **Declared at:** [line 36](../../../runtime/blueprint_runtime.hpp#L36)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

auto result = epok::bp::to_int(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-trace-1"></a>

## `epok::bp::trace`

**Purpose.** Performs `trace` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void trace(uint32_t, uint32_t, EntityHandle, TraceKind = TraceKind::Enter, int32_t = 0)
```

- **Declared at:** [line 325](../../../runtime/blueprint_runtime.hpp#L325)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint32_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `uint32_t` | Input | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `EntityHandle` | Input | Value supplied for `arg3`. See the exact type and module contract. |
| `arg4` | `TraceKind` | Input | Value supplied for `arg4`. See the exact type and module contract. |
| `arg5` | `int32_t` | Input | Value supplied for `arg5`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t arg1
// uint32_t arg2
// EntityHandle arg3
// TraceKind arg4
// int32_t arg5

epok::bp::trace(arg1, arg2, arg3, arg4, arg5);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-tracering-clear-1"></a>

## `epok::bp::TraceRing::clear`

**Purpose.** Clears clear as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 259](../../../runtime/blueprint_runtime.hpp#L259)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::TraceRing& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-tracering-poll-1"></a>

## `epok::bp::TraceRing::poll`

**Purpose.** Polls poll as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool poll(Trace& entry)
```

- **Declared at:** [line 255](../../../runtime/blueprint_runtime.hpp#L255)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `Trace &` | Input/output; inspect the function contract | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Trace & entry

epok::bp::TraceRing& object = /* obtain a valid instance */;

auto result = object.poll(entry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-tracering-push-1"></a>

## `epok::bp::TraceRing::push`

**Purpose.** Pushes push as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
bool push(Trace entry)
```

- **Declared at:** [line 251](../../../runtime/blueprint_runtime.hpp#L251)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entry` | `Trace` | Input | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// Trace entry

epok::bp::TraceRing& object = /* obtain a valid instance */;

auto result = object.push(entry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-tracering-size-1"></a>

## `epok::bp::TraceRing::size`

**Purpose.** Performs `size` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
size_t size() const
```

- **Declared at:** [line 260](../../../runtime/blueprint_runtime.hpp#L260)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

epok::bp::TraceRing& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-uadd-1"></a>

## `epok::bp::uadd`

**Purpose.** Performs `uadd` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr uint32_t uadd(uint32_t a, uint32_t b)
```

- **Declared at:** [line 20](../../../runtime/blueprint_runtime.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::bp::uadd(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-udiv-1"></a>

## `epok::bp::udiv`

**Purpose.** Performs `udiv` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr uint32_t udiv(uint32_t a, uint32_t b)
```

- **Declared at:** [line 26](../../../runtime/blueprint_runtime.hpp#L26)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::bp::udiv(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-umod-1"></a>

## `epok::bp::umod`

**Purpose.** Performs `umod` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr uint32_t umod(uint32_t a, uint32_t b)
```

- **Declared at:** [line 27](../../../runtime/blueprint_runtime.hpp#L27)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::bp::umod(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-umul-1"></a>

## `epok::bp::umul`

**Purpose.** Performs `umul` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr uint32_t umul(uint32_t a, uint32_t b)
```

- **Declared at:** [line 22](../../../runtime/blueprint_runtime.hpp#L22)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::bp::umul(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-usub-1"></a>

## `epok::bp::usub`

**Purpose.** Performs `usub` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
constexpr uint32_t usub(uint32_t a, uint32_t b)
```

- **Declared at:** [line 21](../../../runtime/blueprint_runtime.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `uint32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `uint32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t a
// uint32_t b

auto result = epok::bp::usub(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-vector-operator-1"></a>

## `epok::bp::Vector::operator[]`

**Purpose.** Performs `operator []` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
Fixed& operator[](size_t index)
```

- **Declared at:** [line 44](../../../runtime/blueprint_runtime.hpp#L44)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `Fixed &`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::bp::Vector& object = /* obtain a valid instance */;

auto result = object.operator[](index);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-bp-vector-operator-2"></a>

## `epok::bp::Vector::operator[]`

**Purpose.** Performs `operator []` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
const Fixed& operator[](size_t index) const
```

- **Declared at:** [line 45](../../../runtime/blueprint_runtime.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `const Fixed &`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_runtime.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::bp::Vector& object = /* obtain a valid instance */;

auto result = object.operator[](index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
