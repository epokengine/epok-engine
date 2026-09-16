# Epok API: Instrument Preparation

> **Header:** `"instrument_preparation.hpp"` · **Tier:** Epok runtime API · **Source:** [open header](../../../runtime/instrument_preparation.hpp)

This module covers the instrument preparation module. It documents 13 public callables declared directly in this header.

## Declared types

`epok::instrument::preparation::Cache`, `epok::instrument::preparation::EventStart`, `epok::instrument::preparation::Key`, `epok::instrument::preparation::Storage`

## Callable index

- [`epok::instrument::preparation::bucket_count`](#epok-instrument-preparation-bucket-count-1) — Performs `bucket count` as part of the instrument preparation module.
- [`epok::instrument::preparation::Cache::Cache`](#epok-instrument-preparation-cache-cache-1) — Constructs `epok::instrument::preparation::Cache` for the instrument preparation module.
- [`epok::instrument::preparation::Cache::find`](#epok-instrument-preparation-cache-find-1) — Finds find as part of the instrument preparation module.
- [`epok::instrument::preparation::Cache::insert`](#epok-instrument-preparation-cache-insert-1) — Performs `insert` as part of the instrument preparation module.
- [`epok::instrument::preparation::Cache::prepare`](#epok-instrument-preparation-cache-prepare-1) — Performs `prepare` as part of the instrument preparation module.
- [`epok::instrument::preparation::Cache::slot`](#epok-instrument-preparation-cache-slot-1) — Performs `slot` as part of the instrument preparation module.
- [`epok::instrument::preparation::controls`](#epok-instrument-preparation-controls-1) — Performs `controls` as part of the instrument preparation module.
- [`epok::instrument::preparation::initial_pitch`](#epok-instrument-preparation-initial-pitch-1) — Exact initial SPU pitch for the common AudioSource pitch of 1.0.
- [`epok::instrument::preparation::Key::hash`](#epok-instrument-preparation-key-hash-1) — Reports whether h as part of the instrument preparation module.
- [`epok::instrument::preparation::Key::Key`](#epok-instrument-preparation-key-key-1) — Constructs `epok::instrument::preparation::Key` for the instrument preparation module.
- [`epok::instrument::preparation::Key::Key`](#epok-instrument-preparation-key-key-2) — Constructs `epok::instrument::preparation::Key` for the instrument preparation module.
- [`epok::instrument::preparation::Key::operator==`](#epok-instrument-preparation-key-operator-1) — Performs `operator ==` as part of the instrument preparation module.
- [`epok::instrument::preparation::Storage::Storage<Capacity, Events, References>`](#epok-instrument-preparation-storage-storage-capacity-events-references-1) — Constructs `epok::instrument::preparation::Storage` for the instrument preparation module.

<a id="epok-instrument-preparation-bucket-count-1"></a>

## `epok::instrument::preparation::bucket_count`

**Purpose.** Performs `bucket count` as part of the instrument preparation module.

**Exact declaration**

```cpp
inline constexpr uint16_t bucket_count(uint16_t capacity)
```

- **Declared at:** [line 63](../../../runtime/instrument_preparation.hpp#L63)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `capacity` | `int` | Input | Value supplied for `capacity`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// int capacity

auto result = epok::instrument::preparation::bucket_count(capacity);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-preparation-cache-cache-1"></a>

## `epok::instrument::preparation::Cache::Cache`

**Purpose.** Constructs `epok::instrument::preparation::Cache` for the instrument preparation module.

**Exact declaration**

```cpp
Cache(Key* k,synth::State* s,uint16_t* b,uint16_t n,uint16_t bs):key
```

- **Declared at:** [line 77](../../../runtime/instrument_preparation.hpp#L77)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `k` | `Key *` | Input/output; inspect the function contract | Value supplied for `k`. See the exact type and module contract. |
| `s` | `synth::State *` | Input/output; inspect the function contract | Value supplied for `s`. See the exact type and module contract. |
| `b` | `int *` | Input/output; inspect the function contract | Value supplied for `b`. See the exact type and module contract. |
| `n` | `int` | Input | Value supplied for `n`. See the exact type and module contract. |
| `bs` | `int` | Input | Value supplied for `bs`. See the exact type and module contract. |

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// Key * k
// synth::State * s
// int * b
// int n
// int bs

epok::instrument::preparation::Cache value(k, s, b, n, bs);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-cache-find-1"></a>

## `epok::instrument::preparation::Cache::find`

**Purpose.** Finds find as part of the instrument preparation module.

**Exact declaration**

```cpp
const synth::State* find(uint16_t zone,uint8_t note,uint8_t velocity,const sequence::Channel& channel)const
```

- **Declared at:** [line 86](../../../runtime/instrument_preparation.hpp#L86)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `zone` | `int` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |
| `velocity` | `int` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |

**Returns.** Returns `const synth::State *`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// int zone
// int note
// int velocity
// const sequence::Channel & channel

epok::instrument::preparation::Cache& object = /* obtain a valid instance */;

auto result = object.find(zone, note, velocity, channel);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-cache-insert-1"></a>

## `epok::instrument::preparation::Cache::insert`

**Purpose.** Performs `insert` as part of the instrument preparation module.

**Exact declaration**

```cpp
uint16_t insert(const BankView& bank,uint16_t zone,const sequence::Note& note,const sequence::Channel& channel)
```

- **Declared at:** [line 91](../../../runtime/instrument_preparation.hpp#L91)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `zone` | `int` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `note` | `const sequence::Note &` | Input | Value supplied for `note`. See the exact type and module contract. |
| `channel` | `const sequence::Channel &` | Input | Value supplied for `channel`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// int zone
// const sequence::Note & note
// const sequence::Channel & channel

epok::instrument::preparation::Cache& object = /* obtain a valid instance */;

auto result = object.insert(bank, zone, note, channel);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-cache-prepare-1"></a>

## `epok::instrument::preparation::Cache::prepare`

**Purpose.** Performs `prepare` as part of the instrument preparation module.

**Exact declaration**

```cpp
bool prepare(const BankView& bank,const sequence::Event* events,uint32_t size,uint16_t ppqn)
```

- **Declared at:** [line 101](../../../runtime/instrument_preparation.hpp#L101)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `events` | `const sequence::Event *` | Input | Value supplied for `events`. See the exact type and module contract. |
| `size` | `int` | Input | Value supplied for `size`. See the exact type and module contract. |
| `ppqn` | `int` | Input | Value supplied for `ppqn`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// const sequence::Event * events
// int size
// int ppqn

epok::instrument::preparation::Cache& object = /* obtain a valid instance */;

auto result = object.prepare(bank, events, size, ppqn);
```

**Why choose it.** The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-cache-slot-1"></a>

## `epok::instrument::preparation::Cache::slot`

**Purpose.** Performs `slot` as part of the instrument preparation module.

**Exact declaration**

```cpp
uint16_t slot(const Key& key)const
```

- **Declared at:** [line 78](../../../runtime/instrument_preparation.hpp#L78)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `key` | `const Key &` | Input | Value supplied for `key`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const Key & key

epok::instrument::preparation::Cache& object = /* obtain a valid instance */;

auto result = object.slot(key);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-controls-1"></a>

## `epok::instrument::preparation::controls`

**Purpose.** Performs `controls` as part of the instrument preparation module.

**Exact declaration**

```cpp
inline synth::Controls controls(const sequence::Channel& c)
```

- **Declared at:** [line 32](../../../runtime/instrument_preparation.hpp#L32)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `c` | `const sequence::Channel &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Returns.** Returns `synth::Controls`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const sequence::Channel & c

auto result = epok::instrument::preparation::controls(c);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-initial-pitch-1"></a>

## `epok::instrument::preparation::initial_pitch`

**Purpose.** Exact initial SPU pitch for the common AudioSource pitch of 1.0.

**Details.** Compute it before the clock starts; other source pitches retain the live calculation.

**Exact declaration**

```cpp
inline uint16_t initial_pitch(const BankView& bank,uint16_t zone,const synth::State& state)
```

- **Declared at:** [line 13](../../../runtime/instrument_preparation.hpp#L13)
- **Kind:** `function decl`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `bank` | `const BankView &` | Input | Value supplied for `bank`. See the exact type and module contract. |
| `zone` | `int` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `state` | `const synth::State &` | Input | Value supplied for `state`. See the exact type and module contract. |

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Compute it before the clock starts; other source pitches retain the live calculation.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const BankView & bank
// int zone
// const synth::State & state

auto result = epok::instrument::preparation::initial_pitch(bank, zone, state);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-key-hash-1"></a>

## `epok::instrument::preparation::Key::hash`

**Purpose.** Reports whether h as part of the instrument preparation module.

**Exact declaration**

```cpp
uint32_t hash()const
```

- **Declared at:** [line 56](../../../runtime/instrument_preparation.hpp#L56)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

epok::instrument::preparation::Key& object = /* obtain a valid instance */;

auto result = object.hash();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-preparation-key-key-1"></a>

## `epok::instrument::preparation::Key::Key`

**Purpose.** Constructs `epok::instrument::preparation::Key` for the instrument preparation module.

**Exact declaration**

```cpp
Key()=default
```

- **Declared at:** [line 42](../../../runtime/instrument_preparation.hpp#L42)
- **Kind:** `constructor`

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

epok::instrument::preparation::Key value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="epok-instrument-preparation-key-key-2"></a>

## `epok::instrument::preparation::Key::Key`

**Purpose.** Constructs `epok::instrument::preparation::Key` for the instrument preparation module.

**Exact declaration**

```cpp
Key(uint16_t zone,uint8_t note,uint8_t velocity,const sequence::Channel& c)
```

- **Declared at:** [line 43](../../../runtime/instrument_preparation.hpp#L43)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `zone` | `int` | Input | Value supplied for `zone`. See the exact type and module contract. |
| `note` | `int` | Input | Value supplied for `note`. See the exact type and module contract. |
| `velocity` | `int` | Input | Value supplied for `velocity`. See the exact type and module contract. |
| `c` | `const sequence::Channel &` | Input | Value supplied for `c`. See the exact type and module contract. |

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// int zone
// int note
// int velocity
// const sequence::Channel & c

epok::instrument::preparation::Key value(zone, note, velocity, c);
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-key-operator-1"></a>

## `epok::instrument::preparation::Key::operator==`

**Purpose.** Performs `operator ==` as part of the instrument preparation module.

**Exact declaration**

```cpp
bool operator==(const Key& rhs)const
```

- **Declared at:** [line 52](../../../runtime/instrument_preparation.hpp#L52)
- **Kind:** `cxx method`; qualifiers: `const`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `rhs` | `const Key &` | Input | Value supplied for `rhs`. See the exact type and module contract. |

**Returns.** Returns `bool`. Check the purpose and failure notes before using the value.

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

// Assume these named values have been initialized with valid data:
// const Key & rhs

epok::instrument::preparation::Key& object = /* obtain a valid instance */;

auto result = object.operator==(rhs);
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="epok-instrument-preparation-storage-storage-capacity-events-references-1"></a>

## `epok::instrument::preparation::Storage::Storage<Capacity, Events, References>`

**Purpose.** Constructs `epok::instrument::preparation::Storage` for the instrument preparation module.

**Exact declaration**

```cpp
Storage()
```

- **Declared at:** [line 157](../../../runtime/instrument_preparation.hpp#L157)
- **Kind:** `constructor`

**Use it when.** You need the instrument preparation module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "instrument_preparation.hpp"

epok::instrument::preparation::Storage value();
```

**Why choose it.** It provides direct, allocation-conscious access to the instrument preparation module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
