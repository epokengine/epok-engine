# PsyQo API: Bezier

> **Header:** `"psyqo/bezier.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bezier.hh)

This module covers the bezier module. It documents 2 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Callable index

- [`psyqo::Bezier::cubic`](#psyqo-bezier-cubic-1) — Cubic Bezier curve helper function.
- [`psyqo::Bezier::cubic`](#psyqo-bezier-cubic-2) — Performs `cubic` as part of the bezier module.

<a id="psyqo-bezier-cubic-1"></a>

## `psyqo::Bezier::cubic`

**Purpose.** Cubic Bezier curve helper function.

**Exact declaration**

```cpp
Vec2 cubic(const Vec2& a, const Vec2& b, const Vec2& c, const Vec2& d, FixedPoint<> t)
```

- **Declared at:** [line 45](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bezier.hh#L45)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `a` | `const Vec2 &` | Input | Start of the Bezier curve. |
| `b` | `const Vec2 &` | Input | Control point 1. |
| `c` | `const Vec2 &` | Input | Control point 2. |
| `d` | `const Vec2 &` | Input | End of the Bezier curve. |
| `t` | `FixedPoint<>` | Input | The point on the curve to sample, from 0.0 to 1.0. |

**Returns.** Vec2 The point on the curve at t.

**Use it when.** You need the bezier module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bezier.hh"

// Assume these named values have been initialized with valid data:
// const Vec2 & a
// const Vec2 & b
// const Vec2 & c
// const Vec2 & d
// FixedPoint<> t

auto result = psyqo::Bezier::cubic(a, b, c, d, t);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-bezier-cubic-2"></a>

## `psyqo::Bezier::cubic`

**Purpose.** Performs `cubic` as part of the bezier module.

**Exact declaration**

```cpp
Vec3 cubic(const Vec3& a, const Vec3& b, const Vec3& c, const Vec3& d, FixedPoint<> t)
```

- **Declared at:** [line 46](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bezier.hh#L46)
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

**Use it when.** You need the bezier module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/bezier.hh"

// Assume these named values have been initialized with valid data:
// const Vec3 & a
// const Vec3 & b
// const Vec3 & c
// const Vec3 & d
// FixedPoint<> t

auto result = psyqo::Bezier::cubic(a, b, c, d, t);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
