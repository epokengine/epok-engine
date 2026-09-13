# Native PSX performance

`epok::performance_stats` contains the last completed frame. Timings use the runtime scanline counter (approximately 64 microseconds per unit), rather than Windows or editor FPS. `epok::time.frame_microseconds` measures the interval between runtime frames. GPU rasterization and presentation are not independently measured.

## Counters and timing

### Optional on-screen debug HUD

**Project Settings > Debug** has independent FPS, CPU, Geometry/GTE, GPU Command
DMA and SPU Sample Memory toggles. Apply, then build/Play. They also apply to
exports: turn them off for a clean shipping build. They default to off for both
new and existing projects.

The bottom-left overlay is chained after the game HUD, fades and loading image.
It shares the existing font/CLUT, uses fixed double-buffered packets (no heap),
and allocates no extra VRAM texture. All five enabled use 1,136 bytes of packet/
state RAM in the tested MIPS build, plus initialization guard, code and stack.
A compile-time assertion limits packet/state storage to 2 KiB. All disabled
removes the overlay and its instrumentation; it is small, not zero-cost when on.

* FPS counts rendered runtime frames over at least 30 NTSC vblanks, not host or
  editor frames. It updates about twice a second and initially reads zero.
* CPU measures `Scene::frame` including any waits inside it, but excludes the
  overlay itself and the final `GPU::flip` wait.
* GTE measures the mesh vertex preparation/projection path using the GTE,
  including CPU setup. Lighting and software projection are excluded. It is not
  a hardware busy-cycle percentage.
* GPU measures command DMA observed from CPU-frame entry until completion or
  the overlay sample, whichever comes first. Transfers completed before entry
  are not counted. It is not GPU rasterizer utilization and can include stalls.
* SPU shows the resident ADPCM bank plus the reserved 4 KiB, out of 512 KiB.
  It is neither SPU execution load nor XA streaming-buffer fill.

Time bars fill at one NTSC vblank budget (~263 scanlines / 16.7 ms), with red at
or above budget. They overlap; do not add them. The 16-bit scanline timer wraps
after ~4.2 seconds. Compare performance using matching overlay settings.

### Instrumented counters

The counters cover simulation, world synchronization, collision synchronization, mesh rendering, vertex transformation/projection, polygon processing, shading, fog and emission.

Timings are nested: simulation includes its world/collision queries, collision includes world synchronization, rendering includes vertices and polygons, and polygons include shading/fog/emission. Do not sum nested fields. The 16-bit timer difference wraps after approximately 4.2 seconds per scope.

Work counters report simulation steps, synchronization calls, local/world matrix rebuilds, collider-bound rebuilds and GTE/software vertex counts. A synchronization call does not imply a matrix rebuild.

The general profiler also reads linked `sequence_stats`, `effect_stats`,
`particle_stats` and `actor_stats`. Its `playback` report includes active/alive and peak counts,
completed/cancelled work, skipped bindings/events, dropped effects/particles and
diagnostic overflow. Unlinked services are reported as unavailable. Counts come
from each sampled RAM snapshot; they are not separate completed-frame timers.
The report separates current gauges from cumulative counters, preserves startup
totals, reports capture deltas and flags observed resets and saturation. Sampled
maxima can miss activity between samples; runtime peak counters retain their own
high-water marks. A saturated drop counter cannot quantify further lost work.

`actor_stats` (`epok::actor_stats`, declared in `runtime/actor_tables.hpp`) reports
the Object/Actor/Component model, in the `playback` report under the key `actor`:

| Field | Kind | Meaning |
| --- | --- | --- |
| `alive` | gauge | Registry slots in use (actors, components, the `Level` and the scene script). |
| `peak` | gauge | High-water mark of `alive` since boot. |
| `rejected` | counter | Spawns, component additions and attachments refused by a capacity, domain, cardinality or abstractness rule. |
| `spawned` | counter | Slot acquisitions since boot, including reused slots. |
| `deferred` | counter | Spawn/destroy requests queued because they were made inside a callback. |
| `actors` | gauge | Actors in the current level's table (the scene script is not in it). |
| `components` | gauge | Components owned by those actors. |
| `scene_scripts` | counter | `SceneScriptActor` instances created, one per bank load. |
| `banks_loaded` | counter | Cooked actor tables loaded, so a transition is visible in the counters. |

The values are refreshed once per frame after `audio_tick()` and again on every bank
load and unload. A build whose banks carry no actors still links the symbol, so the
report is `available: true` with an empty level rather than missing; a build from
before this phase has no symbol at all and reports `available: false`.

`EPOK_OBJECT_REGISTRY_CAPACITY` is the cooked slot-table size: the maximum over the
scene banks of (actors + components) plus 32 dynamic slots. Exhausting it increments
`rejected`, never overwrites memory. `epok::level_actor_capacity` (64) and
`epok::actor_component_capacity` (8) bound one level and one actor respectively.

Additional fields report mesh chunks tested/visible, backfaces, clipped polygons, emitted triangles, frame work before and after mesh rendering (`prepare`, `finish`), retained triangles and retained-packet rebuilds. With `EPOK_PROFILE_DETAIL=1`, per-quad shade/fog/emit timers, chunk setup and camera/sprite/HUD timers are also populated. Release builds leave those detail fields at zero. Reading extra timers adds work, so detail-build timings are not release measurements.

The descriptive `streamed_chunks` counter is collected only with
`EPOK_PROFILE_DETAIL=1` (`tools/profile_runtime.py --detail`). Ordinary builds
store `UINT32_MAX` in its existing field; profiler JSON exposes this as `null`
(not collected), not zero streamed chunks. This preserves the runtime ABI.
Read, failure, timeout and dropped-geometry counters remain active independently
of this diagnostic. Compare builds with matching instrumentation; `--detail`
adds timing and counter overhead and is not a release FPS measurement.

GTE validation counters are populated only with `EPOK_VALIDATE_GTE=1`. Validation compares GTE and software results within the runtime's coordinate and screen tolerances; its extra work also affects timing.

## Precomputed chunk visibility

Enable **Precomputed Visibility** under **Project Settings > Engine > Rendering**
to add conservative chunk selection. It is experimental, disabled by default
and may lower FPS. Camera and object transforms remain supported; the setting
does not add occlusion or increase the visible range.

## Retained HUD and geometry

Movement uses fixed 60 Hz simulation independently of rendering. Position
interpolation can smooth entity/camera translations at mismatched rendering
rates; it is a presentation feature, not an FPS optimization. It keeps gameplay
transforms/collision matrices unchanged and adds up to one fixed step of visual
latency. See [Position Interpolation](settings.md) for its switch and limits.

The runtime reuses unchanged HUD and eligible static-mesh packets. Enable
**Retained Packets** in Project Settings to use static geometry retention; it is
on by default. Material, lighting, geometry and transform changes refresh the
packets automatically. Skeletal meshes and scrolling textures use per-frame
packets. Turning retention off provides a comparison and can save RAM.

Streaming bounds retention by the configured triangle budget. Objects that do
not fit use per-frame packets. Monitor native frame timings when adjusting these
options for a game.

## Geometry streaming and frame budgets

**Project Settings > Engine > Streaming** controls geometry CD pages, the RAM
pool, the per-frame triangle budget and preloading. Streaming is experimental,
disabled by default and may lower FPS or stall frames. See
[Geometry streaming](streaming.md) for usage and [Project Settings](settings.md)
for defaults and ranges. Lower triangle budgets save RAM but can omit triangles;
inspect dropped-triangle counters when changing the budget.

## Compilation and measurement

Release builds optimize the runtime's `main.o` with `-O2`; PsyQo and game scripts retain their configured options. `EPOK_RUNTIME_OPT=-Os` permits comparison. Rebuild `main.o` after changing optimization, detail or validation flags.

When using the editor, rebuild it after runtime source changes because it embeds those sources. Standalone C++ exports contain their own runtime snapshot and are rebuilt with their included Makefile.

Use the same scene, input sequence, resolution, build mode and instrumentation for comparisons. Track dropped simulation steps and primitives alongside timings, and inspect rendered frames for regressions. Changing camera position, visible geometry, HUD content or particles can change the measured workload.

Capacity limits are not frame-rate guarantees. Emulator measurements describe the tested workload; physical-console timing and GPU/presentation costs require separate validation. No scene-specific benchmark results are bundled with this guide.
