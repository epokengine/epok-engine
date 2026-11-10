# Epok API: Ui Anchor

> **Header:** `"ui_anchor.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/ui_anchor.hpp)

This module covers the ui anchor module. It documents 10 public callables declared directly in this header.

## Declared types

`epok::ScreenAnchorComponent`, `epok::ScreenAnchorStatus`

## Callable index

- [`epok::ScreenAnchorComponent::class_id`](#epok-screenanchorcomponent-class-id-1) — Performs `class id` as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::end_play`](#epok-screenanchorcomponent-end-play-1) — Ends play as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::on_disable`](#epok-screenanchorcomponent-on-disable-1) — Performs `on disable` as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::on_frame`](#epok-screenanchorcomponent-on-frame-1) — Performs `on frame` as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::refresh`](#epok-screenanchorcomponent-refresh-1) — Performs `refresh` as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::set_target`](#epok-screenanchorcomponent-set-target-1) — Sets target as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::status`](#epok-screenanchorcomponent-status-1) — Performs `status` as part of the ui anchor module.
- [`epok::ScreenAnchorComponent::valid_content`](#epok-screenanchorcomponent-valid-content-1) — Performs `valid content` as part of the ui anchor module.
- [`epok::ui_anchor::update`](#epok-ui-anchor-update-1) — Updates update as part of the ui anchor module.
- [`epok::ui_anchor::valid_content`](#epok-ui-anchor-valid-content-1) — Performs `valid content` as part of the ui anchor module.

<a id="epok-screenanchorcomponent-class-id-1"></a>

## `epok::ScreenAnchorComponent::class_id`

**Purpose.** Performs `class id` as part of the ui anchor module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 16](../../../runtime/ui_anchor.hpp#L16)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-end-play-1"></a>

## `epok::ScreenAnchorComponent::end_play`

**Purpose.** Ends play as part of the ui anchor module.

**Exact declaration**

```cpp
void end_play(EndPlayReason) override
```

- **Declared at:** [line 32](../../../runtime/ui_anchor.hpp#L32)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `EndPlayReason` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

// Assume these named values have been initialized with valid data:
// EndPlayReason arg1

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

object.end_play(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-on-disable-1"></a>

## `epok::ScreenAnchorComponent::on_disable`

**Purpose.** Performs `on disable` as part of the ui anchor module.

**Exact declaration**

```cpp
void on_disable() override
```

- **Declared at:** [line 31](../../../runtime/ui_anchor.hpp#L31)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

object.on_disable();
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-on-frame-1"></a>

## `epok::ScreenAnchorComponent::on_frame`

**Purpose.** Performs `on frame` as part of the ui anchor module.

**Exact declaration**

```cpp
void on_frame(uint32_t) override
```

- **Declared at:** [line 30](../../../runtime/ui_anchor.hpp#L30)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `uint32_t` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t arg1

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

object.on_frame(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-refresh-1"></a>

## `epok::ScreenAnchorComponent::refresh`

**Purpose.** Performs `refresh` as part of the ui anchor module.

**Exact declaration**

```cpp
ScreenAnchorStatus refresh()
```

- **Declared at:** [line 27](../../../runtime/ui_anchor.hpp#L27)
- **Kind:** `cxx method`

**Returns.** Returns `ScreenAnchorStatus`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

auto result = object.refresh();
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-set-target-1"></a>

## `epok::ScreenAnchorComponent::set_target`

**Purpose.** Sets target as part of the ui anchor module.

**Exact declaration**

```cpp
void set_target(ObjectId value)
```

- **Declared at:** [line 28](../../../runtime/ui_anchor.hpp#L28)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `ObjectId` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId value

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

object.set_target(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-status-1"></a>

## `epok::ScreenAnchorComponent::status`

**Purpose.** Performs `status` as part of the ui anchor module.

**Exact declaration**

```cpp
ScreenAnchorStatus status() const
```

- **Declared at:** [line 29](../../../runtime/ui_anchor.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `ScreenAnchorStatus`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

auto result = object.status();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-screenanchorcomponent-valid-content-1"></a>

## `epok::ScreenAnchorComponent::valid_content`

**Purpose.** Performs `valid content` as part of the ui anchor module.

**Exact declaration**

```cpp
bool valid_content() const
```

- **Declared at:** [line 26](../../../runtime/ui_anchor.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

epok::ScreenAnchorComponent& object = /* obtain a valid instance */;

auto result = object.valid_content();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-ui-anchor-update-1"></a>

## `epok::ui_anchor::update`

**Purpose.** Updates update as part of the ui anchor module.

**Exact declaration**

```cpp
ScreenAnchorStatus update(ScreenAnchorComponent& anchor)
```

- **Declared at:** [line 10](../../../runtime/ui_anchor.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `anchor` | `ScreenAnchorComponent &` | Input/output; inspect the function contract | Value supplied for `anchor`. See the exact type and module contract. |

**Returns.** Returns `ScreenAnchorStatus`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

// Assume these named values have been initialized with valid data:
// ScreenAnchorComponent & anchor

auto result = epok::ui_anchor::update(anchor);
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-valid-content-1"></a>

## `epok::ui_anchor::valid_content`

**Purpose.** Performs `valid content` as part of the ui anchor module.

**Exact declaration**

```cpp
bool valid_content(const ScreenAnchorComponent& anchor)
```

- **Declared at:** [line 9](../../../runtime/ui_anchor.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `anchor` | `const ScreenAnchorComponent &` | Input | Value supplied for `anchor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor.hpp"

// Assume these named values have been initialized with valid data:
// const ScreenAnchorComponent & anchor

auto result = epok::ui_anchor::valid_content(anchor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
