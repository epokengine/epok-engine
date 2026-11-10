# Shared 3D character motor

`CharacterMotor3DComponent` is the common native movement mechanism. Add it to an
Actor3D with an enabled Collider3DComponent. It is derivable and its properties,
commands and motion samples use the same reflected declarations in C++, Blueprint
and Lua. It does not read a pad, select animations or move a camera.

## Commands and update order

`CharacterCommand3D.velocity` is desired **world X/Z velocity in world units per
second**. Its two components are X and Z; this is not a raw stick or a displacement.
`heading` is in degrees. Set `turn` to approach that heading and `jump` to request
a jump on the ground. A local player, an NPC and a scripted sequence can produce
the same command.

```cpp
epok::CharacterCommand3D command;
command.velocity = {3.0, 0.0};
command.heading = 90.0;
command.turn = true;
motor.submit(command);
```

The most recent velocity and heading persist until the next command. Submitting
a second command cannot erase a pending jump. The next successful simulation
consumes the jump once, including when the character was airborne; this initial
motor has no jump buffering window or coyote time.

With `automatic_tick=true`, the normal component tick simulates the motor. For a
command produced later in the owning actor's tick, set `automatic_tick=false`,
submit the command, then call `simulate(delta_seconds)` once. Use one update owner
for the motor so it advances once per simulation step. A zero delta returns
`NoStep` and retains a pending jump. `motion()` returns the latest sample.

`stop()` clears desired motion and pending jump. Existing velocity responds to
the configured acceleration response and gravity continues. This lets input
capture stop movement commands without freezing the body in midair.
`reset_motion()` clears velocity, command and sample and reads the owner's current
facing; it does not teleport the owner. Begin/end play reset the motor. Disabling
the owner clears its command; the level's normal activation rules suspend ticks.

`set_facing(degrees)` assigns project-controlled facing for the next successful
step, without another turn-rate calculation. It rejects angles outside
±360000 degrees. `clear_velocity()` clears velocity and pending commands while
preserving facing and grounded contact. It is useful when a menu freezes a
character: stop simulating while the menu is open. An airborne character resumes
gravity on its next simulation step. `stop()` retains its existing meaning of
clearing commands while velocity continues to respond to physics.

## Configuration and results

Defaults preserve the starter's Q12 operation order: gravity 24 world units/s²,
jump speed 9 world units/s, ground stick -0.05 world units per grounded step,
step height 0.6 world units, ground/air response 12/3 per second, turn rate
540 degrees/s and collision mask `0xffffffff`.

`simulate` returns `Applied`, `NoStep`, `InvalidOwner` or
`InvalidConfiguration`. A missing/inactive/non-3D owner, missing data slot or
disabled collider returns `InvalidOwner`. A negative delta, delta above one
second, negative gravity/jump/step/response/turn rate or out-of-range numeric
input returns `InvalidConfiguration` without changing body or motion state.
Numeric limits are gravity/jump/response 4096, desired velocity ±4096 per axis,
state velocity ±8192, step height 64, turn rate 360000 and ground stick -8..0.
Heading subtraction must fit a signed 32-bit Q12 value. These are validation
bounds, not useful tuning defaults.

`measure_speed` defaults to true. Setting it to false skips the horizontal
speed calculation and returns zero in `sample.speed`; displacement, velocity,
facing, contacts and jump results are still returned.

The sample contains actual displacement, internal velocity, facing, horizontal
speed from displacement/delta, grounded/jumped state and initial sweep's
blocked/unresolved-overlap flags. Step recovery can resolve an initial block;
inspect the resulting displacement and grounded state for the final movement.

The native collision service owns wall sliding, grounding and overlap recovery.
The motor preserves the existing lift/retry/settle step-up and ground-query
step-down policy. It adds no physics world or global per-character storage.
Setting `step_height` to zero disables stair recovery and ground snap probes.
Each component owns its state and uses its owner's identity rather than a display
name. The class is single-instance per compatible owner and occupies one of the
existing eight component slots. Unused helpers receive no factory/pool.

## Shared query contexts

`simulate_shared(delta_seconds, query_context, support_revision)` uses the
existing SpatialQueryContextComponent's collision batch. The context must be
live, active and in the motor owner's level. An invalid, expired or foreign-level
reference returns `InvalidContext`, without moving the body or consuming a
pending jump. `simulate()` keeps using the ordinary world query service.

A zero support revision always performs collision queries. A nonzero revision
allows the motor to reuse grounded support for a stationary downward stick step.
The caller must change that revision whenever a world, support, collider or
ancestor-transform change can affect this character's contact. With a manual
context (`automatic_reset=false`), publish direct collider/transform edits as
described in [shared queries](spatial-query-context.md). Resetting the context
invalidates support independently of the caller's revision. The default automatic
context resets each Input tick and is the conservative option.

The bounded per-motor entry checks tuning, registry/owner/context identities,
data-slot pointer and generation, local position/rotation/scale, parent slot and
context-reset epoch. Context replacement, body rebinding, reset and disable
cannot reuse the previous contact. Each motor owns its entry, so sharing a query
context does not mix support results between characters. This adds no physics
world and does not suppress a moving or jumping character's sweep.

## Validation

`tests/runtime/character3d.cpp` compares 1,440 Q12 frames against the original
movement routine using real collision queries: jump apex, wall response, ceiling,
step-up, step-down, diagonal turning and stopping. The lifecycle fixture tests
two independent owners, duplicate rejection, rename, pending jumps, zero/invalid
steps, manual/automatic tick, command stopping, inherited activation and wrong
owner domains.

`tests/integration/verify_character_motor.py` exercises commands and complete
motion records across the native motor boundary from C++, Blueprint and all Lua
execution modes on the console. The fixture's provider classes contain only
submit/simulate/sample calls; command production and observations are native test
instrumentation, with no second movement implementation.

`tests/runtime/character3d_shared.cpp` compares the reflected component with the
frozen Complete integration over 1,440 Q12 steps and 524 idle query skips. It
also covers context lifetime/reset, pending jumps, tuning/pose changes, data-slot
rebinding and generations, and independent consumers.
`tests/integration/verify_character_motor_shared.py` drives actual submit,
shared-simulation, facing and velocity-clear calls from C++, Blueprint and Lua,
including both VM modes and a standalone rebuild.

Character3D and Character2D compositions and automatic simulation stages are now
validated; see [character classes and simulation stages](character-classes.md).
Input sources and animation drivers share those classes. Player, Third Person
and Dungeon use the common 3D foundation; Dungeon Complete remains under migration.

Native integrations can supply the internal `character3d::advance` kernel with
a query adapter implementing `move` and `ground`. This preserves the same
integration order while reusing an already synchronized collision service.
The optional `measure_speed=false` skips only the horizontal speed calculation;
displacement, velocity and contact results remain available. The component
exposes the same choice through its `measure_speed` property.
