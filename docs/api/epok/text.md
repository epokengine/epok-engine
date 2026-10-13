# Epok API: Text

> **Header:** `"text.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/text.hpp)

This module covers native text data and HUD text components. It documents 2 public callables declared directly in this header.

## Declared types

`epok::HudStats`, `epok::Localization`, `epok::Text`, `epok::TextAlign`

## Callable index

- [`epok::Localization::get`](#epok-localization-get-1) — Returns get as part of native text data and HUD text components.
- [`epok::Text::set_text`](#epok-text-set-text-1) — Sets text as part of native text data and HUD text components.

<a id="epok-localization-get-1"></a>

## `epok::Localization::get`

**Purpose.** Returns get as part of native text data and HUD text components.

**Exact declaration**

```cpp
const char* get(int key) const
```

- **Declared at:** [line 34](../../../runtime/text.hpp#L34)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `key` | `int` | Input | Value supplied for `key`. See the exact type and module contract. |

**Returns.** Returns `const char *`. Check the purpose and failure notes before using the value.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "text.hpp"

// Assume these named values have been initialized with valid data:
// int key

epok::Localization& object = /* obtain a valid instance */;

auto result = object.get(key);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-text-set-text-1"></a>

## `epok::Text::set_text`

**Purpose.** Sets text as part of native text data and HUD text components.

**Exact declaration**

```cpp
void set_text(const char* text)
```

- **Declared at:** [line 21](../../../runtime/text.hpp#L21)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `text` | `const char *` | Input | Value supplied for `text`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need native text data and HUD text components and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "text.hpp"

// Assume these named values have been initialized with valid data:
// const char * text

epok::Text& object = /* obtain a valid instance */;

object.set_text(text);
```

**Why choose it.** It provides direct, allocation-conscious access to native text data and HUD text components. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
