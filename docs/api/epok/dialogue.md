# Epok API: Dialogue

> **Header:** `"dialogue.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/dialogue.hpp)

This module covers the dialogue module. It documents 24 public callables declared directly in this header.

## Declared types

`epok::dialogue::Action`, `epok::dialogue::Assignment`, `epok::dialogue::Binding`, `epok::dialogue::Condition`, `epok::dialogue::Conversation`, `epok::dialogue::Entered`, `epok::dialogue::Finished`, `epok::dialogue::Kind`, `epok::dialogue::Node`, `epok::dialogue::Option`, `epok::dialogue::Page`, `epok::dialogue::Phase`, `epok::dialogue::Resource`, `epok::dialogue::Result`, `epok::dialogue::Role`, `epok::dialogue::Session`, `epok::dialogue::Snapshot`, `epok::dialogue::Start`, `epok::dialogue::Started`, `epok::dialogue::Style`, `epok::dialogue::Text`, `epok::dialogue::Timing`, `epok::DialoguePhase`, `epok::DialogueResult`

## Callable index

- [`epok::dialogue::activation_changed`](#epok-dialogue-activation-changed-1) — Performs `activation changed` as part of the dialogue module.
- [`epok::dialogue::resource`](#epok-dialogue-resource-1) — Performs `resource` as part of the dialogue module.
- [`epok::dialogue::session`](#epok-dialogue-session-1) — Performs `session` as part of the dialogue module.
- [`epok::dialogue::Session::active`](#epok-dialogue-session-active-1) — Performs `active` as part of the dialogue module.
- [`epok::dialogue::Session::advance`](#epok-dialogue-session-advance-1) — Performs `advance` as part of the dialogue module.
- [`epok::dialogue::Session::cancel`](#epok-dialogue-session-cancel-1) — Performs `cancel` as part of the dialogue module.
- [`epok::dialogue::Session::choice_id`](#epok-dialogue-session-choice-id-1) — Performs `choice id` as part of the dialogue module.
- [`epok::dialogue::Session::choice_text`](#epok-dialogue-session-choice-text-1) — Performs `choice text` as part of the dialogue module.
- [`epok::dialogue::Session::initiator`](#epok-dialogue-session-initiator-1) — Performs `initiator` as part of the dialogue module.
- [`epok::dialogue::Session::invalidate_presenter`](#epok-dialogue-session-invalidate-presenter-1) — Performs `invalidate presenter` as part of the dialogue module.
- [`epok::dialogue::Session::move_selection`](#epok-dialogue-session-move-selection-1) — Performs `move selection` as part of the dialogue module.
- [`epok::dialogue::Session::presenter`](#epok-dialogue-session-presenter-1) — Performs `presenter` as part of the dialogue module.
- [`epok::dialogue::Session::prune`](#epok-dialogue-session-prune-1) — Performs `prune` as part of the dialogue module.
- [`epok::dialogue::Session::reset`](#epok-dialogue-session-reset-1) — Resets reset as part of the dialogue module.
- [`epok::dialogue::Session::snapshot`](#epok-dialogue-session-snapshot-1) — Performs `snapshot` as part of the dialogue module.
- [`epok::dialogue::Session::source`](#epok-dialogue-session-source-1) — Performs `source` as part of the dialogue module.
- [`epok::dialogue::Session::speaker`](#epok-dialogue-session-speaker-1) — Performs `speaker` as part of the dialogue module.
- [`epok::dialogue::Session::speaker_name`](#epok-dialogue-session-speaker-name-1) — Performs `speaker name` as part of the dialogue module.
- [`epok::dialogue::Session::speaker_portrait`](#epok-dialogue-session-speaker-portrait-1) — Performs `speaker portrait` as part of the dialogue module.
- [`epok::dialogue::Session::start`](#epok-dialogue-session-start-1) — Starts start as part of the dialogue module.
- [`epok::dialogue::Session::text`](#epok-dialogue-session-text-1) — Performs `text` as part of the dialogue module.
- [`epok::dialogue::Session::update`](#epok-dialogue-session-update-1) — Updates update as part of the dialogue module.
- [`epok::dialogue::Session::update_microseconds`](#epok-dialogue-session-update-microseconds-1) — Updates microseconds as part of the dialogue module.
- [`epok::dialogue::terminal_phase`](#epok-dialogue-terminal-phase-1) — Performs `terminal phase` as part of the dialogue module.

<a id="epok-dialogue-activation-changed-1"></a>

## `epok::dialogue::activation_changed`

**Purpose.** Performs `activation changed` as part of the dialogue module.

**Exact declaration**

```cpp
inline void activation_changed()
```

- **Declared at:** [line 382](../../../runtime/dialogue.hpp#L382)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::activation_changed();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-resource-1"></a>

## `epok::dialogue::resource`

**Purpose.** Performs `resource` as part of the dialogue module.

**Exact declaration**

```cpp
inline const Resource* resource(uint64_t conversation,uint64_t style=0,uint64_t widget=0)
```

- **Declared at:** [line 74](../../../runtime/dialogue.hpp#L74)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `conversation` | `uint64_t` | Input | Value supplied for `conversation`. See the exact type and module contract. |
| `style` | `uint64_t` | Input | Value supplied for `style`. See the exact type and module contract. |
| `widget` | `uint64_t` | Input | Value supplied for `widget`. See the exact type and module contract. |

**Returns.** Returns `const Resource *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t conversation
// uint64_t style
// uint64_t widget

auto result = epok::dialogue::resource(conversation, style, widget);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-1"></a>

## `epok::dialogue::session`

**Purpose.** Performs `session` as part of the dialogue module.

**Exact declaration**

```cpp
inline Session& session()
```

- **Declared at:** [line 381](../../../runtime/dialogue.hpp#L381)
- **Kind:** `function decl`

**Returns.** Returns `Session &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

auto result = epok::dialogue::session();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-active-1"></a>

## `epok::dialogue::Session::active`

**Purpose.** Performs `active` as part of the dialogue module.

**Exact declaration**

```cpp
bool active() const
```

- **Declared at:** [line 247](../../../runtime/dialogue.hpp#L247)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.active();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-advance-1"></a>

## `epok::dialogue::Session::advance`

**Purpose.** Performs `advance` as part of the dialogue module.

**Exact declaration**

```cpp
Result advance()
```

- **Declared at:** [line 314](../../../runtime/dialogue.hpp#L314)
- **Kind:** `cxx method`

**Returns.** Returns `Result`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.advance();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-cancel-1"></a>

## `epok::dialogue::Session::cancel`

**Purpose.** Performs `cancel` as part of the dialogue module.

**Exact declaration**

```cpp
Result cancel()
```

- **Declared at:** [line 320](../../../runtime/dialogue.hpp#L320)
- **Kind:** `cxx method`

**Returns.** Returns `Result`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.cancel();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-choice-id-1"></a>

## `epok::dialogue::Session::choice_id`

**Purpose.** Performs `choice id` as part of the dialogue module.

**Exact declaration**

```cpp
uint32_t choice_id(unsigned visible) const
```

- **Declared at:** [line 261](../../../runtime/dialogue.hpp#L261)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `visible` | `unsigned int` | Input | Value supplied for `visible`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int visible

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.choice_id(visible);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-choice-text-1"></a>

## `epok::dialogue::Session::choice_text`

**Purpose.** Performs `choice text` as part of the dialogue module.

**Exact declaration**

```cpp
const char* choice_text(unsigned visible) const
```

- **Declared at:** [line 265](../../../runtime/dialogue.hpp#L265)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `visible` | `unsigned int` | Input | Value supplied for `visible`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// unsigned int visible

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.choice_text(visible);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-initiator-1"></a>

## `epok::dialogue::Session::initiator`

**Purpose.** Performs `initiator` as part of the dialogue module.

**Exact declaration**

```cpp
ObjectId initiator() const
```

- **Declared at:** [line 250](../../../runtime/dialogue.hpp#L250)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.initiator();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-invalidate-presenter-1"></a>

## `epok::dialogue::Session::invalidate_presenter`

**Purpose.** Performs `invalidate presenter` as part of the dialogue module.

**Exact declaration**

```cpp
void invalidate_presenter(ObjectId presenter)
```

- **Declared at:** [line 333](../../../runtime/dialogue.hpp#L333)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter

epok::dialogue::Session& object = /* obtain a valid instance */;

object.invalidate_presenter(presenter);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-move-selection-1"></a>

## `epok::dialogue::Session::move_selection`

**Purpose.** Performs `move selection` as part of the dialogue module.

**Exact declaration**

```cpp
Result move_selection(int32_t delta)
```

- **Declared at:** [line 325](../../../runtime/dialogue.hpp#L325)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delta` | `int32_t` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** Returns `Result`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// int32_t delta

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.move_selection(delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-presenter-1"></a>

## `epok::dialogue::Session::presenter`

**Purpose.** Performs `presenter` as part of the dialogue module.

**Exact declaration**

```cpp
ObjectId presenter() const
```

- **Declared at:** [line 251](../../../runtime/dialogue.hpp#L251)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.presenter();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-prune-1"></a>

## `epok::dialogue::Session::prune`

**Purpose.** Performs `prune` as part of the dialogue module.

**Exact declaration**

```cpp
void prune()
```

- **Declared at:** [line 332](../../../runtime/dialogue.hpp#L332)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

object.prune();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-reset-1"></a>

## `epok::dialogue::Session::reset`

**Purpose.** Resets reset as part of the dialogue module.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 334](../../../runtime/dialogue.hpp#L334)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-snapshot-1"></a>

## `epok::dialogue::Session::snapshot`

**Purpose.** Performs `snapshot` as part of the dialogue module.

**Exact declaration**

```cpp
Snapshot snapshot() const
```

- **Declared at:** [line 248](../../../runtime/dialogue.hpp#L248)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Snapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.snapshot();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-source-1"></a>

## `epok::dialogue::Session::source`

**Purpose.** Performs `source` as part of the dialogue module.

**Exact declaration**

```cpp
ObjectId source() const
```

- **Declared at:** [line 249](../../../runtime/dialogue.hpp#L249)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.source();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-speaker-1"></a>

## `epok::dialogue::Session::speaker`

**Purpose.** Performs `speaker` as part of the dialogue module.

**Exact declaration**

```cpp
ObjectId speaker() const
```

- **Declared at:** [line 252](../../../runtime/dialogue.hpp#L252)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.speaker();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-speaker-name-1"></a>

## `epok::dialogue::Session::speaker_name`

**Purpose.** Performs `speaker name` as part of the dialogue module.

**Exact declaration**

```cpp
const char* speaker_name() const
```

- **Declared at:** [line 253](../../../runtime/dialogue.hpp#L253)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.speaker_name();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-speaker-portrait-1"></a>

## `epok::dialogue::Session::speaker_portrait`

**Purpose.** Performs `speaker portrait` as part of the dialogue module.

**Exact declaration**

```cpp
int32_t speaker_portrait() const
```

- **Declared at:** [line 259](../../../runtime/dialogue.hpp#L259)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.speaker_portrait();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-start-1"></a>

## `epok::dialogue::Session::start`

**Purpose.** Starts start as part of the dialogue module.

**Exact declaration**

```cpp
Result start(const Conversation& table,const Start& options)
```

- **Declared at:** [line 271](../../../runtime/dialogue.hpp#L271)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const Conversation &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `options` | `const Start &` | Input | Value supplied for `options`. See the exact type and module contract. |

**Returns.** Returns `Result`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// const Conversation & table
// const Start & options

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.start(table, options);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dialogue-session-text-1"></a>

## `epok::dialogue::Session::text`

**Purpose.** Performs `text` as part of the dialogue module.

**Exact declaration**

```cpp
const char* text() const
```

- **Declared at:** [line 260](../../../runtime/dialogue.hpp#L260)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

epok::dialogue::Session& object = /* obtain a valid instance */;

auto result = object.text();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-update-1"></a>

## `epok::dialogue::Session::update`

**Purpose.** Updates update as part of the dialogue module.

**Exact declaration**

```cpp
void update(uint32_t milliseconds)
```

- **Declared at:** [line 340](../../../runtime/dialogue.hpp#L340)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `milliseconds` | `uint32_t` | Input | Value supplied for `milliseconds`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t milliseconds

epok::dialogue::Session& object = /* obtain a valid instance */;

object.update(milliseconds);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-session-update-microseconds-1"></a>

## `epok::dialogue::Session::update_microseconds`

**Purpose.** Updates microseconds as part of the dialogue module.

**Exact declaration**

```cpp
void update_microseconds(uint32_t elapsed)
```

- **Declared at:** [line 335](../../../runtime/dialogue.hpp#L335)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t elapsed

epok::dialogue::Session& object = /* obtain a valid instance */;

object.update_microseconds(elapsed);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-dialogue-terminal-phase-1"></a>

## `epok::dialogue::terminal_phase`

**Purpose.** Performs `terminal phase` as part of the dialogue module.

**Exact declaration**

```cpp
inline Phase terminal_phase(Result result)
```

- **Declared at:** [line 20](../../../runtime/dialogue.hpp#L20)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `result` | `Result` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** Returns `Phase`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue.hpp"

// Assume these named values have been initialized with valid data:
// Result result

auto result = epok::dialogue::terminal_phase(result);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
