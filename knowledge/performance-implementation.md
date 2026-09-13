# Performance implementation and acceptance contracts

Internal context for AI agents and engine maintainers. User-facing configuration
belongs in [Project Settings](../docs/settings.md); open work and priorities are
tracked in [Performance technical debt](performance-debt.md).

### Precomputed chunk visibility

**Project Settings > Engine > Rendering > Precomputed Visibility** enables an
additional conservative test before ordinary chunk bounds. It is
**Experimental**, disabled by default and may lower FPS; correctness of masks
does not establish a performance improvement. The exporter builds
deduplicated chunk bitsets for 54 intervals of plane directions and 32 offset
slabs in object-local coordinates. The runtime selects masks using the current
object-to-camera transform, including focal and aspect scaling. Every direction
and offset inside a bucket is included; positions are not sampled at a few
camera points. Object and parent movement, camera rotation and FOV changes
therefore remain supported without invalidating world-space snapshots.
The largest-magnitude normal component is represented exactly after
normalization, while intervals conservatively bound the other two axes. Exact
plane-input comparisons reuse merged masks when the transform is unchanged;
each candidate then needs one bit test. Shared geometry instances replace this
cache when their current transform differs.

An exact comparison of the 12 object-to-camera matrix scalars defers mask
construction until the matrix repeats. A changed matrix records the new key,
invalidates previous masks and bounds decisions, and uses the original bounds
tests without constructing planes or querying masks. The first repeated key
builds its masks; subsequent repetitions record and reuse existing chunk-bounds
decisions in separate validity/result bitsets. Continuously moving matrices and
alternating instances with different transforms therefore use normal culling.
Partial population and skipped chunks remain
unknown until tested; there is no assumption that every chunk ran in an earlier
frame. The two bitsets use eight bytes per 32 chunks.

For narrow chunk bounds, an additional cache stores the exact transformed
center before translation and the transformed extent. Translation-only camera
or object movement reuses these six values and adds the current translation;
it still reruns the complete bounds acceptance test. Rotation, scale, aspect
or FOV changes invalidate the products by comparing all nine basis components.
Recording begins only after those nine values repeat. Continuous rotation and
alternating instance bases keep the original calculation without writing unused
products into the cache; repeating a basis with a different translation is safe.
Each chunk retains the original per-product signed Q12 rounding and padding.
Wide-coordinate chunks keep the original calculation. Storage is zero-initialized
BSS: 24 bytes per chunk plus four bytes per 32 chunks for validity, and two
32-bit pointers per mesh cache. A 168-chunk mesh adds 4064 bytes including those
pointers; disabling precomputed visibility leaves this storage unreferenced.

The masks exclude whole chunks outside the near/side planes; they do not provide
occlusion or a precomputed polygon draw order. Local and camera-space guards
cover GTE quantization. Missing metadata and unsupported plane ranges fall back
to normal bounds tests. `visibility_skipped_chunks` measures work avoided;
`tested_chunks` counts chunk visits after mask rejection, including reused
cached bounds decisions. It does not count only full bounds calculations. Compare CPU timings as
well as counters because per-object mask lookup has a cost. Disabled builds
leave the tables unreferenced so the linker can remove them.

## Streamed retention

Retention uses 28-byte records plus keys per entity slot in static memory.
Ordinary builds reserve `render_capacity / 2` records. With streamed geometry,
`retained_quad_capacity` is the lesser of authored quads plus the spawn reserve
and half the bounded frame capacity. Streamed meshes can retain packets: a
whole object's records are reserved. If the archive exceeds the pool, only
visible chunks rebuild packet data while their geometry pages are pinned.
If the whole archive fits, its pages have permanent slots and retained packets
use the ordinary eager rebuild for each changed parity. Objects that cannot fit the
remaining pool use the per-frame path. This is bounded object allocation,
not chunk-sized retention, and can therefore lose the retention benefit for a
large object with only a small visible portion. Fragment buffers already exist.
Ranges are reclaimed on scene reset; a reused entity slot with different
geometry allocates a new range. Shared record attributes are protected when
keys change across frame parities, including alternating material colors with
fog. None of the retained packet data points into an evictable page.

## Geometry streaming and frame budgets

**Project Settings > Engine > Streaming** controls 64 KiB geometry pages, a
bounded resident pool, the per-frame triangle budget and optional preloading.
Streaming and preloading are **Experimental** and may lower FPS or stall frames;
the streaming master switch remains off by default. Smaller executable files
and correct images do not establish a no-regression performance result.
See [Project Settings](../docs/settings.md) for defaults, limits and persistence.
Editable-mesh vertices and quads are packed on disc while bounds and linked
chunk descriptors remain resident. On the eviction path, culling happens before acquisition.
For archives larger than the pool, `StreamPageCursor.resolve` holds one page pin across sequential chunk views,
avoiding a lease per chunk. The renderer consumes each view before advancing
or releasing the cursor; GPU packets contain copies, so page
eviction cannot invalidate a submitted frame. Streaming can be combined with
precomputed visibility or ordinary frustum tests.

When the complete archive fits the pool, each page has a permanent slot and a
small BSS pointer table publishes it after checksum verification. Before chunk
drawing or retained-packet rebuilding, the renderer validates the object's whole
chain before starting binding I/O, then binds its payload pointers. It publishes
the object root only after every chunk is ready, so partial chains cannot draw.
An eight-byte BSS binding per object slot exists only for whole-archive builds.
Unchanged roots reuse it; replacements validate and bind again. The chunk loop
then uses pointers directly, with no per-chunk resolution, validation, pinning
or LRU updates. Missing pages still use normal reads and stall counters.
A global stream failure blocks objects containing streamed chunks; entirely
resident objects remain drawable and skeletal objects bypass this binding.
Scene reset clears object bindings while loaded pages and their pointer table persist.
Direct views do not increment pool hit counts, which count explicit pin acquisitions.

On the eviction path, immutable descriptors are validated when their chain receives a
retained allocation, and the validated result is reused for that unchanged
chain. Changing chains validates again. Generic calls and objects without a
validated retained allocation still check descriptors normally. Explicit public
leases retain borrow tracking and prevent cursor advancement while live.
Whole-archive object binding validates chains independently of retention.

The page pool has entirely zero initial state and is stored in BSS, preserving
independent instances while avoiding serialized empty page buffers in the
PS-X EXE. This changes executable storage, not the configured RAM reservation.

The authored limit is 7000 triangles per editable mesh; enabling streaming
allows their aggregate level geometry to exceed 7000. Resident primitive and
skeletal geometry keeps its aggregate 7000-triangle limit. The independent
streaming triangle budget limits render-buffer storage, including retained
fragment reservations and triangles produced by clipping. Lowering it saves
RAM but may drop primitives, even when the total level fits on disc. Inspect
the dropped-triangle counter instead of treating the budget as a frame-rate
promise. Collision and baked-light data, textures and other resources retain
their existing memory lifetimes.

After initial script initialization, the runtime gathers the distinct pages
referenced by all active scene objects. If they fit the pool, it loads them
before starting the simulation clock and autoplay. Existing required pages are
pinned before other reads. An oversized set is rejected before any reads and
uses normal demand loading. This initial loading phase is recorded separately
in `streaming_warmup_stats`; its waits also remain in cumulative
`streaming_stats`. It is not a clock reset that hides later gameplay stalls.
Successful warmup sets `stream_warm_scene`, suppressing nearby-page lookahead
until a demand miss or scene change clears it. Demand loading still handles
resources activated or assigned later.

Prefetch checks the next candidate page in spatial chunk order and uses only
free slots. Demand reads own LRU eviction. This is limited lookahead, not a
camera-rail schedule or offline resource-lifetime plan. CD and XA music share
one controller: demand reads may pause active XA, which restarts from the track
beginning afterward; speculative reads do not interrupt XA. Geometry failures
are reported rather than consuming invalid payload pointers.

Track page reads, bytes, evictions, stalls, stall microseconds, errors and failed
chunks alongside render timings. A working set larger than the pool can cause
repeat disc reads every frame. Compare cold loading and warm frames separately.
Long disc stalls can wrap the 16-bit scanline scopes, so the dedicated stall
microsecond counter is necessary for those frames.

## Compilation and measurement

`python tools/compare_runtime.py --baseline BASE --candidate NEW` checks raw
native profile samples rather than trusting summary fields. It rejects higher
median, p95 or worst frame interval/CPU work, dropped simulation steps and
dropped triangles. Optimization overrides, detail timers, GTE validation and
timer units must agree where recorded. `--require-vram` additionally requires
matching `vram.bin` snapshots beside both profiles; use snapshots of the same
deterministic scene state. Without that option, the report explicitly marks
visual validation as unchecked. Measure initial loading and warm traversal
separately and include changing camera paths and eviction/reload.
When every simulation-step group has at least five samples in both captures,
CPU medians are gated within those groups. The overall CPU median remains in
the report with a non-gating explanation because changing the group mixture
can move it without increasing per-step work. Global CPU p95/maximum and every
frame-time and safety check remain strict; incomplete group coverage retains
the overall CPU-median gate.
Run these host tools from the editor repository; standalone game exports carry
their runtime sources and documentation, not the editor's profiling tools.

`python tools/benchmark_visibility.py` runs an optimized **host CPU** experiment
with 168 synthetic chunks, stationary cameras, translation-only and rotating
movement, and shared geometry instances. It uses the renderer's padded AABB
test, exact narrow-product shifts and wide fallback, and exercises the basis
product cache. It compares identical accepted-chunk counts, reports bounds tests
avoided, and takes the median of three 60,000-frame runs. These timings help
identify algorithmic regressions; they are not PlayStation FPS measurements.
The benchmark uses the same policy as the renderer: build masks only after an
exact object-to-camera matrix repeats.
It separately reports the percentage of executed bounds tests that reused basis
products. Host timings remain informational, including any measured slowdown.

## Other renderer implementation details

## Transform and collision reuse

The transform cache compares nine local scalar values at synchronization boundaries. Direct script writes remain supported, including immediate spatial queries within a simulation tick. Changes propagate through parent chains regardless of slot order; generation changes, scene resets and invalid/cyclic ancestry are handled. The runtime still scans transforms because scripts expose mutable fields.

Collider AABBs are reused by world revision and local box values. Layer/mask, trigger, activation and generation metadata remain live. Conservative swept bounds reject unreachable candidates before the Q24 collision calculation; exact endpoint contacts and thin-obstacle sweeps remain supported.

Contiguous entity slots allow pointer validation and index lookup. Collision queries stop at the highest enabled slot, and trigger pairing is skipped when no enabled trigger exists while preserving exit events. Empty particle systems do not request an extra world synchronization per tick.

## Mesh rendering

Object/view composition is shared by an object's mesh chunks. Zero Euler rotations bypass trigonometry. Normal cofactor matrices are shared within an object's draw, preserving non-uniform scale and shear.

The narrow chunk-bounds path detects an independent local X axis once per
object, a common case for pitch-only cameras. It omits the eight known-zero
multiplications while preserving each signed Q12 shift and the original bounds
padding. Other matrices retain the complete calculation. This optimization
applies with streaming and precomputed visibility both enabled or disabled.

Six-bit frustum outcodes discard fully invisible polygons before shading and avoid irrelevant clipping planes. Eligible backfaces and degenerate triangles are discarded before attribute work.

The GPU drawing area handles lateral pixel clipping when projected vertices are valid and spans fit hardware limits: 1023 horizontally and 511 vertically. Near/far intersections and oversized spans retain software clipping. Edge texture interpolation and ordering can differ from software subdivision.

The polygon loop classifies triangles, prepares packed packet attributes and submits GPU primitive words. Material page words are reused until texture or blend mode changes; unlit face colors and modulated packet values are cached per material color. The exporter bakes eight-bit page UVs for editable meshes, with a Q12 fallback for other geometry.

Vertex projection uses GTE RTPS. Local vertices use Q8 coordinates, Q12 rotation remains exact, and row Y is negated for the GPU. Saturation and divider overflow select a software fallback. Each object uses one GTE matrix with chunk origins added to vertices, so vertices shared between chunks receive identical inputs.

For uniformly scaled rotations within the runtime tolerance, lighting rotates its matrix into object space and shades stored normals with GTE NCS. Shear and non-uniform scale retain the cofactor normal transform.

## Retained HUD and geometry

The HUD retains its fragment block per display parity. A layout key includes hierarchy, canvas/rect, image, progress and text state. An unchanged block is re-chained between no-op bookends; changes and scene resets rebuild it.

Static geometry retention is controlled by `rendering.retained_geometry`, enabled by default. Each retained quad owns two fixed fragment slots per parity buffer. The per-frame path allocates from the opposite end of that capacity to avoid overlap.

Rebuilds write packet attributes for one parity while the other parity is transferred. Each frame still classifies triangles, updates screen coordinates and links ordering-table fragments. Rebuild keys include geometry, textures, baked colors, entity generation, material state, lighting flags, world basis and the current light data.

Skeletal meshes and scrolling quads use the per-frame path. An object containing both static and scrolling quads runs both paths. Fog updates packet colors while active and restores retained colors when it stops.

Retention uses 28-byte records plus per-entity keys. Ordinary builds reserve
one record per allocated quad. Streaming bounds retention by the configured
triangle budget; objects that do not fit use per-frame packets. Ranges are
reclaimed on scene reset.
