# Epok API: Blueprint Spawn

> **Header:** `"blueprint_spawn.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/blueprint_spawn.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 22 public callables declared directly in this header.

## Declared types

`epok::bp::ClassId`, `epok::bp::ClassInfo`, `epok::bp::DispatchScope`, `epok::bp::DynamicBinding`, `epok::bp::SpawnStats`, `epok::bp::TypedPool`

## Callable index

- [`epok::bp::activate_spawn_audio`](#epok-bp-activate-spawn-audio-1) — Performs `activate spawn audio` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::authored_behaviour`](#epok-bp-authored-behaviour-1) — Performs `authored behaviour` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::authored_class`](#epok-bp-authored-class-1) — Performs `authored class` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::behaviour`](#epok-bp-behaviour-1) — Performs `behaviour` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::class_is_a`](#epok-bp-class-is-a-1) — Performs `class is a` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::DispatchScope::DispatchScope`](#epok-bp-dispatchscope-dispatchscope-1) — Constructs `epok::bp::DispatchScope` for compiled Blueprint execution and object interaction.
- [`epok::bp::DispatchScope::DispatchScope`](#epok-bp-dispatchscope-dispatchscope-2) — Constructs `epok::bp::DispatchScope` for compiled Blueprint execution and object interaction.
- [`epok::bp::DispatchScope::operator=`](#epok-bp-dispatchscope-operator-1) — Performs `operator =` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::DispatchScope::~DispatchScope`](#epok-bp-dispatchscope-dispatchscope-3) — Releases the resources owned by `epok::bp::DispatchScope`.
- [`epok::bp::find_binding`](#epok-bp-find-binding-1) — Finds binding as part of compiled Blueprint execution and object interaction.
- [`epok::bp::find_class`](#epok-bp-find-class-1) — Finds class as part of compiled Blueprint execution and object interaction.
- [`epok::bp::finish_release`](#epok-bp-finish-release-1) — Performs `finish release` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::is_a`](#epok-bp-is-a-1) — Reports whether a as part of compiled Blueprint execution and object interaction.
- [`epok::bp::reserve`](#epok-bp-reserve-1) — Reserve binds a fresh typed instance but performs no construction callbacks or gameplay dispatch.
- [`epok::bp::retire`](#epok-bp-retire-1) — Performs `retire` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::slot_quarantined`](#epok-bp-slot-quarantined-1) — Performs `slot quarantined` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::spawn`](#epok-bp-spawn-1) — Performs `spawn` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::start_batch`](#epok-bp-start-batch-1) — Starts batch as part of compiled Blueprint execution and object interaction.
- [`epok::bp::start_reserved`](#epok-bp-start-reserved-1) — Starts reserved as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TypedPool::acquire`](#epok-bp-typedpool-acquire-1) — Performs `acquire` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::TypedPool::release`](#epok-bp-typedpool-release-1) — Performs `release` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::visit`](#epok-bp-visit-1) — Visit only bindings that existed at dispatch start.

<a id="epok-bp-activate-spawn-audio-1"></a>

## `epok::bp::activate_spawn_audio`

**Purpose.** Performs `activate spawn audio` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
void activate_spawn_audio(EntityHandle root)
```

- **Declared at:** [line 90](../../../runtime/blueprint_spawn.hpp#L90)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `EntityHandle` | Input | Value supplied for `root`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle root

epok::bp::activate_spawn_audio(root);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.

<a id="epok-bp-authored-behaviour-1"></a>

## `epok::bp::authored_behaviour`

**Purpose.** Performs `authored behaviour` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
Behaviour* authored_behaviour(EntityHandle owner)
```

- **Declared at:** [line 88](../../../runtime/blueprint_spawn.hpp#L88)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Behaviour *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::bp::authored_behaviour(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-authored-class-1"></a>

## `epok::bp::authored_class`

**Purpose.** Performs `authored class` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
ClassId authored_class(EntityHandle owner)
```

- **Declared at:** [line 89](../../../runtime/blueprint_spawn.hpp#L89)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `ClassId`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::bp::authored_class(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-behaviour-1"></a>

## `epok::bp::behaviour`

**Purpose.** Performs `behaviour` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Behaviour* behaviour(EntityHandle owner)
```

- **Declared at:** [line 91](../../../runtime/blueprint_spawn.hpp#L91)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `Behaviour *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::bp::behaviour(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-class-is-a-1"></a>

## `epok::bp::class_is_a`

**Purpose.** Performs `class is a` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool class_is_a(ClassId child, ClassId parent)
```

- **Declared at:** [line 39](../../../runtime/blueprint_spawn.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `child` | `ClassId` | Input | Value supplied for `child`. See the exact type and module contract. |
| `parent` | `ClassId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ClassId child
// ClassId parent

auto result = epok::bp::class_is_a(child, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-dispatchscope-dispatchscope-1"></a>

## `epok::bp::DispatchScope::DispatchScope`

**Purpose.** Constructs `epok::bp::DispatchScope` for compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
DispatchScope(const DispatchScope&) = delete
```

- **Declared at:** [line 74](../../../runtime/blueprint_spawn.hpp#L74)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const DispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// const DispatchScope & arg1

epok::bp::DispatchScope value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-dispatchscope-dispatchscope-2"></a>

## `epok::bp::DispatchScope::DispatchScope`

**Purpose.** Constructs `epok::bp::DispatchScope` for compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
explicit DispatchScope(DynamicBinding& value) : b
```

- **Declared at:** [line 72](../../../runtime/blueprint_spawn.hpp#L72)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `DynamicBinding &` | Input/output; inspect the function contract | Value supplied for `value`. See the exact type and module contract. |

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// DynamicBinding & value

epok::bp::DispatchScope value(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-dispatchscope-operator-1"></a>

## `epok::bp::DispatchScope::operator=`

**Purpose.** Performs `operator =` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
DispatchScope& operator=(const DispatchScope&) = delete
```

- **Declared at:** [line 75](../../../runtime/blueprint_spawn.hpp#L75)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const DispatchScope &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `DispatchScope &`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// const DispatchScope & arg1

epok::bp::DispatchScope& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-dispatchscope-dispatchscope-3"></a>

## `epok::bp::DispatchScope::~DispatchScope`

**Purpose.** Releases the resources owned by `epok::bp::DispatchScope`.

**Exact declaration**

```cpp
~DispatchScope()
```

- **Declared at:** [line 73](../../../runtime/blueprint_spawn.hpp#L73)
- **Kind:** `destructor`

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// `epok::bp::DispatchScope` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-find-binding-1"></a>

## `epok::bp::find_binding`

**Purpose.** Finds binding as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline DynamicBinding* find_binding(EntityHandle owner)
```

- **Declared at:** [line 82](../../../runtime/blueprint_spawn.hpp#L82)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `DynamicBinding *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

auto result = epok::bp::find_binding(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-find-class-1"></a>

## `epok::bp::find_class`

**Purpose.** Finds class as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline const ClassInfo* find_class(ClassId id)
```

- **Declared at:** [line 35](../../../runtime/blueprint_spawn.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ClassId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassInfo *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ClassId id

auto result = epok::bp::find_class(id);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-finish-release-1"></a>

## `epok::bp::finish_release`

**Purpose.** Performs `finish release` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void finish_release(DynamicBinding& binding)
```

- **Declared at:** [line 62](../../../runtime/blueprint_spawn.hpp#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `binding` | `DynamicBinding &` | Input/output; inspect the function contract | Value supplied for `binding`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// DynamicBinding & binding

epok::bp::finish_release(binding);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-is-a-1"></a>

## `epok::bp::is_a`

**Purpose.** Reports whether a as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool is_a(EntityHandle owner, ClassId parent)
```

- **Declared at:** [line 94](../../../runtime/blueprint_spawn.hpp#L94)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |
| `parent` | `ClassId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner
// ClassId parent

auto result = epok::bp::is_a(owner, parent);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-reserve-1"></a>

## `epok::bp::reserve`

**Purpose.** Reserve binds a fresh typed instance but performs no construction callbacks or gameplay dispatch.

**Details.** Template assembly uses one explicit batch before start.

**Exact declaration**

```cpp
inline EntityHandle reserve(ClassId id, const char* name, Entity* parent = nullptr, EntityHandle batch = {})
```

- **Declared at:** [line 134](../../../runtime/blueprint_spawn.hpp#L134)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ClassId` | Input | Value supplied for `id`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `parent` | `Entity *` | Input/output; inspect the function contract | Value supplied for `parent`. See the exact type and module contract. |
| `batch` | `EntityHandle` | Input | Value supplied for `batch`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** Template assembly uses one explicit batch before start.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ClassId id
// const char * name
// Entity * parent
// EntityHandle batch

auto result = epok::bp::reserve(id, name, parent, batch);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-retire-1"></a>

## `epok::bp::retire`

**Purpose.** Performs `retire` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void retire(const bool* doomed, const bool* active, size_t count)
```

- **Declared at:** [line 112](../../../runtime/blueprint_spawn.hpp#L112)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `doomed` | `const bool *` | Input | Value supplied for `doomed`. See the exact type and module contract. |
| `active` | `const bool *` | Input | Value supplied for `active`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// const bool * doomed
// const bool * active
// size_t count

epok::bp::retire(doomed, active, count);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-slot-quarantined-1"></a>

## `epok::bp::slot_quarantined`

**Purpose.** Performs `slot quarantined` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool slot_quarantined(size_t index)
```

- **Declared at:** [line 77](../../../runtime/blueprint_spawn.hpp#L77)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

auto result = epok::bp::slot_quarantined(index);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-spawn-1"></a>

## `epok::bp::spawn`

**Purpose.** Performs `spawn` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline EntityHandle spawn(ClassId id, const char* name, Entity* parent = nullptr)
```

- **Declared at:** [line 168](../../../runtime/blueprint_spawn.hpp#L168)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ClassId` | Input | Value supplied for `id`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `parent` | `Entity *` | Input/output; inspect the function contract | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ClassId id
// const char * name
// Entity * parent

auto result = epok::bp::spawn(id, name, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-start-batch-1"></a>

## `epok::bp::start_batch`

**Purpose.** Starts batch as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void start_batch(EntityHandle root)
```

- **Declared at:** [line 162](../../../runtime/blueprint_spawn.hpp#L162)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `EntityHandle` | Input | Value supplied for `root`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle root

epok::bp::start_batch(root);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-start-reserved-1"></a>

## `epok::bp::start_reserved`

**Purpose.** Starts reserved as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline void start_reserved(EntityHandle owner)
```

- **Declared at:** [line 155](../../../runtime/blueprint_spawn.hpp#L155)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `EntityHandle` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle owner

epok::bp::start_reserved(owner);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-typedpool-acquire-1"></a>

## `epok::bp::TypedPool::acquire`

**Purpose.** Performs `acquire` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
static Behaviour* acquire()
```

- **Declared at:** [line 23](../../../runtime/blueprint_spawn.hpp#L23)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `Behaviour *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

auto result = epok::bp::TypedPool::acquire();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-typedpool-release-1"></a>

## `epok::bp::TypedPool::release`

**Purpose.** Performs `release` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
static void release(Behaviour* instance)
```

- **Declared at:** [line 29](../../../runtime/blueprint_spawn.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `instance` | `Behaviour *` | Input/output; inspect the function contract | Value supplied for `instance`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// Behaviour * instance

epok::bp::TypedPool::release(instance);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-visit-1"></a>

## `epok::bp::visit`

**Purpose.** Visit only bindings that existed at dispatch start.

**Details.** Generations and serials prevent a callback-created replacement from receiving the old slot's event.

**Exact declaration**

```cpp
template<class Callback> inline void visit(Callback callback)
```

- **Declared at:** [line 101](../../../runtime/blueprint_spawn.hpp#L101)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `callback` | `Callback` | Callback | Value supplied for `callback`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Generations and serials prevent a callback-created replacement from receiving the old slot's event.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Callback

// Assume these named values have been initialized with valid data:
// Callback callback

epok::bp& object = /* obtain a valid instance */;

object.visit<Callback>(callback);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.
