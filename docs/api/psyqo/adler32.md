# PsyQo API: Adler32

> **Header:** `"psyqo/adler32.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/adler32.hh)

This module covers the adler32 module. It documents 3 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Callable index

- [`psyqo::adler32`](#psyqo-adler32-1) — Computes the adler32 checksum of a buffer.
- [`psyqo::adler32_bytes`](#psyqo-adler32-bytes-1) — Computes the adler32 checksum of a buffer, only reading bytes.
- [`psyqo::adler32_words`](#psyqo-adler32-words-1) — Computes the adler32 checksum of a buffer, optimized for words.

<a id="psyqo-adler32-1"></a>

## `psyqo::adler32`

**Purpose.** Computes the adler32 checksum of a buffer.

**Details.** This is a very fast checksum algorithm, but it is not cryptographically secure. It can be used to detect data corruption. It is possible to chunk large buffers by chaining the checksum of the previous chunk with the checksum of the next using the `sum` parameter. As an example, here is how to compute the checksum of a 1MB buffer fragmented over 1024 chunks of 1024 bytes: @code {.language-id=cpp} uint32_t checksum = adler32(nullptr, 0); for (unsigned i = 0; i < 1024; i++) { checksum = adler32(buffer + i * 1024, 1024, checksum); }

**Exact declaration**

```cpp
uint32_t adler32(uint8_t* buffer, unsigned length, uint32_t sum = 1)
```

- **Declared at:** [line 53](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/adler32.hh#L53)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buffer` | `uint8_t *` | Input/output; inspect the function contract | The buffer to checksum. |
| `length` | `unsigned int` | Input | Value supplied for `length`. See the exact type and module contract. |
| `sum` | `uint32_t` | Input | The previous sum to continue the checksum for. |

**Returns.** The adler32 checksum of the buffer.

**Use it when.** This is a very fast checksum algorithm, but it is not cryptographically secure. It can be used to detect data corruption. It is possible to chunk large buffers by chaining the checksum of the previous chunk with the checksum of the next using the `sum` parameter. As an example, here is how to compute the checksum of a 1MB buffer fragmented over 1024 chunks of 1024 bytes: @code {.language-id=cpp} uint32_t checksum = adler32(nullptr, 0); for (unsigned i = 0; i < 1024; i++) { checksum = adler32(buffer + i * 1024, 1024, checksum); }

**Usage pattern**

```cpp
#include "psyqo/adler32.hh"

// Assume these named values have been initialized with valid data:
// uint8_t * buffer
// unsigned int length
// uint32_t sum

auto result = psyqo::adler32(buffer, length, sum);
```

**Why choose it.** It provides direct, allocation-conscious access to the adler32 module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-adler32-bytes-1"></a>

## `psyqo::adler32_bytes`

**Purpose.** Computes the adler32 checksum of a buffer, only reading bytes.

**Details.** This is a variant of the `adler32` function, which works exclusively on bytes.

**Exact declaration**

```cpp
uint32_t adler32_bytes(uint8_t* buffer, unsigned length, uint32_t sum = 1)
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/adler32.hh#L66)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buffer` | `uint8_t *` | Input/output; inspect the function contract | The buffer to checksum. |
| `length` | `unsigned int` | Input | Value supplied for `length`. See the exact type and module contract. |
| `sum` | `uint32_t` | Input | The previous sum to continue the checksum for. |

**Returns.** The adler32 checksum of the buffer.

**Use it when.** This is a variant of the `adler32` function, which works exclusively on bytes.

**Usage pattern**

```cpp
#include "psyqo/adler32.hh"

// Assume these named values have been initialized with valid data:
// uint8_t * buffer
// unsigned int length
// uint32_t sum

auto result = psyqo::adler32_bytes(buffer, length, sum);
```

**Why choose it.** It provides direct, allocation-conscious access to the adler32 module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-adler32-words-1"></a>

## `psyqo::adler32_words`

**Purpose.** Computes the adler32 checksum of a buffer, optimized for words.

**Details.** This is a variant of the `adler32` function, which works exclusively on words. It is faster than the byte-oriented version, but the buffer needs to be aligned to a word boundary. It is possible to mix the two versions by using the `sum` parameter as explained in the documentation of the byte-oriented version.

**Exact declaration**

```cpp
uint32_t adler32_words(uint32_t* buffer, unsigned length, uint32_t sum = 1)
```

- **Declared at:** [line 82](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/adler32.hh#L82)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `buffer` | `uint32_t *` | Input/output; inspect the function contract | The buffer to checksum. |
| `length` | `unsigned int` | Input | Value supplied for `length`. See the exact type and module contract. |
| `sum` | `uint32_t` | Input | The previous sum to continue the checksum for. |

**Returns.** The adler32 checksum of the buffer.

**Use it when.** This is a variant of the `adler32` function, which works exclusively on words. It is faster than the byte-oriented version, but the buffer needs to be aligned to a word boundary. It is possible to mix the two versions by using the `sum` parameter as explained in the documentation of the byte-oriented version.

**Usage pattern**

```cpp
#include "psyqo/adler32.hh"

// Assume these named values have been initialized with valid data:
// uint32_t * buffer
// unsigned int length
// uint32_t sum

auto result = psyqo::adler32_words(buffer, length, sum);
```

**Why choose it.** It provides direct, allocation-conscious access to the adler32 module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
