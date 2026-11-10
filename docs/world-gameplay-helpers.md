# World gameplay helpers

The Game Helpers source in Project exposes the ordinary reflected class model.
Drag an actor into its matching 2D or 3D scene view, or attach a component to an
existing actor. Edit defaults, derive a C++, Blueprint or Lua class, and compose
visuals with the normal class tools.

| Class | Composition and extension |
| --- | --- |
| `ExaminableObject3D`, `ExaminableObject2D` | Root and `InteractableComponent`; supply project interaction behavior |
| `DialogueNPC3D`, `DialogueNPC2D` | Root and `DialogueInteractionComponent`; assign a conversation and a placed dialogue widget |
| `Pickup3D`, `Pickup2D` | Root and `PickupComponent`; add a visual and optionally a trigger collider |
| `Door3D`, `Door2D` | Root, collider and `DoorComponent`; add a visual and optional requirement flag |
| `Switch3D`, `Switch2D` | Root and `SwitchComponent`; add a visual and optional target flag |

These presets add no demonstration level, inventory or global helper state.
The pickup, switch and examinable presets occupy two component slots. Doors
occupy three. Each actor has eight component slots; a level supports 64 actors.
Project compositions and derived defaults can consume additional capacity.
Hover a catalog class to preview its resolved actor/component counts, required
component capabilities and referenced project assets. The preview uses the same
composition resolver as placement and installs no files. Placement rejects a
composition that exceeds level or component capacity before publishing changes.

## Pickups and explicit state

Bind `PickupComponent.accepted_initiator` to the actor allowed to collect it.
`collect(initiator, port)` uses the shared interaction availability and range
checks. Override the pure `can_collect` policy and the `collected` event for
project rewards. The native transition marks the pickup collected before
dispatching events, preventing duplicate rewards during reentrant calls.

`collected_flag` can reference a `GameplayFlagComponent` on another actor in the
same level. Collection sets that flag before the reward event. The flag remains
readable when its owner is inactive; destruction invalidates the reference.
Doors therefore do not need to infer ownership from a key actor's visibility.

`deactivate_on_collect` defaults to false. With it enabled, the reward event runs
before deactivation. `is_collected()` reports collection independently of actor
activity. `reset_collection()` clears collection state; explicitly reactivate the
actor and reset any shared flag when the game's restart policy requires them.

For automatic collection, add an ordinary trigger collider and enable
`auto_collect`. Trigger Enter calls the same `collect` operation using
`input_port`; Stay does not award again. Keep the accepted initiator explicit.

## Doors and switches

`DoorComponent.state()` is `Closed`, `Open` or `Locked`. `open(source)`, `close()`,
`lock()`, `unlock()` and `reset_door()` return `HelperResult`. A successful request
returns `Queued`; the component commits it in the Input tick group before
physics. `last_result()` reports the commit result. Opening revalidates the
source, requirement, pure `can_open` policy and collider at commit time.

An unbound requirement permits ordinary opening. A bound flag must exist in the
same level and be true. A locked door requires a true bound flag before opening.
Override `door_changed(previous, current)` for visual changes and
`door_requested(source, result)` for request feedback. Reset restores the authored
initial state, including an initially open door whose requirement is now false.
Unlocking an already open door does not close it.

Bind `collider_3d` or `collider_2d` explicitly, or let the door use its first
compatible own collider. The component remembers the collider's authored enabled
state. It disables blocking while open and restores that state when closed,
rebound or ended. Disabling the door cancels a pending transition.

`SwitchComponent.activate(initiator, port)` also uses shared interaction checks.
Override `can_switch` and `switched(initiator, value)` for project policies and
visuals. `target_flag` binds the shared state; without it the switch owns a local
boolean. `toggle` defaults to true. A one-shot switch returns `Unchanged` after
it is already on. `reset_switch()` restores `initial_value`.

`GameplayFlagComponent.set_value()` emits `value_changed(previous, current)` only
on an actual change. Recursive changes during that event return `Busy`.

## Authoring and runtime boundaries

All operations, typed object references, states and events share the reflected
API across C++, Blueprint and Lua. Lua can use named constants such as
`epok.DoorState.Open` and `epok.HelperResult.Queued` in its gameplay bodies.
Call the parent event when extending a parent class's behavior. Pickup and switch
interaction transitions stay native; extend their policy and result events.
Duplicating a selection remaps references between its copied actors and
components together. References to objects outside that selection stay external.

These components are optional build inputs. A flag does not require interaction
polling. A project that uses none of these classes does not reserve a world helper
pool. Build's memory report is authoritative for linked RAM and executable size.
The presets contain no mesh, sprite, sound or dialogue UI art; add project content
through normal components and use [dialogue widget classes](dialogue-widgets.md)
for editable dialogue visuals.

## Template rules and widget classes

All six templates generate C++, Blueprint or Lua AOT authoring. Shared character,
camera, interaction, UI and device mechanisms keep the same contracts; project
rules stay in the selected language. Generated Lua C++ belongs to build output.
The engine's optional VM execution modes retain separate engine coverage.

Dungeon Complete uses editable player vitality/combat, encounter, run-flow and
economy policies. Its HUD, terminal, shop and inventory are normal UIActor class
compositions with child controls and their own selected-language behavior. Open
them in the common UI class designer to change their layouts, properties and
behavior. Typed scene/audio/encounter/effect binding components provide explicit
references; changing actor names does not define target membership or routing.
