# Epok API: Blueprint Spawn

> **Header:** `"blueprint_spawn.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/blueprint_spawn.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 6 public callables declared directly in this header.

## Declared types

`epok::bp::ClassId`

## Callable index

- [`epok::bp::class_is_a`](#epok-bp-class-is-a-1) — Performs `class is a` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::find_class`](#epok-bp-find-class-1) — Finds class as part of compiled Blueprint execution and object interaction.
- [`epok::bp::is_a`](#epok-bp-is-a-1) — Reports whether a as part of compiled Blueprint execution and object interaction.
- [`epok::bp::is_a`](#epok-bp-is-a-2) — Reports whether a as part of compiled Blueprint execution and object interaction.
- [`epok::bp::object`](#epok-bp-object-1) — Performs `object` as part of compiled Blueprint execution and object interaction.
- [`epok::bp::object`](#epok-bp-object-2) — Performs `object` as part of compiled Blueprint execution and object interaction.

<a id="epok-bp-class-is-a-1"></a>

## `epok::bp::class_is_a`

**Purpose.** Performs `class is a` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool class_is_a(ClassId child, ClassId parent)
```

- **Declared at:** [line 7](../../../runtime/blueprint_spawn.hpp#L7)
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

<a id="epok-bp-find-class-1"></a>

## `epok::bp::find_class`

**Purpose.** Finds class as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline const ClassDescriptor* find_class(ClassId id)
```

- **Declared at:** [line 6](../../../runtime/blueprint_spawn.hpp#L6)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ClassId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `const ClassDescriptor *`. Check the purpose and failure notes before using the value.

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

<a id="epok-bp-is-a-1"></a>

## `epok::bp::is_a`

**Purpose.** Reports whether a as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool is_a(DataHandle id,ClassId type)
```

- **Declared at:** [line 11](../../../runtime/blueprint_spawn.hpp#L11)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `DataHandle` | Input | Value supplied for `id`. See the exact type and module contract. |
| `type` | `ClassId` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle id
// ClassId type

auto result = epok::bp::is_a(id, type);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-is-a-2"></a>

## `epok::bp::is_a`

**Purpose.** Reports whether a as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline bool is_a(ObjectId id,ClassId type)
```

- **Declared at:** [line 10](../../../runtime/blueprint_spawn.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |
| `type` | `ClassId` | Input | Value supplied for `type`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id
// ClassId type

auto result = epok::bp::is_a(id, type);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-bp-object-1"></a>

## `epok::bp::object`

**Purpose.** Performs `object` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Object* object(DataHandle value)
```

- **Declared at:** [line 9](../../../runtime/blueprint_spawn.hpp#L9)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `DataHandle` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// DataHandle value

auto result = epok::bp::object(value);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-bp-object-2"></a>

## `epok::bp::object`

**Purpose.** Performs `object` as part of compiled Blueprint execution and object interaction.

**Exact declaration**

```cpp
inline Object* object(ObjectId id)
```

- **Declared at:** [line 8](../../../runtime/blueprint_spawn.hpp#L8)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `id` | `ObjectId` | Input | Value supplied for `id`. See the exact type and module contract. |

**Returns.** Returns `Object *`. Check the purpose and failure notes before using the value.

**Use it when.** You need compiled Blueprint execution and object interaction and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "blueprint_spawn.hpp"

// Assume these named values have been initialized with valid data:
// ObjectId id

auto result = epok::bp::object(id);
```

**Why choose it.** It provides direct, allocation-conscious access to compiled Blueprint execution and object interaction. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
