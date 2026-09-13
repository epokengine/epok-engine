# Epok API: Actor Tables

> **Header:** `"actor_tables.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/actor_tables.hpp)

This module covers the actor tables module. It documents 7 public callables declared directly in this header.

## Declared types

`epok::ActorComponentRecord`, `epok::ActorRecord`, `epok::ActorStats`, `epok::ActorTable`, `epok::SceneLevel`

## Callable index

- [`epok::load_actor_bank`](#epok-load-actor-bank-1) — Called by the generated load_bank_N() before the legacy initialize_scripts().
- [`epok::SceneLevel::add_component_by_class`](#epok-scenelevel-add-component-by-class-1) — Adds a component named by its cooked class id.
- [`epok::SceneLevel::create_bank_scene_script`](#epok-scenelevel-create-bank-scene-script-1) — Cooked scene script for the bank.
- [`epok::SceneLevel::ensure_bound`](#epok-scenelevel-ensure-bound-1) — Binds the slot table once.
- [`epok::SceneLevel::load_bank`](#epok-scenelevel-load-bank-1) — Loads one cooked bank.
- [`epok::SceneLevel::refresh_stats`](#epok-scenelevel-refresh-stats-1) — Snapshot of the registry and level counters for tools/profile_runtime.py.
- [`epok::unload_actor_bank`](#epok-unload-actor-bank-1) — Called by scene_tick() before the legacy binding teardown, while the actors and their legacy slots are still alive.

<a id="epok-load-actor-bank-1"></a>

## `epok::load_actor_bank`

**Purpose.** Called by the generated load_bank_N() before the legacy initialize_scripts().

**Exact declaration**

```cpp
inline size_t load_actor_bank(const ActorTable& table, Entity* slots, size_t slot_count)
```

- **Declared at:** [line 255](../../../runtime/actor_tables.hpp#L255)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `slots` | `Entity *` | Input/output; inspect the function contract | Value supplied for `slots`. See the exact type and module contract. |
| `slot_count` | `size_t` | Input | Value supplied for `slot_count`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the actor tables module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table
// Entity * slots
// size_t slot_count

auto result = epok::load_actor_bank(table, slots, slot_count);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-add-component-by-class-1"></a>

## `epok::SceneLevel::add_component_by_class`

**Purpose.** Adds a component named by its cooked class id.

**Details.** Mirrors Level::add_component<T> (same owner-domain, cardinality, abstractness and capacity rules) without needing the C++ type at the call site. begin_play is deferred to finish_components().

**Exact declaration**

```cpp
ObjectId add_component_by_class(Actor& owner, const ClassDescriptor& type, const char* name)
```

- **Declared at:** [line 86](../../../runtime/actor_tables.hpp#L86)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `owner` | `Actor &` | Input/output; inspect the function contract | Value supplied for `owner`. See the exact type and module contract. |
| `type` | `const ClassDescriptor &` | Input | Value supplied for `type`. See the exact type and module contract. |
| `name` | `const char *` | Input | Value supplied for `name`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Mirrors Level::add_component<T> (same owner-domain, cardinality, abstractness and capacity rules) without needing the C++ type at the call site. begin_play is deferred to finish_components().

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// Actor & owner
// const ClassDescriptor & type
// const char * name

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.add_component_by_class(owner, type, name);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-create-bank-scene-script-1"></a>

## `epok::SceneLevel::create_bank_scene_script`

**Purpose.** Cooked scene script for the bank.

**Details.** Exactly one per loaded level: the cooked class when the map authored a scene Blueprint, the runtime base otherwise.

**Exact declaration**

```cpp
ObjectId create_bank_scene_script(const ActorTable& table)
```

- **Declared at:** [line 149](../../../runtime/actor_tables.hpp#L149)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** Exactly one per loaded level: the cooked class when the map authored a scene Blueprint, the runtime base otherwise.

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

**Purpose.** Binds the slot table once.

**Details.** Calling it again is a no-op, so a scene transition reuses the same registry and the same Level identity.

**Exact declaration**

```cpp
bool ensure_bound(ObjectRegistry& registry)
```

- **Declared at:** [line 78](../../../runtime/actor_tables.hpp#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `registry` | `ObjectRegistry &` | Input/output; inspect the function contract | Value supplied for `registry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Calling it again is a no-op, so a scene transition reuses the same registry and the same Level identity.

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

**Purpose.** Loads one cooked bank.

**Details.** Returns the number of actors instantiated. Order (design.md section 6, with the deviation recorded in p10-cook-runtime.md): 1. Level::spawn_batch -> reserve, defaults, register, actor begin_play; 2. cooked configuration: components, legacy slot binding, attachment, overrides; 3. begin_play of the components added in step 2; 4. the bank's single SceneScriptActor, then its begin_play. The scene script therefore always observes fully configured actors.

**Exact declaration**

```cpp
size_t load_bank(const ActorTable& table, Entity* slots, size_t slot_count)
```

- **Declared at:** [line 112](../../../runtime/actor_tables.hpp#L112)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `const ActorTable &` | Input | Value supplied for `table`. See the exact type and module contract. |
| `slots` | `Entity *` | Input/output; inspect the function contract | Value supplied for `slots`. See the exact type and module contract. |
| `slot_count` | `size_t` | Input | Value supplied for `slot_count`. See the exact type and module contract. |

**Returns.** Returns `size_t`. Check the purpose and failure notes before using the value.

**Use it when.** Returns the number of actors instantiated. Order (design.md section 6, with the deviation recorded in p10-cook-runtime.md): 1. Level::spawn_batch -> reserve, defaults, register, actor begin_play; 2. cooked configuration: components, legacy slot binding, attachment, overrides; 3. begin_play of the components added in step 2; 4. the bank's single SceneScriptActor, then its begin_play. The scene script therefore always observes fully configured actors.

**Usage pattern**

```cpp
#include "actor_tables.hpp"

// Assume these named values have been initialized with valid data:
// const ActorTable & table
// Entity * slots
// size_t slot_count

epok::SceneLevel& object = /* obtain a valid instance */;

auto result = object.load_bank(table, slots, slot_count);
```

**Why choose it.** It provides direct, allocation-conscious access to the actor tables module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-scenelevel-refresh-stats-1"></a>

## `epok::SceneLevel::refresh_stats`

**Purpose.** Snapshot of the registry and level counters for tools/profile_runtime.py.

**Exact declaration**

```cpp
void refresh_stats()
```

- **Declared at:** [line 164](../../../runtime/actor_tables.hpp#L164)
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

<a id="epok-unload-actor-bank-1"></a>

## `epok::unload_actor_bank`

**Purpose.** Called by scene_tick() before the legacy binding teardown, while the actors and their legacy slots are still alive.

**Exact declaration**

```cpp
inline void unload_actor_bank()
```

- **Declared at:** [line 261](../../../runtime/actor_tables.hpp#L261)
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
