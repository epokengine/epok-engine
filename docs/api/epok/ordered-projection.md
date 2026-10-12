# Epok API: Ordered Projection

> **Header:** `"ordered_projection.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/ordered_projection.hpp)

This module covers the ordered projection module. It documents 11 public callables declared directly in this header.

## Declared types

`epok::GteUnrTable`, `epok::OrderedScreenInside`

## Callable index

- [`epok::gte_divide`](#epok-gte-divide-1) — Performs `gte divide` as part of the ordered projection module.
- [`epok::gte_project_point`](#epok-gte-project-point-1) — Screen position exactly as RTPS/RTPT produce it from GTE MAC results (camera Y up, as ordered_gte_camera_reference returns it), or false where the GTE would flag the vertex: IR or SZ3 saturation, divide overflow, or SX/SY outside -0x400..0x3FF.
- [`epok::ordered_depth_average`](#epok-ordered-depth-average-1) — Exact signed division (truncate toward zero), without MIPS DIV.
- [`epok::ordered_gte_camera_reference`](#epok-ordered-gte-camera-reference-1) — Independent integer oracle for the MAC results of the loaded projection matrix.
- [`epok::ordered_gte_depth_average`](#epok-ordered-gte-depth-average-1) — Performs `ordered gte depth average` as part of the ordered projection module.
- [`epok::ordered_gte_input_fits`](#epok-ordered-gte-input-fits-1) — Q8 operands derived from signed Q12 each fit 28 bits.
- [`epok::ordered_projection_anchor`](#epok-ordered-projection-anchor-1) — Rebase Q8 operands on a 4096-Q8 (16-world-unit) lattice RELATIVE to the camera.
- [`epok::ordered_screen_inside`](#epok-ordered-screen-inside-1) — Performs `ordered screen inside` as part of the ordered projection module.
- [`epok::ordered_screen_inside_bits`](#epok-ordered-screen-inside-bits-1) — Performs `ordered screen inside bits` as part of the ordered projection module.
- [`epok::ordered_screen_outside`](#epok-ordered-screen-outside-1) — Trivial rejection on packed SXY words, the reference pass's test: true when every corner lies on the same outer side of the viewport, exactly when the corners' bounding box misses it.
- [`epok::project_ordered_cpu`](#epok-project-ordered-cpu-1) — Full-range projection is not synonymous with clipping.

<a id="epok-gte-divide-1"></a>

## `epok::gte_divide`

**Purpose.** Performs `gte divide` as part of the ordered projection module.

**Exact declaration**

```cpp
inline bool gte_divide(uint32_t h,uint32_t sz,uint32_t& quotient)
```

- **Declared at:** [line 127](../../../runtime/ordered_projection.hpp#L127)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `h` | `uint32_t` | Input | Value supplied for `h`. See the exact type and module contract. |
| `sz` | `uint32_t` | Input | Value supplied for `sz`. See the exact type and module contract. |
| `quotient` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `quotient`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered projection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t h
// uint32_t sz
// uint32_t & quotient

auto result = epok::gte_divide(h, sz, quotient);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-gte-project-point-1"></a>

## `epok::gte_project_point`

**Purpose.** Screen position exactly as RTPS/RTPT produce it from GTE MAC results (camera Y up, as ordered_gte_camera_reference returns it), or false where the GTE would flag the vertex: IR or SZ3 saturation, divide overflow, or SX/SY outside -0x400..0x3FF.

**Exact declaration**

```cpp
inline bool gte_project_point(const int32_t* camera,int32_t h,int32_t ofx,int32_t ofy,uint32_t& screen)
```

- **Declared at:** [line 142](../../../runtime/ordered_projection.hpp#L142)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `camera` | `const int32_t *` | Input | Value supplied for `camera`. See the exact type and module contract. |
| `h` | `int32_t` | Input | Value supplied for `h`. See the exact type and module contract. |
| `ofx` | `int32_t` | Input | Value supplied for `ofx`. See the exact type and module contract. |
| `ofy` | `int32_t` | Input | Value supplied for `ofy`. See the exact type and module contract. |
| `screen` | `uint32_t &` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered projection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Assume these named values have been initialized with valid data:
// const int32_t * camera
// int32_t h
// int32_t ofx
// int32_t ofy
// uint32_t & screen

auto result = epok::gte_project_point(camera, h, ofx, ofy, screen);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordered-depth-average-1"></a>

## `epok::ordered_depth_average`

**Purpose.** Exact signed division (truncate toward zero), without MIPS DIV.

**Details.** Keeping the two denominators in a conditional expression made GCC merge them into one variable divide. Shift the magnitude first, then divide by 3 via reciprocal.

**Exact declaration**

```cpp
inline int32_t ordered_depth_average(int32_t sum,bool quad)
```

- **Declared at:** [line 60](../../../runtime/ordered_projection.hpp#L60)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sum` | `int32_t` | Input | Value supplied for `sum`. See the exact type and module contract. |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** Keeping the two denominators in a conditional expression made GCC merge them into one variable divide. Shift the magnitude first, then divide by 3 via reciprocal.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Assume these named values have been initialized with valid data:
// int32_t sum
// bool quad

auto result = epok::ordered_depth_average(sum, quad);
```

**Why choose it.** It provides direct, allocation-conscious access to the ordered projection module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-ordered-gte-camera-reference-1"></a>

## `epok::ordered_gte_camera_reference`

**Purpose.** Independent integer oracle for the MAC results of the loaded projection matrix.

**Details.** Model the GTE's sum-then-floor rounding, including the inverted Y row and quantized translation; do not reuse the CPU fallback's different per-product truncation. Intended for validation, not the render hot path.

**Exact declaration**

```cpp
template<class Number> inline void ordered_gte_camera_reference(const Affine<Number>& matrix, const int32_t* world,const int32_t* offset,int32_t* camera)
```

- **Declared at:** [line 108](../../../runtime/ordered_projection.hpp#L108)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `matrix` | `const Affine<Number> &` | Input | Value supplied for `matrix`. See the exact type and module contract. |
| `world` | `const int32_t *` | Input | Value supplied for `world`. See the exact type and module contract. |
| `offset` | `const int32_t *` | Input | Value supplied for `offset`. See the exact type and module contract. |
| `camera` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `camera`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Model the GTE's sum-then-floor rounding, including the inverted Y row and quantized translation; do not reuse the CPU fallback's different per-product truncation. Intended for validation, not the render hot path.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const Affine<Number> & matrix
// const int32_t * world
// const int32_t * offset
// int32_t * camera

epok::ordered_gte_camera_reference<Number>(matrix, world, offset, camera);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordered-gte-depth-average-1"></a>

## `epok::ordered_gte_depth_average`

**Purpose.** Performs `ordered gte depth average` as part of the ordered projection module.

**Exact declaration**

```cpp
inline int32_t ordered_gte_depth_average(int32_t sum,bool quad)
```

- **Declared at:** [line 71](../../../runtime/ordered_projection.hpp#L71)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `sum` | `int32_t` | Input | Value supplied for `sum`. See the exact type and module contract. |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered projection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Assume these named values have been initialized with valid data:
// int32_t sum
// bool quad

auto result = epok::ordered_gte_depth_average(sum, quad);
```

**Why choose it.** It provides direct, allocation-conscious access to the ordered projection module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-ordered-gte-input-fits-1"></a>

## `epok::ordered_gte_input_fits`

**Purpose.** Q8 operands derived from signed Q12 each fit 28 bits.

**Details.** Subtract AFTER the shift: the difference is safe in i32 even across the full world range.

**Exact declaration**

```cpp
inline bool ordered_gte_input_fits(int32_t world_q12,int32_t camera_q8)
```

- **Declared at:** [line 10](../../../runtime/ordered_projection.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `world_q12` | `int32_t` | Input | Value supplied for `world_q12`. See the exact type and module contract. |
| `camera_q8` | `int32_t` | Input | Value supplied for `camera_q8`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Subtract AFTER the shift: the difference is safe in i32 even across the full world range.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Assume these named values have been initialized with valid data:
// int32_t world_q12
// int32_t camera_q8

auto result = epok::ordered_gte_input_fits(world_q12, camera_q8);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations.

<a id="epok-ordered-projection-anchor-1"></a>

## `epok::ordered_projection_anchor`

**Purpose.** Rebase Q8 operands on a 4096-Q8 (16-world-unit) lattice RELATIVE to the camera.

**Details.** R is Q12: R*(v-d*4096)/4096 + (TR+R*d) preserves the complete MAC sum, including fractional bits and negative-Y rounding. No precision drops. Offsets and centers originate in signed Q12. Publish outputs only on success.

**Exact declaration**

```cpp
template<class Number> #if defined(__GNUC__) && !defined(__clang__) __attribute__((noinline,optimize("O3"))) #elif defined(__GNUC__) __attribute__((noinline)) #endif bool ordered_projection_anchor(const Affine<Number>& matrix,const int32_t* base_offset, const int32_t* center,int32_t* offset,int32_t* translation)
```

- **Declared at:** [line 85](../../../runtime/ordered_projection.hpp#L85)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `matrix` | `const Affine<Number> &` | Input | Value supplied for `matrix`. See the exact type and module contract. |
| `base_offset` | `const int32_t *` | Input | Value supplied for `base_offset`. See the exact type and module contract. |
| `center` | `const int32_t *` | Input | Value supplied for `center`. See the exact type and module contract. |
| `offset` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `offset`. See the exact type and module contract. |
| `translation` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `translation`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** R is Q12: R*(v-d*4096)/4096 + (TR+R*d) preserves the complete MAC sum, including fractional bits and negative-Y rounding. No precision drops. Offsets and centers originate in signed Q12. Publish outputs only on success.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Number

// Assume these named values have been initialized with valid data:
// const Affine<Number> & matrix
// const int32_t * base_offset
// const int32_t * center
// int32_t * offset
// int32_t * translation

auto result = epok::ordered_projection_anchor<Number>(matrix, base_offset, center, offset, translation);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordered-screen-inside-1"></a>

## `epok::ordered_screen_inside`

**Purpose.** Performs `ordered screen inside` as part of the ordered projection module.

**Exact declaration**

```cpp
template<unsigned Width,unsigned Height> inline bool ordered_screen_inside(const uint32_t* screen,bool quad)
```

- **Declared at:** [line 36](../../../runtime/ordered_projection.hpp#L36)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `const uint32_t *` | Input | Value supplied for `screen`. See the exact type and module contract. |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered projection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Width, Height

// Assume these named values have been initialized with valid data:
// const uint32_t * screen
// bool quad

auto result = epok::ordered_screen_inside<Width, Height>(screen, quad);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-ordered-screen-inside-bits-1"></a>

## `epok::ordered_screen_inside_bits`

**Purpose.** Performs `ordered screen inside bits` as part of the ordered projection module.

**Exact declaration**

```cpp
template<unsigned Width,unsigned Height> inline uint32_t ordered_screen_inside_bits(uint32_t screen)
```

- **Declared at:** [line 30](../../../runtime/ordered_projection.hpp#L30)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `uint32_t` | Input | Value supplied for `screen`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the ordered projection module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Width, Height

// Assume these named values have been initialized with valid data:
// uint32_t screen

auto result = epok::ordered_screen_inside_bits<Width, Height>(screen);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="epok-ordered-screen-outside-1"></a>

## `epok::ordered_screen_outside`

**Purpose.** Trivial rejection on packed SXY words, the reference pass's test: true when every corner lies on the same outer side of the viewport, exactly when the corners' bounding box misses it.

**Details.** Sign bits give x<0 / y<0. Clearing them and adding 0x8000-W (per half) sets bit 15/31 for x>=W / y>=H without a carry between halves; masking with the inverted sign excludes negatives.

**Exact declaration**

```cpp
template<unsigned Width,unsigned Height> inline bool ordered_screen_outside(const uint32_t* screen,bool quad)
```

- **Declared at:** [line 48](../../../runtime/ordered_projection.hpp#L48)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `screen` | `const uint32_t *` | Input | Value supplied for `screen`. See the exact type and module contract. |
| `quad` | `bool` | Input | Value supplied for `quad`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Sign bits give x<0 / y<0. Clearing them and adding 0x8000-W (per half) sets bit 15/31 for x>=W / y>=H without a carry between halves; masking with the inverted sign excludes negatives.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Width, Height

// Assume these named values have been initialized with valid data:
// const uint32_t * screen
// bool quad

auto result = epok::ordered_screen_outside<Width, Height>(screen, quad);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-project-ordered-cpu-1"></a>

## `epok::project_ordered_cpu`

**Purpose.** Full-range projection is not synonymous with clipping.

**Details.** Keep native packets when all projected corners fit the GPU guard band and span limits.

**Exact declaration**

```cpp
template<class Units,int Width,int Height,class Number> #if defined(__GNUC__) && !defined(__clang__) __attribute__((noinline,optimize("O3"))) #elif defined(__GNUC__) __attribute__((noinline)) #endif bool project_ordered_cpu(const int32_t (*points)[3],unsigned count, const CameraSpace<Number>& view,const Number* row_scale, ProjectedVertex* projected,uint32_t* screen,int32_t* depth, const Affine<Number>* gte_matrix=nullptr,const int32_t* gte_offset=nullptr)
```

- **Declared at:** [line 172](../../../runtime/ordered_projection.hpp#L172)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `points` | `const int32_t (*)[3]` | Callback | Value supplied for `points`. See the exact type and module contract. |
| `count` | `unsigned int` | Input | Value supplied for `count`. See the exact type and module contract. |
| `view` | `const CameraSpace<Number> &` | Input | Value supplied for `view`. See the exact type and module contract. |
| `row_scale` | `const Number *` | Input | Value supplied for `row_scale`. See the exact type and module contract. |
| `projected` | `ProjectedVertex *` | Input/output; inspect the function contract | Value supplied for `projected`. See the exact type and module contract. |
| `screen` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `screen`. See the exact type and module contract. |
| `depth` | `int32_t *` | Input/output; inspect the function contract | Value supplied for `depth`. See the exact type and module contract. |
| `gte_matrix` | `const Affine<Number> *` | Input | Value supplied for `gte_matrix`. See the exact type and module contract. |
| `gte_offset` | `const int32_t *` | Input | Value supplied for `gte_offset`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Keep native packets when all projected corners fit the GPU guard band and span limits.

**Usage pattern**

```cpp
#include "ordered_projection.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Units, Width, Height, Number

// Assume these named values have been initialized with valid data:
// const int32_t (*)[3] points
// unsigned int count
// const CameraSpace<Number> & view
// const Number * row_scale
// ProjectedVertex * projected
// uint32_t * screen
// int32_t * depth
// const Affine<Number> * gte_matrix
// const int32_t * gte_offset

auto result = epok::project_ordered_cpu<Units, Width, Height, Number>(points, count, view, row_scale, projected, screen, depth, gte_matrix, gte_offset);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
