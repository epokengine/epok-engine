# PsyQo API: Gte Kernels

> **Header:** `"psyqo/gte-kernels.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh)

This module covers Geometry Transformation Engine math and register operations. It documents 74 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GTE::Kernels::LM`, `psyqo::GTE::Kernels::MV`, `psyqo::GTE::Kernels::MX`, `psyqo::GTE::Kernels::SF`, `psyqo::GTE::Kernels::TV`

## Callable index

- [`psyqo::GTE::Kernels::avsz3`](#psyqo-gte-kernels-avsz3-1) — AVSZ3 - Average of three Z values (for Triangles) zsf3 * (sz0 + sz1 + sz2) -> otz 5 cycles
- [`psyqo::GTE::Kernels::avsz4`](#psyqo-gte-kernels-avsz4-1) — AVSZ4 - Average of four Z values (for Quads) zsf4 * (sz0 + sz1 + sz2 + sz4) -> otz 6 cycles
- [`psyqo::GTE::Kernels::cc`](#psyqo-gte-kernels-cc-1) — Color Color limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 11 cycles
- [`psyqo::GTE::Kernels::cdp`](#psyqo-gte-kernels-cdp-1) — Color Depth Que limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 13 cycles
- [`psyqo::GTE::Kernels::cp`](#psyqo-gte-kernels-cp-1) — Cross Product (improperly named Outer Product in Sony's lingo) rt.22 * ir3 - rt.33 * ir2 -> ir1 rt.33 * ir1 - rt.11 * ir3 -> ir2 rt.11 * ir2 - rt.22 * ir1 -> ir3 6 cycles
- [`psyqo::GTE::Kernels::dpcl`](#psyqo-gte-kernels-dpcl-1) — DCPL - Depth Cue Color light (1 - dp)·[rgb·sv] + dp·[fc] -> rgb, lv, sv 8 cycles
- [`psyqo::GTE::Kernels::dpcs`](#psyqo-gte-kernels-dpcs-1) — DPCS - Depth Cueing (single) (1 - dp)·[rgb] + dp·[fc] -> rgb, lv, sv 8 cycles
- [`psyqo::GTE::Kernels::dpct`](#psyqo-gte-kernels-dpct-1) — DPCT - Depth Cueing (triple) (1 - dp)·[rgb0] + dp·[fc] -> rgb0, lv, sv (1 - dp)·[rgb1] + dp·[fc] -> rgb1, lv, sv (1 - dp)·[rgb2] + dp·[fc] -> rgb2, lv, sv 17 cycles
- [`psyqo::GTE::Kernels::gpf`](#psyqo-gte-kernels-gpf-1) — General purpose interpolation dp·[sv] -> lv, sv 5 cycles
- [`psyqo::GTE::Kernels::gpl`](#psyqo-gte-kernels-gpl-1) — General purpose interpolation with base [lv] + dp·[sv] -> lv, sv 5 cycles
- [`psyqo::GTE::Kernels::intpl`](#psyqo-gte-kernels-intpl-1) — INTPL - Interpolation of a vector and far color (1 - dp)·[sv] + dp·[fc] -> rgb2, lv, sv 8 cycles
- [`psyqo::GTE::Kernels::lc`](#psyqo-gte-kernels-lc-1) — limit(([lc]·[sv]) >> 12) + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::lcir`](#psyqo-gte-kernels-lcir-1) — ([lc]·[sv]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::lcirbk`](#psyqo-gte-kernels-lcirbk-1) — ([lc]·[sv]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::lcirfc`](#psyqo-gte-kernels-lcirfc-1) — ([lc]·[sv]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::lcirtr`](#psyqo-gte-kernels-lcirtr-1) — ([lc]·[sv]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::lcv0`](#psyqo-gte-kernels-lcv0-1) — ([lc]·[v0]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::lcv0bk`](#psyqo-gte-kernels-lcv0bk-1) — ([lc]·[v0]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::lcv0fc`](#psyqo-gte-kernels-lcv0fc-1) — ([lc]·[v0]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::lcv0tr`](#psyqo-gte-kernels-lcv0tr-1) — ([lc]·[v0]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::lcv1`](#psyqo-gte-kernels-lcv1-1) — ([lc]·[v1]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::lcv1bk`](#psyqo-gte-kernels-lcv1bk-1) — ([lc]·[v1]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::lcv1fc`](#psyqo-gte-kernels-lcv1fc-1) — ([lc]·[v1]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::lcv1tr`](#psyqo-gte-kernels-lcv1tr-1) — ([lc]·[v1]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::lcv2`](#psyqo-gte-kernels-lcv2-1) — ([lc]·[v2]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::lcv2bk`](#psyqo-gte-kernels-lcv2bk-1) — ([lc]·[v2]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::lcv2fc`](#psyqo-gte-kernels-lcv2fc-1) — ([lc]·[v2]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::lcv2tr`](#psyqo-gte-kernels-lcv2tr-1) — ([lc]·[v2]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::ll`](#psyqo-gte-kernels-ll-1) — limit(([ll]·[v0]) >> 12) -> lv, sv
- [`psyqo::GTE::Kernels::llir`](#psyqo-gte-kernels-llir-1) — ([ll]·[sv]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::llirbk`](#psyqo-gte-kernels-llirbk-1) — ([ll]·[sv]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::llirfc`](#psyqo-gte-kernels-llirfc-1) — ([ll]·[sv]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::llirtr`](#psyqo-gte-kernels-llirtr-1) — ([ll]·[sv]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::llv0`](#psyqo-gte-kernels-llv0-1) — ([ll]·[v0]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::llv0bk`](#psyqo-gte-kernels-llv0bk-1) — ([ll]·[v0]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::llv0fc`](#psyqo-gte-kernels-llv0fc-1) — ([ll]·[v0]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::llv0tr`](#psyqo-gte-kernels-llv0tr-1) — ([ll]·[v0]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::llv1`](#psyqo-gte-kernels-llv1-1) — ([ll]·[v1]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::llv1bk`](#psyqo-gte-kernels-llv1bk-1) — ([ll]·[v1]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::llv1fc`](#psyqo-gte-kernels-llv1fc-1) — ([ll]·[v1]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::llv1tr`](#psyqo-gte-kernels-llv1tr-1) — ([ll]·[v1]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::llv2`](#psyqo-gte-kernels-llv2-1) — ([ll]·[v2]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::llv2bk`](#psyqo-gte-kernels-llv2bk-1) — ([ll]·[v2]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::llv2fc`](#psyqo-gte-kernels-llv2fc-1) — ([ll]·[v2]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::llv2tr`](#psyqo-gte-kernels-llv2tr-1) — ([ll]·[v2]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::mvmva`](#psyqo-gte-kernels-mvmva-1) — Multiply vector by matrix and add vector
- [`psyqo::GTE::Kernels::nccs`](#psyqo-gte-kernels-nccs-1) — NCCS - Normal Color Color (single vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 17 cycles
- [`psyqo::GTE::Kernels::ncct`](#psyqo-gte-kernels-ncct-1) — NCCT - Normal Color Color (triple vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 39 cycles
- [`psyqo::GTE::Kernels::ncds`](#psyqo-gte-kernels-ncds-1) — NCDS - Normal color depth cue (single vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 19 cycles
- [`psyqo::GTE::Kernels::ncdt`](#psyqo-gte-kernels-ncdt-1) — NCDT - Normal color depth cue (triple vectors) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 44 cycles
- [`psyqo::GTE::Kernels::nclip`](#psyqo-gte-kernels-nclip-1) — NCLIP - Normal clipping sx0*sy1 + sx1*sy2 + sx2*sy0 - sx0*sy2 - sx1*sy0 - sx2*sy1 -> opz aka determinant of the matrix [sx1 - sx0, sy1 - sy0] [sx2 - sx0, sy2 - sy0] 8 cycles
- [`psyqo::GTE::Kernels::ncs`](#psyqo-gte-kernels-ncs-1) — NCS - Normal color (single) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb2 14 cycles
- [`psyqo::GTE::Kernels::nct`](#psyqo-gte-kernels-nct-1) — NCT - Normal color (triple) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb2 30 cycles
- [`psyqo::GTE::Kernels::rt`](#psyqo-gte-kernels-rt-1) — Coordinate Conversion, Light Source Calculations ([rt]·[v0]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::rtir`](#psyqo-gte-kernels-rtir-1) — ([rt]·[sv]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::rtir_sf0`](#psyqo-gte-kernels-rtir-sf0-1) — [rt]·[sv] -> lv
- [`psyqo::GTE::Kernels::rtirbk`](#psyqo-gte-kernels-rtirbk-1) — ([rt]·[sv]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::rtirfc`](#psyqo-gte-kernels-rtirfc-1) — ([rt]·[sv]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::rtirtr`](#psyqo-gte-kernels-rtirtr-1) — ([rt]·[sv]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::rtps`](#psyqo-gte-kernels-rtps-1) — RTPS - Perspective Transformation (single) pers(([rt]·[v0]) >> 12 + [tr]) -> sxy2 14 cycles
- [`psyqo::GTE::Kernels::rtpt`](#psyqo-gte-kernels-rtpt-1) — RTPT - Perspective Transformation (triple) pers(([rt]·[v0]) >> 12 + [tr]) -> sxy0 pers(([rt]·[v1]) >> 12 + [tr]) -> sxy1 pers(([rt]·[v2]) >> 12 + [tr]) -> sxy2 22 cycles
- [`psyqo::GTE::Kernels::rtv0`](#psyqo-gte-kernels-rtv0-1) — General Matrix Operations ([rt]·[v0]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::rtv0bk`](#psyqo-gte-kernels-rtv0bk-1) — ([rt]·[v0]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::rtv0fc`](#psyqo-gte-kernels-rtv0fc-1) — ([rt]·[v0]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::rtv0tr`](#psyqo-gte-kernels-rtv0tr-1) — ([rt]·[v0]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::rtv1`](#psyqo-gte-kernels-rtv1-1) — ([rt]·[v1]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::rtv1bk`](#psyqo-gte-kernels-rtv1bk-1) — ([rt]·[v1]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::rtv1fc`](#psyqo-gte-kernels-rtv1fc-1) — ([rt]·[v1]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::rtv1tr`](#psyqo-gte-kernels-rtv1tr-1) — ([rt]·[v1]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::rtv2`](#psyqo-gte-kernels-rtv2-1) — ([rt]·[v2]) >> 12 -> lv, sv
- [`psyqo::GTE::Kernels::rtv2bk`](#psyqo-gte-kernels-rtv2bk-1) — ([rt]·[v2]) >> 12 + [bk] -> lv, sv
- [`psyqo::GTE::Kernels::rtv2fc`](#psyqo-gte-kernels-rtv2fc-1) — ([rt]·[v2]) >> 12 + [fc] -> lv, sv
- [`psyqo::GTE::Kernels::rtv2tr`](#psyqo-gte-kernels-rtv2tr-1) — ([rt]·[v2]) >> 12 + [tr] -> lv, sv
- [`psyqo::GTE::Kernels::sqr`](#psyqo-gte-kernels-sqr-1) — Termwise Vector Square [sv.x² >> 12, sv.y² >> 12, sv.z² >> 12] -> lv, sv 5 cycles

<a id="psyqo-gte-kernels-avsz3-1"></a>

## `psyqo::GTE::Kernels::avsz3`

**Purpose.** AVSZ3 - Average of three Z values (for Triangles) zsf3 * (sz0 + sz1 + sz2) -> otz 5 cycles

**Exact declaration**

```cpp
static inline void avsz3()
```

- **Declared at:** [line 248](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L248)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::avsz3();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-avsz4-1"></a>

## `psyqo::GTE::Kernels::avsz4`

**Purpose.** AVSZ4 - Average of four Z values (for Quads) zsf4 * (sz0 + sz1 + sz2 + sz4) -> otz 6 cycles

**Exact declaration**

```cpp
static inline void avsz4()
```

- **Declared at:** [line 257](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L257)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::avsz4();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-cc-1"></a>

## `psyqo::GTE::Kernels::cc`

**Purpose.** Color Color limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 11 cycles

**Exact declaration**

```cpp
static inline void cc()
```

- **Declared at:** [line 225](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L225)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::cc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-cdp-1"></a>

## `psyqo::GTE::Kernels::cdp`

**Purpose.** Color Depth Que limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 13 cycles

**Exact declaration**

```cpp
static inline void cdp()
```

- **Declared at:** [line 215](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L215)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::cdp();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-cp-1"></a>

## `psyqo::GTE::Kernels::cp`

**Purpose.** Cross Product (improperly named Outer Product in Sony's lingo) rt.22 * ir3 - rt.33 * ir2 -> ir1 rt.33 * ir1 - rt.11 * ir3 -> ir2 rt.11 * ir2 - rt.22 * ir1 -> ir3 6 cycles

**Exact declaration**

```cpp
template <SF sf = Shifted> static inline void cp()
```

- **Declared at:** [line 269](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L269)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

// Replace these template arguments with types or values accepted by the declaration:
// sf

psyqo::GTE::Kernels::cp<sf>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-kernels-dpcl-1"></a>

## `psyqo::GTE::Kernels::dpcl`

**Purpose.** DCPL - Depth Cue Color light (1 - dp)·[rgb·sv] + dp·[fc] -> rgb, lv, sv 8 cycles

**Exact declaration**

```cpp
static inline void dpcl()
```

- **Declared at:** [line 80](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L80)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::dpcl();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-dpcs-1"></a>

## `psyqo::GTE::Kernels::dpcs`

**Purpose.** DPCS - Depth Cueing (single) (1 - dp)·[rgb] + dp·[fc] -> rgb, lv, sv 8 cycles

**Exact declaration**

```cpp
static inline void dpcs()
```

- **Declared at:** [line 89](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L89)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::dpcs();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-dpct-1"></a>

## `psyqo::GTE::Kernels::dpct`

**Purpose.** DPCT - Depth Cueing (triple) (1 - dp)·[rgb0] + dp·[fc] -> rgb0, lv, sv (1 - dp)·[rgb1] + dp·[fc] -> rgb1, lv, sv (1 - dp)·[rgb2] + dp·[fc] -> rgb2, lv, sv 17 cycles

**Exact declaration**

```cpp
static inline void dpct()
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L100)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::dpct();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-gpf-1"></a>

## `psyqo::GTE::Kernels::gpf`

**Purpose.** General purpose interpolation dp·[sv] -> lv, sv 5 cycles

**Exact declaration**

```cpp
template <SF sf = Shifted> static inline void gpf()
```

- **Declared at:** [line 285](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L285)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

// Replace these template arguments with types or values accepted by the declaration:
// sf

psyqo::GTE::Kernels::gpf<sf>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-kernels-gpl-1"></a>

## `psyqo::GTE::Kernels::gpl`

**Purpose.** General purpose interpolation with base [lv] + dp·[sv] -> lv, sv 5 cycles

**Exact declaration**

```cpp
template <SF sf = Shifted> static inline void gpl()
```

- **Declared at:** [line 299](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L299)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

// Replace these template arguments with types or values accepted by the declaration:
// sf

psyqo::GTE::Kernels::gpl<sf>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-kernels-intpl-1"></a>

## `psyqo::GTE::Kernels::intpl`

**Purpose.** INTPL - Interpolation of a vector and far color (1 - dp)·[sv] + dp·[fc] -> rgb2, lv, sv 8 cycles

**Exact declaration**

```cpp
static inline void intpl()
```

- **Declared at:** [line 109](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L109)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::intpl();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lc-1"></a>

## `psyqo::GTE::Kernels::lc`

**Purpose.** limit(([lc]·[sv]) >> 12) + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void lc()
```

- **Declared at:** [line 339](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L339)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcir-1"></a>

## `psyqo::GTE::Kernels::lcir`

**Purpose.** ([lc]·[sv]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void lcir()
```

- **Declared at:** [line 415](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L415)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcir();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcirbk-1"></a>

## `psyqo::GTE::Kernels::lcirbk`

**Purpose.** ([lc]·[sv]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void lcirbk()
```

- **Declared at:** [line 431](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L431)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcirbk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcirfc-1"></a>

## `psyqo::GTE::Kernels::lcirfc`

**Purpose.** ([lc]·[sv]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void lcirfc()
```

- **Declared at:** [line 439](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L439)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcirfc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcirtr-1"></a>

## `psyqo::GTE::Kernels::lcirtr`

**Purpose.** ([lc]·[sv]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void lcirtr()
```

- **Declared at:** [line 423](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L423)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcirtr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv0-1"></a>

## `psyqo::GTE::Kernels::lcv0`

**Purpose.** ([lc]·[v0]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void lcv0()
```

- **Declared at:** [line 409](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L409)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv0();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv0bk-1"></a>

## `psyqo::GTE::Kernels::lcv0bk`

**Purpose.** ([lc]·[v0]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv0bk()
```

- **Declared at:** [line 425](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L425)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv0bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv0fc-1"></a>

## `psyqo::GTE::Kernels::lcv0fc`

**Purpose.** ([lc]·[v0]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv0fc()
```

- **Declared at:** [line 433](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L433)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv0fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv0tr-1"></a>

## `psyqo::GTE::Kernels::lcv0tr`

**Purpose.** ([lc]·[v0]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv0tr()
```

- **Declared at:** [line 417](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L417)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv0tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv1-1"></a>

## `psyqo::GTE::Kernels::lcv1`

**Purpose.** ([lc]·[v1]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void lcv1()
```

- **Declared at:** [line 411](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L411)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv1();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv1bk-1"></a>

## `psyqo::GTE::Kernels::lcv1bk`

**Purpose.** ([lc]·[v1]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv1bk()
```

- **Declared at:** [line 427](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L427)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv1bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv1fc-1"></a>

## `psyqo::GTE::Kernels::lcv1fc`

**Purpose.** ([lc]·[v1]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv1fc()
```

- **Declared at:** [line 435](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L435)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv1fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv1tr-1"></a>

## `psyqo::GTE::Kernels::lcv1tr`

**Purpose.** ([lc]·[v1]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv1tr()
```

- **Declared at:** [line 419](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L419)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv1tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv2-1"></a>

## `psyqo::GTE::Kernels::lcv2`

**Purpose.** ([lc]·[v2]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void lcv2()
```

- **Declared at:** [line 413](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L413)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv2();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv2bk-1"></a>

## `psyqo::GTE::Kernels::lcv2bk`

**Purpose.** ([lc]·[v2]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv2bk()
```

- **Declared at:** [line 429](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L429)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv2bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv2fc-1"></a>

## `psyqo::GTE::Kernels::lcv2fc`

**Purpose.** ([lc]·[v2]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv2fc()
```

- **Declared at:** [line 437](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L437)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv2fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-lcv2tr-1"></a>

## `psyqo::GTE::Kernels::lcv2tr`

**Purpose.** ([lc]·[v2]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void lcv2tr()
```

- **Declared at:** [line 421](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L421)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::lcv2tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-ll-1"></a>

## `psyqo::GTE::Kernels::ll`

**Purpose.** limit(([ll]·[v0]) >> 12) -> lv, sv

**Exact declaration**

```cpp
static inline void ll()
```

- **Declared at:** [line 337](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L337)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::ll();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llir-1"></a>

## `psyqo::GTE::Kernels::llir`

**Purpose.** ([ll]·[sv]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void llir()
```

- **Declared at:** [line 383](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L383)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llir();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llirbk-1"></a>

## `psyqo::GTE::Kernels::llirbk`

**Purpose.** ([ll]·[sv]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void llirbk()
```

- **Declared at:** [line 399](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L399)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llirbk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llirfc-1"></a>

## `psyqo::GTE::Kernels::llirfc`

**Purpose.** ([ll]·[sv]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void llirfc()
```

- **Declared at:** [line 407](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L407)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llirfc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llirtr-1"></a>

## `psyqo::GTE::Kernels::llirtr`

**Purpose.** ([ll]·[sv]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void llirtr()
```

- **Declared at:** [line 391](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L391)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llirtr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv0-1"></a>

## `psyqo::GTE::Kernels::llv0`

**Purpose.** ([ll]·[v0]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void llv0()
```

- **Declared at:** [line 377](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L377)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv0();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv0bk-1"></a>

## `psyqo::GTE::Kernels::llv0bk`

**Purpose.** ([ll]·[v0]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void llv0bk()
```

- **Declared at:** [line 393](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L393)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv0bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv0fc-1"></a>

## `psyqo::GTE::Kernels::llv0fc`

**Purpose.** ([ll]·[v0]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void llv0fc()
```

- **Declared at:** [line 401](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L401)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv0fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv0tr-1"></a>

## `psyqo::GTE::Kernels::llv0tr`

**Purpose.** ([ll]·[v0]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void llv0tr()
```

- **Declared at:** [line 385](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L385)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv0tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv1-1"></a>

## `psyqo::GTE::Kernels::llv1`

**Purpose.** ([ll]·[v1]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void llv1()
```

- **Declared at:** [line 379](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L379)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv1();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv1bk-1"></a>

## `psyqo::GTE::Kernels::llv1bk`

**Purpose.** ([ll]·[v1]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void llv1bk()
```

- **Declared at:** [line 395](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L395)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv1bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv1fc-1"></a>

## `psyqo::GTE::Kernels::llv1fc`

**Purpose.** ([ll]·[v1]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void llv1fc()
```

- **Declared at:** [line 403](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L403)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv1fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv1tr-1"></a>

## `psyqo::GTE::Kernels::llv1tr`

**Purpose.** ([ll]·[v1]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void llv1tr()
```

- **Declared at:** [line 387](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L387)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv1tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv2-1"></a>

## `psyqo::GTE::Kernels::llv2`

**Purpose.** ([ll]·[v2]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void llv2()
```

- **Declared at:** [line 381](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L381)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv2();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv2bk-1"></a>

## `psyqo::GTE::Kernels::llv2bk`

**Purpose.** ([ll]·[v2]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void llv2bk()
```

- **Declared at:** [line 397](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L397)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv2bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv2fc-1"></a>

## `psyqo::GTE::Kernels::llv2fc`

**Purpose.** ([ll]·[v2]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void llv2fc()
```

- **Declared at:** [line 405](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L405)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv2fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-llv2tr-1"></a>

## `psyqo::GTE::Kernels::llv2tr`

**Purpose.** ([ll]·[v2]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void llv2tr()
```

- **Declared at:** [line 389](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L389)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::llv2tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-mvmva-1"></a>

## `psyqo::GTE::Kernels::mvmva`

**Purpose.** Multiply vector by matrix and add vector

**Exact declaration**

```cpp
template <MX mx, MV v, TV cv = TV::Zero, SF sf = Shifted, LM lm = Unlimited> void mvmva()
```

- **Declared at:** [line 325](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L325)
- **Kind:** `function template`; qualifiers: `template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

// Replace these template arguments with types or values accepted by the declaration:
// mx, v, cv, sf, lm

psyqo::GTE::Kernels& object = /* obtain a valid instance */;

object.mvmva<mx, v, cv, sf, lm>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.

<a id="psyqo-gte-kernels-nccs-1"></a>

## `psyqo::GTE::Kernels::nccs`

**Purpose.** NCCS - Normal Color Color (single vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 17 cycles

**Exact declaration**

```cpp
static inline void nccs()
```

- **Declared at:** [line 188](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L188)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::nccs();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-ncct-1"></a>

## `psyqo::GTE::Kernels::ncct`

**Purpose.** NCCT - Normal Color Color (triple vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv [rgb·sv] -> rgb2 39 cycles

**Exact declaration**

```cpp
static inline void ncct()
```

- **Declared at:** [line 205](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L205)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::ncct();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-ncds-1"></a>

## `psyqo::GTE::Kernels::ncds`

**Purpose.** NCDS - Normal color depth cue (single vector) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 19 cycles

**Exact declaration**

```cpp
static inline void ncds()
```

- **Declared at:** [line 160](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L160)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::ncds();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-ncdt-1"></a>

## `psyqo::GTE::Kernels::ncdt`

**Purpose.** NCDT - Normal color depth cue (triple vectors) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> sv (1 - dp)·[rgb·sv] + dp·[fc] -> rgb2 44 cycles

**Exact declaration**

```cpp
static inline void ncdt()
```

- **Declared at:** [line 177](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L177)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::ncdt();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-nclip-1"></a>

## `psyqo::GTE::Kernels::nclip`

**Purpose.** NCLIP - Normal clipping sx0*sy1 + sx1*sy2 + sx2*sy0 - sx0*sy2 - sx1*sy0 - sx2*sy1 -> opz aka determinant of the matrix [sx1 - sx0, sy1 - sy0] [sx2 - sx0, sy2 - sy0] 8 cycles

**Exact declaration**

```cpp
static inline void nclip()
```

- **Declared at:** [line 237](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L237)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::nclip();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-ncs-1"></a>

## `psyqo::GTE::Kernels::ncs`

**Purpose.** NCS - Normal color (single) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb2 14 cycles

**Exact declaration**

```cpp
static inline void ncs()
```

- **Declared at:** [line 135](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L135)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::ncs();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-nct-1"></a>

## `psyqo::GTE::Kernels::nct`

**Purpose.** NCT - Normal color (triple) limit(([ll]·[v0]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb0 limit(([ll]·[v1]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb1 limit(([ll]·[v2]) >> 12) -> sv limit(([lc]·[sv]) >> 12) + [bk] -> rgb2 30 cycles

**Exact declaration**

```cpp
static inline void nct()
```

- **Declared at:** [line 149](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L149)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::nct();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rt-1"></a>

## `psyqo::GTE::Kernels::rt`

**Purpose.** Coordinate Conversion, Light Source Calculations ([rt]·[v0]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void rt()
```

- **Declared at:** [line 335](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L335)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rt();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtir-1"></a>

## `psyqo::GTE::Kernels::rtir`

**Purpose.** ([rt]·[sv]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void rtir()
```

- **Declared at:** [line 351](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L351)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtir();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtir-sf0-1"></a>

## `psyqo::GTE::Kernels::rtir_sf0`

**Purpose.** [rt]·[sv] -> lv

**Exact declaration**

```cpp
static inline void rtir_sf0()
```

- **Declared at:** [line 341](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L341)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtir_sf0();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtirbk-1"></a>

## `psyqo::GTE::Kernels::rtirbk`

**Purpose.** ([rt]·[sv]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void rtirbk()
```

- **Declared at:** [line 367](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L367)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtirbk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtirfc-1"></a>

## `psyqo::GTE::Kernels::rtirfc`

**Purpose.** ([rt]·[sv]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void rtirfc()
```

- **Declared at:** [line 375](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L375)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtirfc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtirtr-1"></a>

## `psyqo::GTE::Kernels::rtirtr`

**Purpose.** ([rt]·[sv]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void rtirtr()
```

- **Declared at:** [line 359](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L359)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtirtr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtps-1"></a>

## `psyqo::GTE::Kernels::rtps`

**Purpose.** RTPS - Perspective Transformation (single) pers(([rt]·[v0]) >> 12 + [tr]) -> sxy2 14 cycles

**Exact declaration**

```cpp
static inline void rtps()
```

- **Declared at:** [line 58](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L58)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtps();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtpt-1"></a>

## `psyqo::GTE::Kernels::rtpt`

**Purpose.** RTPT - Perspective Transformation (triple) pers(([rt]·[v0]) >> 12 + [tr]) -> sxy0 pers(([rt]·[v1]) >> 12 + [tr]) -> sxy1 pers(([rt]·[v2]) >> 12 + [tr]) -> sxy2 22 cycles

**Exact declaration**

```cpp
static inline void rtpt()
```

- **Declared at:** [line 69](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L69)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtpt();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv0-1"></a>

## `psyqo::GTE::Kernels::rtv0`

**Purpose.** General Matrix Operations ([rt]·[v0]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void rtv0()
```

- **Declared at:** [line 345](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L345)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv0();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv0bk-1"></a>

## `psyqo::GTE::Kernels::rtv0bk`

**Purpose.** ([rt]·[v0]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv0bk()
```

- **Declared at:** [line 361](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L361)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv0bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv0fc-1"></a>

## `psyqo::GTE::Kernels::rtv0fc`

**Purpose.** ([rt]·[v0]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv0fc()
```

- **Declared at:** [line 369](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L369)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv0fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv0tr-1"></a>

## `psyqo::GTE::Kernels::rtv0tr`

**Purpose.** ([rt]·[v0]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv0tr()
```

- **Declared at:** [line 353](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L353)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv0tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv1-1"></a>

## `psyqo::GTE::Kernels::rtv1`

**Purpose.** ([rt]·[v1]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void rtv1()
```

- **Declared at:** [line 347](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L347)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv1();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv1bk-1"></a>

## `psyqo::GTE::Kernels::rtv1bk`

**Purpose.** ([rt]·[v1]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv1bk()
```

- **Declared at:** [line 363](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L363)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv1bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv1fc-1"></a>

## `psyqo::GTE::Kernels::rtv1fc`

**Purpose.** ([rt]·[v1]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv1fc()
```

- **Declared at:** [line 371](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L371)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv1fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv1tr-1"></a>

## `psyqo::GTE::Kernels::rtv1tr`

**Purpose.** ([rt]·[v1]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv1tr()
```

- **Declared at:** [line 355](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L355)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv1tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv2-1"></a>

## `psyqo::GTE::Kernels::rtv2`

**Purpose.** ([rt]·[v2]) >> 12 -> lv, sv

**Exact declaration**

```cpp
static inline void rtv2()
```

- **Declared at:** [line 349](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L349)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv2();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv2bk-1"></a>

## `psyqo::GTE::Kernels::rtv2bk`

**Purpose.** ([rt]·[v2]) >> 12 + [bk] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv2bk()
```

- **Declared at:** [line 365](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L365)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv2bk();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv2fc-1"></a>

## `psyqo::GTE::Kernels::rtv2fc`

**Purpose.** ([rt]·[v2]) >> 12 + [fc] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv2fc()
```

- **Declared at:** [line 373](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L373)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv2fc();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-rtv2tr-1"></a>

## `psyqo::GTE::Kernels::rtv2tr`

**Purpose.** ([rt]·[v2]) >> 12 + [tr] -> lv, sv

**Exact declaration**

```cpp
static inline void rtv2tr()
```

- **Declared at:** [line 357](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L357)
- **Kind:** `function decl`; qualifiers: `static`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

psyqo::GTE::Kernels::rtv2tr();
```

**Why choose it.** It provides direct, allocation-conscious access to Geometry Transformation Engine math and register operations. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-gte-kernels-sqr-1"></a>

## `psyqo::GTE::Kernels::sqr`

**Purpose.** Termwise Vector Square [sv.x² >> 12, sv.y² >> 12, sv.z² >> 12] -> lv, sv 5 cycles

**Exact declaration**

```cpp
template <SF sf = Shifted> static inline void sqr()
```

- **Declared at:** [line 119](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/gte-kernels.hh#L119)
- **Kind:** `function template`; qualifiers: `static, template`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need Geometry Transformation Engine math and register operations and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/gte-kernels.hh"

// Replace these template arguments with types or values accepted by the declaration:
// sf

psyqo::GTE::Kernels::sqr<sf>();
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.
