# Controller rumble: manual check

Open `Controller Rumble.epokproject` in the Debug editor. This project has no
external art or audio dependencies. Its two maps use built-in HUD text to show
the output slot, service status and last request result. Nothing vibrates until
you explicitly press a motor button.

For **Native PC**, assign devices to Pad 1..4 in **Project Settings > Controls**.
Enable **Editor Preferences > Play > Rumble** and apply the preference. The
Controls page's **Test Rumble (150 ms)** checks the selected device while Play is
stopped; it does not exercise gameplay, the PSX serial driver or the emulator.

For **PlayStation / PCSX-Redux**, use the bundled dependency installed by
Install / Repair, select a physical controller in Redux's own controller
settings, and enable its controller connection. Native PC's device assignments
do not configure Redux. A console uses its own controller directly.

Start Play and click **Game** to capture desktop input. Pad 1 provides all demo
controls, through its gamepad or keyboard mappings. SELECT changes the motor
destination independently, so one player can check the other three assignments.
Use Controls to see or change the current keyboard bindings.
The Play profile uses **Whole Game** so that both scene banks are present. If
changed to **Current Scene**, R2 cannot switch to the map excluded from that build.

| Pad 1 control | Action on the selected output pad |
| --- | --- |
| Cross | Large motor only, intensity 192, 250 ms |
| Circle | Small motor only, 250 ms |
| Square | Both motors, 150 ms |
| Triangle | Both motors, two seconds |
| L1 | Stop immediately at the next legal output update |
| R1 | Toggle the game's per-pad enable setting |
| SELECT | Cycle the motor destination through Pad 1..4 |
| START | Toggle gameplay pause; resume requires a fresh motor request |
| R2 | Change between scenes A and B; an old pulse must not continue |

The state number is 0 disconnected input, 1 unsupported output, 2 probing,
3 disabled, 4 ready, 5 active or 6 paused. Request is 0 no request, 1 accepted or
2 rejected. Acceptance and the active state describe the runtime request, not
proof that a physical motor moved. Redux's capability describes its emulated
DualShock protocol, not whether a physical gamepad is bound.

**Frames** increments on every rendered-frame callback, including during a
gameplay pause. If it keeps changing while State is 0, the game is still running
but the controller input connection was lost. If Frames also stops, report that
separately from the rumble request result.

Serial builds also emit at most sixteen `EPOK PAD1` transfer state changes in the
editor Console. `cmd` is the command byte, `byte` its zero-based transfer index,
`id` the reply ID/high byte and `sio` the SIO0 status. `result` is 0 success,
1 shared bus occupied, 2 receive timeout, 3 ACK timeout, 4 missing device ID,
5 oversized reply or 6 invalid reply header. These records are emitted after
the transaction, outside the timing-sensitive byte loop.

If Unirom reports an unexpected halt or exception, Epok opens Console with a red
error message and preserves the monitor connection for Continue PSX / Reset PSX.
PsyQo aborts and Lua errors also receive red diagnostics. Copy the Console output
or read it through MCP after a failure. A silent freeze may prevent the resident
handler from reporting anything; silence alone is not labeled a CPU exception.

For a previous physical-console execution that remains blocked after the editor
session ended, use **Reset PSX** (toolbar **R**, Game, or PSX connection) before
Play. It can reconnect without uploading a game. During connected serial Play,
Stop also requests a reset before disconnecting. Wait for Unirom before the next
Play; if the resident handler no longer responds, use the console's physical
reset. Remote reset follows the console's boot setup and cannot guarantee which
loader that setup starts.

Start with each motor separately, then both. During a long pulse, check L1,
pause, Stop, Reset, releasing Game capture, disabling Rumble, device removal and
reassignment. Each must cancel the pulse; restoring a device, focus or an enable
setting must not replay it. After a cancellation, issue a new request. Change
scenes during a long pulse and verify the same rule. Check each assigned Native
PC pad independently; identical controller names include separate device IDs.
To check independence, start a two-second pulse, select another pad and start
another pulse. Stopping the second pad must leave the first pulse's own deadline
intact. SELECT itself does not cancel any pulse; START pauses and cancels all.

Physical PSX Pad 1..4 map to the first console socket's direct controller or
multitap A..D. Pad 2 is not the console's second physical socket. Four-controller
PSX acceptance requires an actual multitap; do not infer it from Redux's two-pad
emulation. Check input and Memory Card/PocketStation traffic alongside motors.
Native PC macOS remains unsupported by the locked GilRs backend; Windows/Linux
require a compatible device and OS driver. Host effects have 50 ms timing steps.

The project uses the reflected Input/Time/Scene APIs in Gameplay profile 2.
You can change **Project Settings > Description > Lua Execution** to Native C++,
VM Bytecode or VM Source to exercise the same authored requests through each
Lua backend. C++ and Blueprint expose the same five Input operations; see
[Controller rumble](../../docs/input-collision.md#controller-rumble).

Compilation is separate from acceptance. No controller, emulator or game has
been run as part of preparing this example; physical checks are left to the user.
