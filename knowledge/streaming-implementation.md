# Geometry streaming implementation and technical debt

**Experimental: this feature and its preloading option may lower FPS and cause
frame stalls.** They have not met a general no-regression performance requirement.
Development and performance validation are ongoing. Geometry Streaming remains
off by default; keep it off for workloads whose native comparisons regress.
Smaller executable files and correct images alone are insufficient acceptance.

Enable **Geometry Streaming** in Project Settings > Engine > Streaming
(`streaming_geometry`, off by default).
The exporter moves immutable editable-mesh vertex and quad payloads into
`GEOMETRY.BIN`, a sequence of 64 KiB pages on the generated CD image. Chunk
bounds, counts, links and page offsets remain in the executable so the eviction
path can reject invisible chunks before reading their geometry. The whole-archive
path described below binds each object's complete chain before drawing. Animated transforms
are supported: the immutable mesh payload does not change when an entity moves.
Skeletal meshes, textures, collision data, scripts and audio resources retain
their existing storage paths. See [Project Settings](../docs/settings.md) for the page
pool, per-frame triangle budget and optional nearby-page preloading controls.

Retained packets use a bounded record pool, capped at half the frame triangle
budget. A streamed object's allocation covers all its quads. When the archive
exceeds the page pool, packet data is rebuilt only for visible chunks with their
pages pinned. When the entire archive fits, pages cannot be evicted and retention
uses the ordinary eager rebuild of the object's packets for each changed parity.
Objects that do not
fit the remaining retention pool use the ordinary per-frame rendering path.
Consequently, a very large single mesh may lose the benefit of retained packets
even when only a small part is visible; chunk-sized retention allocation remains
further work. Page streaming and retention budgets do not reserve records for
an unbounded entire world.

`streaming_pool_pages` selects 2–8 resident page slots (default 4): 128–512 KiB
for page payloads. Required reads reuse the least recently used unpinned slot.
For archives larger than the pool, the renderer uses `StreamPageCursor.resolve` to consume chunks sequentially
through one held page pin. Each returned view must be fully consumed before
the next cursor advance or release; its pointers must not escape that scope.
This path creates no per-chunk lease. Generic callers can use `StreamMeshLease`
for an explicit borrow, which prevents the cursor from advancing while live.
GPU packets copy their data; neither DMA destinations nor pinned pages can be
evicted. Speculative prefetch
uses unused slots only and gives XA playback priority. Scene changes preserve
the cache because page IDs are global across the scene banks in one build.
Renderer lookahead is compiled out when the whole archive fits the pool; that
path warms active geometry and rebuilds retained objects eagerly.

Pool payloads and metadata have zero initial state, including empty-slot and
lookup-hint representation. The complete pool therefore belongs in BSS: it
reserves RAM without shipping zero-filled page buffers inside the PS-X EXE.
Each pool instance still owns its own payload and metadata.

When the entire archive fits the configured pool, page N has a permanent slot N.
A small BSS pointer table publishes each page only after its read and checksum
succeed. Before chunk drawing or any retained-packet rebuild, the renderer
validates every descriptor in the object's chain, then binds its mutable
vertex/quad pointers. Validation completes before binding starts any I/O. The
object root is published in `StreamObjectBinding` only after the complete chain
is ready; failed or partial bindings are never drawn. Bounds, topology, page IDs
and offsets remain immutable. An unchanged root reuses its binding; replacing
the root validates and binds the new chain again.

This cache uses eight BSS bytes per object slot on PSX, only in builds where the
whole archive fits. The chunk loop reads the bound pointers directly, without
per-chunk resolution, validation, pinning or LRU updates. Missing pages still use
normal demand reads and stall counters. A later global stream failure blocks
objects containing streamed chunks, including mixed resident/streamed chains;
entirely resident objects remain drawable and skeletal objects bypass binding.
Scene reset clears the object-binding cache while preserving loaded pages and
the global page-pointer table. Generic leases remain supported.
Pool hit counts measure explicit pin acquisitions, so direct stable views do not
increment them. Archives larger than the pool retain the bounded LRU cursor path.

At initial startup, after script initialization, the runtime gathers the pages
referenced by all active scene objects. If their distinct pages fit the pool,
it loads them before starting the simulation clock and autoplay. Pages already
resident are pinned first, so loading another required page cannot evict them.
This is an explicit initial loading phase; its real CD waits remain in the
cumulative streaming counters. If the active working set exceeds the pool, the
warmup is rejected before any reads and ordinary demand loading remains in use.
Later gameplay stalls are never erased by resetting the simulation clock.
After a successful scene warmup, `stream_warm_scene` suppresses nearby-page
lookahead while all required pages remain available. A demand miss or scene
change clears that state and permits normal prefetch checks again.

Missing required geometry is loaded synchronously while CD callbacks continue
to run. This can stall a frame; it is not a promise of seamless traversal on
arbitrary camera paths. `streaming_stats` exposes reads, bytes, stalls, cumulative
stall microseconds, errors, timeouts and XA interruptions. The fixed-step clock
retains its existing catch-up limit; a long demand read can increase
`time.dropped_steps`, including initial demand loads when the warmup set does not
fit the pool. Pool counters expose
hits, misses, evictions and failed reads. Compare camera traversal with and
without streaming, including worst-frame stalls, rather than treating reduced
executable size as a frame-rate improvement.

`tools/profile_runtime.py` records linked `streaming_stats` and
`streaming_warmup_stats` for every sampled frame. Its report has separate
`streaming` and `streaming_warmup` summaries with the first snapshot, last
cumulative snapshot and `during_capture` delta. Warmup reports attempts, pages,
reads, stall microseconds, rejected batches and failures. Warmup waits are a
subset of total streaming waits, so do not add the two totals together. Sampling
begins after completed startup frames: an initial wait can appear in the totals
even when `during_capture` is zero. Frame medians and timing distributions remain
separate; warmup duration must not be presented as steady rendering FPS.

Use matching baseline and candidate native captures with
`python tools/compare_runtime.py --baseline BASE/profile.json --candidate NEW/profile.json`.
The check fails for increased median, p95 or worst frame time, CPU work under
the [step-group comparison policy](performance-implementation.md#compilation-and-measurement), dropped simulation
steps or dropped triangles. When every sampled step-count group has at least
five samples in both captures, per-group CPU medians replace the aggregate
CPU-median gate; CPU p95/maximum remain strict. Incomplete coverage retains the
aggregate gate. Measure both initial loading and warm
camera traversal, including changes of direction and eviction/reload. A passing
capture establishes the tested workload only; it cannot guarantee every possible
camera path, storage device or hardware configuration.

XA and geometry use one controller and one ISO parser. A required geometry read
pauses XA and preserves the current playback request; after the read, playback
restarts **at the beginning of the track**. Explicit stop/replacement requests
remain authoritative. Exact-sector XA resume and interleaved audio/data packing
are not implemented. Use resident geometry when uninterrupted XA playback is
required.

Required reads have a ten-second watchdog. A failed or timed-out stream stops
subsequent geometry acquisitions and reports failure instead of exposing partial
data. A pending DMA slot remains owned until its callback completes. Metadata
range and binary layout checks protect page resolution. Each successfully read
page must also match its exporter-generated FNV1a32 checksum before it becomes
resident, catching corruption and stale disc data even when file sizes match.
This checksum is an accidental-corruption check, not cryptographic authentication.
On the eviction path, the renderer validates a geometry chain's immutable ranges
when assigning its retained allocation and reuses that result while the same
chain remains assigned. A replacement chain is validated again. Unretained
geometry on that path and generic cursor or lease calls retain descriptor
validation by default; callers must not mark arbitrary mutable descriptors as
already validated. Whole-archive builds validate both retained and unretained
object chains through the binding step described above.

The host runtime suite (`python tests/runtime/verify_spatial.py`) checks page
ownership, LRU eviction, pin lifetimes, asynchronous prefetch, demand stalls,
pool exhaustion, transport failure, same-size payload corruption, timeout cleanup
and disabled-build overhead. It also checks sequential cursor views, explicit
borrow protection and validation of untrusted descriptors.
Its CD transport is simulated. The native acceptance tests
`python tests/integration/verify_streaming.py` and
`python tests/integration/verify_streaming_xa.py` additionally exercise generated
CD images in PCSX-Redux, exact VRAM comparisons across six rendering modes,
more pages than pool slots, eviction/reload, retained packets enabled and disabled,
and an archive that fits all four pages in four slots without reloading. Eleven
phases per mode exercise alternating material
colors with fog across both frame parities, XA interruption/restart, explicit
stop and scene lifecycle.
The geometry test's `assert_stream_pool_in_bss` guard checks both the MIPS map
and the PS-X EXE load range, so zero-filled page buffers cannot silently become
part of the serialized executable again.
They create isolated timestamped projects and use ports 8092 and 8093. Run
`cargo build` first. Physical-console validation remains separate.
