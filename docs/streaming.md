# Using geometry streaming

**Experimental: a small pool can omit visible geometry; PCDrv reads can stall frames.**
Geometry Streaming is disabled by default.
See [World optimization techniques](world-optimization.md) for how streaming
combines with the other build-time techniques.

The Windows **Scene** authoring preview always loads the complete scene. Runtime
streaming budgets, page residency and baked camera visibility lists do not hide
geometry or move its free camera. These rules are tested in **Game** (the runtime),
not silently applied to Scene. A future streaming-preview simulation must be an
explicit opt-in mode, off by default.

Material **Double sided** is independent of these rules. It renders either
winding in Scene and the runtime without duplicating geometry; it defaults off.

Open **Project Settings > Engine > Streaming**, enable **Geometry Streaming**,
apply the change, choose **CD on demand** or **PC on demand** in the Play profile,
and rebuild. CD launches the disc image. PC reads `GEOMETRY.BIN` from the build
directory through PCDrv, in PCSX-Redux or NOTPSXSerial. The executable alone
cannot supply that external geometry. See [Play](play.md).

| Control | Default | Effect |
| --- | --- | --- |
| Geometry Streaming | Off | Store editable-mesh geometry in external pages |
| Streaming Pool Pages | 4 | Reserve 2–8 pages of 64 KiB each in RAM |
| Per-frame Triangle Budget | 4096 | Reserve capacity for 512–8192 triangles; excess triangles are omitted and counted |
| Preload Nearby Geometry | On when streaming is enabled | Optional lookahead into unused slots; bounded demand scheduling is independent of this switch |

Visibility is configured independently under **Engine > Rendering**. See
[Project Settings](settings.md) for saved fields and reset behavior.

Streaming applies to editable-mesh render geometry and upload-only texture
pixels. Texture bank activation reuses the geometry page pool to upload pixels
to VRAM, then releases those pages; palettes remain in CPU RAM for animation.
Texture layout and page packing are prepared offline; see
[texture cooking](texture-cooking.md). Collision, scripts, audio and skeletal
meshes keep their existing storage paths. Geometry can move with its entity.
Retained packets remain available within the configured budget.

Mesh cooking preserves authored faces and UV corners: it does not subdivide
polygons merely to fit a storage coordinate range. Each chunk uses either
six-byte `int16[3]` or twelve-byte `int32[3]` Q12 positions. Both retain exactly
the same quantized source coordinates, with a Q8-aligned chunk origin. Small
faces retain compact partitions; a face that needs the wider range widens its
chunk. The resident descriptor records the width and validates alignment and
page bounds before binding a payload. Width is not a reduction in precision.
The renderer checks wide bounds before uploading signed 16-bit GTE vectors,
and uses the software transform when those inputs cannot fit safely.

Startup loads active geometry before gameplay when its pages fit the pool. When
the archive is larger than the pool, each frame selects at most `pool_pages`
camera-near pages from resident chunk bounds. A missing CD page is requested
asynchronously; chunks outside the selected or ready set are omitted for that
frame and counted as `stream_budget_skipped_chunks`, not as I/O failures. This
prevents a full level from repeatedly seeking through more pages than RAM can
hold. CD reads wait while XA owns the drive. Explicit gameplay requests still
use their own queue. PCDrv reads are synchronous, so physical serial throughput
must be measured separately. A read failure prevents affected geometry from
drawing and appears in the runtime error counters.

Use matching scenes and build settings when comparing performance. Monitor
frame times, dropped steps, dropped triangles and CD stalls; reducing executable
size does not imply faster rendering.

## Baked camera visibility tracks (experimental)

A scene may contain an optional `visibility_track`: a root camera UUID, sample
duration (`sample_ms`), a stable source-face UUID table, and camera samples with
position, rotation, horizontal FOV, visible face indices and an optional `cut`.
Tracks are authored in the editor as a camera path (below) or imported.
Tracked meshes must list all their faces; missing/stale face IDs fail the build.
Source-face selections survive chunk partitioning and either vertex width.

The compiler emits XOR changes and two resident bitsets (current/next), then
packs geometry by co-visibility. It measures every current/next page union and
rejects a pool smaller than the worst transition. Across a `cut`, the next view
loads separately. Unlike the default camera-near fallback, this path must not
discard selected geometry to fit a smaller pool. Its triangle/packet budget is
still independent and must be verified with runtime counters.

The camera waits for verified current/next reads before continuing along a
continuous segment. Cuts hide tracked geometry until the new view is ready.
Tracks loop automatically, use the simulation pause state and expose
`visibility_track_stats` (sample, advances, waiting frames, loops). FOV and
shortest-path Euler angles interpolate between samples. Set `cull_backfaces`
to false only when the baked data already supplies the intended triangle sides.

Set `static_world: true` to cook the tracked baked scenery in world space.
Each actor's chunks keep their faces, vertex order and visibility bits, but
their vertices are the actor's authored points transformed to world Q12, and
the renderer draws every such chunk with the camera's matrix on the same
camera-relative lattice as the prepared packet order below. Meshes of
different actors then meet exactly, without the one-pixel seams that per-actor
matrices leave where, for example, a wall meets the ground. This is an
explicit opt-in: covered actors must stay static at runtime, since their
runtime transforms are no longer applied, and mesh vertex queries on them
answer in world space only. Off by default; without it the cook is unchanged.

#### Authoring a camera path

**Build > Camera Path...** creates and edits the route a guided camera
follows. For a scene without a track, the panel explains the three steps;
pick a root camera and **Create camera path**. The new track covers every
face of the scene's ordinary editable meshes (Settings > **Refresh tracked
faces** after adding or removing meshes; baked visibility keeps the faces
that still exist) and starts with one key at the camera.

The panel then shows where the workflow stands: **1 Path**, **2 Samples**,
**3 Visibility**, each marked done or pending, and a coloured box naming the
next step with its button (Update samples, Bake Visibility...).

A key is a camera pose (position, Euler rotation), a horizontal field of view
and a time in seconds. Fly the Scene view to a shot and press **Add key**: it
appends a key two seconds after the last one, or, with the playhead between
keys, becomes **Insert key** at the playhead. Between keys the path follows
smooth curves (Hermite, with Catmull-Rom tangents per second; easing in and
out at the ends and at cuts), turning the short way round.

- **Timeline:** keys are diamonds (red where the camera cuts), the white line
  is the playhead. Click or drag to move the playhead, click a key to select
  it, drag it to retime it, right-click it for Look through, Move to Scene
  view, Cut and Delete. The wheel zooms around the mouse; Shift+wheel
  scrolls.
- **Transport:** go to start, Play/Pause, and the eye, which looks through
  the path camera at the playhead in the Scene view (the view cannot show
  roll).
- **Selected key:** previous/next, Time, Field of view, Transition (*Travel*
  moves smoothly from the previous key, *Cut* jumps like a film cut), and
  Look through, Move to view, Delete.
- **Scene view:** the path is drawn as a curve with a numbered camera per key
  (only dots beyond 80 keys); the selected key is orange and the playhead
  camera white. Click a key there to select it.
- **Shortcuts** with the panel focused: Space plays, K adds a key, Left/Right
  select the previous/next key, Delete deletes it.

**Update samples** evaluates the path every **sample interval** (Settings;
100 ms by default) into the track's samples. The first sample is a cut, so a looping
track jumps back to its start, and a sample is a cut where a cut key falls
since the previous one. Updating clears the baked lists and the prepared
packet order: bake visibility again. A track with a route graph
(`navigation`) keeps its imported samples. An imported track without one can
be edited after **Convert samples to keys**: a key every second and at every
cut, plus keys wherever the curve would stray from the samples by more than
a tenth of a runtime camera unit or a quarter of a degree. On the study scene
1,650 samples became 420 keys within 0.66 world units (camera scale 10) and
0.25 degrees.

The `camera_path` and `visibility_bake` MCP tools do the same for agents:
create, convert, set keys, add a key from the view, select a key, update
samples, preview, then start and poll the bake.

#### Automatic visibility bake (experimental)

In the editor, **Build > Bake Visibility...** opens the bake panel. If the
camera track does not follow an edited camera path yet, it says so and offers
**Update samples first**.

- **Quality:** *Balanced* (recommended; draws faces covering at least one
  display pixel), *Maximum quality* (every face that shows at all, no specks,
  more to draw) or *Fastest drawing* (faces of at least two pixels; fewest
  packets, occasional specks).
- **Draw distance** in world units (160), converted to the runtime camera's
  units.
- **Include camera recordings:** bakes from every recording in
  `.epok/camera-recordings/` (see below), with a **view margin** (10%).
- **Generate prepared packet order:** also rebuilds the scene's packet order
  from the new lists.

The **Bake** button runs it on the GPU in the background with a progress
bar, elapsed time and an estimate of the time left; the editor stays usable. When it finishes, the lists (and order) replace the open
scene's, unless the scene changed meanwhile; save the scene to keep them.
The command-line options below give finer control.


`--bake-visibility-report --scene <scene>` computes every sample's visible
face list on the GPU and compares it with the lists the scene holds, without
changing the scene. For each sample it renders the scene from the sample's
camera, with the runtime's projection (horizontal FOV across the display, the
mesh camera's 0.25 to 128 unit range), and from poses up to the next
continuous sample: evenly spaced (`--bake-substeps`, default 4) and then
halved wherever a face of at least 2 display pixels appears or disappears.
A face counts as visible when one of its pixels is not behind an opaque face.
Blended faces and cut-out faces whose covered texels are more than 5%
transparent (`--bake-see-through`) never hide what is behind them. Rendering
runs at 4x the display resolution (`--bake-scale`).

The report also checks the result between samples: at a quarter, half and
three quarters of the way to the next sample it renders the exact visible set
and counts the faces, and their display pixels, that each sample's list lacks.
`--report <file>` writes the JSON report; `--bake-diagnose <sample>` prints
where one sample's differing faces lie.

`--bake-visibility --scene <scene>` writes the bake into the scene: every
sample's list, and, when the scene has a prepared packet order (or with
`--bake-ordered`), a new order generated from those lists. Neighbouring
triangles that share an edge with the same position, UV and colour, and the
same material, become one quad packet. Each sample's packets are sorted back
to front by mean camera depth in steps of `--bake-order-step` world units
(0.5); within a step they keep the previous sample's order, so consecutive
lists stay similar and their stored deltas small. Primitives are numbered
along a Z-order curve through cells of `--bake-order-cell` world units (16),
then by first appearance: the cooker groups 64 consecutive primitives per
projection block, and spatially compact blocks take the fast 16-bit GTE path.
`--bake-min-pixels` (1) lists a face only if it covers at least that many
display pixels in one of the sample's poses; 0 also keeps faces that never
cover a whole pixel, which cost a packet each. Further options:
`--bake-far` (128), `--bake-fill-gaps` (a face hidden for at most 2
samples between visible ones stays listed, 2), `--bake-view-margin` (widen
what recorded cameras see; 0), `--bake-path-margin` (widen the track's own
path poses; 0) and `--bake-reuse-lists` (keep the lists, only rebuild the
order). Near and far are runtime camera units, as the runtime applies them:
a camera actor scaled by 10 makes 128 reach 1,280 world units, so set
`--bake-far` to the distance the game should draw. Where consecutive lists
change a lot, the cooker starts a checkpoint early so each block of list
deltas still fits one page.

`--bake-check-cameras <json>` adds a check against explicit cameras, such as
those a game actually used: `{"cameras":[{"sample","basis","origin",
"row_scale"}]}` in runtime Q12 camera space.

#### Recording the camera while playing

Lists baked from recorded camera poses follow the camera the game really
renders, including aiming rigs. In **Play options**, with Native PC selected
and a scene with a visibility track, enable **Record camera for visibility
bake**: every frame's camera, in the runtime's camera space, and the active
visibility track sample are written as JSON lines to
`.epok/camera-recordings/<scene>-<time>.jsonl`. From the command line,
`--play-native --record-camera <file> [--stop-after <seconds>]` does the same.
Native PC Play runs the same rail and aim controllers as the console with
every page ready, and records the camera the console would render (to within
the console's fixed-point FOV).

`--bake-visibility --bake-recorded-cameras <file or folder>` (repeatable)
adds, for each sample, what every recorded camera of that sample saw; a small
`--bake-view-margin` (0.1) covers poses between recorded sessions. On the
study scene, lists baked from two thirds of the cameras of a recorded demo
lacked 8.6 display pixels per view for the remaining third (0.4 with a 10%
margin, largest single view 14 pixels), at 1,070 (1,168) faces per sample.

A list is only as good as the poses it is baked from. A camera that aims at
a moving target turns away from the recorded path: on the study scene, real
cameras saw about 3,000 display pixels per view that lists baked from the
path alone lacked. A 30% path margin cut that to about 220, at 30% more
faces. Baked from the demo's 1,222 cameras with a 10% margin, a 16-unit far
limit and the defaults above, the study scene's lists hold 902 faces per
sample (858 imported); the prepared pass costs 1.01x the imported lists'
(median over the 403-sample sweep; p10 0.96x, p90 1.12x), and the start view
matches the imported lists' image except for 1,217 pixels of distant
horizon drawn in a slightly different order. Baking from recorded camera poses is the intended remedy, and a
recording adds everything its camera saw: record real play, not idle or
debug cameras (a Native PC session without input, where the camera turns
back towards a player standing at the start, added up to 1,000 faces per
sample).

Without `navigation`, this is automatic diagnostic playback. It does not
reproduce a source game's original polygon ordering. Texture animation,
resource-list scheduling, collision and gameplay need separate integrations.

### Actor-driven route graph (experimental)

`visibility_track.navigation` selects a distinct root World3D actor as `target`.
It contains `start_route`, `routes` (first/count, automatic flag and endpoint
links), and one parameter record per camera sample (unit world X/Z projection
axis, low/high follow band and sample speed). Non-automatic routes follow the
target's position, not input buttons or a looping clock. The target's movement
and collision controller remain independent. Automatic routes retain signed
speed and their graph entry direction.

Every route begins with a full visibility checkpoint; adjacent records are XOR
deltas and support reverse traversal. During interpolation the renderer uses
the union of both sample masks. Continuous links are budgeted even if initially
disabled, because `rail_set_link_enabled(index, enabled)` can unlock them at
runtime. There are at most 512 links; no nearest-route teleport is inferred.

Cooking deduplicates navigation axes and band/speed records. Camera positions
use bounded int16 deltas against resident origins; rotations and FOV use a
dictionary of the existing Q12 integers. No additional rounding is introduced.
Small tracks or dictionary overflows keep the flat representation when needed.
The runtime decodes only requested camera samples, not the entire track.

An explicit `load_cut` on a link denotes a discontinuous camera jump. It is not
a workaround for an oversized continuous transition. Its destination view must
fit the pool, but source and destination need not coexist: before submitting
another frame, the PSX transport pins/loads all destination pages while the
last complete framebuffer remains displayed. The camera commits only on
success. Failure uses the loading/error presentation rather than rendering
evicted source geometry. Loading time is excluded from gameplay catch-up.

The authoring Scene view is unrestricted by default. This schema currently has
no graph authoring UI, automatic occlusion-visibility generator, or
collision/character system.
The experimental controller uses baked rotations and a bounded integrator; it
does not imply exact reproduction of a source game's camera smoothing.

For reproducible emulator checks, headless `--play-psx` accepts
`--input-replay "30000:16,45000:0,50000:64,65000:0"`: milliseconds after emulator
launch and the ordinary 16-bit pad mask (Up=16, Down=64). Times must increase,
fit within one hour, and finish with all buttons released. This uses the
existing virtual-pad bridge, not RAM edits. `tools/profile_runtime.py` accepts
the same option and records rail state and the consumed target position.

### Static draw bake and Scene View simulation

For tracked editable meshes whose face materials are all unlit and have no UV
scrolling, with scene fog disabled, the PSX compiler now emits 32-byte immutable
draw records instead of 72-byte general-purpose mesh faces. Indices, modulated
color, command, packed UVs, blend mode, depth bias and triangle/quad topology
classification are prepared on the PC.
Instance tint and transforms remain dynamic; texture identity is resolved against
the active palette/VRAM layout. Lit, fogged, scrolling and skeletal content keeps
the general renderer. This is an Epok specialization, not a copy of a source
game's renderer or a new automatic occlusion algorithm.

Triangles use a small submission loop. Quads, near-plane crossings and oversized
screen spans retain checked clipping. A bounded cache (up to 128 entries) derives
vertex selections only when a chunk's exact face mask changes. It owns bit values,
not pointers into evictable pages. Cache collisions rebuild; they cannot remove
geometry. Both the normal and specialized projection paths participate in
`EPOK_VALIDATE_GTE` differential checks.

Tracked object roots also carry a cooked contiguous visibility-word span and
chunk count, so rejecting a hidden object does not traverse its descriptor list.

For the tracked subset of each bank, the final archive layout also produces
deduplicated per-sample page lists. Readiness and working-set collection visit
those lists instead of scanning every linked tracked chunk. Interpolated player
navigation combines both endpoint lists; reverse travel, cancelled requests,
cuts and loop boundaries update page cursors alongside the actual face masks.
The page dictionary conservatively retains inactive actors' pages, within the
already validated all-actor budget. In mixed banks, untracked streamed geometry
keeps the generic bounds-based collector and shares the same bounded working set;
its presence does not disable the tracked subset's bake. Mixed banks retain the
full configured pool even when the tracked page lists alone need less memory.
Page CRC checks, pins, asynchronous read ownership
and cut barriers are unchanged; a baked list is not evidence of a completed read.

Projection and clipping derive the side planes from the actual drawing area.
At 512x240, camera X already includes aspect scaling: the CPU fallback uses the
same H=160 as the GTE, and the horizontal clip plane is 8z +/- 5x, not z +/- x.
The precomputed chunk query uses those same planes with a scaled rounding guard.

An entirely baked archive reserves the measured worst view/continuous-edge union,
with a minimum of two pages, up to the configured pool limit. The deterministic
packer tries a tighter layout; failure to meet that tighter target does not relax
the actual working-set validation. Mixed/untracked archives retain the configured
pool. Packet limits and the minimum free-RAM reserve remain independent checks.

Build PSX produces `.epok/build/scene-stream-bake.json` with the exact cooked face
selection, chunk-to-authoring-face mapping, continuous edges and final page
placement. In a tracked map's Scene View:

- **Simulate baked visibility** is off by default. Enabling it loads the latest
  matching bake; the observer camera remains free and does not select visibility.
- **Baked sample** selects a recorded view. **Continuous edge to** shows the union
  used between adjacent samples, including reverse inspection of the same union.
  This is a residency diagnostic, not a route/gameplay controller.
- **Simulate streaming (manual I/O)** is independently off by default. Complete
  individual page reads to inspect the residency barrier. A destination/union
  appears only when all its pages are ready; while loading, the previous complete
  selection remains displayed. The desktop retains full geometry, so this models
  the draw-selection barrier, not PSX RAM eviction, CD seek timing or console FPS.
- **Build PSX bake** regenerates data; **Reload bake** loads it after compilation.
  Source edits invalidate the selection. Missing, stale or malformed data shows
  a warning and the unrestricted map. Opening another map resets simulation.

Disabling simulation immediately restores full authoring geometry. Simulation
state is transient and never modifies or saves actor visibility. Removing geometry
streaming also removes the generated manifest so an obsolete bake is not reused.

### Experimental prepared packet order

An optional scene `ordered_draw` program binds actor/face/corner identities to
native triangle, half-quad and full-quad packets, with one ordered list per baked
camera sample. This is an explicit import/preparation contract, not an automatic
occlusion bake or the default renderer. Validation requires exact visibility
membership, compatible shared corners/materials, static world geometry, no fog,
no backface culling, unlit faces and no UV animation. Covered actors must remain
static at runtime; animation, actor transform/visibility edits and general
lighting do not modify these immutable world-space packets.

Actors covered by the prepared packet order keep no per-face visibility bits in
RAM: the ordered lists, read from disc with their pages, already name every face
drawn and decide page residency. Their ordinary mesh chunks remain as query
copies, loaded when gameplay requests them rather than by camera distance. On
the study scene this frees about 137 KB of resident RAM.

The cooker places 32-byte packet records and Q12 vertices in the same 64 KiB
page, and checkpoint/delta lists in separately addressed page blocks. Current and
next residency includes both geometry and order data. Three bounded decode
buffers preserve the published list during seeks, missing reads and malformed
deltas. A camera commits only after its complete page set and list are ready.
The native pass reads the list in reverse and uses a monotone depth cursor adapted
to PsyQo's descending OT. G4/GT4 reserve adjacent packet slots atomically; only
near/guard-band/span exceptions use the general triangle clipper. A full-range
CPU projection may still emit a native quad when it fits the GPU.

Ordered ABI 2 cooks projection blocks separately from streaming pages: at most
64 consecutive primitives, split when their Q8 bounds exceed a conservative
240-world-unit span. Oversized individual primitives remain whole. A sorted
resident descriptor table (20 bytes/block) binds native record ranges to their
immutable bounds center. This does not change face order, visibility or world
coordinates and does not require one page per projection block.
Storage/vertex-sharing jobs remain fixed groups of 64 primitives. Projection
spans subdivide only descriptor ranges inside each job, so their bounds cannot
change page packing, native record addresses, order deltas or residency sets.

When inputs no longer fit the current origin, the runtime finds their cooked
block and rebases on a camera-relative 4096-Q8 lattice. Compensating the loaded
GTE translation by integral Q12-matrix products preserves the complete MAC
sum, not just its rounded result. It retains this origin while subsequent
vertices fit. Overflow, near-plane, GTE flags and GPU span guards still fall
back safely. No heap allocation or global precision reduction is involved.

On MIPS, the ordered pass prepares each of the first three corners directly in
its vector-register pair (`load_projection_input<0..2>`, six `MTC2`s), avoiding
the temporary packed RAM array. All corners are loaded even on a range failure;
`RTPT` is issued only after every corner and the matrix are valid. An anchor
retry reloads every pair, and two explicit delay instructions precede `RTPT`.
The array-based `begin_projection_triple` helper retains its six `LWC2`s for
callers that already have packed data. After the checked FLAG read, six `SWC2`
instructions store the screen/Z FIFOs directly. The fourth corner still uses
`LWC2`/`RTPS` and signed MAC3. Assembly declares required memory dependencies; it does
not remove range, FLAG, near-plane or span checks. `tools/test_gte_transfer_codegen.py`
cross-compiles the real helpers at O2/O3 and checks register mappings and delay
slots. That audit complements, not replaces, the independent emulator oracle
or eventual physical-hardware validation.

The ordered pass has separate `ordered_draw_stats` counters, published alongside
completed-frame performance counters. `packets`/`quads` count direct native
emissions; `clipped_candidates` counts candidates sent to the fallback, not its
output triangles. `cpu_direct` includes CPU-projected candidates later rejected
outside the viewport. Existing triangle counters remain buffer-slot equivalents.
`origin_changes` counts translation reloads; `anchored_vertices` counts narrow
inputs after rebasing (including subsequent GTE/viewport rejects), and
`anchor_rejections` counts failed rebasing attempts, not omitted primitives.
The 116-byte diagnostic layout also records the committed `sample`, exact Q12
camera origin/full 3x4 basis/row scales and the frame interval. Publication first
sets `frame=0`, copies both ordered/performance blocks behind compiler barriers,
and writes the completed frame ID last. Readers reject zero/mismatched IDs:
the mutable visibility-track counters can be ahead, and matching a sample alone
does not match an interpolated camera. These fields are diagnostics, not an
additional visibility cache. CPU frame costs still use completed scanline timers.
With `EPOK_VALIDATE_GTE=1`, this pass also repeats every narrow-input corner
through independent RTPS calls, checks accepted RTPT screen/depth FIFOs and
reject decisions, and compares MAC coordinates with an integer sum-then-floor
oracle for the uploaded matrix. The CPU screen reference allows the existing
two-pixel reciprocal tolerance. Cumulative `ordered_validation_stats` preserves
failures between profiler polls and records the first sample/token/corner.
`max_world_camera_delta` is informational: full Q12 camera arithmetic has
different quantization from the Q8 GTE input/MAC oracle. It is not a pixel
equivalence result against the source game. This instrumentation is expensive,
is absent from normal builds, and must be disabled for frame-time comparisons.
For rebased inputs the oracle retains the original camera-relative matrix and
offset, independently checking translation compensation rather than repeating
the same rebase calculation.

This path has booted and completed the imported study route in emulation, but
does not yet establish original-game pixel equivalence, navigation/interpolation
fidelity, stable target FPS or hardware timing. Ordinary scenes are unchanged.
Scene View remains unrestricted by default; bake v3 records explicit native page
sets while keeping editable face selections for its opt-in simulation.

### Bounded CD integrity verification

The CD transport exposes only sectors whose DMA has drained, allowing CPU
verification of a completed prefix while the same contiguous 64 KiB read is
still pending. `streaming_read.hpp` adapts PsyQo's sector action; it does not
split the page into extra seek/read commands. The service checks at most 2 KiB
per invocation (32 slices per page), using the identical cooked FNV-1a hash.
The normal camera frame has one service slice and can borrow **one** additional
slice if its current/next required page is being verified. A shared per-frame
permit prevents repeated readiness checks from multiplying that budget;
speculative-only pages receive no extra slice.

Publication requires **all three**: a successful final read callback, drained
DMA, and the complete matching page checksum. A verified prefix is never ready,
pinnable, publishable as a stable pointer, or evictable. Blocking loading,
explicit demand and camera-cut warmup paths pump the service to completion and
are not subject to the normal frame's 4 KiB maximum. A scene change does not
abandon an owned buffer; a late callback after failure drains it without
publishing it. Retry remains unavailable while work owns the buffer. PCDrv
remains synchronous and validates the whole host read before returning.

`streaming_verify_stats` records processed batches/bytes, cumulative scanlines,
and the last/maximum slice cost. Unlike render counters, these also measure
verification work during loading. These are CPU-work counters, not CD latency or
verified-page counts; processed bytes can include a page that eventually fails.
The separate optional `stream_verify_overlap_bytes` symbol counts bytes hashed
before the final read callback, including warmup. It is not a measurement of
physical DMA-active CPU time. Older builds lacking it report unavailable, not
zero. The original five-word verification counter layout remains unchanged.

The 2026-10-05 `sector-demand` emulator capture completes the same first 1650
sample advances as the preceding `output-init` baseline with 357 versus 514
camera-wait frames, 30 read requests and 1,900,544 completed bytes on both sides
(including startup). Its full 260-second run has zero CPU intervals above
33.333 ms, but four display-origin intervals above that threshold. This is not
proof of stable presentation, raster completion or hardware timing. Exact-view
groups show unchanged render cost and a small +262-cycle median CPU overhead;
the benefit is scheduling, not a faster renderer. A fixed-one-slice predecessor
worsened first-loop waits to 564 and was superseded. Detailed identities,
coverage and limitations are recorded in the study contract.

For deterministic baked camera playback, **Preload Nearby Geometry** can fill
spare working-set slots from up to 1.5 seconds ahead (at most 64 samples).
It uses the existing cooked page dictionaries; it does not redo visibility or
expand the pool. Current/next pages and selected untracked actors take priority.
Lookahead stops before cuts, excludes interactive rail branches, and is disabled
for synchronous PCDrv. The current scheduling window is selected at runtime
from baked sets, not yet a separately cooked I/O-deadline plan. The editor's
opt-in manual residency simulation does not model CD/checksum latency.
## Original-game comparison evidence (2026-10-05)

The study now has a headless, read-only reference executable observer under
`research/psx-reference/tools/original_emulator_session.py`. It records
scenery-pass cycles, post-setup GTE registers, frame-finalization intervals and
actual display-origin changes separately. It uses the existing Redux build and
isolated local state, not desktop automation. Five transport tests, five profile
analysis tests and three ordered-list matching tests pass.

Original demo capture `original-runtime-2026-10-05/session-06` measured 1,223
scenery passes: median 11.082 ms, p95 13.859 ms. These are **not** exact-view
comparisons against Epok's full render counter and establish no speed ratio.
The capture also includes loading gaps; its emulated presentation rate is not
a hardware FPS claim. Code at the observation boundaries matches the original
EXE before and after capture (5,292 verified bytes).

`session-07/slst-matches.json` matches all ordered SLST words in four live
snapshots against exported routes: 388 entries at route 0/sample 1, then 817
entries at route 2/sample 0 (three frames). Their native camera positions match
exactly. A different route shares the second camera position but has a different
list; **position alone is not a visibility identity**. The demo jumps between
routes, whereas the study's full-route stress test concatenates all 38 routes.
Neither aggregate FPS nor nearest-position matching establishes equivalence.

The separately supplied private renderer contract records boundaries, hashes,
scope, raw profiles, known observer-version differences and outstanding work.
Main remains unchanged; this stage adds comparison evidence, not a runtime
optimization or a claim of sustained 30 FPS.

### Compile-time Q12 trigonometry (2026-10-05)

The subsequent camera audit verifies that the original constructs camera
matrices at runtime using a 1025-entry quarter-sine lookup and 4096 angular steps.
Epok now generates its own mathematical quarter table at compile time in
`runtime/fixed_trigonometry.hpp`, used for 3D rotation and FOV. This replaces the
startup-generated PsyQo table; it does not bake dynamic camera matrices or alter
streaming policy. All 4096 phases and the linked MIPS table are checked against
the mathematical values. Native camera/projection tests and 846 editor tests pass.

On the first fully identified source view, the maximum linear projection
discrepancy decreases from 2.988 to 0.981 pixels. This is a coefficient audit,
not GTE/raster/pixel equivalence. Later original frames change orientation even
with the same position and visibility list; dynamic camera fidelity is pending.

The normal 260-second route run has no dropped geometry or new blocking stalls,
and all 224,000 texture/CLUT bytes match VRAM. Static RAM is 1,939,004 bytes.
Its 28.160 ms median / 35.776 ms p95 is descriptive only: changed coefficients
leave no exact view/work matches to claim an FPS improvement. There are still
frames over 33.333 ms and camera residency waits. Full evidence and independent
heavy-GTE validation are recorded in the study contract, not conflated with
normal frame-rate measurements.

### Ordered packed-color decoding (2026-10-05)

`baked_ordered_colors` decodes each pair of baked color indices once for the
two ordered corner layouts, after projection/rejection. It preserves scalar
results in 80,000 uniform/indexed test cases, the 32-byte ABI, runtime palette
reads and all packet/UV behavior. An earlier attempt to move scalar color work
into the RTPT wait window was measured slower and removed, including its empty
assembly scheduling barrier.

The retained decoder measures median −3.427% CPU/frame and −4.149% render across
169 exact-camera/work groups, without increasing static RAM or dropping assets.
Both sides use `EPOK_PROFILE_DETAIL=1` and no heavy GTE oracle. These controlled
profiling comparisons are not final-release or hardware FPS claims; the profile
comparator now rejects different detail modes as well as different GTE-validation
modes. The study records a separate run with both optional diagnostics disabled.

That default-mode run completes 2,198 advances (one full study-route loop) with
no dropped geometry, new blocking stalls or streaming errors; all texture/CLUT
bytes still match. Static RAM is 1,938,996 bytes. Observed frame intervals have
27.584 ms median / 33.664 ms p95, with outliers up to 40.320 ms. This still does
not establish stable presentation cadence or physical-PSX performance. The
editor is left on clean Main with unrestricted Scene View and Play stopped.

### Every-frame presentation evidence (2026-10-05)

The study now observes the default build's `GameScene::frame`, `GPU::flip`
and verified GP1(05h) store through host-side headless Redux breakpoints. No
guest code, scene or build changes are required. ELF-derived hooks and all
365,956 instruction bytes are checked, rather than trusting fixed addresses.

A 260-second capture records 10,663 frames/display-origin requests and a full
study-route loop without dropped geometry, steps or new blocking CD stalls.
Across 10,661 consecutive complete-frame intervals, median/p95/max are
23.727/32.992/39.931 ms; 421 exceed 33.333 ms. Vsync deltas vary (one, two or
three); this is not stable two-refresh presentation. GP1 observations are not
a raster-complete/tearing oracle, and aggregate request rate is not hardware FPS.

All 421 slow intervals spend less than 0.017 ms waiting inside flip before GP1;
their CPU work has a 33.921 ms median, with rendering the dominant component.
Only 63 coincide with page-verification counter advancement. Changing VSync
alone will not remove this CPU bottleneck. Ghidra confirms the original has a
different conditional VSync policy. No limiter or content reduction is added.

These are **the same executable bytes** as the previous default-mode sample:
the different every-frame distribution is not a performance improvement over
10 Hz RAM polling. The study contract preserves methodology, hashes, complete
cost analysis, original disassembly and outstanding camera-residency/fidelity
work. Physical PSX validation remains unavailable by design for this stage.

### Packed-screen acceptance fast path (2026-10-05)

`ordered_screen_inside` tests both signed halves of packed SXY words together.
Fully on-screen triangles/quads satisfy the GPU span bounds without scalar
min/max. Near/depth tests remain; crossing/offscreen polygons use the previous
bounds/fallback/clipping path. This is inspired by the original's packed-SXY
rejection sequence verified in Ghidra, not a replacement of its full predicate
or a reduction in visible content. Over 14.6 million native differential checks
cover signed halves, edges, mixed polygons and six viewport sizes.

Across 2,189 exact camera/work groups, default-mode emulated CPU cost decreases
by a median **3.904%**, render scanlines by **4.914%**. The optimization is kept.
The full route has no dropped geometry/steps or new blocking stalls; Play PSX
from Main also boots with the same executable and all 224,000 texture/CLUT bytes
match a new VRAM readback. Static RAM remains 1,938,996 bytes in this build;
instructions grow by 288 bytes, absorbed by data-section padding. Heap high-water
span remains 20 bytes; stack high-water is not measured.

There are still 114 intervals above 33.333 ms in the every-frame run (maximum
38.176 ms), plus camera residency waits and irregular display-origin pacing.
This is a measured CPU improvement, not completion of stable-FPS, full original
visual-fidelity or physical-hardware validation. Detailed provenance and known
limits are in the study contract and `presentation-2026-10-05` artifacts.

### Lower-cost FNV verification (2026-10-05)

The checksum loop now consumes four bytes per iteration and expresses the FNV
prime as unsigned shifts/adds. R3000 O2/O3 emits eight ALU operations per multiply
instead of ten, with byte-safe loads and no helper calls. Hash values, byte order,
4096-byte slice budget, 64-KiB page size and the publication barrier are unchanged.
Native scalar comparisons cover 24,144 input/split/alignment/page cases plus
100,000 multiply values and bit edges. Streaming backend/pool/resident/PCDrv/off
tests pass; a cross-compiler test checks the actual helper's emitted instructions.

In a full 260-second every-frame route, one 4096-byte slice drops from a median
63 to 42 PSX counter ticks (p95/max 64→43). Exact camera/work comparisons have
zero median CPU/render change: verification is occasional, not a render-speedup.
Intervals above 33.333 ms decrease descriptively from 114 to 84, maximum 36.816 ms;
camera waits and irregular pacing remain. No geometry, steps or page integrity
checks are dropped. This is not stable 30-FPS or physical-PSX validation.

Play PSX from Main uses the measured executable, and all 224,000 texture/CLUT
bytes match VRAM. Instructions grow 224 bytes, absorbed by alignment padding
in this build; static RAM and observed heap span stay unchanged. See the study
contract and `hash-fast-route-01`, `screen-vs-hash-fast.json`, `hash-fast-heap`,
and `hash-fast-play-check` for complete captures, hashes and remaining limits.

### Ordered sub-timers are opt-in (2026-10-05)

Per-polygon vertex/polygon timers in the ordered pass now use the existing
`EPOK_PROFILE_DETAIL` switch. Global CPU/render timing and all work/failure/page
counters remain active. A normal frame with ordered candidates marks the two
incomplete breakdowns `UINT32_MAX`/JSON `null`, after all mesh accumulation;
frames without ordered candidates retain ordinary per-chunk timing. Comparators
explicitly refuse a numerical delta for unavailable or partially available data.

The default full-route capture matches 2741 exact camera/work groups: median
CPU −0.433%, render −0.633%, attributable to removing diagnostic overhead rather
than a new rendering algorithm. Geometry, GTE, projection, packets and page
checks remain unchanged. Every-frame display-origin intervals still include
66 above 33.333 ms (maximum 36.557 ms) and 687 camera residency waits: this is
not a stable-FPS completion claim. The study's `ordered-timers-route-01` and
`hash-vs-ordered-timers.json` retain binary identity and complete measurements.

### Direct ordered GTE inputs

The ordered pass now loads prepared corners directly with MTC2, retaining its
full-Q12 streamed vertices, exact camera-relative anchors and CPU fallback.
The current study's 260-second `direct-input-route-02` matches 2331 exact
camera/work groups against the timer-opt-in baseline: median group CPU cost
falls 2.94% and render cost 3.72%. No archive, selection or resource-budget
changes are involved. The separate 350-second heavy oracle completes a route
with 5,585,110 checked corners, zero camera/batch/screen errors, exact MACs and
the existing two-pixel GTE-vs-CPU reciprocal bound.

The experiment moving ordered screen/depth arrays into the existing scratchpad
was measured separately and removed: +1.12% CPU in 2678 matching groups in the
interpreter. This is not a physical-hardware latency conclusion. Captures and
rejected implementation remain in the study's evidence. Normal rendering still
has slow intervals (24 over 33.333 ms, maximum 35.569 ms in the direct-input
capture), plus pending original-fidelity/presentation and hardware validation.
Do not interpret the heavy diagnostic timings or the normal median as stable
30/60 FPS.

Both modes boot through Play PSX: an explicit detail build returns real timings
in 84 coherent samples; the restored default returns unavailable sub-timers in
84 samples, with identical ELF/EXE to the full-route measurement and all 224,000
texture/CLUT bytes matching a new VRAM readback. No runtime flags remain enabled
in the final default editor session.

### Ordered projection output lifetime (2026-10-05)

The ordered pass no longer clears its screen/depth arrays before every face.
An accepted RTPT writes all three entries, RTPS writes the quad's fourth, and
the CPU fallback writes every used entry even when clipping is required.
Triangle paths never read the unused fourth entry. Poison-output tests cover
these contracts, including rejected GTE batches and near/span CPU fallbacks;
no range, FLAG, clipping, ordering or integrity checks were removed.

`output-init-route-01` completes 260 seconds / 11,573 observed frames. Against
`direct-input-route-02`, 2745 exact camera/work groups show median CPU −1.66%
and render −2.11%. Nine display-origin intervals still exceed 33.333 ms, and
692 camera residency waits remain. All seven CPU frames over that threshold
coincide with a 4096-byte verification slice; this is correlation, not a proof
that scheduling alone solves presentation. The checksum budget is unchanged.

Code shrinks 32 bytes without changing static RAM, cooked inputs or page
publication. Play PSX uses the same ELF/EXE and matches all 224,000 texture/CLUT
bytes. The heavy differential oracle from the prior revision was not rerun for
this output-initialization-only change; host output-contract tests and the
normal full-route run do not replace hardware or full raster-fidelity checks.

### Word-sized visibility XOR runs (2026-10-05)

Cooked visibility changes retain the same varint/run format. Contiguous runs
now toggle partial/full 32-bit words, while singleton changes keep their direct
bit operation. Zero-length helper calls are harmless; every actual mask shift
is 0..31. Tests compare 9,312 alignment/length combinations against the old
bitwise oracle, preserve guard/untouched words, apply changes twice, exercise
mixed decoder entries, and retain the 65,536-bit run test.

The camera-aim study's paired 40-second intro captures isolate this change:
`psx-function-costs-01` versus `psx-word-xor-01`. Cooked scene, display and
geometry bytes are identical. All 1,347 paired visibility/candidate records
and all three final 3,516-byte masks match. At 1,327 identical projection
records, every ordered output counter also matches; 20 poses differ with the
runtime's time quantization, so their packet counts are not treated as exact
camera-matched comparisons.

Inclusive navigation peaks fall from 6.5633 to 6.2718 ms; the first route
transition saves 0.4800 ms. Ordered rendering remains 23.6079 ms median.
The linked build adds 544 instruction bytes and 2,048 static RAM bytes after
alignment (1,995,248 total; VRAM unchanged). This is a preparation improvement,
not a demonstrated general FPS gain: 9 versus 8 GP1 intervals exceed 33.333 ms,
with 3 CPU frames over the threshold in both runs. No visibility, pool, geometry
or rendering budgets were reduced. The workload covers two automatic intro
rails followed by a static target, not the full level or physical hardware.
