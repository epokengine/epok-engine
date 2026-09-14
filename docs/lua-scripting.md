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
---@class Guard : epok.ActorComponent
local Guard = epok.ActorComponent:extend()

Guard.speed = 1.0

function Guard:begin_play()
end

return Guard
```

A worked example with a C++ base, a Lua child and a Lua-derived-from-Lua child
is in [`examples/lua-scripting/`](../examples/lua-scripting/README.md).

## Writing a class

A class declaration is **data, not code**. Every part of it — the parent, the
properties and the function signatures — is read statically from the syntax
tree, so the editor can list classes, properties and functions without a Lua
interpreter, and the same metadata is produced in every build. **The engine
never executes a line of a project's Lua to discover a class.**

A file has exactly four kinds of statement at file scope, in this order:

```lua
---@class Cube : epok.Actor3D          -- optional; see The ---@class line
local Cube = epok.Actor3D:extend()     -- the declaration head

Cube.speed = 90.0                      -- properties, before any method

function Cube:begin_play()             -- methods
end

return Cube                            -- the last statement
```

Anything else at file scope is rejected with
``` Only `local <Class> = <Parent>:extend()`, `<Class>.<name> = <value>` property assignments, method definitions and a final `return <Class>` are allowed at file scope ```.

### The declaration head

`local <Class> = <Parent>:extend()` declares the class. The local must be named
after the file: `Cube.lua` declares `Cube`, and a different local name is
rejected with `The class local is named <X>; Cube.lua declares Cube`. The file
must end with `return <Class>`, and a second `local` is rejected with
`A Lua script declares exactly one class`.

The class name is therefore the file name — which an author already has to keep
unique — and never has to be typed twice. A file whose stem is not a usable C++
identifier is rejected with `The file name is not a usable class name (<stem>)`.

`<Parent>` is written the way the generated Lua definitions spell it, with `.`
where C++ writes `::`:

| Written | Parent |
| --- | --- |
| `epok.Actor3D:extend()` | the engine class `epok::Actor3D` |
| `EnemyBase:extend()` | a project C++ class, or another Lua class, at global scope |
| `game.Enemy:extend()` | the namespaced C++ class `game::Enemy` |

Everything reachable is bound as a value in the generated definitions, so the
editor completes the eligible parents as you type. An unresolved name is
rejected with `Unknown parent <cpp_name>`.

### The `---@class` line

`---@class <Class> : <Parent>` above the declaration head is what gives the Lua
Language Server the type of the local, which is what makes `self.speed`,
`self:take_damage(1.0)` and `Cube.super.tick` complete and type-check in an
editor. The creation template always writes it.

It is **optional for compiling**: the compiler reads the class from the file
name and the parent from the `extend()` call. When it is present it must agree
with both, or it is rejected with `---@class names <X>` or
`---@class extends <X>`.

### Properties

A property is a plain assignment on the class local, written before the first
method:

```lua
Cube.speed = 90.0                       -- Fixed, editable
Cube.spinning = true                    -- Bool
Cube.lives = 3                          -- Int32
Cube.count = epok.UInt32(0)             -- UInt32
Cube.target = epok.ActorRef(EnemyBase)  -- narrowed actor reference
Cube.mode = epok.Enum(Mode, "Idle")     -- reflected enum, by variant name
Cube.colour = epok.Vector3(1.0, 0.5, 0.0)
Cube.hidden = epok.Hidden(1.0)          -- not shown in the Inspector
```

A bare literal declares both the type and the default. Everything a literal
cannot express has exactly one constructor:

| Written | Type | Default | Inspector |
| --- | --- | --- | --- |
| `90.0`, `-2.5` | `Fixed` | the literal | editable |
| `3`, `-7` | `Int32` | the literal | editable |
| `true`, `false` | `Bool` | the literal | editable |
| `epok.Bool(b)` | `Bool` | `b` | editable |
| `epok.Int32(n)` | `Int32` | `n` | editable |
| `epok.UInt32(n)` | `UInt32` | `n` | editable |
| `epok.Fixed(x)` | `Fixed` | `x` | editable |
| `epok.Vector2(x, y)` | `Vector2` | the two components | editable |
| `epok.Vector3(x, y, z)` | `Vector3` | the three components | editable |
| `epok.Enum(E, "Variant")` | the reflected enum `E` | that variant | editable |
| `epok.ActorRef()` / `epok.ActorRef(Class)` | `ActorRef` / `ActorRef<Class>` | null | editable |
| `epok.ComponentRef()` / `epok.ComponentRef(Class)` | `ComponentRef` / `ComponentRef<Class>` | null | editable |
| `epok.ObjectRef()` / `epok.ObjectRef(Class)` | `ObjectRef` / `ObjectRef<Class>` | null | editable |
| `epok.AssetRef("Kind")` | `AssetRef<Kind>` | null | editable |
| `epok.ClassRef(Base)` | `ClassRef<Base>` | null | editable |
| `epok.Hidden(<any of the above>)` | the wrapped type | the wrapped default | **not** editable |

Class operands (`epok.ActorRef(EnemyBase)`, `epok.ClassRef(Base)`,
`epok.Enum(Mode, ...)`) are written **unquoted**, exactly as a parent is; asset
kinds and enum variants are quoted strings. A constructor that is not in the
table is rejected with `epok.<Name> is not an epok value constructor`, and a
value that is neither a literal nor a constructor — an expression such as
`1 + 1`, a string, `nil` — is rejected with
`A property is a literal default or an epok value constructor ...`.

`position`, `rotation`, `scale`, `rect_position`, `rect_size`, `super` and `ref`
are reserved: a property of one of those names would shadow an intrinsic and is
rejected. A property that shadows an inherited member is rejected too.

Only components of vectors cross the boundary in a body, so a `Vector2` or
`Vector3` property is declared and edited whole but read and written as
`.x`, `.y`, `.z`.

### Functions

A method **is** its declaration: there is no separate function table. Its
signature comes from the annotations the Lua Language Server already reads, so
one set of lines types the method in the editor and declares it to the compiler.

```lua
---@param amount Fixed
---@return Fixed
function Cube:take_damage(amount)
    self.speed = self.speed - amount
    return self.speed
end
```

| Method | What it is |
| --- | --- |
| One of the five lifecycle names (`begin_play`, `tick`, `end_play`, `on_enable`, `on_disable`) | An override; the signature comes from the reflected parent, so no annotation is needed. |
| Marked `---@override` | An override of any other reflected parent event; the signature comes from the parent. |
| Annotated with `---@param` / `---@return` | A new callable other classes and Blueprints may call. |
| No parameters and no annotations | A new `void` callable. |
| Parameters but no annotations | Rejected: `Declare the parameter types with ---@param annotations, for example ---@param amount Fixed` |

One `---@param` per parameter, in source order, naming the very parameters of
the method below; a mismatch is rejected with
`<name> takes N parameter(s) but declares M with ---@param` or
`Parameter <written> is annotated as <annotated>`. A method returns at most one
value. `function Cube.take_damage(...)` (a dot) declares a static function and
is rejected.

An override must match the reflected parent signature exactly — same name, same
return type, same parameter names and types — or it is rejected with
`Override <name> signature differs from its reflected parent`. The parent member
must be an event, must not be `final`, must not be `private`, and may be
overridden only once.

### Type names

`void`, `Bool`, `Int32`, `UInt32`, `Fixed`, `Vector2`, `Vector3`,
`ActorRef`, `ComponentRef`, `ObjectRef`, and the parameterized forms
`ActorRef<Class>`, `ComponentRef<Class>`, `ObjectRef<Class>`,
`AssetRef<Kind>`, `ClassRef<Base>`. A class operand may be written with `.` or
`::`, so `ActorRef<epok.Actor3D>` and `ActorRef<epok::Actor3D>` are the same
type.

The vocabulary is closed: an unknown name is a diagnostic
(`<name> is not an epok-lua type name`), never passed through to the generator.

### Identity

Every class, property and function has a persistent identity, and the author
never has to type one.

**A class is identified outside its source text.** The editor records one UUID
per script path in `ProjectSettings/LuaClasses.epoksettings`:

```yaml
classes:
  assets/scripts/Cube.lua: 7bb2a7b2-3a7c-4285-b11e-d6252ac5b7d2
  assets/scripts/Enemies/Guard.lua: 5e21b03e-da85-417a-867b-420cd25b01a4
```

That is deliberate, and it is what makes a Lua class **rename-safe**. The class
name is the file name, so a class that carried its identity in its source could
not be renamed without becoming a different class and orphaning every placed
instance. With the identity beside the script instead:

| Action | Effect |
| --- | --- |
| **Assets > Create > Lua Class** | A fresh UUID is recorded before the file is written. |
| Rename or move a `.lua` in the Content Browser | The entry follows the file; the class keeps its UUID and every placed instance stays bound. |
| Delete a `.lua` in the Content Browser | The entry is removed. |
| Copy a `.lua` in by hand, or merge one in from version control | The next catalog refresh records a UUID for it, and the class is rename-safe from then on. |

The document is a plain text map, one `path: uuid` line per class, sorted by
path, so a version control merge of two authors who each added a class is a
two-line merge and a real conflict is readable. It is validated on read:
canonical UUIDs, unique, on distinct `assets/scripts/**.lua` paths. Only the
settings document is ever written — **an author's `.lua` is never rewritten**.

Until an entry exists, the class is identified by a **derived**
`lua:<Name>` identity. It is stable across machines and checkouts — it depends
on nothing but the class name — so a project that has never been opened in the
editor still compiles reproducibly. Restoring a deleted script from the Content
Browser trash restores the file but not the entry, so the class is adopted again
with a new UUID; re-record the old UUID by hand if instances must stay bound.

Members follow the same rule as reflected C++, where `Id=` is optional and an
unannotated declaration is identified by `cpp:<USR>`:

- **Explicit.** `---@id <uuid>` above a property assignment or a method pins that
  member. It must be a canonical, non-nil UUID, and it **survives a rename**.
- **Derived.** With no `---@id`, the engine assigns
  `lua:<class identity>:<member>` deterministically. It is computed from the
  *name*, so **renaming a member with a derived id changes its identity** and
  orphans what referenced it. Pin a member with `---@id` before renaming it.

```lua
---@id 0f1d2c3b-4a59-4687-9b0c-1d2e3f405162
Cube.health = 100.0
```

A derived id cannot collide with a UUID or with a reflected `cpp:` identity by
construction. Any id — explicit or derived — that collides with another
reflected, Blueprint or Lua identity is rejected, and an explicit id that is not
a canonical UUID is rejected with `<what> id must be a canonical UUID`.

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

Methods are declared with a colon, and their signature is declared by the
annotations above them (see [Functions](#functions)):

```lua
---@param amount Fixed
---@return Fixed
function Guard:wake_up(amount)
    self.awake = true
    self.health = self.health - amount
    return self.health
end
```

### Lifecycle overrides

These five reflected events may be overridden with no annotation at all:

`begin_play`, `tick`, `end_play`, `on_enable`, `on_disable`

Their signature comes from the reflected parent, and the Lua parameter names
must match the reflected parameter names exactly. C++ declares
`virtual void tick(Fixed delta_seconds)`, so the override is written:

```lua
function Guard:tick(delta_seconds)
    self.health = self.health - self.alert_speed * delta_seconds
end
```

A name that does not match is rejected with
`Parameter <name> is declared as <reflected name>`.

### Calling the parent

```lua
function Guard:begin_play()
    Guard.super.begin_play(self)
end
```

The rule is exact:

- The receiver is `<Class>.super`, where `<Class>` is the **enclosing class**,
  written literally. A different name is rejected with
  ``` `<X>.super` must name the enclosing class <Name> ```.
- The running instance is passed explicitly, as the first argument, and it must
  be `self`. Anything else is rejected with
  ``` A qualified parent call passes `self` first, as in `Cube.super.tick(self, delta_seconds)` ```.
- Resolution is **lexical**: it always calls the qualified parent
  implementation, never the most-derived runtime type. There is no dynamic
  `super` value: `<Class>.super` exists only as the target of a call.

A parent method reached this way must either be the method this body overrides,
or be public and callable.

### Calling other methods

`self:name(...)` calls a method of this class through the C++ virtual, so a
derived class's override wins. `<Class>.super.name(self, ...)` is the qualified
parent call. **No other receiver is supported**; anything else is rejected with
``` Only `self:method(...)` and `<Class>.super.method(self, ...)` receivers are supported ```.

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
`<Class>.super.name(self, ...)` through the matching `super_call` switch.
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

### Transform access

Every spatial class also has `self.position`, `self.rotation` and `self.scale`
without declaring anything. They are not properties: they address the actor's
root component through the same runtime functions as the Blueprint **Get/Set
Position**, **Get/Set Rotation** and **Get/Set Scale** nodes, so a Lua body and
a Blueprint graph move one transform.

| Class | `position` | `rotation` | `scale` |
| --- | --- | --- | --- |
| `Actor` with Domain=World3D (`epok::Actor3D` and below) | `Vector{3}` | `Vector{3}` | `Vector{3}` |
| `Actor` with Domain=World2D (`epok::Actor2D` and below) | `Vector{2}` | scalar `Fixed` | `Vector{2}` |
| `ActorComponent` whose `Owners` admit World3D | `Vector{3}` | `Vector{3}` | `Vector{3}` |
| `ActorComponent` whose `Owners` admit only World2D | `Vector{2}` | scalar `Fixed` | `Vector{2}` |

A component has no transform of its own: it addresses the transform of the
actor that owns it.

Only components are readable and writable, exactly as for a `Vector` property —
`self.rotation.y`, and `.x`/`.y` only in World2D. A whole vector
(`self.position = ...`, or reading `self.scale` into a local) is rejected with
the profile's `Whole vector values are not supported` diagnostic. The World2D
`rotation` is the one exception: it is a single `Fixed` angle, so it is read and
written whole and has no components.

```lua
function Spinner:tick(delta_seconds)
    self.rotation.y = self.rotation.y + self.speed * delta_seconds
    self.position.x = self.position.x + 0.5 * delta_seconds
end
```

Two rules follow from the names being intrinsic:

- `position`, `rotation` and `scale` are **reserved**. Declaring a property with
  one of those names is rejected at the declaration, in any class.
- Using them on a class that is not spatial — a `Domain=None` class such as
  `epok::SceneScriptActor`, or a UI class — is a profile diagnostic naming the
  class's family and domain, not a silent no-op.

A **UI** class has its own pair of intrinsic places instead: `self.rect_position`
and `self.rect_size`, each a `Vector{2}` addressed by `.x` and `.y`. They go
through the same runtime functions as the Blueprint **Get/Set Rect Position** and
**Get/Set Rect Size** nodes, and they follow every rule above — reserved names,
components only, and a diagnostic when named outside the UI domain.

```lua
function Panel:tick(delta_seconds)
    self.rect_position.x = self.rect_position.x + 1.0
    self.rect_size.y = 64.0
end
```

## The `epok-lua` v1 profile

The profile is identical in all three execution modes, and its diagnostics never
name a mode.

### Supported

- Types: `Bool`, `Int32`, `UInt32`, `Fixed` (Q12), `Enum`, `Vector2`/`Vector3`
  components (the `.x`, `.y` and `.z` scalars — never a whole vector value),
  `ObjectRef`, `ActorRef`, `ComponentRef`. `AssetRef` and `ClassRef` are
  declarable and Inspector-editable but are never values in a body; they are
  named only as the operand of a builtin that takes one.
- Locals with a single inferred type, checked for definite assignment before
  use.
- `if` / `elseif` / `else`, `do ... end` blocks.
- Numeric `for` with constant integer bounds and a non-zero constant step. The
  loop variable is `Int32`.
- `return` — at most one value, and only as the last statement of a block.
- Single assignment to a local, a property, or a vector component.
- Calls to `self:` methods (including every inherited reflected callable, such
  as `Actor::set_active` and `Actor::destroy`), `<Class>.super.<method>(self, ...)`
  parent methods, and the builtins listed below.
- Comparisons `== ~= < <= > >=`, `and`, `or`, `not`, unary `-`.
- Arithmetic `+ - * /`, and `%` on integers.

### Builtins

Every builtin is a statically resolved call with a fixed arity and fixed operand
types. A mismatch is one diagnostic, with the same text in all three modes.

Each entry lowers to the very `epok::bp::api` entry point the Blueprint node
beside it calls — the AOT backend emits that call directly, and the two VM modes
reach it through a generated per-class dispatch case, never through a Lua-side
reimplementation. The two authoring surfaces are therefore equivalent by
construction rather than by agreement.

#### Language

| Builtin | Signature |
| --- | --- |
| `epok.to_fixed(value)` | `Int32` → `Fixed` |
| `epok.to_int(value)` | `Fixed` → `Int32` |
| `<Class>.super.<method>(self, ...)` | Qualified parent call |

#### Adapters

| Builtin | Returns | Blueprint node |
| --- | --- | --- |
| `epok.input.held(button, port)` | `Bool` | Input Held |
| `epok.input.pressed(button, port)` | `Bool` | Input Pressed |
| `epok.input.released(button, port)` | `Bool` | Input Released |
| `epok.request_scene(index)` | `Bool` | Request Scene |
| `epok.is_valid(ref)` | `Bool` | Is Valid |
| `epok.is_a(ref, "Class")` | `Bool` | Is A |
| `epok.cast(ref, "Class")` | typed ref, null when incompatible | Cast |
| `epok.spawn("Class")`, `epok.spawn("Class", parent)` | `ActorRef<Class>` | Spawn |
| `epok.spawn_class(self.<ClassRef property>)`, with an optional `parent` | `ActorRef<base>` | Spawn Class |
| `epok.owner()` | `ActorRef` | Get Owner (Component classes only) |
| `epok.play_audio(ref)`, `epok.stop_audio(ref)` | `void` | Play Audio, Stop Audio |
| `epok.set_texture(ref, self.<AssetRef property>)` | `void` | Set Texture |
| `epok.set_audio_clip(ref, self.<AssetRef property>)` | `void` | Set Audio Clip |
| `epok.play_sequence(ref)`, `epok.play_effect(ref)` | statement only | Play Sequence, Play Effect |
| `self.ref` | the object's own typed reference | Self |
| `self.rect_position.x` / `.y`, `self.rect_size.x` / `.y` | `Fixed` places | Get/Set Rect Position, Get/Set Rect Size |

`button` and `port` are `UInt32`; `port` is 0 or 1 and a button index of 16 or
more always reads `false`, exactly as the Blueprint node does.

`epok.is_a`, `epok.cast`, `epok.spawn` name their class by an authored name
resolved through the class registry at compile time, so a misspelling is a
compile error rather than a silent null. `epok.spawn` additionally applies the
Blueprint Spawn rule: the class must be a concrete, spawnable Actor. The spawned
instance's logical parent is the actor running the body, as it is for the node.

A spawn made from inside an event or a tick is **queued** and runs when the
current batch finishes, so the reference it returns is not live yet — again
exactly as the Blueprint node behaves. Drive an actor's lifetime from that
actor's own body (`self:destroy()`), not from a stored reference to it.

#### Asset and class operands never cross the boundary

`AssetRef` and `ClassRef` are 64-bit native fields, and profile v1 has no
64-bit value. The builtins that take one therefore accept exactly one spelling:
a direct read of a declared property of this class.

```lua
epok.set_texture(self.ref, self.skin)   -- self.skin is an AssetRef property
```

Anything else is `Asset and class reference arguments must be a direct read of a
declared property of this class, such as epok.set_texture(self.ref, self.skin)`.
In the native mode the generated body reads the field directly; in the two VM
modes the whole call is one generated binding case that reads the same field, so
the id is never packed into a Lua number and never truncated.

#### Not in profile v1

| Spelling | Diagnostic |
| --- | --- |
| `epok.stop_sequence`, `epok.pause_sequence`, `epok.resume_sequence`, `epok.stop_effect`, `epok.pause_effect`, `epok.resume_effect`, `epok.burst_effect`, `epok.effect_sequence`, `epok.play_timeline`, `epok.spawn_particle_effect` | `Sequence and effect handles are not values in the epok-lua profile; play a component sequence or effect as a statement and control it from a Blueprint` |
| `epok.get_transform`, `epok.make_transform`, `epok.transform` | `Whole transforms are not supported by the epok-lua profile; use the components, such as self.position.x and self.scale.z` |

A playback handle is wider than the profile's 32-bit value ABI, so it is never a
value: `epok.play_sequence(ref)` and `epok.play_effect(ref)` are accepted only as
statements, and the handle stays native. Nothing is silently truncated.

There are no other callable globals. Anything else produces
`Unknown global function <name>`.

### The Lua-only workflow

Everything above means a Lua author needs **no C++ and no Blueprint** for the
builtin surface. A `.lua` file alone can read input, change scene, create actors
by class name, ask what an object is, address the transform or the UI rect,
drive audio and textures, and end an actor's run — because `Actor::set_active`,
`Actor::destroy`, `Actor::wants_tick` and `Actor::set_wants_tick` are reflected
and reached through the ordinary `self:` call syntax, like any other inherited
callable.

`examples/lua-scripting/Spawner.lua` and `Spinner.lua` are that workflow end to
end: one class spawns the other in `begin_play`, both end their own run from
`tick`, and neither file has a C++ helper or a graph behind it.

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
  properties remain declarable and Inspector-editable. The intrinsic
  `position`, `rotation` and `scale` follow the same rule.
- **No transform on non-spatial classes.** `position`, `rotation` and `scale`
  exist only for World3D and World2D classes, and `rect_position` and
  `rect_size` only for UI classes. They address the root component of the owning
  actor — never a component's own local transform, and never a parent-relative
  one beyond what the Blueprint nodes already address. Naming one outside its
  domain gives `position, rotation and scale are only available on World3D and
  World2D classes, rect_position and rect_size on UI classes`.
- **`AssetRef` and `ClassRef` in bodies.** They are 64-bit native fields and
  stay Inspector-editable. No profile v1 body reads or writes one as a value;
  the only place one may be named is as the operand of a builtin that takes it
  (`epok.set_texture`, `epok.set_audio_clip`, `epok.spawn_class`), where both
  backends read the native field and the id never crosses the boundary.
- **Sequence and effect handles.** A playback handle is wider than the profile's
  32-bit value ABI, so it is never a value. `epok.play_sequence` and
  `epok.play_effect` are accepted as statements; the builtins that take a handle
  are rejected with a named diagnostic rather than truncating one.
- **Whole transform records.** `GetTransform` and `MakeTransform` have no Lua
  spelling: a `epok::Transform` is not a value in the profile. Address the
  components instead.
- **A spawned reference is not live yet.** `epok.spawn` inside an event or tick
  queues the spawn, exactly as the Blueprint Spawn node does, so the reference
  it returns only becomes valid after the current batch. Drive lifetime from the
  spawned actor's own body.
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

## Editor tooling: generated API definitions

Every time the editor publishes a fresh script catalog — a C++ reflection pass, a
Blueprint change or a `.lua` change — it writes `.epok/lua/epok.d.lua`, a Lua
Language Server (EmmyLua/LuaLS) definition file describing the project's whole
authoring surface:

- every reflected class, from all three providers, with its parent, its own
  properties and its callable and overridable members;
- the intrinsic places a class actually has — `position`, `rotation` and `scale`
  on a World3D or World2D class, `rect_position` and `rect_size` on a UI class —
  and `ref`, the object's own typed reference;
- the `epok` namespace: the property value constructors, the conversions and every
  adapter builtin of profile v1, with the same arities and types the compiler
  enforces;
- the value vocabulary, as `Fixed`, `Int32`, `UInt32`, `Bool`, `Vector2`,
  `Vector3` and the reference and handle types;
- the declaration form itself: `extend()` on the hierarchy root, `super` on every
  Lua class, and every class bound as a *value* under the name its Lua type
  carries, so `epok.Actor3D:extend()`, `EnemyBase:extend()` and
  `Cube.super.tick(self, delta_seconds)` all resolve and type-check.

A C++ class name becomes a Lua type name by writing `.` where C++ writes `::`,
so `epok::Actor3D` is the type `epok.Actor3D`. Methods are declared on a
file-local table, so the only globals the file introduces are `epok` and the
class values above.

The file is generated, never authored: it is rewritten from the registry on
every refresh, is regenerated from scratch if deleted, and nothing in the engine,
the compiler or the cooked build ever reads it. `.epok/` is already ignored by
a new project's `.gitignore`, so it is never committed.

### Pointing an editor at it

Alongside the definitions the editor writes a `.luarc.json` at the project root,
but **only when the project has none** — an existing one is never overwritten:

```json
{
  "runtime.version": "Lua 5.2",
  "workspace.library": [".epok/lua"],
  "diagnostics.globals": ["epok"]
}
```

Any LuaLS client reads it: in VS Code install the Lua extension (sumneko) and
open the project folder; in Neovim, point `lua_ls` at the same workspace. To
configure a client by hand instead, add `.epok/lua` to `workspace.library`.

Completion is tooling only. The compiler is the authority on the profile: a file
the language server accepts can still be rejected, with the diagnostics above.

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
