# PsyQo API: Sio0 Bus

> **Header:** `"psyqo/sio0-bus.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh)

This module covers the sio0 bus module. It documents 7 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::SIO0Bus`, `psyqo::SIO0Bus::Lock`

## Callable index

- [`psyqo::SIO0Bus::acquire`](#psyqo-sio0bus-acquire-1) — Takes ownership of the bus for the calling transaction.
- [`psyqo::SIO0Bus::Lock::Lock`](#psyqo-sio0bus-lock-lock-1) — Constructs `psyqo::SIO0Bus::Lock` for the sio0 bus module.
- [`psyqo::SIO0Bus::Lock::Lock`](#psyqo-sio0bus-lock-lock-2) — Constructs `psyqo::SIO0Bus::Lock` for the sio0 bus module.
- [`psyqo::SIO0Bus::Lock::operator=`](#psyqo-sio0bus-lock-operator-1) — Performs `operator =` as part of the sio0 bus module.
- [`psyqo::SIO0Bus::Lock::~Lock`](#psyqo-sio0bus-lock-lock-3) — Releases the resources owned by `psyqo::SIO0Bus::Lock`.
- [`psyqo::SIO0Bus::owned`](#psyqo-sio0bus-owned-1) — Returns whether the bus is currently owned by a card transaction.
- [`psyqo::SIO0Bus::release`](#psyqo-sio0bus-release-1) — Releases one level of ownership taken by `acquire`.

<a id="psyqo-sio0bus-acquire-1"></a>

## `psyqo::SIO0Bus::acquire`

**Purpose.** Takes ownership of the bus for the calling transaction.

**Exact declaration**

```cpp
static void acquire()
```

- **Declared at:** [line 59](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L59)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

psyqo::SIO0Bus::acquire();
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-sio0bus-lock-lock-1"></a>

## `psyqo::SIO0Bus::Lock::Lock`

**Purpose.** Constructs `psyqo::SIO0Bus::Lock` for the sio0 bus module.

**Exact declaration**

```cpp
Lock()
```

- **Declared at:** [line 80](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L80)
- **Kind:** `constructor`

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

psyqo::SIO0Bus::Lock value();
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-sio0bus-lock-lock-2"></a>

## `psyqo::SIO0Bus::Lock::Lock`

**Purpose.** Constructs `psyqo::SIO0Bus::Lock` for the sio0 bus module.

**Exact declaration**

```cpp
Lock(const Lock &) = delete
```

- **Declared at:** [line 82](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L82)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Lock &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

// Assume these named values have been initialized with valid data:
// const Lock & arg1

psyqo::SIO0Bus::Lock value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-sio0bus-lock-operator-1"></a>

## `psyqo::SIO0Bus::Lock::operator=`

**Purpose.** Performs `operator =` as part of the sio0 bus module.

**Exact declaration**

```cpp
Lock &operator=(const Lock &) = delete
```

- **Declared at:** [line 83](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L83)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Lock &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Lock &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

// Assume these named values have been initialized with valid data:
// const Lock & arg1

psyqo::SIO0Bus::Lock& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-sio0bus-lock-lock-3"></a>

## `psyqo::SIO0Bus::Lock::~Lock`

**Purpose.** Releases the resources owned by `psyqo::SIO0Bus::Lock`.

**Exact declaration**

```cpp
~Lock()
```

- **Declared at:** [line 81](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L81)
- **Kind:** `destructor`

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

// `psyqo::SIO0Bus::Lock` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-sio0bus-owned-1"></a>

## `psyqo::SIO0Bus::owned`

**Purpose.** Returns whether the bus is currently owned by a card transaction.

**Exact declaration**

```cpp
static bool owned()
```

- **Declared at:** [line 71](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L71)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

auto result = psyqo::SIO0Bus::owned();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-sio0bus-release-1"></a>

## `psyqo::SIO0Bus::release`

**Purpose.** Releases one level of ownership taken by `acquire`.

**Exact declaration**

```cpp
static void release()
```

- **Declared at:** [line 64](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/sio0-bus.hh#L64)
- **Kind:** `cxx method`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sio0 bus module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/sio0-bus.hh"

psyqo::SIO0Bus::release();
```

**Why choose it.** It provides direct, allocation-conscious access to the sio0 bus module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
