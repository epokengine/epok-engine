# PsyQo API: Internal / Gpu / Configuration

> **Header:** `"psyqo/internal/gpu/configuration.hh"` · **Tier:** PsyQo low-level API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh)

This module covers GPU setup, command submission and frame synchronization. It documents 5 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU::Configuration`

## Callable index

- [`psyqo::GPU::Configuration::set`](#psyqo-gpu-configuration-set-1) — Sets set as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::Configuration::set`](#psyqo-gpu-configuration-set-2) — Sets set as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::Configuration::set`](#psyqo-gpu-configuration-set-3) — Sets set as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::Configuration::set`](#psyqo-gpu-configuration-set-4) — Sets set as part of GPU setup, command submission and frame synchronization.
- [`psyqo::GPU::Configuration::set`](#psyqo-gpu-configuration-set-5) — Sets set as part of GPU setup, command submission and frame synchronization.

<a id="psyqo-gpu-configuration-set-1"></a>

## `psyqo::GPU::Configuration::set`

**Purpose.** Sets set as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
Configuration &set(ColorMode colorMode)
```

- **Declared at:** [line 67](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh#L67)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `colorMode` | `ColorMode` | Input | Value supplied for `colorMode`. See the exact type and module contract. |

**Returns.** Returns `Configuration &`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/internal/gpu/configuration.hh"

// Assume these named values have been initialized with valid data:
// ColorMode colorMode

psyqo::GPU::Configuration& object = /* obtain a valid instance */;

auto result = object.set(colorMode);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-gpu-configuration-set-2"></a>

## `psyqo::GPU::Configuration::set`

**Purpose.** Sets set as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
Configuration &set(Interlace interlace)
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `interlace` | `Interlace` | Input | Value supplied for `interlace`. See the exact type and module contract. |

**Returns.** Returns `Configuration &`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/internal/gpu/configuration.hh"

// Assume these named values have been initialized with valid data:
// Interlace interlace

psyqo::GPU::Configuration& object = /* obtain a valid instance */;

auto result = object.set(interlace);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-gpu-configuration-set-3"></a>

## `psyqo::GPU::Configuration::set`

**Purpose.** Sets set as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
Configuration &set(MiscSetting setting)
```

- **Declared at:** [line 83](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh#L83)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `setting` | `MiscSetting` | Input | Value supplied for `setting`. See the exact type and module contract. |

**Returns.** Returns `Configuration &`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/internal/gpu/configuration.hh"

// Assume these named values have been initialized with valid data:
// MiscSetting setting

psyqo::GPU::Configuration& object = /* obtain a valid instance */;

auto result = object.set(setting);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-gpu-configuration-set-4"></a>

## `psyqo::GPU::Configuration::set`

**Purpose.** Sets set as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
Configuration &set(Resolution resolution)
```

- **Declared at:** [line 30](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh#L30)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `resolution` | `Resolution` | Input | Value supplied for `resolution`. See the exact type and module contract. |

**Returns.** Returns `Configuration &`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/internal/gpu/configuration.hh"

// Assume these named values have been initialized with valid data:
// Resolution resolution

psyqo::GPU::Configuration& object = /* obtain a valid instance */;

auto result = object.set(resolution);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.

<a id="psyqo-gpu-configuration-set-5"></a>

## `psyqo::GPU::Configuration::set`

**Purpose.** Sets set as part of GPU setup, command submission and frame synchronization.

**Exact declaration**

```cpp
Configuration &set(VideoMode videoMode)
```

- **Declared at:** [line 53](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/internal/gpu/configuration.hh#L53)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `videoMode` | `VideoMode` | Input | Value supplied for `videoMode`. See the exact type and module contract. |

**Returns.** Returns `Configuration &`. Check the purpose and failure notes before using the value.

**Use it when.** You need GPU setup, command submission and frame synchronization and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/internal/gpu/configuration.hh"

// Assume these named values have been initialized with valid data:
// VideoMode videoMode

psyqo::GPU::Configuration& object = /* obtain a valid instance */;

auto result = object.set(videoMode);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call. This is classified as **PsyQo low-level API**. Prefer a higher-level Epok service unless you need this exact control.
