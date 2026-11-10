# Runtime input, time and collision

The Inspector's **Add Box Collider** attaches a serialized collider to any entity.
The selected collider is drawn as a green world box (amber for a trigger).
Center and half extents are local coordinates. The runtime transforms the box
through its complete hierarchy, then uses the enclosing world AABB, including
inherited scale, rotation and shear. Rotated boxes therefore collide against
their conservative bounds rather than an oriented narrow phase.

Local extents must be at least one Q12 unit (1/4096); validation also checks the
transformed size and ±512 world bounds. The same data is written to the exported
C++ scene banks. A collider can be attached, enabled, disabled or removed through
`entity().add<epok::Collider>()`, `get<Collider>()` and `remove<Collider>()`.

## Controller and simulation

For logical actions, shared-button consumption and temporary menu/player control,
see [Gameplay actions and control capture](gameplay-controls.md).

`epok::input` reads pads through Epok's SIO0 controller driver, sharing PsyQo's
Memory Card bus arbiter. Port arguments are
zero-based and default to the first port. `held`, `pressed`, and `released` take
a `epok::Button` (`Cross`, `Start`, `Up`, `Left`, etc.). A disconnection releases
all held buttons. Edges are buffered until a simulation update runs and delivered
once, including when several fixed updates catch up during a rendered frame.

`input.analog(port)` reports whether the current packet contains analog axes.
`input.axis_raw(Axis::LeftX/LeftY/RightX/RightY, port)` returns signed Q12 values
from -4096 to +4096, with positive X right and positive Y up. The centered ADC
value 128 maps exactly to zero; both endpoints reach full magnitude. Digital
controllers, disconnected ports and invalid arguments return zero. Four logical
controllers are available: Pad 1..4 map to a PSX multitap's Pad1a..Pad1d sockets
on native hardware and to the matching virtual pads in Native PC Play. Axis state
is sampled independently of button edges and remains available on catch-up ticks.
The input layer does not impose a dead zone: controllers should apply a radial
dead zone and limit diagonal magnitude before deriving movement speed.

Analog stick (0x53) and analog pad (0x73) packets are supported on every active pad.
The controller or emulator must be configured to send analog packets; digital
keyboard button overrides do not synthesize variable stick values.
The Play dropdown's **Analog controller (port 1)** option selects an analog pad
and enables analog mode for that emulator session. It is saved with the project,
defaults off for existing projects, and preserves machine-local button bindings.
It does not send configuration commands to a physical console controller.

The runtime measures PsyQo GPU time in microseconds and advances simulation at
60 Hz. Catch-up is bounded to eight steps per rendered frame; excess steps are
dropped and counted by `epok::time.dropped_steps`. Q12 `dt` alternates between
68 and 69 raw units, totaling exactly one second every 60 simulation steps.
Animation and particles advance during simulation, independently of rendering.

`epok::time.set_paused(true)` pauses simulation. `Behaviour::frame_update` still
runs once per rendered frame and receives unscaled elapsed microseconds. Use
`input.frame_pressed()` and `frame_released()` there for pause menus. Paused input
edges are discarded so resuming does not replay gameplay actions.

```cpp
void frame_update(epok::Transform&, uint32_t) override {
    if (epok::input.frame_pressed(epok::Button::Start))
        epok::time.set_paused(!epok::time.paused());
}

void update(epok::Transform&, epok::Fixed dt) override {
    using namespace epok;
    Fixed movement[3] = {0.0, Fixed(-6.0) * dt, 0.0};
    if (input.held(Button::Left)) movement[0] -= Fixed(3.0) * dt;
    if (input.held(Button::Right)) movement[0] += Fixed(3.0) * dt;
    const MoveResult result = move_and_slide(entity(), movement);
    if (result.grounded) { /* project-specific locomotion */ }
}
```

## Controller rumble

The reflected Input library exposes the same finite motor requests in C++,
Blueprint and Lua Gameplay profile 2 (native, VM bytecode and VM source):

```cpp
// Large motor intensity, small motor switch, duration in milliseconds, logical pad.
epok::InputLibrary::rumble(192, true, 150, 0);
epok::InputLibrary::stop_rumble(0);
epok::InputLibrary::set_rumble_enabled(false, 0);
const auto status = epok::InputLibrary::rumble_snapshot(0);
```

In Lua, use `epok.input.rumble(192, true, 150, 0)`,
`epok.input.stop_rumble(0)`, `epok.input.set_rumble_enabled(false, 0)` and
`epok.input.rumble_snapshot(0)`. Blueprint's Input category exposes the matching
calls and the pure `rumble_supported` query.

Large intensity is clamped to 0..255; the small motor is on/off. Duration uses
unscaled time and is capped at 60 seconds. A zero duration or two zero motors
stops the pulse. Each new accepted request replaces the previous pulse on that
pad. The boolean result means the request was accepted, including while a
connected controller is being configured; it does not confirm physical movement.
A queued pulse keeps its original deadline. `rumble_supported` becomes true
after negotiation, and the snapshot distinguishes probing, enabled and active
states. Analog sticks alone do not establish motor support.

The PSX driver configures DualShock motors without forcing physical analog mode.
Pad 1..4 use the first physical socket's direct controller or multitap sockets
A..D. Logical Pad 2 does not name the second physical console socket. The driver
waits for Memory Card/PocketStation ownership to end before touching SIO0, so a
pending stop reaches hardware on the next legal transaction.

For Native PC, assign each physical gamepad under **Project Settings > Controls**.
The assignment belongs to the current desktop session; projects retain portable
button/axis profiles. **Test Rumble (150 ms)** targets that selected device.
Changing the selected pad, closing Controls or moving focus away stops that
test. Output failures appear beside the device; hover the status for details
and use Test Rumble to retry after correcting the OS/device problem. Searching
Settings for `rumble` or `vibration` finds the relevant controls and preference.
Windows/Linux output uses the available device backend; Native PC on macOS
reports unsupported with the current GilRs dependency. Host effects use 50 ms
timing steps, so very short pulses are rounded to the host scheduler's resolution.

**Editor Preferences > Play > Rumble** allows or mutes desktop output. A game
can independently disable a pad through `set_rumble_enabled`. Pause, Stop,
Reset, lost Game capture, scene retirement and video playback cancel active
pulses; returning to gameplay requires a new request. Reconnecting or assigning
another device never replays the previous device's pulse.

PCSX-Redux drives its own physical controller. Epok selects an emulated DualShock
when the build uses rumble and preserves the emulator's bindings. Its controller
assignment is separate from Native PC's assignment. Older custom emulators
without motor output produce a Console message; use Install / Repair in
Dependencies to install the pinned build. Device vibration, multitap behavior,
stop latency and console performance still require acceptance on the intended
hardware; this implementation was delivered without running those checks.

For an isolated manual check, open
[`examples/controller-rumble/Controller Rumble.epokproject`](../examples/controller-rumble/Controller%20Rumble.epokproject).
The [example guide](../examples/controller-rumble/README.md) explains the displayed
status codes, individual motor controls, four-slot routing and cancellation
checks. Its two scenes let you change maps during a long pulse. The example
uses reflected Lua operations and can select Native C++, VM Bytecode or VM Source
without changing the script.

For native serial diagnostics, set `EPOK_PAD_TRACE=1` before building to print
bounded controller configuration transitions. The trace requires a serial-debug
build that uses rumble; normal builds omit these messages. Changing this setting
invalidates the existing native build certificate. Controller input and motor
negotiation run independently of trace output.

## Queries and triggers

All query coordinates and displacements are in world space. Queries refresh
transforms so one Behaviour sees movement performed by earlier Behaviours in the
same simulation tick. Movement transforms the resulting displacement back into
the mover's local parent space.

| API | Result |
| --- | --- |
| `raycast(origin, displacement, mask, ignore, triggers)` | Nearest finite-segment hit, fraction, point, normal, and inside flag. The displacement is the entire segment, not a unit direction. |
| `collider_aabb(entity, output)` | Current active collider bounds. Returns false for a missing or disabled collider. |
| `overlap(box, outputHandles, capacity, mask, ignore, triggers)` | Total matching count; writes only up to capacity. Strict touching is not overlap. |
| `query_ground(entity, distance, mask)` | Nearest floor within nonnegative distance below the collider's full footprint. Triggers are ignored. |
| `move_and_slide(entity, displacement, mask)` | Applies a swept movement, up to three sliding planes and bounded overlap recovery. Reports displacement, contacts, grounding and unresolved overlap. |
| `hit_entity(hit)` | Generation-checked entity handle for a ray or ground hit. |

Queries default to all layers. The `ignore` entity defaults to null and raycasts
exclude triggers by default. Overlap includes triggers by default. Movement and
ground automatically ignore the querying entity and intersect the query mask
with its collider mask. `layer` and `mask` are 32-bit bitfields, not layer indices.

Triggers never block movement. `Behaviour::on_trigger(EntityHandle other,
TriggerPhase phase)` receives `Enter`, `Stay` and `Exit` at simulation boundaries.
Both participants receive the callback when they have active scripts. For a
destroyed participant the exit handle is invalid; always check `other.get()`.
Trigger pairs require both colliders' layer/mask filters to match. At most 256
overlapping pairs are retained; excess pairs are dropped deterministically.
Triggers detect overlaps at tick boundaries, so a shape that crosses entirely
through a trigger in one tick should use a segment query for that fast interaction.

Collision provides static boxes, queries and kinematic movement. It does not
implement impulses, rigid bodies, capsule geometry, slopes or mesh triangles.
Initial penetration is resolved for at most eight passes; callers can inspect
`unresolved_overlap` when geometry cannot be resolved. No collider means movement
returns that flag without moving the entity.

## Interaction helpers

In the Content Browser, select **Game Helpers** to browse the existing helper
classes for the current 3D, 2D or UI view. Drag an Actor into its matching scene
view or a component onto the selected Actor's Inspector. Double-click performs
the same placement or attachment. The Actor picker also has a Game Helpers
filter, and Add Component groups compatible helpers together. These operations
use the normal class model, component validation and scene undo/redo.

Add `InteractorComponent` to the player and `InteractableComponent` to an object or NPC in either spatial domain. The interactor finds nearby targets, retains the best eight and chooses by priority, distance and a stable cooked order. The cook derives that order from the component UUID; renaming or rearranging actors does not change a tie. `range_3d` uses world units and `range_2d` uses pixels. Confirmation checks availability again.

Automatic interaction consumes the routed Interact frame edge (Cross by default) before simulation. A press used for interaction cannot also become a later routed Jump press. Set `auto_input=false` to call `interact()` explicitly, or `proximity=false` to require sensor registration. `overflow_count()` reports candidates or registrations that exceed the bounded storage; sixteen registrations retain overlapping sensor identities independently.

`TriggerBox3D` and `TriggerArea2D` appear in the actor picker. Assign their Sensor's typed `target` to the object's Interactable component. Optionally assign `accepted_initiator` to one actor; actors without an Interactor are ignored. Use a separate sensor actor when the target must keep a solid collider. Each explicit sensor costs one actor with its root and sensor components. `set_monitoring(false)` immediately removes its registrations. `binding_status()` distinguishes a missing target, invalid lifetime and mismatched domain or level. Nonpositive dimensions disable the sensor.

`InteractionPrompt` is a UIActor with Text and Presenter components. Place one shared prompt for a player's input port, assign Presenter.interactor and configure its Rect Transform and Text font. An Interactable's `prompt_key` uses the same String Table row index as `TextComponent::set_text_key`; `-1` hides the prompt. Builds include the project table even when no authored Text references a key. They validate configured indices, translated glyphs and the longest selectable prompt against the HUD budget. Language changes update the active prompt. Conversation assets use symbolic localization keys.

Derive an Interactable to implement behavior. The native `can_interact(ObjectId) const` event is a synchronous Boolean query; Blueprint and Lua pure overrides reject writes, consuming input and effectful calls. Use `interaction_started(initiator, port)` for mutations. The default request completes synchronously. Call `hold_open()` during the start event to keep it busy, then `complete(result)` once the operation finishes. Disabling or destroying the target/initiator cancels a held request and releases its busy state.

```cpp
class EPOK_CLASS(Blueprintable) Examine : public epok::InteractableComponent {
public:
    bool can_interact(epok::ObjectId initiator) const override {
        return epok::InteractableComponent::can_interact(initiator);
    }
    void interaction_started(epok::ObjectId initiator, uint32_t port) override {
        epok::InteractableComponent::interaction_started(initiator, port);
        // Apply this project's examination result here.
    }
};
```

Blueprint classes derive from the same native parent and override these events; Call Parent preserves the inherited behavior. The equivalent Lua extension is:

```lua
---@class Examine : epok.InteractableComponent
local Examine = epok.InteractableComponent:extend()
---@override
function Examine:can_interact(initiator)
    return Examine.super.can_interact(self, initiator)
end
---@override
function Examine:interaction_started(initiator, port)
    Examine.super.interaction_started(self, initiator, port)
end
return Examine
```

The interaction foundation is validated with host lifecycle tests, real 2D/3D trigger physics, a localized Native Play prompt, all five PSX provider surfaces and a byte-identical standalone export rebuild.

## Dialogue helpers

Add `DialogueInteractionComponent` to the object or NPC in place of the base
Interactable. Assign its conversation asset and a placed dialogue widget. The
widget is an ordinary `UIActor` class derived from `DialoguePresenter`, with its
own controls and C++, Blueprint or Lua behavior. Edit its hierarchy, geometry,
fonts, images and colors in the common UI class designer. The four editable
starters and their actual component costs are described in
[Dialogue widget classes](dialogue-widgets.md). An optional timing asset supplies
reveal/advance timing and sounds; appearance belongs to the widget's controls.
A missing required control cancels presentation and releases capture.

For conversations with participants, add `DialogueRoleBindingsComponent` and assign it to the interaction's `bindings` reference. Its eight role/target pairs map the conversation's compact participant IDs to live actors or components. Each instance supplies its own map. Conditions use a reflected pure `bool(ObjectId initiator)` event; actions use `void(ObjectId initiator, uint32_t port)`. Cooking validates class/member identities and signatures, and starting validates the live receivers.

`DialogueSessionLibrary::start(source, initiator, port)` returns a `DialogueStartResult` with a handle and result. `snapshot(handle)` reports phase, result, current node, reveal/page/choice state and typed object references. `advance`, `move_selection` and `cancel` require that generation's handle; an old handle cannot affect a replacement session. `owner_session(source)` returns the source's active handle. Completed state remains readable with the original handle until another session starts or the level resets. Blueprint exposes these operations in Dialogue; Lua Gameplay v2 uses `epok.dialogue.start`, `snapshot`, `valid`, `advance`, `move_selection`, `choice_id`, `owner_session` and `cancel`.

Derived interaction classes can override `dialogue_started`, `dialogue_node_entered` and `dialogue_finished` and call the parent. The start event precedes node entry, including conversations that end immediately. A derived widget can override pure `can_present(source)`, `present(snapshot)`, `dialogue_opened(snapshot)` and `dialogue_closed(snapshot)`. Calling parent `present` updates bound content, visibility and focus while preserving authored geometry, fonts, colors and panel material. The base displays localized paginated text, four choices per page, optional speaker names/portraits and configured UI sounds. Source/presenter loss, scene reset and cancellation release modal controls. Reveal uses unscaled frame time, including paused game time.

Dialogue text uses symbolic String Table keys. Pageable tables support long translations; each rendered page remains within the text payload limit. The widget's actual control areas and font metrics determine wrapping and page breaks. Choice labels must fit and produce a cook diagnostic rather than truncation. Builds follow strings, portraits, fonts, panels and sounds from every node and validate the reusable composition against configured HUD budgets before Play. Conversation content has its own graph editor; presentation uses the common UI class designer and explicit Play exercises the class's real behavior.

## Verification

`python tests/runtime/verify_spatial.py` compiles and executes the collision
kernel with the actual PsyQo fixed-point type. It covers thin walls, sliding,
parallel rays, ground footprints, penetration recovery, masks, trigger phases,
slot generations, controller edges, pause, timer wrap and bounded catch-up.
It also executes lifecycle and scene-service tests, including 100 bank switches,
asynchronous XA owner quarantine and teardown callback reentrancy.

`cargo test collision` covers serialization defaults, round trips, malformed
colliders, inherited transforms and generated component initialization.
