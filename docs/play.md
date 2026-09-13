# Play targets and loading

The arrow beside Play opens one menu with checkmarked choices for destination,
content and data source. Each section has one active choice, saved immediately
in `.epokproject`. Both **Build** and **Play** use those saved choices.
New and older projects default to **Embedded emulator**, **Current scene**,
**In executable**. The same fields are editable under **Project Settings >
Maps & Build**, where Apply saves the draft.

| Selector | Choices |
| --- | --- |
| Destination | Embedded PCSX-Redux Game view; separate emulator window; PSX through NOTPSXSerial |
| Scenes submenu | Current scene; Whole game; Selected scenes with inclusion checkboxes and an initial-scene radio button |
| Data | In executable; CD on demand; PC on demand through PCDrv |

Whole game includes every saved map in `assets/scenes`, starting with the project
startup scene. Registered maps retain their order, followed by other maps in path
order. Current scene includes only the open map and its resource dependencies.
Selected scenes includes only checked maps; its initial scene is bank zero.
Clicking the Selected scenes parent row selects the mode and opens its checklist;
hovering opens the checklist without changing mode. The submenu also contains an
explicit radio choice for keyboard navigation. Unchecking the initial scene picks
the next included map, wrapping to the first. An empty selection can be saved, but
Build and Play show a warning until at least one map is included. Missing selected
files cause an error rather than silently changing the game. Builds support up to
16 scene banks. In every mode, an included open map contributes its unsaved
snapshot. Requests for excluded scenes return false.

**Compilation is manual.** Scene/Inspector edits update Scene View immediately;
external changes refresh editor resources and mark the build stale. Returning to
the editor never launches the game compiler, including in legacy projects with
`auto_build: true`. Build prepares the chosen configuration without launching it.
Play validates that output, builds pending changes if needed, then launches the
configured destination. Changing scene scope, inclusion or initial scene
invalidates cached results for the next Build/Play; unchanged native objects and
SDK components may still be reused by the incremental compiler. A running game
keeps its launched snapshot while sources are edited. Changes during an unfinished
build cancel that build and require another manual Build/Play.

**Content selection is separate from memory residency.** Scene definitions,
scripts, textures, collision and resident SFX remain prelinked. Whole game uses
one reusable pool of entity instances, but its prelinked data must fit PSX RAM.
Arbitrary scene packages and texture/audio streaming from PC are not implemented.

CD and PC on demand select the source of external **editable-mesh geometry**.
Enable **Engine > Streaming > Geometry Streaming** to generate `GEOMETRY.BIN`
with checked 64 KiB pages and a bounded RAM pool. CD uses the ISO reader; PC uses
PCDrv in the active build directory. PCSX-Redux starts with that directory as its
PCDrv base. PC reads are synchronous and can stall frames, especially over serial.
Ordinary demand reads do not trigger full-screen transitions. See [streaming](streaming.md).

In executable rejects external geometry. Serial builds omit XA music with a
warning, both In executable and PC on demand: its playback calls do nothing,
`is_playing()` remains false, and resident SPU sound effects still work. Omitted
clips are listed in the asset report warnings and excluded from payload sizes.
Other destinations require the CD profile for XA's physical CD decoder. Choosing
CD without external data still runs the EXE without creating an empty data disc.

## Repeated Play and embedded startup

Stop releases Game input and returns focus to Scene. Once Play has stopped,
other tabs remain freely selectable. On Windows, both Debug and Release editor
builds open without a separate console window; pipeline messages appear in the
editor's Console panel.

After a successful build, `.epok/build/PlayBuild.epokcache` records its certified
inputs and the executable/disc hashes. An unchanged Play validates native include
dependencies, tools, scene/assets, settings, staged files and output bytes, then
reuses the executable and disc without invoking the game compilation or disc
generator. A changed, missing or invalid receipt falls back to a normal build.
Validation still takes some time; the receipt is not permission to run stale code.

Embedded Play runs PCSX-Redux without its GUI, inside a private terminal, and
loads the executable directly with fast boot. The CD stays mounted for external
assets. The separate-window destination still opens the emulator GUI. Runtime
messages are read from `.epok/emulator/runtime.log` into the timestamped Console.
The Windows private terminal requires Windows 10 1809 or newer.

## Physical PSX

Choose **PSX via serial**, then **PSX connection...** in the Play options menu
(also available in **Editor Preferences > Play**). If components are missing or
damaged, Play opens the connection window with **Download components** and
**Download and Play on PSX**. Installation works with the console off.

Both **In executable** and **PC on demand** start a development session: Epok
enables Unirom's resident handler with `/debug`, then sends the program with
`/exe <program> /dest <port> /m` and keeps the monitor connected. PC on demand
also serves PCDrv data from the build directory. The connection window's optional
loader ping remains diagnostic; Play checks port access before compiling and
requires a fresh resident-handler reply after the game starts.
Epok does not install or flash Unirom.

The adapter list shows available USB names and ports. A unique USB adapter can be
selected automatically. Multiple adapters require a choice; Epok never probes all
serial devices. USB serial numbers allow a remembered adapter to follow a COM/device
path change. A missing remembered device is not silently replaced. Without a USB
serial number, the remembered port is the fallback and may need reselection.

The three selectors travel with the project. Adapter identity and speed stay in
the user's `Editor.epokprefs`. Epok always launches its own verified nops copy.
Old custom executable paths are used only as optional import sources: matching
files are copied into this installation, and the saved path is cleared after
successful preparation. Standard speed is the default;
fast mode also depends on the adapter and Unirom.

| Host | Serial requirements |
| --- | --- |
| Windows x86/x64 | Windows 10 1809+ for the private ConPTY terminal; .NET Framework 4.7.2+; accessible COM port |
| macOS | Compatible Mono runtime and accessible serial adapter; the editor discovers Homebrew/Mono outside PATH and can install Mono through existing Homebrew. The project setup currently targets Apple Silicon. |
| Linux | Experimental: Linux x86_64 setup can download editor packages and build the local MIPS compiler; compatible Mono, PTY access and serial-device permissions are still supplied by the distribution. No automatic sudo, group or driver changes. |

The launcher tests actual runtime/terminal capability before opening the serial
port. Other architectures report missing automatic support and preserve the
selected project destination. macOS uses the `/dev/cu.*` callout devices to avoid
duplicate `/dev/tty.*` entries. Windows errors distinguish a missing runtime,
unavailable terminal and inaccessible port from a console that does not respond.
macOS/Linux hardware paths still need validation on those hosts.

Managed NOTPSXSerial files are pinned in `tools/serial-dependency.json`, verified
with SHA-256 and published together under
`<editor executable directory>/tools/notpsxserial/<revision>/`. This also applies
to debug builds, which keep their components under `target/debug/tools/`.
Failed or
cancelled downloads do not replace an installed package. Repair keeps the previous
package recoverable. An installation lock prevents competing editor installers.
The payload includes licenses and the matching source URL. Distributions can ship
`.tools/serial-bundle` for offline installation (`tools/bundle-serial.py`, Windows
setup `-Dependency serial`, and macOS setup prepare that payload); source builds
offer to download it on first use. The installation directory must be writable;
protected system folders and read-only application images report an actionable
installation error. Framework/Mono installation may still
require OS-specific setup. No project or global PATH changes are needed for nops.

`epok-editor --prepare-serial-tools` verifies/installs the automatic tools without
contacting the console; `--list-serial-ports` only enumerates adapters. Neither
requires a project. The nops terminal remains inside Epok so its console API and
monitor work without an extra visible window.

Serial disables CD on demand. Switching to Serial while CD is selected changes
the source to PC on demand and saves that selection. The monitor serves PCDrv
from `.epok/build` and displays upload/TTY output in Console. Epok waits for its
runtime startup message and `Send finished!`, then sends `PING` through that
same monitor and requires `PONG` before enabling session controls. A successful
upload alone is not reported as a connected game. The port stays owned by the
session until Stop, an acknowledged reset or a transport failure.

Serial builds preserve kernel-resident interrupt queue entries below the
executable load boundary and restore the loader's SIO1 interrupt mask after
PSYQo initializes. Obsolete loader/game callbacks are not restored. This uses
the handler already installed by Unirom; no Unirom firmware is linked or flashed.
The runtime handshake detects a handler that no longer responds after startup.

**Pause / Resume** in the toolbar and **Pause PSX**, **Continue PSX**, **Reset PSX**
in Game send `HALT`, `CONT`, and `REST` through the owned monitor. The toolbar's
**R** button also resets the console. Commands run one at a time, and pause state
changes only after a handler acknowledgement. Unsupported commands or a five-second
reply timeout are reported in Console. After a timeout, a fresh `PING`/`PONG`
resynchronizes the session before Continue or Reset can be retried; if that fails,
Epok disconnects with an error. A reset acknowledgement ends the session.
A reboot without an acknowledgement remains unconfirmed; Stop releases that
session. Reset does not automatically reload the game or guarantee a return to
Unirom: wait for the loader before the next Play. MCP exposes these controls as
`editor_control` actions `pause`, `resume`, and `reset_psx`, with
`serial.command_pending` in `editor_state`.

Video and controller input stay on the console. Frame stepping and Blueprint
debugging are emulator features. Stop terminates only this session's nops process;
it does not halt, resume or reset the PSX. PC data requires the monitor to remain connected.
A BIOS PCDrv call can remain blocked after transport disconnection; a host timeout
cannot unwind it. Hardware validation is separate from emulator/process tests.

Protocol references: [NOTPSXSerial](https://github.com/JonathanDotCel/NOTPSXSerial)
and its [PCDrv handler](https://github.com/JonathanDotCel/NOTPSXSerial/blob/master/PCDrv.cs).

## Critical operations

Component installation, build, memory analysis, connection checks and upload display
a compact, centered progress modal. The workspace stays visible and is darkened
behind the modal. Upload percentages come from NOTPSXSerial's completed progress
records; phases without a reported percentage show an activity bar. Detailed
compiler/transport output stays in Console. Editing panels, shortcuts, external file drops, project switching,
settings writes and application-close requests are blocked until the operation
finishes. MCP can read status/logs and request Stop but cannot edit project inputs.
External filesystem edits remain subject to the existing build invalidation checks.

Build and serial preparation offer cancellation and keep the editor locked until
the worker has stopped. Downloads cancel between reads/files (a network request
has a 30-second deadline). The existing dependency installer must finish before
the editor can close because it repairs package files in place. Errors restore
the editor and show the failure; unsaved documents are retained. Play's normal
runtime controls become available once the emulator or PSX is running.

## Memory Analyzer

Build and Play generate an asset/memory report by default. Disable **Generate
Asset Report** under **Project Settings > Maps & Build > Build Reports** to skip
detailed analysis; file sizes are always recorded. The pie-chart button beside
Play opens the last report. If no verified compilation exists, an OK/Cancel modal
offers to build first, shows build progress, then opens the report. If a compiled
snapshot has no report (including when automatic generation was disabled), the
modal offers to analyze those existing files without compiling or launching.
An existing report opens directly, with a warning when source/profile changes
make it a report of a previous build. Generating from old compiled files never
pretends that pending edits were compiled. Changed or missing analysis inputs
require a new build.

The status bar shows the verified EXE size and, for external-data profiles, PC or
CD payload bytes. Serial labels its EXE as the upload payload. In-executable
assets are already part of the EXE and are not counted twice. CD builds also show
the actual disc image size (including container overhead). Sizes disappear when
inputs become stale or any Build/Play request begins; a successful build restores
them. Detailed reports and `build-summary.json` persist beside compiled outputs.
The report button never connects to an emulator or a physical PSX.

The modal includes a capacity bar, clickable treemap, allocation table and
breadcrumbs. Drill into a category/resource to inspect its linked symbols, byte
counts and scene references; **Show asset in Project** locates supported source
assets. **Build and update report** uses the current profile. Source/profile changes mark
an open report out of date. A failed or cancelled analysis does not present a
previous build as a current result; compiler details remain in Console.

* **Main RAM (2 MiB):** linked allocations, including BSS/NOLOAD pools, link
  alignment and the region below the executable load address. Shared assets and
  reusable pools are counted once. Unassigned memory is the remaining budget for
  heap/stack; their runtime and transition peaks are **not measured**.
* **VRAM (1 MiB):** allocations for each active scene bank, with texture pixels,
  palettes, framebuffers and reserved font/loading/display regions. The optional
  physical layout shows the actual 1024 x 512 word address space. Unallocated
  bytes do not guarantee that another texture fits the allocator's placement rules.
* **Audio / SPU (512 KiB):** selected resident ADPCM samples including DMA padding,
  plus the runtime's 4 KiB reserve. XA files appear under Files.
* **Files:** the PS-X EXE (including its header/padding) and currently selected
  external geometry/XA payloads. Cached but unselected files are excluded. Disc
  filesystem/sector overhead is not included.
* **Scratchpad (1 KiB):** separately linked scratchpad allocations.

The report is written to `.epok/build/memory-report.json` (or
`.epok/build-blueprint-debug/memory-report.json` for Blueprint debug). Human-readable
asset attribution is captured during staging and matched to the final linked
symbols. Unattributed bytes remain visible; they are not silently omitted. PCDrv's
external Unirom handler reservation is not measured by the game ELF and is called
out in the report. A successful static report is not proof of a safe runtime peak.

Headless `--analyze-memory` honors the saved Play profile and uses the startup
scene as the current scene, without launching or uploading. Add
`--blueprint-debug` to measure the instrumented build.
`--generate-asset-report` analyzes the last verified compilation without rebuilding.

Console's **Auto-scroll** is enabled by default. Scrolling upward disables it;
scrolling back to the bottom re-enables it. New messages follow the bottom only
when enabled. Build and other operations that lock editing force it on, while
preserving read-only text selection and copying.

## Scene transitions

`epok::request_scene("Night")` uses project defaults:

1. Fade picture and music/SFX out together over 300 ms.
2. Present a black loading frame with `Now loading...` at the lower right.
3. Stop outgoing audio, wait for pending CD callbacks, retire the scene and
   initialize the incoming scene. Warm its geometry when the working set fits.
4. Fade picture and incoming audio in over 300 ms, then resume updates.

Initial boot starts at the loading frame. Synchronizing the simulation clock
prevents load time becoming catch-up ticks. Transition gain multiplies authored
AudioSource volumes without overwriting them. The loading renderer and VRAM
allocation survive scene teardown. Geometry read errors keep the screen black
and muted and offer START to retry once pending I/O has released.

Configure durations, text and an optional imported image under **Maps & Build >
Scene Transitions**. Text supports 95 printable ASCII characters and wraps inside
a TV-safe margin. Images are resident RGB555, at most 64 × 64 pixels with even
width, stored at VRAM `(960,384)` above the font at `(960,448)`. The image appears
above the text; leave text empty to show only an image.

Override a transition in C++:

```cpp
epok::TransitionOptions options;
options.fade_out_ms = 400;
options.fade_in_ms = 600;
options.loading.text = "Entering forest...";
options.loading.color[0] = 160;
epok::request_scene("Forest", options);
```

For a custom image, assign `options.loading.image` to a resident
`epok::LoadingImage{pixels, width, height}`. Descriptor and RGB555 pixels must
outlive the transition; static storage is suitable. Text is copied at request
time. A null image uses the project default; an empty static descriptor suppresses
it. Lifecycle callback requests queue behind the current fade.

Headless `--build-psx --use-play-profile` and `--play-psx --use-play-profile` honor
the saved profile, using startup as the current scene. Without that flag,
headless tools preserve the existing full-bank build. **Package Disc** builds
the whole game from startup with CD data, independently of the Play destination.
