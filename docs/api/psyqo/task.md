# PsyQo API: Task

> **Header:** `"psyqo/task.hh"` · **Tier:** Pinned PsyQo API · **Source:** [open header](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh)

This module covers cooperative asynchronous tasks and callbacks. It documents 19 public callables declared directly in this header.

PsyQo is pinned through Nugget revision `6186b131aacc5853a9161fb076ed34ffe504552d`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.

## Declared types

`psyqo::GPU`, `psyqo::TaskQueue`, `psyqo::TaskQueue::Task`

## Callable index

- [`psyqo::TaskQueue::butCatch`](#psyqo-taskqueue-butcatch-1) — Sets the exception handler.
- [`psyqo::TaskQueue::DelayedTask`](#psyqo-taskqueue-delayedtask-1) — Creates a delayed task.
- [`psyqo::TaskQueue::finally`](#psyqo-taskqueue-finally-1) — Sets the finally handler.
- [`psyqo::TaskQueue::isRunning`](#psyqo-taskqueue-isrunning-1) — Queries the status of the queue.
- [`psyqo::TaskQueue::reset`](#psyqo-taskqueue-reset-1) — Resets the queue.
- [`psyqo::TaskQueue::run`](#psyqo-taskqueue-run-1) — Runs the task queue.
- [`psyqo::TaskQueue::schedule`](#psyqo-taskqueue-schedule-1) — Schedules the task queue to another task queue.
- [`psyqo::TaskQueue::startWith`](#psyqo-taskqueue-startwith-1) — Enqueues a task for execution.
- [`psyqo::TaskQueue::startWith`](#psyqo-taskqueue-startwith-2) — Starts with as part of cooperative asynchronous tasks and callbacks.
- [`psyqo::TaskQueue::Task::complete`](#psyqo-taskqueue-task-complete-1) — Resolves or rejects this task.
- [`psyqo::TaskQueue::Task::operator=`](#psyqo-taskqueue-task-operator-1) — Performs `operator =` as part of cooperative asynchronous tasks and callbacks.
- [`psyqo::TaskQueue::Task::operator=`](#psyqo-taskqueue-task-operator-2) — Performs `operator =` as part of cooperative asynchronous tasks and callbacks.
- [`psyqo::TaskQueue::Task::reject`](#psyqo-taskqueue-task-reject-1) — Rejects this task.
- [`psyqo::TaskQueue::Task::resolve`](#psyqo-taskqueue-task-resolve-1) — Resolves this task.
- [`psyqo::TaskQueue::Task::Task`](#psyqo-taskqueue-task-task-1) — Constructs `psyqo::TaskQueue::Task` for cooperative asynchronous tasks and callbacks.
- [`psyqo::TaskQueue::Task::Task`](#psyqo-taskqueue-task-task-2) — Constructs `psyqo::TaskQueue::Task` for cooperative asynchronous tasks and callbacks.
- [`psyqo::TaskQueue::Task::Task`](#psyqo-taskqueue-task-task-3) — Construct a new Task object
- [`psyqo::TaskQueue::then`](#psyqo-taskqueue-then-1) — Enqueues a task for execution.
- [`psyqo::TaskQueue::then`](#psyqo-taskqueue-then-2) — Performs `then` as part of cooperative asynchronous tasks and callbacks.

<a id="psyqo-taskqueue-butcatch-1"></a>

## `psyqo::TaskQueue::butCatch`

**Purpose.** Sets the exception handler.

**Details.** This method will set the exception handler. The exception handler will be called if any of the tasks in the queue throws an exception. The exception handler will be called with the task queue as its argument. Calling this while the queue is running is undefined behavior.

**Exact declaration**

```cpp
TaskQueue &butCatch(eastl::function<void(TaskQueue *)> &&)
```

- **Declared at:** [line 91](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L91)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `eastl::function<void (TaskQueue *)> &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** This method will set the exception handler. The exception handler will be called if any of the tasks in the queue throws an exception. The exception handler will be called with the task queue as its argument. Calling this while the queue is running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (TaskQueue *)> && arg1

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.butCatch(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-delayedtask-1"></a>

## `psyqo::TaskQueue::DelayedTask`

**Purpose.** Creates a delayed task.

**Details.** This method is a convenience method to create a task that will be executed after a delay. The delay is specified in microseconds.

**Exact declaration**

```cpp
static Task DelayedTask(uint32_t delay, GPU &)
```

- **Declared at:** [line 205](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L205)
- **Kind:** `cxx method`; qualifiers: `static`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `delay` | `uint32_t` | Input | The delay in microseconds. |
| `arg2` | `GPU &` | Input/output; inspect the function contract | Value supplied for `arg2`. See the exact type and module contract. |

**Returns.** Returns `Task`. Check the purpose and failure notes before using the value.

**Use it when.** This method is a convenience method to create a task that will be executed after a delay. The delay is specified in microseconds.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// uint32_t delay
// GPU & arg2

auto result = psyqo::TaskQueue::DelayedTask(delay, arg2);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-finally-1"></a>

## `psyqo::TaskQueue::finally`

**Purpose.** Sets the finally handler.

**Details.** This method will set the finally handler. The finally handler will be called after all tasks in the queue have been executed, or after the exception handler. The finally handler will be called with the task queue as its argument. Calling this while the queue is running is undefined behavior.

**Exact declaration**

```cpp
TaskQueue &finally(eastl::function<void(TaskQueue *)> &&)
```

- **Declared at:** [line 103](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L103)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `eastl::function<void (TaskQueue *)> &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** This method will set the finally handler. The finally handler will be called after all tasks in the queue have been executed, or after the exception handler. The finally handler will be called with the task queue as its argument. Calling this while the queue is running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (TaskQueue *)> && arg1

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.finally(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-isrunning-1"></a>

## `psyqo::TaskQueue::isRunning`

**Purpose.** Queries the status of the queue.

**Exact declaration**

```cpp
bool isRunning() const
```

- **Declared at:** [line 130](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L130)
- **Kind:** `cxx method`; qualifiers: `const`

**Returns.** true if the queue is running, false otherwise.

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.isRunning();
```

**Why choose it.** The method is `const`, so it does not mutate the object through this API surface. The boolean result makes success, availability or state explicit without exceptions. The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Check the return value; `false` is part of normal control flow for many PSX resource operations. Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-reset-1"></a>

## `psyqo::TaskQueue::reset`

**Purpose.** Resets the queue.

**Details.** This method resets the queue to its initial state. Calling this method while the queue is running is undefined behavior.

**Exact declaration**

```cpp
void reset()
```

- **Declared at:** [line 56](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L56)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method resets the queue to its initial state. Calling this method while the queue is running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue& object = /* obtain a valid instance */;

object.reset();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-run-1"></a>

## `psyqo::TaskQueue::run`

**Purpose.** Runs the task queue.

**Details.** This method will start running the queue. The queue will continue to run until all tasks have been executed, or until an exception is thrown. This method can be called multiple times, in order to execute the whole queue multiple times. Calling this while the queue is already running is undefined behavior.

**Exact declaration**

```cpp
void run()
```

- **Declared at:** [line 115](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L115)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method will start running the queue. The queue will continue to run until all tasks have been executed, or until an exception is thrown. This method can be called multiple times, in order to execute the whole queue multiple times. Calling this while the queue is already running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue& object = /* obtain a valid instance */;

object.run();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-schedule-1"></a>

## `psyqo::TaskQueue::schedule`

**Purpose.** Schedules the task queue to another task queue.

**Details.** This method will enable embedding the queue into another one. Exceptions will be cascading.

**Exact declaration**

```cpp
Task schedule()
```

- **Declared at:** [line 123](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L123)
- **Kind:** `cxx method`

**Returns.** Returns `Task`. Check the purpose and failure notes before using the value.

**Use it when.** This method will enable embedding the queue into another one. Exceptions will be cascading.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.schedule();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-startwith-1"></a>

## `psyqo::TaskQueue::startWith`

**Purpose.** Enqueues a task for execution.

**Details.** This method will enqueue a task for execution, while resetting the queue first. This means that any previously enqueued tasks will be removed first. Any exception handler or finally handler will also be cleared out. Calling this while the queue is running is undefined behavior.

**Exact declaration**

```cpp
TaskQueue &startWith(Task &&)
```

- **Declared at:** [line 68](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L68)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Task &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** This method will enqueue a task for execution, while resetting the queue first. This means that any previously enqueued tasks will be removed first. Any exception handler or finally handler will also be cleared out. Calling this while the queue is running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// Task && arg1

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.startWith(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-startwith-2"></a>

## `psyqo::TaskQueue::startWith`

**Purpose.** Starts with as part of cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
TaskQueue &startWith(eastl::function<void(Task *)> &&fun)
```

- **Declared at:** [line 69](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L69)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fun` | `eastl::function<void (Task *)> &&` | Consumed or moved input | Value supplied for `fun`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Task *)> && fun

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.startWith(fun);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-task-complete-1"></a>

## `psyqo::TaskQueue::Task::complete`

**Purpose.** Resolves or rejects this task.

**Details.** This method is to be called by the task's lambda function to resolve or reject the task. It is a convenience method that will either call resolve() or reject() depending on the value of the first argument.

**Exact declaration**

```cpp
void complete(bool success)
```

- **Declared at:** [line 183](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L183)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `success` | `bool` | Input | Value supplied for `success`. See the exact type and module contract. |

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method is to be called by the task's lambda function to resolve or reject the task. It is a convenience method that will either call resolve() or reject() depending on the value of the first argument.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// bool success

psyqo::TaskQueue::Task& object = /* obtain a valid instance */;

object.complete(success);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-task-operator-1"></a>

## `psyqo::TaskQueue::Task::operator=`

**Purpose.** Performs `operator =` as part of cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
Task &operator=(Task &&) = default
```

- **Declared at:** [line 151](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L151)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Task &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Task &`. Check the purpose and failure notes before using the value.

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// Task && arg1

psyqo::TaskQueue::Task& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-task-operator-2"></a>

## `psyqo::TaskQueue::Task::operator=`

**Purpose.** Performs `operator =` as part of cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
Task &operator=(const Task &) = delete
```

- **Declared at:** [line 152](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L152)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Task &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `Task &`. Check the purpose and failure notes before using the value.

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// const Task & arg1

psyqo::TaskQueue::Task& object = /* obtain a valid instance */;

auto result = object.operator=(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-task-reject-1"></a>

## `psyqo::TaskQueue::Task::reject`

**Purpose.** Rejects this task.

**Details.** This method is to be called by the task's lambda function to reject the task. It will call the exception handler, and then the finally handler.

**Exact declaration**

```cpp
void reject()
```

- **Declared at:** [line 172](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L172)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method is to be called by the task's lambda function to reject the task. It will call the exception handler, and then the finally handler.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue::Task& object = /* obtain a valid instance */;

object.reject();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-task-resolve-1"></a>

## `psyqo::TaskQueue::Task::resolve`

**Purpose.** Resolves this task.

**Details.** This method is to be called by the task's lambda function to resolve the task. It will continue to the next task in the queue, or call the finally handler if there are no more tasks.

**Exact declaration**

```cpp
void resolve()
```

- **Declared at:** [line 162](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L162)
- **Kind:** `cxx method`

**Returns.** No value is returned; observe the documented state change or callback.

**Use it when.** This method is to be called by the task's lambda function to resolve the task. It will continue to the next task in the queue, or call the finally handler if there are no more tasks.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

psyqo::TaskQueue::Task& object = /* obtain a valid instance */;

object.resolve();
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed.

<a id="psyqo-taskqueue-task-task-1"></a>

## `psyqo::TaskQueue::Task::Task`

**Purpose.** Constructs `psyqo::TaskQueue::Task` for cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
Task(Task &&) = default
```

- **Declared at:** [line 149](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L149)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Task &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// Task && arg1

psyqo::TaskQueue::Task value(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-task-task-2"></a>

## `psyqo::TaskQueue::Task::Task`

**Purpose.** Constructs `psyqo::TaskQueue::Task` for cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
Task(const Task &) = delete
```

- **Declared at:** [line 150](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L150)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `const Task &` | Input | Value supplied for `arg1`. See the exact type and module contract. |

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// const Task & arg1

psyqo::TaskQueue::Task value(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-task-task-3"></a>

## `psyqo::TaskQueue::Task::Task`

**Purpose.** Construct a new Task object

**Exact declaration**

```cpp
explicit Task(eastl::function<void(Task *)> &&fun) : m_run
```

- **Declared at:** [line 148](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L148)
- **Kind:** `constructor`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fun` | `eastl::function<void (Task *)> &&` | Consumed or moved input | The lambda to execute for this task. It will receive the task as its argument. |

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Task *)> && fun

psyqo::TaskQueue::Task value(fun);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-then-1"></a>

## `psyqo::TaskQueue::then`

**Purpose.** Enqueues a task for execution.

**Details.** This method will enqueue a task for execution. The task will be executed after all previously enqueued tasks have been executed. Calling this while the queue is running is undefined behavior.

**Exact declaration**

```cpp
TaskQueue &then(Task &&)
```

- **Declared at:** [line 78](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L78)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `arg1` | `Task &&` | Consumed or moved input | Value supplied for `arg1`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** This method will enqueue a task for execution. The task will be executed after all previously enqueued tasks have been executed. Calling this while the queue is running is undefined behavior.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// Task && arg1

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.then(arg1);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.

<a id="psyqo-taskqueue-then-2"></a>

## `psyqo::TaskQueue::then`

**Purpose.** Performs `then` as part of cooperative asynchronous tasks and callbacks.

**Exact declaration**

```cpp
TaskQueue &then(eastl::function<void(Task *)> &&fun)
```

- **Declared at:** [line 79](https://github.com/pcsx-redux/nugget/blob/6186b131aacc5853a9161fb076ed34ffe504552d/psyqo/task.hh#L79)
- **Kind:** `cxx method`

**Parameters**

| Name | Type | Role | Meaning |
| --- | --- | --- | --- |
| `fun` | `eastl::function<void (Task *)> &&` | Consumed or moved input | Value supplied for `fun`. See the exact type and module contract. |

**Returns.** Returns `TaskQueue &`. Check the purpose and failure notes before using the value.

**Use it when.** You need cooperative asynchronous tasks and callbacks and the preconditions in the declaration are already satisfied.

**Usage pattern**

```cpp
#include "psyqo/task.hh"

// Assume these named values have been initialized with valid data:
// eastl::function<void (Task *)> && fun

psyqo::TaskQueue& object = /* obtain a valid instance */;

auto result = object.then(fun);
```

**Why choose it.** The asynchronous shape lets the frame loop continue while hardware or queued work completes.

**Trade-offs and warnings.** Captured data and buffers must remain valid until the callback or task has completed. Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.
