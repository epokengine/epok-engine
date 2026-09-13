# Native HUD preview

Open **Scene > 2D**. Authored Canvas components are visible immediately. For
procedural UI, the editor automatically compiles a desktop preview and calls
the controller's `editor_preview(epok::Transform&)` construction hook in an
isolated process. There is no required Simulate or Play action. Ironwood's Title
uses this hook to show the complete menu at frame zero, without its entry fade.
No emulator, MIPS executable, console connection, or disc build is involved.
The first run needs a native C++20 compiler (Visual Studio C++ Build Tools on
Windows). Unchanged source snapshots reuse the compiled executable.

## Two phases, and the boundary between them

The preview has exactly two phases, and which one produced a frame is reported by
the child process rather than assumed by the editor.

| | **Edit** | **Simulate** |
| --- | --- | --- |
| Entered by | the automatic preview (no action needed) | the optional **Interact** session |
| Runs | property binding and public `editor_preview(Transform&)` construction hooks | the ordinary gameplay lifecycle |
| Does **not** run | BeginPlay/`start`, `update`, `frame_update`, the game clock, input | — |

**The edit phase does not run BeginPlay; the simulation does.** The child fixes
its phase at startup and reports it in every frame header, so a frame whose phase
disagrees with the session that asked for it is rejected instead of displayed.

The frame header also carries a protocol version (`HUD_PREVIEW_PROTOCOL_VERSION`,
mirrored in `native/hud_preview.h` and `src/hud_native.rs`) and a capability list.
An executable cached from an older header is rejected with a message asking for a
restart, rather than decoded with the wrong field layout. Bump the version on any
change to the frame header or command record.

Blueprint support is reported as **false**: this preview links C++ controllers
only. A scene driven by a Blueprint class is refused with "Blueprint logic is not
simulated in the native preview", naming the entity — it is never rendered as if
the Blueprint had run. Use Play or the console build for Blueprint classes.

The edit view does not run `start`, `update`, `frame_update`, or lifecycle
notifications. Scene/property edits and imported resource/script changes rebuild
the preview automatically after a brief debounce. Failures appear in the view
and do not retry continuously. Authored Canvas nodes remain editable; nodes
created by a procedural hook are a derived preview and are edited in their
construction code, not saved back as scene entities.

**Interact** is optional, for testing navigation and animation. In that mode,
click the preview to capture input: arrows navigate, K confirms, L goes back,
and Enter is Start. **Pause**, **Resume**, **Step**, and **Restart** control the
session; **Back to editing** restores the automatic edit preview. Step advances
one frame at exactly 60 Hz using an integer
microsecond clock. Restart recompiles changed inputs and resets the process,
including script globals. Editing while
simulating changes the authored document; Restart applies those edits to a new
simulation. Generated entities are temporary and are never saved into the map.

## Shared implementation

`runtime/hud_core.hpp` owns Q12 layout, pixel rounding, hierarchy traversal,
clipping, atlas bounds, nine-slice subdivision, text placement, and HUD budgets.
The PSX renderer consumes that compiler's primitives and retains its packet
cache. Both static editor rendering and native simulation use the same compiler.
The editor's drag layout also calls the shared Q12 resolver.

Both modes use the ordinary generated scene/component/property
initializer, attached C++ source files and their base classes, `lifecycle.hpp`,
`input.hpp`, `time.hpp`, and `memory_card.hpp`. Interactive scripts run `start`,
`frame_update`, and `update`. Edit mode binds properties and invokes only public
`editor_preview(Transform&)` hooks, detected by C++20 compile-time dispatch.
Hooks may be enclosed in `#ifdef EPOK_EDITOR_PREVIEW`; they introduce no console
vtable entry or persisted entity templates. Put UI construction and styling in
shared helpers used by both the hook and gameplay. A script without a hook does
not execute in edit mode; serialized Canvas components still render normally.
Entity storage has the same authored count plus
32 dynamic slots as a console scene. A versioned, bounded binary protocol sends
primitive commands to the editor; script stdout is redirected to a session log.
Frames that fail or stall terminate the child process without running scripts
inside the editor. Compilation is asynchronous and cancellable.

Textures come from the imported PSX assets. The desktop compositor uses nearest
sampling and integer texture tinting; it is not a cycle-accurate GPU emulator.
Exact GPU subpixel rasterization, display fields, and performance on hardware
are outside this preview's guarantees. Shared commands and layout do not by
themselves establish identical framebuffer pixels on every PSX edge case.

## Current scope

This preview covers Canvas/HUD scenes with portable native C++ controllers,
including procedural UI such as Ironwood's Title. Timeline and Particle Effect
components retain their dedicated previews; Blueprint and full 3D gameplay
simulation are not part of this mode. A source file that uses unsupported
hardware services reports its compiler/linker error, rather than silently
substituting gameplay behavior. Attached class implementations are selected by
their header's matching `.cpp` path, including known base classes.

Audio playback state is simulated silently. Memory cards use asynchronous,
process-local storage and disappear on restart. A requested scene transition
stops at the scene boundary and displays the requested destination. Loading the
next scene, real console timing, and physical memory card/audio behavior still
require a target run.

The compiler cache and diagnostics live under `.epok/native-preview/`. They are
derived artifacts. The project's scene and Play profile are not modified by
simulation.

## Reproducible captures

```powershell
rtk proxy target/debug/epok-editor.exe --project ../EpokDemos/Ironwood --preview-hud assets/scenes/Title.epokmap --frames 120 --output title-native.png
```

The command prints the final frame's primitive trace and writes a PNG. The
editor integration can also be captured with `--screenshot-native-hud
--screenshot editor-native.png`; this selects 2D and waits for the automatic edit
preview's first frame. It does not start a simulation.

The CLI takes the usual project lock, so close another editor using that
project or capture a disposable copy. Automatic previews and optional interaction
run inside the existing editor project session.

## Verification on 2026-09-12

The Ironwood integration check (`tests/integration/verify_native_hud.py`) runs
the actual Title controller: 36 entities (28 created dynamically), 59 drawing
commands at frame 120, and zero dropped primitives. It checks deterministic
restart, Configuration and volume, empty temporary memory cards, held-button
debouncing, the ForestClearing transition boundary, and unchanged scene bytes.
The HUD tests also exercise native property overrides, lifecycle, Q12 layout,
clipping/nine-slice budgets, pause and queued frame steps. Edit-mode checks verify
automatic construction at frame zero, no gameplay lifecycle, property refresh,
inactive controllers and Title command equality with its completed runtime intro.

Title builds successfully for PSX using Current scene. A Whole game build of
the current Ironwood data exceeds the RAM region by 4,592 bytes. Recompiling the
same data with the previous HUD renderer reproduces that overflow; it is not
resolved by this preview feature.

The full Rust suite run produced 345 passes, 31 ignored tests, and one failure
in the existing Console/Project docking interaction test. The pre-change test
executable reproduces the same failure (`User can select Console`). The native
HUD checks pass independently. Run ImGui tests with `--test-threads=1` because
their global contexts cannot run concurrently.
