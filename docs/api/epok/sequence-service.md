# Epok API: Sequence Service

> **Header:** `"sequence_service.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/sequence_service.hpp)

This module covers the sequence service module. It documents 1 public callable declared directly in this header.

## Declared types

`epok::(unnamed struct at runtime/sequence_service.hpp:18:8)`, `epok::(unnamed struct at runtime/sequence_service.hpp:24:8)`, `epok::psx_audio::(unnamed enum at runtime/sequence_service.hpp:28:12)`, `epok::psx_audio::(unnamed struct at runtime/sequence_service.hpp:29:8)`, `epok::psx_audio::(unnamed struct at runtime/sequence_service.hpp:31:8)`, `epok::psx_audio::(unnamed struct at runtime/sequence_service.hpp:81:8)`

## Callable index

- [`epok::psx_audio::next_envelope`](#epok-psx-audio-next-envelope-1) — Performs `next envelope` as part of the sequence service module.

<a id="epok-psx-audio-next-envelope-1"></a>

## `epok::psx_audio::next_envelope`

**Purpose.** Performs `next envelope` as part of the sequence service module.

**Exact declaration**

```cpp
inline void next_envelope(Physical& v)
```

- **Declared at:** [line 202](../../../runtime/sequence_service.hpp#L202)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Physical &` | Input/output; inspect the function contract | Value supplied for `v`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the sequence service module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "sequence_service.hpp"

// Assume these named values have been initialized with valid data:
// Physical & v

epok::psx_audio::next_envelope(v);
```

**Why choose it.** The API exposes the hardware service without hiding latency or bounded memory.

**Trade-offs and warnings.** Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
