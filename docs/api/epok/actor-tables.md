# Epok API: Actor Tables

> **Header:** `"actor_tables.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/actor_tables.hpp)

This module covers the actor tables module. It documents 12 public callables declared directly in this header.

## Declared types

`epok::ActorComponentRecord`, `epok::ActorPrototype`, `epok::ActorRecord`, `epok::ActorStats`, `epok::ActorTable`, `epok::SceneLevel`, `epok::SceneLevel::Context`, `epok::SceneReferenceRecord`, `epok::SceneRefKind`

## Callable index

- [`epok::actor_for_slot`](#epok-actor-for-slot-1) — Performs `actor for slot` as part of the actor tables module.
- [`epok::collect_object_quarantine`](#epok-collect-object-quarantine-1) — Performs `collect object quarantine` as part of the actor tables module.
- [`epok::dispatch_slot_trigger`](#epok-dispatch-slot-trigger-1) — Performs `dispatch slot trigger` as part of the actor tables module.
- [`epok::load_actor_bank`](#epok-load-actor-bank-1) — Loads actor bank as part of the actor tables module.
- [`epok::SceneLevel::add_component_by_class`](#epok-scenelevel-add-component-by-class-1) — Adds component by class as part of the actor tables module.
- [`epok::SceneLevel::bind_scene_references`](#epok-scenelevel-bind-scene-references-1) — Performs `bind scene references` as part of the actor tables module.
- [`epok::SceneLevel::create_bank_scene_script`](#epok-scenelevel-create-bank-scene-script-1) — Creates bank scene script as part of the actor tables module.
- [`epok::SceneLevel::ensure_bound`](#epok-scenelevel-ensure-bound-1) — Performs `ensure bound` as part of the actor tables module.
- [`epok::SceneLevel::load_bank`](#epok-scenelevel-load-bank-1) — Loads bank as part of the actor tables module.
- [`epok::SceneLevel::refresh_stats`](#epok-scenelevel-refresh-stats-1) — Performs `refresh stats` as part of the actor tables module.
- [`epok::SceneLevel::spawn_actor`](#epok-scenelevel-spawn-actor-1) — Performs `spawn actor` as part of the actor tables module.
- [`epok::unload_actor_bank`](#epok-unload-actor-bank-1) — Performs `unload actor bank` as part of the actor tables module.

<a id="epok-actor-for-slot-1"></a>

## `epok::actor_for_slot`

**Purpose.** Performs `actor for slot` as part of the actor tables module.

**Exact declaration**

```cpp
inline ObjectId actor_for_slot(const ActorData* data)
```

- **Declared at:** [line 159](../../../runtime/actor_tables.hpp#L159)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const ActorData *` | Input | Value supplied for `data`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * data

auto result = epok::actor_for_slot(data);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-collect-object-quarantine-1"></a>

## `epok::collect_object_quarantine`

**Purpose.** Performs `collect object quarantine` as part of the actor tables module.

**Exact declaration**

```cpp
inline void collect_object_quarantine()
```

- **Declared at:** [line 158](../../../runtime/actor_tables.hpp#L158)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

epok::collect_object_quarantine();
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-dispatch-slot-trigger-1"></a>

## `epok::dispatch_slot_trigger`

**Purpose.** Performs `dispatch slot trigger` as part of the actor tables module.

**Exact declaration**

```cpp
inline size_t dispatch_slot_trigger(DataHandle self,DataHandle other,TriggerPhase phase)
```

- **Declared at:** [line 160](../../../runtime/actor_tables.hpp#L160)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `self` | `DataHandle` | Input | Value supplied for `self`. See the exact type and module contract. |
| `other` | `DataHandle` | Input | Value supplied for `other`. See the exact type and module contract. |
| `phase` | `TriggerPhase` | Input | Value supplied for `phase`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle self
// DataHandle other
// TriggerPhase phase

auto result = epok::dispatch_slot_trigger(self, other, phase);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-load-actor-bank-1"></a>

## `epok::load_actor_bank`

**Purpose.** Loads actor bank as part of the actor tables module.

**Exact declaration**

```cpp
inline size_t load_actor_bank(const ActorTable& table,ActorData* slots,size_t count)
```

- **Declared at:** [line 156](../../../runtime/actor_tables.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `slots` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `slots`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table
// ActorData * slots
// int count

auto result = epok::load_actor_bank(table, slots, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-add-component-by-class-1"></a>

## `epok::SceneLevel::add_component_by_class`

**Purpose.** Adds component by class as part of the actor tables module.

**Exact declaration**

```cpp
ObjectId add_component_by_class(Actor& actor,const ClassDescriptor& type,const char* name)
```

- **Declared at:** [line 33](../../../runtime/actor_tables.hpp#L33)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor &` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// Actor & actor
// const ClassDescriptor & type
// const char * name

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.add_component_by_class(actor, type, name);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-bind-scene-references-1"></a>

## `epok::SceneLevel::bind_scene_references`

**Purpose.** Performs `bind scene references` as part of the actor tables module.

**Exact declaration**

```cpp
size_t bind_scene_references(const ActorTable& table,ObjectId owner)
```

- **Declared at:** [line 75](../../../runtime/actor_tables.hpp#L75)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `owner` | `ObjectId` | Input | Value supplied for `owner`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table
// ObjectId owner

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.bind_scene_references(table, owner);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-create-bank-scene-script-1"></a>

## `epok::SceneLevel::create_bank_scene_script`

**Purpose.** Creates bank scene script as part of the actor tables module.

**Exact declaration**

```cpp
ObjectId create_bank_scene_script(const ActorTable& table)
```

- **Declared at:** [line 68](../../../runtime/actor_tables.hpp#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.create_bank_scene_script(table);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-ensure-bound-1"></a>

## `epok::SceneLevel::ensure_bound`

**Purpose.** Performs `ensure bound` as part of the actor tables module.

**Exact declaration**

```cpp
bool ensure_bound(ObjectRegistry& registry)
```

- **Declared at:** [line 32](../../../runtime/actor_tables.hpp#L32)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `registry` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `registry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// ObjectRegistry & registry

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.ensure_bound(registry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-load-bank-1"></a>

## `epok::SceneLevel::load_bank`

**Purpose.** Loads bank as part of the actor tables module.

**Exact declaration**

```cpp
size_t load_bank(const ActorTable& table,ActorData* slots,size_t count)
```

- **Declared at:** [line 40](../../../runtime/actor_tables.hpp#L40)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `slots` | `ActorData *` | Input/output; inspect the function contract | Value supplied for `slots`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table
// ActorData * slots
// int count

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.load_bank(table, slots, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-refresh-stats-1"></a>

## `epok::SceneLevel::refresh_stats`

**Purpose.** Performs `refresh stats` as part of the actor tables module.

**Exact declaration**

```cpp
void refresh_stats()
```

- **Declared at:** [line 85](../../../runtime/actor_tables.hpp#L85)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

epok::SceneLevel& object = /* obtain a valid instance */;

object.refresh_stats();
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-scenelevel-spawn-actor-1"></a>

## `epok::SceneLevel::spawn_actor`

**Purpose.** Performs `spawn actor` as part of the actor tables module.

**Exact declaration**

```cpp
ObjectId spawn_actor(const ClassDescriptor& type,const char* name,ObjectId parent={}) override
```

- **Declared at:** [line 48](../../../runtime/actor_tables.hpp#L48)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |
| `parent` | `ObjectId` | Input | Value supplied for `parent`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ClassDescriptor & type
// const char * name
// ObjectId parent

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.spawn_actor(type, name, parent);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-unload-actor-bank-1"></a>

## `epok::unload_actor_bank`

**Purpose.** Performs `unload actor bank` as part of the actor tables module.

**Exact declaration**

```cpp
inline void unload_actor_bank()
```

- **Declared at:** [line 157](../../../runtime/actor_tables.hpp#L157)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

epok::unload_actor_bank();
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
