# Epok API: Motion Interpolation

> **Header:** `"motion_interpolation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/motion_interpolation.hpp)

This module covers the motion interpolation module. It documents 13 public callables declared directly in this header.

## Declared types

`epok::MotionInterpolation`, `epok::MotionInterpolation::History`

## Callable index

- [`epok::MotionInterpolation::after_tick`](#epok-motioninterpolation-after-tick-1) — Performs `after tick` as part of the motion interpolation module.
- [`epok::MotionInterpolation::after_tick`](#epok-motioninterpolation-after-tick-2) — Performs `after tick` as part of the motion interpolation module.
- [`epok::MotionInterpolation::before_tick`](#epok-motioninterpolation-before-tick-1) — Performs `before tick` as part of the motion interpolation module.
- [`epok::MotionInterpolation::before_tick`](#epok-motioninterpolation-before-tick-2) — Performs `before tick` as part of the motion interpolation module.
- [`epok::MotionInterpolation::clear`](#epok-motioninterpolation-clear-1) — Clears clear as part of the motion interpolation module.
- [`epok::MotionInterpolation::clear`](#epok-motioninterpolation-clear-2) — Clears clear as part of the motion interpolation module.
- [`epok::MotionInterpolation::local`](#epok-motioninterpolation-local-1) — Camera inverse composition uses the same local translations as meshes.
- [`epok::MotionInterpolation::local`](#epok-motioninterpolation-local-2) — Performs `local` as part of the motion interpolation module.
- [`epok::MotionInterpolation::MotionInterpolation<type-parameter-0-0, Capacity, true>`](#epok-motioninterpolation-motioninterpolation-type-parameter-0-0-capacity-true-1) — Constructs `epok::MotionInterpolation` for the motion interpolation module.
- [`epok::MotionInterpolation::prepare`](#epok-motioninterpolation-prepare-1) — Performs `prepare` as part of the motion interpolation module.
- [`epok::MotionInterpolation::prepare`](#epok-motioninterpolation-prepare-2) — Performs `prepare` as part of the motion interpolation module.
- [`epok::MotionInterpolation::select`](#epok-motioninterpolation-select-1) — Only 3D presentation participants and their ancestors need histories.
- [`epok::MotionInterpolation::select`](#epok-motioninterpolation-select-2) — Performs `select` as part of the motion interpolation module.

<a id="epok-motioninterpolation-after-tick-1"></a>

## `epok::MotionInterpolation::after_tick`

**Purpose.** Performs `after tick` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> void after_tick(const Objects& objects,size_t count)
```

- **Declared at:** [line 93](../../../runtime/motion_interpolation.hpp#L93)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & objects
// int count

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.after_tick<Objects>(objects, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-after-tick-2"></a>

## `epok::MotionInterpolation::after_tick`

**Purpose.** Performs `after tick` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> void after_tick(const Objects&,size_t)
```

- **Declared at:** [line 15](../../../runtime/motion_interpolation.hpp#L15)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Objects &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `int` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & arg1
// int arg2

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.after_tick<Objects>(arg1, arg2);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-before-tick-1"></a>

## `epok::MotionInterpolation::before_tick`

**Purpose.** Performs `before tick` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> void before_tick(const Objects& objects,size_t count)
```

- **Declared at:** [line 83](../../../runtime/motion_interpolation.hpp#L83)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & objects
// int count

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.before_tick<Objects>(objects, count);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-before-tick-2"></a>

## `epok::MotionInterpolation::before_tick`

**Purpose.** Performs `before tick` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> void before_tick(const Objects&,size_t)
```

- **Declared at:** [line 14](../../../runtime/motion_interpolation.hpp#L14)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Objects &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `int` | Input | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & arg1
// int arg2

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.before_tick<Objects>(arg1, arg2);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-clear-1"></a>

## `epok::MotionInterpolation::clear`

**Purpose.** Clears clear as part of the motion interpolation module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 12](../../../runtime/motion_interpolation.hpp#L12)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the motion interpolation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-motioninterpolation-clear-2"></a>

## `epok::MotionInterpolation::clear`

**Purpose.** Clears clear as part of the motion interpolation module.

**Exact declaration**

```cpp
void clear()
```

- **Declared at:** [line 72](../../../runtime/motion_interpolation.hpp#L72)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.clear();
```

**Why choose it.** It provides direct, allocation-conscious access to the motion interpolation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-motioninterpolation-local-1"></a>

## `epok::MotionInterpolation::local`

**Purpose.** Camera inverse composition uses the same local translations as meshes.

**Exact declaration**

```cpp
template<class Transform> Transform local(size_t i,const Transform& value) const
```

- **Declared at:** [line 115](../../../runtime/motion_interpolation.hpp#L115)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `i` | `int` | Input | Value supplied for `i`. See the exact type and module contract. |
| `value` | `const Transform &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Transform`. Check the purpose and failure notes before using the value.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Transform

// Assume these named values have been initialized with valid data:
// int i
// const Transform & value

epok::MotionInterpolation& object = /* obtain a valid instance */;

auto result = object.local<Transform>(i, value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-local-2"></a>

## `epok::MotionInterpolation::local`

**Purpose.** Performs `local` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Transform> Transform local(size_t,const Transform& value) const
```

- **Declared at:** [line 18](../../../runtime/motion_interpolation.hpp#L18)
- **Kind:** `function template`; qualifiers: `const, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `int` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `value` | `const Transform &` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** Returns `Transform`. Check the purpose and failure notes before using the value.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Transform

// Assume these named values have been initialized with valid data:
// int arg1
// const Transform & value

epok::MotionInterpolation& object = /* obtain a valid instance */;

auto result = object.local<Transform>(arg1, value);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-motioninterpolation-type-parameter-0-0-capacity-true-1"></a>

## `epok::MotionInterpolation::MotionInterpolation<type-parameter-0-0, Capacity, true>`

**Purpose.** Constructs `epok::MotionInterpolation` for the motion interpolation module.

**Exact declaration**

```cpp
MotionInterpolation()
```

- **Declared at:** [line 71](../../../runtime/motion_interpolation.hpp#L71)
- **Kind:** `constructor`

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

epok::MotionInterpolation value();
```

**Why choose it.** It provides direct, allocation-conscious access to the motion interpolation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-motioninterpolation-prepare-1"></a>

## `epok::MotionInterpolation::prepare`

**Purpose.** Performs `prepare` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> const std::array<Affine<Number>,Capacity>& prepare(const Objects& objects,const std::array<Affine<Number>,Capacity>& world,size_t count,unsigned fraction)
```

- **Declared at:** [line 102](../../../runtime/motion_interpolation.hpp#L102)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `world` | `const int &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `fraction` | `unsigned int` | Input | Value supplied for `fraction`. See the exact type and module contract. |

**Returns.** Returns `const int &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & objects
// const int & world
// int count
// unsigned int fraction

epok::MotionInterpolation& object = /* obtain a valid instance */;

auto result = object.prepare<Objects>(objects, world, count, fraction);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-prepare-2"></a>

## `epok::MotionInterpolation::prepare`

**Purpose.** Performs `prepare` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects> const std::array<Affine<Number>,Capacity>& prepare(const Objects&,const std::array<Affine<Number>,Capacity>& world,size_t,unsigned)
```

- **Declared at:** [line 16](../../../runtime/motion_interpolation.hpp#L16)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Objects &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `world` | `const int &` | Input | Value supplied for `world`. See the exact type and module contract. |
| `arg3` | `int` | Input | Value supplied for `arg3`. See the exact type and module contract. |
| `arg4` | `unsigned int` | Input | Value supplied for `arg4`. See the exact type and module contract. |

**Returns.** Returns `const int &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects

// Assume these named values have been initialized with valid data:
// const Objects & arg1
// const int & world
// int arg3
// unsigned int arg4

epok::MotionInterpolation& object = /* obtain a valid instance */;

auto result = object.prepare<Objects>(arg1, world, arg3, arg4);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-select-1"></a>

## `epok::MotionInterpolation::select`

**Purpose.** Only 3D presentation participants and their ancestors need histories.

**Details.** Collider-only entities and screen-space menu trees remain authoritative.

**Exact declaration**

```cpp
template<class Objects,class Predicate> void select(const Objects& objects,size_t count,Predicate participant)
```

- **Declared at:** [line 75](../../../runtime/motion_interpolation.hpp#L75)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `objects` | `const Objects &` | Input | Value supplied for `objects`. See the exact type and module contract. |
| `count` | `int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `participant` | `Predicate` | Input | Value supplied for `participant`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Collider-only entities and screen-space menu trees remain authoritative.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects, Predicate

// Assume these named values have been initialized with valid data:
// const Objects & objects
// int count
// Predicate participant

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.select<Objects, Predicate>(objects, count, participant);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-motioninterpolation-select-2"></a>

## `epok::MotionInterpolation::select`

**Purpose.** Performs `select` as part of the motion interpolation module.

**Exact declaration**

```cpp
template<class Objects,class Predicate> void select(const Objects&,size_t,Predicate)
```

- **Declared at:** [line 13](../../../runtime/motion_interpolation.hpp#L13)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Objects &` | Input | Value supplied for `arg1`. See the exact type and module contract. |
| `arg2` | `int` | Input | Value supplied for `arg2`. See the exact type and module contract. |
| `arg3` | `Predicate` | Input | Value supplied for `arg3`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the motion interpolation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "motion_interpolation.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Objects, Predicate

// Assume these named values have been initialized with valid data:
// const Objects & arg1
// int arg2
// Predicate arg3

epok::MotionInterpolation& object = /* obtain a valid instance */;

object.select<Objects, Predicate>(arg1, arg2, arg3);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
