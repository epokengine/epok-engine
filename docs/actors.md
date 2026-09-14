# Actors and components

A map contains one collection of Actors. Actors have a class, an identity, an active state, a logical parent and a set of ActorComponents. C++ and Blueprint classes use the same inheritance and ownership rules.

## Spatial domains

`Actor` is an abstract base without a transform. Its three spatial subclasses provide a root component:

| Actor base | Root component | Scene view |
|---|---|---|
| `Actor3D` | `SceneComponent3D` | 3D world |
| `Actor2D` | `SceneComponent2D` | 2D world |
| `UIActor` | `RectTransformComponent` | Screen interface |

The logical hierarchy controls grouping and inherited activation. Spatial attachment controls transform inheritance and requires matching domains. A UIActor may be grouped beneath an Actor3D without inheriting its 3D transform.

Each map also owns a `SceneScriptActor` for its embedded scene Blueprint. It is managed by the scene, not placed through the Actor picker.

## Creating and reusing Actors

Choose **Instantiate Actor** in the Hierarchy or Scene view. The class selection dialog lists classes for the current view, including compatible C++ and Blueprint subclasses. Search filters the hierarchy. The selected class creates a scene instance with its native default components.

Use **Add Component** in the Inspector to attach a compatible native or Blueprint ActorComponent. Components may declare allowed owner domains, required or excluded components, and whether multiple instances are allowed. A general logic component supports all three domains; a component that accesses a specific spatial root should restrict its owners.

The menu lists only components that can be added to the selected Actor. **Create C++ ActorComponent...** and **Create Blueprint ActorComponent...** restrict the parent picker to ActorComponent classes compatible with that Actor's domain. To create an Actor class, use the Project browser's class creation commands.

Engine classes are read-only. **Edit C++ Class** is available for project classes and opens their actual declaration in VS Code; changing shared class code affects every instance using that class. The Inspector header keeps **A** (Active), **S** (Static) and the instance name on one row, with tooltips on both toggles.

A placed Actor belongs to its map. **Convert to Actor Blueprint** captures its component composition, property values and child Actors in `assets/Blueprints`, and links the selected placement to that class. The reusable class can then be instantiated again. Each placement has independent Actor and component identities and preserves explicitly edited values when the Blueprint changes. References inside the captured subtree are remapped; references outside it must be cleared or replaced before capture.

## Blueprint execution

Lifecycle events are graph entries. They have no execution or data inputs; event parameters are outputs. Each execution output connects to one destination. Connecting it again replaces its previous connection and can be undone. Use `Sequence` for explicitly ordered execution of several branches; each Sequence output still has one destination. Conditional nodes provide a separate output for each outcome.

`Self` identifies the current Actor or ActorComponent. In a component Blueprint, `Get Owner` returns the owning Actor, typed to the declared owner domain when that domain is specific. Transform operations require a compatible Actor reference. Compile reports a missing or incompatible Target in the Blueprint window and highlights the responsible node. Build options are part of that same window.

## Document contract

Maps use version 6, Blueprint assets version 5, and reflection metadata schema 9. The Actor/component documents are authoritative; viewport and runtime service data are projections of those components. There is no separate Entity collection or Behaviour attachment system.

See [Project recreation](migration-actors.md) for projects from older versions.
