# PsyQo API: Soft Math

> **Header:** `"psyqo/soft-math.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh)

This module covers the soft math module. It documents 30 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::SoftMath::Axis`

## Callable index

- [`psyqo::SoftMath::crossProductVec3`](#psyqo-softmath-crossproductvec3-1) — Compute the cross product of two 3D vectors.
- [`psyqo::SoftMath::crossProductVec3`](#psyqo-softmath-crossproductvec3-2) — Performs `cross product vec3` as part of the soft math module.
- [`psyqo::SoftMath::crossProductVec3`](#psyqo-softmath-crossproductvec3-3) — Performs `cross product vec3` as part of the soft math module.
- [`psyqo::SoftMath::crossProductVec3`](#psyqo-softmath-crossproductvec3-4) — Compute the cross product of two 3D vectors.
- [`psyqo::SoftMath::fastNormalizeVec3`](#psyqo-softmath-fastnormalizevec3-1) — Normalizes a 3D vector, using a faster but less accurate algorithm.
- [`psyqo::SoftMath::generateRotationMatrix33`](#psyqo-softmath-generaterotationmatrix33-1) — Generate a rotation matrix for a given angle and axis.
- [`psyqo::SoftMath::generateRotationMatrix33`](#psyqo-softmath-generaterotationmatrix33-2) — Performs `generate rotation matrix33` as part of the soft math module.
- [`psyqo::SoftMath::generateRotationMatrix33`](#psyqo-softmath-generaterotationmatrix33-3) — Performs `generate rotation matrix33` as part of the soft math module.
- [`psyqo::SoftMath::generateRotationMatrix33`](#psyqo-softmath-generaterotationmatrix33-4) — Generate a rotation matrix for a given angle and axis.
- [`psyqo::SoftMath::inverseSquareRoot`](#psyqo-softmath-inversesquareroot-1) — Computes the inverse square root of a fixed point number, given an approximative hint.
- [`psyqo::SoftMath::inverseSquareRoot`](#psyqo-softmath-inversesquareroot-2) — Computes the inverse square root of a fixed point number.
- [`psyqo::SoftMath::matrixDeterminant3`](#psyqo-softmath-matrixdeterminant3-1) — Compute the determinant of a 3x3 matrix.
- [`psyqo::SoftMath::matrixDeterminant3`](#psyqo-softmath-matrixdeterminant3-2) — Performs `matrix determinant3` as part of the soft math module.
- [`psyqo::SoftMath::matrixVecMul3`](#psyqo-softmath-matrixvecmul3-1) — Performs `matrix vec mul3` as part of the soft math module.
- [`psyqo::SoftMath::matrixVecMul3`](#psyqo-softmath-matrixvecmul3-2) — Multiply a 3x3 matrix by a 3D vector.
- [`psyqo::SoftMath::matrixVecMul3xy`](#psyqo-softmath-matrixvecmul3xy-1) — Performs `matrix vec mul3xy` as part of the soft math module.
- [`psyqo::SoftMath::matrixVecMul3xy`](#psyqo-softmath-matrixvecmul3xy-2) — Multiply a 3x3 matrix by a 3D vector, returning only the x and y components.
- [`psyqo::SoftMath::matrixVecMul3z`](#psyqo-softmath-matrixvecmul3z-1) — Multiply a 3x3 matrix by a 3D vector, returning only the z component.
- [`psyqo::SoftMath::matrixVecMul3z`](#psyqo-softmath-matrixvecmul3z-2) — Performs `matrix vec mul3z` as part of the soft math module.
- [`psyqo::SoftMath::multiplyMatrix33`](#psyqo-softmath-multiplymatrix33-1) — Multiply two 3x3 matrices.
- [`psyqo::SoftMath::multiplyMatrix33`](#psyqo-softmath-multiplymatrix33-2) — Performs `multiply matrix33` as part of the soft math module.
- [`psyqo::SoftMath::multiplyMatrix33`](#psyqo-softmath-multiplymatrix33-3) — Performs `multiply matrix33` as part of the soft math module.
- [`psyqo::SoftMath::multiplyMatrix33`](#psyqo-softmath-multiplymatrix33-4) — Multiply two 3x3 matrices.
- [`psyqo::SoftMath::normalizeVec3`](#psyqo-softmath-normalizevec3-1) — Normalizes a 3D vector.
- [`psyqo::SoftMath::normOfVec3`](#psyqo-softmath-normofvec3-1) — Computes the norm of a 3D vector.
- [`psyqo::SoftMath::normOfVec3`](#psyqo-softmath-normofvec3-2) — Performs `norm of vec3` as part of the soft math module.
- [`psyqo::SoftMath::project`](#psyqo-softmath-project-1) — Projects a 3D point onto a 2D plane.
- [`psyqo::SoftMath::scaleMatrix33`](#psyqo-softmath-scalematrix33-1) — Scale a 3x3 matrix by a scalar.
- [`psyqo::SoftMath::squareRoot`](#psyqo-softmath-squareroot-1) — Computes the square root of a fixed point number, given an approximative hint.
- [`psyqo::SoftMath::squareRoot`](#psyqo-softmath-squareroot-2) — Computes the square root of a fixed point number.

<a id="psyqo-softmath-crossproductvec3-1"></a>

## `psyqo::SoftMath::crossProductVec3`

**Purpose.** Compute the cross product of two 3D vectors.

**Exact declaration**

```cpp
Vec3 crossProductVec3(const Vec3 &v1, const Vec3 &v2)
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L160)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v1` | `const Vec3 &` | Input | The first vector. |
| `v2` | `const Vec3 &` | Input | The second vector. |

**Returns.** Vec3 The cross product.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & v1
// const Vec3 & v2

auto result = psyqo::SoftMath::crossProductVec3(v1, v2);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-crossproductvec3-2"></a>

## `psyqo::SoftMath::crossProductVec3`

**Purpose.** Performs `cross product vec3` as part of the soft math module.

**Exact declaration**

```cpp
static inline Vec3 crossProductVec3(const Vec3 *v1, const Vec3 *v2)
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L161)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v1` | `const Vec3 *` | Input | Value supplied for `v1`. See the exact type and module contract. |
| `v2` | `const Vec3 *` | Input | Value supplied for `v2`. See the exact type and module contract. |

**Returns.** Returns `Vec3`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 * v1
// const Vec3 * v2

auto result = psyqo::SoftMath::crossProductVec3(v1, v2);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-crossproductvec3-3"></a>

## `psyqo::SoftMath::crossProductVec3`

**Purpose.** Performs `cross product vec3` as part of the soft math module.

**Exact declaration**

```cpp
static inline void crossProductVec3(const Vec3 *v1, const Vec3 *v2, Vec3 *out)
```

- **Declared at:** [line 148](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L148)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v1` | `const Vec3 *` | Input | Value supplied for `v1`. See the exact type and module contract. |
| `v2` | `const Vec3 *` | Input | Value supplied for `v2`. See the exact type and module contract. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 * v1
// const Vec3 * v2
// Vec3 * out

psyqo::SoftMath::crossProductVec3(v1, v2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-crossproductvec3-4"></a>

## `psyqo::SoftMath::crossProductVec3`

**Purpose.** Compute the cross product of two 3D vectors.

**Exact declaration**

```cpp
void crossProductVec3(const Vec3 &v1, const Vec3 &v2, Vec3 *out)
```

- **Declared at:** [line 147](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L147)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v1` | `const Vec3 &` | Input | The first vector. |
| `v2` | `const Vec3 &` | Input | The second vector. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | The vector to store the result in. May be the same as v1 or v2. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & v1
// const Vec3 & v2
// Vec3 * out

psyqo::SoftMath::crossProductVec3(v1, v2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-fastnormalizevec3-1"></a>

## `psyqo::SoftMath::fastNormalizeVec3`

**Purpose.** Normalizes a 3D vector, using a faster but less accurate algorithm.

**Exact declaration**

```cpp
void fastNormalizeVec3(Vec3 *v)
```

- **Declared at:** [line 235](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L235)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vec3 *` | Input/output; inspect the function contract | The vector to normalize. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Vec3 * v

psyqo::SoftMath::fastNormalizeVec3(v);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-generaterotationmatrix33-1"></a>

## `psyqo::SoftMath::generateRotationMatrix33`

**Purpose.** Generate a rotation matrix for a given angle and axis.

**Exact declaration**

```cpp
Matrix33 generateRotationMatrix33(Angle t, Axis a, const Trig<> &trig)
```

- **Declared at:** [line 60](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `Angle` | Input | The angle to rotate by. |
| `a` | `Axis` | Input | The axis to rotate around. |
| `trig` | `const Trig<> &` | Input | A trigonometry object to use for sine and cosine calculations. |

**Returns.** Matrix33 The rotation matrix.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Angle t
// Axis a
// const Trig<> & trig

auto result = psyqo::SoftMath::generateRotationMatrix33(t, a, trig);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-generaterotationmatrix33-2"></a>

## `psyqo::SoftMath::generateRotationMatrix33`

**Purpose.** Performs `generate rotation matrix33` as part of the soft math module.

**Exact declaration**

```cpp
static inline Matrix33 generateRotationMatrix33(Angle t, Axis a, const Trig<> *trig)
```

- **Declared at:** [line 61](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L61)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `t` | `Angle` | Input | Value supplied for `t`. See the exact type and module contract. |
| `a` | `Axis` | Input | Value supplied for `a`. See the exact type and module contract. |
| `trig` | `const Trig<> *` | Input | Value supplied for `trig`. See the exact type and module contract. |

**Returns.** Returns `Matrix33`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Angle t
// Axis a
// const Trig<> * trig

auto result = psyqo::SoftMath::generateRotationMatrix33(t, a, trig);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-generaterotationmatrix33-3"></a>

## `psyqo::SoftMath::generateRotationMatrix33`

**Purpose.** Performs `generate rotation matrix33` as part of the soft math module.

**Exact declaration**

```cpp
static inline void generateRotationMatrix33(Matrix33 *m, Angle t, Axis a, const Trig<> *trig)
```

- **Declared at:** [line 47](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L47)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `Matrix33 *` | Input/output; inspect the function contract | Value supplied for `m`. See the exact type and module contract. |
| `t` | `Angle` | Input | Value supplied for `t`. See the exact type and module contract. |
| `a` | `Axis` | Input | Value supplied for `a`. See the exact type and module contract. |
| `trig` | `const Trig<> *` | Input | Value supplied for `trig`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Matrix33 * m
// Angle t
// Axis a
// const Trig<> * trig

psyqo::SoftMath::generateRotationMatrix33(m, t, a, trig);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-generaterotationmatrix33-4"></a>

## `psyqo::SoftMath::generateRotationMatrix33`

**Purpose.** Generate a rotation matrix for a given angle and axis.

**Exact declaration**

```cpp
void generateRotationMatrix33(Matrix33 *m, Angle t, Axis a, const Trig<> &trig)
```

- **Declared at:** [line 46](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L46)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `Matrix33 *` | Input/output; inspect the function contract | The matrix to store the result in. |
| `t` | `Angle` | Input | The angle to rotate by. |
| `a` | `Axis` | Input | The axis to rotate around. |
| `trig` | `const Trig<> &` | Input | A trigonometry object to use for sine and cosine calculations. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Matrix33 * m
// Angle t
// Axis a
// const Trig<> & trig

psyqo::SoftMath::generateRotationMatrix33(m, t, a, trig);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-inversesquareroot-1"></a>

## `psyqo::SoftMath::inverseSquareRoot`

**Purpose.** Computes the inverse square root of a fixed point number, given an approximative hint.

**Exact declaration**

```cpp
FixedPoint<> inverseSquareRoot(FixedPoint<> x, FixedPoint<> y)
```

- **Declared at:** [line 202](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L202)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `FixedPoint<>` | Input | The number to compute the inverse square root of. |
| `y` | `FixedPoint<>` | Input | The approximative hint of the result. |

**Returns.** psyqo::FixedPoint<> The inverse square root.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<> x
// FixedPoint<> y

auto result = psyqo::SoftMath::inverseSquareRoot(x, y);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-softmath-inversesquareroot-2"></a>

## `psyqo::SoftMath::inverseSquareRoot`

**Purpose.** Computes the inverse square root of a fixed point number.

**Exact declaration**

```cpp
static inline FixedPoint<> inverseSquareRoot(FixedPoint<> x)
```

- **Declared at:** [line 210](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L210)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `FixedPoint<>` | Input | The number to compute the inverse square root of. |

**Returns.** psyqo::FixedPoint<> The inverse square root.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<> x

auto result = psyqo::SoftMath::inverseSquareRoot(x);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-softmath-matrixdeterminant3-1"></a>

## `psyqo::SoftMath::matrixDeterminant3`

**Purpose.** Compute the determinant of a 3x3 matrix.

**Exact declaration**

```cpp
FixedPoint<> matrixDeterminant3(const Matrix33 &m)
```

- **Declared at:** [line 172](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L172)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | The matrix. |

**Returns.** FixedPoint<> The determinant.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m

auto result = psyqo::SoftMath::matrixDeterminant3(m);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixdeterminant3-2"></a>

## `psyqo::SoftMath::matrixDeterminant3`

**Purpose.** Performs `matrix determinant3` as part of the soft math module.

**Exact declaration**

```cpp
static inline FixedPoint<> matrixDeterminant3(const Matrix33 *m)
```

- **Declared at:** [line 173](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L173)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 *` | Input | Value supplied for `m`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m

auto result = psyqo::SoftMath::matrixDeterminant3(m);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3-1"></a>

## `psyqo::SoftMath::matrixVecMul3`

**Purpose.** Performs `matrix vec mul3` as part of the soft math module.

**Exact declaration**

```cpp
static inline void matrixVecMul3(const Matrix33 *m, const Vec3 *v, Vec3 *out)
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L109)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 *` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 *` | Input | Value supplied for `v`. See the exact type and module contract. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m
// const Vec3 * v
// Vec3 * out

psyqo::SoftMath::matrixVecMul3(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3-2"></a>

## `psyqo::SoftMath::matrixVecMul3`

**Purpose.** Multiply a 3x3 matrix by a 3D vector.

**Exact declaration**

```cpp
void matrixVecMul3(const Matrix33 &m, const Vec3 &v, Vec3 *out)
```

- **Declared at:** [line 108](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L108)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | The matrix. |
| `v` | `const Vec3 &` | Input | The vector. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | The vector to store the result in. May be the same as v. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v
// Vec3 * out

psyqo::SoftMath::matrixVecMul3(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3xy-1"></a>

## `psyqo::SoftMath::matrixVecMul3xy`

**Purpose.** Performs `matrix vec mul3xy` as part of the soft math module.

**Exact declaration**

```cpp
static inline void matrixVecMul3xy(const Matrix33 *m, const Vec3 *v, Vec2 *out)
```

- **Declared at:** [line 122](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L122)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 *` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 *` | Input | Value supplied for `v`. See the exact type and module contract. |
| `out` | `Vec2 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m
// const Vec3 * v
// Vec2 * out

psyqo::SoftMath::matrixVecMul3xy(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3xy-2"></a>

## `psyqo::SoftMath::matrixVecMul3xy`

**Purpose.** Multiply a 3x3 matrix by a 3D vector, returning only the x and y components.

**Exact declaration**

```cpp
void matrixVecMul3xy(const Matrix33 &m, const Vec3 &v, Vec2 *out)
```

- **Declared at:** [line 121](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L121)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | The matrix. |
| `v` | `const Vec3 &` | Input | The vector. |
| `out` | `Vec2 *` | Input/output; inspect the function contract | The vector to store the result in. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v
// Vec2 * out

psyqo::SoftMath::matrixVecMul3xy(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3z-1"></a>

## `psyqo::SoftMath::matrixVecMul3z`

**Purpose.** Multiply a 3x3 matrix by a 3D vector, returning only the z component.

**Exact declaration**

```cpp
FixedPoint<> matrixVecMul3z(const Matrix33 &m, const Vec3 &v)
```

- **Declared at:** [line 134](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L134)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | The matrix. |
| `v` | `const Vec3 &` | Input | The vector. |

**Returns.** FixedPoint<> The z component of the result.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v

auto result = psyqo::SoftMath::matrixVecMul3z(m, v);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-matrixvecmul3z-2"></a>

## `psyqo::SoftMath::matrixVecMul3z`

**Purpose.** Performs `matrix vec mul3z` as part of the soft math module.

**Exact declaration**

```cpp
static inline FixedPoint<> matrixVecMul3z(const Matrix33 *m, const Vec3 *v)
```

- **Declared at:** [line 135](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L135)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 *` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 *` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m
// const Vec3 * v

auto result = psyqo::SoftMath::matrixVecMul3z(m, v);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-multiplymatrix33-1"></a>

## `psyqo::SoftMath::multiplyMatrix33`

**Purpose.** Multiply two 3x3 matrices.

**Exact declaration**

```cpp
Matrix33 multiplyMatrix33(const Matrix33 &m1, const Matrix33 &m2)
```

- **Declared at:** [line 87](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L87)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m1` | `const Matrix33 &` | Input | The first matrix. |
| `m2` | `const Matrix33 &` | Input | The second matrix. |

**Returns.** Matrix33 The result of the multiplication.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m1
// const Matrix33 & m2

auto result = psyqo::SoftMath::multiplyMatrix33(m1, m2);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-multiplymatrix33-2"></a>

## `psyqo::SoftMath::multiplyMatrix33`

**Purpose.** Performs `multiply matrix33` as part of the soft math module.

**Exact declaration**

```cpp
static inline Matrix33 multiplyMatrix33(const Matrix33 *m1, const Matrix33 *m2)
```

- **Declared at:** [line 88](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L88)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m1` | `const Matrix33 *` | Input | Value supplied for `m1`. See the exact type and module contract. |
| `m2` | `const Matrix33 *` | Input | Value supplied for `m2`. See the exact type and module contract. |

**Returns.** Returns `Matrix33`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m1
// const Matrix33 * m2

auto result = psyqo::SoftMath::multiplyMatrix33(m1, m2);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-multiplymatrix33-3"></a>

## `psyqo::SoftMath::multiplyMatrix33`

**Purpose.** Performs `multiply matrix33` as part of the soft math module.

**Exact declaration**

```cpp
static inline void multiplyMatrix33(const Matrix33 *m1, const Matrix33 *m2, Matrix33 *out)
```

- **Declared at:** [line 74](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L74)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m1` | `const Matrix33 *` | Input | Value supplied for `m1`. See the exact type and module contract. |
| `m2` | `const Matrix33 *` | Input | Value supplied for `m2`. See the exact type and module contract. |
| `out` | `Matrix33 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 * m1
// const Matrix33 * m2
// Matrix33 * out

psyqo::SoftMath::multiplyMatrix33(m1, m2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-multiplymatrix33-4"></a>

## `psyqo::SoftMath::multiplyMatrix33`

**Purpose.** Multiply two 3x3 matrices.

**Exact declaration**

```cpp
void multiplyMatrix33(const Matrix33 &m1, const Matrix33 &m2, Matrix33 *out)
```

- **Declared at:** [line 73](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L73)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m1` | `const Matrix33 &` | Input | The first matrix. |
| `m2` | `const Matrix33 &` | Input | The second matrix. |
| `out` | `Matrix33 *` | Input/output; inspect the function contract | The matrix to store the result in. May be the same as m1 or m2. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m1
// const Matrix33 & m2
// Matrix33 * out

psyqo::SoftMath::multiplyMatrix33(m1, m2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-normalizevec3-1"></a>

## `psyqo::SoftMath::normalizeVec3`

**Purpose.** Normalizes a 3D vector.

**Exact declaration**

```cpp
void normalizeVec3(Vec3 *v)
```

- **Declared at:** [line 228](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L228)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vec3 *` | Input/output; inspect the function contract | The vector to normalize. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Vec3 * v

psyqo::SoftMath::normalizeVec3(v);
```

**Why choose it.** It provides direct, allocation-conscious access to the soft math module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-normofvec3-1"></a>

## `psyqo::SoftMath::normOfVec3`

**Purpose.** Computes the norm of a 3D vector.

**Exact declaration**

```cpp
FixedPoint<> normOfVec3(const Vec3 &v)
```

- **Declared at:** [line 218](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L218)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `const Vec3 &` | Input | The vector. |

**Returns.** psyqo::FixedPoint<> The norm.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & v

auto result = psyqo::SoftMath::normOfVec3(v);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-normofvec3-2"></a>

## `psyqo::SoftMath::normOfVec3`

**Purpose.** Performs `norm of vec3` as part of the soft math module.

**Exact declaration**

```cpp
static inline FixedPoint<> normOfVec3(const Vec3 *v)
```

- **Declared at:** [line 219](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L219)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `const Vec3 *` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<>`. Check the purpose and failure notes before using the value.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 * v

auto result = psyqo::SoftMath::normOfVec3(v);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-project-1"></a>

## `psyqo::SoftMath::project`

**Purpose.** Projects a 3D point onto a 2D plane.

**Exact declaration**

```cpp
void project(const Vec3 *v, FixedPoint<> h, Vec2 *out)
```

- **Declared at:** [line 244](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L244)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `const Vec3 *` | Input | The vector to project. |
| `h` | `FixedPoint<>` | Input | The height of the plane. |
| `out` | `Vec2 *` | Input/output; inspect the function contract | The vector to store the result in. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 * v
// FixedPoint<> h
// Vec2 * out

psyqo::SoftMath::project(v, h, out);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-scalematrix33-1"></a>

## `psyqo::SoftMath::scaleMatrix33`

**Purpose.** Scale a 3x3 matrix by a scalar.

**Exact declaration**

```cpp
void scaleMatrix33(Matrix33 *m, FixedPoint<> s)
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L99)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `Matrix33 *` | Input/output; inspect the function contract | The matrix to scale. |
| `s` | `FixedPoint<>` | Input | The scalar to scale by. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// Matrix33 * m
// FixedPoint<> s

psyqo::SoftMath::scaleMatrix33(m, s);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-softmath-squareroot-1"></a>

## `psyqo::SoftMath::squareRoot`

**Purpose.** Computes the square root of a fixed point number, given an approximative hint.

**Exact declaration**

```cpp
FixedPoint<> squareRoot(FixedPoint<> x, FixedPoint<> y)
```

- **Declared at:** [line 184](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L184)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `FixedPoint<>` | Input | The number to compute the square root of. |
| `y` | `FixedPoint<>` | Input | The approximative hint of the result. |

**Returns.** psyqo::FixedPoint<> The square root.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<> x
// FixedPoint<> y

auto result = psyqo::SoftMath::squareRoot(x, y);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-softmath-squareroot-2"></a>

## `psyqo::SoftMath::squareRoot`

**Purpose.** Computes the square root of a fixed point number.

**Exact declaration**

```cpp
static inline FixedPoint<> squareRoot(FixedPoint<> x)
```

- **Declared at:** [line 192](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/soft-math.hh#L192)
- **Kind:** `function decl`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `FixedPoint<>` | Input | The number to compute the square root of. |

**Returns.** psyqo::FixedPoint<> The square root.

**Use it when.** You need the soft math module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/soft-math.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<> x

auto result = psyqo::SoftMath::squareRoot(x);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.
