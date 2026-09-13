# Epok API: Serial Debug

> **Header:** `"serial_debug.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/serial_debug.hpp)

This module covers the serial debug module. It documents 2 public callables declared directly in this header.

## Callable index

- [`epok::serial_debug::capture`](#epok-serial-debug-capture-1) — Performs `capture` as part of the serial debug module.
- [`epok::serial_debug::restore`](#epok-serial-debug-restore-1) — Performs `restore` as part of the serial debug module.

<a id="epok-serial-debug-capture-1"></a>

## `epok::serial_debug::capture`

**Purpose.** Performs `capture` as part of the serial debug module.

**Exact declaration**

```cpp
inline void capture()
```

- **Declared at:** [line 34](../../../runtime/serial_debug.hpp#L34)
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

<a id="epok-serial-debug-restore-1"></a>

## `epok::serial_debug::restore`

**Purpose.** Performs `restore` as part of the serial debug module.

**Exact declaration**

```cpp
inline void restore()
```

- **Declared at:** [line 35](../../../runtime/serial_debug.hpp#L35)
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
