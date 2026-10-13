# Epok API: World2D Physics

> **Header:** `"world2d_physics.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/world2d_physics.hpp)

This module covers the world2d physics module. It documents 25 public callables declared directly in this header.

## Declared types

`epok::Joint2DComponent`, `epok::JointKind2D`, `epok::RigidBody2DComponent`

## Callable index

- [`epok::Joint2DComponent::break_joint`](#epok-joint2dcomponent-break-joint-1) — Cuts the joint for good: the rope snaps.
- [`epok::Joint2DComponent::class_id`](#epok-joint2dcomponent-class-id-1) — Performs `class id` as part of the world2d physics module.
- [`epok::Joint2DComponent::get_length`](#epok-joint2dcomponent-get-length-1) — Returns length as part of the world2d physics module.
- [`epok::Joint2DComponent::is_enabled`](#epok-joint2dcomponent-is-enabled-1) — Reports whether enabled as part of the world2d physics module.
- [`epok::Joint2DComponent::level_attached`](#epok-joint2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d physics module.
- [`epok::Joint2DComponent::set_enabled`](#epok-joint2dcomponent-set-enabled-1) — Sets enabled as part of the world2d physics module.
- [`epok::Joint2DComponent::set_length`](#epok-joint2dcomponent-set-length-1) — Sets length as part of the world2d physics module.
- [`epok::Joint2DComponent::~Joint2DComponent`](#epok-joint2dcomponent-joint2dcomponent-1) — Releases the resources owned by `epok::Joint2DComponent`.
- [`epok::RigidBody2DComponent::apply_force`](#epok-rigidbody2dcomponent-apply-force-1) — A force for the next simulation step only (call it every tick to keep pushing).
- [`epok::RigidBody2DComponent::apply_impulse`](#epok-rigidbody2dcomponent-apply-impulse-1) — Performs `apply impulse` as part of the world2d physics module.
- [`epok::RigidBody2DComponent::class_id`](#epok-rigidbody2dcomponent-class-id-1) — Performs `class id` as part of the world2d physics module.
- [`epok::RigidBody2DComponent::get_angular_velocity`](#epok-rigidbody2dcomponent-get-angular-velocity-1) — Returns angular velocity as part of the world2d physics module.
- [`epok::RigidBody2DComponent::get_impact`](#epok-rigidbody2dcomponent-get-impact-1) — Returns impact as part of the world2d physics module.
- [`epok::RigidBody2DComponent::get_velocity_x`](#epok-rigidbody2dcomponent-get-velocity-x-1) — Returns velocity x as part of the world2d physics module.
- [`epok::RigidBody2DComponent::get_velocity_y`](#epok-rigidbody2dcomponent-get-velocity-y-1) — Returns velocity y as part of the world2d physics module.
- [`epok::RigidBody2DComponent::inverse_mass`](#epok-rigidbody2dcomponent-inverse-mass-1) — Inverse mass in Q12 per unit: 0 for a frozen (immovable) body.
- [`epok::RigidBody2DComponent::is_frozen`](#epok-rigidbody2dcomponent-is-frozen-1) — Reports whether frozen as part of the world2d physics module.
- [`epok::RigidBody2DComponent::is_sleeping`](#epok-rigidbody2dcomponent-is-sleeping-1) — Reports whether sleeping as part of the world2d physics module.
- [`epok::RigidBody2DComponent::is_touching`](#epok-rigidbody2dcomponent-is-touching-1) — Reports whether touching as part of the world2d physics module.
- [`epok::RigidBody2DComponent::level_attached`](#epok-rigidbody2dcomponent-level-attached-1) — Performs `level attached` as part of the world2d physics module.
- [`epok::RigidBody2DComponent::set_frozen`](#epok-rigidbody2dcomponent-set-frozen-1) — A frozen body stays where it is and blocks like a static one.
- [`epok::RigidBody2DComponent::set_velocity`](#epok-rigidbody2dcomponent-set-velocity-1) — Sets velocity as part of the world2d physics module.
- [`epok::RigidBody2DComponent::take_contact`](#epok-rigidbody2dcomponent-take-contact-1) — True once when the body starts touching something; `get_impact` is how hard (px/s).
- [`epok::RigidBody2DComponent::wake`](#epok-rigidbody2dcomponent-wake-1) — Performs `wake` as part of the world2d physics module.
- [`epok::RigidBody2DComponent::~RigidBody2DComponent`](#epok-rigidbody2dcomponent-rigidbody2dcomponent-1) — Releases the resources owned by `epok::RigidBody2DComponent`.

<a id="epok-joint2dcomponent-break-joint-1"></a>

## `epok::Joint2DComponent::break_joint`

**Purpose.** Cuts the joint for good: the rope snaps.

**Exact declaration**

```cpp
void break_joint()
```

- **Declared at:** [line 113](../../../runtime/world2d_physics.hpp#L113)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::Joint2DComponent& object = /* obtain a valid instance */;

object.break_joint();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-class-id-1"></a>

## `epok::Joint2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d physics module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 97](../../../runtime/world2d_physics.hpp#L97)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::Joint2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-get-length-1"></a>

## `epok::Joint2DComponent::get_length`

**Purpose.** Returns length as part of the world2d physics module.

**Exact declaration**

```cpp
Fixed get_length() const
```

- **Declared at:** [line 111](../../../runtime/world2d_physics.hpp#L111)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::Joint2DComponent& object = /* obtain a valid instance */;

auto result = object.get_length();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-is-enabled-1"></a>

## `epok::Joint2DComponent::is_enabled`

**Purpose.** Reports whether enabled as part of the world2d physics module.

**Exact declaration**

```cpp
bool is_enabled() const
```

- **Declared at:** [line 109](../../../runtime/world2d_physics.hpp#L109)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::Joint2DComponent& object = /* obtain a valid instance */;

auto result = object.is_enabled();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-joint2dcomponent-level-attached-1"></a>

## `epok::Joint2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d physics module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 98](../../../runtime/world2d_physics.hpp#L98)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::Joint2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-set-enabled-1"></a>

## `epok::Joint2DComponent::set_enabled`

**Purpose.** Sets enabled as part of the world2d physics module.

**Exact declaration**

```cpp
void set_enabled(bool value)
```

- **Declared at:** [line 108](../../../runtime/world2d_physics.hpp#L108)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::Joint2DComponent& object = /* obtain a valid instance */;

object.set_enabled(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-set-length-1"></a>

## `epok::Joint2DComponent::set_length`

**Purpose.** Sets length as part of the world2d physics module.

**Exact declaration**

```cpp
void set_length(Fixed value)
```

- **Declared at:** [line 110](../../../runtime/world2d_physics.hpp#L110)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `Fixed` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// Fixed value

epok::Joint2DComponent& object = /* obtain a valid instance */;

object.set_length(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-joint2dcomponent-joint2dcomponent-1"></a>

## `epok::Joint2DComponent::~Joint2DComponent`

**Purpose.** Releases the resources owned by `epok::Joint2DComponent`.

**Exact declaration**

```cpp
~Joint2DComponent() override
```

- **Declared at:** [line 99](../../../runtime/world2d_physics.hpp#L99)
- **Kind:** `destructor`

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// `epok::Joint2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-apply-force-1"></a>

## `epok::RigidBody2DComponent::apply_force`

**Purpose.** A force for the next simulation step only (call it every tick to keep pushing).

**Exact declaration**

```cpp
void apply_force(Fixed x, Fixed y)
```

- **Declared at:** [line 56](../../../runtime/world2d_physics.hpp#L56)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.apply_force(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-apply-impulse-1"></a>

## `epok::RigidBody2DComponent::apply_impulse`

**Purpose.** Performs `apply impulse` as part of the world2d physics module.

**Exact declaration**

```cpp
void apply_impulse(Fixed x, Fixed y)
```

- **Declared at:** [line 49](../../../runtime/world2d_physics.hpp#L49)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.apply_impulse(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-class-id-1"></a>

## `epok::RigidBody2DComponent::class_id`

**Purpose.** Performs `class id` as part of the world2d physics module.

**Exact declaration**

```cpp
uint64_t class_id() const override
```

- **Declared at:** [line 34](../../../runtime/world2d_physics.hpp#L34)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.class_id();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-get-angular-velocity-1"></a>

## `epok::RigidBody2DComponent::get_angular_velocity`

**Purpose.** Returns angular velocity as part of the world2d physics module.

**Exact declaration**

```cpp
Fixed get_angular_velocity() const
```

- **Declared at:** [line 68](../../../runtime/world2d_physics.hpp#L68)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.get_angular_velocity();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-get-impact-1"></a>

## `epok::RigidBody2DComponent::get_impact`

**Purpose.** Returns impact as part of the world2d physics module.

**Exact declaration**

```cpp
Fixed get_impact() const
```

- **Declared at:** [line 87](../../../runtime/world2d_physics.hpp#L87)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.get_impact();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-get-velocity-x-1"></a>

## `epok::RigidBody2DComponent::get_velocity_x`

**Purpose.** Returns velocity x as part of the world2d physics module.

**Exact declaration**

```cpp
Fixed get_velocity_x() const
```

- **Declared at:** [line 66](../../../runtime/world2d_physics.hpp#L66)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.get_velocity_x();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-get-velocity-y-1"></a>

## `epok::RigidBody2DComponent::get_velocity_y`

**Purpose.** Returns velocity y as part of the world2d physics module.

**Exact declaration**

```cpp
Fixed get_velocity_y() const
```

- **Declared at:** [line 67](../../../runtime/world2d_physics.hpp#L67)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `Fixed`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.get_velocity_y();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-inverse-mass-1"></a>

## `epok::RigidBody2DComponent::inverse_mass`

**Purpose.** Inverse mass in Q12 per unit: 0 for a frozen (immovable) body.

**Exact declaration**

```cpp
int64_t inverse_mass() const
```

- **Declared at:** [line 90](../../../runtime/world2d_physics.hpp#L90)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int64_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.inverse_mass();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-is-frozen-1"></a>

## `epok::RigidBody2DComponent::is_frozen`

**Purpose.** Reports whether frozen as part of the world2d physics module.

**Exact declaration**

```cpp
bool is_frozen() const
```

- **Declared at:** [line 80](../../../runtime/world2d_physics.hpp#L80)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_frozen();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rigidbody2dcomponent-is-sleeping-1"></a>

## `epok::RigidBody2DComponent::is_sleeping`

**Purpose.** Reports whether sleeping as part of the world2d physics module.

**Exact declaration**

```cpp
bool is_sleeping() const
```

- **Declared at:** [line 69](../../../runtime/world2d_physics.hpp#L69)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_sleeping();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rigidbody2dcomponent-is-touching-1"></a>

## `epok::RigidBody2DComponent::is_touching`

**Purpose.** Reports whether touching as part of the world2d physics module.

**Exact declaration**

```cpp
bool is_touching() const
```

- **Declared at:** [line 88](../../../runtime/world2d_physics.hpp#L88)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.is_touching();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rigidbody2dcomponent-level-attached-1"></a>

## `epok::RigidBody2DComponent::level_attached`

**Purpose.** Performs `level attached` as part of the world2d physics module.

**Exact declaration**

```cpp
void level_attached() override
```

- **Declared at:** [line 35](../../../runtime/world2d_physics.hpp#L35)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.level_attached();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-set-frozen-1"></a>

## `epok::RigidBody2DComponent::set_frozen`

**Purpose.** A frozen body stays where it is and blocks like a static one.

**Exact declaration**

```cpp
void set_frozen(bool value)
```

- **Declared at:** [line 75](../../../runtime/world2d_physics.hpp#L75)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `bool` | Input | Value supplied for `value`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// bool value

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.set_frozen(value);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-set-velocity-1"></a>

## `epok::RigidBody2DComponent::set_velocity`

**Purpose.** Sets velocity as part of the world2d physics module.

**Exact declaration**

```cpp
void set_velocity(Fixed x, Fixed y)
```

- **Declared at:** [line 61](../../../runtime/world2d_physics.hpp#L61)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `Fixed` | Input | Value supplied for `x`. See the exact type and module contract. |
| `y` | `Fixed` | Input | Value supplied for `y`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// Assume these named values have been initialized with valid data:
// Fixed x
// Fixed y

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.set_velocity(x, y);
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-take-contact-1"></a>

## `epok::RigidBody2DComponent::take_contact`

**Purpose.** True once when the body starts touching something; `get_impact` is how hard (px/s).

**Exact declaration**

```cpp
bool take_contact()
```

- **Declared at:** [line 82](../../../runtime/world2d_physics.hpp#L82)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

auto result = object.take_contact();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-rigidbody2dcomponent-wake-1"></a>

## `epok::RigidBody2DComponent::wake`

**Purpose.** Performs `wake` as part of the world2d physics module.

**Exact declaration**

```cpp
void wake()
```

- **Declared at:** [line 70](../../../runtime/world2d_physics.hpp#L70)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

epok::RigidBody2DComponent& object = /* obtain a valid instance */;

object.wake();
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-rigidbody2dcomponent-rigidbody2dcomponent-1"></a>

## `epok::RigidBody2DComponent::~RigidBody2DComponent`

**Purpose.** Releases the resources owned by `epok::RigidBody2DComponent`.

**Exact declaration**

```cpp
~RigidBody2DComponent() override
```

- **Declared at:** [line 36](../../../runtime/world2d_physics.hpp#L36)
- **Kind:** `destructor`

**Use it when.** You need the world2d physics module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "world2d_physics.hpp"

// `epok::RigidBody2DComponent` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the world2d physics module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
