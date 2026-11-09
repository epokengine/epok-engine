# Epok API: Dialogue Helpers

> **Header:** `"dialogue_helpers.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/dialogue_helpers.hpp)

This module covers the dialogue helpers module. It documents 44 public callables declared directly in this header.

## Declared types

`epok::DialogueHandle`, `epok::DialogueInteractionComponent`, `epok::DialogueNPC2D`, `epok::DialogueNPC3D`, `epok::DialoguePresenter`, `epok::DialogueRoleBindingsComponent`, `epok::DialogueSessionLibrary`, `epok::DialogueSnapshot`, `epok::DialogueStartResult`, `epok::DialogueTiming`

## Callable index

- [`epok::dialogue::begin_presenter`](#epok-dialogue-begin-presenter-1) — Begins presenter as part of the dialogue helpers module.
- [`epok::dialogue::bind_widget_layout`](#epok-dialogue-bind-widget-layout-1) — Performs `bind widget layout` as part of the dialogue helpers module.
- [`epok::dialogue::matches_handle`](#epok-dialogue-matches-handle-1) — Compare before pruning: a stale handle cannot touch the current capture.
- [`epok::dialogue::prepare_presenter`](#epok-dialogue-prepare-presenter-1) — Performs `prepare presenter` as part of the dialogue helpers module.
- [`epok::dialogue::presenter_resource`](#epok-dialogue-presenter-resource-1) — Performs `presenter resource` as part of the dialogue helpers module.
- [`epok::dialogue::presenter_timing`](#epok-dialogue-presenter-timing-1) — Performs `presenter timing` as part of the dialogue helpers module.
- [`epok::dialogue::release_presenter`](#epok-dialogue-release-presenter-1) — Performs `release presenter` as part of the dialogue helpers module.
- [`epok::dialogue::snapshot_value`](#epok-dialogue-snapshot-value-1) — Performs `snapshot value` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::class_id`](#epok-dialogueinteractioncomponent-class-id-1) — Performs `class id` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::dialogue_finished`](#epok-dialogueinteractioncomponent-dialogue-finished-1) — Performs `dialogue finished` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::dialogue_node_entered`](#epok-dialogueinteractioncomponent-dialogue-node-entered-1) — Performs `dialogue node entered` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::dialogue_started`](#epok-dialogueinteractioncomponent-dialogue-started-1) — Performs `dialogue started` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::interaction_started`](#epok-dialogueinteractioncomponent-interaction-started-1) — Performs `interaction started` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::last_result`](#epok-dialogueinteractioncomponent-last-result-1) — Performs `last result` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::session`](#epok-dialogueinteractioncomponent-session-1) — Performs `session` as part of the dialogue helpers module.
- [`epok::DialogueInteractionComponent::start_dialogue`](#epok-dialogueinteractioncomponent-start-dialogue-1) — Starts dialogue as part of the dialogue helpers module.
- [`epok::DialogueNPC2D::class_id`](#epok-dialoguenpc2d-class-id-1) — Performs `class id` as part of the dialogue helpers module.
- [`epok::DialogueNPC3D::class_id`](#epok-dialoguenpc3d-class-id-1) — Performs `class id` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::begin`](#epok-dialoguepresenter-begin-1) — Begins begin as part of the dialogue helpers module.
- [`epok::DialoguePresenter::can_present`](#epok-dialoguepresenter-can-present-1) — Performs `can present` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::class_id`](#epok-dialoguepresenter-class-id-1) — Performs `class id` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::clear`](#epok-dialoguepresenter-clear-1) — Clears clear as part of the dialogue helpers module.
- [`epok::DialoguePresenter::dialogue_closed`](#epok-dialoguepresenter-dialogue-closed-1) — Performs `dialogue closed` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::dialogue_opened`](#epok-dialoguepresenter-dialogue-opened-1) — Performs `dialogue opened` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::end_play`](#epok-dialoguepresenter-end-play-1) — Ends play as part of the dialogue helpers module.
- [`epok::DialoguePresenter::on_disable`](#epok-dialoguepresenter-on-disable-1) — Performs `on disable` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::on_frame`](#epok-dialoguepresenter-on-frame-1) — Performs `on frame` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::owns`](#epok-dialoguepresenter-owns-1) — Performs `owns` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::prepare`](#epok-dialoguepresenter-prepare-1) — Performs `prepare` as part of the dialogue helpers module.
- [`epok::DialoguePresenter::present`](#epok-dialoguepresenter-present-1) — Performs `present` as part of the dialogue helpers module.
- [`epok::DialogueRoleBindingsComponent::class_id`](#epok-dialoguerolebindingscomponent-class-id-1) — Performs `class id` as part of the dialogue helpers module.
- [`epok::DialogueRoleBindingsComponent::receiver`](#epok-dialoguerolebindingscomponent-receiver-1) — Performs `receiver` as part of the dialogue helpers module.
- [`epok::DialogueRoleBindingsComponent::role_id`](#epok-dialoguerolebindingscomponent-role-id-1) — Performs `role id` as part of the dialogue helpers module.
- [`epok::DialogueRoleBindingsComponent::set_binding`](#epok-dialoguerolebindingscomponent-set-binding-1) — Sets binding as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::advance`](#epok-dialoguesessionlibrary-advance-1) — Performs `advance` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::cancel`](#epok-dialoguesessionlibrary-cancel-1) — Performs `cancel` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::choice_id`](#epok-dialoguesessionlibrary-choice-id-1) — Performs `choice id` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::move_selection`](#epok-dialoguesessionlibrary-move-selection-1) — Performs `move selection` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::owner_session`](#epok-dialoguesessionlibrary-owner-session-1) — Performs `owner session` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::snapshot`](#epok-dialoguesessionlibrary-snapshot-1) — Performs `snapshot` as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::start`](#epok-dialoguesessionlibrary-start-1) — Starts start as part of the dialogue helpers module.
- [`epok::DialogueSessionLibrary::valid`](#epok-dialoguesessionlibrary-valid-1) — Performs `valid` as part of the dialogue helpers module.
- [`epok::ui_anchor::set_target`](#epok-ui-anchor-set-target-1) — Sets target as part of the dialogue helpers module.
- [`epok::ui_anchor::valid_binding`](#epok-ui-anchor-valid-binding-1) — Performs `valid binding` as part of the dialogue helpers module.

<a id="epok-dialogue-begin-presenter-1"></a>

## `epok::dialogue::begin_presenter`

**Purpose.** Begins presenter as part of the dialogue helpers module.

**Exact declaration**

```cpp
void begin_presenter(ObjectId presenter,ObjectId source)
```

- **Declared at:** [line 73](../../../runtime/dialogue_helpers.hpp#L73)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter
// ObjectId source

epok::dialogue::begin_presenter(presenter, source);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogue-bind-widget-layout-1"></a>

## `epok::dialogue::bind_widget_layout`

**Purpose.** Performs `bind widget layout` as part of the dialogue helpers module.

**Exact declaration**

```cpp
void bind_widget_layout(Actor& presenter,uint64_t layout)
```

- **Declared at:** [line 72](../../../runtime/dialogue_helpers.hpp#L72)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `Actor &` | Input/output; inspect the function contract | Value supplied for `presenter`. See the exact type and module contract. |
| `layout` | `uint64_t` | Input | Value supplied for `layout`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// Actor & presenter
// uint64_t layout

epok::dialogue::bind_widget_layout(presenter, layout);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dialogue-matches-handle-1"></a>

## `epok::dialogue::matches_handle`

**Purpose.** Compare before pruning: a stale handle cannot touch the current capture.

**Exact declaration**

```cpp
inline bool matches_handle(DialogueHandle handle)
```

- **Declared at:** [line 28](../../../runtime/dialogue_helpers.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle

auto result = epok::dialogue::matches_handle(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dialogue-prepare-presenter-1"></a>

## `epok::dialogue::prepare_presenter`

**Purpose.** Performs `prepare presenter` as part of the dialogue helpers module.

**Exact declaration**

```cpp
bool prepare_presenter(ObjectId presenter,ObjectId source,const Resource& resource)
```

- **Declared at:** [line 69](../../../runtime/dialogue_helpers.hpp#L69)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |
| `resource` | `const Resource &` | Input | Value supplied for `resource`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter
// ObjectId source
// const Resource & resource

auto result = epok::dialogue::prepare_presenter(presenter, source, resource);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dialogue-presenter-resource-1"></a>

## `epok::dialogue::presenter_resource`

**Purpose.** Performs `presenter resource` as part of the dialogue helpers module.

**Exact declaration**

```cpp
const Resource* presenter_resource(ObjectId presenter,uint64_t conversation,uint64_t style)
```

- **Declared at:** [line 70](../../../runtime/dialogue_helpers.hpp#L70)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |
| `conversation` | `uint64_t` | Input | Value supplied for `conversation`. See the exact type and module contract. |
| `style` | `uint64_t` | Input | Value supplied for `style`. See the exact type and module contract. |

**Returns.** Returns `const Resource *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter
// uint64_t conversation
// uint64_t style

auto result = epok::dialogue::presenter_resource(presenter, conversation, style);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogue-presenter-timing-1"></a>

## `epok::dialogue::presenter_timing`

**Purpose.** Performs `presenter timing` as part of the dialogue helpers module.

**Exact declaration**

```cpp
Timing presenter_timing(ObjectId presenter,const Resource& resource)
```

- **Declared at:** [line 71](../../../runtime/dialogue_helpers.hpp#L71)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |
| `resource` | `const Resource &` | Input | Value supplied for `resource`. See the exact type and module contract. |

**Returns.** Returns `Timing`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter
// const Resource & resource

auto result = epok::dialogue::presenter_timing(presenter, resource);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dialogue-release-presenter-1"></a>

## `epok::dialogue::release_presenter`

**Purpose.** Performs `release presenter` as part of the dialogue helpers module.

**Exact declaration**

```cpp
void release_presenter(ObjectId presenter,ObjectId source,DialogueResult result)
```

- **Declared at:** [line 74](../../../runtime/dialogue_helpers.hpp#L74)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `presenter` | `ObjectId` | Input | Value supplied for `presenter`. See the exact type and module contract. |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |
| `result` | `DialogueResult` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId presenter
// ObjectId source
// DialogueResult result

epok::dialogue::release_presenter(presenter, source, result);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogue-snapshot-value-1"></a>

## `epok::dialogue::snapshot_value`

**Purpose.** Performs `snapshot value` as part of the dialogue helpers module.

**Exact declaration**

```cpp
inline DialogueSnapshot snapshot_value()
```

- **Declared at:** [line 29](../../../runtime/dialogue_helpers.hpp#L29)
- **Kind:** `function decl`

**Returns.** Returns `DialogueSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

auto result = epok::dialogue::snapshot_value();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-class-id-1"></a>

## `epok::DialogueInteractionComponent::class_id`

**Purpose.** Performs `class id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 135](../../../runtime/dialogue_helpers.hpp#L135)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-dialogue-finished-1"></a>

## `epok::DialogueInteractionComponent::dialogue_finished`

**Purpose.** Performs `dialogue finished` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void dialogue_finished(ObjectId initiator,uint32_t port,DialogueResult result)
```

- **Declared at:** [line 143](../../../runtime/dialogue_helpers.hpp#L143)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |
| `result` | `DialogueResult` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port
// DialogueResult result

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

object.dialogue_finished(initiator, port, result);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-dialogue-node-entered-1"></a>

## `epok::DialogueInteractionComponent::dialogue_node_entered`

**Purpose.** Performs `dialogue node entered` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void dialogue_node_entered(uint32_t node)
```

- **Declared at:** [line 142](../../../runtime/dialogue_helpers.hpp#L142)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `node` | `uint32_t` | Input | Value supplied for `node`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t node

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

object.dialogue_node_entered(node);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-dialogue-started-1"></a>

## `epok::DialogueInteractionComponent::dialogue_started`

**Purpose.** Performs `dialogue started` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void dialogue_started(ObjectId initiator,uint32_t port,DialogueHandle session)
```

- **Declared at:** [line 141](../../../runtime/dialogue_helpers.hpp#L141)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |
| `session` | `DialogueHandle` | Input | Value supplied for `session`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port
// DialogueHandle session

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

object.dialogue_started(initiator, port, session);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-interaction-started-1"></a>

## `epok::DialogueInteractionComponent::interaction_started`

**Purpose.** Performs `interaction started` as part of the dialogue helpers module.

**Exact declaration**

```cpp
void interaction_started(ObjectId initiator,uint32_t port) override
```

- **Declared at:** [line 163](../../../runtime/dialogue_helpers.hpp#L163)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

object.interaction_started(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-last-result-1"></a>

## `epok::DialogueInteractionComponent::last_result`

**Purpose.** Performs `last result` as part of the dialogue helpers module.

**Exact declaration**

```cpp
DialogueResult last_result() const
```

- **Declared at:** [line 144](../../../runtime/dialogue_helpers.hpp#L144)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `DialogueResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

auto result = object.last_result();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-session-1"></a>

## `epok::DialogueInteractionComponent::session`

**Purpose.** Performs `session` as part of the dialogue helpers module.

**Exact declaration**

```cpp
DialogueHandle session() const
```

- **Declared at:** [line 145](../../../runtime/dialogue_helpers.hpp#L145)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `DialogueHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

auto result = object.session();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialogueinteractioncomponent-start-dialogue-1"></a>

## `epok::DialogueInteractionComponent::start_dialogue`

**Purpose.** Starts dialogue as part of the dialogue helpers module.

**Exact declaration**

```cpp
DialogueStartResult start_dialogue(ObjectId initiator,uint32_t port)
```

- **Declared at:** [line 146](../../../runtime/dialogue_helpers.hpp#L146)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `DialogueStartResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId initiator
// uint32_t port

epok::DialogueInteractionComponent& object = /* obtain a valid instance */;

auto result = object.start_dialogue(initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguenpc2d-class-id-1"></a>

## `epok::DialogueNPC2D::class_id`

**Purpose.** Performs `class id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 200](../../../runtime/dialogue_helpers.hpp#L200)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueNPC2D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguenpc3d-class-id-1"></a>

## `epok::DialogueNPC3D::class_id`

**Purpose.** Performs `class id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint64_t class_id()const override
```

- **Declared at:** [line 194](../../../runtime/dialogue_helpers.hpp#L194)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueNPC3D& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-begin-1"></a>

## `epok::DialoguePresenter::begin`

**Purpose.** Begins begin as part of the dialogue helpers module.

**Exact declaration**

```cpp
void begin(ObjectId source)
```

- **Declared at:** [line 322](../../../runtime/dialogue_helpers.hpp#L322)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.begin(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-can-present-1"></a>

## `epok::DialoguePresenter::can_present`

**Purpose.** Performs `can present` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual bool can_present(ObjectId source) const
```

- **Declared at:** [line 250](../../../runtime/dialogue_helpers.hpp#L250)
- **Kind:** `cxx method`; qualifiers: `const, virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source

epok::DialoguePresenter& object = /* obtain a valid instance */;

auto result = object.can_present(source);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dialoguepresenter-class-id-1"></a>

## `epok::DialoguePresenter::class_id`

**Purpose.** Performs `class id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 232](../../../runtime/dialogue_helpers.hpp#L232)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialoguePresenter& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-clear-1"></a>

## `epok::DialoguePresenter::clear`

**Purpose.** Clears clear as part of the dialogue helpers module.

**Exact declaration**

```cpp
void clear(bool closing=false,DialogueResult result=DialogueResult::NoSession)
```

- **Declared at:** [line 292](../../../runtime/dialogue_helpers.hpp#L292)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `closing` | `bool` | Input | Value supplied for `closing`. See the exact type and module contract. |
| `result` | `DialogueResult` | Input | Value supplied for `result`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// bool closing
// DialogueResult result

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.clear(closing, result);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-dialogue-closed-1"></a>

## `epok::DialoguePresenter::dialogue_closed`

**Purpose.** Performs `dialogue closed` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void dialogue_closed(DialogueSnapshot state)
```

- **Declared at:** [line 249](../../../runtime/dialogue_helpers.hpp#L249)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `DialogueSnapshot` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueSnapshot state

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.dialogue_closed(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-dialogue-opened-1"></a>

## `epok::DialoguePresenter::dialogue_opened`

**Purpose.** Performs `dialogue opened` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void dialogue_opened(DialogueSnapshot state)
```

- **Declared at:** [line 248](../../../runtime/dialogue_helpers.hpp#L248)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `DialogueSnapshot` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueSnapshot state

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.dialogue_opened(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-end-play-1"></a>

## `epok::DialoguePresenter::end_play`

**Purpose.** Ends play as part of the dialogue helpers module.

**Exact declaration**

```cpp
void end_play(EndPlayReason reason) override
```

- **Declared at:** [line 334](../../../runtime/dialogue_helpers.hpp#L334)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reason` | `EndPlayReason` | Input | Value supplied for `reason`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason reason

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.end_play(reason);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-on-disable-1"></a>

## `epok::DialoguePresenter::on_disable`

**Purpose.** Performs `on disable` as part of the dialogue helpers module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 333](../../../runtime/dialogue_helpers.hpp#L333)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-on-frame-1"></a>

## `epok::DialoguePresenter::on_frame`

**Purpose.** Performs `on frame` as part of the dialogue helpers module.

**Exact declaration**

```cpp
void on_frame(uint32_t elapsed) override
```

- **Declared at:** [line 323](../../../runtime/dialogue_helpers.hpp#L323)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `elapsed` | `uint32_t` | Input | Value supplied for `elapsed`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t elapsed

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.on_frame(elapsed);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguepresenter-owns-1"></a>

## `epok::DialoguePresenter::owns`

**Purpose.** Performs `owns` as part of the dialogue helpers module.

**Exact declaration**

```cpp
bool owns(ObjectId source) const
```

- **Declared at:** [line 321](../../../runtime/dialogue_helpers.hpp#L321)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source

epok::DialoguePresenter& object = /* obtain a valid instance */;

auto result = object.owns(source);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dialoguepresenter-prepare-1"></a>

## `epok::DialoguePresenter::prepare`

**Purpose.** Performs `prepare` as part of the dialogue helpers module.

**Exact declaration**

```cpp
bool prepare(ObjectId source,const dialogue::Resource& resource)
```

- **Declared at:** [line 317](../../../runtime/dialogue_helpers.hpp#L317)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |
| `resource` | `const dialogue::Resource &` | Input | Value supplied for `resource`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source
// const dialogue::Resource & resource

epok::DialoguePresenter& object = /* obtain a valid instance */;

auto result = object.prepare(source, resource);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-dialoguepresenter-present-1"></a>

## `epok::DialoguePresenter::present`

**Purpose.** Performs `present` as part of the dialogue helpers module.

**Exact declaration**

```cpp
virtual void present(DialogueSnapshot state)
```

- **Declared at:** [line 266](../../../runtime/dialogue_helpers.hpp#L266)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `state` | `DialogueSnapshot` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueSnapshot state

epok::DialoguePresenter& object = /* obtain a valid instance */;

object.present(state);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguerolebindingscomponent-class-id-1"></a>

## `epok::DialogueRoleBindingsComponent::class_id`

**Purpose.** Performs `class id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 79](../../../runtime/dialogue_helpers.hpp#L79)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

epok::DialogueRoleBindingsComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguerolebindingscomponent-receiver-1"></a>

## `epok::DialogueRoleBindingsComponent::receiver`

**Purpose.** Performs `receiver` as part of the dialogue helpers module.

**Exact declaration**

```cpp
ObjectId receiver(uint32_t slot) const
```

- **Declared at:** [line 97](../../../runtime/dialogue_helpers.hpp#L97)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot

epok::DialogueRoleBindingsComponent& object = /* obtain a valid instance */;

auto result = object.receiver(slot);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguerolebindingscomponent-role-id-1"></a>

## `epok::DialogueRoleBindingsComponent::role_id`

**Purpose.** Performs `role id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
uint32_t role_id(uint32_t slot) const
```

- **Declared at:** [line 96](../../../runtime/dialogue_helpers.hpp#L96)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot

epok::DialogueRoleBindingsComponent& object = /* obtain a valid instance */;

auto result = object.role_id(slot);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguerolebindingscomponent-set-binding-1"></a>

## `epok::DialogueRoleBindingsComponent::set_binding`

**Purpose.** Sets binding as part of the dialogue helpers module.

**Exact declaration**

```cpp
bool set_binding(uint32_t slot,uint32_t role,ObjectId target)
```

- **Declared at:** [line 98](../../../runtime/dialogue_helpers.hpp#L98)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `slot` | `uint32_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `role` | `uint32_t` | Input | Value supplied for `role`. See the exact type and module contract. |
| `target` | `ObjectId` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t slot
// uint32_t role
// ObjectId target

epok::DialogueRoleBindingsComponent& object = /* obtain a valid instance */;

auto result = object.set_binding(slot, role, target);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-dialoguesessionlibrary-advance-1"></a>

## `epok::DialogueSessionLibrary::advance`

**Purpose.** Performs `advance` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueResult advance(DialogueHandle handle)
```

- **Declared at:** [line 53](../../../runtime/dialogue_helpers.hpp#L53)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `DialogueResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle

auto result = epok::DialogueSessionLibrary::advance(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-cancel-1"></a>

## `epok::DialogueSessionLibrary::cancel`

**Purpose.** Performs `cancel` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueResult cancel(DialogueHandle handle)
```

- **Declared at:** [line 56](../../../runtime/dialogue_helpers.hpp#L56)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `DialogueResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle

auto result = epok::DialogueSessionLibrary::cancel(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-choice-id-1"></a>

## `epok::DialogueSessionLibrary::choice_id`

**Purpose.** Performs `choice id` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static uint32_t choice_id(DialogueHandle handle,uint32_t visible)
```

- **Declared at:** [line 62](../../../runtime/dialogue_helpers.hpp#L62)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |
| `visible` | `uint32_t` | Input | Value supplied for `visible`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle
// uint32_t visible

auto result = epok::DialogueSessionLibrary::choice_id(handle, visible);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-move-selection-1"></a>

## `epok::DialogueSessionLibrary::move_selection`

**Purpose.** Performs `move selection` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueResult move_selection(DialogueHandle handle,int32_t delta)
```

- **Declared at:** [line 59](../../../runtime/dialogue_helpers.hpp#L59)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |
| `delta` | `int32_t` | Input | Value supplied for `delta`. See the exact type and module contract. |

**Returns.** Returns `DialogueResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle
// int32_t delta

auto result = epok::DialogueSessionLibrary::move_selection(handle, delta);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-owner-session-1"></a>

## `epok::DialogueSessionLibrary::owner_session`

**Purpose.** Performs `owner session` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueHandle owner_session(ObjectId source)
```

- **Declared at:** [line 44](../../../runtime/dialogue_helpers.hpp#L44)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `DialogueHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source

auto result = epok::DialogueSessionLibrary::owner_session(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-snapshot-1"></a>

## `epok::DialogueSessionLibrary::snapshot`

**Purpose.** Performs `snapshot` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueSnapshot snapshot(DialogueHandle handle)
```

- **Declared at:** [line 49](../../../runtime/dialogue_helpers.hpp#L49)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `DialogueSnapshot`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle

auto result = epok::DialogueSessionLibrary::snapshot(handle);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-start-1"></a>

## `epok::DialogueSessionLibrary::start`

**Purpose.** Starts start as part of the dialogue helpers module.

**Exact declaration**

```cpp
static DialogueStartResult start(ObjectId source,ObjectId initiator,uint32_t port)
```

- **Declared at:** [line 39](../../../runtime/dialogue_helpers.hpp#L39)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `ObjectId` | Input | Value supplied for `source`. See the exact type and module contract. |
| `initiator` | `ObjectId` | Input | Value supplied for `initiator`. See the exact type and module contract. |
| `port` | `uint32_t` | Input | Value supplied for `port`. See the exact type and module contract. |

**Returns.** Returns `DialogueStartResult`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId source
// ObjectId initiator
// uint32_t port

auto result = epok::DialogueSessionLibrary::start(source, initiator, port);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dialoguesessionlibrary-valid-1"></a>

## `epok::DialogueSessionLibrary::valid`

**Purpose.** Performs `valid` as part of the dialogue helpers module.

**Exact declaration**

```cpp
static bool valid(DialogueHandle handle)
```

- **Declared at:** [line 40](../../../runtime/dialogue_helpers.hpp#L40)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `handle` | `DialogueHandle` | Input | Value supplied for `handle`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// DialogueHandle handle

auto result = epok::DialogueSessionLibrary::valid(handle);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-ui-anchor-set-target-1"></a>

## `epok::ui_anchor::set_target`

**Purpose.** Sets target as part of the dialogue helpers module.

**Exact declaration**

```cpp
void set_target(ObjectId reference,Actor& owner,ObjectId target)
```

- **Declared at:** [line 7](../../../runtime/dialogue_helpers.hpp#L7)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reference` | `ObjectId` | Input | Value supplied for `reference`. See the exact type and module contract. |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `target` | `ObjectId` | Input | Value supplied for `target`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId reference
// Actor & owner
// ObjectId target

epok::ui_anchor::set_target(reference, owner, target);
```

**Why choose it.** It provides direct, allocation-conscious access to the dialogue helpers module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-valid-binding-1"></a>

## `epok::ui_anchor::valid_binding`

**Purpose.** Performs `valid binding` as part of the dialogue helpers module.

**Exact declaration**

```cpp
bool valid_binding(ObjectId reference,const Actor& owner)
```

- **Declared at:** [line 6](../../../runtime/dialogue_helpers.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reference` | `ObjectId` | Input | Value supplied for `reference`. See the exact type and module contract. |
| `owner` | `const Actor &` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the dialogue helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "dialogue_helpers.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId reference
// const Actor & owner

auto result = epok::ui_anchor::valid_binding(reference, owner);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
