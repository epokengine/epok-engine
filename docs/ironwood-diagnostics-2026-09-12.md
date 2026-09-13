# Ironwood: Game view, Apply and runtime measurements

Measured on 2026-09-12 using a disposable copy of Ironwood, ForestClearing,
Current scene / CD / embedded PCSX-Redux, with all five debug overlays enabled.
The scene document SHA-256 matched the user's project:
`1C60D2D10A4AF2B275B74D2450EB7F09CDF45781BECF4744EF69A057D6E6A581`.
These are emulator measurements of the stationary initial camera, not hardware
benchmarks, whole-game peak memory or a claim about commercial games.

## Implemented editor fixes

Game view defaults to Fit (4:3), filling one available axis without cropping.
Stretch fills both axes. Integer preserves whole scale factors where possible.
The selector is available directly in Game view and Editor Preferences > Play;
it changes presentation only and works without a native rebuild.

Apply no longer resolves the unchanged startup scene or treats a settings change
as a declaration change. Settings dependency nodes immediately invalidate old
build outputs without scheduling an automatic build. Other real source edits
continue to invalidate/rebuild normally. On the Ironwood fixture, Apply Debug
took 105.27 and 107.79 ms; three seconds of subsequent observation per change
produced no build, scene replacement or script-registry refresh. Debug options
still require compilation on the next Play/Build, since they are compile-time.

## Static memory at 640 x 480

| Category | Bytes |
| --- | ---: |
| Main RAM, including linked reserves/load-address area | 1,475,304 / 2,097,152 |
| Pools / runtime state | 788,152 |
| Texture data in main RAM | 197,908 |
| Engine / SDK code | 180,412 |
| Scene data / geometry | 152,540 |
| Unassigned main RAM for heap/stack | 621,848 |
| VRAM allocated | 820,496 / 1,048,576 |
| Framebuffers within VRAM | 614,400 |
| Texture pixels within VRAM | 187,664 |
| SPU reserve / resident samples | 4,096 / 524,288 |

Rows overlap; do not sum them. Heap/stack peaks are not measured. The ~47.45 MiB
build payload is mostly CD data, not resident RAM. Large fixed allocations are
the game renderer object (379,612 bytes), sprite pool (131,320), entity storage
(88,608), retained geometry (53,132), HUD (42,600) and particles (36,104).
The debug overlay itself accounts for only ~1.1 KiB of packet/state storage.

## Native frame timing

| Mode | Samples | Median frame interval | Median CPU frame work |
| --- | ---: | ---: | ---: |
| 640 x 480, ordinary counters | 97 | 29.760 ms | ~29.120 ms |
| 320 x 240, ordinary counters | 95 | 29.696 ms | ~29.120 ms |
| 640 x 480, extra detail timers | 96 | 30.464 ms | ~29.824 ms |

Ordinary 640 x 480 counters report ~20.096 ms in rendering, including ~6.336 ms
vertex work and ~9.280 ms polygon work. Simulation is ~5.632 ms. World/collision
measurements overlap simulation and must not be added. Around 1,100 GTE vertices
and 616–619 retained mesh triangles are processed at this camera; sprite/HUD
primitives are additional. No dropped simulation steps or triangles were
observed in these short captures. Detail timers themselves add overhead.

Lowering resolution did not materially improve frame time here. The evidence
points primarily to CPU traversal/preparation, not pixel fill rate or exhausted
RAM. Follow-up optimization should profile repeated world/collision queries,
mesh vertex/polygon preparation and scene-sized packet/pool reservations. No
runtime optimization or reduction in the user's resolution/art quality was
applied as part of these editor fixes.

The profiler now accepts `--use-play-profile` so Current scene measurements do
not accidentally include the full scene bank. Raw captures were kept under the
task's temporary `epok-profile-ironwood-*-20260912` directories.
