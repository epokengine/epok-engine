# Epok API: Baked Selection

> **Header:** `"baked_selection.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/baked_selection.hpp)

This module covers the baked selection module. It documents 1 public callable declared directly in this header.

## Declared types

`epok::BakedSelectionCache`, `epok::BakedSelectionCache::Entry`

## Callable index

- [`epok::BakedSelectionCache::get`](#epok-bakedselectioncache-get-1) — Returns get as part of the baked selection module.

<a id="epok-bakedselectioncache-get-1"></a>

## `epok::BakedSelectionCache::get`

**Purpose.** Returns get as part of the baked selection module.

**Exact declaration**

```cpp
const Entry& get(const void* key,size_t slot,const uint32_t* mask, const BakedDrawFace* faces,size_t count)
```

- **Declared at:** [line 16](../../../runtime/baked_selection.hpp#L16)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `key` | `const void *` | Input | Value supplied for `key`. See the exact type and module contract. |
| `slot` | `size_t` | Input | Value supplied for `slot`. See the exact type and module contract. |
| `mask` | `const uint32_t *` | Input | Value supplied for `mask`. See the exact type and module contract. |
| `faces` | `const BakedDrawFace *` | Input | Value supplied for `faces`. See the exact type and module contract. |
| `count` | `size_t` | Input | Value supplied for `count`. See the exact type and module contract. |

**Returns.** Returns `const Entry &`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked selection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_selection.hpp"

// Assume these named values have been initialized with valid data:
// const void * key
// size_t slot
// const uint32_t * mask
// const BakedDrawFace * faces
// size_t count

epok::BakedSelectionCache& object = /* obtain a valid instance */;

auto result = object.get(key, slot, mask, faces, count);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked selection module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
