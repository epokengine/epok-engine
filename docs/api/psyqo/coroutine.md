# PsyQo API: Coroutine

> **Header:** `"psyqo/coroutine.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh)

This module covers the coroutine module. It documents 34 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Coroutine`, `psyqo::Coroutine::Awaiter`, `psyqo::Coroutine::ChainAwaiter`, `psyqo::Coroutine::Empty`, `psyqo::Coroutine::promise_type`, `psyqo::Stackful`, `psyqo::StackfulBase`

## Callable index

- [`psyqo::Coroutine::awaiter`](#psyqo-coroutine-awaiter-1) — Creates an `Awaiter` object.
- [`psyqo::Coroutine::Awaiter::await_ready`](#psyqo-coroutine-awaiter-await-ready-1) — Performs `await ready` as part of the coroutine module.
- [`psyqo::Coroutine::Awaiter::await_resume`](#psyqo-coroutine-awaiter-await-resume-1) — Performs `await resume` as part of the coroutine module.
- [`psyqo::Coroutine::Awaiter::await_suspend`](#psyqo-coroutine-awaiter-await-suspend-1) — Performs `await suspend` as part of the coroutine module.
- [`psyqo::Coroutine::Awaiter::Awaiter`](#psyqo-coroutine-awaiter-awaiter-1) — Constructs `psyqo::Coroutine::Awaiter` for the coroutine module.
- [`psyqo::Coroutine::Awaiter::Awaiter`](#psyqo-coroutine-awaiter-awaiter-2) — Constructs `psyqo::Coroutine::Awaiter` for the coroutine module.
- [`psyqo::Coroutine::Awaiter::operator=`](#psyqo-coroutine-awaiter-operator-1) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::Awaiter::operator=`](#psyqo-coroutine-awaiter-operator-2) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::await_ready`](#psyqo-coroutine-chainawaiter-await-ready-1) — Performs `await ready` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::await_resume`](#psyqo-coroutine-chainawaiter-await-resume-1) — Performs `await resume` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::await_suspend`](#psyqo-coroutine-chainawaiter-await-suspend-1) — Performs `await suspend` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::ChainAwaiter`](#psyqo-coroutine-chainawaiter-chainawaiter-1) — Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::ChainAwaiter`](#psyqo-coroutine-chainawaiter-chainawaiter-2) — Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::ChainAwaiter`](#psyqo-coroutine-chainawaiter-chainawaiter-3) — Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::operator=`](#psyqo-coroutine-chainawaiter-operator-1) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::operator=`](#psyqo-coroutine-chainawaiter-operator-2) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::ChainAwaiter::~ChainAwaiter`](#psyqo-coroutine-chainawaiter-chainawaiter-4) — Releases the resources owned by `psyqo::Coroutine::ChainAwaiter`.
- [`psyqo::Coroutine::Coroutine<T>`](#psyqo-coroutine-coroutine-t-1) — Constructs `psyqo::Coroutine` for the coroutine module.
- [`psyqo::Coroutine::Coroutine<T>`](#psyqo-coroutine-coroutine-t-2) — Constructs `psyqo::Coroutine` for the coroutine module.
- [`psyqo::Coroutine::Coroutine<T>`](#psyqo-coroutine-coroutine-t-3) — Constructs `psyqo::Coroutine` for the coroutine module.
- [`psyqo::Coroutine::done`](#psyqo-coroutine-done-1) — Returns the status of the coroutine.
- [`psyqo::Coroutine::operator co_await`](#psyqo-coroutine-operator-co-await-1) — Performs `operator  co await` as part of the coroutine module.
- [`psyqo::Coroutine::operator=`](#psyqo-coroutine-operator-1) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::operator=`](#psyqo-coroutine-operator-2) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Coroutine::resume`](#psyqo-coroutine-resume-1) — Resumes the coroutine.
- [`psyqo::Coroutine::value`](#psyqo-coroutine-value-1) — Returns the value returned by the coroutine.
- [`psyqo::Coroutine::~Coroutine<T>`](#psyqo-coroutine-coroutine-t-4) — Releases the resources owned by `psyqo::Coroutine`.
- [`psyqo::Stackful::initialize`](#psyqo-stackful-initialize-1) — Initialize the coroutine with a function and an argument.
- [`psyqo::Stackful::isAlive`](#psyqo-stackful-isalive-1) — Check if the coroutine is currently alive.
- [`psyqo::Stackful::operator=`](#psyqo-stackful-operator-1) — Performs `operator =` as part of the coroutine module.
- [`psyqo::Stackful::resume`](#psyqo-stackful-resume-1) — Resume the coroutine.
- [`psyqo::Stackful::Stackful<StackSize>`](#psyqo-stackful-stackful-stacksize-1) — Constructs `psyqo::Stackful` for the coroutine module.
- [`psyqo::Stackful::Stackful<StackSize>`](#psyqo-stackful-stackful-stacksize-2) — Constructs `psyqo::Stackful` for the coroutine module.
- [`psyqo::Stackful::yield`](#psyqo-stackful-yield-1) — Yield the coroutine.

<a id="psyqo-coroutine-awaiter-1"></a>

## `psyqo::Coroutine::awaiter`

**Purpose.** Creates an `Awaiter` object.

**Details.** This method is used to create an instance of the `Awaiter` object. It's used to suspend the coroutine after scheduling an asynchronous operation.

**Exact declaration**

```cpp
Awaiter awaiter() &
```

- **Declared at:** [line 131](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L131)
- **Kind:** `cxx method`

**Returns.** Returns `Awaiter`. Check the purpose and failure notes before using the value.

**Use it when.** This method is used to create an instance of the `Awaiter` object. It's used to suspend the coroutine after scheduling an asynchronous operation.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.awaiter();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-awaiter-await-ready-1"></a>

## `psyqo::Coroutine::Awaiter::await_ready`

**Purpose.** Performs `await ready` as part of the coroutine module.

**Exact declaration**

```cpp
constexpr bool await_ready() const noexcept
```

- **Declared at:** [line 111](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L111)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine::Awaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-coroutine-awaiter-await-resume-1"></a>

## `psyqo::Coroutine::Awaiter::await_resume`

**Purpose.** Performs `await resume` as part of the coroutine module.

**Exact declaration**

```cpp
constexpr void await_resume() const noexcept
```

- **Declared at:** [line 117](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L117)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine::Awaiter& object = /* obtain a valid instance */;

object.await_resume();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-awaiter-await-suspend-1"></a>

## `psyqo::Coroutine::Awaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the coroutine module.

**Exact declaration**

```cpp
constexpr void await_suspend(std::coroutine_handle<> h)
```

- **Declared at:** [line 116](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L116)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `std::coroutine_handle<>` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<> h

psyqo::Coroutine::Awaiter& object = /* obtain a valid instance */;

object.await_suspend(h);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-awaiter-awaiter-1"></a>

## `psyqo::Coroutine::Awaiter::Awaiter`

**Purpose.** Constructs `psyqo::Coroutine::Awaiter` for the coroutine module.

**Exact declaration**

```cpp
Awaiter(Awaiter &&other) = default
```

- **Declared at:** [line 107](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L107)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `Awaiter &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// Awaiter && other

psyqo::Coroutine::Awaiter value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-awaiter-awaiter-2"></a>

## `psyqo::Coroutine::Awaiter::Awaiter`

**Purpose.** Constructs `psyqo::Coroutine::Awaiter` for the coroutine module.

**Exact declaration**

```cpp
Awaiter(Awaiter const &) = default
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L109)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Awaiter &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Awaiter & arg1

psyqo::Coroutine::Awaiter value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-awaiter-operator-1"></a>

## `psyqo::Coroutine::Awaiter::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
Awaiter &operator=(Awaiter &&other) = default
```

- **Declared at:** [line 108](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L108)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `Awaiter &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `Awaiter &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// Awaiter && other

psyqo::Coroutine::Awaiter& object = /* obtain a valid instance */;

auto result = object.operator=(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-awaiter-operator-2"></a>

## `psyqo::Coroutine::Awaiter::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
Awaiter &operator=(Awaiter const &) = default
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L110)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Awaiter &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Awaiter &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Awaiter & arg1

psyqo::Coroutine::Awaiter& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-chainawaiter-await-ready-1"></a>

## `psyqo::Coroutine::ChainAwaiter::await_ready`

**Purpose.** Performs `await ready` as part of the coroutine module.

**Exact declaration**

```cpp
constexpr bool await_ready()
```

- **Declared at:** [line 256](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L256)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine::ChainAwaiter& object = /* obtain a valid instance */;

auto result = object.await_ready();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-coroutine-chainawaiter-await-resume-1"></a>

## `psyqo::Coroutine::ChainAwaiter::await_resume`

**Purpose.** Performs `await resume` as part of the coroutine module.

**Exact declaration**

```cpp
constexpr T await_resume()
```

- **Declared at:** [line 263](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L263)
- **Kind:** `cxx method`

**Returns.** Returns `T`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine::ChainAwaiter& object = /* obtain a valid instance */;

auto result = object.await_resume();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-chainawaiter-await-suspend-1"></a>

## `psyqo::Coroutine::ChainAwaiter::await_suspend`

**Purpose.** Performs `await suspend` as part of the coroutine module.

**Exact declaration**

```cpp
void await_suspend(std::coroutine_handle<> h)
```

- **Declared at:** [line 258](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L258)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `std::coroutine_handle<>` | Input | Value supplied for `h`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<> h

psyqo::Coroutine::ChainAwaiter& object = /* obtain a valid instance */;

object.await_suspend(h);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-chainawaiter-chainawaiter-1"></a>

## `psyqo::Coroutine::ChainAwaiter::ChainAwaiter`

**Purpose.** Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.

**Exact declaration**

```cpp
ChainAwaiter(ChainAwaiter &&other) : handl
```

- **Declared at:** [line 251](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L251)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `ChainAwaiter &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// ChainAwaiter && other

psyqo::Coroutine::ChainAwaiter value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-chainawaiter-chainawaiter-2"></a>

## `psyqo::Coroutine::ChainAwaiter::ChainAwaiter`

**Purpose.** Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.

**Exact declaration**

```cpp
ChainAwaiter(const ChainAwaiter &) = delete
```

- **Declared at:** [line 253](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L253)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ChainAwaiter &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const ChainAwaiter & arg1

psyqo::Coroutine::ChainAwaiter value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-chainawaiter-chainawaiter-3"></a>

## `psyqo::Coroutine::ChainAwaiter::ChainAwaiter`

**Purpose.** Constructs `psyqo::Coroutine::ChainAwaiter` for the coroutine module.

**Exact declaration**

```cpp
explicit ChainAwaiter(std::coroutine_handle<Promise> h) : handl
```

- **Declared at:** [line 246](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L246)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `std::coroutine_handle<Promise>` | Input | Value supplied for `h`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// std::coroutine_handle<Promise> h

psyqo::Coroutine::ChainAwaiter value(h);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-chainawaiter-operator-1"></a>

## `psyqo::Coroutine::ChainAwaiter::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
ChainAwaiter &operator=(ChainAwaiter &&) = delete
```

- **Declared at:** [line 252](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L252)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `ChainAwaiter &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ChainAwaiter &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// ChainAwaiter && arg1

psyqo::Coroutine::ChainAwaiter& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-chainawaiter-operator-2"></a>

## `psyqo::Coroutine::ChainAwaiter::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
ChainAwaiter &operator=(const ChainAwaiter &) = delete
```

- **Declared at:** [line 254](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L254)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const ChainAwaiter &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `ChainAwaiter &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const ChainAwaiter & arg1

psyqo::Coroutine::ChainAwaiter& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-chainawaiter-chainawaiter-4"></a>

## `psyqo::Coroutine::ChainAwaiter::~ChainAwaiter`

**Purpose.** Releases the resources owned by `psyqo::Coroutine::ChainAwaiter`.

**Exact declaration**

```cpp
~ChainAwaiter()
```

- **Declared at:** [line 247](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L247)
- **Kind:** `destructor`

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// `psyqo::Coroutine::ChainAwaiter` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-coroutine-t-1"></a>

## `psyqo::Coroutine::Coroutine<T>`

**Purpose.** Constructs `psyqo::Coroutine` for the coroutine module.

**Exact declaration**

```cpp
Coroutine() = default
```

- **Declared at:** [line 58](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L58)
- **Kind:** `constructor`

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine value();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-coroutine-t-2"></a>

## `psyqo::Coroutine::Coroutine<T>`

**Purpose.** Constructs `psyqo::Coroutine` for the coroutine module.

**Exact declaration**

```cpp
Coroutine(Coroutine &&other)
```

- **Declared at:** [line 60](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L60)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `Coroutine<T> &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// Coroutine<T> && other

psyqo::Coroutine value(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-coroutine-t-3"></a>

## `psyqo::Coroutine::Coroutine<T>`

**Purpose.** Constructs `psyqo::Coroutine` for the coroutine module.

**Exact declaration**

```cpp
Coroutine(Coroutine const &) = delete
```

- **Declared at:** [line 91](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L91)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Coroutine<T> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Coroutine<T> & arg1

psyqo::Coroutine value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-done-1"></a>

## `psyqo::Coroutine::done`

**Purpose.** Returns the status of the coroutine.

**Details.** This method returns the status of the coroutine. It will return `true` if the coroutine is done executing, `false` otherwise. The typical usage of this method is to poll it from the scene loop. The first time it returns `true`, the coroutine will be destroyed. The next times, it will return `true` without doing anything, making the polling loop faster.

**Exact declaration**

```cpp
bool done()
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L161)
- **Kind:** `cxx method`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** This method returns the status of the coroutine. It will return `true` if the coroutine is done executing, `false` otherwise. The typical usage of this method is to poll it from the scene loop. The first time it returns `true`, the coroutine will be destroyed. The next times, it will return `true` without doing anything, making the polling loop faster.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.done();
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-coroutine-operator-co-await-1"></a>

## `psyqo::Coroutine::operator co_await`

**Purpose.** Performs `operator  co await` as part of the coroutine module.

**Exact declaration**

```cpp
ChainAwaiter operator co_await() &&
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L277)
- **Kind:** `cxx method`

**Returns.** Returns `ChainAwaiter`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.operator co_await();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-operator-1"></a>

## `psyqo::Coroutine::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
Coroutine &operator=(Coroutine &&other)
```

- **Declared at:** [line 74](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L74)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `Coroutine<T> &&` | Consumed or moved input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `Coroutine<T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// Coroutine<T> && other

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.operator=(other);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-operator-2"></a>

## `psyqo::Coroutine::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
Coroutine &operator=(Coroutine const &) = delete
```

- **Declared at:** [line 92](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L92)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Coroutine<T> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Coroutine<T> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Coroutine<T> & arg1

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-coroutine-resume-1"></a>

## `psyqo::Coroutine::resume`

**Purpose.** Resumes the coroutine.

**Details.** This method resumes the coroutine. It's used to resume the coroutine after an asynchronous operation has completed. It is safe to call it from within the coroutine itself, meaning it is safe to call it from a callback which may execute in the same callstack as the coroutine. In this case, the next `co_yield` on the `Awaiter` object will be a no-op.

**Exact declaration**

```cpp
void resume()
```

- **Declared at:** [line 142](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L142)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method resumes the coroutine. It's used to resume the coroutine after an asynchronous operation has completed. It is safe to call it from within the coroutine itself, meaning it is safe to call it from a callback which may execute in the same callstack as the coroutine. In this case, the next `co_yield` on the `Awaiter` object will be a no-op.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine& object = /* obtain a valid instance */;

object.resume();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-value-1"></a>

## `psyqo::Coroutine::value`

**Purpose.** Returns the value returned by the coroutine.

**Details.** This method returns the value returned by the coroutine. It is only valid to call it after the coroutine has finished executing. The typical usage of this method is to call it after the `done` method returns `true`. The coroutine sets its return value using the `co_return` keyword. Since it is possible for the return type to be `void`, the return type of this method is `T` if `T` is not `void`, and `Empty` if `T` is `void`.

**Exact declaration**

```cpp
const SafeT &value() const
```

- **Declared at:** [line 185](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L185)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const SafeT &`. Check the purpose and failure notes before using the value.

**Use it when.** This method returns the value returned by the coroutine. It is only valid to call it after the coroutine has finished executing. The typical usage of this method is to call it after the `done` method returns `true`. The coroutine sets its return value using the `co_return` keyword. Since it is possible for the return type to be `void`, the return type of this method is `T` if `T` is not `void`, and `Empty` if `T` is `void`.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Coroutine& object = /* obtain a valid instance */;

auto result = object.value();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-coroutine-coroutine-t-4"></a>

## `psyqo::Coroutine::~Coroutine<T>`

**Purpose.** Releases the resources owned by `psyqo::Coroutine`.

**Exact declaration**

```cpp
~Coroutine()
```

- **Declared at:** [line 93](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L93)
- **Kind:** `destructor`

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// `psyqo::Coroutine` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-stackful-initialize-1"></a>

## `psyqo::Stackful::initialize`

**Purpose.** Initialize the coroutine with a function and an argument.

**Exact declaration**

```cpp
void initialize(eastl::function<void()> &&func)
```

- **Declared at:** [line 335](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L335)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `func` | `eastl::function<void ()> &&` | Consumed or moved input | Function to be executed by the coroutine. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void ()> && func

psyqo::Stackful& object = /* obtain a valid instance */;

object.initialize(func);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-stackful-isalive-1"></a>

## `psyqo::Stackful::isAlive`

**Purpose.** Check if the coroutine is currently alive.

**Details.** A coroutine is considered alive if it has been initialized and has not yet completed its execution. It becomes not alive when it returns from its function.

**Exact declaration**

```cpp
bool isAlive() const
```

- **Declared at:** [line 368](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L368)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** true if the coroutine is alive, false otherwise.

**Use it when.** A coroutine is considered alive if it has been initialized and has not yet completed its execution. It becomes not alive when it returns from its function.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Stackful& object = /* obtain a valid instance */;

auto result = object.isAlive();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="psyqo-stackful-operator-1"></a>

## `psyqo::Stackful::operator=`

**Purpose.** Performs `operator =` as part of the coroutine module.

**Exact declaration**

```cpp
Stackful &operator=(const Stackful &) = delete
```

- **Declared at:** [line 327](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L327)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Stackful<StackSize> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Stackful<StackSize> &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Stackful<StackSize> & arg1

psyqo::Stackful& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-stackful-resume-1"></a>

## `psyqo::Stackful::resume`

**Purpose.** Resume the coroutine.

**Details.** This will switch to the coroutine's context and execute it. If the coroutine is not alive, this function does nothing. This function should be called after the coroutine has been initialized, and it will return to the point where the coroutine was last yielded. It can only be called from the "main thread".

**Exact declaration**

```cpp
void resume()
```

- **Declared at:** [line 348](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L348)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will switch to the coroutine's context and execute it. If the coroutine is not alive, this function does nothing. This function should be called after the coroutine has been initialized, and it will return to the point where the coroutine was last yielded. It can only be called from the "main thread".

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Stackful& object = /* obtain a valid instance */;

object.resume();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-stackful-stackful-stacksize-1"></a>

## `psyqo::Stackful::Stackful<StackSize>`

**Purpose.** Constructs `psyqo::Stackful` for the coroutine module.

**Exact declaration**

```cpp
Stackful() = default
```

- **Declared at:** [line 325](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L325)
- **Kind:** `constructor`

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Stackful value();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-stackful-stackful-stacksize-2"></a>

## `psyqo::Stackful::Stackful<StackSize>`

**Purpose.** Constructs `psyqo::Stackful` for the coroutine module.

**Exact declaration**

```cpp
Stackful(const Stackful &) = delete
```

- **Declared at:** [line 326](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L326)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Stackful<StackSize> &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the coroutine module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

// Assume these named values have been initialized with valid data:
// const Stackful<StackSize> & arg1

psyqo::Stackful value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-stackful-yield-1"></a>

## `psyqo::Stackful::yield`

**Purpose.** Yield the coroutine.

**Details.** This will switch back to the main thread and save the coroutine's context. The coroutine can be resumed later using `resume()`. It can only be called from within the coroutine to yield execution.

**Exact declaration**

```cpp
void yield()
```

- **Declared at:** [line 358](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/coroutine.hh#L358)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will switch back to the main thread and save the coroutine's context. The coroutine can be resumed later using `resume()`. It can only be called from within the coroutine to yield execution.

**Usage pattern**

```cpp
#include "psyqo/coroutine.hh"

psyqo::Stackful& object = /* obtain a valid instance */;

object.yield();
```

**Why choose it.** It provides direct, allocation-conscious access to the coroutine module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
