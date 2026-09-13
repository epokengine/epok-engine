# Architecture and repository layout

Epok has two compilation targets: a Rust desktop editor and a native C++ PlayStation runtime. They share scene semantics and are maintained together so changes to serialization, generated code and runtime behavior can be reviewed as one change.

## Ownership

| Path | Owner and purpose |
| --- | --- |
| `src/` | Desktop editor, scene data, preview rendering and build orchestration |
| `resources/editor/` | Editor-owned fonts and icons, embedded in the Rust executable |
| `examples/sample-game/` | Independent demo project and embedded sample-template source |
| `runtime/` | Epok C++ runtime sources staged into builds and exports |
| `integrations/pcsx-redux/` | Host-side Lua adapter for the external emulator |
| `third_party/nugget/` | Pinned upstream SDK submodule; no local source patches |
| `tools/` | Portable dependency setup and resource regeneration |
| `tests/integration/` | Explicit local checks using the MIPS compiler and emulator |
| `docs/images/` | Curated product screenshots |
| `.tools/` | Ignored downloaded tools and archives |
| `<game>/.epok/` | Ignored build staging, emulator state and project lock |
| `<game>/UserSettings/` | Ignored per-project editor layout |
| `artifacts/` | Ignored verification logs, screenshots and memory captures |
| `exports/` | Ignored generated standalone C++ projects |
| `target/` | Ignored Cargo build output |

The repository is the editor installation/development root. New games are identified by one root `.epokproject` descriptor; legacy `ProjectSettings/project.json` remains readable and migrates only explicitly. Games own their assets, caches, layout and exports. Folder/file aliases resolve to one canonical root before locking/reflection. Startup without a project shows the Hub.

`reflection_schema`, `header_tool`, `header_extract` and `reflection` define the versioned semantic C++ extraction boundary. libclang runs in a pinned host process, not in the editor or PSX game. `blueprint` exposes the shared picker/Inspector registry; `scripts` adapts legacy metadata and creates native classes; `script_values` handles typed overrides; `script_backend` separates authoring capabilities/artifacts from execution declaration, binding and reset. Non-native/Lua execution remains unavailable with preserved serialized data and explicit diagnostics. See the [foundation status](initiatives/blueprints/foundation-status.md).

`workspace::Project` owns the canonical game root, versioned manifest and OS lock. `Editor` owns that project for the session lifetime. Closing drops the build job before releasing the lock. Runtime sources are embedded in the editor and staged into the selected game's cache. Tool configuration resolves against the editor installation, independently of game paths and the process working directory. See [Projects](../docs/projects.md) for the format and project lifecycle; [Assets and audio](../docs/assets.md) documents portable imported assets and reconciliation.

## Source responsibilities

| Modules | Responsibility |
| --- | --- |
| `workspace`, `hub` | Project creation, validation, locking, recent list and project selection |
| `main`, `platform`, `native` | Entry point, desktop window/GPU and owned emulator window visibility |
| `editor`, `gui`, `hud_editor`, `lighting_editor` | Editor state, actions, panels and interaction |
| `scene`, `transform`, `scripts` | Scene model, hierarchy, persistence and component metadata |
| `viewport`, `scene_gpu`, `picking`, `gizmo` | CPU/GPU previews, selection and transforms |
| `hud`, `lighting`, `shadows` | Layout, baking, lighting and shadow calculations |
| `project`, `export`, `pipeline` | Configuration, generated C++, staging, compilation and process lifecycle |
| `bridge` | Framed emulator video, controller state and transport |
| `mcp`, `mcp_tools`, `mcp_stdio` | Optional loopback MCP server, queued editor operations and stdio compatibility bridge |
| `assets`, `asset_manager`, `asset_ui` | Portable UUID packages, source recovery, import jobs and asset UI |
| `audio_import`, `audio` | AudioSource validation and offline SPU-ADPCM conversion/banks |
| `mesh`, `mesh_editor`, `mesh_compile` | EditableMesh packages, Blockout interaction/history, material slots and spatial PSX geometry compilation |

The Rust editor is one Cargo package. No separate engine crate or runtime repository is required at this stage.

`timeline`, `timeline_compile`, `timeline_curve`, and `timeline_editor` implement
the TimelineAsset source/validation/cooked-table/preview path using the
existing asset scanner, reflection registry, EntityRef resolver, and atomic asset
writes. `runtime/timeline.hpp` is the measured Q12 evaluation kernel; the shared
director and ParticleEffect pools connect scene components and transient effects
to compiled typed tables. Effects embed TimelineAsset and use internal layer
handles, sharing the existing 256 particles and sprite renderer budgets.
`timeline_adapters` installs the opt-in project-owned native component library
from `templates/TimelineAdapters.hpp`. It uses ordinary reflected Behaviour
classes and native/Blueprint inheritance. Reflection schema v6's optional
`TimelineRequires` metadata declares component availability; host validation and
generated runtime guards use the same inherited requirements. Compiler v5 calls
the unreflected `Behaviour::timeline_sync(property_id, read)` hook around native
property access, capturing live component state and applying only that property
before events/restoration. EffectLayer access stays direct. This introduces no
parallel component catalog, entity identity or continuation mechanism.
`particle_effect_preview` sends validated typed tables through the host-only
`native/effect_preview` C ABI to the same C++ kernels linked into the editor.
SceneGpu draws the resulting sprite snapshots. Native code is never hot-reloaded.
Typed Blueprint playback, marker waits/subscriptions and completion/cancellation
use the existing eight-slot latent continuation table. Captured arguments,
owner/generation checks and reentry epochs keep callbacks within that lifecycle.
`artifact_dependencies` records host artifact provenance with the existing
compiler's IDs and content signatures. Timeline caches publish precise member
and asset dependencies; changes retain stale diagnostic data until revalidation.
`blueprint_dependencies` captures per-class generation footprints from compiler
snapshots, including callable contracts, inherited property layouts, typed asset
plays/markers and imported resources. Only persisted-source compilations publish
these records; the canvas's in-memory validation stays side-effect free.
`playback_staging` captures generated timeline/effect header signatures alongside
their exact compiled inputs and referenced texture mappings. It publishes only
after successful staging, distinguishes build/export destinations by relative
output path, and marks prior headers stale after failed staging. Instance
bindings and overrides belong to scene generation, not reusable effect headers.
`scene_dependencies` carries the actual source snapshot through Build/Play/Export
without adding another scene format or entity identity. Saved file inputs and
editor document inputs have separate project-relative keys. Generated scene
headers capture maps, rendering settings, the script catalog, Blueprint inputs,
resources and reflected override properties. The editor observes these inputs
and uses its existing Stop/rebuild path; lighting bake caches are derived data.
`staging_files` records successful writes through the existing staging writer,
including unchanged files, rather than scanning old output directories. Native
source snapshots and generated script outputs join the same graph. Build tickets
validate manifests before and after compilation and publish executable hashes
only for unchanged inputs. Debug/disc preparation explicitly extends the stage
manifest; each standalone export records its own stage and documentation.
`blueprint-sources` records the persistent class IDs in the actual compiler input
set, including the empty set and duplicate counts. Scene preparation and late
script generation compare their captured sets before publishing staging output.
Shared factory glue and resource selection depend on membership; individual
class headers retain their existing precise footprints. Build tickets and export
publication reread current Blueprint sources through the existing loader, in
addition to native/playback observation. Missing class-set provenance on an older
stage forces regeneration instead of certification against today's inputs.
Generated `Scripts.epokmanifest` v2 separates manifest-relative emitted files
from project-relative authoring dependencies. Host compiler dependency snapshots
retain their original paths in memory; portable serialization never changes
their resolution or introduces a second asset index. Regeneration upgrades v1
generated manifests. Standalone compilation consumes staged sources and the
selected external SDK/toolchain, not paths into the original authoring project.
Resource providers can return captured output hashes and existing input
signatures to the same batch. Audio payloads depend on their imported package
revisions and use the project's validated index, rejecting a changed package
before conversion. The display header observes the existing Rendering generator's
selected settings output, independently of scene changes and geometry-only
settings. Staged wrappers verify these provider hashes against actual writes.
The generated audio bank records its actual output bytes and selected package
snapshots. `audio-selection:<scene input>` projects clip assignments plus script
and linked-template values from the original authored scene. Spatial transforms
and AudioSource playback controls are excluded. Whole-scene and audio projections
use one migrated document snapshot per observed saved file. Saved and open-editor
inputs retain their existing distinct identities. `timeline-audio:<UUID>` and
`blueprint-audio:<UUID>` reuse the typed resource collectors to separate literal
clip selection from curves, event timing and graph logic. Registry defaults,
Blueprint declarations/templates and script-instance values retain conservative
footprints. These projections are provenance data, not another resource resolver.
`artifact_dependency_ui` provides read-only input/consumer navigation, retained
stale reasons and cycle-safe transitive paths from that graph. The panel labels
failed reads and never certifies a cached artifact. Timeline/effect observation
uses partial results from the existing typed loaders: unreadable, missing or
ambiguous sources invalidate their known identities and transitive consumers,
while independent valid sources remain observable. Scene preparation and typed
Blueprint playback/marker compilation select referenced assets from those same
loaders and reject missing/ambiguous required identities. Strict all-source
commands still diagnose the entire catalog. This adds no parser or asset registry.
Open preview identity errors are separate from native reflection failures;
source polling continues even while another document's error is unchanged.
The editor keeps prior source signatures independently of graph publication and
checks changed IDs against the selected stage's transitive inputs. Preview
validation cannot hide a pending source change or certify the running game.
Timeline/effect files are excluded from the coarse native/Blueprint fingerprint.
Pickers use the same partial typed catalog and retain errors alongside valid
choices. Build tickets observe current playback and Blueprint sources both before
and after compilation; standalone export observes them before publishing its manifest.
Precise resource-provider
footprints, catalog failure isolation and complete external toolchain observation
remain in progress. The editor's source watcher stops stale Play through the
existing worker, then rebuilds and restarts when Auto compile is enabled.
See [Timelines](../docs/timelines.md) and the
[phase record](initiatives/timeline-vfx-plan.md) for the current boundary.

MCP runs an authenticated Streamable HTTP service on a dedicated current-thread Tokio runtime. Network handlers enqueue bounded requests; the window loop performs editor operations and captures textures after rendering. Requests waiting beyond their deadline or cancelled by the client cannot mutate the editor. Server state belongs to the open Editor and is dropped on project close. Preferences enable it explicitly and keep credentials outside game data. The stdio mode only proxies an already running server. See [MCP](../docs/mcp.md) for its public contract.

## Build and execution

1. Resolve the startup scene and registered scene banks, EditableMesh/Texture UUIDs and source documents; validate each bank and prepare stale vertex-lighting bakes.
2. Compile editable faces and UVs into spatial chunks, share immutable texture/audio sources, validate each bank's VRAM and component budgets, stage XA music sectors, and generate `audio-bank.hh`, `scene.hh` and an explicit `sources.mk`.
3. Stage the runtime and original scripts into `.epok/build/`.
4. Run Make with the configured MIPS tools and Nugget SDK.
5. Validate the PS-X EXE header, then launch the owned emulator for Play.

Unchanged staged files retain their timestamps, allowing Make to reuse objects. The explicit source list prevents deleted scripts from being linked from an old cache. Nugget may relink even when script objects do not change.

Export stages the same runtime and generated scene into the selected game's `exports/` and includes build instructions and license notices. The exported project uses native C++; it does not need the desktop application to run.

## Preview and emulation

Scene is a wgpu editor preview with depth buffering. Game is the actual emulator display, delivered by the adapter through `PCSX.GPU.takeScreenShot()`, including active-buffer dimensions and 16/24-bit RGB formats.

PCSX-Redux runs externally with OpenBIOS, a software GPU, interpreter CPU and 2 MB RAM. Epok does not link the emulator into the Rust executable or modify its distribution.

The adapter uses loopback TCP with an OS-assigned port and a per-session identifier. It keeps one request in flight and only the newest video frame. Lua executes on the host inside PCSX-Redux; it is never linked into the PSX game. HTTP controls compilation verification and emulator execution state.

The Game view owns controller focus. Losing focus releases buttons. Stop and cleanup target the emulator process launched by this session.

## Dependency updates

The Nugget gitlink and the revision in `tools/dependencies.json` must agree. Update them together, preserving the selected SDK revision until clean builds and runtime checks pass. Avoid recursive initialization of its unrelated nested submodules.

Keep MIPS tools and emulator versions/hashes in the same manifest. Rust dependencies are resolved by `Cargo.lock`, and the host toolchain is selected by `rust-toolchain.toml`. Do not use floating download URLs or commit build products.

`audio_decode` integrates Symphonia on the host. `music` encodes/interleaves XA with the pinned psxavenc tool. `disc` writes the ISO manifest and invokes mkpsxiso after the native build; music-enabled Play boots the resulting CUE. `runtime/music.hpp` uses PsyQo async CD actions/ISO9660 and leaves sample voices available for SFX.


The [skeletal pipeline](../docs/skeletal.md) converts FBX on the host with ufbx, stores UUID-linked subassets, and emits shared immutable target tables plus independent Animator states. The PSX uses rigid bone transforms and a shared vertex scratch buffer.
