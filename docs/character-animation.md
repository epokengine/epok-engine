# Character animation drivers

`SkeletalLocomotionComponent` and `SpriteLocomotion2DComponent` are optional,
derivable PostPhysics components. They consume the shared motor's latest motion
sample and use the existing skeletal/sprite playback implementations. They read
no pad input and copy no movement solver. Provider/runtime acceptance is in
progress; these classes have not replaced production template controllers yet.

## Bindings and configuration

An empty `motor` reference selects the owner's compatible character motor. An
empty `visual` selects the controlled Character's visual reference, then a Mesh3D
or Sprite2D component on the controlled actor. Explicit bindings must be active
objects in the same Level. A visual can belong to a separate actor. The component
Inspector edits these ordinary typed references and the configuration records.

`LocomotionClips` contains six signed indices into the bound visual's clip bank:
idle, walk, run, jump_up, jump_down and land. A negative index leaves that state
unconfigured. Skeletal defaults are all -1. Sprite defaults use idle 0 and walk/run
1; air states are unconfigured and `animate_air` defaults to false, matching the
starter's idle/run policy. No character-specific clip names are built into the
engine. Assign the visual's ordinary model or SpriteFrames asset separately.

`LocomotionTuning` groups idle speed, run-enter/run-exit hysteresis, walk/run cycle
speeds and minimum/maximum playback rates. Skeletal defaults retain the former
third-person thresholds and cycle speeds: 0.12, 3.6/3.0, 2.6/6.0 and 0.2–1.6.
Sprite defaults use pixel speeds: 0.12, 128/112, 96/192 and 0.2–1.6.

Skeletal has seven exposed properties; Sprite has nine. Their hierarchies retain
nine/seven property slots for project extensions within the common limit of 16.
Each driver consumes one normal component slot. Character + local input + visual
+ animation uses six slots in 3D or seven in 2D, leaving two/one component slots.
A separate visual actor and camera count toward the normal Level actor budget.

## Selection and playback

Idle, Walk and Run use speed and hysteresis. An airborne character selects
JumpUp/JumpDown from velocity along its up direction, then Land on ground contact.
Land holds until the bound clip finishes. Sprite `animate_air` enables these air
transitions; its default idle/run policy ignores grounding. `face_motion` retains
the last horizontal facing at rest and uses the body's floor tangent for motion.

The skeletal driver's `distance_driven` default advances walk/run cycles with
ground speed and pauses the automatic cursor, preserving the former Q12 phase
sequence. Other states retain automatic playback at rate 1. An overridden rate
uses an explicit clock, including looping and clamped non-looping clips. Sprite
uses its existing speed scale and frame advance service, preserving frame events
and authored durations. Its `distance_driven` defaults to false. Sprite looping
belongs to the clip asset; a selected looping value must agree with that asset.

`select_animation(motion, previous, finished)` is an overridable event returning
`LocomotionSelection {state, clip, looping, rate}`. `animation_applied(sample)`
runs after a valid selection reaches the visual. The five-word sample contains
status, state, clip, rate and whether a transition/rebinding restarted playback.
Call Parent to keep the default state policy, or return a project selection.
Both events are available in C++, Blueprint and Lua. Blueprint can split the
returned record's pins. Gameplay v2 Lua can edit local snapshot fields:

```lua
function MyAnimation:select_animation(motion, previous, finished)
    local selected = MyAnimation.super.select_animation(self, motion, previous, finished)
    local edited = selected
    edited.rate = 1.25
    return edited
end
```

Here `MyAnimation` is the project's derived animation component class.
Local record/vector edits preserve copy semantics in generated C++ and both VM
modes, including nested members. A native property record is read into a local
before editing; assign the complete record back when changing configuration.

## Status and lifetime

`update(dt)`, `sample()`, `reset()` and `set_enabled()` share the normal component
API. Disable `automatic_tick` before manually updating. Zero delta preserves
playback. Invalid owner, motor, visual, tuning/selection or missing clip returns
an explicit status without applying playback changes. Reentry returns Busy.
Callbacks revalidate object identities and bindings before applying a selection.
Disabling/resetting clears driver state and leaves the shared visual untouched;
the next valid update establishes a new selection. Disable a driver while another
project system owns that visual's playback.

## Current evidence

The native fixture compares 2,400 Q12 choices/phases with the former third-person
policy, then exercises actual motors, playback, sprite events/facing/rate,
air/landing, same-step order, foreign bindings and callback destruction.
Focused reflection tests validate the configuration, records and optional pools.
Lua tests cover typed local copy updates and existing compiler behavior.
The console fixture `tests/integration/verify_character_animation.py` passes all
six domain/execution runs: 18 provider cases and 2,160 steps/callbacks. Three
providers override rate, retain the parent policy and check snapshot independence,
callbacks, all six states and actual playback cursors. Their per-step traces agree
exactly across 10,800 compared words. The workspace passes 1,275 tests / 88 ignored.
Both VM-source standalone exports rebuild byte-identically; unused Basic retains
zero RAM/EXE delta. Evidence: `artifacts/gameplay-helpers/p5/animation-validation.json`.
Production template adoption remains a separate P5 acceptance requirement.
