# Epok API: Skeletal

> **Header:** `"skeletal.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/skeletal.hpp)

This module covers rigid skeletal animation and pose evaluation. It documents 2 public callables declared directly in this header.

## Declared types

`epok::skeletal_detail::Scratch`

## Callable index

- [`epok::skeletal_detail::pose_matrix`](#epok-skeletal-detail-pose-matrix-1) — Performs `pose matrix` as part of rigid skeletal animation and pose evaluation.
- [`epok::skeletal_detail::Scratch::pose`](#epok-skeletal-detail-scratch-pose-1) — Performs `pose` as part of rigid skeletal animation and pose evaluation.

<a id="epok-skeletal-detail-pose-matrix-1"></a>

## `epok::skeletal_detail::pose_matrix`

**Purpose.** Performs `pose matrix` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
inline Affine<Fixed> pose_matrix(const BonePose& p)
```

- **Declared at:** [line 7](../../../runtime/skeletal.hpp#L7)
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

<a id="epok-skeletal-detail-scratch-pose-1"></a>

## `epok::skeletal_detail::Scratch::pose`

**Purpose.** Performs `pose` as part of rigid skeletal animation and pose evaluation.

**Exact declaration**

```cpp
void pose(const SkeletalMesh& model, const Animator& animator)
```

- **Declared at:** [line 21](../../../runtime/skeletal.hpp#L21)
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

object.pose(model, animator);
```

**Why choose it.** It provides direct, allocation-conscious access to rigid skeletal animation and pose evaluation. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
