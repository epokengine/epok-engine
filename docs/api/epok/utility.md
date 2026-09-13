# Epok API: Utility

> **Header:** `"utility.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/utility.hpp)

This module covers the utility module. It documents 23 public callables declared directly in this header.

## Declared types

`epok::Ease`, `epok::Event`, `epok::EventQueue`, `epok::Focus`, `epok::Sequence`, `epok::SequenceStep`, `epok::Tween`

## Callable index

- [`epok::ease`](#epok-ease-1) — Performs `ease` as part of the utility module.
- [`epok::EventQueue::clear`](#epok-eventqueue-clear-1) — Clears clear as part of the utility module.
- [`epok::EventQueue::emit`](#epok-eventqueue-emit-1) — Performs `emit` as part of the utility module.
- [`epok::EventQueue::poll`](#epok-eventqueue-poll-1) — Polls poll as part of the utility module.
- [`epok::EventQueue::size`](#epok-eventqueue-size-1) — Performs `size` as part of the utility module.
- [`epok::Focus::add`](#epok-focus-add-1) — Adds add as part of the utility module.
- [`epok::Focus::clear`](#epok-focus-clear-1) — Clears clear as part of the utility module.
- [`epok::Focus::current`](#epok-focus-current-1) — Performs `current` as part of the utility module.
- [`epok::Focus::move`](#epok-focus-move-1) — Performs `move` as part of the utility module.
- [`epok::Focus::navigate`](#epok-focus-navigate-1) — Performs `navigate` as part of the utility module.
- [`epok::layout_list`](#epok-layout-list-1) — List layout in native HUD pixels; anchoring remains the RectTransform system.
- [`epok::lerp`](#epok-lerp-1) — Performs `lerp` as part of the utility module.
- [`epok::Sequence::advance`](#epok-sequence-advance-1) — Performs `advance` as part of the utility module.
- [`epok::Sequence::cancel`](#epok-sequence-cancel-1) — Performs `cancel` as part of the utility module.
- [`epok::Sequence::playing`](#epok-sequence-playing-1) — Starts ing as part of the utility module.
- [`epok::Sequence::start`](#epok-sequence-start-1) — Starts start as part of the utility module.
- [`epok::Sequence::take_completion`](#epok-sequence-take-completion-1) — Performs `take completion` as part of the utility module.
- [`epok::Tween::advance`](#epok-tween-advance-1) — Performs `advance` as part of the utility module.
- [`epok::Tween::cancel`](#epok-tween-cancel-1) — Performs `cancel` as part of the utility module.
- [`epok::Tween::playing`](#epok-tween-playing-1) — Starts ing as part of the utility module.
- [`epok::Tween::start`](#epok-tween-start-1) — Starts start as part of the utility module.
- [`epok::Tween::take_completion`](#epok-tween-take-completion-1) — Performs `take completion` as part of the utility module.
- [`epok::Tween::value`](#epok-tween-value-1) — Performs `value` as part of the utility module.

<a id="epok-ease-1"></a>

## `epok::ease`

**Purpose.** Performs `ease` as part of the utility module.

**Exact declaration**

```cpp
inline Fixed ease(Fixed t,Ease kind)
```

- **Declared at:** [line 6](../../../runtime/utility.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `Fixed` | Input | Value supplied for `t`. See the exact type and module contract. |
| `kind` | `Ease` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Fixed t
// Ease kind

auto result = epok::ease(t, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-eventqueue-clear-1"></a>

## `epok::EventQueue::clear`

**Purpose.** Clears clear as part of the utility module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 26](../../../runtime/utility.hpp#L26)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::EventQueue& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-eventqueue-emit-1"></a>

## `epok::EventQueue::emit`

**Purpose.** Performs `emit` as part of the utility module.

**Exact declaration**

```cpp
bool emit(Event event)
```

- **Declared at:** [line 24](../../../runtime/utility.hpp#L24)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `Event` | Input | Value supplied for `event`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Event event

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.emit(event);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-eventqueue-poll-1"></a>

## `epok::EventQueue::poll`

**Purpose.** Polls poll as part of the utility module.

**Exact declaration**

```cpp
bool poll(Event& event)
```

- **Declared at:** [line 25](../../../runtime/utility.hpp#L25)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `Event &` | Input/output; inspect the function contract | Value supplied for `event`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Event & event

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.poll(event);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-eventqueue-size-1"></a>

## `epok::EventQueue::size`

**Purpose.** Performs `size` as part of the utility module.

**Exact declaration**

```cpp
size_t size()const
```

- **Declared at:** [line 26](../../../runtime/utility.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-focus-add-1"></a>

## `epok::Focus::add`

**Purpose.** Adds add as part of the utility module.

**Exact declaration**

```cpp
bool add(EntityHandle entity)
```

- **Declared at:** [line 43](../../../runtime/utility.hpp#L43)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `EntityHandle` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle entity

epok::Focus& object = /* obtain a valid instance */;

auto result = object.add(entity);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-focus-clear-1"></a>

## `epok::Focus::clear`

**Purpose.** Clears clear as part of the utility module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 44](../../../runtime/utility.hpp#L44)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Focus& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focus-current-1"></a>

## `epok::Focus::current`

**Purpose.** Performs `current` as part of the utility module.

**Exact declaration**

```cpp
EntityHandle current()const
```

- **Declared at:** [line 45](../../../runtime/utility.hpp#L45)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Focus& object = /* obtain a valid instance */;

auto result = object.current();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focus-move-1"></a>

## `epok::Focus::move`

**Purpose.** Performs `move` as part of the utility module.

**Exact declaration**

```cpp
bool move(int direction,bool wrap=true)
```

- **Declared at:** [line 46](../../../runtime/utility.hpp#L46)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `direction` | `int` | Input | Value supplied for `direction`. See the exact type and module contract. |
| `wrap` | `bool` | Input | Value supplied for `wrap`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int direction
// bool wrap

epok::Focus& object = /* obtain a valid instance */;

auto result = object.move(direction, wrap);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-focus-navigate-1"></a>

## `epok::Focus::navigate`

**Purpose.** Performs `navigate` as part of the utility module.

**Exact declaration**

```cpp
template<size_t N>void navigate(EventQueue<N>& queue,unsigned port=0)
```

- **Declared at:** [line 47](../../../runtime/utility.hpp#L47)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `queue` | `EventQueue<N> &` | Input/output; inspect the function contract | Value supplied for `queue`. See the exact type and module contract. |
| `port` | `unsigned int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// EventQueue<N> & queue
// unsigned int port

epok::Focus& object = /* obtain a valid instance */;

object.navigate<N>(queue, port);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-layout-list-1"></a>

## `epok::layout_list`

**Purpose.** List layout in native HUD pixels; anchoring remains the RectTransform system.

**Exact declaration**

```cpp
inline void layout_list(EntityHandle* children,size_t count,Fixed item_extent,Fixed spacing,bool vertical=true)
```

- **Declared at:** [line 50](../../../runtime/utility.hpp#L50)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `children` | `EntityHandle *` | Input/output; inspect the function contract | Value supplied for `children`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `item_extent` | `Fixed` | Input | Value supplied for `item_extent`. See the exact type and module contract. |
| `spacing` | `Fixed` | Input | Value supplied for `spacing`. See the exact type and module contract. |
| `vertical` | `bool` | Input | Value supplied for `vertical`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle * children
// size_t count
// Fixed item_extent
// Fixed spacing
// bool vertical

epok::layout_list(children, count, item_extent, spacing, vertical);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lerp-1"></a>

## `epok::lerp`

**Purpose.** Performs `lerp` as part of the utility module.

**Exact declaration**

```cpp
inline Fixed lerp(Fixed a,Fixed b,Fixed t)
```

- **Declared at:** [line 4](../../../runtime/utility.hpp#L4)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |
| `t` | `Fixed` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b
// Fixed t

auto result = epok::lerp(a, b, t);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-advance-1"></a>

## `epok::Sequence::advance`

**Purpose.** Performs `advance` as part of the utility module.

**Exact declaration**

```cpp
template<size_t N>void advance(Fixed dt,EventQueue<N>& queue,EntityHandle source={})
```

- **Declared at:** [line 33](../../../runtime/utility.hpp#L33)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `queue` | `EventQueue<N> &` | Input/output; inspect the function contract | Value supplied for `queue`. See the exact type and module contract. |
| `source` | `EntityHandle` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// Fixed dt
// EventQueue<N> & queue
// EntityHandle source

epok::Sequence& object = /* obtain a valid instance */;

object.advance<N>(dt, queue, source);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-cancel-1"></a>

## `epok::Sequence::cancel`

**Purpose.** Performs `cancel` as part of the utility module.

**Exact declaration**

```cpp
void cancel()
```

- **Declared at:** [line 34](../../../runtime/utility.hpp#L34)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Sequence& object = /* obtain a valid instance */;

object.cancel();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-playing-1"></a>

## `epok::Sequence::playing`

**Purpose.** Starts ing as part of the utility module.

**Exact declaration**

```cpp
bool playing()const
```

- **Declared at:** [line 34](../../../runtime/utility.hpp#L34)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Sequence& object = /* obtain a valid instance */;

auto result = object.playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-sequence-start-1"></a>

## `epok::Sequence::start`

**Purpose.** Starts start as part of the utility module.

**Exact declaration**

```cpp
bool start(const SequenceStep* data,size_t length)
```

- **Declared at:** [line 32](../../../runtime/utility.hpp#L32)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const SequenceStep *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `length` | `size_t` | Input | Value supplied for `length`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// const SequenceStep * data
// size_t length

epok::Sequence& object = /* obtain a valid instance */;

auto result = object.start(data, length);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-sequence-take-completion-1"></a>

## `epok::Sequence::take_completion`

**Purpose.** Performs `take completion` as part of the utility module.

**Exact declaration**

```cpp
bool take_completion()
```

- **Declared at:** [line 35](../../../runtime/utility.hpp#L35)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Sequence& object = /* obtain a valid instance */;

auto result = object.take_completion();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-advance-1"></a>

## `epok::Tween::advance`

**Purpose.** Performs `advance` as part of the utility module.

**Exact declaration**

```cpp
Fixed advance(Fixed dt)
```

- **Declared at:** [line 15](../../../runtime/utility.hpp#L15)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `Fixed` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Fixed dt

epok::Tween& object = /* obtain a valid instance */;

auto result = object.advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-cancel-1"></a>

## `epok::Tween::cancel`

**Purpose.** Performs `cancel` as part of the utility module.

**Exact declaration**

```cpp
void cancel()
```

- **Declared at:** [line 16](../../../runtime/utility.hpp#L16)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

object.cancel();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-playing-1"></a>

## `epok::Tween::playing`

**Purpose.** Starts ing as part of the utility module.

**Exact declaration**

```cpp
bool playing()const
```

- **Declared at:** [line 16](../../../runtime/utility.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.playing();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-start-1"></a>

## `epok::Tween::start`

**Purpose.** Starts start as part of the utility module.

**Exact declaration**

```cpp
bool start(Fixed a,Fixed b,Fixed seconds,Ease mode=Ease::Linear)
```

- **Declared at:** [line 13](../../../runtime/utility.hpp#L13)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |
| `seconds` | `Fixed` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `mode` | `Ease` | Input | Value supplied for `mode`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b
// Fixed seconds
// Ease mode

epok::Tween& object = /* obtain a valid instance */;

auto result = object.start(a, b, seconds, mode);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-take-completion-1"></a>

## `epok::Tween::take_completion`

**Purpose.** Performs `take completion` as part of the utility module.

**Exact declaration**

```cpp
bool take_completion()
```

- **Declared at:** [line 17](../../../runtime/utility.hpp#L17)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.take_completion();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-value-1"></a>

## `epok::Tween::value`

**Purpose.** Performs `value` as part of the utility module.

**Exact declaration**

```cpp
Fixed value() const
```

- **Declared at:** [line 14](../../../runtime/utility.hpp#L14)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
