# PsyQo API: Strings Helpers

> **Header:** `"psyqo/strings-helpers.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/strings-helpers.hh)

This module covers the strings helpers module. It documents 1 public callable declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Callable index

- [`psyqo::startsWith`](#psyqo-startswith-1) — Starts s with as part of the strings helpers module.

<a id="psyqo-startswith-1"></a>

## `psyqo::startsWith`

**Purpose.** Starts s with as part of the strings helpers module.

**Exact declaration**

```cpp
static inline bool startsWith(const eastl::string_view &s1, const eastl::string_view &s2)
```

- **Declared at:** [line 34](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/strings-helpers.hh#L34)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `s1` | `const eastl::string_view &` | Input | Value supplied for `s1`. See the exact type and module contract. |
| `s2` | `const eastl::string_view &` | Input | Value supplied for `s2`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the strings helpers module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/strings-helpers.hh"

// Assume these named values have been initialized with valid data:
// const eastl::string_view & s1
// const eastl::string_view & s2

auto result = psyqo::startsWith(s1, s2);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
