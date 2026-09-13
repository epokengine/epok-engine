# Epok API: Actor Blueprint

> **Header:** `"actor_blueprint.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/actor_blueprint.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 10 public callables declared directly in this header.

## Callable index

- [`epok::bp::actor_entity`](#epok-bp-actor-entity-1) — Legacy slot behind the actor's root scene component.
- [`epok::bp::actor_handle`](#epok-bp-actor-handle-1) — Blueprint node ABI handle for the actor's legacy slot.
- [`epok::bp::actor_transform`](#epok-bp-actor-transform-1) — Performs `actor transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_entity`](#epok-bp-component-entity-1) — Performs `component entity` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_handle`](#epok-bp-component-handle-1) — Performs `component handle` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_owner`](#epok-bp-component-owner-1) — Owning actor of a component, and its handle/slot, for Component-family Blueprints.
- [`epok::bp::component_owner_id`](#epok-bp-component-owner-id-1) — Performs `component owner id` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::component_transform`](#epok-bp-component-transform-1) — Performs `component transform` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::object_transform_fallback`](#epok-bp-object-transform-fallback-1) — Transform alias target.
- [`epok::bp::spawn_actor`](#epok-bp-spawn-actor-1) — Bounded actor spawn for the SpawnActor node.

<a id="epok-bp-actor-entity-1"></a>

## `epok::bp::actor_entity`

**Purpose.** Legacy slot behind the actor's root scene component.

**Details.** Null for Actor2D/UIActor, whose roots carry their own transform storage, and for an actor that has no root yet.

**Exact declaration**

```cpp
inline Entity* actor_entity(Actor* actor)
```

- **Declared at:** [line 19](../../../runtime/actor_blueprint.hpp#L19)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

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
inline EntityHandle actor_handle(Actor* actor)
```

- **Declared at:** [line 23](../../../runtime/actor_blueprint.hpp#L23)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `actor` | `Actor *` | Input/output; inspect the function contract | Value supplied for `actor`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 50](../../../runtime/actor_blueprint.hpp#L50)
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
inline Entity* component_entity(ActorComponent* component)
```

- **Declared at:** [line 36](../../../runtime/actor_blueprint.hpp#L36)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `Entity *`. Check the purpose and failure notes before using the value.

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
inline EntityHandle component_handle(ActorComponent* component)
```

- **Declared at:** [line 39](../../../runtime/actor_blueprint.hpp#L39)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `component` | `ActorComponent *` | Input/output; inspect the function contract | Value supplied for `component`. See the exact type and module contract. |

**Returns.** Returns `EntityHandle`. Check the purpose and failure notes before using the value.

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

- **Declared at:** [line 29](../../../runtime/actor_blueprint.hpp#L29)
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

- **Declared at:** [line 32](../../../runtime/actor_blueprint.hpp#L32)
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

- **Declared at:** [line 54](../../../runtime/actor_blueprint.hpp#L54)
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

<a id="epok-bp-object-transform-fallback-1"></a>

## `epok::bp::object_transform_fallback`

**Purpose.** Transform alias target.

**Details.** The fallback keeps a reference alias well formed when the actor has no legacy slot; generated code still emits an explicit skip before using the alias, so the fallback is never observed by a running graph.

**Exact declaration**

```cpp
inline Transform& object_transform_fallback()
```

- **Declared at:** [line 46](../../../runtime/actor_blueprint.hpp#L46)
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

<a id="epok-bp-spawn-actor-1"></a>

## `epok::bp::spawn_actor`

**Purpose.** Bounded actor spawn for the SpawnActor node.

**Details.** The class must be Actor family and concrete; the cook rejects anything else at compile time, and this rejects a stale or unknown compact id at run time.

**Exact declaration**

```cpp
inline ObjectId spawn_actor(Actor* context, uint64_t class_id, ObjectId logical_parent)
```

- **Declared at:** [line 61](../../../runtime/actor_blueprint.hpp#L61)
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
