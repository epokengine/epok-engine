# Epok API: Serial Debug

> **Header:** `"serial_debug.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/serial_debug.hpp)

This module covers the serial debug module. It documents 3 public callables declared directly in this header.

## Callable index

- [`epok::serial_debug::capture`](#epok-serial-debug-capture-1) — Performs `capture` as part of the serial debug module.
- [`epok::serial_debug::install`](#epok-serial-debug-install-1) — Performs `install` as part of the serial debug module.
- [`epok::serial_debug::restore`](#epok-serial-debug-restore-1) — Performs `restore` as part of the serial debug module.

<a id="epok-serial-debug-capture-1"></a>

## `epok::serial_debug::capture`

**Purpose.** Performs `capture` as part of the serial debug module.

**Exact declaration**

```cpp
inline void capture()
```

- **Declared at:** [line 55](../../../runtime/serial_debug.hpp#L55)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the serial debug module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_debug.hpp"

epok::serial_debug::capture();
```

**Why choose it.** It provides direct, allocation-conscious access to the serial debug module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-serial-debug-install-1"></a>

## `epok::serial_debug::install`

**Purpose.** Performs `install` as part of the serial debug module.

**Exact declaration**

```cpp
template<class GPU> inline void install(GPU&)
```

- **Declared at:** [line 57](../../../runtime/serial_debug.hpp#L57)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the serial debug module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_debug.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// GPU

// Assume these named values have been initialized with valid data:
// GPU & arg1

epok::serial_debug& object = /* obtain a valid instance */;

object.install<GPU>(arg1);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-serial-debug-restore-1"></a>

## `epok::serial_debug::restore`

**Purpose.** Performs `restore` as part of the serial debug module.

**Exact declaration**

```cpp
inline void restore()
```

- **Declared at:** [line 56](../../../runtime/serial_debug.hpp#L56)
- **Kind:** `function decl`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the serial debug module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "serial_debug.hpp"

epok::serial_debug::restore();
```

**Why choose it.** It provides direct, allocation-conscious access to the serial debug module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
