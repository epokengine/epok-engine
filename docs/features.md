# Complete feature catalog

This page inventories the capabilities implemented in Epok Engine's `develop`
branch. It is the broad map of the product; the linked guides contain the exact
workflow, limits and native API. A feature appears here only when there is an
editor/runtime implementation or a checked build path in the repository.

Epok has two cooperating products:

- a Rust desktop editor for authoring, previewing and building a game; and
- a C++20/PsyQo runtime generated and compiled as native MIPS code for the
  original PlayStation.

The project is experimental. "Implemented" does not mean that a feature is
unbounded or validated on every host and console. See [Known boundaries](#known-boundaries).

## Projects, startup and workspace

- **Project Hub.** Start without `--project` to create, find, open and remove
  projects from the recent list. Projects are independent folders rather than
  content stored inside the editor repository.
- **Project descriptors.** A root `.epokproject` YAML document stores the game
  name, startup scene, Play profile and transition settings. Paths passed to the
  editor can be either the folder or descriptor. Canonical path resolution and
  a per-project lock prevent the same project being edited twice.
- **Templates.** New-project choices are Basic, Sample game and Third Person.
  Creation claims a new directory and never merges into existing content.
- **Legacy migration and recovery.** Legacy manifests and JSON documents remain
  readable. Migration is explicit, keeps backups and has recovery choices for an
  interrupted migration. See [Projects](projects.md) and [formats](formats.md).
- **Recent projects and optional file association.** The editor keeps a
  per-user recent list. Windows can register `.epokproject` for the current user
  with the supplied script without replacing an unrelated default silently.
- **Branded asynchronous startup.** Project preparation happens behind a compact
  splash/loading presentation. Previous docking layouts are restored only when
  their panels remain valid. Long build, installation, analysis and transfer
  operations use a modal progress surface and block conflicting edits.
- **Portable configuration.** Project settings travel with the game; editor,
  MCP, emulator, serial-adapter and Content Browser preferences remain local.
  The editor installation owns its pinned tools independently of the current
  working directory. See [Settings](settings.md).

## Desktop editor

- **Dockable workspace.** Scene, Game, Hierarchy, Inspector, Project/Content
  Browser, Console and specialized asset editors can be docked and restored.
- **Scene View.** A GPU preview supports perspective and orthographic/2D modes,
  orbit, pan, fly navigation, framing, grid and wireframe display. Picking
  selects scene objects without changing authored data.
- **Hierarchy and transforms.** Create, duplicate, delete, rename and parent
  entities. Position, rotation and scale are local to the parent; transform
  gizmos edit the selected object. Scene changes have dirty-state prompts and
  normal save shortcuts.
- **Component Inspector.** Edit materials, scripts and typed script properties,
  cameras, meshes, collision, lighting, shadows, sprites, particle emitters,
  skeletal animation, audio, timelines, effects and HUD components.
- **Numeric editing.** Numeric properties accept direct text entry and
  click-drag adjustment with the appropriate cursor feedback.
- **Console.** Build, import, emulator and serial output is available in a
  read-only selectable text view. Every displayed line carries a local timestamp
  with millisecond precision; native command-echo noise is filtered while real
  compiler/transport output remains visible. Auto-scroll follows new output,
  pauses when scrolling up, resumes at the bottom, and is forced during builds.
- **Artifact dependencies.** A read-only dependency view traces generated
  artifacts to authored scenes, assets, reflection data and native inputs. It
  retains stale reasons and supports cycle-safe transitive paths instead of
  certifying old output.
- **Responsive source observation.** Source and external-tool inputs are polled
  in the background with bounded caches. Relevant changes invalidate or restart
  Auto compile without blocking the UI; unrelated damaged assets do not stop an
  independent document preview or build.

See [Using the editor](editor.md) for controls and [architecture](../knowledge/architecture.md)
for ownership boundaries.

## Content Browser and asset management

- **Native project browser.** Browse the real `assets/` tree in tiles or a
  Name/Type/Path table, including empty folders. Use Back/Forward, breadcrumbs,
  parent navigation, recursive search, type filters, sorting, resizable source
  tree and adjustable thumbnail size.
- **Favorites and Collections.** Save folder shortcuts and named sets of asset
  paths per project. Collections reference content without moving it and follow
  in-editor moves.
- **Native type presentation.** Blueprint, Scene, Texture, Mesh, Skeleton,
  Audio, Animation, Material, Timeline and Particle Effect files use distinct
  icons. Presentation names hide compound Epok suffixes without renaming files.
- **File Inspector.** Single selection shows path, size, type and available
  package/import metadata. A file selection never replaces the selected scene
  object or assigns a component accidentally.
- **Image preview.** Raw and imported textures show aspect-correct thumbnails
  and a larger checkerboard preview. The Inspector preview supports fit/reset
  and wheel zoom.
- **Audio preview.** WAV, MP3, FLAC and OGG sources and imported clips show a
  waveform, duration and Play/Stop. Imported auditions apply the saved trim,
  channel, rate and normalization settings before PSX compression.
- **Model preview.** Editable meshes, model packages and raw FBX/OBJ files have a
  GPU preview with orbit, zoom and frame/reset. Skeleton and animation assets use
  the associated model bind pose.
- **Bounded background cache.** Visible previews decode off the UI thread,
  refresh after file/reimport changes and release GPU/audio resources on eviction
  or project close. A bad preview does not disable ordinary file operations.
- **Processed-source visibility.** Importable source media is hidden by default
  once processed and can be shown for comparison. C++ source stays visible.
- **Safe file operations.** Create folders, rename, move, copy, duplicate, copy a
  path and reveal content in the OS file manager. Multi-item drag/drop preflights
  collisions, self-nesting and project boundaries. Session Undo reverses the last
  move.
- **Recoverable deletion.** Delete moves content to a project trash directory
  with a restore manifest. The last session batch can be restored without
  overwriting existing paths.
- **Identity-aware duplication.** Supported imported packages, Timeline assets
  and Particle Effects receive new UUIDs. Operations that would duplicate a C++
  or Blueprint class identity are rejected in favor of their creation/import
  workflows.
- **Reference safety and reconciliation.** Asset UUIDs survive moves and
  reimports. Missing sources can be relinked; external moves are reconciled by
  identity; referenced assets cannot be trashed through guarded asset operations.

The complete interaction and preview contract is in the
[Content Browser guide](content-browser.md). Import package rules are in
[Assets, import and audio](assets.md).

## Scenes, entities and geometry

- **YAML scenes.** `.epokmap` files describe the environment and an ordered entity
  hierarchy. Empty entities are valid transform/group nodes and every component
  is validated before native staging.
- **EditableMesh / Blockout.** Create Box, Plane, Ramp and Stairs primitives or a
  custom mesh document. Edit vertices, faces and material slots; extrude, inset,
  bevel, subdivide and build ramps. Geometry editing has local Undo/Redo and
  preserves stable asset UUIDs. See [Blockout](blockout.md).
- **Spatial compilation.** Editable faces are converted to bounded PSX chunks
  with UV/material data, visibility bounds and optional external page locations.
- **Static OBJ/MTL import.** Import triangular or quad OBJ geometry and material
  colors into editable assets with configurable scale. See
  [Static OBJ import](static-mesh-import.md).
- **Scene banks.** Register up to 15 maps in addition to startup bank zero.
  Banks share immutable textures/audio and one reusable object pool while keeping
  independently validated VRAM layouts. See [Runtime services](runtime-services.md).
- **Linked scene and template data.** Asset/template links refresh through the
  same typed component applicator used at authoring time, preserving explicit
  instance overrides.

## Actors, components and the class model

- **Object / Actor / Component classes.** A reflected class tree with four families
  (Object, Actor, Component, Level/World) alongside the original Behaviour family.
  `Actor3D`, `Actor2D` and `UIActor` are the placeable actors; `SceneScriptActor` is
  the per-map script the level loader creates.
- **Domains.** Each actor and component belongs to the 3D world, the 2D world, the UI
  or none. The domain comes from the class, drives the editor's 3D/2D/UI hierarchy
  filtering and gates which components an actor may own.
- **Components with contracts.** A component declares its owner domains, whether it
  can be an actor's root, its cardinality, its required/excluded companions and the
  target capability it needs. `AudioComponent` is shared by all three domains.
- **C++ and Blueprint chains.** `EPOK_CLASS(Blueprintable, Placeable, Spawnable,
  Domain=World3D, Id="…")` and `EPOK_COMPONENT(Root, Name="…")` declare actors and
  their default components; Blueprints derive from them with Begin Play / Tick /
  End Play, typed actor/component references and a Spawn Actor node.
- **Scene Blueprint per map.** Each map owns one `SceneScriptActor` class, edited
  from Map Settings, compiled with the project, and able to name actors, components
  and entities of its own map by identity.
- **Legacy compatibility.** Existing entities and Behaviours are unchanged and are
  additionally presented as a derived actor view; a Behaviour on an actor runs
  through a compatibility component, and an entity carrying both 3D and UI data is
  reported as a pending conversion rather than split.
- **Cooked tables and counters.** Actors, components, overrides and the scene script
  are cooked into per-bank tables with a bounded slot registry, and `actor_stats`
  reports alive/peak/rejected/spawned/deferred counts.

Host-validated in the editor and by the C++ host test suite; PSX build, export and
emulator validation are pending on an SDK machine. See
[Actors and components](actors.md) and
[Migrating an existing project](migration-actors.md).

## Textures, materials and display

- **PNG import.** Convert source PNGs into portable `.epokasset` packages with a
  source snapshot, 8-bit indexed pixels and a 256-color CLUT. Reimport keeps the
  UUID. See [Textures](textures.md).
- **VRAM packing and validation.** The editor places texture pixels and palettes
  alongside framebuffers and reserved HUD/loading regions, validating each scene
  bank against the 1 MiB address space.
- **Materials.** Per-object/face tint, unlit mode, texture assignment, UV region,
  depth bias, UV scrolling and Cutout/Average/Add/Subtract/AddQuarter blend modes
  compile to native GPU packets.
- **Display configuration.** Project resolution and progressive/interlaced NTSC
  frame clear/display behavior are generated into the runtime. PAL output is not
  exposed by the current runtime.
- **PSX raster path.** The native renderer uses Q12 transforms, GTE projection,
  Gouraud triangles, ordering tables, near/far clipping, backface/frustum tests
  and optional retained packet caches for unchanged geometry.

## Lighting, shadows and environment

- **Light components.** Directional and point lights support Baked, Realtime and
  Mixed modes, color, intensity, range and priority.
- **Vertex-light bake.** Bake static mesh lighting in the editor, fingerprint the
  result and reject stale bakes. Meshes choose baked or realtime reception and
  expose subdivision/shadow controls.
- **Native realtime lighting.** A bounded renderer selects local light sources
  and shades geometry through the GTE-aware runtime path.
- **Shadows.** Static bake occlusion and moving blob shadows are implemented with
  explicit triangle/resource counters. See [Lighting](lighting.md).
- **Fog and screen fade.** Per-scene depth fog affects supported geometry and is
  authorable from C++, Blueprint and Lua through the `Scene` group, which rejects
  an invalid distance range without mutating anything; the runtime also exposes a
  full-screen fade used directly and by transitions.
- **Animated surfaces.** Material UV scrolling supports water-like motion, and
  palette animators cycle imported CLUT entries. See
  [Environment effects](environment-effects.md) and
  [Palette animation](palette-animation.md).

## Cameras, sprites, particles and skeletal animation

- **Cameras.** The first active camera is the default; gameplay can select another
  active camera. FOV and world-to-framebuffer projection are available natively.
- **World sprites.** Fixed, upright and spherical billboards support texture
  regions, tint, blend/depth settings and clipping/culling statistics.
- **Sprite animation.** Flipbook clips have per-frame duration, looping,
  pause/resume, completion and a bounded event queue.
- **Particle emitters.** Scene emitters and effect-owned emitters share a global
  256-particle pool, bounded per-emitter creation, deterministic seeds, bursts
  and dropped/peak counters. See [Sprites and particles](sprites-particles.md).
- **Particle Effect assets.** A `.particle-effect.json` contains up to eight
  editable layers and an embedded Timeline. Presets, bursts, typed overrides,
  scene components, transient playback handles and a native-kernel editor preview
  are implemented. See [Using the VFX editor](vfx-editor.md).
- **FBX skeletal import.** Import a mesh, armature and clips as a UUID-linked
  ModelSource package, inspect the skeleton, preview clips and place a character.
  The PSX profile supports up to 64 bones, 512 vertices and 1,024 triangles with
  one rigid bone per vertex, quantized 30 Hz clips and up to 32 clips per model.
  Each model can compile to bone-grouped direct GTE skinning or compressed baked
  vertex frames; conservative animation envelopes cull characters before
  pose/decode work. See [Skeletal characters](skeletal.md).
- **Textured skeletal characters.** Each skeletal triangle carries optional
  per-corner texture coordinates that keep atlas seams distinct without
  duplicating shared cooked positions, cooked through the same texture-page helper
  as static meshes. Assign a texture and edit material state in the model window;
  legacy untextured models remain readable and render unchanged.

## Timelines and sequenced effects

- **Timeline assets and scene directors.** Create reusable typed sequences or
  attach an automatic Timeline component to an entity. Assets have stable UUIDs,
  configurable timebase/loop/restore behavior, slots, tracks, markers and events.
- **Native sequencer.** A dedicated Scene/Sequencer layout edits a Timeline asset
  in place with a frame ruler, searchable track tree, snapping, box selection,
  multi-key editing, reverse and looping transport, a curve editor and a live
  scene-camera preview that scrubs onto a disposable scene clone. The preview
  auto-binds only unambiguous targets and never mutates the authored scene.
- **Property tracks and sections.** Q12 curves support Linear, Step, Smoothstep,
  Ease In and Ease Out interpolation and up to 256 keys. A property track may be
  split into non-overlapping sections, each with a source offset and a rational
  playback rate for time remapping, and vector properties expand into per-lane
  X/Y/Z/W channels. Typed adapters read/apply native or Blueprint component
  properties and validate required component availability.
- **Event tracks.** Invoke reflected functions with cooked typed arguments.
  Marker/event dispatch handles multiple crossings and bounded diagnostics.
- **Playback control.** Play, stop, pause, resume, seek and query bounded handles.
  Scene and transient effect directors each have measured capacity.
- **Blueprint integration.** Blueprint nodes play Timeline/Effect assets, bind
  slots, wait for a marker or completion/cancellation, subscribe to repeated
  markers and control/burst active playback.
- **Dependency-safe cooking.** Precise source, reflection, member, resource and
  generated-header footprints prevent changed or ambiguous content from
  certifying stale output. Unsaved open documents participate in Play using the
  documented save/discard rules.

See the complete [Timeline reference](timelines.md), the
[spell tutorial](spell-tutorial.md) and [troubleshooting guide](blueprints-vfx-troubleshooting.md).

## HUD and 2D interface

- **Canvas hierarchy.** Canvas and RectTransform provide anchored parent-relative
  layout in the editor's 2D mode and native HUD renderer.
- **Widgets.** Solid rectangles, atlas images, nine-slice borders, text and
  progress bars support color and bounded ordering.
- **Text.** The bundled bitmap font covers printable ASCII plus documented Spanish
  characters, wrapping and fixed rows/columns. Generated C++ exposes mutable text.
- **Retained rendering.** Static HUD packets can be retained between frames while
  dynamic text/value changes invalidate the appropriate output.
- **Navigation utilities.** `Focus<N>` and `layout_list` provide allocation-free
  selection and list layout for game-owned menus.

See [HUD and 2D entities](hud.md).

## Audio

- **Source formats.** WAV, MP3, FLAC, OGG/Vorbis and PCM sources can be decoded on
  the host. Imported metadata includes usage, channels, rate, trim, normalization
  and looping.
- **Resident SFX.** Convert to PSX SPU ADPCM, stage a shared bank and play through
  `AudioSource` with clip, volume, pitch, looping, autoplay and runtime controls.
  The bank observes typed clip selections rather than unrelated transform/data
  changes.
- **Native music sequences (Epok Pulse).** Convert a MIDI and SoundFont into a
  resident `MusicSequence` and cook it entirely on the host into a bounded,
  absolute-timestamped SPU command stream, so the console only dispatches register
  commands to the hardware voices. A software-synthesizer reference driver remains
  selectable, a Target Preview auditions the cooked result, and a sequence can be
  rendered to a PCM WAV as a bridge to streamed disc audio. Import automatically
  fits the sample bank to the PSX budget.
- **XA background music.** Encode/interleave disc XA, generate the disc manifest
  and drive asynchronous ISO/CD playback with loop/end/error state.
- **Transition audio.** Scene transitions multiply authored audio gain during
  fade-out/fade-in and stop outgoing sources safely after pending callbacks.
- **Capacity.** Resident audio is validated against the 512 KiB SPU budget with a
  runtime reserve. See [Assets, import and audio](assets.md).

## Native C++ gameplay and reflection

- **Unified gameplay API.** A single engine-owned facade defines every public
  gameplay operation once in annotated C++, and the registry derives Blueprint
  nodes, Lua definitions, native adapters and VM bindings from it. Values,
  lifecycle order, failure handling and skeletal sampling behave identically on
  the C++, Blueprint and Lua (native, VM bytecode and VM source) surfaces, and
  each operation's capability requirements are tracked into the cook manifest.
- **C++20 Actors and components.** Create project classes, derive from eligible
  native or generated classes and implement reflected lifecycle/event methods.
  Exported classes compile directly into the MIPS executable.
- **Semantic reflection.** A pinned libclang extractor records stable class IDs,
  inheritance, editable properties, functions, parameter directions and Timeline
  metadata. Versioned manifests and caches are validated against source/tool
  inputs.
- **Inspector values.** Bool, signed/unsigned integer, fixed point, vectors,
  transforms, strings, entity/class references and typed asset references have
  checked defaults and instance overrides where supported.
- **Lifecycle.** All bindings and properties exist before `start`; active scripts
  receive `on_enable`, then fixed-step `update`, and later `on_disable` /
  `on_destroy`. Default-constructible, assignable Behaviour state is restored on
  bank reload.
- **Runtime utilities.** Allocation-free Tween easing over seventeen Q12 curves
  with delay and loop plans, scalar and Vector3, bounded EventQueue, timed
  Sequence, UI Focus and list-layout helpers are included.

See [C++ scripting and exports](scripting.md).

## Lua gameplay scripting

- **Reflected Lua classes.** Author `assets/scripts/*.lua` classes by extending
  eligible C++, Blueprint or Lua parents. The shared class registry exposes them
  to the Inspector, parent pickers, defaults, attachment, spawning and Blueprint
  inheritance instead of maintaining a separate Lua-only object model.
- **Code-first declarations and stable identity.** A class is declared with code
  such as `local Guard = epok.Actor3D:extend()`. Plain assignments define typed
  Inspector properties, Lua Language Server annotations describe new callables,
  and `ProjectSettings/LuaClasses.epoksettings` preserves class UUIDs across
  source-file renames.
- **Three project-wide execution modes.** Native C++ lowers the typed script IR
  to C++ and links no interpreter. Lua VM bytecode cooks a verified 32-bit chunk
  on the host and links the no-parser archive; Lua VM source packages normalized
  text and links the parser archive. Play, Build and export use exactly the saved
  mode and invalidate incompatible artifacts.
- **Shared engine operations.** Intrinsic spatial/UI fields and typed builtins
  cover input, scene requests, validity/casts, spawning, ownership, audio,
  textures and component sequence/effect playback. Actor and ActorComponent
  operations are reflected once for C++, Blueprint and Lua callers.
- **Bounded target runtime.** VM modes use fixed arenas (96 KiB for bytecode and
  128 KiB for source by default), keep 64-bit identities outside the VM's int32
  boundary and route numeric operations through the Blueprint runtime contract.
- **Editor definitions and diagnostics.** Registry refresh writes
  `.epok/lua/epok.d.lua` and creates `.luarc.json` when absent, exposing native,
  Blueprint and Lua inheritance, properties, events, records, enums and builtins
  to Lua Language Server tooling.

See [Lua scripting](lua-scripting.md) and the [Lua VM runtime](lua-vm-runtime.md).

## Blueprint visual gameplay

- **Native compiled graphs.** `.epokbp` classes compile ahead of time to generated
  C++ and then native MIPS code. No graph interpreter runs on the console.
- **Creation and inheritance.** Create a class with a native or Blueprint parent,
  persistent class identity, inherited variables/functions/components and
  defaults, event overrides and explicit Call Parent.
- **Graph authoring.** Components, My Blueprint, canvas and contextual Details
  panes support typed execution/data wires, contextual creation by right-dragging
  from a compatible pin, a categorized action menu, node search,
  reroutes and graph navigation.
- **Typed values and pins.** Literal and connected bool, integer, unsigned,
  fixed-point, vector, transform, string, entity, class and asset values are
  validated. Composite pins can split and recombine; inline values are edited
  generically by type.
- **Flow and expressions.** Events, function entry/return, Branch, Delay,
  variables, casts, arithmetic/comparison, reflected calls, Blueprint calls,
  transform/input/entity/audio/scene helpers and Timeline/VFX playback are lowered
  through a typed IR with saturating console arithmetic where documented.
- **Functions and cross-instance calls.** Define Blueprint functions with typed
  inputs/outputs, call them on self or compatible instances and validate execution
  connectivity, cycles, missing returns and class assignability.
- **Entity templates.** Capture component hierarchies as reusable class templates,
  inherit/override them, refresh linked instances, place classes and spawn a class
  dynamically from a bounded 32-slot runtime pool.
- **Debugging.** Instrumented Play provides per-node breakpoints, continue, pause,
  step and typed value snapshots/traces. Release builds omit the instrumentation.
- **Editor history and safety.** Graph/template Undo/Redo, atomic writes,
  dependency invalidation and compile diagnostics prevent a stale graph from being
  treated as current native output.

Start with [Your first Blueprint](blueprints-tutorial.md), then use the
[Blueprint reference](blueprints.md).

## Input, collision and runtime object services

- **Controller input.** Two ports expose connected, held, pressed, released and
  frame-edge state for the standard PSX buttons. The embedded Game view owns focus
  and releases buttons when focus is lost. An opt-in analog-controller mode adds
  signed Q12 stick axes for both sticks with an exact center and no dead zone.
- **Measured fixed step.** Host frame time feeds a fixed 60 Hz simulation with
  bounded catch-up, pause, single-step, tick counters and interpolation data.
- **AABB collision.** Collider layers/masks, solid and trigger modes, overlap,
  swept movement and trigger Enter/Stay/Exit events run in bounded storage.
  Collision is conservative and not a rigid-body solver.
- **Entity handles.** Slot-plus-generation handles detect destroyed/reused objects.
  Runtime creation, subtree destruction, activation and lookup are bounded.
- **Scene lifetime.** Deactivation affects descendants and rendering/audio/
  collision participation without deleting authored data. Destruction invalidates
  handles, particles and dynamic bindings.

See [Input, time and collision](input-collision.md) and
[Runtime services](runtime-services.md).

## Play, emulation and physical-console workflow

- **Persistent Play profile.** Choose destination (embedded PCSX-Redux, separate
  emulator window or PSX through serial), content (current open scene including
  unsaved edits or whole game from startup) and data source (in executable, CD on
  demand or PCDrv host on demand). Choices are saved in `.epokproject`.
- **Integrated Game view.** Epok launches its own PCSX-Redux process, receives the
  actual 16/24-bit framebuffer through an authenticated loopback adapter, forwards
  controller state and supports Play, Stop, Pause and Step.
- **Owned external emulator.** Windowed mode uses the same built output while the
  editor tracks and cleans up only the process/session it launched.
- **PSX over NOTPSXSerial.** Enumerate/select a serial adapter, install a pinned
  hash-verified nops package and keep a resident debug/TTY session for executable
  and PC-on-demand builds. Pause, Continue and Reset use the owned monitor;
  PC-on-demand also serves PCDrv. Play verifies the handler after runtime startup.
  Adapter identity and speed are user settings.
- **Safe installation and transfer.** Downloads are cancellable, installed
  packages are replaced transactionally, an installation lock prevents competing
  repairs and offline serial bundles are supported. Epok does not install Unirom.
- **Critical-operation locking.** Builds, tool preparation, checks and uploads
  preserve unsaved work while blocking project mutations, external drops and
  shutdown until their worker reaches a safe terminal state.

See [Play targets and loading](play.md).

## Build, memory analysis, loading and distribution

- **Generated native project.** Validate the selected scenes/assets, generate
  C++ tables and explicit source lists, stage embedded runtime/scripts and invoke
  pinned Make/GCC/Nugget tools. Unchanged staged files preserve timestamps for
  incremental compilation.
- **Input provenance.** Scene snapshots, settings, assets, Blueprints, timelines,
  generated headers, native sources/includes, compiler/linker flags, tools and SDK
  inputs are captured before/after work. Changes during a job prevent launch or
  publication of stale output.
- **Private SDK builds.** The editor and standalone launchers build certified
  private Nugget objects/archives without overwriting shared SDK outputs; a host
  lock rejects conflicting builds.
- **Memory Analyzer.** Builds generate reports by default. Open the last report,
  build first when needed, or analyze an existing compilation without launching it.
  The status bar shows current EXE and external-data sizes; stale reports retain a warning.
  Inspect Main RAM, VRAM, SPU/audio, scratchpad and generated files. The UI
  provides capacity bars, a clickable treemap, allocation table, breadcrumbs,
  symbol/source attribution, stale detection and asset location. JSON is written
  beside the build.
- **Scene transitions.** Requests fade picture and audio, display a resident black
  loading frame with configurable text/64x64 RGB555 image, retire the old bank,
  warm streamed geometry, load the new bank and fade in. Failed reads stay muted
  and allow retry. C++ can override transition options per request.
- **Executable and disc packaging.** Produce a PS-X EXE; package ISO or BIN/CUE
  with XA/external geometry and a user-supplied system-area license when required.
- **Standalone export.** Generate a relocatable C++/PsyQo project with runtime,
  scripts, assets, Make launchers, manifests, guides, licenses and notices. It can
  rebuild without the Rust editor, libclang extractor or original authoring tree.
- **Version/release policy.** `develop` carries the next version, `release` carries
  the released version and tag workflows verify the repository `VERSION`, Cargo
  metadata, version progression and bounded job timeouts. See
  [Release process](release-process.md).

See [Play targets and loading](play.md), [standalone runtime](../runtime/README.md)
and [performance](performance.md).

## Geometry streaming and performance

- **Optional geometry archive.** External EditableMesh chunks are packed into
  checked 64 KiB `GEOMETRY.BIN` pages with a configured bounded RAM pool.
- **CD or PCDrv backend.** Runtime page lookup, reservation, pin/unpin, prefetch,
  synchronous fallback and scene warmup share the same pool contract. Required
  reads can stall and CD geometry can interact with XA playback.
- **Visibility.** Precomputed chunk bounds, frustum outcodes, cached basis/bounds
  tests and spatial chunks reduce submitted geometry.
- **Retained geometry/HUD.** Optional retained GPU packets reuse unchanged static
  work while preserving per-frame visibility and resource bounds.
- **Counters.** Runtime structures expose frame timing, dropped fixed steps,
  geometry/clip/lighting/sprite/particle/HUD/streaming counts, retained rebuilds,
  resource use and scene/spawn/playback capacity failures.
- **Profiling tools.** Headless CPU/GPU scene profiles, deterministic routes,
  runtime capture comparison, VRAM comparison and visibility benchmarks are
  included for maintainers.

See [Geometry streaming](streaming.md) and [Native PSX performance](performance.md).

## Memory Card persistence

- **Asynchronous service.** Probe, read, write and list operations are serialized
  through a process-owned service with status, cancellation/owner safety and a
  PsyQo filesystem backend.
- **Save format.** Validate names and capacities, write title/icon metadata and
  use interrupted-write recovery so a failed replacement does not silently lose
  the previous save.
- **Scene independence.** Pending operations survive scene transitions and their
  callbacks are protected from destroyed game objects.

See [Memory Card service](memory-card.md).

## External tools / MCP integration

- **Optional local server.** Authenticated Streamable HTTP binds only to IPv4
  loopback; a stdio bridge connects clients to the already-running editor. The
  feature is disabled by default and requires no specific client provider.
- **Twenty-four tools.** `editor_state`, `logs_read`, `scene_read`, `scene_schema`,
  `scene_apply`, `scene_actors`, `scene_add_actor`, `scene_remove_actor`,
  `scene_set_actor`, `scene_history`, `scene_save`, `scene_open`, `entity_select`,
  `editor_view`, `viewer_screenshot`, `editor_control`, `game_input`,
  `project_settings`, `project_files`, `asset_list`, `asset_import`,
  `mesh_create`, `asset_document` and `asset_manage` cover inspection, guarded
  scene/actor/file edits, screenshots, imports, build/play and serial preparation.
- **Transactional guards.** Scene batches are atomic and revision-checked;
  project-file writes require SHA-256 revisions and keep backups; queues and
  responses have limits/timeouts; cancelled queued calls cannot mutate later.
- **Real captures and runtime control.** Return full Scene, HUD, Game or editor
  PNGs, move the editor camera, inject bounded controller input and operate the
  editor/emulator asynchronously.
- **Read-only resources.** Clients can read the embedded guide, state, scene,
  schema and settings resources in addition to tool calls.

See [Integrations / MCP](mcp.md) for schemas, security and limits.

## Command-line and automation surface

The editor exposes the following implemented headless families. Run them with
`--project <folder-or-descriptor>` unless the command is installation-wide.

| Family | Commands |
| --- | --- |
| Build and run | `--build-psx`, `--play-psx`, `--use-play-profile`, `--blueprint-debug`, `--analyze-memory`, `--export-psx` |
| Projects | `--create-project`, `--template`, `--migrate-project`, `--recover-project` |
| Assets | `--scan-assets`, `--import-texture`, `--import-obj`, `--import-fbx`, `--import-audio`, `--reimport-asset`, `--inspect-asset` |
| Native gameplay | `--reflect`, `--new-script`, `--parent`, `--new-blueprint`, `--compile-blueprints`, `--add-actor`, `--scene` |
| Timeline and VFX | `--new-timeline`, `--compile-timelines`, `--install-timeline-adapters`, `--new-particle-effect`, `--preset`, `--validate-particle-effects`, `--preview-particle-effect` |
| Editor/runtime QA | `--bake-lighting`, `--profile-scene`, `--profile-scene-cpu`, `--profile-editor`, screenshot commands and bounded `--steps` / `--stop-after` execution |
| Integration | `--mcp-stdio`, `--prepare-serial-tools`, `--list-serial-ports` |

Most families have detailed examples in [Getting started](getting-started.md),
[Assets](assets.md), [Blueprints](blueprints.md), [Timelines](timelines.md) and
[Play](play.md).

## Validation and developer tooling

- Rust unit tests cover serialization, UI interaction harnesses, asset safety,
  Blueprint connectivity/split pins/action menus, Timeline compilation and the
  editor's pure logic.
- C++ host tests exercise fixed-point/runtime helpers, visibility, clipping,
  transforms, collision, lifecycle, transitions, streaming pools, particles,
  Timeline directors, Blueprint runtime/spawn/playback and Memory Card behavior.
- Python integration checks exercise project creation/migration, imports,
  reflection, staging, PSX builds, emulator captures, display/HUD/lighting,
  streaming/XA, Blueprint and VFX workflows, memory reports and Play profiles.
- Visual fixtures and curated captures cover the Content Browser, file Inspector,
  Blueprints, asset dialogs and branded startup. Performance scripts compare
  measured routes, runtime captures and VRAM output.
- Dependency manifests pin downloadable tools and validate hashes; setup scripts
  keep portable tools under `.tools/` and avoid changing global PATH.

See [Testing](../knowledge/maintainers/testing.md) and
[Shared resources](../knowledge/maintainers/resources.md).

## Known boundaries

These are deliberate descriptions of the present implementation, not promised
features:

- Windows x64, macOS Apple Silicon and Linux x86_64 are the supported editor
  targets. Blueprint reflection/authoring is available on Windows and Linux;
  macOS does not yet provision that toolchain. Linux editor and Play support,
  plus serial hardware paths outside Windows, remain experimental.
- Console builds and PCSX-Redux execution are validated. Physical-console timing,
  controllers, serial paths on every host and real Memory Card media still need
  hardware validation.
- Skeletal source skinning is rigid: one bone per vertex. Target playback selects
  direct GTE bone ranges or compressed baked vertex frames. Textured skeletal
  materials, blended weights and animation blending are not implemented.
- Collision is conservative AABB overlap/sweep with triggers, not a rigid-body
  physics engine.
- Entity multiselection is not implemented, and Undo/Redo is not editor-wide. It is
  scoped to the open map's last 32 scene edits, Blueprint graphs/templates,
  Timelines/Effects, Blockout, Content Browser moves and MCP scene batches as
  documented by each tool.
- The actor model has no Pawn/Character archetypes or positional 2D/3D audio.
  The bounded 2D collision toolkit provides box/circle overlap, raycasts, sliding
  movement and triggers, not a general rigid-body solver. Actor hierarchy editing
  and compatible component add/remove are implemented; cross-domain conversion is
  not offered.
- Scene banks, textures, scripts and resident audio must fit PSX memory. Geometry
  streaming covers EditableMesh pages only; it is not arbitrary map, texture or
  audio streaming.
- Sorting uses PSX ordering-table constraints. Intersecting polygons can still
  show ordering artifacts, and capacity validation is not a frame-rate guarantee.
- Release builds produce an editor executable, not a packaged installer/app
  bundle. A PlayStation system-area license, BIOS and Unirom are not distributed.
- `epok-lua` v1 is a deliberately bounded, statically checked profile rather than
  general Lua 5.2 compatibility. It has no coroutines, metatables, standard
  libraries, dynamic table growth or source-level debugger. Live native-code hot
  reload and general external-source packaging are not implemented.

## Source coverage map

This map makes the catalog auditable when modules are added. Test-only modules
live beside the corresponding implementation and are summarized under
[Validation and developer tooling](#validation-and-developer-tooling).

| Capability | Rust editor modules |
| --- | --- |
| Application/workspace | `main`, `platform`, `loading`, `branding`, `workspace`, `hub`, `editor`, `gui`, `busy_ui`, `document` |
| Settings/project UI | `settings`, `settings_ui`, `project`, `project_browser`, `console` |
| Asset system | `assets`, `asset_manager`, `asset_ui`, `asset_inspector`, `content_preview`, `import_settings`, `artifact_dependencies`, `artifact_dependency_ui` |
| Texture/model/audio import | `texture`, `model_import`, `obj_import`, `audio_decode`, `audio_import`, `preview_audio` |
| Scene and preview | `scene`, `transform`, `viewport`, `scene_gpu`, `picking`, `gizmo`, `scene_bank`, `scene_dependencies`, `scene_view_mode` |
| Object/Actor model | `object_model`, `actor_document` |
| Geometry and rendering | `mesh`, `mesh_ops`, `mesh_editor`, `mesh_compile`, `lighting`, `lighting_editor`, `shadows`, `effects`, `palette` |
| Components and UI | `bitmap_font`, `collision`, `collision_editor`, `hud`, `hud_editor`, `sprites`, `sprites_editor`, `particles`, `skeletal`, `skeletal_compile`, `skeletal_ui`, `third_person` |
| Native scripting/reflection | `scripts`, `script_values`, `script_backend`, `reflection`, `reflection_schema`, `header_extract`, `header_tool`, `native`, `native_metadata` |
| Lua scripting | `lua_aot`, `lua_api_stub`, `lua_asset`, `lua_bytecode`, `lua_compile`, `lua_dependencies`, `lua_frontend`, `lua_identity`, `lua_vm`, `script_ir` |
| Blueprints | `blueprint`, `blueprint_asset`, `blueprint_compile`, `blueprint_debug`, `blueprint_debug_ui`, `blueprint_dependencies`, `blueprint_editor`, `blueprint_ir`, `blueprint_playback`, `blueprint_refs`, `blueprint_spawn`, `blueprint_templates`, `blueprint_template_editor`, `blueprint_workflow`, `blueprint_action_menu`, `blueprint_inline_values`, `blueprint_split_pins` |
| Timeline/VFX | `timeline`, `timeline_adapters`, `timeline_compile`, `timeline_curve`, `timeline_editor`, `timeline_runtime`, `timeline_scene`, `particle_effect`, `particle_effect_editor`, `particle_effect_preview`, `particle_effect_scene`, `playback_staging` |
| Build/play/export | `build_inputs`, `staging_files`, `pipeline`, `play`, `play_ui`, `disc`, `export`, `export_ui`, `bridge`, `dependencies` |
| Physical console and analysis | `serial`, `serial_support`, `serial_terminal`, `serial_ui`, `memory`, `memory_ui`, `streaming`, `music`, `audio` |
| Assistant integration | `mcp`, `mcp_tools`, `mcp_stdio` |
| In-crate validation | `mcp_tests`, `skeletal_tests`, plus the Blueprint and Timeline test modules declared only for test builds |

| Native runtime area | C++ runtime files |
| --- | --- |
| Core scene/lifetime | `epok.hpp`, `lifecycle.hpp`, `scene_service.hpp`, `affine.hpp`, `transform_cache.hpp`, `time.hpp`, `transition.hpp`, `loading_renderer.hpp`, `utility.hpp` |
| Object/Actor model | `object_model.hpp`, `actor_tables.hpp`, `actor_blueprint.hpp`, `world2d.hpp` |
| Rendering/resources | `frame_clear.hpp`, `gte_geometry.hpp`, `frustum.hpp`, `polygon.hpp`, `visibility.hpp`, `retained.hpp`, `texture_types.hpp`, `texture.hpp`, `resources.hpp`, `lighting.hpp`, `shadows.hpp` |
| Visual content | `sprite_types.hpp`, `sprites.hpp`, `particle_types.hpp`, `particles.hpp`, `effect_types.hpp`, `effects.hpp`, `particle_effect_runtime.hpp`, `particle_effect_service.hpp`, `palette_types.hpp`, `palette.hpp`, `skeletal.hpp`, `hud.hpp`, `text.hpp` |
| Gameplay services | `input.hpp`, `collision.hpp`, `audio.hpp`, `music.hpp`, `memory_card.hpp`, `memory_card_backend.hpp`, `streaming_pool.hpp`, `streaming.hpp` |
| Blueprint/Lua/Timeline | `blueprint_api.hpp`, `blueprint_runtime.hpp`, `blueprint_debug.hpp`, `blueprint_spawn.hpp`, `blueprint_template.hpp`, `blueprint_playback_service.hpp`, `lua_runtime.hpp`, `playback_types.hpp`, `timeline.hpp`, `timeline_runtime.hpp`, `timeline_service.hpp` |

`runtime/main.cpp`, `runtime/Makefile`, `runtime/build.ps1`, `runtime/build.sh`
and `runtime/build-inputs.mk` assemble these services into the generated native
application and its standalone build workflow.

When a new production module or runtime service lands, update this catalog and
its focused guide in the same change.
