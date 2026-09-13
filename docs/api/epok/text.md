# Epok API: Text

> **Header:** `"text.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/text.hpp)

This module covers native text data and HUD text components. It documents 1 public callable declared directly in this header.

## Declared types

`epok::HudStats`, `epok::Text`

## Callable index

- [`epok::Text::set_text`](#epok-text-set-text-1) — Sets text as part of native text data and HUD text components.

<a id="epok-text-set-text-1"></a>

## `epok::Text::set_text`

**Purpose.** Sets text as part of native text data and HUD text components.

**Exact declaration**

```cpp
void set_text(const char* text)
```

- **Declared at:** [line 11](../../../runtime/text.hpp#L11)
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
