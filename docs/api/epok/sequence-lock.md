# Epok API: Sequence Lock

> **Header:** `"sequence_lock.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_lock.hpp)

This module covers the sequence lock module. It documents 2 public callables declared directly in this header.

## Declared types

`epok::SequenceLock`

## Callable index

- [`epok::SequenceLock::SequenceLock`](#epok-sequencelock-sequencelock-1) — Constructs `epok::SequenceLock` for the sequence lock module.
- [`epok::SequenceLock::~SequenceLock`](#epok-sequencelock-sequencelock-2) — Releases the resources owned by `epok::SequenceLock`.

<a id="epok-sequencelock-sequencelock-1"></a>

## `epok::SequenceLock::SequenceLock`

**Purpose.** Constructs `epok::SequenceLock` for the sequence lock module.

**Exact declaration**

```cpp
SequenceLock() : p
```

- **Declared at:** [line 13](../../../runtime/sequence_lock.hpp#L13)
- **Kind:** `constructor`

**Use it when.** You need the sequence lock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_lock.hpp"

epok::SequenceLock value();
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence lock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-sequencelock-sequencelock-2"></a>

## `epok::SequenceLock::~SequenceLock`

**Purpose.** Releases the resources owned by `epok::SequenceLock`.

**Exact declaration**

```cpp
~SequenceLock()
```

- **Declared at:** [line 18](../../../runtime/sequence_lock.hpp#L18)
- **Kind:** `destructor`

**Use it when.** You need the sequence lock module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_lock.hpp"

// `epok::SequenceLock` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the sequence lock module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
