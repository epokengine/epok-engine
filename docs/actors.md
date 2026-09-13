# Actors and components

Epok has two ways to describe what a map contains. The original one is the
**entity**: a slot with optional Transform, Mesh, Camera, Light, Sprite, Audio and
HUD data, plus at most one Behaviour script. The newer one is the **actor**: a
class you write, deriving from an engine class, that owns **components**.

Both live in the same map. Nothing you already made stops working, and nothing is
converted behind your back. This page is the reference for the actor half:
[Migrating an existing project](migration-actors.md) is the companion for a map
you already have.

> **Availability.** Actors need the project's class model, which comes from the
> reflection toolchain. That toolchain is provisioned on Windows x64 and Linux
> x86_64; macOS does not yet provision it, so on macOS the Actor menus are empty
> and the actor tools report that no model is available. See
> [Blueprints](blueprints.md) for the same requirement.
>
> **Validation status.** Everything described here is exercised by the editor and
> by host C++ tests. Building a game with actors for the console, exporting it and
> measuring it in the emulator is **pending on a machine with the PlayStation
> SDK**; no console budget figure is claimed here.

## The class tree

Every reflected class belongs to exactly one **family**, and a class never changes
family by changing its parent.

| Family | Root class | What it is |
| --- | --- | --- |
| Object | `epok::Object` | The root of the class system. Abstract. |
| Actor | `epok::Actor` | Something a map contains. Abstract; it has no transform of its own. |
| Component | `epok::ActorComponent` | Behaviour or data attached to exactly one actor. Abstract. |
| Level / World | `epok::Level`, `epok::World` | The loaded map and its services. You do not derive from these. |
| Behaviour | `epok::Behaviour` | The original script family, unchanged. |

The concrete actors you place are:

| Class | Domain | Root component | Use it for |
| --- | --- | --- | --- |
| `epok::Actor3D` | 3D world | `epok::SceneComponent3D` | Anything in the 3D world: characters, props, triggers, cameras. |
| `epok::Actor2D` | 2D world | `epok::SceneComponent2D` | 2D world objects with an XY position, rotation, scale and draw order. |
| `epok::UIActor` | UI | `epok::RectTransformComponent` | Screen-space interface built on the Canvas layout rules. |
| `epok::SceneScriptActor` | none | none | The one-per-map scene Blueprint. It is created by the level loader and can never be placed or spawned by hand. |

**One parent, one family.** A class has exactly one parent and inherits its family
and its domain from that parent. You can refine a domain that was not set — that is
how `epok::Actor3D` derives from the domain-less `epok::Actor` — but you cannot
change a domain that is already fixed, and you cannot reparent an actor class onto
a component class or the other way round. The editor refuses such a change before it
tries to compile anything, and says which rule it broke.

### Components

A component belongs to exactly one actor and has its own identity and lifetime.
Each component class declares which actor domains may own it, whether it can be an
actor's **root**, whether an actor may hold more than one of it, and what other
components it requires or excludes.

| Component | Owners | Notes |
| --- | --- | --- |
| `epok::SceneComponent3D` | 3D | Root of a 3D actor. Carries the Transform. |
| `epok::SceneComponent2D` | 2D | Root of a 2D actor. Carries position, rotation, scale and draw order. |
| `epok::UIComponent` | UI | Abstract base for interface components. |
| `epok::RectTransformComponent` | UI | Root of a UI actor. Carries the anchored rectangle. |
| `epok::AudioComponent` | 3D, 2D and UI | A view over one audio source. An actor may hold several. |

`epok::AudioComponent` is the one shared component: the same class works on a 3D
actor, a 2D actor and a UI actor, which is why a menu click and a footstep are the
same kind of thing to author.

**Roots.** An actor in a spatial domain has exactly one root component, and the root
must match the actor's domain. The root is what gives the actor a place; the actor
class itself has none. `epok::Actor3D`, `epok::Actor2D` and `epok::UIActor` each
declare their root, so a new actor already has one. A domain-less actor — a scene
script, or your own logic-only actor — has no root and needs none.

**Attachment vs. hierarchy.** Two different relationships exist and they are
authored separately:

- the **logical parent** is the shape of the Hierarchy: it decides what is listed
  under what, and deactivating a parent deactivates its children;
- **attachment** is spatial: a component follows another component's transform.
  Attachment only works inside one domain, and cycles are refused.

## Domains and what each view shows

A **domain** is `World3D`, `World2D`, `UI` or none. It comes from the class and is
never stored separately, so an actor can never disagree with its own class about
where it lives.

Scene's **3D / 2D / UI** buttons choose the authoring mode, and the Hierarchy lists
what belongs to that mode: 3D shows the 3D world, UI shows canvases and rect
elements, 2D shows 2D actors. Domain-less logic and resource objects appear in all
three modes. An item from another spatial domain that has children in the current one
still appears, greyed out, so branches keep their shape; those lines cannot be
selected, renamed or used as a drop target. Filtering never edits the map, and a
selection survives a mode switch. See [Using the editor](editor.md).

Domain-less actors — scene scripts and logic-only classes — are grouped under
**Logic** in the Actor creation menu and remain editable in every authoring mode.

## Writing an actor class in C++

Annotate the class after the `class` keyword, exactly as for a Behaviour. The actor
options are comma separated:

```cpp
#pragma once
#include "epok.hpp"

class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World3D,
                 Id="1f0d6b1a-3c44-4f2f-93ba-9a0e5a1c7d20") Enemy
    : public epok::Actor3D {
public:
    EPOK_PROPERTY(EditAnywhere) epok::Fixed speed = 2.0;
    EPOK_PROPERTY(EditAnywhere) epok::Fixed health = 100.0;

    EPOK_COMPONENT(Name="Voice") epok::AudioComponent voice;

    void begin_play() override { /* ... */ }
    void tick(epok::Fixed dt) override { /* ... */ }
    void end_play(epok::EndPlayReason reason) override { /* ... */ }
};
```

`EPOK_CLASS(...)` accepts these options in addition to the ones Behaviours already
use (`Blueprintable`, `Id="uuid"`, `TimelineRequires=...`):

| Option | Applies to | Meaning |
| --- | --- | --- |
| `Family=Object\|Actor\|Component\|World\|Level` | the engine's own base classes | Declares a family root. Your classes inherit their family and must not redeclare a different one. |
| `Domain=World3D\|World2D\|UI\|None` | actors, components | The domain. Inherited; a class may refine an inherited `None`, never change a fixed one. |
| `Placeable` | actors | May be placed in a map from the editor. |
| `Spawnable` | actors | May be spawned at run time by **Spawn Actor**. |
| `SceneManaged` | actors | Created by the level loader only. Cannot be combined with `Placeable` or `Spawnable`. |
| `Abstract` | any | Marks a base that has no pure virtual members as abstract anyway. |
| `Root` | components | May be an actor's root component. |
| `Owners=World3D\|World2D\|UI` | components | Actor domains that may own it, separated by `\|` with no spaces. A derived component may only narrow this set. |
| `Requires=A\|B`, `Excludes=A\|B` | components | Component classes that must, or must not, be on the same actor. |
| `Cardinality=Single\|Multiple` | components | Default `Single`. |
| `Capability=name` | components | A capability the build target must provide, such as `audio`. |

`EPOK_COMPONENT(...)` marks a member of a reflected component type as a component
the class always has. Its options are `Root`, `Name="display"`, `AttachTo=fieldName`
and `Id="uuid"`. `Root` and `AttachTo` are mutually exclusive: a root has nothing to
attach to. Constructors stay defaulted, as they do for every reflected class, so
defaults remain declarative and inspectable.

No option value may contain a comma.

A component class is written the same way with `epok::ActorComponent` (or a more
specific base) as the parent:

```cpp
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D,
                 Cardinality=Multiple, Id="…") Flammable
    : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) epok::Fixed burn_seconds = 3.0;
};
```

## Actor Blueprints

An actor class can also be a Blueprint. Choose an actor parent in **New Blueprint**
and the new class starts with **Begin Play**, **Tick** and **End Play** instead of
the Behaviour events. A chain may be as long as you like as long as it starts at a
C++ class: `epok::Actor3D → Enemy (C++) → BP_Goblin → BP_GoblinFire` is a normal
project. The one direction that is refused is a C++ class deriving from a Blueprint.

Blueprint graphs on actors use the same nodes as everywhere else, plus:

- **Spawn Actor**, which takes an actor class reference and an optional logical
  parent and returns a typed actor reference. It refuses a class that is not in the
  Actor family. The existing **Spawn** and **Spawn Class** nodes are unchanged and
  still spawn Behaviour-bound entities.
- **Get Owner** on a component Blueprint, which returns the actor the component
  belongs to.
- The typed reference values `ObjectRef`, `ActorRef` and `ComponentRef`. A reference
  to a more derived class can be used where a base one is expected, and actor or
  component references widen to `ObjectRef`.

See [Blueprints](blueprints.md) for the full node and pin reference.

## Placing actors

Use the **Actor** submenu in any creation menu — the GameObject menu, the Hierarchy
context menu or the map root menu. The submenu lists the project's placeable classes
grouped as 3D, 2D, UI and Logic; abstract classes, component classes and scene
scripts never appear. A new actor is created at the map root with the components its
class declares, or with the root component its domain needs when it declares none.
Creating an actor from a child's context menu still creates a root actor.

Selecting an actor opens the **Actor** section of the Inspector: its class and
domain, its identity, its logical parent, an editable name, an **Active** checkbox
and the list of its components — with a `root` marker on the root and a
`(class default)` marker on the ones the class brings. A class the project cannot
resolve is shown in amber with a "class unresolved" note instead of being dropped.

Adding and removing components from the Inspector, renaming, duplicating, deleting
and drag-reparenting actors are not available yet. Until they are, use the
[MCP actor tools](mcp.md#actors) or the `--add-actor` command line for those edits.

Scene edits, including actor edits, are covered by **Edit > Undo Scene Edit** and
**Redo Scene Edit** (Ctrl+Z / Ctrl+Y) for the last 32 changes to the open map.

## Defaults and overrides

Values flow from the C++ class, to each Blueprint in the chain, to the placed actor.
An actor stores only what you actually changed; everything else keeps following its
class. Resetting a value removes the override and resumes inheritance. This is the
same rule Blueprint class defaults already use, applied to actors and to their
components.

A property the class chain no longer declares is reported when the game is built,
naming the property, instead of being dropped silently.

One current limitation is worth knowing before you rely on it: a Blueprint actor's
**Begin Play** graph reads its class defaults, not the per-instance overrides of that
placed actor. Every later callback — `on_enable`, `tick`, other actors, the scene
script — sees the overrides. Read overridden properties from Tick, or from a later
event, rather than from Begin Play.

## The Scene Blueprint

Every map owns one scene Blueprint: a class deriving from `epok::SceneScriptActor`
that the level loader creates, and that you never place or spawn. It is where the
logic that belongs to one map lives.

Click the **map root** — the first line of the Hierarchy, named after the scene — to
open **Map Settings**. It shows the map's name, document version and entity/actor
counts, the scene Blueprint section and the map's HUD budget.

A map that has none is given a default as soon as it is opened, using the project's
**Default Scene Blueprint Parent** (Project Settings > Project > Description) when
that names a `SceneScriptActor` subclass, and `epok::SceneScriptActor` otherwise.
Creating that default is not an edit: the map is not marked as changed and the file
on disk is not touched, so it is written the next time you save. On a host where the
project's class model cannot be built — macOS today — no default is proposed at all,
and the map simply has no scene Blueprint until it is opened on a host that has the
toolchain.

Changing the parent is one transaction: it is validated and the project is compiled
first, and a refusal keeps the previous parent with every default, event and wire
intact. The scene Blueprint is part of the map, so **Undo Scene Edit** steps through
it, its edits mark the *map* dirty, and the Blueprint editor's Save saves the map.

Inside a scene Blueprint — and only there — a variable of an actor, component or
entity reference type may name something **of that same map** by identity. See
[Blueprints](blueprints.md#scene-blueprint) for the scoping rules.

## Legacy entities and Behaviours

Opening an existing map changes nothing on disk. What you see is:

- your entities, exactly as they were, in the Hierarchy and the Inspector;
- a derived actor view of those entities wherever a tool needs one — for example
  `scene_actors` over MCP lists both the actors you authored and the derived view of
  the entities you have not migrated.

The derivation is fixed and deterministic. A Camera, Mesh or Empty entity reads as an
`epok::Actor3D` with the matching components; a Canvas or RectTransform entity reads
as an `epok::UIActor`; a Behaviour on an entity reads as a
`epok::LegacyBehaviourComponent`, which receives `start`, `update`, `frame_update`,
`on_enable`, `on_disable`, `on_destroy` and `on_trigger` exactly as before. The full
table is in [Migrating an existing project](migration-actors.md).

One case cannot be derived cleanly: an entity that carries **both** 3D data and UI
data. Splitting it into two actors would break every reference to it, so the view
keeps one 3D actor and reports a *pending conversion* diagnostic instead. The entity
keeps working; the editor action that asks you what the UI half should become is not
implemented yet.

## Limits

These are descriptions of what exists today, not promises:

- **No positional audio.** `epok::AudioComponent` plays, stops and reports a source.
  Attenuation, panning and a position-aware audio source are not implemented in
  either 2D or 3D.
- **2D is a world, not a physics engine.** The 2D runtime provides transforms and a
  hierarchy, a camera and its projection, draw ordering, axis-aligned box and circle
  colliders, raycasts, axis-separated sliding movement, trigger Enter/Stay/Exit and
  picking. Oriented boxes, rigid-body dynamics, tilemaps and navigation are out of
  scope. 2D authoring in the editor — a real viewport, Camera2D and Sprite2D
  authoring, gizmos — arrives in a later release; the 2D mode currently lists this
  map's 2D actors and shows a placeholder viewport.
- **No gameplay archetypes.** There is no Pawn, Character, controller or movement
  component. `epok::Actor3D`, `epok::Actor2D` and `epok::UIActor` are the minimal
  classes; everything above them is yours.
- **Component adapters are partial.** The mesh, camera, light, collider, sprite and
  UI component classes exist in the derived view of legacy entities, but the editor
  does not yet let you add them to an authored actor.
- **Capacity is fixed at build time.** One level holds up to 64 actors and each actor
  up to 8 components; the slot table is sized from the map that needs the most, plus
  32 slots for run-time spawning. Exhausting a bound refuses the operation and counts
  it in `actor_stats` — see [Performance](performance.md) — instead of overwriting
  memory.
- **Console validation is pending.** Actor content is validated on the host; the MIPS
  build, the standalone export and the emulator budget for actor content still have
  to be run on a machine with the PlayStation SDK.

## A worked example

This is the walkthrough the actor model was designed against. Steps 1 to 6 are
authoring steps you can follow in the editor on a host with the reflection
toolchain; step 7 is the console run, which is **pending SDK validation** and is
listed so the sequence is complete.

1. **Write the C++ base.** Create `Enemy` with **New C++ Script**, choose
   `epok::Actor3D` as its parent, and give it the annotation and properties from
   [Writing an actor class in C++](#writing-an-actor-class-in-c) above. Compile.

2. **Derive a Blueprint.** **New Blueprint**, parent `Enemy`, name `BP_Goblin`. Its
   graph opens with Begin Play, Tick and End Play. Set `speed` and `health` in Class
   Defaults; those are this class's defaults, not an instance's values.

3. **Derive it again.** **New Blueprint**, parent `BP_Goblin`, name `BP_GoblinFire`.
   Override Tick, add **Call Parent** so the goblin logic keeps running, and add
   whatever makes this one burn. The chain is now
   `epok::Actor3D → Enemy → BP_Goblin → BP_GoblinFire`.

4. **Place them.** In the 3D mode, use **Actor > 3D > BP_GoblinFire** from the
   Hierarchy context menu. Select the new actor and change `health` in the Inspector:
   that one value is now an override on that one actor, and `speed` still follows
   `BP_Goblin`.

5. **Give both an audio component.** `Enemy` already declares
   `EPOK_COMPONENT(Name="Voice") epok::AudioComponent voice;`, so every goblin has
   one and it shows in the Inspector as a class default. Now add a `epok::UIActor`
   from **Actor > UI**, and declare an `epok::AudioComponent` on its class the same
   way for the menu sound. The component class is the same in both places; only its
   owner's domain differs.

6. **Coordinate them from the map.** Click the map root, open **Map Settings**, and
   **Create Scene Blueprint**. Open it and, in its variables, add an `ActorRef` to the
   goblin and an `ActorRef` to the UI actor — a scene Blueprint may name actors of its
   own map by identity. Begin Play on the scene script runs after every actor of the
   map has begun play, so it is the right place to wire the two together.

   Then change the map's parent: write a C++ `DungeonSceneScript : epok::SceneScriptActor`,
   compile, and choose it in Map Settings. The change is validated and compiled as one
   transaction; on refusal the previous parent and the whole graph come back.

7. **Run it on the console.** Build, then Play. **This step has not been validated in
   this delivery**: the MIPS build, the standalone export and the emulator run for a
   map with actor content are pending on a machine with the PlayStation SDK.

## See also

- [Migrating an existing project](migration-actors.md) — what changes for a map you
  already have.
- [Blueprints](blueprints.md) — families, parents, typed references, Spawn Actor and
  the scene Blueprint.
- [Using the editor](editor.md) — view modes, Hierarchy filtering, Map Settings, undo.
- [Document formats](formats.md) — scene document version 5 and the Blueprint asset
  version.
- [AI assistants / MCP](mcp.md#actors) — the actor tools and the command line.
- [Performance](performance.md) — `actor_stats` and the capacity constants.
