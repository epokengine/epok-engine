# Epok Lua scripting — implementation contract

Status: implementation contract for the Lua scripting initiative (three selectable
execution modes over one shared authoring profile). Baseline: `develop` @ 32fc510.
Feasibility evidence: `tests/integration/lua_feasibility/README.md`.

This file is the shared contract. Every milestone builds against it.

## 1. Product contract

Exactly one Lua execution mode per project/build, selected in Project Settings:

| Setting value | Enum | Implementation |
| --- | --- | --- |
| Native C++ | `LuaExecution::NativeCpp` | AOT: typed IR lowered to C++, compiled to MIPS |
| Lua VM — bytecode | `LuaExecution::VmBytecode` | PsyQo Lua, `liblua-epok-noparser.a`, cooked bytecode |
| Lua VM — source | `LuaExecution::VmSource` | PsyQo Lua, `liblua-epok-parser.a`, packaged normalized source |

The selection is an exclusive enum on the project-owned `workspace::Manifest`, never
three booleans. Play, Build and export all resolve the same field. Only the libraries
and runtime support of the selected mode are linked.

## 2. Language profile: `epok-lua` v1

One versioned profile, identical in all three modes. A valid program for the profile is
valid in every mode; there is no backend-specific user API and no way for a script to
observe which mode is active.

Supported (initial profile):
- Types: `Bool`, `Int32`, `UInt32`, `Fixed` (Q12), `Enum`, `Vector{2,3}`, `AssetRef`,
  `ClassRef`, `ObjectRef`, `ActorRef`, `ComponentRef` — the existing `schema::Type` set.
- Declarations: `epok.class { ... }` metadata table, read statically from the AST. It is
  never executed to discover classes. Only `name` and `extends` are required: the engine
  assigns the identity and the profile (see §4), so `profile`, the class `id` and every
  member `id` are optional. A written `profile` pins the script and must equal the
  supported profile; a written `id` must be a canonical, non-nil UUID.
- Locals with inferred, fixed, single types; definite assignment checked.
- Conditions are `Bool` only. `and`/`or` accept `Bool` operands only, short-circuit.
- Statically resolved calls with fixed return arity.
- Fields of known shape backed by native storage.
- Intrinsic transform places on spatial classes: `self.position`, `self.rotation`
  and `self.scale`, reserved names that are not reflected properties. World3D
  exposes three `Vector{3}` places; World2D exposes `Vector{2}` position and
  scale plus a scalar `Fixed` rotation. A component addresses its owner's
  transform. Component access only, exactly as for a `Vector` property (the
  World2D rotation being a scalar is the sole whole-value case). Both backends
  lower them to the `epok::bp::api` position/rotation/scale entry points the
  Blueprint Get/Set nodes call, so the two authoring surfaces are equivalent by
  construction. Non-spatial classes get a profile diagnostic.
- Constant-bounded numeric `for`.
- Explicit parent call: `epok.super(Class, self):method(...)`.
- Intrinsic UI places on UI-domain classes: `self.rect_position` and
  `self.rect_size`, `Vector{2}` addressed by component, lowering to the
  `epok::bp::api` rect entry points the Blueprint Get/Set Rect nodes call.
- **Builtins.** Statically resolved calls with fixed arity and operand types,
  each one a `blueprint_asset::Builtin` node. Both backends lower a builtin
  through the single shared `blueprint_ir::builtin_cpp`, so a Lua call and the
  matching Blueprint node are the same generated `epok::bp::api` call by
  construction: the AOT backend emits it inline, and the VM modes reach it
  through a generated per-class `builtin_call` switch indexed by a dense,
  frontend-assigned call-site index. Nothing of a builtin is reimplemented in
  Lua, and no adapter-specific C function is registered per operation.

  | Lua surface | Blueprint builtin |
  | --- | --- |
  | `epok.input.held/pressed/released(button, port)` | InputHeld/InputPressed/InputReleased |
  | `epok.request_scene(index)` | RequestScene |
  | `epok.is_valid(ref)` | IsValid |
  | `epok.is_a(ref, "Class")`, `epok.cast(ref, "Class")` | IsA, Cast |
  | `epok.spawn("Class"[, parent])` | Spawn |
  | `epok.spawn_class(self.<ClassRef property>[, parent])` | SpawnClass |
  | `epok.owner()` | GetOwner |
  | `epok.play_audio(ref)`, `epok.stop_audio(ref)` | PlayAudio, StopAudio |
  | `epok.set_texture(ref, self.<AssetRef property>)` | SetTexture |
  | `epok.set_audio_clip(ref, self.<AssetRef property>)` | SetAudioClip |
  | `epok.play_sequence(ref)`, `epok.play_effect(ref)` (statement only) | PlaySequenceComponent, PlayEffectComponent |
  | `self.ref` | SelfObject |
  | `self.rect_position.x/.y`, `self.rect_size.x/.y` | Get/SetRectPosition, Get/SetRectSize |

  An `AssetRef`/`ClassRef` operand is accepted in exactly one spelling, a direct
  read of a declared property of the class. The native backend reads the field
  inline; the VM backends read it inside the binding case, so the 64-bit id
  never enters Lua. Impure builtins take the same `CheckOwner`-style guard after
  the statement as a self call, because they may spawn or destroy.

  Not in v1, each with a named diagnostic and never a silent truncation:
  the handle-taking sequence and effect builtins plus `PlayTimelineAsset` and
  `SpawnParticleEffect` (a `timeline::Handle`/`effects::Handle` is a 16-bit
  index plus a 32-bit generation and does not fit the §10.1 32-bit value ABI),
  and `GetTransform`/`MakeTransform` (whole-record rule). `SetActive` and
  `DestroyActor` have no builtin: `Actor::set_active` and `Actor::destroy` are
  reflected, so Lua reaches them as ordinary inherited callables on `self`.

Rejected by the common frontend in all three modes, with one source diagnostic:
dynamic tables, metatables, `load`/`loadstring`/`dofile`, closures, varargs, general
multiple returns, unbounded recursion or loops, string concatenation, coroutines,
`require`. Unsupported source produces the same message regardless of the selected mode.
Mode-specific failures (toolchain, bytecode ABI, capacity) report their real cause and
must never be presented as a request to rewrite a valid script.

## 3. Numeric contract

Identical in all three modes, taken from `runtime/blueprint_runtime.hpp`:
`saturate`, `iadd/isub/imul/idiv/imod/ineg`, `uadd/usub/umul/udiv/umod`,
`add/sub/mul/div/neg`, `from_int/to_int`. Q12 raw `int32_t`, 4096 = 1.0. Saturating.
Division and modulo by zero yield 0. Division truncates toward zero.
`ineg(INT32_MIN) == INT32_MAX`. Comparisons are raw integer comparisons.

VM parity is achieved by **calling the same C++ functions**: the normalized Lua emitted
for VM modes performs every arithmetic operation through C functions registered into the
VM that forward directly to `epok::bp::*`. The original user source is never executed
under stock fork arithmetic.

## 4. Provider and registry identity

- Authoring provider: `Extension { id: "lua", version: 1 }` — stable across all modes.
- Execution backend: always `schema::native_backend()` (`native` v1). The physical type
  is a real C++ subclass in every mode; only method bodies differ.
- Identity mirrors reflected C++, where `EPOK_*(Id=...)` is optional and an unannotated
  declaration is identified by `cpp:<USR>`. An explicit `id` is a canonical, non-nil UUID
  and survives a rename. With no `id` the engine derives one deterministically from the
  declaration: `lua:<Name>` for a class, `lua:<class identity>:<member>` for a property or
  a function, where the class identity is the explicit UUID if there is one and the class
  name otherwise. A bare lifecycle method with no `functions` entry uses the same member
  scheme. A derived id is stable across machines and across moves but changes when the
  declaration is renamed, so a rename needs an explicit id or a migration; `epok lua new`
  therefore writes an explicit class UUID into every new file.
- A derived id cannot collide with a UUID or a reflected `cpp:` identity by construction.
  The compiler still rejects any collision across the project, and rejects an explicit id
  that is not a canonical, non-nil UUID.
- Identities never depend on the mode. Serialized defaults, overrides and references are
  mode-independent. Generated artifacts are named by a filesystem-safe stem of the class
  identity: the UUID itself, or `lua_<Name>` for a derived identity.
- Lua classes are published into the same `blueprint::Registry` as C++ and Blueprint
  classes, before body checking, so all three can call each other.

## 5. Inheritance from reflected C++

A Lua class selects its parent through the same registry and the same eligibility rule
as a Blueprint, `script_backend::can_derive`: `blueprintable && !final_class && backend
== native`, plus a provider rule that depends on who is authoring. No second C++
annotation is required.

The provider rule as implemented: a C++ or Lua parent is eligible for every author; a
**Blueprint** parent is eligible only when the author is itself a Blueprint. So a Lua
class derives from C++ or Lua only, while a Blueprint may derive from a C++, Lua or
Blueprint parent.

**Lua → Blueprint parents are deferred.** They are not a missing check but an unresolved
declaration-order question (Lua compiles before Blueprints), and are out of scope for
profile v1. The reverse direction — a Blueprint deriving from a Lua class — is supported
in the registry, but the CLI `epok-editor --new-blueprint --parent <Name>` resolves its
parent from the reflected C++ registry plus compiled Blueprints only, so a Blueprint with
a Lua parent cannot currently be authored from the command line; use the editor.

The generated physical type is always `class Guard : public EnemyBase`.
- `NativeCpp`: Lua bodies compiled to native method bodies.
- `VmBytecode` / `VmSource`: typed trampolines that enter the VM.

Inherited exposed properties occupy their native base fields; there is exactly one
authoritative native representation. Only reflected virtual events may be overridden;
inherited signature, visibility, `final` and abstract-completion rules are enforced.
Native calls through a base reference reach Lua overrides in all three modes because the
override is a real C++ virtual override in every mode.

`epok.super(Guard, self):begin_play()` lowers to qualified `EnemyBase::begin_play()` in
AOT, and in VM modes to a generated per-class super-dispatcher that performs the same
qualified native call. Resolution is lexical, never the most-derived runtime type.

## 6. Module layout (Rust)

| Module | Responsibility |
| --- | --- |
| `src/lua_asset.rs` | `.lua` discovery, `epok.class` declaration extraction, ids, asset file model |
| `src/lua_frontend.rs` | lexer, parser, scopes, type inference, profile enforcement, diagnostics |
| `src/script_ir.rs` | shared typed body IR (language-neutral), reused by both backends |
| `src/lua_compile.rs` | orchestration: declarations, registry publication, backend dispatch |
| `src/lua_aot.rs` | IR -> C++ subclass with native bodies |
| `src/lua_vm.rs` | IR -> normalized Lua + C++ facade with trampolines; source/bytecode packaging |
| `src/lua_bytecode.rs` | bytecode cooking and ABI verification |

## 7. Runtime (C++)

`runtime/lua_runtime.hpp` — compiled only when a VM mode is selected:
- A statically sized arena allocator. **The runtime has no heap**; `psyqo_realloc` /
  `psyqo_free` are bound by `psyqo-lua.mk`, and the VM arena must be a static budget.
- VM bootstrap before `epok::initialize_scripts()` in `GameScene::start`.
- Registration of Epok arithmetic C functions forwarding to `epok::bp::*`.
- Generated per-class method tables; event bitmask so absent handlers cost nothing.
- Field access through generated typed accessors into the native object's fields.
- Every entry into Lua wraps `epok::ObjectDispatchScope` and validates `ObjectId`.

## 8. Build integration

- `Manifest::lua_execution` participates in `project::fingerprint` automatically
  (the manifest is byte-hashed), and explicitly in `scene_dependencies::apply_settings`
  as `scene-lua-settings`, consumed by `project::stage_with_playback`.
- `sources.mk` gains the mode define and, for VM modes, the extra sources and
  `LIBRARIES` entry. Library paths are hashed as build inputs automatically.
- A mode change invalidates execution artifacts; staging keys are mode-scoped so an AOT
  object cannot survive into a VM build. Compilation failure must not leave a stale
  artifact runnable and must not fall back silently.
- Exports stage through the same path and must rebuild standalone.

## 9. Acceptance

The same `.lua` files must produce equivalent results in all three modes for properties,
inheritance, calls, events, references, pause, destruction and save/load; plus numeric
edge cases, unsupported-feature diagnostics, mode changes in both directions, cache
invalidation and clean exports.

## 10. Backend interface (fixed for M3/M4, implemented concurrently)

### 10.1 Value ABI shared by every VM boundary
Every value crossing between generated C++ and Lua is an `int32_t`:
`Bool` 0/1, `Int32` as-is, `UInt32` as its bit pattern, `Fixed` as Q12 raw,
`Enum` as its integer value, `ObjectRef/ActorRef/ComponentRef` as
`int32_t((generation << 16) | index)` of `epok::ObjectId`. `AssetRef`/`ClassRef` (64-bit)
are Inspector-editable native fields but are not readable/writable from Lua bodies in
profile v1 (frontend diagnostic); they carry no field slot at all, and the builtins that
take one read the native field inside the generated `builtin_call` case. `self` is a light userdata pointing at the native
`epok::Object`; it is only valid inside the current call (the profile has no storage
that could retain it).

### 10.2 Normalized Lua chunk (emitted by `lua_vm.rs`, one per class)
```lua
local C = {}
function C.<method>(self, p0, p1) ... end   -- one per MethodIr, name = schema::Function.name
return C
```
Bodies use only: locals, `if/elseif/else`, numeric `for` with literal bounds, `return`,
Lua booleans for `Bool`, and calls to the registered globals below. No raw `+ - * / %`
on user values ever appears — every arithmetic op is a helper call so semantics are the
runtime's `epok::bp::*` functions:
`__epok_iadd/isub/imul/idiv/imod/ineg`, `__epok_uadd/usub/umul/udiv/umod`,
`__epok_fadd/fsub/fmul/fdiv/fneg`, `__epok_from_int/__epok_to_int`,
`__epok_ult/ule` (unsigned ordering; signed ordering uses Lua `<` on int32 which matches
C++ for Int32/Fixed/Enum), field access `__epok_getf(self, slot)` /
`__epok_setf(self, slot, v)`, self dispatch `__epok_call(self, method_slot, ...)` (goes
through the C++ virtual so overrides in derived classes are honored exactly like AOT),
parent dispatch `__epok_super(self, method_slot, ...)` (qualified `Parent::m`), and
builtin dispatch `__epok_builtin(self, site, ...)` (the generated `epok::bp::api` case
for that call site).
Field slots and method slots are per-class dense indices published in the generated
`ClassBinding`; inherited reflected properties get slots too.

### 10.3 Generated C++ per Lua class (emitted by `lua_aot.rs` for BOTH modes)
Same shell in every mode: `class Name : public Parent { static_class_id; class_id()
override; fields for own properties; ctor applying defaults; ... }` written to
`scripts/generated/lua/<class-stem>.hpp`, the class UUID or `lua_<Name>` (§4). Method
bodies:
- Native mode: lowered IR (`epok::bp::*` helpers, `Parent::m(...)` for CallParent,
  `this->m(...)` for CallSelf, `#line` directives back to the `.lua` source).
- VM modes: trampolines against `runtime/lua_runtime.hpp`:
```cpp
void tick(epok::Fixed dt) override {
    epok::lua::Frame f(*this, kEpokLuaClass_<index>, /*slot*/ 2);
    if (!f.bound()) return;             // absent handler: never enters Lua
    f.arg(dt.raw()); f.call(0);
}
epok::Fixed damage(epok::Fixed amount) override {
    epok::lua::Frame f(*this, kEpokLuaClass_<index>, 3);
    if (!f.bound()) return {};
    f.arg(amount.raw()); if (!f.call(1)) return {};
    return epok::Fixed(f.ret(), epok::Fixed::RAW);
}
```
plus, in VM modes, one `epok::lua::ClassBinding` per class emitted into
`scripts/generated/lua/lua_bindings.cpp` (a native source): field get/set switch over
slots (own + inherited reflected fields, vector components as separate slots), a
`self_call` switch (C++ virtual call by slot) and a `super_call` switch (qualified
parent call by slot), the method-name table, and a pointer to the chunk payload
(`lua_vm.rs` supplies the payload bytes and its symbol name).

### 10.4 Runtime API (`runtime/lua_runtime.hpp`, namespace `epok::lua`, VM modes only)
```cpp
struct ClassBinding {
    uint64_t class_id; const char* name;
    const unsigned char* chunk; size_t chunk_size;        // source (mode 2) or bytecode (mode 1)
    const char* const* methods; uint32_t method_count;    // slot -> Lua function name
    int32_t (*get_field)(Object&, uint32_t slot);
    void (*set_field)(Object&, uint32_t slot, int32_t value);
    int32_t (*self_call)(Object&, uint32_t slot, const int32_t* args, uint32_t argc);
    int32_t (*super_call)(Object&, uint32_t slot, const int32_t* args, uint32_t argc);
    int32_t (*builtin_call)(Object&, uint32_t site, const int32_t* args, uint32_t argc);
};
extern const ClassBinding class_bindings[]; extern const uint32_t class_binding_count;
void initialize();   // static arena, lua_newstate, register helpers, load every chunk once,
                     // cache method functions in the registry, build per-class bound bitmask
class Frame {        // one Lua call; wraps ObjectDispatchScope; validates the object
public:
    Frame(Object& self, uint32_t class_index, uint32_t slot);
    bool bound() const;            // false when the class has no Lua body for this slot
    void arg(int32_t value);
    bool call(unsigned results);   // pcall; on error: report once, return false
    int32_t ret() const;
};
}
```
`initialize()` runs in `GameScene::start` after `install_actor_service_hooks()` and
before `epok::initialize_scripts()`, guarded by `#if EPOK_LUA_MODE != 0`. The arena is a
static `EPOK_LUA_ARENA_BYTES` buffer (default 96 KiB); exhaustion aborts with a clear
message, never silently. `luaI_sprintf/luaI_realloc/luaI_free` are provided by the
runtime itself; `libpsyqo-lua.a` is NOT linked. Only the registered `__epok_*` helpers
exist as globals — no `load`, `dofile`, `require`, or standard libraries.

### 10.5 Packaging
- Mode 2 (source): chunk = normalized Lua text.
- Mode 1 (bytecode): chunk = `lua_bytecode::cook(text)` — pinned psxlua parser compiled on
  the host by `build.rs` with a 32-bit-ABI dumper (`native/lua/epok_ldump32.c`) whose
  header is asserted byte-for-byte equal to `1B 4C 75 61 52 00 01 04 04 04 04 01 19 93 0D
  0A 1A 0A`; an emulator test compares host-cooked bytes with a target `luaU_dump`.
- Archives: `runtime/lua.mk` builds `liblua-epok-parser.a` (parser) or `liblua-epok-noparser.a`
  from `$(NUGGET_DIR)/third_party/psxlua/src` into the build directory with the
  upstream PSX flags, so the two variants never share object files.
