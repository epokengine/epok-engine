# PsyQo API: Application

> **Header:** `"psyqo/application.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh)

This module covers the application module. It documents 10 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::Application`, `psyqo::Scene`

## Callable index

- [`psyqo::Application::createScene`](#psyqo-application-createscene-1) — Create the root scene object.
- [`psyqo::Application::frame`](#psyqo-application-frame-1) — The frame method.
- [`psyqo::Application::getCurrentScene`](#psyqo-application-getcurrentscene-1) — Get the current scene object.
- [`psyqo::Application::gpu`](#psyqo-application-gpu-1) — Get the GPU object.
- [`psyqo::Application::popScene`](#psyqo-application-popscene-1) — Pop a scene object from the stack.
- [`psyqo::Application::prepare`](#psyqo-application-prepare-1) — Prepare the objects for the application
- [`psyqo::Application::pushScene`](#psyqo-application-pushscene-1) — Push a scene object onto the stack.
- [`psyqo::Application::run`](#psyqo-application-run-1) — Runs the main loop.
- [`psyqo::Application::start`](#psyqo-application-start-1) — Start the application.
- [`psyqo::Application::~Application`](#psyqo-application-application-1) — Releases the resources owned by `psyqo::Application`.

<a id="psyqo-application-createscene-1"></a>

## `psyqo::Application::createScene`

**Purpose.** Create the root scene object.

**Details.** This will be called once before the main loop. It should create the root scene object and push it onto the stack. This will only be called if the `frame` method of the `Application` class hasn't been overridden. If you override the `frame` method, you are responsible for managing your own scene system.

**Exact declaration**

```cpp
virtual void createScene()
```

- **Declared at:** [line 86](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L86)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will be called once before the main loop. It should create the root scene object and push it onto the stack. This will only be called if the `frame` method of the `Application` class hasn't been overridden. If you override the `frame` method, you are responsible for managing your own scene system.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

object.createScene();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-frame-1"></a>

## `psyqo::Application::frame`

**Purpose.** The frame method.

**Details.** The default implementation of this method will call the `createScene` method if the scene stack is empty, and then call the `frame` method of the current scene. This method is called once per frame, and should be used to update the application state. If you override this method, you are responsible for managing your own scene system.

**Exact declaration**

```cpp
virtual void frame()
```

- **Declared at:** [line 137](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L137)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** The default implementation of this method will call the `createScene` method if the scene stack is empty, and then call the `frame` method of the current scene. This method is called once per frame, and should be used to update the application state. If you override this method, you are responsible for managing your own scene system.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

object.frame();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-getcurrentscene-1"></a>

## `psyqo::Application::getCurrentScene`

**Purpose.** Get the current scene object.

**Details.** Returns the top scene object on the stack.

**Exact declaration**

```cpp
Scene* getCurrentScene()
```

- **Declared at:** [line 100](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L100)
- **Kind:** `cxx method`

**Returns.** Returns `Scene *`. Check the purpose and failure notes before using the value.

**Use it when.** Returns the top scene object on the stack.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

auto result = object.getCurrentScene();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-gpu-1"></a>

## `psyqo::Application::gpu`

**Purpose.** Get the GPU object.

**Details.** Simple accessor for the `GPU` object.

**Exact declaration**

```cpp
psyqo::GPU& gpu()
```

- **Declared at:** [line 93](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L93)
- **Kind:** `cxx method`

**Returns.** Returns `psyqo::GPU &`. Check the purpose and failure notes before using the value.

**Use it when.** Simple accessor for the `GPU` object.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

auto result = object.gpu();
```

**Why choose it.** The API maps closely to PSX GPU work, giving predictable ordering and low overhead.

**Trade-offs and warnings.** Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.

<a id="psyqo-application-popscene-1"></a>

## `psyqo::Application::popScene`

**Purpose.** Pop a scene object from the stack.

**Details.** Pops the top scene object from the stack. There can be only one active scene at a time. Popping a scene object will cause the current scene to be teared down and the new top scene, if any, to be started. If the scene stack ends up being empty, the `createScene` method will be called again. Calling this method when the stack is empty will return `nullptr`.

**Exact declaration**

```cpp
Scene* popScene()
```

- **Declared at:** [line 125](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L125)
- **Kind:** `cxx method`

**Returns.** the popped scene, potentially for deletion if needed.

**Use it when.** Pops the top scene object from the stack. There can be only one active scene at a time. Popping a scene object will cause the current scene to be teared down and the new top scene, if any, to be started. If the scene stack ends up being empty, the `createScene` method will be called again. Calling this method when the stack is empty will return `nullptr`.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

auto result = object.popScene();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-prepare-1"></a>

## `psyqo::Application::prepare`

**Purpose.** Prepare the objects for the application

**Details.** This will be called once before the main loop, and should be used to initialize any other objects necessary. Do not try to access any hardware resources during this call, as interrupts are disabled at this point.

**Exact declaration**

```cpp
virtual void prepare()
```

- **Declared at:** [line 66](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L66)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will be called once before the main loop, and should be used to initialize any other objects necessary. Do not try to access any hardware resources during this call, as interrupts are disabled at this point.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

object.prepare();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-pushscene-1"></a>

## `psyqo::Application::pushScene`

**Purpose.** Push a scene object onto the stack.

**Details.** Pushes a new scene object onto the stack. There can be only one active scene at a time. Pushing a scene object will cause the current scene, if any, to be teared down, and the new scene to be started.

**Exact declaration**

```cpp
void pushScene(Scene* scene)
```

- **Declared at:** [line 110](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L110)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `scene` | `Scene *` | Input/output; inspect the function contract | Value supplied for `scene`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** Pushes a new scene object onto the stack. There can be only one active scene at a time. Pushing a scene object will cause the current scene, if any, to be teared down, and the new scene to be started.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

// Assume these named values have been initialized with valid data:
// Scene * scene

psyqo::Application& object = /* obtain a valid instance */;

object.pushScene(scene);
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-application-run-1"></a>

## `psyqo::Application::run`

**Purpose.** Runs the main loop.

**Details.** Call this from the `main` function. It will never return.

**Exact declaration**

```cpp
int run()
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L56)
- **Kind:** `cxx method`

**Returns.** Returns `int`. Check the purpose and failure notes before using the value.

**Use it when.** Call this from the `main` function. It will never return.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

auto result = object.run();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-start-1"></a>

## `psyqo::Application::start`

**Purpose.** Start the application.

**Details.** This will be called once before the main loop, and after the `prepare` method. It should be used to initialize any hardware resources necessary, as interrupts are enabled.

**Exact declaration**

```cpp
virtual void start()
```

- **Declared at:** [line 75](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L75)
- **Kind:** `cxx method`; qualifiers: `virtual`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This will be called once before the main loop, and after the `prepare` method. It should be used to initialize any hardware resources necessary, as interrupts are enabled.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

psyqo::Application& object = /* obtain a valid instance */;

object.start();
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.

<a id="psyqo-application-application-1"></a>

## `psyqo::Application::~Application`

**Purpose.** Releases the resources owned by `psyqo::Application`.

**Exact declaration**

```cpp
virtual ~Application() = default
```

- **Declared at:** [line 139](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/application.hh#L139)
- **Kind:** `destructor`; qualifiers: `virtual`

**Use it when.** You need the application module and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/application.hh"

// `psyqo::Application` cleans up when its owning scope ends.
```

**Why choose it.** It provides direct, allocation-conscious access to the application module. No exception-based error path is implied by the signature.

**Trade-offs and warnings.** Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.
