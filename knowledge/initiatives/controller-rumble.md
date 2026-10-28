# Controller rumble: investigation and implementation plan

Status: implemented in the working tree; runtime, automated and physical-device
validation intentionally deferred to the user. No tests or game execution were
run. Debug editor binaries are built for the handoff.
Investigated on 2026-10-24 against Epok `9d20d0f56a45122dd668749c0c31308327687dd0`
and the current working tree. Existing uncommitted Native PC, scene lifecycle,
editor and settings changes are part of the observed baseline and must be
preserved. The investigation below records the pre-implementation baseline; the delivery
notes at the end describe the resulting implementation and remaining acceptance.

## Recommended scope

Implement a bounded DualShock-style rumble service shared by C++, Blueprint and
both Lua execution modes. Support a variable large motor, an on/off small motor,
finite pulses, explicit stop and device capability reporting. Route output to
the same logical pad that supplies input.

The first complete delivery should cover PSX pad negotiation/output, Native PC
on supported Windows/Linux devices, and a verified PCSX-Redux version. Test the
existing first-port multitap topology before claiming four-player PSX rumble.
Native PC on macOS needs a separate backend milestone: the current dependency
cannot deliver force feedback there. Unsupported devices must remain playable.

Defer effect assets, curves, layering, priorities, spatial attenuation, continuous
effects, trigger motors, HD haptics and legacy one-motor controller protocols.
These are not prerequisites for a useful first release.

## What the current code actually provides

| Area | Evidence | Consequence |
| --- | --- | --- |
| Input runtime | `runtime/input.hpp`, `Input::sample`, `poll`, `poll_multitap` | Four logical states, buttons and analog axes; no motor output or capability state. |
| PSX integration | `runtime/main.cpp`, `Game::prepare`, `GameScene::frame` | Uses `AdvancedPad` in Fast mode and always calls `input.poll_multitap(pad)`. |
| PSX SDK | `third_party/nugget/psyqo/advancedpad.hh`, `src/advancedpad.cpp` | Configuration opcodes exist as private constants, but configuration is explicitly TODO. `outputDefault` and `outputMultitap` send no motor commands. |
| Shared serial bus | `third_party/nugget/psyqo/sio0-bus.hh`, `AdvancedPad::readPad` | Pad polling yields while Memory Card/PocketStation transactions own SIO0. Rumble must respect this ownership. |
| Host input | `src/platform.rs`, `src/editor.rs`, `src/controls.rs` | `gilrs` events lose `event.id`. Buttons and axes from every physical gamepad share one state. Hotplug events are ignored. |
| Logical profiles | `src/controls.rs`, `Settings`, `Slot`, `Profile`, `PadState` | Four virtual pads and project-owned mappings exist, but no physical gamepad assignment. `PadState` has no connected/capability fields. |
| Native execution | `native/native_play_runner.cpp`, `src/native_play.rs`, `native/native_play_protocol.h` | EPN3 transports input inward and actor/HUD/audio/loading data outward. No rumble path. Runner currently samples all four pads as connected. |
| Game view focus | `src/gui.rs`, `src/platform.rs` | Input is gated by Game capture; application focus loss clears capture/buttons. Motor output will need its own immediate cancellation. |
| Public authoring | `runtime/gameplay_api.hpp`, `InputLibrary`; `knowledge/initiatives/gameplay-api-parity/contract.md` | Reflected declarations should produce Blueprint nodes, Lua APIs and adapters. No separate hand-maintained rumble node family is needed. |
| Export | `src/project.rs`, embedded runtime file list | A new runtime header must be embedded/staged or standalone exports will omit it. |
| Native staging/cache | `src/hud_simulation.rs`, runner source/protocol embedding | Native PC and HUD preview share staging infrastructure; update dependency hashing and keep previews from driving physical motors. |
| Lifecycle | `runtime/scene_lifecycle.hpp`, `runtime/lifecycle.hpp`, both `reset_runtime_services` implementations | Common scene teardown exists, but target-specific services are reset separately. Both paths need cancellation. |

### Dependency and emulator findings

- Nugget is the Epok fork, pinned to
  `bfc3f65010efaaf979e10852e4cf203a2eff1b13`. Its checkout was clean during this
  investigation. Extend that fork and update the pinned revision reproducibly.
- `Cargo.toml` allows `gilrs = "0.11"`; `Cargo.lock` resolves `gilrs 0.11.2` and
  `gilrs-core 0.6.8`. The cached source confirms Strong/Weak effects, `stop`,
  `set_gain`, finite repetition and `is_ff_supported`. The default Windows
  backend is WGI; the platform comment mentioning XInput is not the feature
  configuration. [GilRs force-feedback API](https://docs.rs/gilrs/0.11.0/gilrs/ff/index.html).
- In the resolved source, `gilrs/src/ff/time.rs` uses 50 ms ticks. Do not promise
  60 Hz physical modulation or precise sub-50 ms pulses. The local
  `gilrs-core/src/platform/macos/gamepad.rs` returns false from
  `is_ff_supported`, and `macos/ff.rs` is a no-op. Those are source findings for
  the locked versions, not assumptions about every future GilRs release.
- The configured local emulator is `.tools/redux/pcsx-redux.exe`.
  `.tools/redux/version.json` reports **25270.20260904.6.x64**, changeset
  `40e2b989c41101d353ac28be0629b80a3d890acf`. The corresponding source handles
  the unlock reply but has no SDL rumble output. It is unsuitable as evidence
  of physical rumble. [Installed-version source](https://raw.githubusercontent.com/grumpycoders/pcsx-redux/40e2b989c41101d353ac28be0629b80a3d890acf/src/core/pad.cc).
- `tools/dependencies.json` specifies **28745.20261020.31.x64**, a different
  version. Its exact rumble behavior has not been verified. Upstream `main`
  observed on the investigation date implements motor mapping, SDL output and
  a timeout, but that does not prove support in either packaged version.
  [Upstream pad implementation](https://raw.githubusercontent.com/grumpycoders/pcsx-redux/main/src/core/pad.cc).
- Epok's emulator bridge is TCP plus Lua (`src/bridge.rs` and
  `integrations/pcsx-redux/bridge.lua`). It currently overrides digital buttons
  on pad 1 and optionally selects analog mode. It has no motor return path.
  Prefer emulator-owned rumble for PSX execution; do not add a second host writer
  for the same controller.

### Port numbering is a real constraint

Current PSX gameplay indexes `AdvancedPad::Pad1a..Pad1d` as logical ports 0..3.
The second physical socket is `Pad2a`, SDK index 4, and is not sampled by the
current main loop. Therefore logical port 1 is **not** the second physical socket.
The older `Input::poll` helper is not the active runtime path.

Preserve the current mapping for this feature. A direct controller on physical
port 1 supplies logical port 0; logical ports 1..3 require its multitap. Share
the mapping helper between reads and writes. Adding a configurable two-direct-pad
topology is a separate, explicit input compatibility change; never silently send
logical player 2's rumble to the second socket.

## Proposed public contract

Add `runtime/rumble.hpp` with a fixed four-slot service. Expose the public
functions through the existing reflected `InputLibrary`, with stable new UUIDs:

```cpp
// Proposed signatures, not available today.
static bool rumble(uint32_t large, bool small, uint32_t duration_ms, uint32_t port);
static void stop_rumble(uint32_t port);
static bool rumble_supported(uint32_t port);
static void set_rumble_enabled(bool enabled, uint32_t port);
```

For example, `epok::InputLibrary::rumble(192, true, 150, 0)` requests a 150 ms
pulse on logical player 1. Blueprint gets an impure callable node with the same
arguments and a pure support query; Lua gets those reflected operations through
the existing registry. Expose the gameplay enable state in the diagnostic
snapshot as well. Game menus may persist that choice through their existing save
system; do not silently write a save file from this API.

| Contract | Decision |
| --- | --- |
| Motors | Large 0..255, clamped at the reflected boundary; small is boolean. Desktop output maps large to Strong and small to Weak. |
| Duration | Unscaled milliseconds, maximum 60,000. Zero duration or both motors zero cancels the current pulse. No indefinite operation in v1. |
| Invalid arguments | Port >=4 returns false/no-op. Invalid transport values are rejected, not narrowed silently. |
| Overlap | Latest accepted command replaces the pulse on that port. No hidden mixing or allocation. |
| Return value | Reports request acceptance, not proof that a physical motor moved. Disconnected, disabled and known-unsupported pads reject nonzero requests. |
| Negotiation | A connected pad still being probed may retain one request until its original deadline. Readiness never restarts or extends that pulse. |
| Capability | `rumble_supported` becomes true when this backend has established support. It is independent of analog-stick availability and user enable state. |
| Pause/scene/focus | Cancel and output zero. Resume does not revive a cancelled pulse; gameplay must request another. |
| Reconnect | Re-probe and begin with zero output. Never replay the previous device's pulse. |
| Disabled state | Effective output requires both gameplay enable and desktop preference enable. Neither can override the other. |

Keep backend status separate from input connectivity. Internal diagnostics should
distinguish probing, ready, unsupported, disconnected, disabled and backend error,
and include requested output, last submitted output and time remaining. A valid
emulated PSX handshake establishes protocol support, not host motor availability.

### Time and resource handling

- Store pulse state in bounded memory; no runtime effect list or heap allocation
  on PSX. Cap duration before converting to microseconds and use wrap-safe
  unsigned elapsed time.
- Advance expiration once per rendered frame against unscaled time, including
  frames with no simulation tick. Several catch-up ticks must not age it several
  times. Stage final output after gameplay callbacks so replacement is deterministic.
- Stop on gameplay pause, editor Pause/Stop/Reset, frame-step/debugger suspension,
  lost Game input capture, process/bridge failure, scene retirement and FMV entry.
  Suppress nonzero requests during these states. Add hooks before early returns
  in loading/video paths; `time.synchronize` must not prolong an old pulse.
- Clear pending pulses before teardown callbacks can enqueue replacement work;
  reject requests while teardown is active. Re-enable only for the new scene.
- Give host effects a finite lease, initially 250 ms with refresh at most every
  100 ms while a fresh active session snapshot exists. Never extend beyond a
  pulse's remaining duration. Validate actual scheduling and rounding in the
  GilRs prototype; these are proposed bounds, not measured guarantees.
- PSX cannot send zero while a card transaction owns SIO0. Expire locally, then
  make zero the next legal motor output. Test the physical stop delay and
  controller watchdog; do not promise immediate stop during a locked bus or a
  hung console.

## Backend design

### PSX: extend the existing AdvancedPad owner

Add per-pad configuration/readiness and desired motor state to the pinned SDK
driver. Do not install a second SIO0 poller. Configuration must be a bounded
state machine interleaved with input polling, with a retry limit/backoff and
separate packet parsing so configuration replies cannot become button edges.

Use enter-config `0x43`, establish the standard two-motor mapping with `0x4D`,
exit-config, then send motor values in normal `0x42` reads. Verify replies and
re-negotiate after disconnect/reset or loss of configuration. Do not treat
`0x73`, `0x53`, or analog input alone as evidence of rumble capability. Avoid
forcing analog mode (`0x44`) unless separately requested by controller policy.
The large motor is analog and the small motor digital; low large-motor values
may not physically start a motor. [PSX controller protocol](https://psx-spx.consoledev.net/controllersandmemorycards/#controllers-vibrationrumble-control).

First validate a direct DualShock, including one starting in digital mode.
Then implement and test per-socket multitap configuration and motor output.
Buffered multitap exchanges are not equivalent to a direct-pad packet: preserve
their address, command-position and timing behavior. Keep a dedicated topology
test gate; until it passes, report unsupported rumble on unvalidated sockets
while retaining their normal input.

All operations must obey `SIO0Bus::owned()`. Add bounded receive/ACK failure paths:
the present driver has unbounded RX/ACK-deassert waits as well as a bounded
ACK-arrival wait. Failed configuration must release SIO control and recover
without hanging the frame or disabling normal digital input. Use a deterministic
serial stub around the actual driver, extending the test pattern used by
`tests/runtime/pocketstation_sio_stub.hpp`.

Keep initialization opt-in through a build capability such as `input.rumble`,
integrated with the existing native requirement and reflected operation pipeline.
The relevant host integration points include `src/script_backend.rs` for native
requirements and `src/project.rs` for capability-driven runtime generation.
An unused rumble feature must not start configuration traffic or allocate a
host effect. Ensure direct C++ uses are detected or can declare the capability
explicitly; reachability cannot depend only on Blueprint nodes.

Update the SDK gitlink and every pinned revision reference, including
`tools/dependencies.json`, Linux/macOS setup, macOS packaging and runtime export
instructions/notices. Preserve the SDK's existing memory-card arbitration.

### Native PC: identity, bidirectional transport and host output

1. Preserve `gilrs::GamepadId` in platform events, track input per device and
   handle connection/disconnection explicitly. Assign one physical device per
   virtual pad; v1 disallows duplicate device assignment. Two identical models
   must remain distinct within a session.
2. Keep host identifiers in machine-local preferences/session state. Existing
   project mappings remain portable. Do not serialize a process-local GamepadId
   into the project or assume a model UUID uniquely identifies two identical
   controllers. Stop the old device before reassignment. Legacy profiles can
   resolve a default device for Pad 1; other slots require explicit assignment.
3. Extend the **next available protocol version after EPN3** in both directions.
   Carry virtual-pad connectivity separately from physical rumble capability and
   local enable policy toward the C++ runner. Keyboard-backed virtual pads can be
   connected with rumble unsupported; disabled slots must not report connected.
4. Return one bounded desired motor state per pad, remaining duration and a
   monotonically changing command revision. Tag shared state with Play session
   identity and scene generation. Never infer a new pulse from every repeated
   frame snapshot or reset a timer simply because the same output was resent.
5. Publish a bounded latest snapshot from `native_play.rs` to the platform event
   loop, which already owns GilRs. Keep device handles there; avoid a second
   GilRs context and avoid an unbounded per-frame event queue. Invalidate stale
   generation/session updates, and gate refresh on liveness/focus independently
   of a worker that can wait up to five seconds for a runner response.
6. Implement a small `src/rumble.rs` host driver behind a mockable output boundary.
   Retain at most two effects per assigned device, one Strong and one Weak,
   update gain/state, explicitly stop and drop on teardown. Override the
   builder's infinite-repeat default. Confirm finite lease refresh behavior with
   the resolved library before relying on it; do not recreate effects at 60 Hz.
7. Bump protocol/cache inputs together. Validate lengths, counts, ranges and
   truncated packets on both sides. Update old packet fixtures and the initial
   frame path: startup can precede delivery of physical-device capabilities.
   Include `src/hud_simulation.rs` in the staging/cache change. HUD and other
   editor previews must use an unavailable/no-output backend even when they
   execute a script that calls rumble; only active Play and the explicit device
   test can submit physical output.

For Windows/Linux, use the existing dependency and query actual capability.
Failures must disable that output and produce a useful, rate-limited diagnostic.
macOS initially reports unsupported. A later SDL3 or native GameController/Core
Haptics backend needs its own dependency/licensing review, input-to-device
identity reconciliation and tests; adding it is not a one-line GilRs fix.

### PCSX-Redux: let the emulator drive its devices

The exported PSX program should issue the same SIO protocol in embedded and
windowed emulator Play. Select/configure an emulated DualShock and let the
emulator map its motors to its physical controller. Do not equate GilRs IDs with
the emulator's controller indices or mirror the same pulse through both systems.

The first investigation milestone must identify a distributable emulator build
with working two-motor output and verify it, then update dependency pins/checksums
if needed. Confirm the existing Lua configuration API for that exact build,
preserve user bindings, and expose a clear unavailable status for an older custom
emulator. Confirm Pause/Stop/Reset and focus-loss behavior: the observed upstream
timeout is a fallback, not evidence of immediate cancellation through Epok.
If tighter stop requires an emulator API change, track that dependency explicitly
rather than bypassing the emulated controller with a host-only gameplay mailbox.

Do not promise four-pad emulator support from a four-slot Epok API. Validate the
selected emulator's multitap implementation separately. The inspected local
emulator source contains two emulated pads.

## Editor and authoring experience

Use Project Settings > Controls for logical slots and reusable input mappings.
Show machine-local physical assignments alongside them without exporting those
identifiers. Add a desktop rumble enable preference, per-pad support/status, and
an explicit short Test Rumble action that stops when the panel loses focus or
closes. Testing should target only the selected device, using the same driver.

Show the current Play backend and meaningful unavailable reasons (no assigned
device, unsupported device/backend, disabled, emulator too old). Keep protocol
and motor-register details out of the normal UI. The public runtime retains its
own per-pad enable API for in-game accessibility settings.

Register reflected operations in `runtime/gameplay_api.hpp` and use the existing
Clang/operation pipeline for C++, Blueprint, Lua native and Lua VM parity.
Regenerate Lua definitions and `docs/api/` through their existing generators.
Document real supported combinations in `docs/input-collision.md`, Controls and
Play documentation only when implementation and evidence exist.

## Delivery sequence and exit criteria

| Step | Work and principal files | Exit criterion |
| --- | --- | --- |
| P0: remove uncertainty | Exact Redux build; resolved GilRs finite-effect prototype; direct-pad configuration trace; tests/evidence only | Known motor path and measured timing for each first-release backend; Mac and multitap limits recorded. No dependency update based solely on `main`. |
| P1: portable service | New `runtime/rumble.hpp`; `runtime/epok.hpp`; frame/lifecycle hooks in both runners; `src/project.rs` staging | Bounded expiration, replace/stop, invalid ports, disabled state, wraparound, pause and scene teardown pass without physical hardware. |
| P2: device routing | `src/controls.rs`, `src/platform.rs`, `src/editor.rs`, `src/controls_ui.rs`, local settings | Two devices drive independent pads; hotplug/reassign never sends output to the wrong player; old project profiles still load. |
| P3: Native PC output | `native/native_play_protocol.h`, runner, `src/native_play.rs`, `src/hud_simulation.rs`, new host driver, platform/Play cancellation | Typed rumble reaches assigned hardware; old/truncated packets reject; stale sessions and every stop condition pass. |
| P4: PSX driver | Nugget AdvancedPad state machine, SDK tests/pins, runtime adapter and capability cooking | Direct digital/analog DualShock negotiation/output verified; digital-only pads unaffected; SIO0 coexistence and bounded timeout recovery pass. |
| P4b: multitap | Per-socket driver negotiation/output, shared logical-port routing | Independent Pad1a..d output verified with input and card traffic; no accidental writes to Pad2a. Required for four-player PSX support. |
| P5: authoring and emulator integration | Reflected library, registry/parity fixtures, Controls UI, Lua bridge/setup if required, documentation | Equivalent C++/Blueprint/Lua requests; embedded/windowed verified emulator drives motors; standalone export includes the feature. |
| P6: acceptance and Mac follow-up | Hardware/OS evidence, performance comparison; separate Mac backend if required | Publish the verified support matrix. Mac remains explicitly unavailable until its backend passes. |

Implement the service before integration. Device routing must precede host output.
P0 and P4/P4b contain the greatest uncertainty; UI polish should follow proof of
the motor paths. Avoid changing the input port topology or adding an effect-asset
system while closing this feature.

## Validation plan

| Layer | Required coverage |
| --- | --- |
| Service tests | Independent four-slot state; latest-wins; zero stop; clamping; unsupported/disconnected/disabled; negotiation expiry; 60-second bound; clock wrap; 0/1/many simulation ticks; no stale restart after pause, FMV or scene retirement. |
| Actual PSX driver with stubbed SIO | Command order and lengths; accepted/refused configuration; digital-mode DualShock; small/large output bytes; reconnection; malformed replies; RX/ACK timeouts; multitap routing; no bus access while a card owns it; normal input edges preserved. |
| Host routing tests | Two different and two identical controllers; disconnect/reconnect and ID reuse; reassignment; keyboard-only slot; disabled slot; duplicate assignment rejection; focus, panel close, disabled preference and shutdown. |
| Transport tests | Protocol mismatch, truncation, out-of-range fields, startup capability exchange, repeated snapshots, expiration, scene/session generation changes, delayed frames, crash/EOF, blocked worker and Pause/Reset races. |
| Authoring parity | Real extracted function signatures/UUIDs; mutation nodes have execution pins; query is pure; one fixture each for C++, Blueprint, Lua native and VM produces the same normalized motor requests. |
| Export/build | New header staging, native source/protocol cache invalidation, optional capability discovery, pinned SDK/setup/package consistency, standalone PsyQo build using exported sources. |
| Real devices | Windows and Linux on at least one supported dual-motor controller; original PSX with DualShock; four-socket multitap if claimed; actual two channels, large-motor threshold, duration, reconnect, rapid replacement and stop latency. |
| Emulator | Exact executable version/hash, embedded/windowed Play, physical controller mapping, hardware output, pause/focus/reset/close; record protocol success separately from motor motion. |
| Regression/performance | Existing input/analog/edge tests, scene transitions, Memory Card/PocketStation, FMV; compare no-feature baseline and enabled steady/configuration traffic, executable/RAM cost and frame-time tails. |

Extend `tests/runtime/verify_spatial.py` with focused rumble fixtures and reuse its
existing selective source arguments. Add focused Rust tests around routing and
packet decoding, plus a real Native PC fixture following the existing scene/HUD
integration patterns. Run `verify_gameplay_api_parity.py` and the relevant
Blueprint/Lua suites after reflecting the new operations. Check generated API
reference and SDK/export consistency. Synthetic tests establish logic and wire
behavior; they do not certify physical vibration or console performance.

Before any substantial build, inspect Cargo configuration and `CARGO_TARGET_DIR`,
free disk and cache size. Reuse canonical `target/` with the pinned toolchain and
existing debug profiles. Build `epok-header-tool` before tests that require the
extractor. Do not create a task-specific Cargo cache. No build or cache cleanup
was needed for the initial investigation; implementation used the canonical
Debug cache without deleting validation artifacts.

Keep future measurements under `artifacts/rumble/` with OS/backend, controller
model, connection type, exact executable/SDK versions, input/output traces and
observed timing. The feature is complete only for the combinations whose
acceptance evidence is recorded; a successful API call is insufficient.


## Implementation delivery (2026-10-24)

The implementation includes the finite four-pad service, reflected Input
operations and diagnostic value, Native PC EPN4 transport, host device identity
and assignment, host actuator leases, Controls device testing, editor preference,
scene/video/pause cancellation and standalone runtime staging.

The PSX integration uses an engine-owned `runtime/controller_pad.hpp` instead of
editing the pinned Nugget checkout. It is the sole pad poller, shares PsyQo's
SIO0 ownership arbiter and uses the documented direct multitap addresses 01..04.
Every byte/ACK wait is bounded; IRQ acknowledgement preserves unrelated pending
interrupts. Negotiation verifies the motor-map echo and retains normal input on
non-DualShock devices. The original SDK revision remains unchanged, and the
engine embeds the adapter in every export. This supersedes the proposed SDK
fork/gitlink update in P4.

The cooked capability is named `rumble` (the native requirement metadata accepts
hyphenated identifiers, not dotted names). Both direct C++ source observation
and reflected operations retain it. Builds without the capability do not send
configuration commands. Lua API definitions are generated from the same registry
on project catalog refresh; there is no separately maintained rumble signature.

PCSX-Redux continues to own its SDL actuator output. The bridge selects an
emulated DualShock for a rumble-enabled build, applies desktop enable policy and
writes a bounded cancellation-only RAM control record discovered in the link
map. It never mirrors motor output through GilRs. Its sequence counter cancels
old pulses even when disable/resume happens entirely between emulated frames.
The pad poller consumes that cancellation before submitting its next packet.
Old emulator builds without the Rumble setting produce a Console warning.

Windows setup now pins **29417.20261024.9.x64**, changeset
`997b68983b4195064483780e41d40e86f4a09145`; that package was installed locally from
its published archive after integrity comparison. Linux setup pins the archive
for the same changeset. The current macOS emulator pin `f9cd9b2c` already includes
SDL rumble and motor mapping in its source and was retained. These are source
and distribution checks, not motor or execution tests. Native PC on macOS still
reports unsupported with the locked GilRs backend, as scoped for the first
release.

Host assignment is explicitly session-local. Pad 1 automatically chooses an
available device; other slots require assignment, and moving a device removes
its previous assignment. Disconnection clears that device's state and identity;
a manually selected disconnected device is not silently replaced. The editor's
own output preference and the game's per-pad enable state both gate output.

The API reference and parity inventory were regenerated from source. The API
extractor reports no diagnostics. The new PSX controller header also passes the
pinned MIPS compiler's syntax phase. Editor and reflection-extractor Debug
binaries are compiled with the locked dependencies; these are development
builds, not functional acceptance runs. Per user instruction, acceptance runs
(P0 physical timing, emulator execution, host tests, full game execution, multitap
hardware, regressions and performance measurements) are deferred until the
complete implementation is ready for the user to exercise. No claim of measured
hardware latency, physical four-pad behavior or successful test execution is
made by this delivery.

## Final implementation review and manual handoff (2026-10-24)

The follow-up review closed the remaining implementation issues found before
handoff:

- Redux's output-enable policy no longer depends on successful initialization of
  the independent GilRs context. Native PC still uses the platform-owned GilRs
  driver; Redux still owns its SDL output.
- Host effect ownership now includes the logical device identity, not just the
  reusable GilRs device number. A disconnect/reconnect or reassignment drops old
  effects and error state even if the OS reuses the same device number.
- A PSX one-frame actuator lease is consumed after its first legal poll instead
  of before it. Leases still age while Memory Card/PocketStation owns SIO0.
- Controls reports synchronous backend output failures, provides explicit retry
  through Test Rumble, distinguishes no assignment, and ends a test when the
  selected pad changes. Both Settings searches include rumble/vibration terms.
- Compiling the manual example exposed an existing Windows Lua VM build-recipe
  failure: cmd.exe cannot execute the Unix `mkdir -p` spelling. `runtime/lua.mk`
  now creates its generated directories through PowerShell on Windows and keeps
  the existing Unix recipe elsewhere. The embedded and exported makefile are
  the same source.

`examples/controller-rumble/Controller Rumble.epokproject` is the isolated manual
handoff project. It uses no external art/audio assets and provides separate
large/small/both-motor pulses, four output destinations, explicit stop, per-pad
enable, gameplay pause and two scene banks. HUD labels show the selected pad,
runtime state and request acceptance. Its README describes host assignment,
Redux's separate binding, cancellation, simultaneous-slot checks and hardware
limits. It never initiates output without a user motor-button press.

The editor and reflection extractor are built in the canonical Cargo Debug
directory with locked dependencies. The API reference has been regenerated with
130 Epok headers / 2195 callables, 53 PsyQo headers / 945 callables, and no
extraction diagnostics. Development compilation is not functional acceptance;
no controller effect, emulator or game process has been run for this handoff.
Native PC macOS remains outside the agreed first-release backend scope.

The manual project's PSX executable was compiled and linked successfully for
all three Lua modes, without executing it. Reported static RAM / executable
payload: Native C++ 1.03 MiB / 488 KiB; VM Bytecode 1.33 MiB / 566 KiB; VM Source
1.39 MiB / 594 KiB. Each build passed the existing memory-budget gate. Logs are
kept locally under the example's `.epok/rumble-build-<mode>.log`. The descriptor
was restored to Native C++ and that mode was compiled last, leaving its PS-EXE
in the example's canonical `.epok/build/` directory. These are build outputs,
not physical motor, gameplay, regression or performance acceptance results.

## First physical-console report (2026-10-26)

The follow-up also requests visible runtime failure reporting. Console now keeps
a red error message above its selectable transcript, with an `ERROR:` log entry
and MCP `console_error`/`runtime_error` state. The serial worker parses complete
PsyQo abort/Lua error lines and the pinned nops halt marker across PTY chunks.
An unexpected resident halt keeps the session connected and enables recovery
controls even when startup did not reach `runtime ready`. The nops monitor itself
attempts the register dump; Epok identifies its exception description without
sending extra commands. Requested pauses suppress halt/exception alerts because
the dumped Cause register can be stale. Unsupported commands, timeouts, transport
and build/Play failures are reported separately. No silent-game watchdog or
automatic reset/resume is introduced. Physical confirmation remains pending.

The user ran a Sony DualShock connected directly to physical socket 1 through
NOTPSXSerial on COM12. The game appeared frozen; the photograph showed State 0
and Request 2. Editor MCP reported an active serial session, a responsive editor,
no editor/game transport error and no received emulator/native frame (expected
for this target). Console ended with `runtime ready`, `startup ready` and the
resident serial handshake, without an exception report. This does not prove the
render loop was still progressing: the original HUD had no heartbeat.

Source review found that the new transport sent only five bytes for a digital
reply even when sending configuration entry. The SCPH-1200 exception is documented
in [PsxNewLib's command definitions](https://github.com/SukkoPera/PsxNewLib/blob/master/src/PsxNewLib.h):
short configuration packets do not work on that Sony model. Configuration
transactions now clock all nine command bytes while expecting ACK only inside
the advertised response. Ordinary reads keep their response-derived length.

The ACK wait now reads the SIO0-local interrupt latch instead of depending on
the global I_STAT latch surviving another interrupt handler. The driver still
waits for physical ACK release and retains bounded RX/ACK waits, bus ownership
checks and DTR cleanup. Ordinary digital/analog reads accept the zero high-ID
quirk already described by the pinned PsyQo loader-compatibility code;
configuration-mode headers remain strict. Register semantics are documented in
[PSX-SPX's SIO reference](https://psx-spx.consoledev.net/serialinterfacessio/).

The example adds a rendered-frame counter, and rumble-enabled serial builds
emit a bounded trace of transfer state changes after completing each transaction.
These changes provide evidence for the next physical attempt. They do not turn
the failed hardware report into a pass; confirmation on the user's DualShock
remains pending. The user's in-memory scene was backed up to
`.epok/rumble-scene-before-repair.json`, saved through MCP and preserved before
the example HUD was extended. Stopping the serial monitor only disconnected the
PC side; no console reset, pause or executable upload was issued.

Compilation after these changes succeeded: Debug editor (`cargo build --locked
--bins`, zero errors; 46 existing warnings) and the example's serial Whole Game
PSX build (500.0 KiB EXE, 1.05 MiB static RAM). No tests or gameplay were run.
The rebuilt editor was reopened with Controller Rumble. MCP confirmed Main has
19 actors, no unsaved edits, no active job/Play, COM12 selected and the new
`console_error`/`runtime_error` fields available. The hardware retry is left to
the user; the original freeze has not been claimed resolved by compilation.

### Serial recovery follow-up

The user points out that the physical console still runs the previous blocked
execution after Epok disconnects. Explicit Stop on connected serial Play now
queues `StopAndReset`: wait for the pending command/barrier, request REST, then
disconnect after acknowledgement or a bounded failure (twelve seconds total,
five seconds for REST). Internal cancellation/editor shutdown retain the plain
disconnect operation, and cancelling a build does not reset the console.
Reset PSX also works with no Play job, through a separate selected-adapter
`nops /reset` operation bounded to eight seconds. It neither installs a resident
handler nor uploads another game. Toolbar R, Game, connection settings and MCP
share the same action. A missing acknowledgement becomes a red Console error.
PING alone cannot distinguish a progressing game from a responsive resident
handler; Play still requires the loader rather than silently overwriting an
unknown execution. Remote reset cannot guarantee Unirom unless the console's
boot setup returns there. No reset/upload/game execution was sent while
implementing this follow-up.

The recovery follow-up compiles in the canonical Debug target with zero errors
and 46 existing warnings. Existing channel-only Stop expectations were updated
for the explicit serial reset control; no tests or physical recovery commands
were run. The example PSX binary is unchanged by this editor-only follow-up.

### User hardware confirmation

On 2026-10-26 the user reported that rumble now works perfectly in the example,
following the physical PSX / Sony DualShock directly on socket 1 attempt above.
This is user-provided confirmation of that setup; additional devices, multitap,
host platforms and individual lifecycle cases retain their own validation scope.
The user also confirmed that remote Reset rebooted the physical PSX, but their
FreePSXBoot/Unirom startup then used the slow loading path and stalled. They
explicitly deferred that firmware/boot-flow issue while continuing rumble checks.

### Dungeon Complete damage feedback

The bundled Dungeon Complete template now consumes the C++ Input rumble API on
accepted player damage: Pad 1, large 160, small enabled, 120 ms. The existing
controls/invulnerability gate runs first. Its Combat HUD Canvas owns an initially
disabled Image frame; a 220 ms rendered-time pulse narrows six-pixel red edges
and dims them, including during death holds/menus. Reset and end_play clear the
frame and stop the pad pulse. No feedback is retriggered by immune/invalid hits.

To preserve the full 64-actor level, an untextured Image with nonzero borders now
emits a hollow frame through the shared HUD compiler (four bounded rectangles,
clipping/rotation/tint retained); zero borders keep the usual filled panel.
The center is never drawn by this frame. HUD budget estimation, Inspector labels
and documentation match the new case. No textures, new actors or schema/ABI
fields were added. A focused HUD contract case was added but not executed,
following the user's preference to run tests themselves.

Compilation succeeded for the Debug editor (zero errors, 46 existing warnings)
and a fresh template copy at `.epok/dungeon-damage-feedback`, using Serial / PC
on demand: 1.85 MiB static RAM, 1.07 MiB serial EXE and 256 KiB external assets.
Compiler output confirms `EPOK_RUMBLE`. The bundled template retains its default
Embedded profile. API reference regeneration still reports 130 Epok headers /
2196 callables and 53 PsyQo headers / 945 callables. No gameplay, emulator,
motor test, serial reset or upload was executed during preparation.
