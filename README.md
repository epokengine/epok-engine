<p align="center">
  <img src="resources/branding/epok-lockup.png" alt="Epok Engine — geometric E emblem" width="640" />
</p>

<h1 align="center">Epok Engine</h1>

<p align="center">
  <strong>A desktop game engine for the original PlayStation.</strong><br />
  Build worlds, animate characters and create gameplay with Blueprints or C++.
</p>

<p align="center">
  Rust + Dear ImGui &nbsp;·&nbsp; C++20 + PsyQo &nbsp;·&nbsp; PS-X EXE + BIN/CUE
</p>

<p align="center">
  <a href="#getting-started">Get started</a> &nbsp;·&nbsp;
  <a href="#features">Features</a> &nbsp;·&nbsp;
  <a href="docs/features.md">Complete catalog</a> &nbsp;·&nbsp;
  <a href="#blueprints">Blueprints</a> &nbsp;·&nbsp;
  <a href="#examples">Examples</a> &nbsp;·&nbsp;
  <a href="#external-tools-and-automation">External tools</a> &nbsp;·&nbsp;
  <a href="docs/getting-started.md">Documentation</a>
</p>

---

Epok brings a visual editor workflow to PSX development. Create a project, arrange a scene, import assets, connect Blueprint graphs or write C++ behaviours, and press **Play**. Your game compiles to a native MIPS executable and runs inside an integrated PCSX-Redux Game view.

**Windows x64, macOS Apple Silicon and Linux x86_64 · Experimental · MIT-licensed original code.** The editor and runtime are in active development. Emulator validation is available; physical-console validation is pending.

![Epok editor with the scene viewport, editable 2.5D courtyard, scene hierarchy and component inspector](docs/images/epok-editor.png)

<p align="center"><sub>The included 2.5D courtyard, open in the real editor. Screenshots in this README show the working application.</sub></p>

## Features

For the exhaustive, source-mapped inventory of implemented editor, asset,
gameplay, runtime, build, Play, serial, analysis and automation capabilities, see
the **[complete feature catalog](docs/features.md)**.

![Epok Engine project Hub](docs/images/epok-hub.png)

| | What you can do |
| --- | --- |
| **Build your world** | Arrange entities in a dockable editor with transform gizmos, parenting and a component Inspector. Create and edit geometry with [Blockout](docs/blockout.md), including extrusion, bevels and geometry Undo/Redo. Sculpt and paint ground with [Terrain](docs/terrain.md). |
| **Bring in your assets** | Import [PNG textures](docs/textures.md), [OBJ/MTL models](docs/static-mesh-import.md) and [FBX characters](docs/skeletal.md). Asset identities survive moves and reimports. |
| **Animate and add effects** | Build layered effects in the [VFX editor](docs/vfx-editor.md) with presets, Q12 curves, bursts and preview controls. Reuse [timelines](docs/timelines.md) for scene sequences and connect their markers to Blueprint gameplay. Use sprites, flipbooks, [particle emitters](docs/sprites-particles.md), fog and palette cycling. |
| **Model your game objects** | Compose maps from [actors and components](docs/actors.md): `Actor3D`, `Actor2D` and `UIActor` classes written in C++ or as Blueprints, with typed components, per-map scene Blueprints and a 3D / 2D / UI authoring mode. Maps use one Actor collection. Older projects must be recreated; see [project recreation](docs/migration-actors.md). |
| **Write native gameplay** | Author C++20 behaviours with Inspector properties. Connect [input and collision](docs/input-collision.md), [scene transitions and object lifecycle](docs/runtime-services.md), cameras, tweens and [Memory Card storage](docs/memory-card.md). |
| **Navigate NPCs** | Bake a bounded [NavLite](docs/navigation.md) walking graph from static box colliders, then reuse `NavigationAgentComponent` with a shared PSX search budget. Version 1 supports flat surfaces and obstacles. |
| **Build visual gameplay** | Create [Blueprint classes](docs/blueprints.md) with native or visual parents, inherited defaults, typed graph nodes, functions, event overrides and Call Parent. Author entity templates, spawn classes and debug node execution. Graphs compile ahead of time to native C++; no graph VM runs on the PSX. |
| **Script gameplay in Lua** | Author classes in [Lua](docs/lua-scripting.md) that subclass reflected C++ or other Lua classes, with Inspector properties, event overrides, typed gameplay-service/component calls and explicit parent calls. One project-wide setting compiles the same scripts to native C++ or runs them through a Lua VM linked into the game. New projects use bounded, statically typed Gameplay profile 2; older manifests retain the scalar Legacy v1 profile. Neither profile is general Lua compatibility. |
| **Light and render** | Combine baked vertex lighting, bounded realtime GTE lighting, static shadows and blob shadows. Author for PSX rendering limits with native [performance counters](docs/performance.md). |
| **Add audio and UI** | Import WAV, MP3, FLAC or OGG for SPU sound effects and XA music. Build [HUDs](docs/hud.md) with text, atlases, nine-slice images, progress bars and navigation. |
| **Play and export** | Choose [embedded/windowed PCSX-Redux or PSX through NOTPSXSerial](docs/play.md), the current scene or whole game, and resident/CD/PC geometry. Customize loading transitions with synchronized picture/audio fades. Build PS-X executables, package physical-disc BIN/CUE or ISO images with a user-supplied system-area license, or [export a standalone PsyQo project](runtime/README.md). |

Projects live independently of the editor. Scenes, scripts and imported assets stay in your game folder; the engine owns the runtime and development tools.

## Blueprints

**Visual authoring. Native PSX execution.** Build reusable gameplay classes in a
node editor with Components, My Blueprint and a contextual Details panel.
Reflected C++ methods and Blueprint functions share typed pins; child classes
inherit defaults and behavior, override events and explicitly Call Parent.

![Epok Blueprint editor with component hierarchy, inherited variables, typed graph connections and contextual Details](docs/images/blueprint-editor.png)

<p align="center"><sub>A compiled interaction Blueprint in the real Windows editor. Original editable nodes and controls, not a mockup.</sub></p>

Blueprints include bounded Delay and Timeline nodes, checked entity/asset/class
references, linked component templates, dynamic class spawning and cross-instance
calls. Instrumented Play supports node breakpoints, stepping and typed values;
release builds omit debugger instrumentation. Exported C++ projects rebuild
without the editor or reflection extractor.

Start with [Your first Blueprint](docs/blueprints-tutorial.md), then author the
effect it drives with the [VFX editor](docs/vfx-editor.md). The [Blueprint reference](docs/blueprints.md)
documents execution and console-side limits. Blueprints use Epok's own asset format and
bounded native backend; foreign assets/APIs and live native-code patching are not
supported. Blueprint reflection/authoring is available on Windows x64 and Linux
x86_64; general macOS editor support does not yet include that runtime.

## Getting started

Clone the Epok source repository, then run the commands below.

### Prerequisites

- **Windows x64**, **macOS 11+ on Apple Silicon**, or **Linux x86_64**
- [Git](https://git-scm.com/) and [Rust through rustup](https://rustup.rs/)
- On Windows: Visual Studio Build Tools with **Desktop development with C++** and a **Windows SDK**
- On macOS: Xcode Command Line Tools and [Homebrew](https://brew.sh/)
- On Linux: a native C/C++ compiler, Make, curl, Python 3, tar, xz and bzip2

Clone into a path without spaces, then run setup:

```powershell
git clone https://github.com/franadoriv/epok-engine.git Epok
cd Epok
powershell -ExecutionPolicy Bypass -File tools/setup.ps1
cargo run --locked
```

Setup initializes the pinned Nugget SDK and installs verified portable MIPS tools, PCSX-Redux, psxavenc and mkpsxiso under `.tools/`. Rustup selects the repository's pinned toolchain. Setup does not change your global PATH.

On macOS Apple Silicon, run:

```sh
xcode-select --install
./tools/setup-macos.sh
make run
```

The setup script installs Rust and the upstream MIPS toolchain with Homebrew, builds pinned host audio/disc utilities under `.tools/macos/`, downloads PCSX-Redux into `.tools/macos/redux/`, and writes an untracked `Local.epokconfig`. Start Epok with `make run` or `./tools/run-macos.sh`; neither command requires changing your shell PATH.

On Linux x86_64, run:

```sh
./tools/setup-linux.sh
make run
```

Linux setup initializes Nugget, builds the pinned MIPS compiler under
`.tools/linux/`, downloads verified PCSX-Redux, psxavenc, mkpsxiso and libclang
packages, and writes `Local.epokconfig`. It does not require `sudo` or change the
system `PATH`; the local compiler build can take several minutes.

### Your first game

1. Create a **Sample** project in the Hub.
2. Select **Cube** and edit its **Spinner** component.
3. Press **Play** to compile and run the game.
4. Use **Pause**, **Step** and **Stop** to inspect it. Save scene changes with **Ctrl+S**.

Choose the **Third Person** template for an optimized editable arena and an animated player with camera-relative movement, jumping, collision and an orbit camera. New Project also asks how its starter gameplay should be written — C++, Blueprint or Lua — and generates the same scene either way; the choice never locks the project to one system.

See [Getting started](docs/getting-started.md) for configuration and troubleshooting.

<details>
<summary><strong>Common development commands</strong></summary>

| Command | Purpose |
| --- | --- |
| `cargo run --locked` | Open the project Hub |
| `cargo run --locked -- --project examples/sample-game` | Open the included sample project |
| `cargo build --locked --release` | Build the optimized editor |
| `make app` (macOS) | Build `Epok Engine.app` and optionally replace its Desktop copy |
| `cargo test --locked` | Run the default Rust test suite |
| `.\.tools\mips\bin\make.exe check` | Run formatting, tests, Clippy and the debug build |

A release build produces the editor executable. On macOS, `make app` additionally creates a Finder application bundle and asks before placing or replacing it on the Desktop. See [all development commands](docs/getting-started.md#common-development-commands) and [local testing](knowledge/maintainers/testing.md).

</details>

## Examples

### Sprites, textured environments and portrait UI

![A game built with Epok showing a sprite character, textured forest and portrait dialogue UI](docs/images/forest-dialogue.png)

<p align="center"><sub>Native 640 × 480 output captured in PCSX-Redux. A visual showcase of world sprites, textured geometry and HUD rendering. Character and environment artwork is synthetic prototype imagery prepared for PSX textures.</sub></p>

See [Sprites and particles](docs/sprites-particles.md), [Textures](docs/textures.md) and [HUD](docs/hud.md) for the engine features used in this scene.

### A playable third-person scene

Choose the **Third Person** template for an animated player with camera-relative movement, jumping, collision and an obstacle-aware orbit camera in an optimized arena. See [Third Person template](docs/third-person.md).

Engine APIs are documented under [Input and collision](docs/input-collision.md), [Sprites and particles](docs/sprites-particles.md), [HUD](docs/hud.md) and [Runtime services](docs/runtime-services.md).

### From FBX to an animated character

Import a mesh with its armature and clips, inspect the skeleton, preview animation and add the character to your scene. Reimport preserves asset identities and edited material colors.

Try **Assets → Import sample character (FBX)…**, select the imported **ModelSource**, then choose **Add character to scene**.

<details>
<summary><strong>See the skeletal animation workflow</strong></summary>

![Epok skeletal preview with the original mannequin, animation controls and bone overlay](docs/images/skeletal-preview.png)

The included mannequin has 96 vertices, 144 triangles and Idle/Walk clips. The PSX skeletal profile uses one bone per vertex and can compile the same quantized 30 Hz clips as direct rigid GTE skinning or compressed baked vertex frames. See [Skeletal characters](docs/skeletal.md) for the workflow and limits.

</details>

## External tools and automation

Connect an MCP-compatible client to work directly with the running editor. Epok exposes **24 tools** for scenes, actors, assets, scripts, screenshots, builds and emulator controls.

A connected client can arrange entities, move the Scene camera, capture Scene/Game/HUD/editor views, inspect build logs and control Play, Pause and Step. Scene batches support Undo/Redo and revision checks; file replacements retain local backups.

> “Inspect this scene, add a blue cube beside the platform, frame it and show me a screenshot. Then build the game and check the logs.”

Enable **Edit → Editor Preferences → Integrations / MCP → Enable MCP Server → Apply**, then copy the HTTP or stdio client configuration. MCP is **off by default**, uses a local access key and listens only on your computer. Epok does not require an external account or a specific provider.

<details>
<summary><strong>View the connection settings</strong></summary>

![Epok Integrations / MCP preferences with the optional server disabled](docs/images/mcp-preferences.png)

</details>

See the [MCP guide](docs/mcp.md) for setup, the full tool list and current limits.

## Documentation

Browse the **[Epok documentation](docs/getting-started.md)** for guides, workflows and API references. The Markdown sources are also linked below.

| Area | Guides |
| --- | --- |
| **Start and configure** | [Complete feature catalog](docs/features.md) · [Getting started](docs/getting-started.md) · [Projects](docs/projects.md) · [Settings](docs/settings.md) · [Editor](docs/editor.md) · [Migrating to actors](docs/migration-actors.md) |
| **Create content** | [Blockout](docs/blockout.md) · [Terrain](docs/terrain.md) · [Third Person arena](docs/third-person.md) · [Static model import](docs/static-mesh-import.md) · [Skeletal characters](docs/skeletal.md) |
| **Render and animate** | [Textures](docs/textures.md) · [Lighting](docs/lighting.md) · [Sprites and particles](docs/sprites-particles.md) · [Environment effects](docs/environment-effects.md) · [Palette animation](docs/palette-animation.md) |
| **Build gameplay** | [Gameplay API](docs/gameplay-api.md) · [Actors and components](docs/actors.md) · [Blueprints](docs/blueprints.md) · [C++ scripting](docs/scripting.md) · [Lua scripting](docs/lua-scripting.md) · [Input and collision](docs/input-collision.md) · [Runtime services](docs/runtime-services.md) · [Cameras and resources](docs/camera-resources.md) · [Memory Card](docs/memory-card.md) |
| **Look up C++ APIs** | [Complete API reference](docs/api/index.md) · [Epok runtime API](docs/api/epok.md) · [PsyQo API](docs/api/psyqo.md) |
| **Sound and interface** | [Assets and audio](docs/assets.md) · [HUD](docs/hud.md) |
| **Extend and verify** | [Integrations / MCP](docs/mcp.md) · [Architecture](knowledge/architecture.md) · [Performance](docs/performance.md) · [Lua VM runtime](docs/lua-vm-runtime.md) · [Testing](knowledge/maintainers/testing.md) · [Runtime and export](runtime/README.md) |

## Current limits

Epok targets the original hardware's constraints. Keep these boundaries in mind when planning a project:

- **Animation:** rigid GTE skinning and compressed baked vertex clips are selectable per imported model; skeletal textures, blended skin weights and animation blending remain future work.
- **Editing:** the open map's last 32 scene edits, Blueprint graphs/templates, Blockout geometry and MCP scene batches have Undo/Redo. Entity multiselection remains future work.
- **Actors:** actor and component classes, per-map scene Blueprints and the 3D/2D/UI modes are host-validated; PSX build and emulator validation of actor content is pending on an SDK machine. There are no Pawn/Character archetypes, no positional audio and no 2D rigid-body physics.
- **Rendering:** frustum clipping and ordering tables are used; intersecting polygons can still produce sorting artifacts. Capacity limits are not frame-rate guarantees.
- **Simulation:** measured time drives fixed 60 Hz steps with bounded catch-up. Collision uses conservative AABBs, rather than rigid-body physics.
- **Resources:** prelinked scene banks share resources and reuse active objects and VRAM. Resident data must fit PSX RAM; arbitrary map streaming from CD is not implemented.
- **Validation:** builds and emulator execution are verified. Physical controllers, console performance and persistence on real Memory Cards still need hardware validation.

See [Textures](docs/textures.md), [Sprites and particles](docs/sprites-particles.md), [HUD](docs/hud.md), [Runtime services](docs/runtime-services.md) and [Resource counters](docs/camera-resources.md) for feature budgets and limits.

## Contributing

See [Contributing](CONTRIBUTING.md) for setup, local checks and change guidelines.
Full validation runs locally; GitHub Actions enforces the release branch and
version policy.

## License and credits

Epok's original code and included mannequin are [MIT licensed](LICENSE). Fonts, SDKs and other dependencies retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md) and [runtime notices](runtime/THIRD_PARTY_NOTICES.md).

Built with Rust, Dear ImGui, PsyQo and PCSX-Redux. Proprietary PlayStation BIOS images are not bundled.

Python validation and migration tools require `python -m pip install -r tools/requirements.txt`. See [document formats](docs/formats.md) for YAML and the UniQo-to-Epok migration.
