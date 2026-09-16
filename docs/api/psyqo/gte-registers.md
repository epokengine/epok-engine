# PsyQo API: Gte Registers

> **Header:** `"psyqo/gte-registers.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh)

This module covers Geometry Transformation Engine math and register operations. It documents 33 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GTE::PackedVec3`, `psyqo::GTE::PseudoRegister`, `psyqo::GTE::Register`, `psyqo::GTE::Safety`

## Callable index

- [`psyqo::GTE::clear`](#psyqo-gte-clear-1) — Clear a GTE register.
- [`psyqo::GTE::PackedVec3::operator int`](#psyqo-gte-packedvec3-operator-int-1) — Performs `operator  int` as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::PackedVec3::PackedVec3`](#psyqo-gte-packedvec3-packedvec3-1) — Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::PackedVec3::PackedVec3`](#psyqo-gte-packedvec3-packedvec3-2) — Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::PackedVec3::PackedVec3`](#psyqo-gte-packedvec3-packedvec3-3) — Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::read`](#psyqo-gte-read-1) — Reads read as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::read`](#psyqo-gte-read-2) — Reads read as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::read`](#psyqo-gte-read-3) — Reads read as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::read`](#psyqo-gte-read-4) — Reads read as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::read`](#psyqo-gte-read-5) — Read a 32-bits value from a GTE register to memory.
- [`psyqo::GTE::readRaw`](#psyqo-gte-readraw-1) — Reads a 32-bits value from a GTE register.
- [`psyqo::GTE::readSafe`](#psyqo-gte-readsafe-1) — Reads safe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::readSafe`](#psyqo-gte-readsafe-2) — Reads a short vector from a GTE pseudo register, adding nops after the operation.
- [`psyqo::GTE::readUnsafe`](#psyqo-gte-readunsafe-1) — Reads unsafe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::readUnsafe`](#psyqo-gte-readunsafe-2) — Reads a short vector from a GTE pseudo register, without adding nops after the operation.
- [`psyqo::GTE::write`](#psyqo-gte-write-1) — Writes a 32-bits value to a GTE register from memory.
- [`psyqo::GTE::write`](#psyqo-gte-write-2) — Write a 32-bits value to a GTE register.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-1) — Writes safe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-2) — Writes safe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-3) — The following are template specializations for the various GTE registers.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-4) — Writes safe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-5) — Write a 3x3 matrix to a GTE pseudo register, adding nops after the operation.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-6) — Write a 3D vector to a GTE pseudo register, adding nops after the operation.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-7) — Writes a 16-bits fixed point number to a GTE register, adding nops after the operation.
- [`psyqo::GTE::writeSafe`](#psyqo-gte-writesafe-8) — Writes two 16-bits fixed point numbers to a GTE register, adding nops after the operation.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-1) — Writes unsafe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-2) — Writes unsafe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-3) — Writes unsafe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-4) — Writes unsafe as part of Geometry Transformation Engine math and register operations.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-5) — Write a 3x3 matrix to a GTE pseudo register, without adding nops after the operation.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-6) — Write a 3D vector to a GTE pseudo register, without adding nops after the operation.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-7) — Writes a 16-bits fixed point number to a GTE register, without adding nops after the operation.
- [`psyqo::GTE::writeUnsafe`](#psyqo-gte-writeunsafe-8) — Writes two 16-bits fixed point numbers to a GTE register, without adding nops after the operation.

<a id="psyqo-gte-clear-1"></a>

## `psyqo::GTE::clear`

**Purpose.** Clear a GTE register.

**Exact declaration**

```cpp
template <Register reg, Safety safety = Safe> static inline void clear()
```

- **Declared at:** [line 163](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L163)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, safety

psyqo::GTE::clear<reg, safety>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-packedvec3-operator-int-1"></a>

## `psyqo::GTE::PackedVec3::operator int`

**Purpose.** Performs `operator  int` as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
operator Vec3() const
```

- **Declared at:** [line 61](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L61)
- **Kind:** `conversion function`; qualifiers: `const`

**Returns.** Returns `Vec3`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

psyqo::GTE::PackedVec3& object = /* obtain a valid instance */;

auto result = object.operator int();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-packedvec3-packedvec3-1"></a>

## `psyqo::GTE::PackedVec3::PackedVec3`

**Purpose.** Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
PackedVec3() = default
```

- **Declared at:** [line 54](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L54)
- **Kind:** `constructor`

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

psyqo::GTE::PackedVec3 value();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-packedvec3-packedvec3-2"></a>

## `psyqo::GTE::PackedVec3::PackedVec3`

**Purpose.** Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
explicit PackedVec3(Short x_, Short y_, Short z_) : x
```

- **Declared at:** [line 55](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L55)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x_` | `Short` | Input | Value supplied for `x_`. See the exact type and module contract. |
| `y_` | `Short` | Input | Value supplied for `y_`. See the exact type and module contract. |
| `z_` | `Short` | Input | Value supplied for `z_`. See the exact type and module contract. |

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Short x_
// Short y_
// Short z_

psyqo::GTE::PackedVec3 value(x_, y_, z_);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-packedvec3-packedvec3-3"></a>

## `psyqo::GTE::PackedVec3::PackedVec3`

**Purpose.** Constructs `psyqo::GTE::PackedVec3` for Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
explicit PackedVec3(const Vec3& v)
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L56)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `const Vec3 &` | Input | Value supplied for `v`. See the exact type and module contract. |

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & v

psyqo::GTE::PackedVec3 value(v);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-read-1"></a>

## `psyqo::GTE::read`

**Purpose.** Reads read as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> [[deprecated("Use the reference version instead")]] inline void read<PseudoRegister::LV>(Vec3* ptr)
```

- **Declared at:** [line 967](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L967)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `ptr`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Vec3 * ptr

psyqo::GTE::read(ptr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-read-2"></a>

## `psyqo::GTE::read`

**Purpose.** Reads read as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void read<PseudoRegister::LV>(Vec3& ptr)
```

- **Declared at:** [line 974](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L974)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `Vec3 &` | Input/output; inspect the function contract | Value supplied for `ptr`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Vec3 & ptr

psyqo::GTE::read(ptr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-read-3"></a>

## `psyqo::GTE::read`

**Purpose.** Reads read as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> [[deprecated("Use the reference version instead")]] static inline void read(Vec3* ptr)
```

- **Declared at:** [line 440](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L440)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `ptr`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Vec3 * ptr

psyqo::GTE::read<reg, valid>(ptr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-read-4"></a>

## `psyqo::GTE::read`

**Purpose.** Reads read as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline void read(Vec3& vec)
```

- **Declared at:** [line 448](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L448)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `vec` | `Vec3 &` | Input/output; inspect the function contract | Value supplied for `vec`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Vec3 & vec

psyqo::GTE::read<reg, valid>(vec);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-read-5"></a>

## `psyqo::GTE::read`

**Purpose.** Read a 32-bits value from a GTE register to memory.

**Exact declaration**

```cpp
template <Register reg> static inline void read(uint32_t* ptr)
```

- **Declared at:** [line 396](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L396)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `int *` | Input/output; inspect the function contract | The pointer to the memory location to write to. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg

// Assume these named values have been initialized with valid data:
// int * ptr

psyqo::GTE::read<reg>(ptr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-readraw-1"></a>

## `psyqo::GTE::readRaw`

**Purpose.** Reads a 32-bits value from a GTE register.

**Exact declaration**

```cpp
template <Register reg, Safety safety = Safe> static inline uint32_t readRaw()
```

- **Declared at:** [line 367](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L367)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** uint32_t The value read from the register.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, safety

auto result = psyqo::GTE::readRaw<reg, safety>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-gte-readsafe-1"></a>

## `psyqo::GTE::readSafe`

**Purpose.** Reads safe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline PackedVec3 readSafe<PseudoRegister::SV>()
```

- **Declared at:** [line 939](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L939)
- **Kind:** `function decl`; qualifiers: `template`

**Returns.** Returns `PackedVec3`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

auto result = psyqo::GTE::readSafe();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-readsafe-2"></a>

## `psyqo::GTE::readSafe`

**Purpose.** Reads a short vector from a GTE pseudo register, adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline PackedVec3 readSafe()
```

- **Declared at:** [line 413](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L413)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** PackedVec3 The vector read from the pseudo register.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

auto result = psyqo::GTE::readSafe<reg, valid>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-readunsafe-1"></a>

## `psyqo::GTE::readUnsafe`

**Purpose.** Reads unsafe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline PackedVec3 readUnsafe<PseudoRegister::SV>()
```

- **Declared at:** [line 953](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L953)
- **Kind:** `function decl`; qualifiers: `template`

**Returns.** Returns `PackedVec3`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

auto result = psyqo::GTE::readUnsafe();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-readunsafe-2"></a>

## `psyqo::GTE::readUnsafe`

**Purpose.** Reads a short vector from a GTE pseudo register, without adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline PackedVec3 readUnsafe()
```

- **Declared at:** [line 430](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L430)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** PackedVec3 The vector read from the pseudo register.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

auto result = psyqo::GTE::readUnsafe<reg, valid>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-write-1"></a>

## `psyqo::GTE::write`

**Purpose.** Writes a 32-bits value to a GTE register from memory.

**Exact declaration**

```cpp
template <Register reg, Safety safety = Safe> static inline void write(const uint32_t* ptr)
```

- **Declared at:** [line 346](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L346)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `ptr` | `const int *` | Input | The pointer to the value to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, safety

// Assume these named values have been initialized with valid data:
// const int * ptr

psyqo::GTE::write<reg, safety>(ptr);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-write-2"></a>

## `psyqo::GTE::write`

**Purpose.** Write a 32-bits value to a GTE register.

**Exact declaration**

```cpp
template <Register reg, Safety safety = Safe> static inline void write(uint32_t value)
```

- **Declared at:** [line 184](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L184)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int` | Input | The value to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, safety

// Assume these named values have been initialized with valid data:
// int value

psyqo::GTE::write<reg, safety>(value);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writesafe-1"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Writes safe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeSafe<PseudoRegister::Rotation>(const Matrix33& in)
```

- **Declared at:** [line 823](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L823)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Matrix33 &` | Input | Value supplied for `in`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & in

psyqo::GTE::writeSafe(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writesafe-2"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Writes safe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeSafe<PseudoRegister::V0>(const Vec3& in)
```

- **Declared at:** [line 850](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L850)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Vec3 &` | Input | Value supplied for `in`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & in

psyqo::GTE::writeSafe(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writesafe-3"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** The following are template specializations for the various GTE registers.

**Exact declaration**

```cpp
template <> inline void writeSafe<Register::VXY0>(Short x_, Short y_)
```

- **Declared at:** [line 457](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L457)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x_` | `Short` | Input | Value supplied for `x_`. See the exact type and module contract. |
| `y_` | `Short` | Input | Value supplied for `y_`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Short x_
// Short y_

psyqo::GTE::writeSafe(x_, y_);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writesafe-4"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Writes safe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeSafe<Register::VZ0>(Short z_)
```

- **Declared at:** [line 464](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L464)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `z_` | `Short` | Input | Value supplied for `z_`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Short z_

psyqo::GTE::writeSafe(z_);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writesafe-5"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Write a 3x3 matrix to a GTE pseudo register, adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline void writeSafe(const Matrix33& in)
```

- **Declared at:** [line 274](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L274)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Matrix33 &` | Input | The matrix to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// const Matrix33 & in

psyqo::GTE::writeSafe<reg, valid>(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writesafe-6"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Write a 3D vector to a GTE pseudo register, adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline void writeSafe(const Vec3& in)
```

- **Declared at:** [line 298](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L298)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Vec3 &` | Input | The vector to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// const Vec3 & in

psyqo::GTE::writeSafe<reg, valid>(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writesafe-7"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Writes a 16-bits fixed point number to a GTE register, adding nops after the operation.

**Exact declaration**

```cpp
template <Register reg, bool valid = false> static inline void writeSafe(Short fp)
```

- **Declared at:** [line 209](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L209)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fp` | `Short` | Input | The fixed point number to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Short fp

psyqo::GTE::writeSafe<reg, valid>(fp);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writesafe-8"></a>

## `psyqo::GTE::writeSafe`

**Purpose.** Writes two 16-bits fixed point numbers to a GTE register, adding nops after the operation.

**Exact declaration**

```cpp
template <Register reg, bool valid = false> static inline void writeSafe(Short low, Short hi)
```

- **Declared at:** [line 233](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L233)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `low` | `Short` | Input | The value to write to the lower 16 bits of the register. |
| `hi` | `Short` | Input | The value to write to the higher 16 bits of the register. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Short low
// Short hi

psyqo::GTE::writeSafe<reg, valid>(low, hi);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writeunsafe-1"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes unsafe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeUnsafe<PseudoRegister::Rotation>(const Matrix33& in)
```

- **Declared at:** [line 881](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L881)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Matrix33 &` | Input | Value supplied for `in`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & in

psyqo::GTE::writeUnsafe(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writeunsafe-2"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes unsafe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeUnsafe<PseudoRegister::V0>(const Vec3& in)
```

- **Declared at:** [line 908](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L908)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Vec3 &` | Input | Value supplied for `in`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & in

psyqo::GTE::writeUnsafe(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writeunsafe-3"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes unsafe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeUnsafe<Register::VXY0>(Short x_, Short y_)
```

- **Declared at:** [line 640](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L640)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x_` | `Short` | Input | Value supplied for `x_`. See the exact type and module contract. |
| `y_` | `Short` | Input | Value supplied for `y_`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Short x_
// Short y_

psyqo::GTE::writeUnsafe(x_, y_);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writeunsafe-4"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes unsafe as part of Geometry Transformation Engine math and register operations.

**Exact declaration**

```cpp
template <> inline void writeUnsafe<Register::VZ0>(Short z_)
```

- **Declared at:** [line 647](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L647)
- **Kind:** `function decl`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `z_` | `Short` | Input | Value supplied for `z_`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Assume these named values have been initialized with valid data:
// Short z_

psyqo::GTE::writeUnsafe(z_);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writeunsafe-5"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Write a 3x3 matrix to a GTE pseudo register, without adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline void writeUnsafe(const Matrix33& in)
```

- **Declared at:** [line 286](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L286)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Matrix33 &` | Input | The matrix to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// const Matrix33 & in

psyqo::GTE::writeUnsafe<reg, valid>(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writeunsafe-6"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Write a 3D vector to a GTE pseudo register, without adding nops after the operation.

**Exact declaration**

```cpp
template <PseudoRegister reg, bool valid = false> static inline void writeUnsafe(const Vec3& in)
```

- **Declared at:** [line 322](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L322)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `in` | `const Vec3 &` | Input | The vector to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// const Vec3 & in

psyqo::GTE::writeUnsafe<reg, valid>(in);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gte-writeunsafe-7"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes a 16-bits fixed point number to a GTE register, without adding nops after the operation.

**Exact declaration**

```cpp
template <Register reg, bool valid = false> static inline void writeUnsafe(Short fp)
```

- **Declared at:** [line 221](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L221)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fp` | `Short` | Input | The fixed point number to write. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Short fp

psyqo::GTE::writeUnsafe<reg, valid>(fp);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-writeunsafe-8"></a>

## `psyqo::GTE::writeUnsafe`

**Purpose.** Writes two 16-bits fixed point numbers to a GTE register, without adding nops after the operation.

**Exact declaration**

```cpp
template <Register reg, bool valid = false> static inline void writeUnsafe(Short low, Short hi)
```

- **Declared at:** [line 246](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-registers.hh#L246)
- **Kind:** `function template`; qualifiers: `static, template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `low` | `Short` | Input | The value to write to the lower 16 bits of the register. |
| `hi` | `Short` | Input | The value to write to the higher 16 bits of the register. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-registers.hh"

// Replace these template arguments with types or values accepted by the declaration:
// reg, valid

// Assume these named values have been initialized with valid data:
// Short low
// Short hi

psyqo::GTE::writeUnsafe<reg, valid>(low, hi);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.
