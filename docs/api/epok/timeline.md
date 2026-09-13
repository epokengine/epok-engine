# Epok API: Timeline

> **Header:** `"timeline.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/timeline.hpp)

This module covers fixed-step simulation time and frame timing. It documents 3 public callables declared directly in this header.

## Declared types

`epok::timeline::Argument`, `epok::timeline::Curve`, `epok::timeline::Interpolation`, `epok::timeline::Key`, `epok::timeline::Marker`, `epok::timeline::Signal`

## Callable index

- [`epok::timeline::poll_marker`](#epok-timeline-poll-marker-1) — Sorted tables use (tick, persistent marker ID) at cook time.
- [`epok::timeline::sample`](#epok-timeline-sample-1) — Same signed division toward zero and wide intermediate as bp::Timeline.
- [`epok::timeline::saturate`](#epok-timeline-saturate-1) — Performs `saturate` as part of fixed-step simulation time and frame timing.

<a id="epok-timeline-poll-marker-1"></a>

## `epok::timeline::poll_marker`

**Purpose.** Sorted tables use (tick, persistent marker ID) at cook time.

**Details.** The caller stores one cursor per playback, resets it only on restart, and supplies clamped time. Zero-time markers are dispatched on the first positive advance. Polling at the same time cannot repeat a marker. No callback or side effect is retained.

**Exact declaration**

```cpp
inline bool poll_marker(const Marker* markers, uint16_t count, uint16_t& cursor, int32_t tick, uint16_t& id)
```

- **Declared at:** [line 57](../../../runtime/timeline.hpp#L57)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `markers` | `const Marker *` | Input | Value supplied for `markers`. See the exact type and module contract. |
| `count` | `uint16_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `cursor` | `uint16_t &` | Input/output; inspect the function contract | Value supplied for `cursor`. See the exact type and module contract. |
| `tick` | `int32_t` | Input | Value supplied for `tick`. See the exact type and module contract. |
| `id` | `uint16_t &` | Input/output; inspect the function contract | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** The caller stores one cursor per playback, resets it only on restart, and supplies clamped time. Zero-time markers are dispatched on the first positive advance. Polling at the same time cannot repeat a marker. No callback or side effect is retained.

**Usage pattern**

```cpp
#include "timeline.hpp"

// Assume these named values have been initialized with valid data:
// const Marker * markers
// uint16_t count
// uint16_t & cursor
// int32_t tick
// uint16_t & id

auto result = epok::timeline::poll_marker(markers, count, cursor, tick, id);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-timeline-sample-1"></a>

## `epok::timeline::sample`

**Purpose.** Same signed division toward zero and wide intermediate as bp::Timeline.

**Details.** Ticks and values are raw Q12. Do not quantize alpha before interpolation.

**Exact declaration**

```cpp
inline int32_t sample(Curve curve, int32_t tick)
```

- **Declared at:** [line 22](../../../runtime/timeline.hpp#L22)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `curve` | `Curve` | Input | Value supplied for `curve`. See the exact type and module contract. |
| `tick` | `int32_t` | Input | Value supplied for `tick`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Ticks and values are raw Q12. Do not quantize alpha before interpolation.

**Usage pattern**

```cpp
#include "timeline.hpp"

// Assume these named values have been initialized with valid data:
// Curve curve
// int32_t tick

auto result = epok::timeline::sample(curve, tick);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-timeline-saturate-1"></a>

## `epok::timeline::saturate`

**Purpose.** Performs `saturate` as part of fixed-step simulation time and frame timing.

**Exact declaration**

```cpp
constexpr int32_t saturate(int64_t value)
```

- **Declared at:** [line 17](../../../runtime/timeline.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int64_t` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need fixed-step simulation time and frame timing and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "timeline.hpp"

// Assume these named values have been initialized with valid data:
// int64_t value

auto result = epok::timeline::saturate(value);
```

**Why choose it.** It provides direct, allocation-conscious access to fixed-step simulation time and frame timing. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
