# Epok API: Utility

> **Header:** `"utility.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/utility.hpp)

This module covers the utility module. It documents 64 public callables declared directly in this header.

## Declared types

`epok::Ease`, `epok::EventQueue`, `epok::EventQueueMutation`, `epok::EventQueuePoll`, `epok::Focus`, `epok::FocusLibrary`, `epok::FocusSnapshot`, `epok::GameplayEventQueue4`, `epok::GameplayEventSample`, `epok::GameplayTweenState`, `epok::QueueEvent`, `epok::Sequence`, `epok::SequenceStep`, `epok::Tween`, `epok::TweenAdvanceSample`, `epok::TweenLoop`, `epok::UtilityLibrary`

## Callable index

- [`epok::ease`](#epok-ease-1) — Q12 easing.
- [`epok::EventQueue::clear`](#epok-eventqueue-clear-1) — Clears clear as part of the utility module.
- [`epok::EventQueue::emit`](#epok-eventqueue-emit-1) — Performs `emit` as part of the utility module.
- [`epok::EventQueue::poll`](#epok-eventqueue-poll-1) — Polls poll as part of the utility module.
- [`epok::EventQueue::restore`](#epok-eventqueue-restore-1) — Performs `restore` as part of the utility module.
- [`epok::EventQueue::size`](#epok-eventqueue-size-1) — Performs `size` as part of the utility module.
- [`epok::EventQueue::snapshot`](#epok-eventqueue-snapshot-1) — Performs `snapshot` as part of the utility module.
- [`epok::Focus::add`](#epok-focus-add-1) — Adds add as part of the utility module.
- [`epok::Focus::clear`](#epok-focus-clear-1) — Clears clear as part of the utility module.
- [`epok::Focus::current`](#epok-focus-current-1) — Performs `current` as part of the utility module.
- [`epok::Focus::move`](#epok-focus-move-1) — Performs `move` as part of the utility module.
- [`epok::Focus::navigate`](#epok-focus-navigate-1) — Performs `navigate` as part of the utility module.
- [`epok::Focus::size`](#epok-focus-size-1) — Performs `size` as part of the utility module.
- [`epok::FocusLibrary::add`](#epok-focuslibrary-add-1) — Adds add as part of the utility module.
- [`epok::FocusLibrary::clear`](#epok-focuslibrary-clear-1) — Clears clear as part of the utility module.
- [`epok::FocusLibrary::layout`](#epok-focuslibrary-layout-1) — Performs `layout` as part of the utility module.
- [`epok::FocusLibrary::move`](#epok-focuslibrary-move-1) — Performs `move` as part of the utility module.
- [`epok::FocusLibrary::navigate`](#epok-focuslibrary-navigate-1) — Performs `navigate` as part of the utility module.
- [`epok::FocusLibrary::snapshot`](#epok-focuslibrary-snapshot-1) — Performs `snapshot` as part of the utility module.
- [`epok::gameplay_event`](#epok-gameplay-event-1) — Performs `gameplay event` as part of the utility module.
- [`epok::gameplay_event`](#epok-gameplay-event-2) — Performs `gameplay event` as part of the utility module.
- [`epok::gameplay_event_queue`](#epok-gameplay-event-queue-1) — Performs `gameplay event queue` as part of the utility module.
- [`epok::gameplay_event_queue`](#epok-gameplay-event-queue-2) — Performs `gameplay event queue` as part of the utility module.
- [`epok::gameplay_focus`](#epok-gameplay-focus-1) — Performs `gameplay focus` as part of the utility module.
- [`epok::gameplay_tween`](#epok-gameplay-tween-1) — Performs `gameplay tween` as part of the utility module.
- [`epok::gameplay_tween_state`](#epok-gameplay-tween-state-1) — Performs `gameplay tween state` as part of the utility module.
- [`epok::layout_list`](#epok-layout-list-1) — List layout in native HUD pixels; anchoring remains the RectTransform system.
- [`epok::lerp`](#epok-lerp-1) — Performs `lerp` as part of the utility module.
- [`epok::Sequence::advance`](#epok-sequence-advance-1) — Performs `advance` as part of the utility module.
- [`epok::Sequence::cancel`](#epok-sequence-cancel-1) — Performs `cancel` as part of the utility module.
- [`epok::Sequence::playing`](#epok-sequence-playing-1) — Starts ing as part of the utility module.
- [`epok::Sequence::start`](#epok-sequence-start-1) — Starts start as part of the utility module.
- [`epok::Sequence::take_completion`](#epok-sequence-take-completion-1) — Performs `take completion` as part of the utility module.
- [`epok::Tween::advance`](#epok-tween-advance-1) — Whole completed legs are counted by one division, so a long delta cannot spin here and an unbounded ping-pong keeps the correct leg parity.
- [`epok::Tween::alpha`](#epok-tween-alpha-1) — The eased interpolation parameter.
- [`epok::Tween::cancel`](#epok-tween-cancel-1) — Performs `cancel` as part of the utility module.
- [`epok::Tween::completion_pending`](#epok-tween-completion-pending-1) — Performs `completion pending` as part of the utility module.
- [`epok::Tween::cycles_remaining`](#epok-tween-cycles-remaining-1) — Performs `cycles remaining` as part of the utility module.
- [`epok::Tween::delay_value`](#epok-tween-delay-value-1) — Performs `delay value` as part of the utility module.
- [`epok::Tween::duration_value`](#epok-tween-duration-value-1) — Performs `duration value` as part of the utility module.
- [`epok::Tween::ease_kind`](#epok-tween-ease-kind-1) — Performs `ease kind` as part of the utility module.
- [`epok::Tween::elapsed_value`](#epok-tween-elapsed-value-1) — Performs `elapsed value` as part of the utility module.
- [`epok::Tween::end_value`](#epok-tween-end-value-1) — Ends value as part of the utility module.
- [`epok::Tween::loop_mode`](#epok-tween-loop-mode-1) — Performs `loop mode` as part of the utility module.
- [`epok::Tween::playing`](#epok-tween-playing-1) — Starts ing as part of the utility module.
- [`epok::Tween::ratio`](#epok-tween-ratio-1) — Performs `ratio` as part of the utility module.
- [`epok::Tween::restore`](#epok-tween-restore-1) — Performs `restore` as part of the utility module.
- [`epok::Tween::reversed_leg`](#epok-tween-reversed-leg-1) — Performs `reversed leg` as part of the utility module.
- [`epok::Tween::schedule`](#epok-tween-schedule-1) — `wait` holds the first leg back without shortening it.
- [`epok::Tween::start`](#epok-tween-start-1) — Starts start as part of the utility module.
- [`epok::Tween::start_value`](#epok-tween-start-value-1) — Starts value as part of the utility module.
- [`epok::Tween::take_completion`](#epok-tween-take-completion-1) — Performs `take completion` as part of the utility module.
- [`epok::Tween::value`](#epok-tween-value-1) — Performs `value` as part of the utility module.
- [`epok::utility_actor_data`](#epok-utility-actor-data-1) — Performs `utility actor data` as part of the utility module.
- [`epok::utility_actor_id`](#epok-utility-actor-id-1) — Performs `utility actor id` as part of the utility module.
- [`epok::UtilityLibrary::ease`](#epok-utilitylibrary-ease-1) — The easing catalogue on its own, for curves applied to something that is not a tween.
- [`epok::UtilityLibrary::event_queue_clear`](#epok-utilitylibrary-event-queue-clear-1) — Performs `event queue clear` as part of the utility module.
- [`epok::UtilityLibrary::event_queue_emit`](#epok-utilitylibrary-event-queue-emit-1) — Performs `event queue emit` as part of the utility module.
- [`epok::UtilityLibrary::event_queue_poll`](#epok-utilitylibrary-event-queue-poll-1) — Performs `event queue poll` as part of the utility module.
- [`epok::UtilityLibrary::tween_advance`](#epok-utilitylibrary-tween-advance-1) — Performs `tween advance` as part of the utility module.
- [`epok::UtilityLibrary::tween_cancel`](#epok-utilitylibrary-tween-cancel-1) — Performs `tween cancel` as part of the utility module.
- [`epok::UtilityLibrary::tween_schedule`](#epok-utilitylibrary-tween-schedule-1) — The full plan: `delay_seconds` holds the first leg back, `loop` and `legs` decide the replay, and `legs` 0 loops without end.
- [`epok::UtilityLibrary::tween_start`](#epok-utilitylibrary-tween-start-1) — Performs `tween start` as part of the utility module.
- [`epok::UtilityLibrary::tween_value`](#epok-utilitylibrary-tween-value-1) — Performs `tween value` as part of the utility module.

<a id="epok-ease-1"></a>

## `epok::ease`

**Purpose.** Q12 easing.

**Details.** Every curve returns exactly 0 at t=0 and exactly 4096 at t=1, which is what lets Tween::advance land on `to` the moment elapsed reaches duration. The In/Out halves double the argument before taking the power rather than scaling a truncated power afterwards: scaling afterwards costs an order of magnitude of accuracy by the fifth power. SmoothStep, InQuad and OutQuad keep their original expressions so already authored tweens keep their exact shape; OutQuad therefore truncates the whole product instead of only the square.

**Exact declaration**

```cpp
inline Fixed ease(Fixed t,Ease kind)
```

- **Declared at:** [line 24](../../../runtime/utility.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `int` | Input | Value supplied for `t`. See the exact type and module contract. |
| `kind` | `Ease` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Every curve returns exactly 0 at t=0 and exactly 4096 at t=1, which is what lets Tween::advance land on `to` the moment elapsed reaches duration. The In/Out halves double the argument before taking the power rather than scaling a truncated power afterwards: scaling afterwards costs an order of magnitude of accuracy by the fifth power. SmoothStep, InQuad and OutQuad keep their original expressions so already authored tweens keep their exact shape; OutQuad therefore truncates the whole product instead of only the square.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int t
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

- **Declared at:** [line 108](../../../runtime/utility.hpp#L108)
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
bool emit(QueueEvent event)
```

- **Declared at:** [line 106](../../../runtime/utility.hpp#L106)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `QueueEvent` | Input | Value supplied for `event`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// QueueEvent event

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
bool poll(QueueEvent& event)
```

- **Declared at:** [line 107](../../../runtime/utility.hpp#L107)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `event` | `QueueEvent &` | Input/output; inspect the function contract | Value supplied for `event`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// QueueEvent & event

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.poll(event);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-eventqueue-restore-1"></a>

## `epok::EventQueue::restore`

**Purpose.** Performs `restore` as part of the utility module.

**Exact declaration**

```cpp
void restore(const QueueEvent* input,size_t length,uint32_t dropped_count=0)
```

- **Declared at:** [line 110](../../../runtime/utility.hpp#L110)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `input` | `const QueueEvent *` | Input | Value supplied for `input`. See the exact type and module contract. |
| `length` | `int` | Input | Value supplied for `length`. See the exact type and module contract. |
| `dropped_count` | `int` | Input | Value supplied for `dropped_count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// const QueueEvent * input
// int length
// int dropped_count

epok::EventQueue& object = /* obtain a valid instance */;

object.restore(input, length, dropped_count);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-eventqueue-size-1"></a>

## `epok::EventQueue::size`

**Purpose.** Performs `size` as part of the utility module.

**Exact declaration**

```cpp
size_t size()const
```

- **Declared at:** [line 108](../../../runtime/utility.hpp#L108)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-eventqueue-snapshot-1"></a>

## `epok::EventQueue::snapshot`

**Purpose.** Performs `snapshot` as part of the utility module.

**Exact declaration**

```cpp
size_t snapshot(QueueEvent* output,size_t capacity)const
```

- **Declared at:** [line 109](../../../runtime/utility.hpp#L109)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `output` | `QueueEvent *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `capacity` | `int` | Input | Value supplied for `capacity`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// QueueEvent * output
// int capacity

epok::EventQueue& object = /* obtain a valid instance */;

auto result = object.snapshot(output, capacity);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-focus-add-1"></a>

## `epok::Focus::add`

**Purpose.** Adds add as part of the utility module.

**Exact declaration**

```cpp
bool add(DataHandle entity)
```

- **Declared at:** [line 127](../../../runtime/utility.hpp#L127)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `DataHandle` | Input | Value supplied for `entity`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle entity

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

- **Declared at:** [line 128](../../../runtime/utility.hpp#L128)
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
DataHandle current()const
```

- **Declared at:** [line 130](../../../runtime/utility.hpp#L130)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 131](../../../runtime/utility.hpp#L131)
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

- **Declared at:** [line 132](../../../runtime/utility.hpp#L132)
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

<a id="epok-focus-size-1"></a>

## `epok::Focus::size`

**Purpose.** Performs `size` as part of the utility module.

**Exact declaration**

```cpp
size_t size()const
```

- **Declared at:** [line 129](../../../runtime/utility.hpp#L129)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Focus& object = /* obtain a valid instance */;

auto result = object.size();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focuslibrary-add-1"></a>

## `epok::FocusLibrary::add`

**Purpose.** Adds add as part of the utility module.

**Exact declaration**

```cpp
static bool add(ObjectId actor)
```

- **Declared at:** [line 167](../../../runtime/utility.hpp#L167)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `ObjectId` | Input | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId actor

auto result = epok::FocusLibrary::add(actor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-focuslibrary-clear-1"></a>

## `epok::FocusLibrary::clear`

**Purpose.** Clears clear as part of the utility module.

**Exact declaration**

```cpp
static void clear()
```

- **Declared at:** [line 166](../../../runtime/utility.hpp#L166)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::FocusLibrary::clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focuslibrary-layout-1"></a>

## `epok::FocusLibrary::layout`

**Purpose.** Performs `layout` as part of the utility module.

**Exact declaration**

```cpp
static void layout(ObjectBatch8 actors,Fixed item_extent,Fixed spacing,bool vertical)
```

- **Declared at:** [line 171](../../../runtime/utility.hpp#L171)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actors` | `ObjectBatch8` | Input | Value supplied for `actors`. See the exact type and module contract. |
| `item_extent` | `int` | Input | Value supplied for `item_extent`. See the exact type and module contract. |
| `spacing` | `int` | Input | Value supplied for `spacing`. See the exact type and module contract. |
| `vertical` | `bool` | Input | Value supplied for `vertical`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// ObjectBatch8 actors
// int item_extent
// int spacing
// bool vertical

epok::FocusLibrary::layout(actors, item_extent, spacing, vertical);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focuslibrary-move-1"></a>

## `epok::FocusLibrary::move`

**Purpose.** Performs `move` as part of the utility module.

**Exact declaration**

```cpp
static bool move(int32_t direction,bool wrap)
```

- **Declared at:** [line 168](../../../runtime/utility.hpp#L168)
- **Kind:** `cxx method`; qualifiers: `static`

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

auto result = epok::FocusLibrary::move(direction, wrap);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-focuslibrary-navigate-1"></a>

## `epok::FocusLibrary::navigate`

**Purpose.** Performs `navigate` as part of the utility module.

**Exact declaration**

```cpp
static GameplayEventQueue4 navigate(uint32_t port)
```

- **Declared at:** [line 169](../../../runtime/utility.hpp#L169)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `port` | `int` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `GameplayEventQueue4`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int port

auto result = epok::FocusLibrary::navigate(port);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-focuslibrary-snapshot-1"></a>

## `epok::FocusLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the utility module.

**Exact declaration**

```cpp
static FocusSnapshot snapshot()
```

- **Declared at:** [line 170](../../../runtime/utility.hpp#L170)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `FocusSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

auto result = epok::FocusLibrary::snapshot();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-event-1"></a>

## `epok::gameplay_event`

**Purpose.** Performs `gameplay event` as part of the utility module.

**Exact declaration**

```cpp
inline GameplayEventSample gameplay_event(QueueEvent value)
```

- **Declared at:** [line 160](../../../runtime/utility.hpp#L160)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `QueueEvent` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `GameplayEventSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// QueueEvent value

auto result = epok::gameplay_event(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-event-2"></a>

## `epok::gameplay_event`

**Purpose.** Performs `gameplay event` as part of the utility module.

**Exact declaration**

```cpp
inline QueueEvent gameplay_event(GameplayEventSample value)
```

- **Declared at:** [line 161](../../../runtime/utility.hpp#L161)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `GameplayEventSample` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `QueueEvent`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayEventSample value

auto result = epok::gameplay_event(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-event-queue-1"></a>

## `epok::gameplay_event_queue`

**Purpose.** Performs `gameplay event queue` as part of the utility module.

**Exact declaration**

```cpp
inline EventQueue<4> gameplay_event_queue(GameplayEventQueue4 state)
```

- **Declared at:** [line 163](../../../runtime/utility.hpp#L163)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayEventQueue4` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `EventQueue<4>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayEventQueue4 state

auto result = epok::gameplay_event_queue(state);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-gameplay-event-queue-2"></a>

## `epok::gameplay_event_queue`

**Purpose.** Performs `gameplay event queue` as part of the utility module.

**Exact declaration**

```cpp
inline GameplayEventQueue4 gameplay_event_queue(EventQueue<4>& value)
```

- **Declared at:** [line 162](../../../runtime/utility.hpp#L162)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `EventQueue<4> &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `GameplayEventQueue4`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// EventQueue<4> & value

auto result = epok::gameplay_event_queue(value);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gameplay-focus-1"></a>

## `epok::gameplay_focus`

**Purpose.** Performs `gameplay focus` as part of the utility module.

**Exact declaration**

```cpp
inline Focus<16>& gameplay_focus()
```

- **Declared at:** [line 157](../../../runtime/utility.hpp#L157)
- **Kind:** `function decl`

**Returns.** Returns `Focus<16> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

auto result = epok::gameplay_focus();
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-tween-1"></a>

## `epok::gameplay_tween`

**Purpose.** Performs `gameplay tween` as part of the utility module.

**Exact declaration**

```cpp
inline Tween gameplay_tween(GameplayTweenState state)
```

- **Declared at:** [line 159](../../../runtime/utility.hpp#L159)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayTweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `Tween`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayTweenState state

auto result = epok::gameplay_tween(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-gameplay-tween-state-1"></a>

## `epok::gameplay_tween_state`

**Purpose.** Performs `gameplay tween state` as part of the utility module.

**Exact declaration**

```cpp
inline GameplayTweenState gameplay_tween_state(const Tween& value)
```

- **Declared at:** [line 158](../../../runtime/utility.hpp#L158)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `const Tween &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `GameplayTweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// const Tween & value

auto result = epok::gameplay_tween_state(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-layout-list-1"></a>

## `epok::layout_list`

**Purpose.** List layout in native HUD pixels; anchoring remains the RectTransform system.

**Exact declaration**

```cpp
inline void layout_list(DataHandle* children,size_t count,Fixed item_extent,Fixed spacing,bool vertical=true)
```

- **Declared at:** [line 135](../../../runtime/utility.hpp#L135)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `children` | `DataHandle *` | Input/output; inspect the function contract | Value supplied for `children`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `item_extent` | `int` | Input | Value supplied for `item_extent`. See the exact type and module contract. |
| `spacing` | `int` | Input | Value supplied for `spacing`. See the exact type and module contract. |
| `vertical` | `bool` | Input | Value supplied for `vertical`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle * children
// int count
// int item_extent
// int spacing
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

- **Declared at:** [line 7](../../../runtime/utility.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int` | Input | Value supplied for `b`. See the exact type and module contract. |
| `t` | `int` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int a
// int b
// int t

auto result = epok::lerp(a, b, t);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequence-advance-1"></a>

## `epok::Sequence::advance`

**Purpose.** Performs `advance` as part of the utility module.

**Exact declaration**

```cpp
template<size_t N>void advance(Fixed dt,EventQueue<N>& queue,DataHandle source={})
```

- **Declared at:** [line 117](../../../runtime/utility.hpp#L117)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `int` | Input | Value supplied for `dt`. See the exact type and module contract. |
| `queue` | `EventQueue<N> &` | Input/output; inspect the function contract | Value supplied for `queue`. See the exact type and module contract. |
| `source` | `DataHandle` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// N

// Assume these named values have been initialized with valid data:
// int dt
// EventQueue<N> & queue
// DataHandle source

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

- **Declared at:** [line 118](../../../runtime/utility.hpp#L118)
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

- **Declared at:** [line 118](../../../runtime/utility.hpp#L118)
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

- **Declared at:** [line 116](../../../runtime/utility.hpp#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const SequenceStep *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `length` | `int` | Input | Value supplied for `length`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// const SequenceStep * data
// int length

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

- **Declared at:** [line 119](../../../runtime/utility.hpp#L119)
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

**Purpose.** Whole completed legs are counted by one division, so a long delta cannot spin here and an unbounded ping-pong keeps the correct leg parity.

**Exact declaration**

```cpp
Fixed advance(Fixed dt)
```

- **Declared at:** [line 76](../../../runtime/utility.hpp#L76)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dt` | `int` | Input | Value supplied for `dt`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int dt

epok::Tween& object = /* obtain a valid instance */;

auto result = object.advance(dt);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-alpha-1"></a>

## `epok::Tween::alpha`

**Purpose.** The eased interpolation parameter.

**Details.** A reversed leg mirrors the curve about both axes, which is the same shape with the endpoints swapped and keeps both turnarounds exact. Vector tweens read this value and lerp each component with it, so they agree with three scalar tweens raw unit for raw unit; a zero duration answers 1 so both forms report `to` whatever leg a record claims.

**Exact declaration**

```cpp
Fixed alpha() const
```

- **Declared at:** [line 72](../../../runtime/utility.hpp#L72)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** A reversed leg mirrors the curve about both axes, which is the same shape with the endpoints swapped and keeps both turnarounds exact. Vector tweens read this value and lerp each component with it, so they agree with three scalar tweens raw unit for raw unit; a zero duration answers 1 so both forms report `to` whatever leg a record claims.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.alpha();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-cancel-1"></a>

## `epok::Tween::cancel`

**Purpose.** Performs `cancel` as part of the utility module.

**Exact declaration**

```cpp
void cancel()
```

- **Declared at:** [line 90](../../../runtime/utility.hpp#L90)
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

<a id="epok-tween-completion-pending-1"></a>

## `epok::Tween::completion_pending`

**Purpose.** Performs `completion pending` as part of the utility module.

**Exact declaration**

```cpp
bool completion_pending()const
```

- **Declared at:** [line 94](../../../runtime/utility.hpp#L94)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.completion_pending();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-cycles-remaining-1"></a>

## `epok::Tween::cycles_remaining`

**Purpose.** Performs `cycles remaining` as part of the utility module.

**Exact declaration**

```cpp
uint32_t cycles_remaining()const
```

- **Declared at:** [line 96](../../../runtime/utility.hpp#L96)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.cycles_remaining();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-delay-value-1"></a>

## `epok::Tween::delay_value`

**Purpose.** Performs `delay value` as part of the utility module.

**Exact declaration**

```cpp
Fixed delay_value()const
```

- **Declared at:** [line 95](../../../runtime/utility.hpp#L95)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.delay_value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-duration-value-1"></a>

## `epok::Tween::duration_value`

**Purpose.** Performs `duration value` as part of the utility module.

**Exact declaration**

```cpp
Fixed duration_value()const
```

- **Declared at:** [line 93](../../../runtime/utility.hpp#L93)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.duration_value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-ease-kind-1"></a>

## `epok::Tween::ease_kind`

**Purpose.** Performs `ease kind` as part of the utility module.

**Exact declaration**

```cpp
Ease ease_kind()const
```

- **Declared at:** [line 94](../../../runtime/utility.hpp#L94)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Ease`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.ease_kind();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-elapsed-value-1"></a>

## `epok::Tween::elapsed_value`

**Purpose.** Performs `elapsed value` as part of the utility module.

**Exact declaration**

```cpp
Fixed elapsed_value()const
```

- **Declared at:** [line 93](../../../runtime/utility.hpp#L93)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.elapsed_value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-end-value-1"></a>

## `epok::Tween::end_value`

**Purpose.** Ends value as part of the utility module.

**Exact declaration**

```cpp
Fixed end_value()const
```

- **Declared at:** [line 92](../../../runtime/utility.hpp#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.end_value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-loop-mode-1"></a>

## `epok::Tween::loop_mode`

**Purpose.** Performs `loop mode` as part of the utility module.

**Exact declaration**

```cpp
TweenLoop loop_mode()const
```

- **Declared at:** [line 95](../../../runtime/utility.hpp#L95)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `TweenLoop`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.loop_mode();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-playing-1"></a>

## `epok::Tween::playing`

**Purpose.** Starts ing as part of the utility module.

**Exact declaration**

```cpp
bool playing()const
```

- **Declared at:** [line 90](../../../runtime/utility.hpp#L90)
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

<a id="epok-tween-ratio-1"></a>

## `epok::Tween::ratio`

**Purpose.** Performs `ratio` as part of the utility module.

**Exact declaration**

```cpp
Fixed ratio() const
```

- **Declared at:** [line 66](../../../runtime/utility.hpp#L66)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.ratio();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-restore-1"></a>

## `epok::Tween::restore`

**Purpose.** Performs `restore` as part of the utility module.

**Exact declaration**

```cpp
void restore(Fixed a,Fixed b,Fixed seconds,Fixed progress,Ease mode,bool active,bool completion,Fixed wait=0.0,TweenLoop loop=TweenLoop::None,uint32_t legs=1,bool reverse=false)
```

- **Declared at:** [line 97](../../../runtime/utility.hpp#L97)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int` | Input | Value supplied for `b`. See the exact type and module contract. |
| `seconds` | `int` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `progress` | `int` | Input | Value supplied for `progress`. See the exact type and module contract. |
| `mode` | `Ease` | Input | Value supplied for `mode`. See the exact type and module contract. |
| `active` | `bool` | Input | Value supplied for `active`. See the exact type and module contract. |
| `completion` | `bool` | Input | Value supplied for `completion`. See the exact type and module contract. |
| `wait` | `int` | Input | Value supplied for `wait`. See the exact type and module contract. |
| `loop` | `TweenLoop` | Input | Value supplied for `loop`. See the exact type and module contract. |
| `legs` | `int` | Input | Value supplied for `legs`. See the exact type and module contract. |
| `reverse` | `bool` | Input | Value supplied for `reverse`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int a
// int b
// int seconds
// int progress
// Ease mode
// bool active
// bool completion
// int wait
// TweenLoop loop
// int legs
// bool reverse

epok::Tween& object = /* obtain a valid instance */;

object.restore(a, b, seconds, progress, mode, active, completion, wait, loop, legs, reverse);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-reversed-leg-1"></a>

## `epok::Tween::reversed_leg`

**Purpose.** Performs `reversed leg` as part of the utility module.

**Exact declaration**

```cpp
bool reversed_leg()const
```

- **Declared at:** [line 96](../../../runtime/utility.hpp#L96)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.reversed_leg();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-schedule-1"></a>

## `epok::Tween::schedule`

**Purpose.** `wait` holds the first leg back without shortening it.

**Details.** `legs` counts the duration spans to play and 0 asks for an unbounded replay; without a loop mode exactly one leg plays whatever `legs` says. A zero duration is already finished on arrival, so it ignores the wait rather than deferring `to`.

**Exact declaration**

```cpp
bool schedule(Fixed a,Fixed b,Fixed seconds,Ease mode,Fixed wait,TweenLoop loop,uint32_t legs)
```

- **Declared at:** [line 61](../../../runtime/utility.hpp#L61)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int` | Input | Value supplied for `b`. See the exact type and module contract. |
| `seconds` | `int` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `mode` | `Ease` | Input | Value supplied for `mode`. See the exact type and module contract. |
| `wait` | `int` | Input | Value supplied for `wait`. See the exact type and module contract. |
| `loop` | `TweenLoop` | Input | Value supplied for `loop`. See the exact type and module contract. |
| `legs` | `int` | Input | Value supplied for `legs`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** `legs` counts the duration spans to play and 0 asks for an unbounded replay; without a loop mode exactly one leg plays whatever `legs` says. A zero duration is already finished on arrival, so it ignores the wait rather than deferring `to`.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int a
// int b
// int seconds
// Ease mode
// int wait
// TweenLoop loop
// int legs

epok::Tween& object = /* obtain a valid instance */;

auto result = object.schedule(a, b, seconds, mode, wait, loop, legs);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-tween-start-1"></a>

## `epok::Tween::start`

**Purpose.** Starts start as part of the utility module.

**Exact declaration**

```cpp
bool start(Fixed a,Fixed b,Fixed seconds,Ease mode=Ease::Linear)
```

- **Declared at:** [line 56](../../../runtime/utility.hpp#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int` | Input | Value supplied for `b`. See the exact type and module contract. |
| `seconds` | `int` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `mode` | `Ease` | Input | Value supplied for `mode`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int a
// int b
// int seconds
// Ease mode

epok::Tween& object = /* obtain a valid instance */;

auto result = object.start(a, b, seconds, mode);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-tween-start-value-1"></a>

## `epok::Tween::start_value`

**Purpose.** Starts value as part of the utility module.

**Exact declaration**

```cpp
Fixed start_value()const
```

- **Declared at:** [line 92](../../../runtime/utility.hpp#L92)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.start_value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-tween-take-completion-1"></a>

## `epok::Tween::take_completion`

**Purpose.** Performs `take completion` as part of the utility module.

**Exact declaration**

```cpp
bool take_completion()
```

- **Declared at:** [line 91](../../../runtime/utility.hpp#L91)
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

- **Declared at:** [line 73](../../../runtime/utility.hpp#L73)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

epok::Tween& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utility-actor-data-1"></a>

## `epok::utility_actor_data`

**Purpose.** Performs `utility actor data` as part of the utility module.

**Exact declaration**

```cpp
inline ActorData* utility_actor_data(ObjectId id)
```

- **Declared at:** [line 155](../../../runtime/utility.hpp#L155)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::utility_actor_data(id);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utility-actor-id-1"></a>

## `epok::utility_actor_id`

**Purpose.** Performs `utility actor id` as part of the utility module.

**Exact declaration**

```cpp
inline ObjectId utility_actor_id(DataHandle value)
```

- **Declared at:** [line 156](../../../runtime/utility.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `DataHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle value

auto result = epok::utility_actor_id(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilitylibrary-ease-1"></a>

## `epok::UtilityLibrary::ease`

**Purpose.** The easing catalogue on its own, for curves applied to something that is not a tween.

**Details.** `t` is clamped to 0..1 and both endpoints are exact.

**Exact declaration**

```cpp
static Fixed ease(Fixed t,Ease easing)
```

- **Declared at:** [line 184](../../../runtime/utility.hpp#L184)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `int` | Input | Value supplied for `t`. See the exact type and module contract. |
| `easing` | `Ease` | Input | Value supplied for `easing`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** `t` is clamped to 0..1 and both endpoints are exact.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int t
// Ease easing

auto result = epok::UtilityLibrary::ease(t, easing);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilitylibrary-event-queue-clear-1"></a>

## `epok::UtilityLibrary::event_queue_clear`

**Purpose.** Performs `event queue clear` as part of the utility module.

**Exact declaration**

```cpp
static GameplayEventQueue4 event_queue_clear()
```

- **Declared at:** [line 185](../../../runtime/utility.hpp#L185)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `GameplayEventQueue4`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

auto result = epok::UtilityLibrary::event_queue_clear();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-utilitylibrary-event-queue-emit-1"></a>

## `epok::UtilityLibrary::event_queue_emit`

**Purpose.** Performs `event queue emit` as part of the utility module.

**Exact declaration**

```cpp
static EventQueueMutation event_queue_emit(GameplayEventQueue4 state,uint32_t kind,int32_t value,ObjectId source)
```

- **Declared at:** [line 186](../../../runtime/utility.hpp#L186)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayEventQueue4` | Input | Value supplied for `state`. See the exact type and module contract. |
| `kind` | `int` | Input | Value supplied for `kind`. See the exact type and module contract. |
| `value` | `int` | Input | Value supplied for `value`. See the exact type and module contract. |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `EventQueueMutation`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayEventQueue4 state
// int kind
// int value
// ObjectId source

auto result = epok::UtilityLibrary::event_queue_emit(state, kind, value, source);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-utilitylibrary-event-queue-poll-1"></a>

## `epok::UtilityLibrary::event_queue_poll`

**Purpose.** Performs `event queue poll` as part of the utility module.

**Exact declaration**

```cpp
static EventQueuePoll event_queue_poll(GameplayEventQueue4 state)
```

- **Declared at:** [line 187](../../../runtime/utility.hpp#L187)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayEventQueue4` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `EventQueuePoll`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayEventQueue4 state

auto result = epok::UtilityLibrary::event_queue_poll(state);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-utilitylibrary-tween-advance-1"></a>

## `epok::UtilityLibrary::tween_advance`

**Purpose.** Performs `tween advance` as part of the utility module.

**Exact declaration**

```cpp
static TweenAdvanceSample tween_advance(GameplayTweenState state,Fixed delta_seconds)
```

- **Declared at:** [line 175](../../../runtime/utility.hpp#L175)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayTweenState` | Input | Value supplied for `state`. See the exact type and module contract. |
| `delta_seconds` | `int` | Input | Value supplied for `delta_seconds`. See the exact type and module contract. |

**Returns.** Returns `TweenAdvanceSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayTweenState state
// int delta_seconds

auto result = epok::UtilityLibrary::tween_advance(state, delta_seconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilitylibrary-tween-cancel-1"></a>

## `epok::UtilityLibrary::tween_cancel`

**Purpose.** Performs `tween cancel` as part of the utility module.

**Exact declaration**

```cpp
static GameplayTweenState tween_cancel(GameplayTweenState state)
```

- **Declared at:** [line 176](../../../runtime/utility.hpp#L176)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayTweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `GameplayTweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayTweenState state

auto result = epok::UtilityLibrary::tween_cancel(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilitylibrary-tween-schedule-1"></a>

## `epok::UtilityLibrary::tween_schedule`

**Purpose.** The full plan: `delay_seconds` holds the first leg back, `loop` and `legs` decide the replay, and `legs` 0 loops without end.

**Details.** A rejected plan returns a cancelled state rather than a half-applied one.

**Exact declaration**

```cpp
static GameplayTweenState tween_schedule(Fixed from,Fixed to,Fixed seconds,Ease easing,Fixed delay_seconds,TweenLoop loop,uint32_t legs)
```

- **Declared at:** [line 181](../../../runtime/utility.hpp#L181)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |
| `seconds` | `int` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `easing` | `Ease` | Input | Value supplied for `easing`. See the exact type and module contract. |
| `delay_seconds` | `int` | Input | Value supplied for `delay_seconds`. See the exact type and module contract. |
| `loop` | `TweenLoop` | Input | Value supplied for `loop`. See the exact type and module contract. |
| `legs` | `int` | Input | Value supplied for `legs`. See the exact type and module contract. |

**Returns.** Returns `GameplayTweenState`. Check the purpose and failure notes before using the value.

**Use it when.** A rejected plan returns a cancelled state rather than a half-applied one.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int from
// int to
// int seconds
// Ease easing
// int delay_seconds
// TweenLoop loop
// int legs

auto result = epok::UtilityLibrary::tween_schedule(from, to, seconds, easing, delay_seconds, loop, legs);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="epok-utilitylibrary-tween-start-1"></a>

## `epok::UtilityLibrary::tween_start`

**Purpose.** Performs `tween start` as part of the utility module.

**Exact declaration**

```cpp
static GameplayTweenState tween_start(Fixed from,Fixed to,Fixed seconds,Ease easing)
```

- **Declared at:** [line 174](../../../runtime/utility.hpp#L174)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `from` | `int` | Input | Value supplied for `from`. See the exact type and module contract. |
| `to` | `int` | Input | Value supplied for `to`. See the exact type and module contract. |
| `seconds` | `int` | Input | Value supplied for `seconds`. See the exact type and module contract. |
| `easing` | `Ease` | Input | Value supplied for `easing`. See the exact type and module contract. |

**Returns.** Returns `GameplayTweenState`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// int from
// int to
// int seconds
// Ease easing

auto result = epok::UtilityLibrary::tween_start(from, to, seconds, easing);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-utilitylibrary-tween-value-1"></a>

## `epok::UtilityLibrary::tween_value`

**Purpose.** Performs `tween value` as part of the utility module.

**Exact declaration**

```cpp
static Fixed tween_value(GameplayTweenState state)
```

- **Declared at:** [line 177](../../../runtime/utility.hpp#L177)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `GameplayTweenState` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the utility module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "utility.hpp"

// Assume these named values have been initialized with valid data:
// GameplayTweenState state

auto result = epok::UtilityLibrary::tween_value(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the utility module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
