# Blueprints, reflection, and inheritable classes in Epok

Date: 2026-09-08. Historical research and architecture proposal; see the current
implementation update below before interpreting historical status statements.

## Expanded implementation update — 2026-09-08

The user's expanded request now includes the implemented Blueprint asset format,
typed IR/native compiler, graph editor, defaults/inheritance/events/Call Parent,
continuations/timelines, checked references and cross-instance calls, component
templates and typed dynamic factories, closed host construction and instrumented
PCSX-Redux node debugging. See [validation status](validation-status.md) for
current evidence, measured costs and limits, and [completion gates](completion-work.md)
for completed acceptance. [Blueprints](../../../docs/blueprints.md) is the user guide.

The native ImGui canvas reuses the editor's existing context/draw lists rather
than introducing a second ImGui dependency. Same-flags native/BP measurements
and real emulator acceptance are now available; no general FPS benefit, external
asset/API parity or physical-console verification is claimed. Implementation and
final visual acceptance are complete; publication is a separate release process.
Lua and post-BP VFX remain separate.

Current validation: 154 default Rust tests passed with seven explicitly ignored;
the extended ImGui interaction test passed clipping, search, component-selection
preservation and Add/Undo. Explicit real-extractor resource-migration, inheritance/
standalone-export, native feature and live debugger acceptance passed. Final native
visual QA passed against both supplied references, with the capture saved as
`docs/images/blueprint-editor.png`. Native Compile, checkbox editing/Undo clearing
dirty state, Components selection, Compute Strength and Class Defaults passed.
The historical outstanding work below is not the current completion list.

The foundation update below and remaining numbered research sections retain
their original point-in-time scope; they do not override this current record.

## Implemented foundation update — 2026-09-08

The first foundation is now implemented and validated; see [foundation status](foundation-status.md) for exact delivered behavior, local checks, supported limits, and unavailable desktop checks. The sections below retain the original design/research context rather than silently replacing it. Their historical “not implemented” statements describe the pre-implementation investigation, not the current source.

Decisions validated in code: isolated pinned libclang against real MIPS/PsyQo headers; semantic UUID/USR metadata rather than duplicated JSON; one picker/Inspector registry; original annotated C++ assets with inherited behavior; explicit typed overrides and in-memory scene migration; backend-owned native code generation/reset plus reserved non-C++ resource/invocation boundaries; authoritative `.epokproject` descriptors with explicit locked recovery. No Lua VM or graph editor is implemented. Native inheritance, independent values, standalone rebuild and bank reset passed MIPS/PCSX-Redux acceptance. Live Windows creation, inherited Inspector values, attachment Undo/Redo, saving overrides and Reset to Inherited passed after unlocking; real ImGui interaction and isolated registry tests also passed. Native picker input timed out in desktop automation, and actual registered double-click remains unverified without changing live associations.

The next implementation is a data-only Blueprint asset compiler, followed by a small typed executable graph and Call Parent. No performance benefit or full feature parity is claimed.

## Recommendation

Build a shared class system for C++, Blueprints, the Inspector, and the class picker. Generate reflection from annotated C++ declarations, represent each Blueprint as a class asset, and compile its graphs to C++ before the existing MIPS build.

Reserve an optional Lua script provider backed by Nugget's `psyqo-lua`. Lua is neither implemented nor linked at this stage; the class, property, and execution models must accommodate it without redesigning persistence or the picker. See section 12.

Include a project entry-file workflow in the foundation milestone: a root-level `MyGame.epokproject` descriptor with JSON contents, folder/file opening, and migration from the existing manifest. This is a planned addition, not current behavior; see section 14.

The product goal is class-based visual authoring: choosing a parent class, inheriting members and defaults, overriding events, connecting nodes, and creating instances. The proposed PSX implementation uses its own ahead-of-time compilation and reduced console-side reflection. Full feature parity includes much more than a node canvas and requires several milestones.

Generating C++ is a recommendation based on the existing pipeline and PSX resource constraints; there are no measurements yet demonstrating its cost relative to a compact VM. Validate it with a vertical prototype before expanding the language.

## 1. Current implementation

These findings come from the current working tree, which already contains changes for other features. This research introduces a design document only.

| Code location | Finding | Consequence |
| --- | --- | --- |
| [scripts.rs](../../../src/scripts.rs), `Script`, `Property`, `catalog` | Manual `.epokscript` descriptor, a name, and up to 16 `f32` properties; flat traversal of `assets/scripts` | No discovery of classes, functions, types, or parents |
| [scripts.rs](../../../src/scripts.rs), `create` | Generates `BehaviourNNN final : public epok::Behaviour` and a rotation example | `final` prevents deriving from generated scripts; name, folder, and parent selection are missing |
| [scene.rs](../../../src/scene.rs), `ScriptBinding`, `Entity` | One `Option<ScriptBinding>` per entity; `BTreeMap<String, f32>` values | The first version can retain one behaviour per entity; typed values and stable identity are needed |
| [editor.rs](../../../src/editor.rs), `attach` | Copies every default into the instance | Inherited values cannot be distinguished from explicit overrides |
| [gui.rs](../../../src/gui.rs), Script Inspector | Materializes defaults through `entry(...).or_insert(...)` and renders numeric controls | Reading a property must not turn it into an override in the new model |
| [epok.hpp](../../../runtime/epok.hpp), `Behaviour` | Polymorphic base with `start`, `update`, `frame_update`, activation, destruction, and trigger hooks; `update` is pure virtual | Useful entry points already exist; concrete classes must implement or inherit `update` |
| [project.rs](../../../src/project.rs), `scene_header_body`, `stage_into` | Emits C++ instances and property assignments; copies catalog `.hpp/.cpp` files and generates `sources.mk` | A natural backend for Blueprints exists; dependencies and generated code must be included |
| [scene_bank.rs](../../../src/scene_bank.rs), bank loading | Resets behaviours through `Class{}` and assignment | Class state must respect the existing contract or use typed reset operations instead |
| [assets.rs](../../../src/assets.rs), C++ indexing | Script visibility depends on the current descriptor | Blueprints and new descriptors also require integration here |
| [project.rs](../../../src/project.rs), `fingerprint` | Observes immediate files in `assets/scripts` | Subfolders, shared headers, and Blueprints require transitive invalidation |
| [architecture.md](../../architecture.md) | Rust/ImGui/wgpu editor; gameplay runs in PCSX-Redux with 2 MB RAM | The editor cannot load or execute MIPS classes as native host classes |
| [psyqo.mk](../../../third_party/nugget/psyqo/psyqo.mk), [common.mk](../../../third_party/nugget/common.mk) | C++20, `-fno-exceptions`, `-fno-rtti` | The solution must not depend on standard RTTI, exceptions, or a newer C++ version |

`Entity` is currently a component container, not a hierarchy equivalent to `AActor`. Deriving from an arbitrary SDK class does not automatically make a script attachable.

## 2. Reflection and visual-scripting patterns

Class and member annotations with a code-generation step before C++ compilation are an appropriate pattern for discovering the user API. Querying `typeid` is insufficient.

A Blueprint is an asset defining a class that can extend C++ or another Blueprint. The picker and Class Viewer operate on that hierarchy and filter valid bases.

A historical compiler design describes validation and dependency scheduling, generated classes, default values, an intermediate representation, and bytecode emission for a VM. It is a conceptual reference, not evidence that every internal detail remains identical in a current implementation. Epok can adopt the frontend/IR/backend separation and provide its own C++ backend.

## 3. Shared architecture

```text
Annotated C++ headers --> extractor --------+
                                           +--> Class and type registry
Blueprint assets ------> declarations -----+              |
                                          +---------------+---------------+
                                      Class Picker     Inspector      Node catalog
                                                                          |
                                                       Graphs --> validation --> typed IR
                                                                          |
                                                          Generated C++ + native code
                                                                          |
                                                         Existing staging --> Make/MIPS --> PSX
```

The IR is an intermediate representation independent of ImGui and the visual format. The catalog can describe a class without opening its graph or executing game code.

### Editor registry

Proposed descriptors: `TypeInfo`, `ClassInfo`, `PropertyInfo`, `FunctionInfo`, `EnumInfo`, and `StructInfo`. A class needs persistent identity, a qualified C++ name when applicable, an authoring provider, an execution backend, a parent, flags, declaration location, members, and defaults. Separate authoring (`cpp`, `blueprint`, future `lua`) from execution (`native`, future `lua-vm`): a Blueprint compiled to C++ remains a Blueprint asset. A function needs a typed signature, parameter directions, return type, visibility, and call/override capabilities.

Initial flags: `Abstract`, `Final`, `Blueprintable`, `Attachable`, `Editable`, `ReadOnly`, `Callable`, `Pure`, `Event`, and categories. Distinguish inheritance eligibility, Blueprint creation eligibility, and instantiation/attachment eligibility: an abstract class can be a valid parent.

Unifying native and visual classes here avoids three incompatible lists for the picker, nodes, and Inspector. Expose input, transform, audio, and scene APIs through adapters with simple annotated signatures.

### Reflection on PSX

Keep descriptive metadata, documentation, and node positions on the host. Emit only the data the game needs: compact IDs, type relationships, and generated accessors, invokers, and factories where dynamic operations require them. Statically resolved nodes call C++ directly.

`is_a`, class references, and checked casts would use the custom registry without `dynamic_cast`. Do not calculate polymorphic member offsets with `offsetof` or copy host offsets to MIPS: generate typed access compiled for the target. Check ID/hash collisions during cooking; do not use `typeid().hash_code()` or names as persistent identity.

Enumerating and invoking every member by name during gameplay would require an additional runtime reflection profile, with deliberately retained tables and measured costs. The initial profile retains what the program uses and does not promise universal introspection.

## 4. Extracting C++ reflection

Use a host executable, `epok-header-tool`, based on Clang LibTooling and invoked during game build preparation. Clang supports tools that traverse the AST and use a compilation database. Source: [LibTooling](https://clang.llvm.org/docs/LibTooling.html).

Illustrative proposed syntax, not an existing API:

```cpp
#pragma once
#include "epok.hpp"
#include "reflection.hpp"

EPOK_CLASS(Blueprintable)
class Enemy : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere, BlueprintReadWrite, Category="Combat")
    epok::Fixed health = 100.0;

    EPOK_FUNCTION(BlueprintCallable)
    void take_damage(epok::Fixed amount);

    EPOK_FUNCTION(BlueprintEvent)
    virtual void on_death() {}

    void update(epok::Transform&, epok::Fixed) override {}
};
```

The macros produce annotations recognized by the extractor without affecting the normal GCC build. `on_death` would be a virtual method with a native default implementation; `take_damage` can invoke it and reach the generated Blueprint override. Nonvirtual functions can be callable without being overridable. These macro names are a proposed Epok API, not reused external macros.

The extractor emits a versioned manifest consumable from Rust, access/registration code where needed, and file/line diagnostics. Headers are the semantic authority; a versioned sidecar may store IDs and rename redirects, but must not manually duplicate signatures or types. The wizard assigns persistent IDs; renaming preserves identity or requires an explicit migration.

Requirements to validate before adopting this approach:

1. Parse real headers with C++20, SDK defines, and include paths, including namespaces and inheritance. Pin a distributable Clang version in the editor dependencies.
2. Generate `compile_commands.json` from effective build arguments. Translate GCC-only flags and configure the MIPS target and its includes; parsing against Windows headers does not validate the game. Clang documents the need to configure the target, sysroot, and paths explicitly: [CrossCompilation](https://clang.llvm.org/docs/CrossCompilation.html).
3. Verify PsyQo compatibility. If the extractor cannot handle some SDK constructs, isolate a compatible declaration facade and check it against the MIPS build; do not silently hide errors behind divergent stubs.
4. Extract defaults from a declarative subset: bool, integers, enums, Fixed literals, and supported aggregates. Do not evaluate arbitrary constructors or function calls by executing MIPS code in the editor. Require an explicit declarative default and a verifiable precedence rule for nonconstant cases.
5. Reject unsupported signatures with useful diagnostics: varargs, open templates, multiple behaviour inheritance, and pointers without a lifetime policy. Unexposed C++ remains available for internal game implementation.
6. Cache by transitive header content, options, extractor version, and schema. Retain the last valid version for navigation, label it stale, and block builds when used classes fail validation.

Alternatives considered:

| Option | Assessment for Epok |
| --- | --- |
| Extended manual JSON | Useful as an adapter for existing scripts; maintaining two manual definitions still permits divergence |
| Custom C++ parser | Reasonable only for an explicitly restricted grammar; regex does not handle general C++ |
| Clang AST + annotations | Recommended; greater initial distribution and configuration cost, better semantic information |
| EnTT meta or RTTR | Provide C++ registration and introspection, but still require the Rust editor bridge, extraction, assets, and visual compiler |

Registration APIs are documented in [EnTT meta](https://github.com/skypjack/entt/wiki/Runtime-reflection-system) and [RTTR registration](https://www.rttr.org/doc/master/classrttr_1_1registration.html). These libraries have not been benchmarked on PSX here and are not claimed to be incompatible. The reason to prefer custom generation is control over the host/target contract and retention of only the required data.

## 5. Inheritance and script creation

Use one creation dialog with asset type, name, folder, and parent class. Provide incremental search, a complete tree, common bases, and origin filters. Show inherited members, documentation, and the reason a class is ineligible. Add a Create Derived Class action to the class context menu.

| New asset | Initially supported bases | Output |
| --- | --- | --- |
| C++ script | `Behaviour` and valid public C++ classes that are not `final` | `.hpp/.cpp` with the correct include and an identity descriptor |
| Blueprint class | `Blueprintable` C++ classes and other valid Blueprints | Class asset referencing its parent, defaults, members, and graphs |
| Data-only Blueprint class | The same bases | Inherits behaviour and changes defaults without requiring nodes |
| Lua script — future, provider initially unavailable | Bases enabled by the Lua adapter; later Lua classes | `.lua` source and a typed declaration for the shared registry |

The abstract `Behaviour` root must be selectable. When generating a concrete class directly from it, the wizard implements `update`; when deriving from a class that already implements it, inherit that implementation unless the user requests an override. Do not generate an empty `update` that accidentally suppresses parent behaviour. Make `final` an explicit option, disabled by default for user classes.

Target chain: `Behaviour → Enemy (C++) → BP_Enemy → BP_Boss`. Each level can add properties and behaviour. An override replaces the inherited body; a Call Parent node emits a qualified parent call and avoids recursive virtual dispatch.

Initially disallow handwritten C++ classes derived from Blueprints. Although the backend generates C++, treating generated classes as bases for handwritten code introduces generation dependencies and cycles that should be avoided. Reverse interoperability uses virtual events and explicit native bases/interfaces. This is a rule of the proposed design.

Also validate base visibility, constructibility, assignment/reset, unresolved abstract methods, reserved C++ names, duplicates, and Windows case-only collisions. Create files transactionally: cancellation or failure must not leave a partial script. Create and Create and Attach are distinct actions; attachment must record undo and handle an existing behaviour.

## 6. Defaults, typed properties, and migration

Initial types: `Bool`, `Int32`, `Fixed`, `Vec2`, `Vec3`, enums, and small registered structs. Introduce `EntityRef`, `AssetRef`, and `ClassRef<Base>` with explicit resolution and validation. Arrays have declared capacities; dynamic text and collections must not enter the runtime implicitly.

Resolve values in this order: declarative native default → overrides at each derived class → instance overrides. Unmodified values come from the parent; the Inspector stores only explicit edits and offers Reset to Inherited. A child class changing a default does not alter parent instances or sibling classes.

Keep class, property, function, node, and pin identity independent of display names. Use deterministic serialization, versioned formats, and rename redirects. Store members declared at each class level and resolve inherited members instead of copying them into each child. Separate visual layout from semantics so moving nodes does not force game recompilation.

Load existing scripts through a legacy adapter. Initial migration preserves all serialized values as explicit overrides: it is no longer possible to know whether the user edited them or they were copied defaults. Do not infer intent from numeric equality. Offer user-selected restoration of inheritance afterward. Do not automatically remove `final` from existing scripts.

Existing bindings use names, and entities do not have an authoring UUID. Add stable entity identity before supporting reliable persistent references; during loading/cooking, convert references to slots/handles and resolve them before `start`. Do not serialize an `EntityHandle` from a previous run or treat an entity name as a unique identifier.

Removed fields or incompatible types must produce diagnostics and preserve recoverable orphaned data until migration. Reparenting validates members, events, pins, and references before acceptance; never silently delete connections. Increase the scene version and provide migration because the current loader only accepts version 1.

## 7. Blueprints as a compiled language

An asset must contain its UUID, version, parent, variable/function declarations, default overrides, graphs, node/pin/link IDs, and layout. Initial proposal: `.epokbp` authoring files with a derived index under `.epok/`; settle integration with existing asset packages when implementing the indexer.

Compiler pipeline:

1. Index declarations from every class and order inheritance; detect missing parents and cycles.
2. Resolve functions and properties by ID and validate types, access, context, arguments, and returns.
3. Distinguish execution flow from data dependencies. Start with Branch, Sequence, and bounded structured loops; reject stateless data cycles and Exec connections outside supported control flow.
4. Lower to typed IR with blocks, temporaries, calls, stores, branches, and returns. Each operation retains its source node ID for errors and traces.
5. Define evaluation order: evaluate pure reads per use/consumer under a documented rule; do not cache reads across writes or duplicate effectful nodes. Random and input consumption are not pure expressions for visual convenience.
6. Emit one C++ class per used visual class, methods per function/event, and per-instance state. Share code across instances, resolve defaults without duplicating graphs, and maintain explicit dependencies in `sources.mk`.
7. Compile with MIPS and map generated-code errors to assets/nodes. Exports must include all required C++ and data without requiring Clang or the editor to rebuild the exported snapshot.

Arithmetic must preserve Q12, conversions, rounding, and the language's overflow/division-by-zero policies. Host validation must not silently substitute floating-point behaviour for PSX semantics. Check visual graph cost limits; calls to arbitrary C++ still require profiling.

Initial nodes: Start, Update, Frame Update, Enable/Disable/Destroy, Trigger, Get/Set Property, registered C++ calls, Self, Branch, Sequence, bool/int/Fixed operations, vector construction/decomposition, transform, input, and audio. Add APIs through small adapters; do not promise automatic exposure of every C++ function.

`Delay`, Timeline, and latent functions need stateful continuations: a counter/program counter, variables surviving suspension, an owner handle, and a maximum capacity. Never block a frame. Cancel on destruction/scene replacement and define explicit deactivation/pause policies. Use simulation time or frame time according to the contract. Ordinary synchronous functions do not allow `Delay`; latent events belong to a later phase.

## 8. Node editor and debugging

`imgui-node-editor` provides selection, movement, zoom, links, and Blueprint-inspired styling. It is the most direct visual candidate. Its Blueprint example depends on an ImGui layout extension and must not be copied as though it were independent. Prototype a small C wrapper and Rust bindings against the same ImGui copy used by `imgui-sys`; avoid linking a second incompatible copy/context. Source: [official repository](https://github.com/thedmd/imgui-node-editor).

`imnodes` has Rust bindings and, in the inspected branch manifests, depends on `imgui = 0.12` and `imgui-sys = 0.12`, matching Epok. This establishes declared version compatibility, not tested docking, input, and wgpu integration. Sources: [Rust manifest](https://raw.githubusercontent.com/benmkw/imnodes-rs/main/Cargo.toml), [FFI manifest](https://raw.githubusercontent.com/benmkw/imnodes-rs/main/imnodes-sys/Cargo.toml), [API](https://docs.rs/imnodes/latest/imnodes/).

Provisional decision: try `imgui-node-editor` first for interaction fidelity; use `imnodes` if wrapper cost or actual compatibility is worse. Pin revisions and retain licenses when incorporating either. Both are frontend libraries; Epok still owns the graph model, validation, undo/redo, and compiler.

The target experience includes a context menu filtered by the dragged pin, distinct Exec pins, type colors, inherited variables/functions, a defaults Inspector, parent-class breadcrumbs, reroutes, comments, duplication, copy/paste, and navigable compiler messages.

Initial debugging: per-node errors, inspectable generated C++, and optional traces with node/instance IDs. Breakpoints, variable inspection, and stepping need instrumentation, symbol mapping, and an extension to the PCSX-Redux bridge; existing image capture does not provide them. Traces must be bounded and droppable, with an overflow counter.

Compiling or editing during Play does not imply safe hot reload. The first version rebuilds and restarts Play. Editor construction scripts require another host backend and a controlled editing API; loading a manifest cannot execute the MIPS runtime inside Rust. That capability belongs to a separate phase.

## 9. Instantiation and Actor Blueprint parity

The first deliverable is a behaviour Blueprint attached to an existing entity. Supporting an Actor Blueprint workflow additionally requires an entity/component template containing hierarchy, components, resources, and inherited overrides that can be placed in scenes and instantiated.

Do not convert all existing components into a C++ Actor hierarchy. Introduce a gameplay class that owns/references an entity and a component template while preserving existing storage. `Character`, `Pawn`, and `GameMode` should appear as bases only once Epok provides corresponding implementations and contracts.

`create_entity` alone does not instantiate a selected script class. `Spawn(ClassRef<Base>)` requires per-type factories, behaviour capacity, typed creation/reset/destruction, and dynamic binding registration. The current scene-bank `TableView<Binding>` is prebuilt and must be extended. Return explicit failure when capacity is exhausted.

Preserve generations and the existing lifecycle order. Resolve references and bind all scripts before callbacks. Do not use `delete Behaviour*`: the current base lacks a virtual destructor, and the proposal uses bounded storage with typed teardown. Scene changes cancel owner continuations and pending events before reusing slots.

## 10. Phases and acceptance criteria

The first foundation milestone also includes the project descriptor tasks and acceptance criteria in section 14. Resolve the project root before initializing reflection, asset discovery, or provider caches so opening a descriptor or its folder uses the same project context.

| Phase | Deliverable | Acceptance criterion |
| --- | --- | --- |
| 0. Feasibility | Extract a real header, emit a minimal graph to C++, and prototype the canvas | Build with the pinned SDK, run in the emulator, match handwritten C++ behaviour, and record RAM/code/time |
| 1. Classes and reflection | Registry, legacy adapter, typed values, C++ wizard with class picker | Create `Enemy` and `Boss : Enemy`, inspect inherited properties, and preserve base behaviour and values through save/load/export |
| 2. Visual data classes | Blueprint asset, native/visual parent, overrides, and migration | For `BP_Enemy → BP_Boss`, a parent default edit updates only descendants/instances without overrides |
| 3. Executable graphs | Node UI, IR, C++ backend, events, and calls | An enemy receives damage, runs an overridden event, and calls its parent; errors identify nodes and the export builds independently |
| 4. Reusable logic | Visual functions, bounded loops, events/continuations, and traces | Delay does not block, two instances do not share state, and destruction/scene changes cancel pending work |
| 5. Entity classes | Component templates, scene placement, and class spawning | Repeated creation/destruction neither loses capacity nor incorrectly preserves handles across slot reuse |
| 6. Advanced parity | Construction scripts, expanded interfaces, visual debugging, and possible compilation during Play | Each capability has a backend, lifetime policy, and measurements before claiming full parity |

High-value tests: signature/visibility/abstract/final extraction, multilevel inheritance and Call Parent, default/ID migration, broken references without data loss, effect evaluation order, host/target Fixed equivalence, transitive invalidation, independent instances, bank resets, and editor-independent exports.

Measure the prototype with representative instance counts and per-frame workloads. Compare handwritten and generated C++ under identical flags and scenes: EXE size, linked sections, data/pool/continuation RAM, stack, and CPU time. Do not allocate an additional RAM budget without checking the current game and renderer. AOT can save VM dispatch while increasing code size; a compact VM remains an alternative if measurements favor density or iteration.

This is a project with several milestones, not a small UI improvement. There is no reliable basis for a full-parity delivery date before the prototype and a defined advanced-feature scope.

## 11. Proposed implementation map

Illustrative module names; these modules have not been created:

| Modules | Responsibility |
| --- | --- |
| `src/reflection.rs`, `src/class_registry.rs` | Shared schema, loading, and type/inheritance resolution |
| `src/script_wizard.rs`, `src/class_picker.rs` | Transactional creation and parent selection |
| `src/script_backend.rs` | Editor provider contract: capabilities, metadata, validation, and artifact preparation; no active Lua provider |
| `src/blueprint.rs`, `src/blueprint_ir.rs`, `src/blueprint_compile.rs` | Assets, validation, IR, and C++ emission |
| `src/blueprint_editor.rs` | Canvas and editing operations with undo/redo |
| `tools/epok-header-tool/` | Host extractor and manifest generation |
| `runtime/reflection.hpp`, later `runtime/blueprint_runtime.hpp` | Minimal typed access and bounded dynamic/latent operation support |
| Existing `scripts`, `scene`, `editor`, `gui`, `assets`, `asset_ui` modules | Legacy compatibility, persistence, indexing, Inspector, and window integration |
| Existing `project`, `scene_bank`, `export`, `pipeline` modules | Generation order, transitive dependencies, staging, reset, and export |
| `workspace` and game templates | Inheritable new scripts and examples consistent with the wizard |

New class/graph operations can be exposed through the editor MCP by reusing validation, revisions, and undo. Do not maintain a second generation path without the same invariants.

## 12. Reserved Lua integration through PsyQo/Nugget

Scope: prepare the architecture for future Lua scripts without implementing the VM, bindings, or an operational Lua wizard now. This reservation belongs in the design of phases 1–3; adding Lua does not need to wait for advanced Blueprint parity.

### Available foundation and verified limits

Nugget includes [psyqo-lua](../../../third_party/nugget/psyqo-lua/README.md), its [C++ wrapper](../../../third_party/nugget/psyqo-lua/lua.hh), [implementation](../../../third_party/nugget/psyqo-lua/src/lua.cpp), and [Make rules](../../../third_party/nugget/psyqo-lua/psyqo-lua.mk). It integrates a VM running inside the PSX game, distinct from the host Lua used by the PCSX-Redux bridge. This is also documented in the [official repository](https://github.com/grumpycoders/pcsx-redux/tree/main/src/mips/psyqo-lua).

The [pinned psxlua README](../../../third_party/nugget/third_party/psxlua/README) identifies a Lua 5.2.4 fork with 32-bit integer numbers and without the `os`, `io`, `math`, and `package` libraries. PsyQo separately provides `push`, `toFixedPoint`, and `FixedPoint` metatables. Scripts therefore need the explicit Fixed API; a Lua decimal must not be treated as automatically representing Q12.

That README estimates a library binary footprint of roughly 110 kB with the parser and 85 kB without it. These are upstream estimates, not measurements of an Epok executable or heap, table, stack, or GC usage. The inspected `psyqo-lua` Makefile links `liblua.a`, including the parser. Also, `setupFixedPointMetatable` loads embedded source during initialization: switching to `liblua-noparser.a` would require adapting that startup path. Do not use bytecode from a generic `luac`: pin compatible version, ABI/numeric format, and toolchain, and validate on the target.

### Contracts to reserve from the beginning

| Extension point | Design rule |
| --- | --- |
| Editor provider | Query capabilities/availability, obtain class declarations, validate, and produce build artifacts. Compile implementations into the host; a dynamically loaded plugin system is unnecessary |
| Persistence | Shared `ClassId`, member IDs, typed values, and overrides. Versioned backend/provider identifiers; preserve assets for unavailable backends and diagnose them without reinterpreting them as C++ |
| Build | Script preparation produces native sources, packaged resources, dependencies, and required runtime capabilities. Do not assume every class produces exactly one `.hpp/.cpp` pair |
| Reflection and Inspector | Names, types, defaults, and signatures come from the registry. Future Lua classes supply a typed manifest without executing scripts for inspection |
| Invocation | Known functions retain direct C++ calls. Reserve typed access/call adapters for interpreted destinations without converting the entire runtime to dynamic dispatch |
| Instance lifetime | Initialize/bind, apply defaults, dispatch events, cancel tasks, and release/reset resources through provider operations. Never assume copying `Class{}` resets every backend correctly |
| Picker | Ask the provider which parents and operations it supports. Registering a class does not imply that every language can derive from it |

Interpreted instance state remains private to its backend. Serializable API values remain `Bool`, `Int32`, `Fixed`, structs, and typed references; do not store `lua_State*`, stack indices, or Lua tables in scenes. Class identity and logical type compatibility do not authorize a memory cast to a C++ class that is not physically present in the object.

### Future C++ and Blueprint integration

For attaching Lua scripts, propose a native adapter implementing `Behaviour`, with an independent table/environment per instance and checked owner references. One shared VM per game session is the starting point to measure; do not create one VM per entity. The provider manages registry references and explicit closure: the inspected wrapper permits copies of a VM reference and exposes `close()`, so VM ownership must remain outside individual behaviours.

For Lua to extend a C++ base such as `Enemy`, later generate an adapter derived from `Enemy` that forwards enabled events to Lua while retaining native members and a qualified parent call. A Lua table alone neither inherits the C++ layout nor overrides its vtable. Start with `Behaviour` and explicitly supported bases; do not promise arbitrary cross-language inheritance among C++, Blueprints, and Lua. Lua-to-Lua inheritance can use prototypes/metatables and the shared registry once the provider implements it.

Blueprint/Lua calls use functions/events declared in the same registry and typed wrappers with explicit return/error policies. The visual compiler retains call destination and capability in the IR rather than assuming every `FunctionId` maps directly to a C++ symbol. Declare interpreted functions and their API as dependencies so cooking retains their wrappers. This does not change the initial Blueprint backend to Lua.

### Conditions for enabling Lua later

Enable it as an optional project/build capability and package scripts from known assets. With Lua disabled, C++/Blueprint games must not link the VM, bindings, or Lua resources. Using a Lua class without an available/enabled provider must produce an actionable error rather than ignore the behaviour. Export the artifacts and rules needed to rebuild independently while respecting pinned SDK and `psxlua` submodule revisions.

Define memory, per-frame work, instruction budgets, and GC policy, then measure on PSX. Use protected script entry calls and return file/line/instance diagnostics. On error, disable or cancel the affected operation under a defined contract and leave the Lua stack in a known state. Lua error propagation differs from C++ exceptions: review wrappers so they do not depend on RAII destructors skipped by an error jump. Do not allow an unbounded Lua loop to block the game.

Entity references use generation-checked handles. Destruction and scene replacement release instance tables and cancel coroutines/tasks before slot reuse. Do not permit `yield` through a synchronous native call that does not support suspension; reuse the engine's latent-task contract. Specify pause/deactivation behaviour as well.

When implementing the early phases, validate the reserved contracts with a fake editor/build provider supplying a class without C++ source, properties, and diagnostics, without a real VM. Verify preservation of unavailable-provider assets, parent filtering, and authoring/execution separation. Confirm builds without Lua introduce no Lua dependencies. Actual support later needs tests for independent instances, Q12 conversion, native/visual callbacks, errors, cancellation, memory usage, and export.

## 13. Verified scope and remaining work

Verified: contracts and limitations in the cited local code; reflection, picker, and compilation workflows in Epic's primary documentation; published Clang and node-library capabilities; declared ImGui version alignment in `imnodes`.

Remaining work: compile the extractor against PsyQo headers, integrate the canvas, implement and profile the visual backend, settle the asset format, and validate PCSX-Redux debugging. Lua is a designed extension checked against the local SDK, with no Epok integration or measurements. No performance prototype or feature implementation has been completed during this research.

The recommended first implementation is **reflection + class registry + class picker + C++ inheritance**, designed from the outset to support Blueprint classes. Follow it with data-only Blueprints and a small executable graph from authoring through PSX execution. This order delivers useful functionality and validates the architecture before expanding the visual catalog.

## 14. Project descriptor and opening workflow

Status: requested addition to the Blueprint foundation milestone; not implemented.

### Current behavior and reference models

Epok currently opens a directory. [workspace.rs](../../../src/workspace.rs) canonicalizes that directory, reads `ProjectSettings/project.json`, validates the startup scene and script catalog, and acquires `.epok/project.lock`. The Hub uses a folder picker, and `--project` expects a directory. Selecting the manifest itself is not currently supported. See the current [project guide](../../../docs/projects.md).

Project managers can add existing projects by selecting their folder, while project descriptors can identify a project and store JSON configuration.

A custom extension identifies the file's purpose; JSON describes its contents. These choices are compatible. Opening an entry file identifies the surrounding project directory and its settings. It does not embed all assets in that file or require loading every asset into memory.

### Proposed layout and ownership

```text
MyGame/
  MyGame.epokproject       # Versioned JSON project descriptor
  assets/                  # Scenes, native scripts, Blueprints, and other assets
  ProjectSettings/         # Any remaining project settings with distinct ownership
  UserSettings/            # Local editor state
  .epok/                  # Generated output, caches, and project lock
```

Use `.epokproject` as the proposed extension. Keep the existing scene and asset extensions unchanged. The descriptor lives directly in the project root and is the single authority for the current manifest fields: format/editor versions, name, startup scene, auto-build, and rendering settings. Do not keep two writable copies of these settings in the new descriptor and `ProjectSettings/project.json`. Any later split of settings must define one owner for each field.

The parent directory determines the root; the descriptor's filename is not a persistent project or class ID. Store portable asset paths relative to that root and retain containment checks, including symlink handling. Copying the descriptor alone does not copy the game: moving or sharing a project still requires its assets and tracked settings.

### Foundation task list

- [ ] Implement one project-path resolver shared by the Hub, CLI, tests, and tooling. Accept either a project directory or an explicit `.epokproject` file and resolve both to the same canonical root. Directory discovery must reject multiple descriptors with an actionable diagnostic instead of choosing arbitrarily.
- [ ] Create the descriptor for new projects and update bundled templates and project-creation tools. Route all settings reads and writes through the resolved manifest location; remove hard-coded legacy-path assumptions.
- [ ] Add a native descriptor picker to Open Project while retaining folder opening. Accept `--project <file-or-folder>` and a positional descriptor argument for desktop launch. Preserve existing save/discard/cancel behavior when switching projects.
- [ ] Keep legacy `ProjectSettings/project.json` projects readable. Provide a versioned, recoverable migration that preserves every supported setting and asset path, validates before replacing the active manifest, and retains a backup outside active discovery. Do not silently rewrite a project on read. If both active formats exist, report ambiguity and offer an explicit migration/recovery path rather than merging or overwriting values.
- [ ] Normalize recent-project entries and locks by canonical root, not by the selected file path. Opening through the descriptor and folder must not create duplicate recent entries or bypass the existing single-project lock.
- [ ] Add Windows file type/icon registration and double-click launch through the distribution or installer workflow, with correct quoting for paths containing spaces and clean uninstall behavior. Keep portable builds usable through the Hub and CLI without requiring registration; do not mutate file associations merely by opening a project.
- [ ] Make reflection, Blueprint generation, provider configuration, staging, and standalone export consume the resolved project context. Keep generated caches under `.epok/` and preserve current relative asset paths and future Lua's disabled-by-default policy.
- [ ] Update the project and getting-started guides, templates, and affected integration fixtures in English after implementation. Explain that the descriptor opens a folder-based project and is not a scene, archive, executable, or container for its assets.

Acceptance: open the same new project by folder, descriptor, CLI, and registered double-click launch; obtain the same startup scene, settings, class catalog, recent entry, and lock. Validate legacy open and migration, interrupted migration recovery, malformed/unsupported descriptors, multiple descriptors, both active formats, cancellation, spaces and non-ASCII paths, and relocation of the complete project folder. Rebuild and export a project after migration. Report unavailable installer/desktop validation explicitly. This planning change does not implement any of these tasks.
