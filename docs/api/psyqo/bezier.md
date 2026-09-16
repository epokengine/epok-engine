# PsyQo API: Bezier

> **Header:** `"psyqo/bezier.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/bezier.hh)

This module covers the bezier module. It documents 1 public callable declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Callable index

- [`psyqo::Bezier::cubic`](#psyqo-bezier-cubic-1) — Cubic Bezier curve helper function.

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
| `t` | `int` | Input | The point on the curve to sample, from 0.0 to 1.0. |

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
// int t

auto result = psyqo::Bezier::cubic(a, b, c, d, t);
```

**Why choose it.** Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.

**Trade-offs and warnings.** Stay within the documented range and account for quantization before chaining several operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
