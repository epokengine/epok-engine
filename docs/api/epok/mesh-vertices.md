# Epok API: Mesh Vertices

> **Header:** `"mesh_vertices.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/mesh_vertices.hpp)

This module covers the mesh vertices module. It documents 15 public callables declared directly in this header.

## Declared types

`epok::MeshVertex`, `epok::MeshVertices`

## Callable index

- [`epok::MeshVertex::operator[]`](#epok-meshvertex-operator-1) — Performs `operator []` as part of the mesh vertices module.
- [`epok::MeshVertices::bind`](#epok-meshvertices-bind-1) — Performs `bind` as part of the mesh vertices module.
- [`epok::MeshVertices::MeshVertices`](#epok-meshvertices-meshvertices-1) — Constructs `epok::MeshVertices` for the mesh vertices module.
- [`epok::MeshVertices::MeshVertices`](#epok-meshvertices-meshvertices-2) — Constructs `epok::MeshVertices` for the mesh vertices module.
- [`epok::MeshVertices::MeshVertices`](#epok-meshvertices-meshvertices-3) — Constructs `epok::MeshVertices` for the mesh vertices module.
- [`epok::MeshVertices::MeshVertices`](#epok-meshvertices-meshvertices-4) — Constructs `epok::MeshVertices` for the mesh vertices module.
- [`epok::MeshVertices::MeshVertices`](#epok-meshvertices-meshvertices-5) — Constructs `epok::MeshVertices` for the mesh vertices module.
- [`epok::MeshVertices::narrow_data`](#epok-meshvertices-narrow-data-1) — Performs `narrow data` as part of the mesh vertices module.
- [`epok::MeshVertices::operator bool`](#epok-meshvertices-operator-bool-1) — Performs `operator  bool` as part of the mesh vertices module.
- [`epok::MeshVertices::operator+`](#epok-meshvertices-operator-1) — Performs `operator +` as part of the mesh vertices module.
- [`epok::MeshVertices::operator==`](#epok-meshvertices-operator-2) — Performs `operator ==` as part of the mesh vertices module.
- [`epok::MeshVertices::operator[]`](#epok-meshvertices-operator-3) — Performs `operator []` as part of the mesh vertices module.
- [`epok::MeshVertices::valid_format`](#epok-meshvertices-valid-format-1) — Performs `valid format` as part of the mesh vertices module.
- [`epok::MeshVertices::wide`](#epok-meshvertices-wide-1) — Performs `wide` as part of the mesh vertices module.
- [`epok::MeshVertices::wide_data`](#epok-meshvertices-wide-data-1) — Performs `wide data` as part of the mesh vertices module.

<a id="epok-meshvertex-operator-1"></a>

## `epok::MeshVertex::operator[]`

**Purpose.** Performs `operator []` as part of the mesh vertices module.

**Exact declaration**

```cpp
int32_t operator[](size_t axis) const
```

- **Declared at:** [line 8](../../../runtime/mesh_vertices.hpp#L8)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `axis` | `size_t` | Input | Value supplied for `axis`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// size_t axis

epok::MeshVertex& object = /* obtain a valid instance */;

auto result = object.operator[](axis);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-bind-1"></a>

## `epok::MeshVertices::bind`

**Purpose.** Performs `bind` as part of the mesh vertices module.

**Exact declaration**

```cpp
MeshVertices bind(const void* p) const
```

- **Declared at:** [line 29](../../../runtime/mesh_vertices.hpp#L29)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const void *` | Input | Value supplied for `p`. See the exact type and module contract. |

**Returns.** Returns `MeshVertices`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// const void * p

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.bind(p);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshvertices-meshvertices-1"></a>

## `epok::MeshVertices::MeshVertices`

**Purpose.** Constructs `epok::MeshVertices` for the mesh vertices module.

**Exact declaration**

```cpp
constexpr MeshVertices() = default
```

- **Declared at:** [line 18](../../../runtime/mesh_vertices.hpp#L18)
- **Kind:** `constructor`

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices value();
```

**Why choose it.** It provides direct, allocation-conscious access to the mesh vertices module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-meshvertices-2"></a>

## `epok::MeshVertices::MeshVertices`

**Purpose.** Constructs `epok::MeshVertices` for the mesh vertices module.

**Exact declaration**

```cpp
constexpr MeshVertices(const int16_t (*p)[3]) : data(p)
```

- **Declared at:** [line 20](../../../runtime/mesh_vertices.hpp#L20)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int16_t (*)[3]` | Callback | Value supplied for `p`. See the exact type and module contract. |

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// const int16_t (*)[3] p

epok::MeshVertices value(p);
```

**Why choose it.** It provides direct, allocation-conscious access to the mesh vertices module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshvertices-meshvertices-3"></a>

## `epok::MeshVertices::MeshVertices`

**Purpose.** Constructs `epok::MeshVertices` for the mesh vertices module.

**Exact declaration**

```cpp
constexpr MeshVertices(const int32_t (*p)[3]) : data(p), stride(12)
```

- **Declared at:** [line 21](../../../runtime/mesh_vertices.hpp#L21)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const int32_t (*)[3]` | Callback | Value supplied for `p`. See the exact type and module contract. |

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t (*)[3] p

epok::MeshVertices value(p);
```

**Why choose it.** It provides direct, allocation-conscious access to the mesh vertices module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshvertices-meshvertices-4"></a>

## `epok::MeshVertices::MeshVertices`

**Purpose.** Constructs `epok::MeshVertices` for the mesh vertices module.

**Exact declaration**

```cpp
constexpr MeshVertices(const void* p, uint8_t bytes) : data(p), stride(bytes)
```

- **Declared at:** [line 22](../../../runtime/mesh_vertices.hpp#L22)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `p` | `const void *` | Input | Value supplied for `p`. See the exact type and module contract. |
| `bytes` | `uint8_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// const void * p
// uint8_t bytes

epok::MeshVertices value(p, bytes);
```

**Why choose it.** It provides direct, allocation-conscious access to the mesh vertices module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshvertices-meshvertices-5"></a>

## `epok::MeshVertices::MeshVertices`

**Purpose.** Constructs `epok::MeshVertices` for the mesh vertices module.

**Exact declaration**

```cpp
constexpr MeshVertices(decltype(nullptr))
```

- **Declared at:** [line 19](../../../runtime/mesh_vertices.hpp#L19)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `decltype(nullptr)` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// decltype(nullptr) arg1

epok::MeshVertices value(arg1);
```

**Why choose it.** It provides direct, allocation-conscious access to the mesh vertices module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-narrow-data-1"></a>

## `epok::MeshVertices::narrow_data`

**Purpose.** Performs `narrow data` as part of the mesh vertices module.

**Exact declaration**

```cpp
auto narrow_data() const -> const int16_t (*)[3]
```

- **Declared at:** [line 33](../../../runtime/mesh_vertices.hpp#L33)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const int16_t (*)[3]`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.narrow_data();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-operator-bool-1"></a>

## `epok::MeshVertices::operator bool`

**Purpose.** Performs `operator  bool` as part of the mesh vertices module.

**Exact declaration**

```cpp
explicit constexpr operator bool() const
```

- **Declared at:** [line 23](../../../runtime/mesh_vertices.hpp#L23)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.operator bool();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-meshvertices-operator-1"></a>

## `epok::MeshVertices::operator+`

**Purpose.** Performs `operator +` as part of the mesh vertices module.

**Exact declaration**

```cpp
MeshVertices operator+(size_t first) const
```

- **Declared at:** [line 30](../../../runtime/mesh_vertices.hpp#L30)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `first` | `size_t` | Input | Value supplied for `first`. See the exact type and module contract. |

**Returns.** Returns `MeshVertices`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// size_t first

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.operator+(first);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-operator-2"></a>

## `epok::MeshVertices::operator==`

**Purpose.** Performs `operator ==` as part of the mesh vertices module.

**Exact declaration**

```cpp
constexpr bool operator==(const MeshVertices& other) const
```

- **Declared at:** [line 26](../../../runtime/mesh_vertices.hpp#L26)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `other` | `const MeshVertices &` | Input | Value supplied for `other`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// const MeshVertices & other

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.operator==(other);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-meshvertices-operator-3"></a>

## `epok::MeshVertices::operator[]`

**Purpose.** Performs `operator []` as part of the mesh vertices module.

**Exact declaration**

```cpp
MeshVertex operator[](size_t index) const
```

- **Declared at:** [line 39](../../../runtime/mesh_vertices.hpp#L39)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `index` | `size_t` | Input | Value supplied for `index`. See the exact type and module contract. |

**Returns.** Returns `MeshVertex`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

// Assume these named values have been initialized with valid data:
// size_t index

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.operator[](index);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-meshvertices-valid-format-1"></a>

## `epok::MeshVertices::valid_format`

**Purpose.** Performs `valid format` as part of the mesh vertices module.

**Exact declaration**

```cpp
constexpr bool valid_format() const
```

- **Declared at:** [line 24](../../../runtime/mesh_vertices.hpp#L24)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.valid_format();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-meshvertices-wide-1"></a>

## `epok::MeshVertices::wide`

**Purpose.** Performs `wide` as part of the mesh vertices module.

**Exact declaration**

```cpp
constexpr bool wide() const
```

- **Declared at:** [line 25](../../../runtime/mesh_vertices.hpp#L25)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.wide();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-meshvertices-wide-data-1"></a>

## `epok::MeshVertices::wide_data`

**Purpose.** Performs `wide data` as part of the mesh vertices module.

**Exact declaration**

```cpp
auto wide_data() const -> const int32_t (*)[3]
```

- **Declared at:** [line 36](../../../runtime/mesh_vertices.hpp#L36)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `const int32_t (*)[3]`. Check the purpose and failure notes before using the value.

**Use it when.** You need the mesh vertices module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "mesh_vertices.hpp"

epok::MeshVertices& object = /* obtain a valid instance */;

auto result = object.wide_data();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
