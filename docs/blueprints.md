# Blueprints

Start with [Your first Blueprint](blueprints-tutorial.md) for a guided editor
exercise. This page is the reference for types, inheritance and execution.
For visual gameplay, continue with [VFX authoring](vfx-editor.md) and the
[marker-driven spell tutorial](spell-tutorial.md). Keep
[troubleshooting](blueprints-vfx-troubleshooting.md) nearby when repairing graphs.

Blueprints are original `.epokbp` class assets, not generated C++ files.
They derive from a reflected native C++ class or another Blueprint. Generated
C++ runs through the native MIPS/PsyQo backend: no graph interpreter, C++ RTTI,
exceptions, Lua VM, or host-computed C++ member offsets are shipped to the PSX.

The current reflection/Blueprint toolchain is provisioned on Windows x64 and
Linux x86_64. It requires the pinned libclang and a built `epok-header-tool`
beside the editor. General macOS editor support does not yet provision that
Blueprint extractor runtime; Blueprint authoring is not currently supported
there. Existing legacy-script workflows are separate from this requirement.

## Create and edit

Use **New Blueprint** in the Project browser, or **New Blueprint** in Add
Component. Choose a reflected Blueprintable parent. Abstract parents are valid
authoring parents; implement all abstract events before attachment or spawning.
Final classes and final events cannot be extended.

Double-click the asset to open its canvas. **Edit Script** on a Blueprint
component opens the same asset. Use **Create derived class** to retain its
inheritance instead of copying its data. Native classes and visual descendants
share the class selector, component attachment, and typed Inspector.

The left-hand Components tree browses the class template; My Blueprint groups
graphs, functions and inherited variables with search and Add menus. Selecting
an item opens its controls in the right-hand Details panel. Class Settings and
Class Defaults are available in the toolbar. The type picker also lists enum
types already exposed by the native registry.

The Blueprint editor exposes class defaults, variables, functions, event
overrides, a searchable node catalog, and typed pins. Drag an output pin to an
input, move/select nodes, pan and zoom, and use Frame All to recover the graph.
Copy/paste remaps node/pin/link identities; undo/redo retains document history.
Comments and reroutes are authoring-only. **Compile** checks the current draft,
shows diagnostics with node navigation, and can show generated native code.
Save before building; unsaved drafts are never silently replaced by disk code.

Behaviour Blueprints start with **On Start**, **On Update**, and **On Trigger**
in one event canvas. Actor and Component Blueprints start with **Begin Play**,
**Tick** and **End Play** instead; see [Class families](#class-families).
These map to the PSX runtime's `start`, `update`, and
`on_trigger`; Trigger provides the other entity and Enter/Stay/Exit phase.
The corresponding collider must be configured as a trigger to receive overlaps.
Opening an older Blueprint adds missing supported events as an undoable draft
edit; save to retain them. Existing event graphs are preserved. Implemented
parent events receive a **Call Parent** node so inherited logic keeps running.
Other lifecycle events remain available through **Add > Override event**.

Double-click a data or execution wire to insert a **Reroute** point. Drag its
center to organize the wire, or connect to its sides to add branches. The point
retains the wire's type and has no runtime operation. Splitting a wire and
moving its point are separate undo steps. **Layout > Reroute** also adds one
from the action menu.

## Split and recombine pins

Right-click an unconnected Vector2, Vector3, Transform or native struct pin and
choose **Split Struct Pin**. Inputs and outputs expose typed member pins; a
Transform exposes Position, Rotation and Scale, each of which can split again
into X, Y and Z. Edit input defaults in Details or connect compatible outputs.
Right-click any child and choose **Recombine Struct Pin** to restore its parent.
Disconnect all children of that parent first; recombining preserves their values.
Both operations support undo/redo, save/reopen and copy/paste.

The **Pin Actions** menu also offers **Promote to Variable**. An unconnected
input creates a typed variable preserving its literal default and a connected
Get node. An output creates a variable and a connected Set node; an existing
Next execution connection is routed through that Set. For a pure source,
connect the new Set to the desired execution flow. Names are made unique and
the whole operation is one undo step.

Click a numeric value directly on a node to edit it, then press Enter or click
away to commit; Escape cancels. Vector2 and Vector3 display separate X/Y(/Z)
fields even before splitting. Split scalar fields use the same editor,
including fields of nested native structs. Fixed, signed and unsigned integers
retain their type/range checks, and committing a value is one undo step.
Connected pins show their wire instead of a default-value editor. These rules
are driven by each pin's type and connection state, independently of node kind.

While dragging a wire over a compatible pin, the endpoint snaps to its center,
the pin glows and fills, and the preview wire draws above the nodes. Releasing
the mouse makes the connection. Hovering alone never changes the graph. The
preview uses the same type, scope, cycle and occupied-pin checks as connecting.
Connecting to a literal input replaces its default; an existing wire must be
disconnected explicitly. Click-to-start/click-to-connect remains available.

Drop a dragged pin on empty canvas to open the action menu with **Context
Sensitive** enabled. It shows actions with at least one connectable opposite pin,
using the same validation as hover and connection. Choosing an action places it
at the drop position and wires the best matching pin automatically; exact types
take priority. This also works when dragging from an input to create an upstream
node, including typed reroutes. Creation and wiring undo together. Escape or
dismissing the menu cancels the pending wire without changing the graph.

Native function parameters and return values expose public fields of plain C++
structs, including nested structs, supported scalars, enums and Fixed[2/3] arrays.
Structs with bases, constructors, destructors, virtual methods, private/const
fields, bitfields, pointers or unsupported containers cannot be split. Opaque
runtime handles have no exposed fields. Mutable-reference **inputs** require
whole writable storage and cannot split; event outputs can expose their fields.
Split pins compile to native field access and value construction, without
introducing runtime graph nodes.

## Inheritance and identity

Class, variable, function, node, pin, and link identities are stable. A class
rename retains its identity; changing its parent is validated transactionally.
Missing parents, members, incompatible types, duplicate IDs, and orphaned
overrides are errors, not permission to discard data. Use explicit remap/reset
controls to repair references. A malformed graph does not prevent opening its
project for repair. External file edits block conflicting saves.

Class defaults flow from native parent to Blueprint parent to child, then to
explicit scene overrides. A value equal to its parent is still an override if
the user explicitly set it. Reset removes the override and resumes inheritance.
Overriding an event replaces that implementation unless the graph calls
**Call Parent**. Parent dispatch is a qualified native call, not redispatch into
the same override. Ordinary non-overridden behavior remains inherited.

## Class families

A Blueprint joins the family of its parent, and a class never changes family. The
three families you can author in are **Behaviour** (the original one), **Actor** and
**Component**. [Actors and components](actors.md) is the reference for the model
itself; this section is what the Blueprint editor does with it.

| | Behaviour | Actor | Component |
| --- | --- | --- | --- |
| Parent picker | yes | yes | yes |
| Events the new graph starts with | On Start, On Update, On Trigger | Begin Play, Tick, End Play | Begin Play, Tick, End Play |
| `Self` is | the script bound to an entity | the actor | the component |
| Owner access | `entity()` | the actor's own root, when its domain has one | **Get Owner**, returning a typed actor reference |

**Parent eligibility.** The parent picker lists the classes you may actually derive
from: same family, compatible domain, Blueprintable, not final, and no cycle.
Abstract parents remain valid authoring parents. A C++ class may derive from a C++
class and a Blueprint from either, but a C++ class may never derive from a
Blueprint-generated class.

**Reparenting** is checked against those rules *before* anything is compiled, so a
refusal tells you which rule it broke — crossing families, an incompatible domain, a
parent you cannot derive from, or a cycle — instead of surfacing as a compile error.
The previous parent and the whole graph are restored on refusal, exactly as they
already were for Behaviour Blueprints.

An actor with no spatial domain — a scene script, or a domain-less logic actor — has
no transform at all. Graph nodes that need one do nothing rather than dereferencing a
missing owner, so a 2D or UI actor never faults on a node that expects a 3D slot.

### Typed object references

Three reference value types join entity and class references:

| Type | Names | Assignable to |
| --- | --- | --- |
| `ActorRef<Class>` | one actor | a less derived `ActorRef`, or `ObjectRef` |
| `ComponentRef<Class>` | one component | a less derived `ComponentRef`, or `ObjectRef` |
| `ObjectRef` | either of the above | `ObjectRef` |

They are identities, not pointers: a reference to something that has been destroyed
fails its validity check and the node does nothing, the same guarantee entity handles
give. A reference is never written into a saved document as a live value — persisted
defaults carry the null identity — except for the map-scoped identities a scene
Blueprint may name, described below. `EntityRef` keeps its existing meaning for
legacy content.

### Spawn Actor and Spawn Class

**Spawn Actor** takes a class reference plus an optional logical parent and returns
an `ActorRef` of that base. It is refused at compile time when the class does not
resolve to the Actor family, naming the family it found, and it is not available in
a Behaviour Blueprint. At run time it re-checks the class before constructing
anything, so a stale class reference spawns nothing rather than the wrong type.

**Spawn** and **Spawn Class** are unchanged. They still spawn Behaviour-bound
entities from a fixed class or a typed `ClassRef<Base>`, including Behaviour bases,
and existing graphs keep compiling to the same code.

## Scene Blueprint

Every map owns one **scene Blueprint**: a class that derives from
`epok::SceneScriptActor`, is created by the level loader, and is never placed or
spawned by hand. It is where the logic that belongs to a map lives — the door
that only this level has, the sequence that starts when the level does.

Open **Map Settings** (click the map root in the Hierarchy) to work with it. A map
that does not have one yet is given the default as soon as it is opened: the
parent is the project's *Default Scene Blueprint Parent* when that names a
`SceneScriptActor` subclass, and `epok::SceneScriptActor` otherwise. Creating it
is not an edit — the map is not marked as changed and the file on disk is not
touched — so the default is written the next time the map is saved, like any
other part of the document. On a host where the project's class model cannot be
built at all, because the reflection toolchain is not provisioned there, no default
is proposed; the map keeps working and simply has no scene Blueprint until it is
opened on a host that has the toolchain.

- **Parent** lists the project's `SceneScriptActor` subclasses and nothing else.
  Changing it is transactional exactly like reparenting a Blueprint asset: the
  whole project is validated and compiled first, and a refusal keeps the previous
  parent with every default, event and wire intact. **Edit > Undo Scene Edit**
  steps back through it, because the scene Blueprint is part of the map.
- **Open Scene Blueprint** edits the graph in the Blueprint editor. The document
  is the map's, not a `.epokbp` file: edits mark the *map* dirty, Undo and Redo
  are the map's, and the editor's **Save** saves the map. The Blueprint editor's
  Revert is unavailable for it; undo the map instead.
- A scene Blueprint compiles even when no entity in the map carries a Behaviour,
  and two maps may share one C++ `SceneScriptActor` base.

Inside a scene Blueprint — and only there — a variable of an actor or entity
reference type may name an actor, component or entity **of that same map** by
identity. The reference is resolved when the project is compiled, against the map
that owns the Blueprint; naming something that is not in the map is an error that
reports the identity, and the same value in an ordinary `.epokbp` Blueprint is
refused because such a Blueprint has no map. Nothing is ever looked up by name
while the game runs. Map-scoped identities belong in variables, not in graph
literals: a literal pin that carries one is refused with a note to promote it.

Duplicating a map in the Project browser gives the copy its own identities: a new
class for its scene Blueprint, new actors and entities, and every map-scoped
reference rewritten to the copy. Shared content — textures, meshes, sounds — is
referenced, not copied.

## Execution and types

Compilation follows execution wires from each event/function entry and includes
the upstream data nodes consumed by that live execution. Disconnected islands
remain editable but produce no code, asset cooking requirements, or diagnostics
for unfinished pins and missing callable/asset references. Connecting them makes
their validation active again. Reading an impure result still requires its
execution input to be connected and to run before the consumer.

Supported authoring values include bool, signed/unsigned 32-bit integers,
Fixed Q12, reflected enums, bounded Fixed vectors, entity handles, imported
Texture/AudioClip references, class references and the typed object, actor and
component references described under [Class families](#class-families). Pins reject incompatible
links; class/entity covariance follows the reflected ancestry. Unsafe raw
pointers, arbitrary source text, dynamic containers, and open-ended recursion
have no graph execution policy and are rejected.

Execution nodes include event/function entry, calls, Call Parent, setters,
Branch, Sequence, bounded For, Return, Delay, Timeline and Stop Timeline.
Pure expressions include literals, parameters, getters, arithmetic, comparison,
Boolean operations, vector operations and reroutes. Engine adapters cover self,
handle validity, local transforms, input, activity/destruction, scene requests,
class tests/spawning and entity audio/texture assignment. Native annotated
methods extend the callable surface without adding an interpreted language.

Make Vector combines two or three Fixed inputs. Vector Component extracts one
checked component without a dynamic index. Spawn Class accepts a typed
`ClassRef<Base>` from a variable or pin; the selected runtime class must have a
compatible registered factory. A missing class or exhausted pool returns an
invalid handle. The fixed-class Spawn node remains available.

**Call on instance** invokes a reflected public callable on a typed Target
entity, including another Blueprint. Checked casts retain handle-generation
validation. An invalid, destroyed, or wrong-class target does nothing and returns
the type's empty value. Impure returned values are captured once, even when
multiple pins consume them. Cross-instance dispatch has a 32-call depth ceiling;
destroyed dynamic receivers stay allocated until their active calls unwind.

Impure calls with return values execute once, in execution-wire order. Reading
their result before the call is rejected. Pure data dependencies cannot cycle.
Nested structured loops have a combined 4096-iteration compile bound; expression
depth and expansion are also bounded. Division by zero produces zero; signed
integer/Q12 arithmetic saturates, and division rounds toward zero.

Delay yields; even zero delay resumes on a later tick. Each graph has a bounded
continuation frame; retriggering replaces its earlier pending invocation.
Arguments and live temporaries survive suspension. Inactive ancestors and
paused simulation freeze timers. Destruction, reused handles and scene changes
cancel stale work. Timeline uses bounded, strictly increasing Q12 time/value
keys and exposes Updated/Finished outputs; looping and Stop Timeline are explicit.

Sequence and Effect nodes also play the shared [TimelineAsset system](timelines.md).
Their typed playback handles are runtime values with null defaults, never saved
slots or generations. Marker/completion waits share the existing eight latent
slots, preserve results across pause and playback-slot reuse, and use explicit
Completed/Cancelled outcomes. Blueprint source v3 adds these nodes; opening v1/v2
documents migrates in memory and preserves their original bytes until Save.
Direct asset Play/Spawn nodes select a persistent asset UUID and expose its typed
external binding slots. Slot renames/reordering retain connections; removed slots
remain visible as stale connections until repaired. Subscribe to marker delivers
future crossings through that same continuation table, retaining a compact
backlog while the reached branch suspends. See [Timeline playback](timelines.md)
for owner, pause, completion and cancellation rules, and open the
`timeline-spell` example's `BP_Fireball` for an authored combat graph.

## Entity templates and construction

The class's template defines a stable-ID root and component/child hierarchy.
Use the template editor to add entities and edit inherited component members,
or capture a selected scene subtree into the class. Captured internal references
are remapped; references outside the subtree must be removed or represented by
an explicit instance input. Place a class to create a linked scene instance.

Unchanged members follow future template edits. Instance edits record explicit
member overrides; reset restores inheritance. New inherited children receive
fresh scene IDs. Removed/replaced template identities are reported for explicit
repair, preserving the existing scene data. Unlink retains the current entities
as ordinary scene objects. Placement, reset and unlink support scene undo.

Construction uses a closed host backend: bounded transform, activity, color and
parent operations applied with Q12-compatible rules. It does **not** execute
arbitrary native constructors or game MIPS code on the host. The same resolved
prototype supplies editor placement and runtime spawning.

Dynamic spawning reserves the entire template, binds all scripts, applies
defaults and resolves internal handles before any Start callback. Capacity or
validation failure rolls back the unstarted allocation without gameplay destroy
callbacks. Spawned scripts use typed per-class pools (four instances per class),
with 32 dynamic slots, at most 64 cooked classes, 32 entities per template, and
a 64 KiB compile-time typed-pool storage ceiling. Returned handles must be
checked for validity. Instance-generation checks prevent stale-slot reuse.

Resources reachable only through templates or AssetRef values are cooked too.
Dynamic resources use a fixed shared resident texture layout across scene banks;
the normal VRAM/audio limits still apply and overflow is an actionable cook error.

## Debugging and iteration

Enable **Instrument Blueprint Debugger** before Play. Select a graph node and
set a breakpoint; saved all-instance breakpoints also catch Start. The debugger
supports a real node pause, next-node step, continue, and run-local
instance/generation breakpoints. Current class/node mappings, recent bounded
traces, and typed property values come from the generated source map and a
versioned target snapshot. The first 16 reflected members are inspectable in
the fixed snapshot. This is not arbitrary native memory inspection.

Instrumentation uses an execution breakpoint in the owned PCSX-Redux interpreter
so synchronous graph calls retain their actual native stack. Release builds omit
the snapshot and hook work. Debug builds use a separate cache. Changes after a
debug build invalidate source navigation until rebuilding; there is no live
native-code or object-layout patching. Pause freezes the owned emulator, not
other running emulator instances.

Saved graph or native source changes stop outdated Play. With Auto compile
enabled, the editor rebuilds and restarts from authored defaults after the old
worker finishes and edits settle. Invalid sources leave Play stopped until repair;
an explicit Stop cancels automatic restart. Save or discard an open modified
graph, timeline or effect before Build/Play. Runtime state is not preserved.
Layout-only node moves do not change the semantic C++ cache key. Export produces
standalone native sources and a build script; neither the editor nor the
reflection extractor is required to rebuild the exported game.

Successful persisted-source compilations record their dependency footprints in
the host artifact graph under `.epok/ArtifactDependencies.json`. These include
reflected class/function contracts, inherited debugger property layouts, parent
and called Blueprints, imported resources, and referenced timelines, effects and
markers. The compiler captures the same source snapshots used for generation;
validating an unsaved canvas does not publish a project artifact. Removed or
invalid dependencies retain the previous generated signature with stale reasons.
The dependency viewer also covers scenes, generated output, staging and export.
Builds validate current sources and reject stale output; restoring an earlier
source does not certify a retained executable without fresh staging.

## Reproducible examples and acceptance

From the engine checkout after `cargo build --bins`:

```powershell
python tests/integration/create_blueprint_visual_fixture.py --destination "D:\Games\BlueprintCanvas"
python tests/integration/verify_blueprints.py --keep --emulator
python tests/integration/verify_blueprint_features.py --keep --emulator
```

The canvas generator refuses existing destinations and prints the command to
open the compiled example. The acceptance scripts retain isolated playable
projects when `--keep` is present: native Enemy / Blueprint Enemy / Blueprint Boss
inheritance, and a template/Timeline/resource/cross-instance-call example.
Run emulator checks sequentially with no other Play session on port 8077.
Reports are written under ignored `artifacts/blueprints/`.

## Command line

```powershell
epok-editor.exe --project "D:\Games\Example" --new-blueprint BP_Enemy --parent Enemy
epok-editor.exe --project "D:\Games\Example" --compile-blueprints
epok-editor.exe --project "D:\Games\Example" --build-psx
epok-editor.exe --project "D:\Games\Example" --play-psx --blueprint-debug
epok-editor.exe --project "D:\Games\Example" --export-psx
```

The native canvas uses compact node headers, typed pins and curved execution/data
wires. Its visual language does not imply external API or asset compatibility.
Lua is a separately reserved provider. The reusable TimelineAsset and VFX editors
are available alongside the older scalar Blueprint Timeline node; see
[Using the VFX editor](vfx-editor.md) for their authoring workflow.
