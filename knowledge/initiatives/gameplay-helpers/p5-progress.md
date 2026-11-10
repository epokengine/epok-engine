# P5 progress and acceptance evidence

Status: in progress. P0-P4 are validated. The complete P5 acceptance criteria and
P6-P10 remain required in [the implementation plan](plan.md).

## Shared 3D motor extraction

The first implementation is `runtime/character3d.hpp`. It extracts the existing
third-person Q12 movement sequence into a native command-driven mechanism and a
reflected, derivable CharacterMotor3DComponent. Desired world X/Z velocity,
heading and jump are separate from pad sampling, animation and camera logic.
The public contract is in [the motor documentation](../../../docs/character-motor.md).

The component uses existing Actor3D ownership, Level lifecycle and collision
queries. It has nine editable properties and occupies one component slot. It
does not change the eight-component or 64-actor limits. The shared SDK staging
list includes its header; the helper retention path excludes an unused motor's
factory and pool, while source/member/class demand and inheritance retain it.

| Evidence | Current result |
| --- | --- |
| `tests/runtime/character3d.cpp` | 1,440 frames match the frozen original routine using real CollisionWorld queries; jump apex, wall, ceiling, step-up/down, turning and stopping assertions pass |
| `tests/runtime/character3d_lifecycle.cpp` | Real Level/registry tests pass: independent owners, rename, single-instance/domain constraints, jump latching, zero/invalid steps, manual/automatic tick, activation, destruction and continuing gravity after stop |
| `character_motor_uses_shared_reflected_records_without_an_unused_pool` | Pass: native reflection, nine properties, five methods, command/sample layouts of five/thirteen wire words, demanded/unused pools and byte-exact SDK staging |
| `tests/integration/verify_character_motor.py` | Pass in Native C++, VM bytecode and VM source: three providers per mode, 120 frames per provider, 1,080 frames total; actual movement and complete returned records agree across providers and modes |
| Workspace / API | 1,259 tests pass / 88 ignored; formatting, Clippy with warnings denied, both debug binaries, API freshness and five parity-tool tests pass |
| Unused Basic | 968,624 static RAM bytes / 425,984 EXE bytes: zero size delta from P4; motor factory/pool absent. Executable hash changed; no byte-identity claim |
| Native regression | Spatial/input/collision, gameplay values, control capture, interaction, dialogue widgets and object-model host suites pass |
| Standalone export | Pass: VM-source export includes the motor SDK header and rebuilds a byte-identical executable; final evidence is `artifacts/gameplay-helpers/p5/motor-validation.json` |

The console fixture places independent C++, Blueprint and Lua components over
the same native motor. It compares 120 frames per provider, actual displacement,
jump/grounded state and a digest of all thirteen returned sample words. Every
execution has a unique marker and each Lua mode changes that marker, so data from
the previous emulator run cannot satisfy acceptance.

The three-provider fixture uses 1,010,088 static RAM bytes in Native C++, 1,321,112
in VM bytecode and 1,382,552 in VM source. These are whole-project costs including
all three callers and test instrumentation, not a per-character estimate. All
cases finish grounded, perform one jump and agree on the complete frame digest.

An initial fixture used incorrect serialized collider property names. Its first
failure is retained in `p5-motor-native.log`; current fixtures use the canonical
`collider` component data and a large floor. A subsequent check added the unique
execution marker to prevent accepting RAM left over before a new executable boots.
That marker exposed an orphan emulator left by the first failed assertion. The
fixture now checks exclusive port ownership on Windows and terminates its entire
child process tree on failure. The orphan's command line was verified to belong
to this fixture before it was stopped. These fixture changes do not alter the
native movement algorithm.

The broader host regression exposed three object-only test fixtures that lacked
the data-index adapter required by P4's focus cleanup. They now explicitly return
no spatial index; the interaction fixture's sensor-only data still has no compact
HUD slot. Real slot identity/reuse remains covered by the widget tests.

## Camera classes and runtime milestone

Status: camera mechanisms and inheritance validated. P5 remains in progress.
Acceptance manifest: `artifacts/gameplay-helpers/p5/camera-validation.json`.

`runtime/camera_rig.hpp` now contains CameraRig3D, the four native presets,
SpringArmComponent and CameraOrbitInputComponent. They use normal reflected
properties, component defaults, typed references and overridable view events.
See [camera classes and contracts](../../../docs/camera-rigs.md).

The native host test passes 720 Q12 original-camera positions, including 133
obstructions, and covers all presets, smoothing, fixed views, roll, switching,
routed look capture, callback reentry and destruction. Native reflection and
placement tests pass all four presets with their actual component counts.
The fresh workspace suite passes 1,264 tests / 88 ignored, including a derived
camera in Native Play with inherited components and a custom view event. Nine
native regression suites, Clippy with warnings denied, formatting, generated API
freshness and five parity-tool tests pass. The inventory has 869 public gameplay
declarations and zero pending classifications. All four rigs pass C++, Blueprint
and Lua derivation in every Lua execution mode: 12 console runs, 36 provider
cases and 504 camera checks. Each rig's full nine-word view records agree across
providers and modes. All four VM-source standalone exports rebuild
byte-identically. The behavior/provider matrix and exports use 640x480.

A separate default-view fixture keeps the authored angles, verifies them from
RAM, waits at least ten frame callbacks after selecting each rig, and captures
all four views at 320x240 progressive. All four images were visually inspected;
the isometric capture uses yaw 45 / pitch 35.264. The earlier behavior captures
follow deliberate angle mutations and are not evidence of the preset's initial
framing. See `tests/integration/verify_camera_views.py` and
`artifacts/gameplay-helpers/p5/camera-visual-review.json`.

Direct 480i VRAM reads in that fixture contain alternating old/current fields,
including old loading-screen pixels. Original images and raw buffers are
preserved under `camera-views/raw-480i/`. Capture/field synchronization remains
an explicit P10 visual check; progressive visual acceptance and passing 480i
behavior/export checks do not resolve it or certify physical hardware.

The first console run exposed incomplete native camera placement through CLI/MCP;
both now use the same class placement/materialization path as the editor. A new
MCP test checks all inherited components and undo. An unused Basic build also
exposed 12,288 extra bytes in both static RAM and executable size from the camera
umbrella include. The camera header is now capability-gated for console builds
and host runners; the fresh matrix and Basic build validate this correction.
Basic retains 968,624
static RAM bytes / 425,984 EXE bytes, byte-identical to the motor baseline
(`138f3aae958eaf41e702e036e87b7474096da29c36b69274dbac9b1901bbc559`).
All seven camera helper pools and the camera capability remain absent. Evidence:
`artifacts/gameplay-helpers/p5/basic-camera-cost.json`.

Reflection schema 11 adds constant constructor defaults. Parent defaults remain
editable and preserve property identities through source catalogs and the shared
Blueprint/Lua property resolver. Calls, initializer lists, hidden state writes,
duplicate assignments and macros are rejected. The focused helper suite passes
seven tests, including these positive and negative constructor cases.

## Character compositions and tick stages validated

Acceptance: `artifacts/gameplay-helpers/p5/character-classes-validation.json`.
This milestone does not close P5 or the remaining phases.

Character3D now embeds its normal root, collider and motor (three slots).
Character2D embeds its root, collider, existing body and command motor (four
slots). Both use normal component defaults and typed visual/camera references.
The SDK stages `character2d.hpp`; unused character factories/pools are omitted.
The focused reflection/placement test passes both native compositions and the
three/ten-word 2D command/sample records.

Level now dispatches Input, Gameplay, Physics and PostPhysics. Native phase
constants are inherited by C++/Blueprint/Lua generated classes. Packed private
actor flags retain the prior host Actor (128 bytes) and ClassDescriptor (96)
sizes; host Level grows from 960 to 968 bytes. The first Basic measurement
retains 968,624 static bytes / 425,984 EXE bytes, with all character pools absent.
The executable hash changes with the shared dispatch code. Final Basic budget
validation passes with zero size delta and hash
`568a54d2974826b9bf6a21bfe84064c2c69071a39b0350e32682366ca8e4f2dd`.
Normal properties and unrelated constants named `tick_group` retain Gameplay;
only a correctly typed public native policy selects another stage.

The real-solver 2D reference test passes 720 Q12 comparisons: 478 grounded, 210
wall, six ceiling, six jump and 176 one-way-platform contacts, including 120
path-follow carry steps. It also tests pending jumps, zero/invalid updates,
foreign body references, sensor-before-solid composition and floating movement.
The existing World2D service suite passes. Its object-only fixture now explicitly
returns no compact renderer/HUD slot, as required by P4 focus cleanup.

The final SDK passes six console runs: both domains in all three Lua execution
modes, with C++, Blueprint and Lua subclasses in each. All 18 provider cases
agree over 2,160 automatic simulation steps, including actual displacement,
complete motion-record returns, pending status handling and same-step camera
following. Both VM-source standalone exports rebuild byte-identically. Native
Play passes both domains with an observer placed before the character.

The full workspace passes 1,266 tests / 88 ignored. Ten native suites, Clippy
with warnings denied, formatting, API freshness and five parity-tool tests pass.
The API inventory has 900 public gameplay declarations and zero pending entries.
An outdated descriptor-string assertion was updated to include the new native
phase field; its initial failure log is retained. See
[the implemented contracts](../../../docs/character-classes.md).

## Command producers validated

Acceptance: `artifacts/gameplay-helpers/p5/command-producers-validation.json`.
This milestone leaves animation, template adoption and P6-P10 open.

Delivery, 30 October 2026: shared extensible command sources,
ScriptedCommand2D/3D, MoveTo2D/3D, LocalPlayer2D/3D input and Player/NPC presets
are implemented. Player/NPC use four native slots in 3D and five in 2D, leaving
four/three extension slots. Producers run in Gameplay and submit to the existing
Physics-stage motors. Motor command revisions prevent an older source's release
or lifecycle callback from clearing a newer submission. Commands and destination
vectors use the common reflected configuration value Inspector.

Six native suites pass (`p5-command-native.log`): command policies, both motors,
camera rigs, routed controls and tick stages. The new producer fixture exercises
four ports, D-pad/analog input, explicit/active camera direction, independent
capture with continued gravity, jump edge latching/consumption, NPC target/point
policies, floating/floor-tangent 2D movement, same-step dispatch, source handoff,
foreign references and callback reentry/destruction. Nine helper tests pass
(`p5-command-helpers.log`), including preset placement, optional pools/markers,
six/four-word result records, nonzero nested configuration defaults and rejection
of unsupported runtime records or incomplete/dynamic configuration defaults.
The lexical capability test also passes (`p5-command-reflection.log`).

The first reflection run rejected command/destination values as runtime-only
records. Configuration extraction now accepts the explicit complete constant
initializers of command and GameplayVector values; result/identity records stay
ineligible. The failure is retained in `p5-command-helpers-initial-failure.log`.
A compile error in the active-camera fallback was corrected to derive heading
from the world forward basis, and that fallback passes the native test.

The final workspace suite passes 1,269 tests / 88 ignored, including Native Play
in both domains with movement capture and continued gravity. Clippy with warnings
denied, formatting, API freshness (142 headers / 2,539 callables), five parity
tool tests and the 945-public-declaration parity inventory pass. The new input
query has a pinned identity and prunes captures from an unavailable registry.
The initial workspace failures were an outdated operation-count assertion and a
new frame callback reading tick edges; both are corrected, with initial logs
retained. The frame callback now uses `frame_sample`.

Unused Basic remains byte-identical at 968,624 static bytes / 425,984 EXE bytes,
with all twelve producer/preset pools omitted. Evidence:
`artifacts/gameplay-helpers/p5/commands-basic-cost.json` and
`p5-command-workspace-final.log`, `p5-command-clippy-final.log`,
`p5-command-api-check.log`, `p5-command-native-play-final.log`.

Console acceptance passes all six domain/mode runs: 18 provider cases and 2,160
automatic steps/submission callbacks agree exactly. Each mode records the same
analyzed/played executable hash and probe address. Both VM-source standalone
exports rebuild byte-identically with matching current SDK headers.
An additional Native Play test passes authored nonzero nested command properties
and NPC destination properties through scene staging and into both motors
(`p5-command-authored-native.log`). This test was added after the 1,269-test full
suite; its focused run passes independently.

An initial eight-second VM-bytecode run left the completion marker
unwritten; a fresh thirty-second run completes all 120 steps identically in the
three providers. The final matrix uses the longer execution window and records
the probe address and executable hashes. No cause for the earlier incomplete
observation is asserted.
`tests/integration/verify_character_commands.py` exercises calculated result
returns, submission callbacks and actual automatic motion for three providers
in both domains. These new classes have not replaced production templates yet.

## Animation drivers validated

Implemented `SkeletalLocomotionComponent` and `SpriteLocomotion2DComponent` as
ordinary optional PostPhysics components. They use typed motor/visual bindings,
editable clip/tuning records, selection and applied events, and normal playback
services. There are seven/nine exposed properties, leaving nine/seven hierarchy
property slots. A character with local input, visual and driver uses six/seven
component slots in 3D/2D. See [the current contracts](../../../docs/character-animation.md).

Six native regression suites pass. The new fixture compares 2,400 exact Q12
state/phase decisions with the former starter, then checks real motor playback,
sprite events/facing/rate, air/landing, same-step order, foreign bindings and
reentry/destruction in both driver hooks. Focused reflection/capability tests
pass, including the staged header and omission of unused factories. The API
reference currently generates 143 headers / 2,574 callables.

The first Lua extension fixture exposed a missing Gameplay v2 operation: local
record-field assignment. Typed local fields now lower through shared IR to C++
member writes or VM functional wire replacements. Nested/vector offsets are
validated; replacement preserves value copies and never aliases native data.
Direct edits of a native record property require a local copy and whole-value
assignment. Profile v1 remains unchanged. The compiler unit and 80 Lua-related
regressions pass. The console fixture checks independent copies, nested motion
edits, booleans/enums, rate overrides, applied callbacks and actual cursors in
each provider. Native Play passes both domains. The final full workspace run
passes 1,275 tests / 88 ignored, and unused Basic remains byte-identical at
968,624 static RAM bytes and a 425,984-byte EXE. Clippy, formatting, API freshness,
five parity-tool regressions and the parity inventory pass (995 public operations,
6,257 total, zero pending).

The VM-bytecode matrix found a real parity failure: the Lua actor left Land three
steps before C++/Blueprint. Per-step traces show the first divergence at step 66.
The VM method boundary supplied numeric 0/1 for boolean parameters without
normalizing them; Lua treated `finished=0` as true. Chunk generation now converts
boolean parameters before executing the body. The targeted regression, fresh
workspace run and repeated console matrix pass after this correction. Initial
failure logs and traces are retained. All six domain/mode runs pass: 18 provider
cases, 2,160 steps/callbacks and 10,800 exactly compared trace words. The 3D
VM-source standalone rebuilds for both domains are byte-identical, with matching
current SDK headers. Acceptance is recorded in
`artifacts/gameplay-helpers/p5/animation-validation.json`.

Production template migration is under acceptance. The component milestone
does not close P5 or the later template/helper phases.

## Third Person starter foundation validated

New Third Person projects (and the Dungeon starter built on them) now compose
Character3D, an inherited motor, project input derived from
LocalPlayer3DInputComponent, SkeletalLocomotionComponent and ThirdPersonCamera
with SpringArm/orbit input. Player/camera compositions use five/four slots;
the visual remains a child actor. Typed identity references replace all
controller display-name searches. Each flavor installs only its own small
project extension; legacy movement/animation/orbit implementations remain test
comparisons and are not installed into generated projects.

The five generation/compiler tests pass. The fresh workspace passes 1,276 tests /
88 ignored; Clippy passes with warnings denied.
Unused Basic remains byte-identical at 968,624 static RAM bytes / 425,984 EXE
bytes. The renamed C++ template passes the real console route: 240 steps, all
six animation states, two jumps, camera/motor/playback checks and gravity
progress during an input capture triggered by the actual jump event. Blueprint
and all three Lua execution cases also pass with identical semantic results:
five cases / 1,200 automatic steps. C++, Blueprint and Lua VM-source standalone
exports all rebuild byte-identically with matching SDK headers.

Inserting the route's empty input actor exposed positional access to an otherwise
valid identity-based lighting cache. Scene-header generation now matches cached
colors by actor UUID for declarations and object bindings. A focused regression
passes after inserting an empty actor before baked geometry, including in the
fresh full workspace run.
Logs use the `p5-third-person-*` prefix. Scoped acceptance is recorded in
`artifacts/gameplay-helpers/p5/third-person-validation.json`. This route does not
certify every arena obstacle or Dungeon runtime; remaining starter routes,
Platformer adoption and P6-P10 remain open.

## Template adoption acceptance history

The final real starter routes below complete adoption of the common mechanisms
in the production starter templates, including the obstacle, one-way and lift
checks that remained open at the earlier milestones.

The Third Person/Dungeon starter now uses a project controller that
derives from shared local-player input; Character3D, locomotion and camera
components replace the copied systems. Scoped Third Person and Platformer
acceptance is recorded above/below. Obstacle/one-way/lift and Dungeon runtime
routes remained open at that point; the final acceptance below closes them.

## Platformer starter foundation validated

The generator now builds on Character2D, shared local-player input and the sprite
locomotion driver. C++/Lua input sources and the generated Blueprint subclass no
longer copy the motor or sprite policy. Coin rules remain project-owned in the
selected flavor. The camera moves to an attached Actor2D so the player can use
seven component slots and retain one extension slot.

The first composition test found an extra Collider2D: that class intentionally
permits multiple shapes, so generic hydration does not choose an authored one.
The fresh template now explicitly binds its single authored shape to the
Character2D collider declaration before hydration. Three generation/compiler/
staging tests and Clippy with warnings denied pass. The initial C++ console route
passes 240 automatic steps, idle/walk playback, two jumps, twenty gravity steps
during capture, attached-camera following and one collected coin. Its static RAM
is 1,693,000 bytes including the observation fixture. The full five-case matrix
and three byte-identical standalone exports now pass. One-way/moving-platform
template routes and visual acceptance remain separate checks. Scoped acceptance:
`artifacts/gameplay-helpers/p5/platformer-validation.json`.

The repeated workspace passes 1,277 tests / 88 ignored after updating the split-
screen UI test to select the new Player Camera actor. C++, Blueprint, Lua native
and Lua VM bytecode pass the console route. VM source fails the actual budget:
2,067,584 static bytes leave 29,568 bytes, below the unchanged 65,536-byte runtime
reserve. The largest avoidable allocation is the Lua wire scratch pool (135,168
bytes for 16 frames × 64 values × 32 words plus tags).

The VM packager now emits a program-wide value-width header from typed bodies,
field slots and generated method signatures. Runtime scratch entries use that
width, with a three-word vector minimum and the existing 32-word language limit;
frame/value counts and allocator budget remain unchanged. Nine VM unit tests
pass, including full-width parameter/member snapshots and a >32-word negative
control. The fresh VM-source route passes with identical semantic results at
1,948,800 static bytes, saving 118,784 bytes and leaving 148,352 bytes free. Four
real console cases validate three-word and full thirty-two-word snapshots in
both VM modes, including independent local copies and all fields returning to
C++. The new full workspace passes 1,278 tests / 88 ignored. Clippy, API freshness
(143 headers / 2,574 callables) and parity inventory (995 public / 6,257 total,
zero pending) pass. The repeated Platformer matrix now passes all five cases
with identical motion/playback/coin results. Twelve callback-provider cases
pass across both domains and VM modes with 7,200 exactly compared trace words.
The compact three-word case also passes vector round trips and independent
local edits in both VM modes. Unused Basic remains byte-identical with zero
RAM/EXE delta. All three flavor exports rebuild byte-identically, including the
generated three-word capacity header in the Lua VM-source export. The optimized
path is included in the scoped Platformer acceptance manifest.

## Real starter surface routes validated

The shipped Platformer geometry passes five execution cases with identical
results: 14 upward one-way passages, 86 grounded observations on that platform,
one normal jump and 160 carried observations on the actual moving lift (158 with
platform motion). The arena passes five cases: 26 step-up observations, step-down,
70 wall-blocked observations, 70 spring-arm obstruction checks and 80 clear/full-
distance checks. The fixtures seed poses and pad input; the selected project's
normal command source, motor, animation and camera execute the route.

Dungeon passes C++, Blueprint and Lua Native C++ with the renamed player, camera,
key and door. An unrelated actor actually enters the key trigger and is rejected;
the assigned player then collects it and opens the linked door. VM bytecode fails
the unchanged memory gate: 2,093,712 static bytes leave only 3,440 bytes against
the required 65,536-byte reserve. Evidence is preserved under
`artifacts/gameplay-helpers/p5/dungeon-foundation/lua/`, including
`vm-bytecode-headroom-failure-memory.json`.

The map allocates 131,324 bytes for the legacy 2,048-triangle sprite/VFX pool,
although it contains no billboard sprites, emitters or particle effects. New
Dungeon projects now select the existing editable 64-triangle rendering profile;
other templates and the renderer's global defaults retain their settings.
Generation tests assert that content still has no sprite/VFX users. The complete
fresh Dungeon five-case route now passes with identical motion/key/door results,
zero observed sprite/particle use and real rendered-frame telemetry. Bytecode
uses 1,966,736 static bytes and leaves 130,416 bytes free; source uses 2,026,128
and leaves 71,024. The original failure is retained separately. The fifteen
real starter cases cover 4,200 steps and are sealed in
`artifacts/gameplay-helpers/p5/starter-routes-validation.json`, with per-scenario
editor hashes, matching current SDK hashes and played executable identities.

P5 is complete as the common character/camera/locomotion foundation and starter
adoption phase. The fresh workspace passes 1,284 tests / 88 ignored; Clippy,
formatting, whitespace and API-reference freshness pass. P6 has a scoped catalog/
resource-transaction milestone, while its world-gameplay helpers and P7–P10 remain
open. The 480i capture, complete-game frame-time failure, runtime heap/stack peaks
and complete thirty-case/export/budget acceptance retain their later gates.
