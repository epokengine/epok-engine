# Epok API: Baked Packet

> **Header:** `"baked_packet.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/baked_packet.hpp)

This module covers the baked packet module. It documents 4 public callables declared directly in this header.

## Declared types

`epok::BakedPacket`, `epok::BakedTextureWords`, `epok::MonotoneDrawBucket`

## Callable index

- [`epok::baked_second_half`](#epok-baked-second-half-1) — Performs `baked second half` as part of the baked packet module.
- [`epok::baked_texture_words`](#epok-baked-texture-words-1) — A face's cooked texture words as the packet takes them: the flags cleared and the display mode bits (as baked_texture_packet) added to the TPAGE.
- [`epok::BakedPacket::write`](#epok-bakedpacket-write-1) — Destination excludes its DMA tag.
- [`epok::MonotoneDrawBucket::next`](#epok-monotonedrawbucket-next-1) — Performs `next` as part of the baked packet module.

<a id="epok-baked-second-half-1"></a>

## `epok::baked_second_half`

**Purpose.** Performs `baked second half` as part of the baked packet module.

**Exact declaration**

```cpp
inline BakedTextureWords baked_second_half(BakedTextureWords source)
```

- **Declared at:** [line 10](../../../runtime/baked_packet.hpp#L10)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `source` | `BakedTextureWords` | Input | Value supplied for `source`. See the exact type and module contract. |

**Returns.** Returns `BakedTextureWords`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked packet module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_packet.hpp"

// Assume these named values have been initialized with valid data:
// BakedTextureWords source

auto result = epok::baked_second_half(source);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked packet module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-baked-texture-words-1"></a>

## `epok::baked_texture_words`

**Purpose.** A face's cooked texture words as the packet takes them: the flags cleared and the display mode bits (as baked_texture_packet) added to the TPAGE.

**Exact declaration**

```cpp
template<bool Interlaced,bool Dithering> inline BakedTextureWords baked_texture_words(const BakedDrawFace& face)
```

- **Declared at:** [line 17](../../../runtime/baked_packet.hpp#L17)
- **Kind:** `function template`; qualifiers: `template`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `face` | `const BakedDrawFace &` | Input | Value supplied for `face`. See the exact type and module contract. |

**Returns.** Returns `BakedTextureWords`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked packet module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_packet.hpp"

// Replace these template arguments with types or values accepted by the declaration:
// Interlaced, Dithering

// Assume these named values have been initialized with valid data:
// const BakedDrawFace & face

auto result = epok::baked_texture_words<Interlaced, Dithering>(face);
```

**Why choose it.** Template dispatch is resolved at compile time and normally adds no runtime indirection.

**Trade-offs and warnings.** Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-bakedpacket-write-1"></a>

## `epok::BakedPacket::write`

**Purpose.** Destination excludes its DMA tag.

**Details.** Caller supplies projected corner order, raw RGB24 and the cooked triangle opcode (quad bit is added here). No lighting, sort, clipping, allocation, projection or asset lookup here.

**Exact declaration**

```cpp
static inline void write(uint32_t* output,const uint32_t* rgb,const uint32_t* screen, BakedTextureWords uv,uint32_t command)
```

- **Declared at:** [line 27](../../../runtime/baked_packet.hpp#L27)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `output` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `output`. See the exact type and module contract. |
| `rgb` | `const uint32_t *` | Input | Value supplied for `rgb`. See the exact type and module contract. |
| `screen` | `const uint32_t *` | Input | Value supplied for `screen`. See the exact type and module contract. |
| `uv` | `BakedTextureWords` | Input | Value supplied for `uv`. See the exact type and module contract. |
| `command` | `uint32_t` | Input | Value supplied for `command`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Caller supplies projected corner order, raw RGB24 and the cooked triangle opcode (quad bit is added here). No lighting, sort, clipping, allocation, projection or asset lookup here.

**Usage pattern**

```cpp
#include "baked_packet.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * output
// const uint32_t * rgb
// const uint32_t * screen
// BakedTextureWords uv
// uint32_t command

epok::BakedPacket::write(output, rgb, screen, uv, command);
```

**Why choose it.** It provides direct, allocation-conscious access to the baked packet module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-monotonedrawbucket-next-1"></a>

## `epok::MonotoneDrawBucket::next`

**Purpose.** Performs `next` as part of the baked packet module.

**Exact declaration**

```cpp
uint16_t next(uint16_t candidate)
```

- **Declared at:** [line 47](../../../runtime/baked_packet.hpp#L47)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `candidate` | `uint16_t` | Input | Value supplied for `candidate`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need the baked packet module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "baked_packet.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t candidate

epok::MonotoneDrawBucket& object = /* obtain a valid instance */;

auto result = object.next(candidate);
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.
