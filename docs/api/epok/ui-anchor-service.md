# Epok API: Ui Anchor Service

> **Header:** `"ui_anchor_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/ui_anchor_service.hpp)

This module covers the ui anchor service module. It documents 5 public callables declared directly in this header.

## Declared types

`epok::ui_anchor::detail::Content`

## Callable index

- [`epok::ui_anchor::detail::add`](#epok-ui-anchor-detail-add-1) — Adds add as part of the ui anchor service module.
- [`epok::ui_anchor::detail::content`](#epok-ui-anchor-detail-content-1) — Performs `content` as part of the ui anchor service module.
- [`epok::ui_anchor::detail::fits`](#epok-ui-anchor-detail-fits-1) — Performs `fits` as part of the ui anchor service module.
- [`epok::ui_anchor::detail::live`](#epok-ui-anchor-detail-live-1) — Performs `live` as part of the ui anchor service module.
- [`epok::ui_anchor::detail::project`](#epok-ui-anchor-detail-project-1) — Performs `project` as part of the ui anchor service module.

<a id="epok-ui-anchor-detail-add-1"></a>

## `epok::ui_anchor::detail::add`

**Purpose.** Adds add as part of the ui anchor service module.

**Exact declaration**

```cpp
inline bool add(Fixed a,Fixed b,Fixed& out)
```

- **Declared at:** [line 16](../../../runtime/ui_anchor_service.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `Fixed` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `Fixed` | Input | Value supplied for `b`. See the exact type and module contract. |
| `out` | `Fixed &` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor_service.hpp"

// Assume these named values have been initialized with valid data:
// Fixed a
// Fixed b
// Fixed & out

auto result = epok::ui_anchor::detail::add(a, b, out);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-detail-content-1"></a>

## `epok::ui_anchor::detail::content`

**Purpose.** Performs `content` as part of the ui anchor service module.

**Exact declaration**

```cpp
inline Content content(const ScreenAnchorComponent& anchor)
```

- **Declared at:** [line 21](../../../runtime/ui_anchor_service.hpp#L21)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `anchor` | `const ScreenAnchorComponent &` | Input | Value supplied for `anchor`. See the exact type and module contract. |

**Returns.** Returns `Content`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor_service.hpp"

// Assume these named values have been initialized with valid data:
// const ScreenAnchorComponent & anchor

auto result = epok::ui_anchor::detail::content(anchor);
```

**Why choose it.** It provides direct, allocation-conscious access to the ui anchor service module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-detail-fits-1"></a>

## `epok::ui_anchor::detail::fits`

**Purpose.** Performs `fits` as part of the ui anchor service module.

**Exact declaration**

```cpp
inline bool fits(const Content& box,Fixed margin)
```

- **Declared at:** [line 49](../../../runtime/ui_anchor_service.hpp#L49)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `box` | `const Content &` | Input | Value supplied for `box`. See the exact type and module contract. |
| `margin` | `Fixed` | Input | Value supplied for `margin`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor_service.hpp"

// Assume these named values have been initialized with valid data:
// const Content & box
// Fixed margin

auto result = epok::ui_anchor::detail::fits(box, margin);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-detail-live-1"></a>

## `epok::ui_anchor::detail::live`

**Purpose.** Performs `live` as part of the ui anchor service module.

**Exact declaration**

```cpp
inline bool live(Actor* actor)
```

- **Declared at:** [line 10](../../../runtime/ui_anchor_service.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor_service.hpp"

// Assume these named values have been initialized with valid data:
// Actor * actor

auto result = epok::ui_anchor::detail::live(actor);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ui-anchor-detail-project-1"></a>

## `epok::ui_anchor::detail::project`

**Purpose.** Performs `project` as part of the ui anchor service module.

**Exact declaration**

```cpp
inline bool project(const ScreenAnchorComponent& anchor,Fixed* screen)
```

- **Declared at:** [line 55](../../../runtime/ui_anchor_service.hpp#L55)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `anchor` | `const ScreenAnchorComponent &` | Input | Value supplied for `anchor`. See the exact type and module contract. |
| `screen` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ui anchor service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ui_anchor_service.hpp"

// Assume these named values have been initialized with valid data:
// const ScreenAnchorComponent & anchor
// Fixed * screen

auto result = epok::ui_anchor::detail::project(anchor, screen);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
