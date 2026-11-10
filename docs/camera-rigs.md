# Camera rig classes

The native CameraRig3D actor owns a Camera3DComponent and a typed target reference.
FirstPersonCamera, ThirdPersonCamera, TopDownCamera and IsometricCamera are ordinary
derivable classes. Place them through the normal Actor workflow, assign `target`
in the Inspector and select their Camera component as the active view. There is
no separate camera authoring panel or class catalog.

The **Game Helpers > Characters** source in the Content Browser exposes these
same reflected classes in the 3D view. Drag a camera Actor into the scene or a
compatible SpringArm/orbit component onto an Actor's Inspector. The Actor picker
also has a Game Helpers filter.

| Class | Initial view | Components / remaining slots |
| --- | --- | --- |
| FirstPersonCamera | Distance 0, height offset 1.6, pitch 0, pitch range -85..85 | Root + Camera: 2 / 6 |
| ThirdPersonCamera | Distance 5.4, height offset 1.1, pitch 18, pitch range -10..55; collision boom | Root + Camera + SpringArm: 3 / 5 |
| TopDownCamera | Distance 10, pitch 90, no height offset | Root + Camera: 2 / 6 |
| IsometricCamera | Distance 8, yaw 45, pitch 35.264, no height offset | Root + Camera: 2 / 6 |

All positions, offsets and distances use 3D world units; angles use degrees.
IsometricCamera is a fixed-pitch **perspective** preset. Its projection remains
the existing renderer's perspective projection. Field of view and background
remain properties of the normal Camera3DComponent.

## Follow, input and switching

The target can be any live Actor3D in the same level, including a target whose
position comes from an existing spatial parent. References use object identity,
so display-name changes do not affect following. The rig itself is an independent
world camera: use logical parenting to organize it; a spatially attached rig
reports InvalidOwner rather than writing world coordinates into a local transform.

Offsets are world-axis offsets from the target. `follow_response=0` follows
immediately. A positive response smooths the focus point using a clamped
response-times-delta factor. Changing target or calling `snap()` resets that
history. A rig with `follow_target=false` uses its authored pose as a fixed view
and does not require a target. Authored roll is preserved.

Add CameraOrbitInputComponent to a first-person or third-person rig to use routed
right-stick input. It occupies one extra component slot and reads only its
configured pad port. Its typed `rig` binding defaults to its owning rig; it can
control another live rig in the same level. By default, only the current camera
consumes its look commands. The default yaw/pitch rates are 140/70 degrees/s.
Look capture prevents turning; movement capture leaves turning available.

Input runs as a normal component tick before its owning rig's actor tick. For
scripted updates, disable `automatic_tick` and call `refresh(delta_seconds)` once
after producing commands. `set_angles` wraps yaw and clamps pitch to the authored
range; `add_look` accepts yaw/pitch deltas in degrees. Neither method reads a pad.
An NPC, replay or cinematic can drive these methods directly.

`make_active()` snaps the rig and uses the existing active-camera service. It
returns false when the pose is unavailable, its camera is disabled or a callback
invalidated the owner. Other rig instances retain their own follow and boom
state. `sample()` returns the latest result; `refresh()` returns its own applied
result as well.

## Optional SpringArm

ThirdPersonCamera includes a SpringArmComponent. Other rigs can add it through
the ordinary component Inspector. An explicit `spring_arm` binding must point to
a SpringArm on that rig; an empty binding discovers its own singleton arm.

The initial collision policy preserves the starter's finite ray query, ignores
the target's own collider and skips triggers. It subtracts a 0.04 pull-in fraction
from the hit and clamps it to a minimum fraction of 0.12. The mask, pull-in,
minimum fraction and collision enable flag are editable. Retraction is immediate;
`extension_response=0` extends immediately, while a positive response smooths safe
extension after an obstruction disappears. It adds no physics world.

The arm's `solve(origin, offset, ignore, delta_seconds)` API can also be used on
another Actor3D. Its sample contains the resolved world position, fraction and
blocked state. `reset()` clears extension history. The rig sample's distance is
the requested boom length; its position is the actual resolved camera position.

`adaptive_pull_in=true` halves the hit fraction when an obstruction is closer
than twice the configured pull-in, then applies the minimum fraction. This keeps
a positive safe clearance near the origin when `minimum_fraction=0`. Its default
is false, preserving the starter's original collision policy.

For systems sharing a [query context](spatial-query-context.md), use
`solve_shared(origin, offset, ignore, delta_seconds, query_context, collision_revision)`.
The context and a nonempty ignore reference must be live objects in the arm
owner's level. An unavailable context or invalid ignore returns InvalidTarget;
an inactive arm owner returns InvalidOwner. Failed operations apply no pose.

Revision zero performs a fresh ray every time. A nonzero revision permits reuse
when the consumer, revision, ignore identity, mask, origin and offset all match.
Advance that revision after relevant direct world edits. Resetting the context,
publishing an actor or moving through its typed API also invalidates the cache.
The context owns only one recent native ray: a different consumer replaces it
and executes its own query. Eviction never drops work. Resetting the arm forces
its next query to refresh. Smoothing still advances on every successful solve.

The shared ray cache occupies the optional query context, rather than every
camera instance. Native consumers borrowing the raw batch must publish their
direct edits or update the supplied collision revision themselves.

## Inheritance and events

All three authoring providers can derive from the native presets. Override
`calculate_view(delta_seconds)` to change the view policy. Calling the parent
retains following, smoothing and optional boom collision; the returned record
contains position, yaw, pitch, roll, requested distance, blocked state and status.
`refresh` applies a successful view and emits `view_updated(view)`. This also
works for derived fixed-camera policies. A nested refresh from either callback
returns Busy. Disabling or destroying an owner during a callback cannot apply a
stale pose or activate a destroyed camera.

The base has 13 editable properties, leaving three project properties within the
existing 16-property hierarchy budget. No actor/component limits are raised.
Cameras occupy a separate actor from their targets and count toward the level's
64-actor budget. Unused camera helper classes receive no factories or pools;
routed-input state is requested when CameraOrbitInputComponent is used.

An optional native ancestor can remain as type metadata without reserving its
own creation pool. Its descendants retain inherited controls, callbacks and
casts. Placing the ancestor, selecting it in a ClassRef property/default, or
referencing it from gameplay retains its normal factory. This includes native
type names, compact class IDs, stable UUID constants and Lua class builtins.
Inherited component factories and their usual spare capacity remain available.

C++ descendants can declare constant defaults for inherited scalar properties:

```cpp
class EPOK_CLASS(Blueprintable) ShoulderCamera : public epok::ThirdPersonCamera {
public:
    ShoulderCamera() { offset_x=0.4; distance=4.5; pitch_max=65.0; }
};
```

Reflection schema 11 records these assignments without executing constructors.
The Inspector, source catalog, Blueprint/Lua inheritance and runtime construction
use the same values and property identities. This restricted constructor syntax
accepts parameterless assignments to reflected scalar properties. Calls, loops,
initializer lists, macros, duplicate assignments and unreflected state writes
remain rejected. Fixed defaults use decimal floating-point literals, as other
reflected Fixed defaults do. General runtime setup belongs in lifecycle events.

## Results and lifecycle

Zero delta returns NoStep without advancing follow or arm state. Invalid negative
or above-one-second deltas and invalid tuning return InvalidConfiguration.
Missing/inactive targets return InvalidTarget; invalid explicit arm bindings
return InvalidArm. No unsuccessful refresh applies a new pose. Changing targets,
disabling the rig and end play reset follow history; the arm resets through the
normal component lifecycle.

Host acceptance compares 720 exact Q12 boom positions with the original routine,
including 133 obstructed positions. It exercises all four presets, independent
identity, follow/extension smoothing, masks, fixed-camera roll, switching, routed
look capture, nested callbacks and destruction during callbacks. The console
fixture is `tests/integration/verify_camera_rigs.py`; it derives each preset in
C++, Blueprint and Lua and exercises each Lua execution mode.

Acceptance covers 36 derived provider cases / 504 camera checks at 640x480,
four byte-identical standalone rebuilds, and a derived Native Play camera.
`tests/integration/verify_camera_views.py` separately captures the initial native
preset views at 320x240 progressive, checking unchanged angles from RAM and
waiting at least ten frame callbacks. Its four images have been visually
inspected. Behavior-test screenshots follow deliberate angle changes.

The default-view fixture's raw 480i VRAM captures show alternating old/current
fields. This remains a P10 capture/field-synchronization check; the progressive
images and passing 480i behavior/export checks do not close it. Use `--width 640`
and a separate `--output` folder to reproduce the diagnostic capture.

The umbrella header loads these classes only when the build retains a camera
helper or detects a native reference to its types. Generated builds and Native
Play enable the same `camera-rig` capability. Native libraries can declare
`EPOK_NATIVE_REQUIREMENTS("camera-rig")`; direct `#include "camera_rig.hpp"`
also works. Unused projects retain no camera pools or camera header code.
