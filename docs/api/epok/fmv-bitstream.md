# Epok API: Fmv Bitstream

> **Header:** `"fmv_bitstream.hpp"` · **Tier:** Engine internal · **Source:** [open header](../../../runtime/fmv_bitstream.hpp)

This module covers full-motion video clips and playback statistics. It documents 17 public callables declared directly in this header.

## Declared types

`epok::fmv::AcTables`, `epok::fmv::BsHeader`, `epok::fmv::BsResult`, `epok::fmv::BsStatus`, `epok::fmv::detail::AcBuild`, `epok::fmv::detail::AcCode`, `epok::fmv::detail::BitReader`, `epok::fmv::QuantTables`

## Callable index

- [`epok::fmv::decode_frame`](#epok-fmv-decode-frame-1) — Decodes one frame into `out`: the MDEC decode command followed by the run-length codes, padded with end-of-block codes to whole 32-word DMA blocks.
- [`epok::fmv::detail::ac_build`](#epok-fmv-detail-ac-build-1) — Performs `ac build` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::ac_entry`](#epok-fmv-detail-ac-entry-1) — Performs `ac entry` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::ac_fill`](#epok-fmv-detail-ac-fill-1) — Fills every table slot whose leading bits are `code`.
- [`epok::fmv::detail::BitReader::BitReader`](#epok-fmv-detail-bitreader-bitreader-1) — Constructs `epok::fmv::detail::BitReader` for full-motion video clips and playback statistics.
- [`epok::fmv::detail::BitReader::overrun`](#epok-fmv-detail-bitreader-overrun-1) — Performs `overrun` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::BitReader::peek16`](#epok-fmv-detail-bitreader-peek16-1) — Performs `peek16` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::BitReader::refill`](#epok-fmv-detail-bitreader-refill-1) — Performs `refill` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::BitReader::skip`](#epok-fmv-detail-bitreader-skip-1) — Performs `skip` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::BitReader::take`](#epok-fmv-detail-bitreader-take-1) — Performs `take` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::dc_size`](#epok-fmv-detail-dc-size-1) — MPEG-1 DC size: the number of differential bits that follow.
- [`epok::fmv::detail::extend`](#epok-fmv-detail-extend-1) — Performs `extend` as part of full-motion video clips and playback statistics.
- [`epok::fmv::detail::fast_fill`](#epok-fmv-detail-fast-fill-1) — Performs `fast fill` as part of full-motion video clips and playback statistics.
- [`epok::fmv::divide_rounded`](#epok-fmv-divide-rounded-1) — Performs `divide rounded` as part of full-motion video clips and playback statistics.
- [`epok::fmv::quant_tables`](#epok-fmv-quant-tables-1) — Performs `quant tables` as part of full-motion video clips and playback statistics.
- [`epok::fmv::read_header`](#epok-fmv-read-header-1) — Reads header as part of full-motion video clips and playback statistics.
- [`epok::fmv::tv_dc`](#epok-fmv-tv-dc-1) — DC levels are a quarter of the block's mean pixel offset from 128.

<a id="epok-fmv-decode-frame-1"></a>

## `epok::fmv::decode_frame`

**Purpose.** Decodes one frame into `out`: the MDEC decode command followed by the run-length codes, padded with end-of-block codes to whole 32-word DMA blocks.

**Details.** `capacity` counts 32-bit words and must be at least 1 + the padded codes. `tv_levels` expands limited-range DC values; pair it with `tv_quant`.

**Exact declaration**

```cpp
inline BsResult decode_frame(const uint8_t *frame, uint32_t bytes, uint32_t width, uint32_t height, uint32_t *out, uint32_t capacity, bool tv_levels = false)
```

- **Declared at:** [line 290](../../../runtime/fmv_bitstream.hpp#L290)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `const uint8_t *` | Input | Value supplied for `frame`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |
| `width` | `uint32_t` | Input | Value supplied for `width`. See the exact type and module contract. |
| `height` | `uint32_t` | Input | Value supplied for `height`. See the exact type and module contract. |
| `out` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `out`. See the exact type and module contract. |
| `capacity` | `uint32_t` | Input | Value supplied for `capacity`. See the exact type and module contract. |
| `tv_levels` | `bool` | Input | Value supplied for `tv_levels`. See the exact type and module contract. |

**Returns.** Returns `BsResult`. Check the purpose and failure notes before using the value.

**Use it when.** `capacity` counts 32-bit words and must be at least 1 + the padded codes. `tv_levels` expands limited-range DC values; pair it with `tv_quant`.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * frame
// uint32_t bytes
// uint32_t width
// uint32_t height
// uint32_t * out
// uint32_t capacity
// bool tv_levels

auto result = epok::fmv::decode_frame(frame, bytes, width, height, out, capacity, tv_levels);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-detail-ac-build-1"></a>

## `epok::fmv::detail::ac_build`

**Purpose.** Performs `ac build` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr AcBuild ac_build()
```

- **Declared at:** [line 163](../../../runtime/fmv_bitstream.hpp#L163)
- **Kind:** `function decl`

**Returns.** Returns `AcBuild`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

auto result = epok::fmv::detail::ac_build();
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-ac-entry-1"></a>

## `epok::fmv::detail::ac_entry`

**Purpose.** Performs `ac entry` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr uint16_t ac_entry(uint32_t length, uint32_t run, uint32_t level)
```

- **Declared at:** [line 135](../../../runtime/fmv_bitstream.hpp#L135)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `length` | `uint32_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `run` | `uint32_t` | Input | Value supplied for `run`. See the exact type and module contract. |
| `level` | `uint32_t` | Input | Value supplied for `level`. See the exact type and module contract. |

**Returns.** Returns `uint16_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t length
// uint32_t run
// uint32_t level

auto result = epok::fmv::detail::ac_entry(length, run, level);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-ac-fill-1"></a>

## `epok::fmv::detail::ac_fill`

**Purpose.** Fills every table slot whose leading bits are `code`.

**Details.** Returns false when a slot is already taken, which would mean the code list is not prefix-free.

**Exact declaration**

```cpp
constexpr bool ac_fill(uint16_t *table, uint32_t index_bits, uint32_t code, uint32_t length, uint16_t entry)
```

- **Declared at:** [line 140](../../../runtime/fmv_bitstream.hpp#L140)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `uint16_t *` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `index_bits` | `uint32_t` | Input | Value supplied for `index_bits`. See the exact type and module contract. |
| `code` | `uint32_t` | Input | Value supplied for `code`. See the exact type and module contract. |
| `length` | `uint32_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `entry` | `uint16_t` | Input | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** Returns false when a slot is already taken, which would mean the code list is not prefix-free.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint16_t * table
// uint32_t index_bits
// uint32_t code
// uint32_t length
// uint16_t entry

auto result = epok::fmv::detail::ac_fill(table, index_bits, code, length, entry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-detail-bitreader-bitreader-1"></a>

## `epok::fmv::detail::BitReader::BitReader`

**Purpose.** Constructs `epok::fmv::detail::BitReader` for full-motion video clips and playback statistics.

**Exact declaration**

```cpp
BitReader(const uint8_t *data, uint32_t bytes)
```

- **Declared at:** [line 198](../../../runtime/fmv_bitstream.hpp#L198)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `data` | `const uint8_t *` | Input | Value supplied for `data`. See the exact type and module contract. |
| `bytes` | `uint32_t` | Input | Value supplied for `bytes`. See the exact type and module contract. |

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * data
// uint32_t bytes

epok::fmv::detail::BitReader value(data, bytes);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-detail-bitreader-overrun-1"></a>

## `epok::fmv::detail::BitReader::overrun`

**Purpose.** Performs `overrun` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
bool overrun() const
```

- **Declared at:** [line 204](../../../runtime/fmv_bitstream.hpp#L204)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

epok::fmv::detail::BitReader& object = /* obtain a valid instance */;

auto result = object.overrun();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-bitreader-peek16-1"></a>

## `epok::fmv::detail::BitReader::peek16`

**Purpose.** Performs `peek16` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
uint32_t peek16() const
```

- **Declared at:** [line 217](../../../runtime/fmv_bitstream.hpp#L217)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

epok::fmv::detail::BitReader& object = /* obtain a valid instance */;

auto result = object.peek16();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-bitreader-refill-1"></a>

## `epok::fmv::detail::BitReader::refill`

**Purpose.** Performs `refill` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
void refill()
```

- **Declared at:** [line 205](../../../runtime/fmv_bitstream.hpp#L205)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

epok::fmv::detail::BitReader& object = /* obtain a valid instance */;

object.refill();
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-bitreader-skip-1"></a>

## `epok::fmv::detail::BitReader::skip`

**Purpose.** Performs `skip` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
void skip(uint32_t bits)
```

- **Declared at:** [line 218](../../../runtime/fmv_bitstream.hpp#L218)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bits` | `uint32_t` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t bits

epok::fmv::detail::BitReader& object = /* obtain a valid instance */;

object.skip(bits);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-bitreader-take-1"></a>

## `epok::fmv::detail::BitReader::take`

**Purpose.** Performs `take` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
uint32_t take(uint32_t bits)
```

- **Declared at:** [line 223](../../../runtime/fmv_bitstream.hpp#L223)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bits` | `uint32_t` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** Returns `uint32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t bits

epok::fmv::detail::BitReader& object = /* obtain a valid instance */;

auto result = object.take(bits);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-dc-size-1"></a>

## `epok::fmv::detail::dc_size`

**Purpose.** MPEG-1 DC size: the number of differential bits that follow.

**Exact declaration**

```cpp
inline int32_t dc_size(BitReader &bits, bool luma)
```

- **Declared at:** [line 233](../../../runtime/fmv_bitstream.hpp#L233)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bits` | `BitReader &` | Input/output; inspect the function contract | Value supplied for `bits`. See the exact type and module contract. |
| `luma` | `bool` | Input | Value supplied for `luma`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// BitReader & bits
// bool luma

auto result = epok::fmv::detail::dc_size(bits, luma);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-detail-extend-1"></a>

## `epok::fmv::detail::extend`

**Purpose.** Performs `extend` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr int32_t extend(uint32_t value, uint32_t bits)
```

- **Declared at:** [line 229](../../../runtime/fmv_bitstream.hpp#L229)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `uint32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `bits` | `uint32_t` | Input | Value supplied for `bits`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t value
// uint32_t bits

auto result = epok::fmv::detail::extend(value, bits);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-detail-fast-fill-1"></a>

## `epok::fmv::detail::fast_fill`

**Purpose.** Performs `fast fill` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr bool fast_fill(uint32_t *table, uint32_t code, uint32_t length, uint32_t entry)
```

- **Declared at:** [line 150](../../../runtime/fmv_bitstream.hpp#L150)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `table` | `uint32_t *` | Input/output; inspect the function contract | Value supplied for `table`. See the exact type and module contract. |
| `code` | `uint32_t` | Input | Value supplied for `code`. See the exact type and module contract. |
| `length` | `uint32_t` | Input | Value supplied for `length`. See the exact type and module contract. |
| `entry` | `uint32_t` | Input | Value supplied for `entry`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// uint32_t * table
// uint32_t code
// uint32_t length
// uint32_t entry

auto result = epok::fmv::detail::fast_fill(table, code, length, entry);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-divide-rounded-1"></a>

## `epok::fmv::divide_rounded`

**Purpose.** Performs `divide rounded` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr int32_t divide_rounded(int32_t value, int32_t divisor)
```

- **Declared at:** [line 75](../../../runtime/fmv_bitstream.hpp#L75)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `value` | `int32_t` | Input | Value supplied for `value`. See the exact type and module contract. |
| `divisor` | `int32_t` | Input | Value supplied for `divisor`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// int32_t value
// int32_t divisor

auto result = epok::fmv::divide_rounded(value, divisor);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-quant-tables-1"></a>

## `epok::fmv::quant_tables`

**Purpose.** Performs `quant tables` as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
constexpr QuantTables quant_tables(bool tv_levels)
```

- **Declared at:** [line 64](../../../runtime/fmv_bitstream.hpp#L64)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `tv_levels` | `bool` | Input | Value supplied for `tv_levels`. See the exact type and module contract. |

**Returns.** Returns `QuantTables`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// bool tv_levels

auto result = epok::fmv::quant_tables(tv_levels);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.

<a id="epok-fmv-read-header-1"></a>

## `epok::fmv::read_header`

**Purpose.** Reads header as part of full-motion video clips and playback statistics.

**Exact declaration**

```cpp
inline BsHeader read_header(const uint8_t *frame)
```

- **Declared at:** [line 275](../../../runtime/fmv_bitstream.hpp#L275)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `frame` | `const uint8_t *` | Input | Value supplied for `frame`. See the exact type and module contract. |

**Returns.** Returns `BsHeader`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// const uint8_t * frame

auto result = epok::fmv::read_header(frame);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-fmv-tv-dc-1"></a>

## `epok::fmv::tv_dc`

**Purpose.** DC levels are a quarter of the block's mean pixel offset from 128.

**Exact declaration**

```cpp
constexpr int32_t tv_dc(int32_t dc, bool luma)
```

- **Declared at:** [line 79](../../../runtime/fmv_bitstream.hpp#L79)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `dc` | `int32_t` | Input | Value supplied for `dc`. See the exact type and module contract. |
| `luma` | `bool` | Input | Value supplied for `luma`. See the exact type and module contract. |

**Returns.** Returns `int32_t`. Check the purpose and failure notes before using the value.

**Use it when.** You need full-motion video clips and playback statistics and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "fmv_bitstream.hpp"

// Assume these named values have been initialized with valid data:
// int32_t dc
// bool luma

auto result = epok::fmv::tv_dc(dc, luma);
```

**Why choose it.** It provides direct, allocation-conscious access to full-motion video clips and playback statistics. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** This is classified as **Engine internal**. Prefer a higher-level Epok service unless you need this exact control.
