# Epok native performance improvement guide (PSX)

> September 8, 2026 update: conservative chunk selection and geometry streaming
> are implemented and configurable in Project Settings. See sections 9 and 10
> for scope, measurements, subsequent corrections, and limitations. Earlier
> rounds are retained as historical records. ForestTest's stationary activation
> comparisons now pass. Matched-state moving tests still contain small phase
> regressions; visibility and streaming remain experimental and off by default.

Handoff document for the contributor continuing to optimize Epok's native
runtime. It records the completed work, the rendering path, known pitfalls, the
measurement and verification workflow, and remaining work with proposed designs
and priorities. The focus is the engine: any game project can serve as a test
workload, provided it is measured using the same workflow.

Engine: `D:\GitProjects\GameEngines\PSX\Epok` (Git repository, remote
`https://github.com/franadoriv/UniQo.git`). Date: September 8, 2026.

## 1. Working rules

- Improve the engine, not the scene: do not reduce resolution or content to
  improve figures. Every gain must be general and, where appropriate,
  configurable in the editor through Project Settings and enabled by default
  when measurements support that choice.
- Preserve direct script writes to transforms and correct collision queries
  within the same tick. Scripts expose mutable fields, so the runtime scans
  transforms at every synchronization; it can make that scan cheaper but cannot
  eliminate it.
- Measure before and after every change, compare the native image (VRAM), and
  verify movement. Do not promise improvements before measuring them.
- Never overwrite earlier measurements: every profile goes into a new folder.
- All measurements use PCSX-Redux's emulated clock, not editor FPS or physical
  console timings. Hardware validation has not been performed yet.

## 2. Measured state — historical rounds 1–3

The reference workload used for these rounds was a 640 × 480 interlaced
exploration scene with 29 entities, 1,832 static-mesh triangles (terrain, rocks,
and vegetation in one editable mesh containing 168 chunks), an animated sprite
with a blob shadow, one directional light, linear fog, and a text HUD. Idle
medians, in scanline-counter units (~64 µs):

| Milestone | Units | µs | FPS |
| --- | ---: | ---: | ---: |
| Instrumented baseline | 10,512 | 672,768 | 1.5 |
| Round 1: caches, outcodes, guard band, MVMVA, `-O2` | 1,084 | 69,760 | 14.3 |
| Round 2: compact polygon loop | 755 | 48,704 | 20.5 |
| Round 2: Q8 RTPS projection | 714 | 46,080 | 21.7 |
| Round 2: one GTE matrix per object, collision bounds, indexed activity | 679 | 43,840 | 22.8 |
| Round 2: local-space lighting (one NCS per face) | 590 | 38,144 | 26.2 |
| Round 2: exporter page UVs, bounds without `Fixed` | 509 | 33,024 | 30.3 |
| Round 2: division-free `rotate`, 32-bit bounds | 488 | 31,616 | 31.6 |
| Round 2: retained HUD | 477 | 30,912 | 32.4 |
| Round 3: retained packets (optional, enabled by default) | 414 | 26,944 | 37.1 |

During the control route (walking, running, diagonal movement, collision):
25,216–28,160 µs, zero dropped simulation steps, and zero dropped triangles.
With retained packets disabled in the same build: 477 units, meaning that the
per-frame path did not regress. PS-EXE size: 493,568 bytes; RAM usage ~1.2 MB
out of 2 MB.

Detailed-build breakdown (`--detail`, 426 units including timer overhead;
counters are nested and must not be added together):

| Component | Units | Notes |
| --- | ---: | --- |
| Simulation (2 steps at 60 Hz) | 47 | Transform scanning ~28; nested collision work ~35 |
| Frame preparation | 19 | Camera view, palettes, lighting preparation ~10 |
| Per-chunk bounds and per-object lighting (`setup`) | 69 | 168 chunks tested, 72 visible |
| Vertex projection | 99 | 1,092 vertices using RTPS |
| Polygon loop | 158 | Emission 45, fog 11; 607 triangles, all retained |
| Sprites, blobs, particles | 12 | |
| HUD | 4 | 15 before retention |
| Frame completion (`finish`) | 17 | |

Estimated presentation wait: ~0.4 ms. These profiles suggest a CPU preparation
bottleneck. They do not independently establish that the GPU finishes earlier
on hardware: GPU/presentation timing still needs independent measurement
(see 7.6).

## 3. Frame architecture (`Epok/runtime/main.cpp`, `GameScene::frame`)

This describes the retained-rendering architecture established by the historical
rounds. Sections 7 and 10 describe subsequent visibility and streaming additions.

1. Reset counters, `scene_tick`, controller polling, `time.advance`.
2. Each script's `frame_update`; for each fixed step: script `update`
   (controllers usually call `move_and_slide`, which synchronizes transforms
   and collisions), `refresh_collisions` at the end of the step, triggers,
   and animators.
3. `refresh_world` (transform cache), palettes, `lighting.prepare`, `camera_view`.
4. For every active object with a mesh:
   - `object_view = view ∘ world[index]`, with X/Y rows scaled by focal length
     (FOV) and aspect ratio; `load_projection_matrix` loads one GTE matrix
     per object.
   - Decide whether the object is retained (`retained.allocate`), and at its
     first visible chunk compare the rebuild key and rebuild the current
     parity if the key changed.
   - For each chunk: conservative camera-space bounds test (32-bit when
     coefficients are small), lazy object-lighting setup
     (`lighting.shade(..., generic)` and `localize`), RTPS vertex pass
     (`project_geometry_vertex`: Q8 camera coordinates from MAC1..3, screen
     coordinates from SXY2, FLAG selects the `project_cpu` CPU fallback),
     frustum outcode and per-vertex fog amount; then the retained polygon loop
     and/or the per-frame loop.
5. Blobs, sprites, particles, retained HUD, `gpu().chain(table)`, counter copy.

Key structures and files:

- `runtime/polygon.hpp`: `ProjectedVertex` (Q8 camera coordinates, packed screen
  coordinates, fog, outcode, visible), `QuadPacket`, `CameraUnits`/`CameraQ8`
  (near 64, far 32768, division-free ordering-table bucket), exact division-free
  helpers (`modulate_channel`, `scale_channel`, `fog_amount`, `blend_fog`),
  `project_cpu`, `clip_polygon`.
- `runtime/main.cpp`: `MaterialState` (CLUT/page/command words per material),
  `PolygonEmitter` (`classify`, `direct`, `retained_emit`, out-of-line `clipped`,
  `unprojected` for scrolling UVs), `quad_colors` and `rebuild_retained` lambdas.
- `runtime/gte_geometry.hpp`: `load_projection_matrix`, `load_projection_screen`,
  `project_geometry_vertex`, `gte_projection_flags`, `gte_shared_origin_limit`;
  the round 1 MVMVA helpers remain there.
- `runtime/frustum.hpp`: `frustum_outcode_units`, `frustum_outcode32`,
  `gpu_clip_safe`.
- `runtime/lighting.hpp`: `LightingRenderer` (`prepare`, `shade`, `localize`,
  `signature`), `mesh_shade_normal`, `mesh_shade_local`.
- `runtime/retained.hpp`: `RetainedKey`, `RetainedQuad`, `RetainedGeometry`.
- `runtime/hud.hpp` (retained HUD), `runtime/collision.hpp` (`CollisionWorld`),
  `runtime/transform_cache.hpp`, `runtime/epok.hpp` (`PerformanceStats`,
  `MeshQuad` with `packed_uv`/`uvw`).
- Editor: `src/mesh_compile.rs` (page UVs), `src/settings.rs` and
  `src/settings_ui.rs` (Rendering options generating `display.hh`),
  `src/project.rs` (embedded runtime source list).

## 4. Implemented optimizations and why they work

### Round 1

Transform caching with direct-write detection and hierarchical propagation;
AABB caching and swept-volume rejection; constant-time entity indexing;
skipping empty particle work; object/camera composition shared across an object's
chunks; six-bit frustum outcodes and early rejection; GPU side clipping within
its limits (1023 × 511); GTE affine transformation (MVMVA) with CPU perspective;
normal cofactors shared per object; `main.o` compiled with `-O2`.

### Round 2

1. **Compact polygon loop.** Disassembly diagnosis: `frame()` occupied 19 KB
   of code and the emission lambda 5 KB, with nine stack arguments per triangle
   and ten actual integer divisions (this mips1 GCC emits `div` even for
   constant divisors such as 255 or 3072). Solution: classify each triangle once
   (outcodes, visibility, extent, area, orientation), prepare one packet per
   quad, and write the primitive's nine words directly; move clipping and UV
   scrolling out of line; eliminate division from hot paths.
2. **Q8 RTPS projection.** Local vertices lose 4 bits (Q12 → Q8) so camera
   coordinates and SZ fit the GTE's 16-bit stage; Q12 rotation remains exact;
   the Y row is negated (GPU Y points downward); H = height × 2/3 and OFX/OFY
   specify the center; non-square-pixel aspect ratios are folded into the X
   row. FLAG activates the CPU fallback on saturation or divider overflow
   (z < 0.625 units). Differential validation (`--validate-gte`): camera
   coordinates within 128 Q12 units and screen coordinates within 2 px of
   exact projection.
3. **One GTE matrix per object.** Separately rounded chunk translations opened
   seams between chunks; the chunk origin (a multiple of 16 Q12 units) is now
   added to each vertex, so a shared vertex receives identical inputs.
   `gte_shared_origin_limit` selects a per-chunk matrix fallback for huge
   objects.
4. **Simulation.** `CollisionWorld` bounds each query by the last enabled slot
   and skips trigger pairing when no triggers exist (Exit events are still
   emitted); indexed activity (`is_active_slot`) in runtime loops; branchless
   transform scanning.
5. **Local-space lighting.** If the object's basis is rotation with uniform
   scale (tolerance 1/256), `localize` rotates the lighting matrix into object
   space, and each lit face performs a single NCS using its stored normal;
   shear and nonuniform scale retain the cofactor path.
6. **Exporter page UVs** (`MeshQuad::packed_uv`, `uvw`) using the same VRAM
   layout as texture descriptors; **per-chunk bounds without `Fixed`**;
   division-free `rotate`.
7. **Retained HUD:** a primitive block per parity, re-chained between two
   markers when the layout key is unchanged.

### Round 3: retained packets (`runtime/retained.hpp`)

- Each static quad owns two fixed primitive slots per parity. The per-frame
  path allocates downward from the end of the capacity, while the retained
  path allocates upward from the start; `allocate` checks that they do not cross.
- A rebuild writes the constant words (command and colors, page UVs, palette,
  and page) once, **only for the parity being built**, because PsyQo transfers
  the opposite parity by DMA while `frame()` runs (`GPU::flip` in
  `third_party/nugget/psyqo/src/gpu.cpp`). Each parity stores its own key.
- Key: geometry pointer, active texture bank, baked-color table, entity
  generation, complete material, lighting flags, world basis, and
  `LightingRenderer::signature` (a hash of the loaded lighting matrix, colors,
  ambient light, and tint). Point-light smoothing rebuilds during transitions
  and stabilizes afterward.
- Each frame: classification, three screen-coordinate words per triangle,
  and ordering-table linking. Fog rewrites colors while affecting the quad
  and once more when it stops (`fogged` bit per parity).
- Exclusions: skeletal meshes and scrolling-UV quads; mixed objects execute
  both loops. Memory: 28 bytes per scene quad plus keys per slot, in `.bss`.
  Ranges are not reclaimed until the scene resets.
- Project option: Project Settings > Engine > Rendering > Retained Packets,
  `rendering.retained_geometry` in `ProjectSettings/project.json`, reaching
  the runtime as `epok::retained_geometry` in `display.hh`. With the option
  enabled, VRAM is byte-for-byte identical to the per-frame path.

### Tested and rejected

- **RTPT in groups of three**, with screen-derived outcodes and lazy camera
  X/Y calculation: no gain in the emulator (vertex work 100 → 104 units).
  On hardware it would save ~8 GTE cycles per vertex; this cannot be verified
  without a console.

### Accepted and documented regressions

- The GTE rounds perspective downward where the CPU truncated toward zero:
  the left/top half shifts by up to one pixel compared with captures before
  round 2. Since then, every change in these historical rounds produced
  identical VRAM.
- Fog is evaluated at Q8 depth: differences of at most one color level.

## 5. Known pitfalls — read before changing anything

- **Runtime sources are embedded in the editor** (`include_bytes!` in
  `src/project.rs`). After editing `runtime/*`, run `rtk cargo build` (and
  `--release` if using that build); otherwise, `--build-psx` compiles the old
  runtime. Add every new header to the list in `src/project.rs`.
- **GCC emits `div`/`divu` for constant divisors.** Inspect disassembly with
  `.tools/mips/bin/mipsel-none-elf-objdump.exe -d -C` after changing hot paths;
  target zero divisions and no `int64_t` work per vertex or triangle.
- **Two primitive parities:** never write primitives belonging to the parity
  that is not being built inside `frame()`.
- **Units:** Q8 camera coordinates for mesh rendering; Q12 for scripts,
  collisions, sprites, blobs, and `camera_project`. Rendering fog uses Q8
  thresholds (`start >> 4`).
- **Deterministic emulator:** identical builds produce identical medians and
  identical `vram.bin`; a difference of one unit is real.
- **Emulator ports:** each project sets `web_port` in `Local.epokconfig`;
  Epok's existing integration tests use 8077. Tests launching emulators must
  run sequentially if they share a port, and the measured project must be
  closed in the editor. The later streaming tests use 8092 and 8093.
- `tools/profile_runtime.py` defaults to `examples/sample-game`; pass
  `--project <path>` for another project. `--detail` and `--validate-gte`
  recompile `main.o` with additional work (their timings are not release
  timings) and remove `main.o` when finished.
- Host tests (`tests/runtime/*.cpp`, through `verify_spatial.py`) compile
  with MSVC or g++ and do not execute the GTE; they cover the clipper,
  colors, fog, outcodes, and collisions.
- `Epok/docs/*` remains generic, in English, and without scene-specific
  figures by the user's documentation policy; reference figures belong in
  knowledge documents such as this one. Markdown knowledge notes belong in
  `knowledge/`; documentation and README files retain their normal locations.
- `artifacts/` is in `.gitignore`: profiles exist only on disk.

## 6. Measurement and verification workflow — checklist for each change

From `Epok`, using the `rtk` prefix required by the user's environment:

```powershell
rtk cargo build
rtk proxy python tools/profile_runtime.py --project <project> --output artifacts/performance/<name>
rtk proxy python tools/compare_vram.py artifacts/performance/<baseline>/vram.bin artifacts/performance/<name>/vram.bin --png artifacts/performance/<name>/vram-diff.png
rtk proxy python tools/profile_runtime.py --project <project> --output artifacts/performance/<name>-detail --detail
rtk proxy python tools/profile_runtime.py --project <project> --output artifacts/performance/<name>-validation --validate-gte
rtk proxy python tests/runtime/verify_spatial.py
rtk proxy python tests/runtime/verify_sprites_particles.py
rtk proxy python tests/integration/verify_hud.py
rtk cargo test
```

Historical acceptance criteria: median `frame_scanlines` no greater than the
baseline; identical `vram.bin` or explained and documented differences;
`gte_validation_errors` = 0; the test project's movement test PASS with zero
dropped steps and triangles; count background-colored pixels inside the play
area when changing projection or vertices (this detects seams);
`retained_triangles` and `retained_rebuilds` consistent (everything retained,
zero rebuilds in steady state). For a project-option A/B comparison: toggle
the value in `ProjectSettings/project.json`, rebuild, measure, and restore it.
Profiles from the three historical rounds are in `Epok/artifacts/performance/`;
the reference at that point was the folder ending in `retained-final`.
Section 10 adds the stricter current regression checks.

## 7. Prioritized work and proposed designs, with current implementation status

### 7.1 Precomputed visibility — implemented with plane intervals; experimental

The implementation uses conservative masks in mesh-local space, covering full
intervals of plane directions and offsets. It queries the current object/camera
matrix and retains runtime frustum tests and clipping. It supports movement,
rotation, FOV changes, and inherited transforms without stale lists.
`precomputed_visibility` is configured under Engine > Rendering and remains
disabled by default. It implements neither occlusion nor precomputed polygon
ordering. It uses 54 direction intervals and 32 distance intervals; cached
combined masks and exact bounds results are reused while geometry and the
object-to-camera matrix remain unchanged. Unsupported ranges fall back to the
ordinary conservative tests. The option remains Experimental and may lower FPS.

The corner/center design below is **historical and was not implemented**:
it did not guarantee coverage of intermediate positions and arbitrary camera
orientations.

Crash Bandicoot technique: precompute off-console which geometry is visible
from each camera position. This targets two measured components: `setup`
(69 units to test all chunk bounds) and the part of the polygon loop that
rejects invisible quads using outcodes.

- Editor: at export, discretize the camera's reachable volume (configured
  in the scene or inferred from collision bounds) into a cell grid. For each
  cell, run the same conservative bounds test as the runtime at its corners
  and center, union the results, and emit a per-cell list of visible chunks
  for each object. Format in `scene.hh`: cell origin and size, dimensions,
  and packed lists in `inline constexpr uint16_t`.
- Runtime: before the chunk loop, calculate the cell from the camera position;
  inside the grid, iterate only listed chunks and skip the bounds test;
  outside the grid, use the existing path. Objects whose transforms are
  animated by scripts do not enter the lists.
- Invalidation: any edit re-exports; FOV is part of the precomputation.
- Project option in Rendering, generated in `display.hh`, with an entry in
  `docs/settings.md`, enabled by default if a gain is measured.
- Verification: identical VRAM with the option enabled and disabled throughout
  the movement route; a counter in `PerformanceStats` for chunks skipped by
  the list.

### 7.2 GTE orientation and depth — retained path

Replace `classify`'s screen-space area with NCLIP and average depth with AVSZ3
after an RTPT of the three vertices. RTPT produced no emulator gain because
GTE latencies are not modeled; the actual gain would be on hardware. Proceed
only if an emulator gain is measurable or a console is available.

### 7.3 Simulation — 47 units per frame, ~24 per step

Half the cost comes from two transform scans per step (one inside
`move_and_slide` and another at the end of the step). Options compatible with
direct writes: compare a hash of 36 bytes; skip the closing scan if no script
executed code since the last scan (a flag in the binding loop); restrict the
scan to slots with scripts or affected descendants. Measure `world_scanlines`
and `world_syncs`.

### 7.4 Vertex projection — 99 units, ~196 cycles per vertex; RTPS takes ~15

The remainder is memory traffic: vertices in RAM without a data cache, and
20 bytes written per `ProjectedVertex`. Ideas: compact `ProjectedVertex`;
decide fog per chunk using the camera-space box rather than per vertex;
manually schedule `Unsafe` GTE reads (hardware only); use the 1 KB scratchpad
for the active vertex batch.

### 7.5 PsyQo level — small, unmeasured

Compile PsyQo with `-O2` instead of `-Os` (`third_party/nugget/common.mk`);
use `OrderingTable<512, Safe::No>` because the emitter already bounds depth;
use `Unsafe` GTE register access. Do not modify PsyQo (a third-party submodule):
change only how it is used.

### 7.6 Hardware validation and GPU measurement

There is no independent GPU/presentation measurement. In 480i, `GPU::flip`
waits for field parity on the console but not in PCSX (`pcsx_present()`), so
the emulator omits that cost. Before treating 30 FPS as established on hardware:
measure on a console or with verifiable GPU timing, and expose a presentation
wait counter if PsyQo permits it.

### 7.7 CD page streaming — experimental for editable geometry

`GEOMETRY.BIN` contains 64 KiB pages of vertices and faces; the executable
retains bounds, counts, links, and offsets. A pool of 2–8 pages uses LRU and
pins when the archive exceeds its capacity, validates per-page checksums,
and can preload the next candidate only when an unused slot and the CD are
available. GPU buffers are sized by a per-frame triangle budget independent
of total map size. The retained pool supports streamed geometry through a
bounded allocation. Textures, scripts, collisions, and baked colors remain
resident; skeletal meshes retain their existing path.

When the complete archive fits, page N has permanent slot N. Checksum-verified
payload pointers can then remain bound to immutable object geometry, and
retention uses the resident path's eager per-object rebuild for each changed
parity. A changed geometry root is validated and bound again; a partial or
failed binding is never exposed for drawing. Larger archives retain sequential
cursor views: consume each view before advancing or releasing the cursor, with
one page pin shared across consecutive chunks and no lease per chunk. Generic
leases keep explicit borrow tracking. Descriptor validation is reused only for
an unchanged, validated immutable chain.

All pool storage is initially zero and belongs in BSS, so empty page buffers
reserve RAM without being serialized inside the PS-EXE. Initial warmup loads
the active scene's distinct pages before the simulation clock starts if they
fit; otherwise it rejects the batch before reads. Successful warmup suppresses
nearby-page lookahead until a demand miss or scene change. Real loading costs
remain visible in the streaming and warmup counters.

Controls: Engine > Streaming, `streaming_geometry` (off),
`streaming_pool_pages` (4), `streaming_triangle_budget` (4096), and
`streaming_prefetch` (on). Streaming is not automatically enabled in ForestTest.

A required read can stall the frame and drop simulation steps because of the
existing catch-up limit. Geometry shares the controller with XA: it pauses
playback, then restarts the track from the beginning; explicit stop requests
and scene changes remain authoritative. In the historical native test, reads
during playback took ~420–432 ms, so ~200 ms of theoretical transfer time is
not a guaranteed total latency. Interleaved audio/data, uninterrupted XA, and
streaming of every resource type are not implemented. Do not promise
uninterrupted traversal. This feature and preloading remain Experimental and
may lower FPS.

### 7.8 Retained-packet follow-up

Reclaim freed ranges without resetting the scene; add an `allocate` rejection
counter to `PerformanceStats`; provide a pool-sizing option; investigate
retaining static sprites and blobs. Streamed retention is currently bounded
by whole-object allocation: oversized objects fall back to per-frame rendering
even if only a small part is visible. Chunk-sized allocation remains future work.

## 8. Repository state and deliverables — historical snapshot

This section preserves the repository state recorded after rounds 1–3. It is
not a claim about the current branch, HEAD, publication status, or working tree.

- Everything was on `main` (`3f89927`, published to `origin`): rounds 1 and 2
  (`a05aaa2`), round 3 (`6bac644`), and two commits from another agent
  (generic English documentation and renaming the profiler to
  `tools/profile_runtime.py`). There were no branches awaiting merge; work
  was to start from `main`. The working tree contained uncommitted changes
  from another session in `runtime/effects.hpp`, `runtime/main.cpp`, and
  `docs/environment-effects.md` (environment effects): preserve them.
- Tools created: `tools/profile_runtime.py` (mesh and retention fields,
  estimated presentation wait, `--detail`, `--validate-gte`, `--optimization`),
  `tools/compare_vram.py`, `tests/runtime/polygon.cpp`, and the
  `EPOK_PROFILE_DETAIL` and `EPOK_VALIDATE_GTE` flags in `runtime/Makefile`.
- A desktop shortcut named "Epok" points to
  `Epok\target\release\epok-editor.exe`.

## 9. Experimental visibility and streaming prototype — September 8, 2026

**Historical status: experimental, with lower FPS in the initial profiles.
Not accepted as a performance optimization.** Results in this section describe
the first prototype; subsequent corrections are recorded in section 10.
Reducing the PS-EXE or preserving the image is insufficient to accept a feature
that worsens frame times. Both options remain disabled by default.

Work performed in parallel and reviewed through cross-checks:

1. Conservative chunk-mask export; frustum and GTE integration.
2. Page, offset, and checksum exporter; data-CD generation even without music;
   removal of geometry payload from the PS-EXE.
3. Bounded pool, pins, LRU, actual reads, preloading, and XA arbitration.
4. Per-frame GPU budget, per-mesh limits, and exclusion of streamed geometry
   from retained-packet reservations in this initial prototype.
5. Settings categories, tests, review, and debug/release builds.

Project Settings > Engine > Rendering keeps Retained Packets (on) and adds
Precomputed Visibility (off). Engine > Streaming groups Geometry Streaming
(off), Streaming Pool Pages (4, 256 KiB), Per-frame Triangle Budget (4096), and
Preload Nearby Geometry (on when streaming is used). Changes apply on rebuild.

Visibility preserved ForestTest's VRAM but did not reject additional chunks
in that map; its query cost was not offset. Streaming also preserved VRAM but
lost the benefit of retained packets for those meshes and added initial CD
loading. This is why both additions remained optional.

Profiles from that prototype (do not directly compare with the historical 414:
the working tree already included new environment effects):

| Configuration | Median scanlines | Frame interval, µs |
| --- | ---: | ---: |
| Before this work, including existing effects | 436 | 28288 |
| Initial prototype's final build, default values | 441 | 28672 |
| Visibility enabled, A/B profile | 459 | 29760 |
| Streaming enabled, steady state | 530 | 34304 |

The default path showed a measured residual cost of five units (~1.1%), so
that prototype was not presented as a ForestTest FPS improvement. Streaming's
benefit was its resident-geometry budget. Differential GTE validation with
both features active produced zero errors (observed maxima: 48 Q12 camera
units and two pixels); its instrumented timings are not release measurements.

Evidence saved in new Epok folders:

- `artifacts/streaming-review/1788837596791027600/`: isolated ForestTest copy,
  A/B profiles, and movement test. Walking, running, diagonal movement, and
  collision PASS; zero dropped triangles. The 60 dropped simulation steps
  occurred during initial loading; the counter stayed at 60 throughout the
  control route.
- `artifacts/streaming/1788838564295257900/report.json`: 2400 faces, four pages,
  and two slots, with 21 byte-for-byte equivalent captures (resident,
  visibility, and streaming, seven states each). Eviction and reload verified.
  The PS-EXE shrank from 428032 to 335872 bytes. Zero read errors or missing
  chunks.
- `artifacts/streaming-xa/1788838686455250000/report.json`: six phases of XA
  and actual geometry reads, three pages/two slots, playback, interruption,
  restart, explicit stop, and scene reload during XA. Zero errors/timeouts.

Also completed: 105 passing Rust tests (three ignored because of external
requirements), the host runtime suite, warning-free Clippy analysis, and native
RPG and HUD acceptance. Tests were added for intervals, page integrity, pending
DMA, pins, and the absence of page reservations/CD traffic when disabled.

Main files: `src/mesh_visibility.rs`, `src/streaming.rs`,
`runtime/visibility.hpp`, `runtime/streaming.hpp`, `runtime/streaming_pool.hpp`;
internal contracts and debt in `knowledge/streaming-implementation.md` and
`knowledge/performance-implementation.md`; user controls in `docs/settings.md`. Section 8 describes the earlier state: this delivery is
in the working tree and does not imply a new commit or push.

Historical relationship: visibility and ordering precomputation are described
in [Andy Gavin, Making Crash Bandicoot, part 3](https://all-things-andy-gavin.com/2011/02/04/making-crash-bandicoot-part-3/)
and [Dave Baggett](https://news.ycombinator.com/item?id=9277704).
Paging and packing are described in
[Teaching an Old Dog New Bits, part 3](https://all-things-andy-gavin.com/2011/03/28/crash-bandicoot-teaching-an-old-dog-new-bits-part-3/).
This work adapts chunk selection and geometry loading; it does not reproduce
Crash's occlusion, precomputed polygon ordering, or entire memory system.

## 10. Prototype corrections and acceptance criteria

Visibility and streaming are labeled **Experimental — may lower FPS** in
Project Settings, with their controls grouped under Rendering and Streaming.
Both options remain off by default. Reduced memory usage does not justify
presenting an FPS regression as a completed optimization. The final stationary
ForestTest comparison passes both against the original baseline and when
activating streaming over the current disabled and visibility configurations.
The v19 matched-state moving benchmark improves globally but still fails small
phase timing gates when activating streaming. See
[deterministic validation](performance-deterministic-validation.md); the
stationary pass does not establish complete moving acceptance.
See [the current validation report](performance-validation.md) for complete
metrics, sample counts, provenance and machine-readable checks.

| Configuration | CPU ticks, one / two simulation steps | Median frame interval, µs | FPS from median interval |
| --- | ---: | ---: | ---: |
| Original baseline, existing environment effects included | 408 / 436 | 28288 | 35.351 |
| Current defaults | 382 / 410 | 26624 | 37.560 |
| Geometry Streaming | 381 / 409 | 26560 | 37.651 |
| Precomputed Visibility | 360 / 388 | 23488 | 42.575 |
| Both options | 360 / 388 | 23488 | 42.575 |

These are final v19 production captures, with detailed profiling disabled.
CPU values are medians within equal simulation-step groups. Frame intervals
are quantized, and changing the proportion of one-step and two-step frames can
shift a global median. The linked report includes FPS derived from mean intervals
as well, so these median-based FPS figures must not be confused with average FPS.
The stationary activation comparisons preserve median, p95 and worst sampled frame
time, per-step CPU work and exact stationary VRAM, with no dropped steps or
triangles. Initial loading remains separately visible: two ForestTest pages take
about 1.057 seconds before the simulation starts, with no additional reads or
stalls during the accepted stationary capture.

Corrections after the section 9 prototype:

- Streaming uses retained packets again. When eviction is possible, it
  rebuilds only visible chunks while their page is pinned, with correct
  invalidation of colors and materials shared between the two video parities.
- Consecutive chunks share one page pin and use payload pointers without
  copying the complete descriptor. CD polling is skipped while inactive.
  Sequential cursor views must be consumed before the cursor advances or is
  released; explicit generic leases continue to protect active borrows.
- Visibility uses 54 direction intervals and 32 distance intervals, reusing
  the matrix, combined masks, and exact bounds results while geometry and
  transforms remain unchanged. Mask lookup is deferred until the exact matrix repeats.
  A separate cache stores exact narrow-range basis products after the nine basis
  coefficients repeat, reusing them during translation and invalidating them
  for rotation, scale or FOV changes. Continuously changing bases do not write
  products that cannot be reused. Skeletal geometry bypasses these caches.
  This adds 24 bytes per chunk plus validity bits and two pointers; 168 chunks
  require 4064 additional bytes, discarded when visibility is disabled.
- Frustum testing avoids eight multiplications per chunk when X is
  independent, preserving the same rounding and retaining the full
  calculation for other matrices.
- If all pages referenced by active objects fit the pool, they load before
  the simulation clock starts. Their real cost remains in the general
  counters and is also identified in `streaming_warmup_stats`. Successful
  warmup suppresses nearby-page lookahead until a demand miss or scene change.
- When the complete archive fits the pool, each page has a fixed slot:
  pointers are published only after checksum validation and cannot become
  invalid through eviction. Retention uses the resident path's per-object
  rebuild and avoids per-chunk packet-validity checks every frame. The current
  source also validates and binds each immutable object chain once, publishing
  the root only after every chunk is ready; a changed root binds again, and
  later stream failures block drawing. Direct stable views do not increment
  pool hit counters, which count explicit pin acquisitions.
- Descriptive per-chunk streaming counts run only with detailed profiling.
  Ordinary profiles mark them as not collected; safety, loading and dropped-work
  counters remain active. Timing acceptance uses matching production builds.
- The pool is initialized entirely to zero so it resides in BSS. Empty
  buffers are not serialized in the PS-EXE; a native test checks the linker
  map and verifies that the reservation lies outside the executable's
  loaded image.

The baseline was reconstructed in an isolated folder from commit
`8fe73066bace7a895d2258a31067ab68d7718a79`, incorporating only the three
environment-effect files already modified before this work. Hashes and
provenance are recorded in
`Epok/artifacts/streaming-review/1788837596791027600/baseline-provenance.json`.
The repeat produced 105 samples, a median of 436 scanlines / 28288 µs, and
zero dropped steps or triangles, matching the initial reference.

`tools/compare_runtime.py` rejects increases in median, p95, and worst frame,
dropped steps, and dropped triangles. It checks compatible build options and,
with `--require-vram`, byte-for-byte equality of the native capture. It also
compares CPU work grouped by equal simulation-step counts when both profiles
have at least five samples in that group. Changing the proportion of frames
with one versus two steps therefore cannot hide additional work.
When all groups are covered, the aggregate CPU median is explicitly informative;
group medians, global CPU tails and all frame-time checks remain strict. With
incomplete group coverage the aggregate CPU median remains a gate.
`tools/profile_forest_route.py` records walking, running, diagonal movement,
and collision by phase, including actual ticks and polling-induced overshoot.

Correctness coverage now includes six rendering modes with eleven phases each
in `tests/integration/verify_streaming.py`: resident, visibility, streaming,
resident retained, streaming retained, and streaming with the complete archive
fitting the retained path's pool. The fitting case requires exactly one read
per page throughout all captures, while the oversized-archive case must
demonstrate eviction and reload. These checks also require identical VRAM,
actual retained triangles, no dropped triangles, and BSS page storage outside
the PS-EXE load image. These are correctness checks, not FPS acceptance.
The final 66-phase result is recorded in
`artifacts/streaming/1788853200130266600/report.json`, using matching detailed
builds for exact streamed versus visible chunk counts. Production FPS captures
use detail disabled and report that descriptive counter as not collected. The latest six-phase XA result is in
`artifacts/streaming-xa/1788849247095774600/report.json`. A host emulator startup
crash occurred in an earlier attempt and did not supply gameplay measurements;
the clean repeat completed all 66 phases. The test bootstrap no longer forces
resume while the emulator is still initializing.

Validation of one route does not guarantee all possible scenes: a working set
larger than the pool still needs demand reads, and a mesh too large for the
retention reservation uses the per-frame path. These limits, together with XA
restarting at the beginning of the track, keep the feature experimental.
Physical-console validation remains separate.
