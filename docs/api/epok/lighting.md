# Epok API: Lighting

> **Header:** `"lighting.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/lighting.hpp)

This module covers the lighting module. It documents 16 public callables declared directly in this header.

## Declared types

`epok::lighting_detail::Local`, `epok::lighting_detail::MeshNormalTransform`, `epok::lighting_detail::Source`, `epok::lighting_detail::Vector`, `epok::LightingRenderer`

## Callable index

- [`epok::lighting_detail::clamp`](#epok-lighting-detail-clamp-1) — Performs `clamp` as part of the lighting module.
- [`epok::lighting_detail::direction`](#epok-lighting-detail-direction-1) — Performs `direction` as part of the lighting module.
- [`epok::lighting_detail::face_normal`](#epok-lighting-detail-face-normal-1) — Performs `face normal` as part of the lighting module.
- [`epok::lighting_detail::mesh_normal`](#epok-lighting-detail-mesh-normal-1) — Performs `mesh normal` as part of the lighting module.
- [`epok::lighting_detail::mesh_shade`](#epok-lighting-detail-mesh-shade-1) — Performs `mesh shade` as part of the lighting module.
- [`epok::lighting_detail::mesh_shade_local`](#epok-lighting-detail-mesh-shade-local-1) — Lit face whose normal is already in the space of the GTE light matrix (see LightingRenderer::localize).
- [`epok::lighting_detail::mesh_shade_normal`](#epok-lighting-detail-mesh-shade-normal-1) — Shades one face with the light registers prepared by LightingRenderer::shade.
- [`epok::lighting_detail::MeshNormalTransform::apply`](#epok-lighting-detail-meshnormaltransform-apply-1) — Performs `apply` as part of the lighting module.
- [`epok::lighting_detail::normalize`](#epok-lighting-detail-normalize-1) — Performs `normalize` as part of the lighting module.
- [`epok::lighting_detail::pair`](#epok-lighting-detail-pair-1) — Performs `pair` as part of the lighting module.
- [`epok::lighting_detail::sqrt64`](#epok-lighting-detail-sqrt64-1) — Performs `sqrt64` as part of the lighting module.
- [`epok::LightingRenderer::clear`](#epok-lightingrenderer-clear-1) — Clears clear as part of the lighting module.
- [`epok::LightingRenderer::localize`](#epok-lightingrenderer-localize-1) — Rotates the light matrix loaded by shade() into an object's local space when its world basis is a rotation with uniform scale (within 1/256).
- [`epok::LightingRenderer::prepare`](#epok-lightingrenderer-prepare-1) — Performs `prepare` as part of the lighting module.
- [`epok::LightingRenderer::reset_owner`](#epok-lightingrenderer-reset-owner-1) — Resets owner as part of the lighting module.
- [`epok::LightingRenderer::shade`](#epok-lightingrenderer-shade-1) — Performs `shade` as part of the lighting module.

<a id="epok-lighting-detail-clamp-1"></a>

## `epok::lighting_detail::clamp`

**Purpose.** Performs `clamp` as part of the lighting module.

**Exact declaration**

```cpp
inline int32_t clamp(int32_t v,int32_t low,int32_t high)
```

- **Declared at:** [line 13](../../../runtime/lighting.hpp#L13)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `int32_t` | Input | Value supplied for `v`. See the exact type and module contract. |
| `low` | `int32_t` | Input | Value supplied for `low`. See the exact type and module contract. |
| `high` | `int32_t` | Input | Value supplied for `high`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// int32_t v
// int32_t low
// int32_t high

auto result = epok::lighting_detail::clamp(v, low, high);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-lighting-detail-direction-1"></a>

## `epok::lighting_detail::direction`

**Purpose.** Performs `direction` as part of the lighting module.

**Exact declaration**

```cpp
inline Vector direction(const Affine<Fixed>& m)
```

- **Declared at:** [line 17](../../../runtime/lighting.hpp#L17)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Fixed> &` | Input | Value supplied for `m`. See the exact type and module contract. |

**Returns.** Returns `Vector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & m

auto result = epok::lighting_detail::direction(m);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-face-normal-1"></a>

## `epok::lighting_detail::face_normal`

**Purpose.** Performs `face normal` as part of the lighting module.

**Exact declaration**

```cpp
inline Vector face_normal(const Affine<Fixed>& m,int face)
```

- **Declared at:** [line 18](../../../runtime/lighting.hpp#L18)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Fixed> &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `face` | `int` | Input | Value supplied for `face`. See the exact type and module contract. |

**Returns.** Returns `Vector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & m
// int face

auto result = epok::lighting_detail::face_normal(m, face);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-mesh-normal-1"></a>

## `epok::lighting_detail::mesh_normal`

**Purpose.** Performs `mesh normal` as part of the lighting module.

**Exact declaration**

```cpp
inline Vector mesh_normal(const Affine<Fixed>& m,const int16_t* normal)
```

- **Declared at:** [line 61](../../../runtime/lighting.hpp#L61)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Fixed> &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `normal` | `const int16_t *` | Input | Value supplied for `normal`. See the exact type and module contract. |

**Returns.** Returns `Vector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & m
// const int16_t * normal

auto result = epok::lighting_detail::mesh_normal(m, normal);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-mesh-shade-1"></a>

## `epok::lighting_detail::mesh_shade`

**Purpose.** Performs `mesh shade` as part of the lighting module.

**Exact declaration**

```cpp
inline psyqo::Color mesh_shade(const Affine<Fixed> &world, const MeshQuad &face, const Material &tint, bool enabled,MeshNormalTransform* transform=nullptr)
```

- **Declared at:** [line 84](../../../runtime/lighting.hpp#L84)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `face` | `const MeshQuad &` | Input | Value supplied for `face`. See the exact type and module contract. |
| `tint` | `const Material &` | Input | Value supplied for `tint`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |
| `transform` | `MeshNormalTransform *` | Input/output; inspect the function contract | Value supplied for `transform`. See the exact type and module contract. |

**Returns.** Returns `psyqo::Color`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & world
// const MeshQuad & face
// const Material & tint
// bool enabled
// MeshNormalTransform * transform

auto result = epok::lighting_detail::mesh_shade(world, face, tint, enabled, transform);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-mesh-shade-local-1"></a>

## `epok::lighting_detail::mesh_shade_local`

**Purpose.** Lit face whose normal is already in the space of the GTE light matrix (see LightingRenderer::localize).

**Details.** Callers handle the unlit cases.

**Exact declaration**

```cpp
inline psyqo::Color mesh_shade_local(const int16_t* normal, const Material& face_material, const Material& tint)
```

- **Declared at:** [line 90](../../../runtime/lighting.hpp#L90)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `normal` | `const int16_t *` | Input | Value supplied for `normal`. See the exact type and module contract. |
| `face_material` | `const Material &` | Input | Value supplied for `face_material`. See the exact type and module contract. |
| `tint` | `const Material &` | Input | Value supplied for `tint`. See the exact type and module contract. |

**Returns.** Returns `psyqo::Color`. Check the purpose and failure notes before using the value.

**Use it when.** Callers handle the unlit cases.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const int16_t * normal
// const Material & face_material
// const Material & tint

auto result = epok::lighting_detail::mesh_shade_local(normal, face_material, tint);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-mesh-shade-normal-1"></a>

## `epok::lighting_detail::mesh_shade_normal`

**Purpose.** Shades one face with the light registers prepared by LightingRenderer::shade.

**Details.** The normal is separate from the face so callers can substitute a runtime one.

**Exact declaration**

```cpp
inline psyqo::Color mesh_shade_normal(const Affine<Fixed> &world, const int16_t* normal, const Material& face_material, const Material &tint, bool enabled,MeshNormalTransform* transform=nullptr)
```

- **Declared at:** [line 66](../../../runtime/lighting.hpp#L66)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `normal` | `const int16_t *` | Input | Value supplied for `normal`. See the exact type and module contract. |
| `face_material` | `const Material &` | Input | Value supplied for `face_material`. See the exact type and module contract. |
| `tint` | `const Material &` | Input | Value supplied for `tint`. See the exact type and module contract. |
| `enabled` | `bool` | Input | Value supplied for `enabled`. See the exact type and module contract. |
| `transform` | `MeshNormalTransform *` | Input/output; inspect the function contract | Value supplied for `transform`. See the exact type and module contract. |

**Returns.** Returns `psyqo::Color`. Check the purpose and failure notes before using the value.

**Use it when.** The normal is separate from the face so callers can substitute a runtime one.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & world
// const int16_t * normal
// const Material & face_material
// const Material & tint
// bool enabled
// MeshNormalTransform * transform

auto result = epok::lighting_detail::mesh_shade_normal(world, normal, face_material, tint, enabled, transform);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-meshnormaltransform-apply-1"></a>

## `epok::lighting_detail::MeshNormalTransform::apply`

**Purpose.** Performs `apply` as part of the lighting module.

**Exact declaration**

```cpp
Vector apply(const Affine<Fixed>& m,const int16_t* normal)
```

- **Declared at:** [line 29](../../../runtime/lighting.hpp#L29)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Affine<Fixed> &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `normal` | `const int16_t *` | Input | Value supplied for `normal`. See the exact type and module contract. |

**Returns.** Returns `Vector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & m
// const int16_t * normal

epok::lighting_detail::MeshNormalTransform& object = /* obtain a valid instance */;

auto result = object.apply(m, normal);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lighting-detail-normalize-1"></a>

## `epok::lighting_detail::normalize`

**Purpose.** Performs `normalize` as part of the lighting module.

**Exact declaration**

```cpp
inline Vector normalize(Vector in)
```

- **Declared at:** [line 16](../../../runtime/lighting.hpp#L16)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `Vector` | Input | Value supplied for `in`. See the exact type and module contract. |

**Returns.** Returns `Vector`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// Vector in

auto result = epok::lighting_detail::normalize(in);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-lighting-detail-pair-1"></a>

## `epok::lighting_detail::pair`

**Purpose.** Performs `pair` as part of the lighting module.

**Exact declaration**

```cpp
inline uint32_t pair(int32_t a,int32_t b)
```

- **Declared at:** [line 100](../../../runtime/lighting.hpp#L100)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `int32_t` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `int32_t` | Input | Value supplied for `b`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// int32_t a
// int32_t b

auto result = epok::lighting_detail::pair(a, b);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-lighting-detail-sqrt64-1"></a>

## `epok::lighting_detail::sqrt64`

**Purpose.** Performs `sqrt64` as part of the lighting module.

**Exact declaration**

```cpp
inline uint32_t sqrt64(uint64_t v)
```

- **Declared at:** [line 14](../../../runtime/lighting.hpp#L14)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `uint64_t` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// uint64_t v

auto result = epok::lighting_detail::sqrt64(v);
```

**Why choose it.** It provides direct, allocation-conscious access to the lighting module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-lightingrenderer-clear-1"></a>

## `epok::LightingRenderer::clear`

**Purpose.** Clears clear as part of the lighting module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 124](../../../runtime/lighting.hpp#L124)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

epok::LightingRenderer& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-lightingrenderer-localize-1"></a>

## `epok::LightingRenderer::localize`

**Purpose.** Rotates the light matrix loaded by shade() into an object's local space when its world basis is a rotation with uniform scale (within 1/256).

**Details.** Face normals then feed NCS directly. Shear or non-uniform scale keeps the registers untouched and returns false for the per-face cofactor path.

**Exact declaration**

```cpp
bool localize(const Affine<Fixed>& world)
```

- **Declared at:** [line 180](../../../runtime/lighting.hpp#L180)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world` | `const Affine<Fixed> &` | Input | Value supplied for `world`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Face normals then feed NCS directly. Shear or non-uniform scale keeps the registers untouched and returns false for the per-face cofactor path.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const Affine<Fixed> & world

epok::LightingRenderer& object = /* obtain a valid instance */;

auto result = object.localize(world);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lightingrenderer-prepare-1"></a>

## `epok::LightingRenderer::prepare`

**Purpose.** Performs `prepare` as part of the lighting module.

**Exact declaration**

```cpp
void prepare(const std::array<Entity,N>& objects,const std::array<Affine<Fixed>,N>& world,size_t object_count)
```

- **Declared at:** [line 128](../../../runtime/lighting.hpp#L128)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const std::array<Entity, N> &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const std::array<Affine<Fixed>, N> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `object_count` | `size_t` | Input | Value supplied for `object_count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// const std::array<Entity, N> & objects
// const std::array<Affine<Fixed>, N> & world
// size_t object_count

epok::LightingRenderer& object = /* obtain a valid instance */;

object.prepare(objects, world, object_count);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-lightingrenderer-reset-owner-1"></a>

## `epok::LightingRenderer::reset_owner`

**Purpose.** Resets owner as part of the lighting module.

**Exact declaration**

```cpp
void reset_owner(size_t index)
```

- **Declared at:** [line 120](../../../runtime/lighting.hpp#L120)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::LightingRenderer& object = /* obtain a valid instance */;

object.reset_owner(index);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="epok-lightingrenderer-shade-1"></a>

## `epok::LightingRenderer::shade`

**Purpose.** Performs `shade` as part of the lighting module.

**Exact declaration**

```cpp
std::array<psyqo::Color,6> shade(size_t object,const std::array<Entity,N>& objects,const std::array<Affine<Fixed>,N>& world,bool generic=false)
```

- **Declared at:** [line 134](../../../runtime/lighting.hpp#L134)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `object` | `size_t` | Input | Value supplied for `object`. See the exact type and module contract. |
| `objects` | `const std::array<Entity, N> &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const std::array<Affine<Fixed>, N> &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `generic` | `bool` | Input | Value supplied for `generic`. See the exact type and module contract. |

**Returns.** Returns `std::array<psyqo::Color, 6>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the lighting module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "lighting.hpp"

// Assume these named values have been initialized with valid data:
// size_t object
// const std::array<Entity, N> & objects
// const std::array<Affine<Fixed>, N> & world
// bool generic

epok::LightingRenderer& object = /* obtain a valid instance */;

auto result = object.shade(object, objects, world, generic);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
