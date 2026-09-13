# PsyQo API: Primitives / Control

> **Header:** `"psyqo/primitives/control.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh)

This module covers typed PlayStation GPU primitives. It documents 14 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Prim::DrawingAreaEnd`, `psyqo::Prim::DrawingAreaStart`, `psyqo::Prim::DrawingOffset`, `psyqo::Prim::MaskControl`, `psyqo::Prim::MaskControl::Set`, `psyqo::Prim::MaskControl::Test`, `psyqo::Prim::Scissor`, `psyqo::Prim::TPage`, `psyqo::Prim::VRAMUpload`

## Callable index

- [`psyqo::Prim::DrawingAreaEnd::DrawingAreaEnd`](#psyqo-prim-drawingareaend-drawingareaend-1) — Constructs `psyqo::Prim::DrawingAreaEnd` for typed PlayStation GPU primitives.
- [`psyqo::Prim::DrawingAreaStart::DrawingAreaStart`](#psyqo-prim-drawingareastart-drawingareastart-1) — Constructs `psyqo::Prim::DrawingAreaStart` for typed PlayStation GPU primitives.
- [`psyqo::Prim::DrawingOffset::DrawingOffset`](#psyqo-prim-drawingoffset-drawingoffset-1) — Constructs `psyqo::Prim::DrawingOffset` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::MaskControl`](#psyqo-prim-maskcontrol-maskcontrol-1) — Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::MaskControl`](#psyqo-prim-maskcontrol-maskcontrol-2) — Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::MaskControl`](#psyqo-prim-maskcontrol-maskcontrol-3) — Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::MaskControl`](#psyqo-prim-maskcontrol-maskcontrol-4) — Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::MaskControl`](#psyqo-prim-maskcontrol-maskcontrol-5) — Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::set`](#psyqo-prim-maskcontrol-set-1) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::set`](#psyqo-prim-maskcontrol-set-2) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::set`](#psyqo-prim-maskcontrol-set-3) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::MaskControl::set`](#psyqo-prim-maskcontrol-set-4) — Sets set as part of typed PlayStation GPU primitives.
- [`psyqo::Prim::TPage::TPage`](#psyqo-prim-tpage-tpage-1) — Constructs `psyqo::Prim::TPage` for typed PlayStation GPU primitives.
- [`psyqo::Prim::VRAMUpload::VRAMUpload`](#psyqo-prim-vramupload-vramupload-1) — Constructs `psyqo::Prim::VRAMUpload` for typed PlayStation GPU primitives.

<a id="psyqo-prim-drawingareaend-drawingareaend-1"></a>

## `psyqo::Prim::DrawingAreaEnd::DrawingAreaEnd`

**Purpose.** Constructs `psyqo::Prim::DrawingAreaEnd` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
DrawingAreaEnd(Vertex p) : c
```

- **Declared at:** [line 75](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L75)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `Vertex` | Input | Value supplied for `p`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Vertex p

psyqo::Prim::DrawingAreaEnd value(p);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-prim-drawingareastart-drawingareastart-1"></a>

## `psyqo::Prim::DrawingAreaStart::DrawingAreaStart`

**Purpose.** Constructs `psyqo::Prim::DrawingAreaStart` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
DrawingAreaStart(Vertex p) : c
```

- **Declared at:** [line 57](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L57)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `Vertex` | Input | Value supplied for `p`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Vertex p

psyqo::Prim::DrawingAreaStart value(p);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-prim-drawingoffset-drawingoffset-1"></a>

## `psyqo::Prim::DrawingOffset::DrawingOffset`

**Purpose.** Constructs `psyqo::Prim::DrawingOffset` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
DrawingOffset(Vertex p) : c
```

- **Declared at:** [line 93](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L93)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `Vertex` | Input | Value supplied for `p`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Vertex p

psyqo::Prim::DrawingOffset value(p);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-prim-maskcontrol-maskcontrol-1"></a>

## `psyqo::Prim::MaskControl::MaskControl`

**Purpose.** Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl() : c
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L123)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

psyqo::Prim::MaskControl value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-maskcontrol-2"></a>

## `psyqo::Prim::MaskControl::MaskControl`

**Purpose.** Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl(Set set) : M
```

- **Declared at:** [line 126](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L126)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Set set

psyqo::Prim::MaskControl value(set);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-maskcontrol-3"></a>

## `psyqo::Prim::MaskControl::MaskControl`

**Purpose.** Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl(Set set, Test test) : c
```

- **Declared at:** [line 124](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L124)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Set set
// Test test

psyqo::Prim::MaskControl value(set, test);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-maskcontrol-4"></a>

## `psyqo::Prim::MaskControl::MaskControl`

**Purpose.** Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl(Test test) : M
```

- **Declared at:** [line 127](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L127)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Test test

psyqo::Prim::MaskControl value(test);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-maskcontrol-5"></a>

## `psyqo::Prim::MaskControl::MaskControl`

**Purpose.** Constructs `psyqo::Prim::MaskControl` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl(Test test, Set set) : M
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L125)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Test test
// Set set

psyqo::Prim::MaskControl value(test, set);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-set-1"></a>

## `psyqo::Prim::MaskControl::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl &set(Set set)
```

- **Declared at:** [line 128](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |

**Returns.** Returns `MaskControl &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Set set

psyqo::Prim::MaskControl& object = /* obtain a valid instance */;

auto result = object.set(set);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-set-2"></a>

## `psyqo::Prim::MaskControl::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl &set(Set set, Test test)
```

- **Declared at:** [line 138](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L138)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |

**Returns.** Returns `MaskControl &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Set set
// Test test

psyqo::Prim::MaskControl& object = /* obtain a valid instance */;

auto result = object.set(set, test);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-set-3"></a>

## `psyqo::Prim::MaskControl::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl &set(Test test)
```

- **Declared at:** [line 133](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L133)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |

**Returns.** Returns `MaskControl &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Test test

psyqo::Prim::MaskControl& object = /* obtain a valid instance */;

auto result = object.set(test);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-maskcontrol-set-4"></a>

## `psyqo::Prim::MaskControl::set`

**Purpose.** Sets set as part of typed PlayStation GPU primitives.

**Exact declaration**

```cpp
MaskControl &set(Test test, Set set)
```

- **Declared at:** [line 139](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L139)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `test` | `Test` | Input | Value supplied for `test`. See the exact type and module contract. |
| `set` | `Set` | Input | Value supplied for `set`. See the exact type and module contract. |

**Returns.** Returns `MaskControl &`. Check the purpose and failure notes before using the value.

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

// Assume these named values have been initialized with valid data:
// Test test
// Set set

psyqo::Prim::MaskControl& object = /* obtain a valid instance */;

auto result = object.set(test, set);
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-tpage-tpage-1"></a>

## `psyqo::Prim::TPage::TPage`

**Purpose.** Constructs `psyqo::Prim::TPage` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
TPage() : c
```

- **Declared at:** [line 38](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L38)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

psyqo::Prim::TPage value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-prim-vramupload-vramupload-1"></a>

## `psyqo::Prim::VRAMUpload::VRAMUpload`

**Purpose.** Constructs `psyqo::Prim::VRAMUpload` for typed PlayStation GPU primitives.

**Exact declaration**

```cpp
VRAMUpload() : c
```

- **Declared at:** [line 169](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/primitives/control.hh#L169)
- **Kind:** `constructor`

**Use it when.** You need typed PlayStation GPU primitives and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/primitives/control.hh"

psyqo::Prim::VRAMUpload value();
```

**Why choose it.** It provides direct, allocation-conscious access to typed PlayStation GPU primitives. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
