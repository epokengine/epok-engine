# PsyQo API: Scene

> **Header:** `"psyqo/scene.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/scene.hh)

This module covers scene lifecycle and scene-stack control. It documents 4 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::Scene`, `psyqo::Scene::StartReason`, `psyqo::Scene::TearDownReason`

## Callable index

- [`psyqo::Scene::frame`](#psyqo-scene-frame-1) — Renders a frame.
- [`psyqo::Scene::start`](#psyqo-scene-start-1) — Starts the scene.
- [`psyqo::Scene::teardown`](#psyqo-scene-teardown-1) — Tears down the scene.
- [`psyqo::Scene::~Scene`](#psyqo-scene-scene-1) — Releases the resources owned by `psyqo::Scene`.

<a id="psyqo-scene-frame-1"></a>

## `psyqo::Scene::frame`

**Purpose.** Renders a frame.

**Details.** This method will be called when the scene is active, every time a new frame is to be rendered.

**Exact declaration**

```cpp
virtual void frame()
```

- **Declared at:** [line 65](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/scene.hh#L65)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will be called when the scene is active, every time a new frame is to be rendered.

**Usage pattern**

```cpp
#include "psyqo/scene.hh"

psyqo::Scene& object = /* obtain a valid instance */;

object.frame();
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-scene-start-1"></a>

## `psyqo::Scene::start`

**Purpose.** Starts the scene.

**Details.** This method will be called when the scene is started. It is meant to set the environment in a suitable manner. A scene starts when it becomes the active scene, either when being pushed or when the previous scene is popped. The argument will indicate whether the scene is started because it just got pushed, or because another one is getting popped.

**Exact declaration**

```cpp
virtual void start(StartReason reason)
```

- **Declared at:** [line 57](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/scene.hh#L57)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reason` | `StartReason` | Input | The reason why the scene is started. Create or Resume. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will be called when the scene is started. It is meant to set the environment in a suitable manner. A scene starts when it becomes the active scene, either when being pushed or when the previous scene is popped. The argument will indicate whether the scene is started because it just got pushed, or because another one is getting popped.

**Usage pattern**

```cpp
#include "psyqo/scene.hh"

// Assume these named values have been initialized with valid data:
// StartReason reason

psyqo::Scene& object = /* obtain a valid instance */;

object.start(reason);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-scene-teardown-1"></a>

## `psyqo::Scene::teardown`

**Purpose.** Tears down the scene.

**Details.** This method will be called when the scene is no longer the active scene. It is meant to clean up the environment, basically reversing the effects of `start`. The argument will indicate whether the scene is being popped, or if another scene is pushed on the stack.

**Exact declaration**

```cpp
virtual void teardown(TearDownReason reason)
```

- **Declared at:** [line 77](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/scene.hh#L77)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `reason` | `TearDownReason` | Input | The reason why the scene is being torn down. DESTROY or PAUSE. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will be called when the scene is no longer the active scene. It is meant to clean up the environment, basically reversing the effects of `start`. The argument will indicate whether the scene is being popped, or if another scene is pushed on the stack.

**Usage pattern**

```cpp
#include "psyqo/scene.hh"

// Assume these named values have been initialized with valid data:
// TearDownReason reason

psyqo::Scene& object = /* obtain a valid instance */;

object.teardown(reason);
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-scene-scene-1"></a>

## `psyqo::Scene::~Scene`

**Purpose.** Releases the resources owned by `psyqo::Scene`.

**Exact declaration**

```cpp
virtual ~Scene() = default
```

- **Declared at:** [line 79](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/scene.hh#L79)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need scene lifecycle and scene-stack control and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/scene.hh"

// `psyqo::Scene` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to scene lifecycle and scene-stack control. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
