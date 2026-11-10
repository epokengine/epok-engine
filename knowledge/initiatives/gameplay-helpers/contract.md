# Gameplay helper implementation contracts

P0 establishes these contracts before changing shipped gameplay. Native declarations become the signature authority when each phase is implemented. Existing actor-only and gameplay API parity contracts remain binding.

The final template delivery requires **18 combinations**: all six templates in
C++, Blueprint and Lua Native C++ AOT. Lua source owns editable game rules;
generated C++ is a build artifact. VM bytecode/source keep their independent
engine functionality and regressions, without a Dungeon Complete execution gate
for this delivery. Functionality, 2MiB RAM, 64KiB runtime reserve, 25ms frame-time
and standalone-export requirements remain unchanged for every required case, with
one exception below.

### Dungeon Complete frame-time gate (user decision, 7 November 2026)

Dungeon Complete is held to parity with P0, not to zero slow frames, because P0
never met zero. 25,000us still defines a slow frame. On the identical pad schedule,
inside the same rendered-frame window, each of its three providers must report:

| Measure | Guards route | Boss route |
| --- | ---: | ---: |
| Window, rendered frames | first 993 | first 988 |
| Slow frames, at most | 10 | 8 |
| Maximum combat frame, us | 27,008 | 25,280 |
| Maximum frame anywhere, us | 27,328 | 27,328 |

This replaces only the zero-slow requirement for this one template. Every other
template keeps its existing gate. A later decision may restore zero slow frames.

Later on 7 November the user accepted the reached result as technical debt, because
the game runs smoothly in general and the work had taken long enough. The table stays
the target, but for this template it is reported and does not block delivery. Every
functional, memory, export and content gate is unchanged, and so is every other
template's frame gate. Measured on the same routes, in the same window:

| Provider | Slow, guards / boss | Maximum combat frame, us | Maximum frame anywhere, us |
| --- | ---: | ---: | ---: |
| C++ | 10 / 10 | 26,752 / 26,624 | 26,752 |
| Lua | 19 / 9 | 27,648 / 28,544 | 28,544 |
| Blueprint | 25 / 10 | 29,376 / 27,520 | 29,376 |

Open against the target: all three boss combat maxima, C++ boss slow count, Lua and
Blueprint guard slow counts and their maxima. Only presentation waits for the drawn step;
every step still simulates the same game whatever the frame rate. These counts move by
several frames between nearly identical builds, because a small change decides which
frames land on a two-step frame. Compare them over more than one build.

## Ownership

| Complete game subsystem | Shared native mechanism | Rules owned by each selected project flavor |
| --- | --- | --- |
| Movement and animation | Fixed-point body integration, collision queries, locomotion playback | Desired movement, jump decisions, combat animation policy |
| Cameras | Follow/orbit evaluation, obstruction queries, projection | Camera mode selection, aim/lock transitions and targeting decisions |
| Combat | Bounded projectile/effect storage, health operations, spatial queries | Attacks, damage amounts, cooldowns, invulnerability and reward decisions |
| Encounters | Typed actor activation and shared snapshots | Guard and boss states, activation conditions and victory/defeat |
| Inventory | Bounded records, serialization, item definition data | Use, buy, sell, limits, prices and upgrade decisions |
| Save | Versioned byte encoding, asynchronous Memory Card transport | Commit/rollback and persistent versus transient state |
| PocketStation | Device protocol, installation/inspection and asynchronous transport | Terminal flow, transfer accounting, save-before-acknowledgement and receipts |
| UI and dialogue | Text/layout/reveal, focus, session evaluation, capture tokens | Conversation conditions/events, shop/inventory actions and game messages |

No provider delegates its project rules to an original hidden DungeonGame implementation. The Pocket Lua device program remains common content; its PSX orchestration uses the selected flavor.

## Stable identity allocation

Use UUID v5 in the namespace derived by `uuid5(NAMESPACE_URL, "https://epok.engine/gameplay-helpers/v1")`. Public names are allocated once; later renames retain the allocated identity. P1's checked native IDs and subsequent reserved identities are recorded in `identities.json`.

Template role IDs identify Player, Visual, Camera, GameRules, prompt/presenter and target bindings separately from their class identities. A shared composition builder returns the role mapping; per-flavor classes bind it using generation-checked actor/component references. No display-name lookup or class-name inference is an identity contract.

## Gameplay actions

The append-only GameplayAction enum starts with MoveX, MoveY, LookX, LookY, Jump, Interact, Confirm, Cancel, Pause, Attack, Aim, CycleTarget, Inventory and Sprint, followed by eight custom action slots. Axis actions expose signed Q12 values. Button actions expose zero or one. The existing host input profiles continue converting devices into the four logical pads.

Default bindings preserve current gameplay: movement uses LeftX/LeftY with digital overrides; look uses RightX/RightY; Jump, Interact and Confirm use Cross; Cancel uses Circle; Pause uses Start; Attack uses Square; Aim uses L1; CycleTarget uses R1; Inventory uses Triangle; Sprint uses L3. P1 provides bounded per-port button/axis rebinding.

GameplayInputLibrary is the shared reflected function library. Core value types are:

```cpp
struct InputCapture { uint32_t slot; uint32_t generation; };
struct GameplayActionSample {
    Fixed value;
    bool held, pressed, released, connected;
};
```

These occupy two and five ABI wire words respectively, below the current 32-word record limit. Null captures use an invalid slot and zero generation. Tokens are opaque values; fields do not authorize reuse after expiry.

The library exposes action masks, base/frame samples, context samples, consuming press operations, capture/release/validity and bounded binding configuration. Temporary routed button/axis adapters retain the current controllers' algorithms until P5. Raw InputLibrary is unchanged.

`owner_capture(owner, exact_action_mask, port)` returns the newest matching live token. This lets Lua retain a context across callbacks through its owner identity without adding arbitrary record-valued property constructors to the language. Captures remain owned by the native service and must not be reacquired every tick.

### Capture priority and lifecycle

- Sixteen live captures are available. Capture returns an invalid token on exhaustion, an invalid owner/port, an empty mask or unknown actions, with no partial mutation.
- Each capture records owner ObjectId, level ObjectId, input port, action mask and signed priority. Higher priority wins; the newest live token breaks equal-priority ties. Lower captures resume after the winner releases.
- Action aliases share physical input ownership. Capturing Confirm on Cross also prevents the uncaptured Jump/Interact aliases from receiving Cross. Context reads require the requested logical action to belong to that token's mask.
- Owner and level generations are checked before every public routing read/mutation and at dispatch boundaries. An inactive, destroyed, ended or unloaded owner expires its captures; inherited parent activation applies.
- Releasing one token never releases another. Disabled/destroyed owners, nested callbacks, reset, level exit and out-of-order release cannot retain a lock or revive an old token.
- Bindings and captures are reset at scene-service reset. Token generations are not reset; wrap exhausts allocation rather than reviving an old token.
- Capturing Move/Look gates command input, not physics or time. Existing gravity and collision continue. P1 does not globally pause gameplay when a context opens.

### Frame and simulation edges

The runtime samples raw pads once per rendered frame, then advances frame callbacks, followed by zero or more simulation steps. The router uses the same ordering and owns its own bounded edge buffers.

1. Begin frame copies raw frame edges into frame and pending simulation buffers. A rapid press/release on frames without a tick survives until a simulation step.
2. Begin tick transfers pending edges once; subsequent catch-up ticks see no repeated edge.
3. Consuming a press clears that physical source in frame, tick and pending views, including action aliases. Plain sample methods are state reads and do not consume.
4. Capture acquisition/release/expiry discards affected pending/current edges so a context transition cannot reinterpret the opening or closing press as a jump, another interaction or another confirmation.
5. While paused/loading, simulation edges are discarded after frame input handling. Frame input remains usable; a session chooses one update domain and does not advance from both callbacks.

Unrouted raw input remains available and retains its current semantics. Capture protects callers of the gameplay router; migrated template entry points must consistently use it.

The `gameplay-control` capability selects runtime hooks. Function-local service storage and capability-gated dispatch prevent unused Basic projects from reserving a capture pool or running router polling.

## Interaction and dialogue

Interaction has a pure, const, bool availability query and separate mutation events. Predicate overrides and plain returned records must compile in native, Blueprint and all Lua modes before implementation uses those shapes.

Interactor candidates are capped at eight. Selection is explicit priority, distance, then stable authored/cooked role identity. Confirmation revalidates the target. Trigger filtering names the accepted initiator and uses generation-checked references.

Dialogue node kinds are Line, Choice, Branch, Event and End. Stable node/option/participant IDs are independent of layout. Sessions bind reusable participant roles to scene references. Text uses localization keys; styles reference fonts, portraits, panels and audio by asset identity. Conditions/actions bind validated reflected signatures by member IDs, not runtime string evaluation.

P3's version-1 authoring profile permits 1,024 nodes, eight participant roles, 64 callback bindings and 64 options per Choice (four displayed per page). Conditions are public reflected `pure const bool(ObjectId initiator)` methods; actions are public reflected callables/events with `void(ObjectId initiator, uint32_t port)`. A conversation stores class/member identities and participant UUIDs, never scene object UUIDs. The cook sorts node/role/binding tables by authored UUID and diagnoses collisions before mapping runtime identities to 32 bits; reordering the document does not change the cooked contract. Text references name a String Table by asset UUID and a symbolic key.

The initial profile has one modal session per level, four visible choices per page and at most 32 immediate node transitions per advance. Exactly-once node entry events are guarded against reentry and source/presenter destruction. Session end reasons distinguish complete, user cancel, source unavailable, scene unload, invalid binding and transition-budget failure.

UTF-8 and color markup remain intact during reveal. Full-string layout determines wrapping; each displayed page fits every selected translation into the 511-byte payload and configured primitive budget. Cooking follows every reachable conversation/style resource, including content absent from the initial authored Text.

Pageable String Tables opt out of the legacy HUD-table selection and permit up to 65,535 UTF-8 bytes per entry. Dialogue pages retain color state and split only at scalar boundaries. Every choice label and participant name must fit one configured label page; cooking reports the table/key/language rather than truncating it. Empty translations preserve that table's first-authored-column fallback. The project HUD table defines runtime locale indices; if absent, the project's lowest-UUID String Table does. Referenced tables map columns by trimmed, case-insensitive names and must contain exactly that locale set. Changing the selected scene set cannot change the meaning of a language index.

The P3 native kernel uses generation-checked scene receivers, one modal capture and fixed arrays. Frames advance reveal in unscaled microseconds. Session reset, source/presenter deactivation or destruction, scene unload and capture loss release control and discard resource pointers before notifying completion. Session generations never reset or wrap into a previously issued value; exhausting the counter rejects another start. The reflected facade must require the matching generation for mutations so an old handle cannot advance a newer conversation.

## Character and camera presets

Character3D derives Actor3D; Character2D derives Actor2D and reuses CharacterBody2DComponent. Input, scripted and AI command sources share the motor contract. Visuals, animation roles and cameras are typed configuration.

CameraRig3D derives Actor3D and owns a Camera3DComponent. Derivable presets are FirstPersonCamera, ThirdPersonCamera, TopDownCamera and IsometricCamera. They share explicit target, follow offsets, yaw/pitch limits and routed look input. SpringArmComponent provides configurable boom length, collision retraction and optional smoothing; rigs that do not require it omit it.

The isometric preset uses the current perspective renderer with fixed angle/distance. True orthographic rendering is a separate feature. Camera switching selects a live rig and releases the previous input capture; missing/destroyed targets fail predictably. Camera actor/component costs count toward the same map bounds.

SpringArm is a bounded camera-boom mechanism built on Epok's existing obstruction queries.

## P0 measurement gate

Generation covers all 18 template/flavor cells and records the four currently unavailable execution cells explicitly. Fresh linked baselines cover all six C++ templates and Third Person Blueprint plus three Lua modes. Existing host input/object-model/gameplay tests and real compiler predicate/record probes must pass.

Native runtime captures cover representative movement and complete-game explore/combat/boss/interface/restart states using the existing tooling. Static ELF allocation and unassigned RAM are measured separately from runtime peaks. Current tooling does not measure heap/stack peaks; those fields remain unknown, and P8/P10 retain a required peak-instrumentation gate. Arithmetic arena headroom is a planning estimate, not proof that a future complete-game Lua port fits.

No required failed check is relabeled as passed. Record pre-existing failures and their effect on the P1 dependency before deciding whether P0 is validated.
