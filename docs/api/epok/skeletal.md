# Epok API: Skeletal

> **Header:** `"skeletal.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/skeletal.hpp)

This module covers rigid skeletal animation and pose evaluation. It documents 19 public callables declared directly in this header.

## Declared types

`epok::skeletal_detail::Scratch`, `epok::skeletal_query_detail::BatchScratch`

## Callable index

- [`epok::skeletal_detail::aim_pitch_matrix`](#epok-skeletal-detail-aim-pitch-matrix-1) — A small-angle quaternion approximation keeps additive aiming deterministic on host and MIPS without introducing a per-character trigonometry service.
- [`epok::skeletal_detail::local_pose_matrix`](#epok-skeletal-detail-local-pose-matrix-1) — Performs `local pose matrix` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::pose_matrix`](#epok-skeletal-detail-pose-matrix-1) — Performs `pose matrix` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::decode_vertices`](#epok-skeletal-detail-scratch-decode-vertices-1) — Performs `decode vertices` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::frame_index`](#epok-skeletal-detail-scratch-frame-index-1) — Performs `frame index` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::pose`](#epok-skeletal-detail-scratch-pose-1) — Performs `pose` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::pose_bones`](#epok-skeletal-detail-scratch-pose-bones-1) — Performs `pose bones` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::skin_vertices`](#epok-skeletal-detail-scratch-skin-vertices-1) — Performs `skin vertices` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::apply_space`](#epok-skeletal-query-detail-apply-space-1) — Performs `apply space` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::baked_vertex`](#epok-skeletal-query-detail-baked-vertex-1) — Performs `baked vertex` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::batch_scratch`](#epok-skeletal-query-detail-batch-scratch-1) — Performs `batch scratch` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::bone_matrix`](#epok-skeletal-query-detail-bone-matrix-1) — Performs `bone matrix` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::model_vertex`](#epok-skeletal-query-detail-model-vertex-1) — Performs `model vertex` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::pose_all`](#epok-skeletal-query-detail-pose-all-1) — Performs `pose all` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::selected_pose`](#epok-skeletal-query-detail-selected-pose-1) — Performs `selected pose` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::stats`](#epok-skeletal-query-detail-stats-1) — Performs `stats` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::valid_pose`](#epok-skeletal-query-detail-valid-pose-1) — Performs `valid pose` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_query_detail::valid_space`](#epok-skeletal-query-detail-valid-space-1) — Performs `valid space` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_sample_vertex_impl`](#epok-skeletal-sample-vertex-impl-1) — Performs `skeletal sample vertex impl` as part of rigid skeletal animation and pose evaluation.

<a id="epok-skeletal-detail-aim-pitch-matrix-1"></a>

## `epok::skeletal_detail::aim_pitch_matrix`

**Purpose.** A small-angle quaternion approximation keeps additive aiming deterministic on host and MIPS without introducing a per-character trigonometry service.

**Details.** The controller clamps this to a practical upper-body range ([-35, 45] degrees).

**Exact declaration**

```cpp
inline Affine<Fixed> aim_pitch_matrix(Fixed degrees)
```

- **Declared at:** [line 45](../../../runtime/skeletal.hpp#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `degrees` | `Fixed` | Input | Value supplied for `degrees`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** The controller clamps this to a practical upper-body range ([-35, 45] degrees).

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// Fixed degrees

auto result = epok::skeletal_detail::aim_pitch_matrix(degrees);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletal-detail-local-pose-matrix-1"></a>

## `epok::skeletal_detail::local_pose_matrix`

**Purpose.** Performs `local pose matrix` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline Affine<Fixed> local_pose_matrix(const BonePose& pose,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch,size_t bone)
```

- **Declared at:** [line 58](../../../runtime/skeletal.hpp#L58)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pose` | `const BonePose &` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |
| `bone` | `size_t` | Input | Value supplied for `bone`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const BonePose & pose
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch
// size_t bone

auto result = epok::skeletal_detail::local_pose_matrix(pose, aim_bone, aim_stop_bones, aim_pitch, bone);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-pose-matrix-1"></a>

## `epok::skeletal_detail::pose_matrix`

**Purpose.** Performs `pose matrix` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline Affine<Fixed> pose_matrix(const BonePose& p)
```

- **Declared at:** [line 28](../../../runtime/skeletal.hpp#L28)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const BonePose &` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `Affine<Fixed>`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const BonePose & p

auto result = epok::skeletal_detail::pose_matrix(p);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-scratch-decode-vertices-1"></a>

## `epok::skeletal_detail::Scratch::decode_vertices`

**Purpose.** Performs `decode vertices` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
__attribute__((noinline,optimize("O3"))) #endif void decode_vertices(const SkeletalMesh& model,const Animator& animator)
```

- **Declared at:** [line 101](../../../runtime/skeletal.hpp#L101)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator

epok::skeletal_detail::Scratch& object = /* obtain a valid instance */;

object.decode_vertices(model, animator);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-scratch-frame-index-1"></a>

## `epok::skeletal_detail::Scratch::frame_index`

**Purpose.** Performs `frame index` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
static uint32_t frame_index(const AnimationClip* clip,const Animator& animator)
```

- **Declared at:** [line 70](../../../runtime/skeletal.hpp#L70)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `clip` | `const AnimationClip *` | Input | Value supplied for `clip`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const AnimationClip * clip
// const Animator & animator

auto result = epok::skeletal_detail::Scratch::frame_index(clip, animator);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-scratch-pose-1"></a>

## `epok::skeletal_detail::Scratch::pose`

**Purpose.** Performs `pose` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
void pose(const SkeletalMesh& model,const Animator& animator,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch)
```

- **Declared at:** [line 117](../../../runtime/skeletal.hpp#L117)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch

epok::skeletal_detail::Scratch& object = /* obtain a valid instance */;

object.pose(model, animator, aim_bone, aim_stop_bones, aim_pitch);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-scratch-pose-bones-1"></a>

## `epok::skeletal_detail::Scratch::pose_bones`

**Purpose.** Performs `pose bones` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
__attribute__((noinline,optimize("O3"))) #endif void pose_bones(const SkeletalMesh& model,const Animator& animator,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch)
```

- **Declared at:** [line 81](../../../runtime/skeletal.hpp#L81)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch

epok::skeletal_detail::Scratch& object = /* obtain a valid instance */;

object.pose_bones(model, animator, aim_bone, aim_stop_bones, aim_pitch);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-detail-scratch-skin-vertices-1"></a>

## `epok::skeletal_detail::Scratch::skin_vertices`

**Purpose.** Performs `skin vertices` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
void skin_vertices(const SkeletalMesh& model)
```

- **Declared at:** [line 90](../../../runtime/skeletal.hpp#L90)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model

epok::skeletal_detail::Scratch& object = /* obtain a valid instance */;

object.skin_vertices(model);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-apply-space-1"></a>

## `epok::skeletal_query_detail::apply_space`

**Purpose.** Performs `apply space` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool apply_space(const ActorData& entity,CoordinateSpace space,Fixed* position,SkeletalError& error)
```

- **Declared at:** [line 184](../../../runtime/skeletal.hpp#L184)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData &` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |
| `position` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `position`. See the exact type and module contract. |
| `error` | `SkeletalError &` | Input/output; inspect the function contract | Value supplied for `error`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData & entity
// CoordinateSpace space
// Fixed * position
// SkeletalError & error

auto result = epok::skeletal_query_detail::apply_space(entity, space, position, error);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-baked-vertex-1"></a>

## `epok::skeletal_query_detail::baked_vertex`

**Purpose.** Performs `baked vertex` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool baked_vertex(const SkeletalMesh& model,const Animator& animator,uint32_t vertex,PoseKind pose,Fixed* output)
```

- **Declared at:** [line 160](../../../runtime/skeletal.hpp#L160)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `output` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// uint32_t vertex
// PoseKind pose
// Fixed * output

auto result = epok::skeletal_query_detail::baked_vertex(model, animator, vertex, pose, output);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-batch-scratch-1"></a>

## `epok::skeletal_query_detail::batch_scratch`

**Purpose.** Performs `batch scratch` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline BatchScratch& batch_scratch()
```

- **Declared at:** [line 155](../../../runtime/skeletal.hpp#L155)
- **Kind:** `function decl`

**Returns.** Returns `BatchScratch &`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

auto result = epok::skeletal_query_detail::batch_scratch();
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletal-query-detail-bone-matrix-1"></a>

## `epok::skeletal_query_detail::bone_matrix`

**Purpose.** Performs `bone matrix` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool bone_matrix(const SkeletalMesh& model,const Animator& animator,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch,uint32_t bone,PoseKind pose,Affine<Fixed>& output)
```

- **Declared at:** [line 142](../../../runtime/skeletal.hpp#L142)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |
| `bone` | `uint32_t` | Input | Value supplied for `bone`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `output` | `Affine<Fixed> &` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch
// uint32_t bone
// PoseKind pose
// Affine<Fixed> & output

auto result = epok::skeletal_query_detail::bone_matrix(model, animator, aim_bone, aim_stop_bones, aim_pitch, bone, pose, output);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-model-vertex-1"></a>

## `epok::skeletal_query_detail::model_vertex`

**Purpose.** Performs `model vertex` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool model_vertex(const SkeletalMesh& model,const Animator& animator,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch,uint32_t portable,PoseKind pose,Fixed* output,SkeletalError& error)
```

- **Declared at:** [line 175](../../../runtime/skeletal.hpp#L175)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |
| `portable` | `uint32_t` | Input | Value supplied for `portable`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `output` | `Fixed *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `error` | `SkeletalError &` | Input/output; inspect the function contract | Value supplied for `error`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch
// uint32_t portable
// PoseKind pose
// Fixed * output
// SkeletalError & error

auto result = epok::skeletal_query_detail::model_vertex(model, animator, aim_bone, aim_stop_bones, aim_pitch, portable, pose, output, error);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-pose-all-1"></a>

## `epok::skeletal_query_detail::pose_all`

**Purpose.** Performs `pose all` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool pose_all(const SkeletalMesh& model,const Animator& animator,int16_t aim_bone,const int16_t* aim_stop_bones,Fixed aim_pitch,PoseKind pose)
```

- **Declared at:** [line 156](../../../runtime/skeletal.hpp#L156)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `aim_bone` | `int16_t` | Input | Value supplied for `aim_bone`. See the exact type and module contract. |
| `aim_stop_bones` | `const int16_t *` | Input | Value supplied for `aim_stop_bones`. See the exact type and module contract. |
| `aim_pitch` | `Fixed` | Input | Value supplied for `aim_pitch`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// int16_t aim_bone
// const int16_t * aim_stop_bones
// Fixed aim_pitch
// PoseKind pose

auto result = epok::skeletal_query_detail::pose_all(model, animator, aim_bone, aim_stop_bones, aim_pitch, pose);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-selected-pose-1"></a>

## `epok::skeletal_query_detail::selected_pose`

**Purpose.** Performs `selected pose` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline const BonePose* selected_pose(const SkeletalMesh& model,const Animator& animator,size_t bone,PoseKind kind)
```

- **Declared at:** [line 132](../../../runtime/skeletal.hpp#L132)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `model` | `const SkeletalMesh &` | Input | Value supplied for `model`. See the exact type and module contract. |
| `animator` | `const Animator &` | Input | Value supplied for `animator`. See the exact type and module contract. |
| `bone` | `size_t` | Input | Value supplied for `bone`. See the exact type and module contract. |
| `kind` | `PoseKind` | Input | Value supplied for `kind`. See the exact type and module contract. |

**Returns.** Returns `const BonePose *`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const SkeletalMesh & model
// const Animator & animator
// size_t bone
// PoseKind kind

auto result = epok::skeletal_query_detail::selected_pose(model, animator, bone, kind);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-skeletal-query-detail-stats-1"></a>

## `epok::skeletal_query_detail::stats`

**Purpose.** Performs `stats` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline SkeletalQueryStats& stats()
```

- **Declared at:** [line 129](../../../runtime/skeletal.hpp#L129)
- **Kind:** `function decl`

**Returns.** Returns `SkeletalQueryStats &`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

auto result = epok::skeletal_query_detail::stats();
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-skeletal-query-detail-valid-pose-1"></a>

## `epok::skeletal_query_detail::valid_pose`

**Purpose.** Performs `valid pose` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool valid_pose(PoseKind pose)
```

- **Declared at:** [line 130](../../../runtime/skeletal.hpp#L130)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// PoseKind pose

auto result = epok::skeletal_query_detail::valid_pose(pose);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-skeletal-query-detail-valid-space-1"></a>

## `epok::skeletal_query_detail::valid_space`

**Purpose.** Performs `valid space` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline bool valid_space(CoordinateSpace space)
```

- **Declared at:** [line 131](../../../runtime/skeletal.hpp#L131)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// CoordinateSpace space

auto result = epok::skeletal_query_detail::valid_space(space);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-skeletal-sample-vertex-impl-1"></a>

## `epok::skeletal_sample_vertex_impl`

**Purpose.** Performs `skeletal sample vertex impl` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline VertexSample skeletal_sample_vertex_impl(const ActorData* entity,uint32_t vertex,PoseKind pose,CoordinateSpace space,bool account_request)
```

- **Declared at:** [line 191](../../../runtime/skeletal.hpp#L191)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `entity` | `const ActorData *` | Input | Value supplied for `entity`. See the exact type and module contract. |
| `vertex` | `uint32_t` | Input | Value supplied for `vertex`. See the exact type and module contract. |
| `pose` | `PoseKind` | Input | Value supplied for `pose`. See the exact type and module contract. |
| `space` | `CoordinateSpace` | Input | Value supplied for `space`. See the exact type and module contract. |
| `account_request` | `bool` | Input | Value supplied for `account_request`. See the exact type and module contract. |

**Returns.** Returns `VertexSample`. Check the purpose and failure notes before using the value.

**Use it when.** You need rigid skeletal animation and pose evaluation and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "skeletal.hpp"

// Assume these named values have been initialized with valid data:
// const ActorData * entity
// uint32_t vertex
// PoseKind pose
// CoordinateSpace space
// bool account_request

auto result = epok::skeletal_sample_vertex_impl(entity, vertex, pose, space, account_request);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
