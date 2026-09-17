# Epok C++ runtime

This runtime executes exported scenes and C++ components natively on PlayStation using PsyQo, Q12 fixed-point arithmetic, Gouraud triangles and ordering tables.

See [native performance](docs/performance.md) in a standalone export for frame
counters, transform/collision caching, GTE geometry, clipping paths and profiling.

Inside the Epok repository, build through the editor or `cargo run --locked -- --project examples/sample-game --build-psx`. The instructions below apply to a **generated standalone export**, which includes `scene.hh`, `sources.mk`, `scene.epokmap` and a `scripts/` directory. The unexported runtime directory alone is not a complete game project.

## Build an exported project

No editor, host Clang extractor, or Lua VM is required by native exports. On Windows the included `build.ps1` accepts `-Make <make.exe> -Nugget <SDK directory> -ToolchainBin <MIPS bin>`. It normalizes the working directory for GNU tools, including Unicode directories with an ASCII Windows short alias. If the volume has no suitable alias, move the complete export to an ASCII directory. Keep SDK/tool installation paths ASCII and without spaces.

Install GCC MIPS 16.2.0 and Make, and put their binaries on PATH for the current shell. Use a path without spaces.

From the export directory:

```sh
git clone --no-checkout https://github.com/pcsx-redux/nugget.git third_party/nugget
git -C third_party/nugget checkout --detach 6186b131aacc5853a9161fb076ed34ffe504552d
make BUILD=Release NUGGET_DIR=third_party/nugget
```

Do not initialize Nugget's nested submodules recursively. They are not required by this PsyQo build, and this mirror revision contains an unrelated invalid submodule path.

Output is `epok.ps-exe`. The ELF and map files support debugging. The exported Makefile requires an explicit `NUGGET_DIR` when the SDK is elsewhere. For an existing portable Epok setup, prepend its `.tools/mips/bin` to the current shell's PATH and point `NUGGET_DIR` to its `third_party/nugget`.

The default standalone Make target dispatches to `build.ps1` on Windows or
`build.sh` on Unix. These launchers rebuild current SDK sources into private
objects and a fresh archive, then rebuild the application. They preserve existing
shared SDK objects and libraries and do not depend on editor caches or file
timestamps to detect changes. Generated private SDK object directories remain in
the SDK folder for diagnosis; SDK archives stay under the export's `sdk/` folder.
Run native builds sequentially against a given SDK. Explicit `make all` bypasses
the standalone launcher and retains Nugget's ordinary incremental behavior.

The exported game does not require the Rust editor or Lua. Run the PS-X executable using a compatible emulator or a suitable console loading method. Music-enabled or geometry-streaming exports require the generated CD image as described below. Hardware validation is still pending.

`Scripts.epokmanifest` v2 lists emitted files relative to its own directory and
authoring dependencies relative to the original project. Those dependencies are
provenance, not inputs that standalone Make must reopen. Copy the complete export
when changing folders; use the selected SDK/toolchain paths for the new build.
Exports include the [Timeline/VFX guide](docs/timelines.md) and
[Blueprint guide](docs/blueprints.md) alongside the compiled runtime sources.

## Actors and components

Maps contain Actors with typed ActorComponents. `Actor3D`, `Actor2D` and `UIActor` derive from the transform-free `Actor` base and own roots for their spatial domain. The logical parent controls inherited activation and lifetime; compatible spatial attachments control transform inheritance.

The loader creates all Actors and components, assigns authored properties and resolves references before calling `begin_play`. Components tick before their owner, followed by the map's `SceneScriptActor`. Measured elapsed time drives the fixed 60 Hz simulation, with at most eight recovery steps per frame. Q12 steps alternate 68/69 raw units to preserve elapsed simulation time. `time.set_paused` pauses simulation; component `frame_update` remains available for pause-menu input.

An ActorComponent obtains its Actor through `get_owner()`. Class metadata declares compatible owner domains. Access the corresponding spatial root for transform operations. References use generation-checked `ObjectId`; destroying an Actor invalidates its references and destroys its child Actors and components. The Level provides bounded spawning, component attachment and activation. Creation during a callback is deferred until that dispatch completes.

Camera3DComponent selects a view through its owning Actor. Transform values are local to the spatial parent. After ticks, the runtime composes parent-first matrices and reuses them for rendering. Actors without render components participate in the hierarchy without emitting geometry.

## Display

The generated `display.hh` selects the native NTSC output dimensions. The default is 640 x 480 interlaced; Project Settings also offers 256/320/368/512/640 widths at 240 progressive or 480 interlaced lines. Projection preserves the 4:3 camera view. High-resolution output costs more GPU fill work and interlacing may flicker on a CRT.

## Materials and lighting

Each object has an RGB material. Unlit uses that color directly. Baked Vertex uses colors prepared by the editor; Realtime uses bounded GTE lighting with one directional and one point light per object. Mixed lights contribute to the bake or realtime receivers without doubling their contribution.

`Light`, `MeshLighting` and `BlobShadow` are available through the component API. Ambient lighting is held in `lighting_environment`. `lighting_stats` reports the last complete frame's counters and CPU timing in video scanlines (approximately 64 microseconds), excluding the final GPU wait.

Baked receivers and their ancestors must remain static. C++ light changes do not regenerate baked shadows or geometry. Blob shadows use eight triangles on an axis-aligned horizontal floor, with up to 32 visible shadows. Textured meshes preserve baked and GTE lighting. Materials support color modulation, cutout and the four PSX semitransparency modes.

## Editable geometry

EditableMesh assets compile into linked spatial `MeshGeometry` chunks with relative Q12 positions, bounds, face normals, material colors and baked-color offsets. The runtime rejects chunk bounds before vertex transformation, then clips triangles against the frustum and rejects backfaces. Group names and editable source data stay on the host. Scene material overrides are compiled into face materials; the legacy cube tint is initialized to white for these objects.

`mesh_stats` reports tested/visible chunks, transformed vertices, rejected backfaces and clipped input triangles. Use it with `lighting_stats.dropped_triangles` and frame timing to profile a level. The clipping reserve is bounded; capacity limits do not guarantee frame rate. See the editor's Blockout guide for authoring limits.

`performance_stats.streamed_chunks` is descriptive and collected only with
`EPOK_PROFILE_DETAIL=1`. Ordinary builds store `UINT32_MAX` (not collected),
which the editor profiler exposes as JSON `null`. Read, failure and dropped
geometry counters stay active. The profiler's existing `--detail` option enables
this diagnostic and extra timers; its overhead makes those timings unsuitable
as release FPS measurements.

Project Settings > Engine > Rendering offers optional conservative visibility
masks. Visibility and geometry streaming are **Experimental**, remain off by
default and may lower FPS; nearby-page preloading is also Experimental.
Engine > Streaming offers geometry CD pages, a bounded page pool, a
per-frame triangle budget and optional nearby-page preloading. Exporting with
streaming enabled includes `GEOMETRY.BIN` and a CD manifest; launching only the
PS-X executable cannot supply the paged payload. Required reads can stall and
restart XA music. See [geometry streaming](docs/streaming.md) for configuration,
resource lifetimes and CD launch requirements.

Streamed meshes can retain prepared packets within the configured budget;
objects that do not fit use per-frame packets. Startup loads active pages before
gameplay when they fit the pool. Later required reads may stall rendering.

## HUD

Canvas, RectTransform, Image, Text and ProgressBar belong to the same scene entities. HUD rendering follows the 3D world at the configured output resolution. Anchors and pivots are 0..1; position and Size Delta use Q12 native pixels with +Y up. The 3D Transform does not affect UI layout. Text remains 8 x 16 pixels.

Add RectTransform before graphics, beneath a Canvas or another RectTransform. Text uses the attributed mig68000 8 x 16 bitmap plus derived Spanish glyphs, supports 511 UTF-8 bytes, explicit newlines and optional wrapping.

Image uses a texture index and atlas region; optional nine-slice borders preserve corners. `hud-config.hh` sets layout/rectangle/text/glyph budgets validated by the editor. Excess dynamic draws are counted in `hud_stats`. `utility.hpp` supplies focus, navigation, lists and event queues. Parent masks, text rotation/scaling and world-space Canvas remain outside this profile.

## Textures, sprites and particles

Imported PNGs compile into 8-bit PSX palettes and texture banks. Transparent texels use palette index zero; visible black stays opaque. Framebuffer and bitmap-font space are reserved, and each scene bank's VRAM occupancy is validated. Shared source pixels remain in main RAM and are uploaded when their bank becomes active.

`Sprite` supports atlas regions, pivot, size, flips, fixed/upright/spherical orientation, tint, lighting, blending and ordering bias. `SpriteAnimator` supports timed clips, looping, completion and a bounded event FIFO. Sprites share the world's ordering table and frustum clipping.

`ParticleEmitter` drives an independent 256-particle pool, with continuous/burst emission, seeded spread, velocity/gravity, finite lifetime, size/color interpolation, local/world space and flipbooks. Limits and dropped work are observable through particle and sprite statistics.

`resource_usage` reports slots, active VRAM, resident texture sources and rendering work. `fog_environment` applies vertex depth cueing; materials can scroll UVs with seam subdivision, and `PaletteAnimator` cycles a shared texture's nontransparent CLUT range. See `docs/camera-resources.md`, `docs/environment-effects.md` and `docs/palette-animation.md` for the measurement scope and effect limits.

## Input, world queries and scene lifetime

`input` exposes held/pressed/released states on both physical controller ports. AdvancedPad shares the bus safely with the Memory Card service. Collider queries include overlap, segment raycast, ground and swept kinematic movement, plus trigger enter/stay/exit callbacks.

`request_scene(name)` or `request_scene(size_t(index))` queues one of the exported scene banks. A switch stops outgoing audio, waits for XA callbacks, invalidates handles, clears simulation pools and initializes incoming components/scripts. Banks share one reusable object pool and immutable resources. Their metadata and nonstreamed resources remain prelinked and must fit main RAM. With geometry streaming enabled, immutable editable-mesh payloads use CD pages with build-global IDs; the page cache survives bank changes. Textures, collision data, scripts and other resources keep their existing storage paths.

`memory_card` offers asynchronous probe/read/write/list with service-owned buffers, paired recovery records and readback verification. It never formats cards automatically. The game's progress format remains project-owned.

`utility.hpp` also supplies value tweens, easing, bounded events and timed sequences. See the exported `docs/` directory for full API examples and limits.

## Licensing

Epok runtime sources are MIT licensed. A generated export includes `LICENSE`, `THIRD_PARTY_NOTICES.md` and `licenses/`. User-authored game scripts/assets retain their owners' chosen licenses. SDK components and the system font retain their own notices.

PsyQo: https://github.com/pcsx-redux/nugget/tree/main/psyqo
Epok Engine: see the editor distribution for project documentation.

## Audio

Exports include `audio.hpp`, `music.hpp` and a generated `audio-bank.hh`. Sampled Resident clips are mono SPU-ADPCM; Stream clips are XA files in `music/`. Role (SFX/Music/Ambience/Dialogue) is independent of residency. Source recordings and editor import packages are not shipped to the console.

MusicSequence uses a resident EPSQ event stream and an explicitly assigned or project-default SoundBank. Exports that reference sequences also include the portable `sequence_kernel.hpp` and the PSX sequence service, payload and clock headers. `audio/*.epsq` and `audio/*.epsb` are embedded in the executable, including immutable sample DMA sources in main RAM. Samples are additionally uploaded to SPU RAM. Sequence playback performs no CD reads. All referenced banks are preloaded; the combined SFX and bank limit is 520,192 bytes after the 4 KiB capture reserve. Identical samples deduplicate within each bank; independently authored banks and standalone SFX retain separate allocations.

AudioSource and Blueprint audio references accept MusicSequence. `play()`, `stop()`, `is_playing()`, autoplay, enable/disable, gain and sample pitch use the same interface. Pitch does not change sequence tempo. Four sequence instances share 24 physical SPU voices with SFX. Each song defaults to 16 logical voices, explicitly configurable from 1 to 24. Shared stealing retains priority then age; unavailable lower-priority notes are counted. Scene switches wait for sequence retirement as well as XA callbacks before reusing scene owners.

Timer 0 requests approximately 529 Hz service (system clock, target 64000), Timer 2 measures elapsed CPU/8 time, and the existing GPU Timer 1 provides a wrap guard. Timer 0 does **not** have a CPU/8 divider; the previous configuration actually requested about 2000 IRQ/s, not its documented 250 Hz. Timing is independent of rendered frames. A gap too long to measure unambiguously stops sequences with a clock fault. Native EPSQ v3 streams execute editor-compiled SPU commands and use hardware ADSR. Sparse pitch/gain automation is cooked at 250 Hz plus event boundaries. Native-only builds omit software synthesis and MIDI matching storage. Legacy portable banks retain linear software ADSR, and Software reference retains the previous SoundFont synthesis path. Both paths share priority/layer admission and optional Room reverb. `epok::music_sequence_stats` exposes starts, loops, physical steals, priority-denied notes, capacity errors, peak voices, active instances, pitch clamps, clock faults, maximum service gap and CPU/8 service ticks; `sequence_timing_stats.service_ticks` accumulates measured service time. Read audio counters under `epok::SequenceLock` for a consistent snapshot. Offline/native voice steals are reported during cooking. Physical hardware validation remains pending.

Access an assigned source with `entity().get<epok::AudioSource>()`. Both profiles support play/stop, looping, volume and priority. SFX provides 24 voices and pitch control. XA provides one CD stream at fixed pitch; a higher/equal-priority music request can switch tracks. Music loops and changes include a CD seek, so seamless playback is not guaranteed. The CD controller is owned by the music service. `epok::music_stats` reports playback state and errors.

When the export contains `disc.xml`, first compile the executable, then use [mkpsxiso 2.30](https://github.com/Lameguy64/mkpsxiso/releases/tag/v2.30) in the export directory:

```sh
mkpsxiso -y disc.xml
```

Outputs are `epok.bin` and `epok.cue`. Boot the CUE in a compatible emulator, retaining the neighboring BIN. This image uses `SYSTEM.CNF` to load `EPOK.EXE`; OpenBIOS boot and XA playback are covered by integration tests. No proprietary Sony license sectors or BIOS are distributed; real-console boot depends on the user's loading setup and is not verified. The XA converter is only needed when importing/regenerating assets, not when building a complete export.

From the editor, **File > Package PSX Disc...** packages a physical-disc target even when the game has no XA music or streaming data. Choose the target region and output format, then provide a system-area license file you are authorized to use. Epok stores this local path in `UserSettings/DiscExport.epokprefs`, which is not part of the project source. BIN/CUE is the recommended format for PlayStation CD-Rs; ISO is provided for burners that accept only that extension. A compatible loader or modchip is still required for CD-R media because its SCEx pregroove cannot be written by ordinary CD burners.


Skeletal meshes use `Animator` for clip selection/play/pause/resume/stop. Geometry, skeletons and quantized 30 Hz clips are shared; one bone transforms each vertex. No FBX parser runs on the console. See the editor repository `docs/skeletal.md` for the profile and authoring limits.
