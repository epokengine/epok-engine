# PsyQo API: Gte Math

> **Header:** `"psyqo/gte-math.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh)

This module covers Geometry Transformation Engine math and register operations. It documents 11 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Callable index

- [`psyqo::GteMath::crossProductVec3`](#psyqo-gtemath-crossproductvec3-1) — Cross product of two vectors.
- [`psyqo::GteMath::cubic`](#psyqo-gtemath-cubic-1) — Evaluates a cubic Bezier at t.
- [`psyqo::GteMath::cubicDerivative`](#psyqo-gtemath-cubicderivative-1) — Evaluates the derivative of a cubic Bezier at t.
- [`psyqo::GteMath::fastNormalizeVec3`](#psyqo-gtemath-fastnormalizevec3-1) — Normalises a vector in place, without the exact square root.
- [`psyqo::GteMath::fromColumns`](#psyqo-gtemath-fromcolumns-1) — Builds a matrix from three column vectors.
- [`psyqo::GteMath::inverseSquareRootSeed`](#psyqo-gtemath-inversesquarerootseed-1) — The seed for an inverse square root, from the GTE's leading-bit count.
- [`psyqo::GteMath::matrixVecMul3`](#psyqo-gtemath-matrixvecmul3-1) — Multiplies a vector by a matrix.
- [`psyqo::GteMath::matrixVecMul3xy`](#psyqo-gtemath-matrixvecmul3xy-1) — Multiplies a vector by a matrix, keeping only x and y. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.
- [`psyqo::GteMath::matrixVecMul3z`](#psyqo-gtemath-matrixvecmul3z-1) — Multiplies a vector by a matrix, keeping only z. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.
- [`psyqo::GteMath::multiplyMatrix33`](#psyqo-gtemath-multiplymatrix33-1) — Multiplies two matrices.
- [`psyqo::GteMath::normalizeVec3`](#psyqo-gtemath-normalizevec3-1) — Normalises a vector in place.

<a id="psyqo-gtemath-crossproductvec3-1"></a>

## `psyqo::GteMath::crossProductVec3`

**Purpose.** Cross product of two vectors.

**Details.** 2.56x on hardware, but read the clobber list before reaching for it. CLOBBERS: R11, R22 and R33 - the ROTATION MATRIX DIAGONAL - plus IR1-3 and MAC1-3. The GTE's cross product takes its first operand from those three matrix elements rather than from a vector register, so calling this destroys a loaded rotation matrix. That makes it unsafe inside a projection loop unless you reload RT afterwards. It is free at load time, when nothing owns RT yet.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE void crossProductVec3(const Vec3 &v1, const Vec3 &v2, Vec3 *out)
```

- **Declared at:** [line 191](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L191)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v1` | `const Vec3 &` | Input | Value supplied for `v1`. See the exact type and module contract. |
| `v2` | `const Vec3 &` | Input | Value supplied for `v2`. See the exact type and module contract. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** 2.56x on hardware, but read the clobber list before reaching for it. CLOBBERS: R11, R22 and R33 - the ROTATION MATRIX DIAGONAL - plus IR1-3 and MAC1-3. The GTE's cross product takes its first operand from those three matrix elements rather than from a vector register, so calling this destroys a loaded rotation matrix. That makes it unsafe inside a projection loop unless you reload RT afterwards. It is free at load time, when nothing owns RT yet.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & v1
// const Vec3 & v2
// Vec3 * out

psyqo::GteMath::crossProductVec3(v1, v2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-cubic-1"></a>

## `psyqo::GteMath::cubic`

**Purpose.** Evaluates a cubic Bezier at t.

**Details.** A cubic Bezier is a weighted sum of four control points, and MVMVA computes Mx * Vx + Tx. Load three control points as the matrix COLUMNS and the Bernstein weights as the vector, and the fourth term rides in the translation register. One GTE op plus three multiplies, against twelve multiplies in software. This is the operation that justifies the header on its own: a curve sampled once per frame is how you get smooth camera motion along a 3D path, and that is a realtime cost rather than a load-time one. Verified against psyqo::Bezier::cubic: worst component deviation 3 raw out of 4096. CLOBBERS: the rotation matrix (RT), the translation vector (TR), V0, IR1-3, MAC1-3.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE Vec3 cubic(const Vec3 &a, const Vec3 &b, const Vec3 &c, const Vec3 &d, FixedPoint<> t)
```

- **Declared at:** [line 277](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L277)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vec3 &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vec3 &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Vec3 &` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `const Vec3 &` | Input | Value supplied for `d`. See the exact type and module contract. |
| `t` | `FixedPoint<>` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `Vec3`. Check the purpose and failure notes before using the value.

**Use it when.** A cubic Bezier is a weighted sum of four control points, and MVMVA computes Mx * Vx + Tx. Load three control points as the matrix COLUMNS and the Bernstein weights as the vector, and the fourth term rides in the translation register. One GTE op plus three multiplies, against twelve multiplies in software. This is the operation that justifies the header on its own: a curve sampled once per frame is how you get smooth camera motion along a 3D path, and that is a realtime cost rather than a load-time one. Verified against psyqo::Bezier::cubic: worst component deviation 3 raw out of 4096. CLOBBERS: the rotation matrix (RT), the translation vector (TR), V0, IR1-3, MAC1-3.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & a
// const Vec3 & b
// const Vec3 & c
// const Vec3 & d
// FixedPoint<> t

auto result = psyqo::GteMath::cubic(a, b, c, d, t);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-cubicderivative-1"></a>

## `psyqo::GteMath::cubicDerivative`

**Purpose.** Evaluates the derivative of a cubic Bezier at t.

**Details.** The derivative is a quadratic Bezier over the differences of consecutive control points, so it is only THREE terms and needs no translation vector at all - one MVMVA and nothing else. The factor of three that belongs in a true derivative is dropped. The usual consumer is a tangent that gets crossed and normalised, where only the direction survives; scale it yourself if you need the real magnitude. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3. Not TR.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE Vec3 cubicDerivative(const Vec3 &a, const Vec3 &b, const Vec3 &c, const Vec3 &d, FixedPoint<> t)
```

- **Declared at:** [line 303](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L303)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vec3 &` | Input | Value supplied for `a`. See the exact type and module contract. |
| `b` | `const Vec3 &` | Input | Value supplied for `b`. See the exact type and module contract. |
| `c` | `const Vec3 &` | Input | Value supplied for `c`. See the exact type and module contract. |
| `d` | `const Vec3 &` | Input | Value supplied for `d`. See the exact type and module contract. |
| `t` | `FixedPoint<>` | Input | Value supplied for `t`. See the exact type and module contract. |

**Returns.** Returns `Vec3`. Check the purpose and failure notes before using the value.

**Use it when.** The derivative is a quadratic Bezier over the differences of consecutive control points, so it is only THREE terms and needs no translation vector at all - one MVMVA and nothing else. The factor of three that belongs in a true derivative is dropped. The usual consumer is a tangent that gets crossed and normalised, where only the direction survives; scale it yourself if you need the real magnitude. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3. Not TR.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & a
// const Vec3 & b
// const Vec3 & c
// const Vec3 & d
// FixedPoint<> t

auto result = psyqo::GteMath::cubicDerivative(a, b, c, d, t);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-fastnormalizevec3-1"></a>

## `psyqo::GteMath::fastNormalizeVec3`

**Purpose.** Normalises a vector in place, without the exact square root.

**Details.** Same as normalizeVec3 here. SoftMath draws a distinction between an exact normalize and a fast one; on this path there is no reason for two, and the "fast" one in SoftMath seeds inverseSquareRoot with x * 2, which moves the wrong way as the vector shrinks. CLOBBERS: LZCS and LZCR only.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE void fastNormalizeVec3(Vec3 *v)
```

- **Declared at:** [line 256](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L256)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `v`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Same as normalizeVec3 here. SoftMath draws a distinction between an exact normalize and a fast one; on this path there is no reason for two, and the "fast" one in SoftMath seeds inverseSquareRoot with x * 2, which moves the wrong way as the vector shrinks. CLOBBERS: LZCS and LZCR only.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// Vec3 * v

psyqo::GteMath::fastNormalizeVec3(v);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-fromcolumns-1"></a>

## `psyqo::GteMath::fromColumns`

**Purpose.** Builds a matrix from three column vectors.

**Details.** Matrix33 stores ROWS. Several GTE formulations want the matrix built from columns - a weighted sum of three vectors is one MVMVA only if those vectors are the columns - so this scatters each one across the three rows. Getting it backwards produces plausible-looking garbage, which is why it is a named function rather than an open-coded initialiser at each site.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE Matrix33 fromColumns(const Vec3 &c0, const Vec3 &c1, const Vec3 &c2)
```

- **Declared at:** [line 141](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L141)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c0` | `const Vec3 &` | Input | Value supplied for `c0`. See the exact type and module contract. |
| `c1` | `const Vec3 &` | Input | Value supplied for `c1`. See the exact type and module contract. |
| `c2` | `const Vec3 &` | Input | Value supplied for `c2`. See the exact type and module contract. |

**Returns.** Returns `Matrix33`. Check the purpose and failure notes before using the value.

**Use it when.** Matrix33 stores ROWS. Several GTE formulations want the matrix built from columns - a weighted sum of three vectors is one MVMVA only if those vectors are the columns - so this scatters each one across the three rows. Getting it backwards produces plausible-looking garbage, which is why it is a named function rather than an open-coded initialiser at each site.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & c0
// const Vec3 & c1
// const Vec3 & c2

auto result = psyqo::GteMath::fromColumns(c0, c1, c2);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-inversesquarerootseed-1"></a>

## `psyqo::GteMath::inverseSquareRootSeed`

**Purpose.** The seed for an inverse square root, from the GTE's leading-bit count.

**Details.** The seed wants HALF the exponent, since the target is 1/sqrt(x) and not 1/x. With lzcr = 31 - floor(log2(x.raw())), that is (5 + lzcr) / 2 in 20.12. Shifting by the whole count is a seed for the wrong function: it is correct only where x == 0.0625 and diverges either side, and because SoftMath::inverseSquareRoot is four Newton steps with no convergence check, a seed outside the basin explodes rather than degrading. Swept over all 4096 representable values below 1.0, the halved exponent has a worst relative error of 0.48% and no failures; the unhalved one breaks 199 of them. CLOBBERS: LZCS and LZCR only. Safe to call with a matrix loaded. NOTE: LZCS/LZCR do not interlock the way the cop2 commands do - they are the one corner of the GTE where the hardware will not stall for you - so the write must be Safe.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE FixedPoint<> inverseSquareRootSeed(FixedPoint<> x)
```

- **Declared at:** [line 222](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L222)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `x` | `FixedPoint<>` | Input | Value supplied for `x`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<>`. Check the purpose and failure notes before using the value.

**Use it when.** The seed wants HALF the exponent, since the target is 1/sqrt(x) and not 1/x. With lzcr = 31 - floor(log2(x.raw())), that is (5 + lzcr) / 2 in 20.12. Shifting by the whole count is a seed for the wrong function: it is correct only where x == 0.0625 and diverges either side, and because SoftMath::inverseSquareRoot is four Newton steps with no convergence check, a seed outside the basin explodes rather than degrading. Swept over all 4096 representable values below 1.0, the halved exponent has a worst relative error of 0.48% and no failures; the unhalved one breaks 199 of them. CLOBBERS: LZCS and LZCR only. Safe to call with a matrix loaded. NOTE: LZCS/LZCR do not interlock the way the cop2 commands do - they are the one corner of the GTE where the hardware will not stall for you - so the write must be Safe.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// FixedPoint<> x

auto result = psyqo::GteMath::inverseSquareRootSeed(x);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations.

<a id="psyqo-gtemath-matrixvecmul3-1"></a>

## `psyqo::GteMath::matrixVecMul3`

**Purpose.** Multiplies a vector by a matrix.

**Details.** Drop-in for SoftMath::matrixVecMul3. 2.15x on hardware even paying the matrix upload here. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE void matrixVecMul3(const Matrix33 &m, const Vec3 &v, Vec3 *out)
```

- **Declared at:** [line 99](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L99)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 &` | Input | Value supplied for `v`. See the exact type and module contract. |
| `out` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Drop-in for SoftMath::matrixVecMul3. 2.15x on hardware even paying the matrix upload here. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v
// Vec3 * out

psyqo::GteMath::matrixVecMul3(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-matrixvecmul3xy-1"></a>

## `psyqo::GteMath::matrixVecMul3xy`

**Purpose.** Multiplies a vector by a matrix, keeping only x and y. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE void matrixVecMul3xy(const Matrix33 &m, const Vec3 &v, Vec2 *out)
```

- **Declared at:** [line 111](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L111)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 &` | Input | Value supplied for `v`. See the exact type and module contract. |
| `out` | `Vec2 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v
// Vec2 * out

psyqo::GteMath::matrixVecMul3xy(m, v, out);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-matrixvecmul3z-1"></a>

## `psyqo::GteMath::matrixVecMul3z`

**Purpose.** Multiplies a vector by a matrix, keeping only z. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE FixedPoint<> matrixVecMul3z(const Matrix33 &m, const Vec3 &v)
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L125)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m` | `const Matrix33 &` | Input | Value supplied for `m`. See the exact type and module contract. |
| `v` | `const Vec3 &` | Input | Value supplied for `v`. See the exact type and module contract. |

**Returns.** Returns `FixedPoint<>`. Check the purpose and failure notes before using the value.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m
// const Vec3 & v

auto result = psyqo::GteMath::matrixVecMul3z(m, v);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-multiplymatrix33-1"></a>

## `psyqo::GteMath::multiplyMatrix33`

**Purpose.** Multiplies two matrices.

**Details.** out = m2 * m1, matching SoftMath::multiplyMatrix33's convention. Column j of the product is m2 applied to column j of m1, so this is three MVMVA against 27 fixed-point multiplies: 5.46x on hardware. Aliasing is fine: the result is accumulated before anything is written back, so `out` may be `&m1` or `&m2`. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Exact declaration**

```cpp
PSYQO_GTE_MATH_INLINE void multiplyMatrix33(const Matrix33 &m1, const Matrix33 &m2, Matrix33 *out)
```

- **Declared at:** [line 161](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L161)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `m1` | `const Matrix33 &` | Input | Value supplied for `m1`. See the exact type and module contract. |
| `m2` | `const Matrix33 &` | Input | Value supplied for `m2`. See the exact type and module contract. |
| `out` | `Matrix33 *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** out = m2 * m1, matching SoftMath::multiplyMatrix33's convention. Column j of the product is m2 applied to column j of m1, so this is three MVMVA against 27 fixed-point multiplies: 5.46x on hardware. Aliasing is fine: the result is accumulated before anything is written back, so `out` may be `&m1` or `&m2`. CLOBBERS: the rotation matrix (RT), V0, IR1-3, MAC1-3.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// const Matrix33 & m1
// const Matrix33 & m2
// Matrix33 * out

psyqo::GteMath::multiplyMatrix33(m1, m2, out);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-gtemath-normalizevec3-1"></a>

## `psyqo::GteMath::normalizeVec3`

**Purpose.** Normalises a vector in place.

**Details.** Drop-in for SoftMath::normalizeVec3, which reaches 1/sqrt through squareRoot's shift-subtract loop at about 3588 cycles a vector. This keeps the squared length on the CPU - three multiplies - and takes only the seed from the GTE, then uses the existing Newton refinement. Measured against the exact version over 800 vectors, worst component deviation is 10 raw out of 4096, i.e. 0.24%. A zero-length vector is left as (0, 0, 1) rather than dividing by zero: SoftMath::squareRoot returns 0 for x.raw() <= 1 and normalizeVec3 then divides by it. CLOBBERS: LZCS and LZCR only.

**Exact declaration**

```cpp
void normalizeVec3(Vec3 *v)
```

- **Declared at:** [line 244](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-math.hh#L244)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `v` | `Vec3 *` | Input/output; inspect the function contract | Value supplied for `v`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Drop-in for SoftMath::normalizeVec3, which reaches 1/sqrt through squareRoot's shift-subtract loop at about 3588 cycles a vector. This keeps the squared length on the CPU - three multiplies - and takes only the seed from the GTE, then uses the existing Newton refinement. Measured against the exact version over 800 vectors, worst component deviation is 10 raw out of 4096, i.e. 0.24%. A zero-length vector is left as (0, 0, 1) rather than dividing by zero: SoftMath::squareRoot returns 0 for x.raw() <= 1 and normalizeVec3 then divides by it. CLOBBERS: LZCS and LZCR only.

**Usage pattern**

```cpp
#include "psyqo/gte-math.hh"

// Assume these named values have been initialized with valid data:
// Vec3 * v

psyqo::GteMath::normalizeVec3(v);
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
