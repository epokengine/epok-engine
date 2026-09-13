# Your first Blueprint

Build a small visual behaviour, compile it and inspect its execution. This guide
introduces the editor before you move on to [Blueprint reference](blueprints.md)
or [a spell driven by markers](spell-tutorial.md).

## Before you start

Use Windows x64 or Linux x86_64 with the editor's reflection dependencies
installed. The `epok-header-tool` executable must be beside the editor; setup
supplies the pinned libclang runtime. Blueprint authoring is not currently
supported on macOS.
Follow [Getting started](getting-started.md) if classes or the extractor are missing.

For a ready-made learning project, run this from the engine checkout after
building the editor binaries:

```powershell
cargo build --locked --bins
python tests/integration/create_blueprint_visual_fixture.py --destination "D:\Games\BlueprintCanvas"
```

Choose a destination that does not exist. The command creates a separate project
with a compiled `BP_Interaction` class and prints how to open it. It does not
replace your game. Open that project, then double-click `BP_Interaction.epokbp`
in the Project browser. The fixture is also a useful reference if you prefer to
create a Blueprint in your own project.

## Find your way around the editor

| Area | Use it for |
| --- | --- |
| Components | Browse the class's root and child entity template. These are the objects instances will contain. |
| My Blueprint | Find graphs, functions, variables and inherited members. Select an item to edit its details. |
| Canvas | Connect execution and typed data pins; select, move, copy and organize nodes. |
| Details | Edit the selected node or member, including literal values and member settings. |
| Class Settings | Inspect the parent and class-level settings. |
| Class Defaults | Set the values new instances inherit. |
| Compile and diagnostics | Check the draft and navigate to invalid nodes or links. |

Drag an output pin onto a compatible input pin. Execution wires decide **when**
something happens; data wires supply **what value** an operation uses. A data
connection alone does not execute a setter or an impure call. Use **Frame All**
if the graph is outside the visible canvas. Panning and zooming change the view,
not the saved gameplay.

## Read the example before changing it

`BP_Interaction` derives from the native `InteractionBase` class. Its parent
exposes an `enabled` Boolean, a Fixed `progress` value, callable methods and an
`on_interaction` event. The parent calls that event from Start.

Follow the event's execution wires from its entry. The graph reads `enabled`,
branches, calls the native methods and uses a Delay. The `compute_strength`
function demonstrates a typed function result. Select each node to inspect its
inputs and follow data wires back to their source.

This separation matters when designing your own class: an event needs a caller.
Adding a function to My Blueprint does not make it run every frame. A reflected
native event, another graph call or an existing lifecycle entry must invoke it.

## Make a change and test it

The generator creates the class asset, not a running instance. In the scene,
create an Empty entity, attach `BP_Interaction` through the script selector and
save the scene. Leave the entity active and its inherited `enabled` value true.

1. Select the **Delay** node in the interaction graph. Change its seconds input
   from `0.25` to `1.0`. This input is a Fixed value in seconds.
2. Choose **Compile**. Resolve any diagnostic before continuing. Compile checks
   the current canvas; it does not silently save it.
3. Save the Blueprint. Save the scene too if you changed the placed instance.
4. Enable **Instrument Blueprint Debugger** before starting Play.
5. Select a node after the Delay and set a breakpoint. Start **Play**.
6. Inspect the paused node and typed values, then use next-node stepping or
   Continue. The delayed branch must resume later instead of blocking a frame.

The example changes behaviour state; it is not a visible movement demo. Use the
debugger to inspect the execution and `progress`. The fixed snapshot exposes the
first 16 reflected members. [The spell tutorial](spell-tutorial.md) provides a
visible follow-up with an effect, target health and cancellation.

## Create a class in your own project

1. Choose **New Blueprint** in Project or Add Component.
2. Select a reflected **Blueprintable** parent appropriate to the behaviour.
   For example, the learning project supplies `InteractionBase`. A final class
   cannot be used as a parent.
3. Name the asset and open it. Add an event override from the parent's exposed
   events, or create a function that another event will call.
4. Add a node through the searchable node catalog. Connect its execution input
   from the event entry, then connect typed data or set its literal inputs.
5. Compile, save, and attach the class to an entity through the ordinary script
   selector. An asset in Project is a class definition, not a running instance.
6. Save the scene and Play. Inspect the placed entity if the event does not run.

If there are no appropriate parent classes, add or expose a native parent using
the [C++ scripting guide](scripting.md). Do not choose an unrelated class just to
make a selector accept the graph. Abstract parents are allowed, but all abstract
events must be implemented before attaching or spawning the class.

## Variables, functions and types

Use a variable for state that belongs to an instance. Use function parameters
for values supplied by a caller. Set class defaults in the toolbar; set an
individual entity's overrides in the scene Inspector.

| Value | Typical use | Common mistake |
| --- | --- | --- |
| Bool | Enabled flags and Branch conditions | Connecting a numeric value directly to a Boolean pin |
| Int32 / UInt32 | Counts, indexes and nonnegative quantities | Assuming unsigned values can represent a negative result |
| Fixed | Time, movement and Q12-compatible scalar values | Treating it as an arbitrary host floating-point number |
| Fixed vector | Two- or three-component positions or directions | Connecting vectors with different lengths |
| EntityRef | A scene target constrained to a compatible class | Assuming a destroyed or missing target is still valid |
| AssetRef / ClassRef | Imported resources or spawnable classes | Supplying a filename where a typed reference is required |

Use **Make Vector** and **Vector Component** to construct or split vectors. Check
entity validity before gameplay that depends on a target. For a cross-instance
method, use **Call on instance** with a compatible typed Target; it does not mean
“call this method on every entity.” Invalid receivers are skipped safely.

Pure expressions calculate values when consumed. Impure calls require execution
wires and produce a result once. Their result cannot be read before that call
has executed. Avoid cycles in data dependencies and unbounded recursion.

## Inheritance and reusable objects

Choose **Create derived class** when making a variation of an existing class.
The child inherits its parent's members and defaults. Use **Call Parent** inside
an overridden event when the parent's implementation must also execute.

An explicit scene override keeps its value even if you later change the class
default. **Reset** removes that override and resumes inheritance. Simply setting
the same value as the parent does not remove an override.

Use Components to author a reusable entity hierarchy, or capture a scene subtree
as a class template. Place the class to create a linked instance. Internal
references remap to that instance; references outside the captured subtree need
explicit inputs or removal. Unlink keeps the current scene objects but ends
template propagation. See [Entity templates and construction](blueprints.md#entity-templates-and-construction)
for spawning limits and the host construction subset.

## Save, undo and rebuild

Undo/Redo applies to the Blueprint document; copy/paste generates fresh node and
pin identities. Moving a node changes its layout without changing its meaning.
Renaming a member retains its identity, but deleting it can leave connections
that need explicit repair. Do not edit generated C++ to repair a source graph.

Save or discard modified Blueprints, timelines and effects before Build/Play.
Native or graph changes stop obsolete Play. With Auto compile enabled, valid
saved changes rebuild and restart from authored defaults. They do not preserve
the previous runtime state or patch native code in a running PSX process.

## Continue learning

- [Blueprint reference](blueprints.md): types, inheritance, execution and debugger contracts.
- [VFX editor](vfx-editor.md): create an effect without gameplay wiring.
- [Spell tutorial](spell-tutorial.md): spawn an effect and apply damage at Impact.
- [Blueprint and VFX troubleshooting](blueprints-vfx-troubleshooting.md): repair classes, bindings, playback and stale output.
