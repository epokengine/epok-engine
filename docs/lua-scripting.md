# Lua scripting

Epok can author gameplay classes in Lua. A Lua class is a real subclass of a
reflected C++ class: it inherits properties, overrides reflected events, and is
published into the same class registry as C++ and Blueprint classes, so all
three can see and call each other.

Lua authoring uses one versioned language profile, **`epok-lua` v1**, and one
project-wide execution setting that chooses how method bodies are implemented on
the console. The same scripts are authored once; the setting selects whether
they are compiled to native MIPS or interpreted by a Lua VM linked into the
game. See [The Lua VM runtime](lua-vm-runtime.md) for what a VM build contains.

`epok-lua` is **not general Lua compatibility**. It is a small statically typed
subset with a closed set of constructs, listed in full below. Source outside the
profile is rejected with one diagnostic, identically in every execution mode.

## Creating a Lua class

Each `.lua` file declares exactly one class and lives under the project's
`assets/scripts/` (subfolders allowed). Symbolic links inside `assets/scripts`
are rejected by discovery.

| Where | What it does |
| --- | --- |
| **Assets > Create > Lua Class** | Opens the creation dialog with a searchable parent list. |
| **Add Component > Create Lua ActorComponent...** | Same dialog, restricted to component parents, and attaches the result to the selected actor. |
| Content Browser **Add > Lua Class...** | Same dialog, targeting the browsed folder. |
| CLI | `epok-editor --project <dir> --new-lua-class <Name> [--parent <CppName>] [--folder <Sub/Folder>]` |

The CLI parent defaults to `epok::ActorComponent` and the folder defaults to the
root of `assets/scripts`. The command prints the path of the created file.

Creation is a transaction: the file is written, the whole project is recompiled,
and if the new class does not compile, the file and any directories the command
created are removed again.

The generated template is:

```lua
local Guard = epok.class {
    profile = 1,
    id = "7bb2a7b2-3a7c-4285-b11e-d6252ac5b7d2",
    name = "Guard",
    extends = "epok::ActorComponent",
    properties = {},
    functions = {}
}

function Guard:begin_play()
end

return Guard
```

A worked example with a C++ base, a Lua child and a Lua-derived-from-Lua child
is in [`examples/lua-scripting/`](../examples/lua-scripting/README.md).

## The `epok.class` metadata table

`epok.class { ... }` is **data, not code**. It is read statically from the
syntax tree and is never executed, so the editor can list classes, properties
and functions without a Lua interpreter, and the same metadata is produced in
every build.

The file must contain exactly one `local <Name> = epok.class { ... }` statement
and must end with `return <Name>`. Methods must be declared on that same local.

| Key | Required | Meaning |
| --- | --- | --- |
| `profile` | yes | Language profile version. Must be `1`; any other value is rejected. |
| `id` | yes | Canonical UUID of the class. |
| `name` | yes | Generated C++ class name. Must be a valid identifier and must not start with `epok_`. |
| `extends` | yes | `cpp_name` of the parent class. |
| `properties` | no | Table of `<name> = { ... }` property declarations. |
| `functions` | no | Table of `<name> = { ... }` function declarations. |

### Property entries

| Key | Required | Meaning |
| --- | --- | --- |
| `id` | yes | Canonical UUID of the property. |
| `type` | yes | One of the type names below. |
| `default` | no | A literal value. Defaults to the type's zero value. |
| `editable` | no | `true` exposes the property in the Inspector. Defaults to `false`. |

Defaults are literals only — a number, `true`/`false`, or a table of numbers for
a vector. An expression is rejected with `Default is not a literal value`.

### Function entries

| Key | Required | Meaning |
| --- | --- | --- |
| `id` | yes | Canonical UUID of the function. |
| `parameters` | no | Ordered list of `{ name = "...", type = "..." }`. |
| `returns` | no | A type name, or `"void"`. Defaults to `"void"`. |
| `callable` | no | Declares the method callable from other classes. |
| `overrides` | no | Name of the reflected parent event this method overrides. |

An entry with `overrides` must match the reflected parent signature exactly —
same name, same return type, same parameter names and types — or it is rejected
with `Override <name> signature differs from its reflected parent`. The parent
member must be an event, must not be `final`, must not be `private`, and may be
overridden only once.

### Type names

`void`, `Bool`, `Int32`, `UInt32`, `Fixed`, `Vector2`, `Vector3`,
`ActorRef`, `ComponentRef`, `ObjectRef`, and the parameterized forms
`ActorRef<Class>`, `ComponentRef<Class>`, `ObjectRef<Class>`,
`AssetRef<Kind>`, `ClassRef<Base>`.

The vocabulary is closed: an unknown name is a diagnostic
(`<name> is not an epok-lua type name`), never passed through to the generator.

### Identity

`id` values are UUIDs that belong to the **source file**, exactly as for C++ and
Blueprint members. Moving or renaming the `.lua` file does not change them, so
serialized instance overrides and references survive. A UUID that is not
canonical, is nil, or collides with any reflected, Blueprint or other Lua
identity is rejected.

A lifecycle event written as a bare method with no `functions` entry gets a
derived id, computed from the class id and the method name, so it is also stable
across moves and renames.

### Parent eligibility

A Lua class selects its parent through the same registry and the same rule as a
Blueprint: the parent must be `Blueprintable`, must not be `final`, and must use
the native execution backend. No extra C++ annotation is required.

- A Lua class may extend a reflected **C++** class or another **Lua** class.
- A **Blueprint** may extend a Lua class. Lua compiles before Blueprints, so the
  Lua type already exists when the Blueprint resolves its parent. Create it from
  the editor: the CLI `--new-blueprint --parent` does not yet resolve Lua
  parents (see [Limits](#limits-and-not-yet)).
- A Lua class **may not** extend a Blueprint class. That is rejected with
  `Lua classes derive from C++ or Lua classes; Blueprint parents are not
  supported yet`.

The whole hierarchy is limited to **16 properties**; exceeding it is rejected
with `Lua hierarchy exceeds the 16-property runtime budget`.

## Writing methods

Methods are declared with a colon:

```lua
function Guard:wake_up(amount)
    self.awake = true
    self.health = self.health - amount
    return self.health
end
```

`function Guard.wake_up(...)` (a dot) declares a static function and is
rejected. Every method must either appear in `functions` or be a reflected
lifecycle event.

### Lifecycle overrides

These five reflected events may be overridden without a `functions` entry:

`begin_play`, `tick`, `end_play`, `on_enable`, `on_disable`

Their signature comes from the reflected parent, and the Lua parameter names
must match the reflected parameter names exactly. C++ declares
`virtual void tick(Fixed)` with an unnamed parameter, so reflection names it
`arg0` and the override is written:

```lua
function Guard:tick(arg0)
    self.health = self.health - self.alert_speed * arg0
end
```

A name that does not match is rejected with
`Parameter <name> is declared as <reflected name>`.

### Calling the parent

```lua
epok.super(Guard, self):begin_play()
```

The rule is exact:

- The first argument is the **enclosing class name**, written literally. A
  different name is rejected with
  ``` `epok.super` must name the enclosing class <Name> ```.
- The second argument is `self`.
- It must be followed immediately by a method call. On its own it is rejected
  with ``` `epok.super(Class, self)` must be followed by a method call ```.
- Resolution is **lexical**: it always calls the qualified parent
  implementation, never the most-derived runtime type.

A parent method reached this way must either be the method this body overrides,
or be public and callable.

### Calling other methods

`self:name(...)` calls a method of this class through the C++ virtual, so a
derived class's override wins. `epok.super(Class, self):name(...)` is the
qualified parent call. **No other receiver is supported**; anything else is
rejected with
``` Only `self` and `epok.super(Class, self)` receivers are supported ```.

Reflected native methods that are `BlueprintCallable` and public are visible to
Lua bodies by name, **in every execution mode**, whether the Lua class declares
them or merely inherits them:

```lua
function Guard:wake_up(amount)
    self.awake = true
    -- Declared by the native parent, not by this class.
    self:apply_damage(amount)
    return self.health
end
```

In the VM modes the generated binding publishes a dispatch slot for the class's
own methods *and* for every inherited reflected function, so an inherited
callable is reached through the same `self_call` switch as an own method, and
`epok.super(Class, self):name(...)` through the matching `super_call` switch.
Declaring a new, non-override callable is likewise supported in every mode.

Recursion is rejected: a cycle in the call graph of methods declared in the same
chunk produces `Recursive calls are not supported by the epok-lua profile`.

### Properties and the Inspector

`self.<name>` reads and writes a property declared by this class or inherited
from any ancestor. `self.<vector>.x`, `.y` and `.z` address vector components.

Lua classes use the same class/instance infrastructure as Blueprints: declared
defaults appear in the Inspector, editing a field stores an explicit instance
override, and **Reset to Inherited** removes the override and restores the
current class default. Inherited exposed properties occupy their native base
fields — there is exactly one authoritative representation, not a copy.

The **Edit Lua Class** source action opens the authored `.lua`, never the
generated C++.

## The `epok-lua` v1 profile

The profile is identical in all three execution modes, and its diagnostics never
name a mode.

### Supported

- Types: `Bool`, `Int32`, `UInt32`, `Fixed` (Q12), `Enum`, `Vector2`/`Vector3`
  components (the `.x`, `.y` and `.z` scalars — never a whole vector value),
  `ObjectRef`, `ActorRef`, `ComponentRef`.
- Locals with a single inferred type, checked for definite assignment before
  use.
- `if` / `elseif` / `else`, `do ... end` blocks.
- Numeric `for` with constant integer bounds and a non-zero constant step. The
  loop variable is `Int32`.
- `return` — at most one value, and only as the last statement of a block.
- Single assignment to a local, a property, or a vector component.
- Calls to `self:` methods, `epok.super(Class, self):` parent methods, and the
  two conversion builtins.
- Comparisons `== ~= < <= > >=`, `and`, `or`, `not`, unary `-`.
- Arithmetic `+ - * /`, and `%` on integers.

### Builtins

| Builtin | Signature |
| --- | --- |
| `epok.to_fixed(value)` | `Int32` → `Fixed` |
| `epok.to_int(value)` | `Fixed` → `Int32` |
| `epok.super(Class, self)` | Qualified parent receiver (must be called on) |

There are no other callable globals. Anything else produces
`Unknown global function <name>`.

### Rejected constructs

Each of these produces exactly one diagnostic, with the text shown:

| Construct | Diagnostic |
| --- | --- |
| `...` | `Varargs are not supported by the epok-lua profile` |
| `function(...) end` | `Anonymous functions and closures are not supported by the epok-lua profile` |
| nested `function` | `Nested function definitions are not supported by the epok-lua profile` |
| `a, b = 1, 2` | `Multiple assignment is not supported by the epok-lua profile` |
| `return a, b` | `Multiple return values are not supported by the epok-lua profile` |
| `..` | `String concatenation is not supported by the epok-lua profile` |
| `#` | `The length operator is not supported by the epok-lua profile` |
| `^` | `The power operator is not supported by the epok-lua profile` |
| `//` | `Floor division is not supported by the epok-lua profile` |
| `%` on non-integers | `The modulo operator requires Int32 or UInt32 operands` |
| `while` | `while loops are not supported by the epok-lua profile; use a constant-bounded numeric for` |
| `repeat` | `repeat loops are not supported by the epok-lua profile; use a constant-bounded numeric for` |
| `for k, v in ...` | `Generic for loops are not supported by the epok-lua profile` |
| `break` | `break is not supported by the epok-lua profile` |
| `goto`, `::label::` | `goto and labels are not supported by the epok-lua profile` |
| `setmetatable`, `getmetatable`, `rawget`, `rawset`, `rawequal` | `Metatables are not supported by the epok-lua profile` |
| `load`, `loadstring`, `loadfile`, `dofile`, `require` | `load, loadstring, dofile and require are not supported by the epok-lua profile` |
| a string value in a body | `String values are not supported by the epok-lua profile` |
| `nil` | `nil is not supported by the epok-lua profile` |
| `{ ... }` in a body | `Table constructors are not supported by the epok-lua profile` |
| `a[i]` | `Indexed access is not supported by the epok-lua profile` |
| non-`Bool` `and`/`or` | `and/or require Bool operands` |
| non-`Bool` condition | `Conditions must be Bool` |
| a call cycle | `Recursive calls are not supported by the epok-lua profile` |
| non-constant `for` bounds | `Numeric for bounds must be constant integers` |
| an oversized `for` | `Numeric for exceeds the 65536 iteration limit` |
| nesting past 32 levels | `Nesting depth exceeds the 32 level limit` |
| `AssetRef` / `ClassRef` in a body | `Asset and class references are not readable or writable from a body in the epok-lua profile` |
| a whole `Vector2` / `Vector3` value in a body | `Whole vector values are not supported by the epok-lua profile; use the .x, .y and .z components` |
| a bare integer literal with no context | `Numeric literal has no contextual type; annotate the target or use an explicit conversion` |

Coroutines and modules are covered by the same set: `coroutine` is not a
registered global, and `require` is rejected outright. There is no string type
in the profile at all.

The nesting limit is 32 (`MAX_DEPTH`) and the numeric `for` iteration limit is
65536 (`MAX_ITERATIONS`).

### Numeric rules

The numeric contract is the runtime's own Q12 implementation, shared with
Blueprints and with the generated C++ in every mode.

- `Fixed` is a raw `int32_t` with `4096 == 1.0`. A literal `0.5` is stored as
  raw `2048`; `1.5` as `6144`.
- Arithmetic **saturates** at the 32-bit bounds; it does not wrap.
  `ineg(INT32_MIN) == INT32_MAX`.
- Division and modulo **by zero yield 0**, never a fault.
- Division **truncates toward zero**.
- Comparisons are raw integer comparisons. Unsigned ordering is explicit, so a
  `UInt32` compares as unsigned.
- Both operands of a binary operator must have exactly the same type;
  `Binary inputs must have exactly the same type` otherwise. There is no
  implicit numeric coercion — use `epok.to_fixed` / `epok.to_int`.
- Literals are typed **by context**. A decimal literal with no context is
  `Fixed`; a bare integer literal with no context is a diagnostic. In
  `self.health - amount` the literal context comes from the property; in
  `self.steps_remaining - 1` the `1` is `Int32`.
- Unary `-` requires `Int32` or `Fixed`. `not` requires `Bool`.
- `and` / `or` accept `Bool` operands only. Conditions are `Bool` only — there
  is no truthiness.

### Short-circuit evaluation

`and` and `or` short-circuit, and the guarantee covers **side effects**, not just
the result value: when the left operand already decides the answer, the right
operand is **not evaluated**, so a call on the right does not run.

```lua
-- `self:bump()` is not called at all when `self.awake` is false.
if self.awake and self:bump() then
    self:defeated()
end
```

This holds identically in all three execution modes. In Native C++ the operator
lowers to C++ `&&` / `||`; in the VM modes both operands are `Bool`, so the
interpreter's own short-circuit is exact. Where the right operand needs a
statement to evaluate (a call), the frontend hoists it into a guarded block
rather than emitting it eagerly, which is what makes the two backends agree.

## Execution modes

One project, one mode. The setting lives in
**Project Settings > Scripting > Lua Execution** and is stored in the project's
`<Name>.epokproject` descriptor as:

```yaml
lua_execution: native_cpp   # or vm_bytecode, or vm_source
```

| Setting | Stored value | What it links |
| --- | --- | --- |
| **Native C++** | `native_cpp` | Nothing extra. Method bodies are lowered to C++ and compiled to MIPS. |
| **Lua VM — bytecode** | `vm_bytecode` | `lua/liblua-epok-noparser.a`, plus cooked bytecode payloads. |
| **Lua VM — source** | `vm_source` | `lua/liblua-epok-parser.a`, plus packaged normalized Lua text. |

Projects written before the setting existed read as `native_cpp`; they are never
migrated to a VM automatically.

**The promise.** Scripts are unchanged across modes. The declarations,
identities, property layout, serialized defaults and overrides, and the
generated physical C++ type are all mode-independent — `class Guard : public
EnemyBase` in every mode. Only the *implementation of method bodies* differs:
lowered native code in Native C++, and typed trampolines into the VM in the two
VM modes. Because the override is a real C++ virtual override in every mode, a
native call through a base reference reaches the Lua body in every mode.

**Play, Build and export all resolve the same field.** There is no per-run
override and no per-script mode.

**Changing the mode invalidates execution artifacts.** The mode participates in
the project fingerprint (the manifest is byte-hashed) and explicitly in the
scene input `scene-lua-settings`. Staging keys are mode-scoped, so an AOT object
cannot survive into a VM build and vice versa. The build also writes a generated
`lua-config.hh` carrying `EPOK_LUA_MODE` so VM-only translation units exclude
themselves.

**Failures never fall back.** If a VM mode cannot package or cook its chunks,
that is the compilation's failure, reported with its real cause. It never
silently degrades to native bodies, and a failed compilation must not leave a
stale artifact runnable.

## Diagnostics

Every Lua diagnostic is formatted as:

```
<file>:<line>:<column>: message
```

for example:

```
assets/scripts/Ticker.lua:13:5: while loops are not supported by the epok-lua profile; use a constant-bounded numeric for
```

Profile diagnostics never mention an execution mode: the same source is rejected
identically whichever mode the project selects. Mode-specific failures
(toolchain, bytecode ABI, capacity) report their real cause instead of asking
you to rewrite a valid script.

## Limits and not-yet

Current to this revision:

- **Vector values in bodies.** Vector *components* of a property
  (`self.offset.x`) work. A whole-vector value — a vector local, a vector
  assignment or vector arithmetic — is rejected **by the frontend, uniformly in
  all three modes**, with `Whole vector values are not supported by the
  epok-lua profile; use the .x, .y and .z components`. This is a profile rule,
  not a VM-backend restriction: it does not change when the project switches
  mode, and the diagnostic never names a mode. `Vector2` and `Vector3`
  properties remain declarable and Inspector-editable.
- **`AssetRef` and `ClassRef` in bodies.** They are 64-bit native fields and
  stay Inspector-editable, but no profile v1 body reads or writes one.
- **Lua extending Blueprint.** Not supported. A Lua class may pick a C++ or a
  Lua parent only; a Blueprint parent is rejected. The reverse direction
  (Blueprint extending Lua) is supported in the editor — but the CLI
  `--new-blueprint --parent <Name>` resolves parents from the reflected C++
  registry and compiled Blueprints only, so a Blueprint with a Lua parent
  cannot currently be created from the command line. Use **Assets > Create >
  Blueprint** instead.
- **No strings, arrays, tables, modules or coroutines.** There is no string type
  and no `require`.
- **No hot reload.** Changing a `.lua` file rebuilds and relinks like any other
  source change; native code is never patched into a running game.
- **No source-level debugging.** VM builds keep chunk debug information, so a
  runtime Lua error names the authored `.lua` file and line, but there is no
  stepping or breakpoint support for Lua bodies.
- **No per-script mode mixing.** The mode is a project-wide setting.

## Standalone exports

An exported project rebuilds with Make, the pinned Nugget SDK and a MIPS
toolchain, without the editor.

- **Native C++**: no extra dependency. The Lua bodies are already C++ in the
  export.
- **VM modes**: the export additionally needs Nugget's nested
  `third_party/psxlua` submodule, because `runtime/lua.mk` builds the
  interpreter archive from it during `make`. The three setup scripts
  (`tools/setup-macos.sh`, `tools/setup-linux.sh`, `tools/setup.ps1`)
  initialize and verify psxlua alongside Nugget at its pinned revision.

No host Lua, no `luac` and no network access is needed at export build time —
the chunk payloads are already inside the generated
`scripts/generated/lua/lua_chunks.cpp`.

## See also

- [The Lua VM runtime](lua-vm-runtime.md) — arena budget, linked archives,
  bytecode ABI verification and what has and has not been measured.
- [`examples/lua-scripting/`](../examples/lua-scripting/README.md) — a runnable
  minimal example.
- [C++ scripting and exports](scripting.md) — writing the reflected bases Lua
  classes extend.
- [Blueprints](blueprints.md) — the visual authoring provider that shares the
  same registry, Inspector and inheritance rules.
