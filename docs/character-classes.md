# Character classes and simulation order

Character3D and Character2D are native, placeable, derivable actor classes. They
share reflected command/sample APIs across C++, Blueprint and Lua. They contain
no pad polling, game rules or animation selection. Optional command producers
provide local-player, point/target and scripted policies. Their provider/runtime
milestone and the shared locomotion drivers are validated. Third Person,
Platformer and Dungeon pass their foundation routes across the five execution
modes. See the [P5 progress record](../knowledge/initiatives/gameplay-helpers/p5-progress.md)
for the evidence and later complete-game/resource gates.

Find these classes under **Content Browser > Game Helpers > Characters**, filtered
for the current 3D or 2D view. Drag a player/NPC Actor into the scene or a compatible
command/animation component onto an Actor's Inspector. They use the same class
defaults, inheritance, component limits and undo/redo as normal Actor authoring.

| Class | Native composition | Slots used / available |
| --- | --- | --- |
| Character3D | Actor3D root, Collider3DComponent, CharacterMotor3DComponent | 3 / 5 |
| Character2D | Actor2D root, Collider2DComponent, CharacterBody2DComponent, CharacterMotor2DComponent | 4 / 4 |
| Player3D | Character3D and LocalPlayer3DInputComponent | 4 / 4 |
| Player2D | Character2D and LocalPlayer2DInputComponent | 5 / 3 |
| NPC3D | Character3D and MoveTo3DComponent | 4 / 4 |
| NPC2D | Character2D and MoveTo2DComponent | 5 / 3 |

The colliders use their ordinary defaults. Edit their bounds, layers and masks
through the normal component Inspector. The visual and camera are optional typed
references, so a prototype can choose its own visual composition and camera rig.
Neither class creates a camera actor or loads art solely by being placed.

Both actors expose `submit_command`, `stop_movement`, `motion` and `motor_id`.
Subclass `tick` can submit commands; the shared motor consumes them in the same
simulation step. Native default component identities and properties use the
same inheritance and placement resolver as other actor classes. Character3D
uses 3D world units; Character2D uses pixels.

## Command producers

`CharacterCommandSource3DComponent` and `CharacterCommandSource2DComponent`
provide overridable `calculate_command(dt)` and `command_submitted(result)`
events, plus `update`, `sample`, `release_command` and `set_enabled`. They use
Gameplay ticks, before the motor's Physics stage. Disable automatic ticking
when updating a source manually. An empty `motor` reference selects the owner's
motor; an explicit reference may control another active actor in the same Level.

Calculation returns a command and a status: Applied, NoStep, InvalidOwner,
InvalidMotor, InvalidTarget, InvalidConfiguration or Busy. The result records
use six wire words in 3D and four in 2D. The submission event runs after a valid
command reaches the motor. Reentry reports Busy. Zero delta preserves pending
commands; invalid configuration or unavailable input releases the source's
command. Releasing, disabling or destroying an old source cannot clear a newer
source's submission or a command submitted directly to the motor. Multiple
enabled sources use their normal Gameplay order; choose one active policy when
exclusive control is desired.

`ScriptedCommand3DComponent` and `ScriptedCommand2DComponent` expose a structured
`desired` command in the ordinary Inspector. Their jump field clears after a
successful submission; the motor retains the edge until an applied physics step.
Use them for authored movement, or derive a source in C++, Blueprint or Lua to
calculate commands from project rules.

Local-player sources read logical MoveX, MoveY and Jump for ports 0–3, including
the existing D-pad/analog mapping and deadzone. Player3D defaults to a maximum
speed of 6 world units/second, camera-relative movement and turning. Its explicit
camera reference takes precedence over Character3D.camera, then the active
camera's world forward direction, then zero yaw. Player2D defaults to 96
pixels/second. Non-floating bodies move along their configured floor tangent;
floating bodies use XY movement with Up toward negative pixel Y.

Movement, jump and camera capture retain the shared input router's independent
action groups and physical aliases. Capturing movement sends no desired planar
movement while the motor continues gravity and collision. Capturing Jump also
clears a pending jump owned by that input source. A jump edge is consumed only
after successful submission. Disconnected ports release their source command.

MoveTo3D and MoveTo2D accept a point (`use_point`) or a typed actor target. Actor
targets take precedence, use world position and must be active in the same Level.
Speed and stop distance use the domain's normal units. Desired speed is capped
by the remaining distance and timestep. The 3D motor still applies acceleration;
the policy is direct steering and does not perform pathfinding or guarantee
braking before obstacles. Non-floating 2D characters walk along their floor
tangent and require a project policy for jumping. Floating bodies steer in XY.
An unset target with `use_point` disabled produces an idle command.

## Simulation stages

Level dispatches the simulation stages in this order:

1. Input: camera orbit input and other native input-stage hooks.
2. Gameplay: ordinary component and actor ticks, including character rules.
3. Physics: the shared character motors.
4. PostPhysics: camera rigs and presentation-stage hooks.

This ordering is global within the Level. A camera placed before its target and
a producer component appended after a motor still observe the correct step.
Component order within a stage, component-before-actor order, and scene-script
order remain stable. Every stage rechecks activation and lifetime. Pausing uses
the existing simulation clock; frame callbacks retain their existing ordering.

Native C++ classes select a stage with a public constant:

```cpp
static constexpr epok::TickGroup tick_group = epok::TickGroup::PostPhysics;
```

Blueprint and Lua subclasses inherit that native policy through their generated
class. A normal field or unrelated constant named `tick_group` does not select a
stage. Classes without a native policy retain Gameplay. Stage selection does not
add another lifecycle, scheduler, script VM or serialized authoring schema.

## 2D motor

CharacterCommand2D contains a desired velocity vector and a jump edge (three ABI
words). The ten-word CharacterMotionSample2D reports status, velocity, the last
slide displacement, grounded/jumped, wall/ceiling contacts and blocked state.

CharacterMotor2DComponent has four editable properties: an optional same-owner
body reference, gravity (720), jump speed (264), and automatic tick (true). An
empty body reference selects the owner's existing CharacterBody2DComponent.
Body configuration owns collision masks, floor angles, snap and floating mode.

For a non-floating body, desired velocity is projected onto the floor tangent.
The motor preserves normal velocity, applies gravity along the body's opposite
up direction and jumps along its up direction. The up vector must be normalized.
With the default up direction `(0,-1)`, positive Y is downward. Floating bodies
use the full desired velocity directly and do not apply gravity or jumping.

The motor calls the existing body's explicit-step adapter. `move_and_slide()`
retains its original behavior; `move_and_slide_step(dt)` uses the same collision,
snap, one-way and moving-platform implementation without changing the global
clock. Platform support and path-follow updates retain the existing World2D
service ordering. The sample's displacement is the slide operation's displacement.

Zero delta does not move or consume a pending jump. Invalid owner/body/shape,
negative or excessive delta, invalid gravity/jump speed and invalid vectors
return a status without applying movement. Jump submissions latch until an
applied step; stopping clears desired movement and pending jump while retaining
physical velocity. Manual simulation requires disabling automatic tick.

## Evidence and pending acceptance

The host test compares 720 exact Q12 steps with the original platformer sequence
on the real World2D solver, with positive floor/wall/ceiling, one-way-platform and
path-follow carry coverage. Tick-order tests cover inherited phases, producer
placement, one simulation per step, activation and destruction.

`tests/integration/verify_character_classes.py` tests derived C++/Blueprint/Lua
actors, automatic motors, complete motion-record round trips, actual displacement
and camera-before-player ordering. Six runs cover 18 provider cases and 2,160
automatic steps. Both VM-source standalone exports rebuild byte-identically;
Native Play passes both domains. The workspace passes 1,266 tests / 88 ignored,
and unused Basic retains the prior RAM/EXE sizes.

The command-producer milestone passes 1,269 workspace tests / 88 ignored and
an additional focused Native Play test for authored nested command/destination
properties in both domains. Six native suites cover the policies and shared
mechanisms. The producer console matrix covers 18 provider cases and 2,160
steps/callbacks, with identical records and actual motion in every domain/mode.
Both standalone exports rebuild byte-identically, and unused Basic remains
byte-identical. See `artifacts/gameplay-helpers/p5/command-producers-validation.json`
and the P5 progress record. The animation-driver milestone also passes 1,275
workspace tests / 88 ignored, 18 console provider cases, exact playback traces
and both byte-identical standalone exports. The final production starter routes
pass fifteen cases / 4,200 steps, including actual one-way/lift traversal,
step/wall/camera obstruction and player-only Dungeon collection/progression.
The complete game and final helper/template resource matrix remain later phases.
