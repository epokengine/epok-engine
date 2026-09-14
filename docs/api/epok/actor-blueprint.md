# Epok API: Actor Blueprint

> **Header:** `"actor_blueprint.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/actor_blueprint.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 14 public callables declared directly in this header.

## Callable index

- [`epok::bp::actor_entity`](#epok-bp-actor-entity-1) — Legacy slot behind the actor's root scene component.
- [`epok::bp::actor_handle`](#epok-bp-actor-handle-1) — Blueprint node ABI handle for the actor's legacy slot.
- [`epok::bp::actor_transform`](#epok-bp-actor-transform-1) — Performs `actor transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_entity`](#epok-bp-component-entity-1) — Performs `component entity` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_handle`](#epok-bp-component-handle-1) — Performs `component handle` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_owner`](#epok-bp-component-owner-1) — Owning actor of a component, and its handle/slot, for Component-family Blueprints.
- [`epok::bp::component_owner_id`](#epok-bp-component-owner-id-1) — Performs `component owner id` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_transform`](#epok-bp-component-transform-1) — Performs `component transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::data_handle`](#epok-bp-data-handle-1) — Performs `data handle` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::object_data`](#epok-bp-object-data-1) — Performs `object data` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::object_transform_fallback`](#epok-bp-object-transform-fallback-1) — Transform alias target.
- [`epok::bp::owner_actor`](#epok-bp-owner-actor-1) — Performs `owner actor` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::root_component`](#epok-bp-root-component-1) — Performs `root component` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::spawn_actor`](#epok-bp-spawn-actor-1) — Bounded actor spawn for the SpawnActor node.

<a id="epok-bp-actor-entity-1"></a>

## `epok::bp::actor_entity`

**Purpose.** Legacy slot behind the actor's root scene component.

**Details.** Null for Actor2D/UIActor, whose roots carry their own transform storage, and for an actor that has no root yet.

**Exact declaration**

```cpp
inline ActorData* actor_entity(Actor* actor)
```

- **Declared at:** [line 31](../../../runtime/actor_blueprint.hpp#L31)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** Null for Actor2D/UIActor, whose roots carry their own transform storage, and for an actor that has no root yet.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// Actor * actor

auto result = epok::bp::actor_entity(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-actor-handle-1"></a>

## `epok::bp::actor_handle`

**Purpose.** Blueprint node ABI handle for the actor's legacy slot.

**Details.** A null slot yields a null handle, which every epok::bp::api entry point already treats as "do nothing".

**Exact declaration**

```cpp
inline DataHandle actor_handle(Actor* actor)
```

- **Declared at:** [line 35](../../../runtime/actor_blueprint.hpp#L35)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** A null slot yields a null handle, which every epok::bp::api entry point already treats as "do nothing".

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// Actor * actor

auto result = epok::bp::actor_handle(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-actor-transform-1"></a>

## `epok::bp::actor_transform`

**Purpose.** Performs `actor transform` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Transform& actor_transform(Actor* actor)
```

- **Declared at:** [line 62](../../../runtime/actor_blueprint.hpp#L62)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `Transform &`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// Actor * actor

auto result = epok::bp::actor_transform(actor);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-component-entity-1"></a>

## `epok::bp::component_entity`

**Purpose.** Performs `component entity` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline ActorData* component_entity(ActorComponent* component)
```

- **Declared at:** [line 48](../../../runtime/actor_blueprint.hpp#L48)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent * component

auto result = epok::bp::component_entity(component);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-component-handle-1"></a>

## `epok::bp::component_handle`

**Purpose.** Performs `component handle` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline DataHandle component_handle(ActorComponent* component)
```

- **Declared at:** [line 51](../../../runtime/actor_blueprint.hpp#L51)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent * component

auto result = epok::bp::component_handle(component);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-component-owner-1"></a>

## `epok::bp::component_owner`

**Purpose.** Owning actor of a component, and its handle/slot, for Component-family Blueprints.

**Exact declaration**

```cpp
inline Actor* component_owner(ActorComponent* component)
```

- **Declared at:** [line 41](../../../runtime/actor_blueprint.hpp#L41)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `Actor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent * component

auto result = epok::bp::component_owner(component);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-component-owner-id-1"></a>

## `epok::bp::component_owner_id`

**Purpose.** Performs `component owner id` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline ObjectId component_owner_id(ActorComponent* component)
```

- **Declared at:** [line 44](../../../runtime/actor_blueprint.hpp#L44)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent * component

auto result = epok::bp::component_owner_id(component);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-component-transform-1"></a>

## `epok::bp::component_transform`

**Purpose.** Performs `component transform` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Transform& component_transform(ActorComponent* component)
```

- **Declared at:** [line 66](../../../runtime/actor_blueprint.hpp#L66)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `Transform &`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ActorComponent * component

auto result = epok::bp::component_transform(component);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bp-data-handle-1"></a>

## `epok::bp::data_handle`

**Purpose.** Performs `data handle` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline DataHandle data_handle(ObjectId id)
```

- **Declared at:** [line 24](../../../runtime/actor_blueprint.hpp#L24)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `DataHandle`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::bp::data_handle(id);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-object-data-1"></a>

## `epok::bp::object_data`

**Purpose.** Performs `object data` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline ActorData* object_data(ObjectId id)
```

- **Declared at:** [line 23](../../../runtime/actor_blueprint.hpp#L23)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `ActorData *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::bp::object_data(id);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-object-transform-fallback-1"></a>

## `epok::bp::object_transform_fallback`

**Purpose.** Transform alias target.

**Details.** The fallback keeps a reference alias well formed when the actor has no legacy slot; generated code still emits an explicit skip before using the alias, so the fallback is never observed by a running graph.

**Exact declaration**

```cpp
inline Transform& object_transform_fallback()
```

- **Declared at:** [line 58](../../../runtime/actor_blueprint.hpp#L58)
- **Kind:** `function decl`

**Returns.** Returns `Transform &`. Check the purpose and failure notes before using the value.

**Use it when.** The fallback keeps a reference alias well formed when the actor has no legacy slot; generated code still emits an explicit skip before using the alias, so the fallback is never observed by a running graph.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

auto result = epok::bp::object_transform_fallback();
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-owner-actor-1"></a>

## `epok::bp::owner_actor`

**Purpose.** Performs `owner actor` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Actor* owner_actor(ObjectId id)
```

- **Declared at:** [line 17](../../../runtime/actor_blueprint.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `Actor *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::bp::owner_actor(id);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-root-component-1"></a>

## `epok::bp::root_component`

**Purpose.** Performs `root component` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
template<class Root> Root* root_component(ObjectId id)
```

- **Declared at:** [line 25](../../../runtime/actor_blueprint.hpp#L25)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `Root *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Root

// Assume these named values have been initialized with valid data:
// ObjectId id

epok::bp& object = /* obtain a valid instance */;

auto result = object.root_component<Root>(id);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-bp-spawn-actor-1"></a>

## `epok::bp::spawn_actor`

**Purpose.** Bounded actor spawn for the SpawnActor node.

**Details.** The class must be Actor family and concrete; the cook rejects anything else at compile time, and this rejects a stale or unknown compact id at run time.

**Exact declaration**

```cpp
inline ObjectId spawn_actor(Actor* context, uint64_t class_id, ObjectId logical_parent)
```

- **Declared at:** [line 73](../../../runtime/actor_blueprint.hpp#L73)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `context` | `Actor *` | Input/output; inspect the function contract | Value supplied for `context`. See the exact type and module contract. |
| `class_id` | `uint64_t` | Input | Value supplied for `class_id`. See the exact type and module contract. |
| `logical_parent` | `ObjectId` | Input | Value supplied for `logical_parent`. See the exact type and module contract. |

**Returns.** Returns `ObjectId`. Check the purpose and failure notes before using the value.

**Use it when.** The class must be Actor family and concrete; the cook rejects anything else at compile time, and this rejects a stale or unknown compact id at run time.

**Usage pattern**

```cpp
#include "actor_blueprint.hpp"

// Assume these named values have been initialized with valid data:
// Actor * context
// uint64_t class_id
// ObjectId logical_parent

auto result = epok::bp::spawn_actor(context, class_id, logical_parent);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
