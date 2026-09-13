# Epok API: Blueprint Template

> **Header:** `"blueprint_template.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/blueprint_template.hpp)

This module covers compiled Blueprint execution and object interaction. It documents 1 public callable declared directly in this header.

## Declared types

`epok::bp::TemplateBinding`

## Callable index

- [`epok::bp::instantiate_template`](#epok-bp-instantiate-template-1) — Prototypes are the host-constructed, normally cooked Object records.

<a id="epok-bp-instantiate-template-1"></a>

## `epok::bp::instantiate_template`

**Purpose.** Prototypes are the host-constructed, normally cooked Object records.

**Details.** Their resource pointers reference immutable generated data, never editor addresses. Template parents are local indices. The root keeps the caller's name/parent; all other component values (including local transform/active) use the template.

**Exact declaration**

```cpp
inline bool instantiate_template(EntityHandle root,const Entity* prototypes,size_t count,size_t root_index, const TemplateBinding* bindings,size_t binding_count, bool(*configure_components)(const EntityHandle*,size_t)=nullptr)
```

- **Declared at:** [line 13](../../../runtime/blueprint_template.hpp#L13)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `root` | `EntityHandle` | Input | Value supplied for `root`. See the exact type and module contract. |
| `prototypes` | `const Entity *` | Input | Value supplied for `prototypes`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |
| `root_index` | `size_t` | Input | Value supplied for `root_index`. See the exact type and module contract. |
| `bindings` | `const TemplateBinding *` | Input | Value supplied for `bindings`. See the exact type and module contract. |
| `binding_count` | `size_t` | Input | Value supplied for `binding_count`. See the exact type and module contract. |
| `configure_components` | `bool (*)(const EntityHandle *, size_t)` | Callback | Value supplied for `configure_components`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Their resource pointers reference immutable generated data, never editor addresses. Template parents are local indices. The root keeps the caller's name/parent; all other component values (including local transform/active) use the template.

**Usage pattern**

```cpp
#include "blueprint_template.hpp"

// Assume these named values have been initialized with valid data:
// EntityHandle root
// const Entity * prototypes
// size_t count
// size_t root_index
// const TemplateBinding * bindings
// size_t binding_count
// bool (*)(const EntityHandle *, size_t) configure_components

auto result = epok::bp::instantiate_template(root, prototypes, count, root_index, bindings, binding_count, configure_components);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
