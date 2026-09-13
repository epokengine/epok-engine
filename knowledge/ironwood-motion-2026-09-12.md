# Ironwood movement timing and visual interpolation

## Scope

The requested work was to investigate slow-feeling running, optimize the runtime
and evaluate motion interpolation. Walking/running speed, scene content,
resolution, collision behavior and the user's Ironwood files were not changed.
The desktop shortcut still targets `target/debug/epok-editor.exe`; the native
game remains an optimized MIPS Release build.

## Implementation

- `TransformCache` compares the nine transform inputs without a transposed
  stack temporary. Root matrices use a direct copy instead of an identity
  composition. Direct writes, invalid parents, cycles, generations and the
  original bounded hierarchy semantics remain covered by host tests.
- Camera view construction also avoids its first identity composition.
- `MotionInterpolation` keeps render-only translation histories and matrices.
  It snapshots the latest fixed step, interpolates with the clock remainder,
  and follows participating entities' parent translations and current bases.
  Gameplay transforms, the collision world and spatial queries are untouched.
- Only active meshes, sprites, cameras, lights, blob shadows and their ancestors
  participate. Screen-space menus and collider-only objects do not need motion
  histories. Unchanged positions skip interpolation arithmetic.
- Scene changes, pause, catch-up-limit frames, generation/reparent changes and
  camera cuts snap. Out-of-tick position writes invalidate that history.
  Scripts should call `reset_motion_interpolation()` after an intentional
  in-tick teleport. The reset survives the rest of that tick.
- Project Settings > Engine > Rendering > Movement exposes Position
  Interpolation, enabled by default and applied on the next native build.
  Disabled builds strip the motion state. Enabled state was 6,116 bytes for
  Ironwood's 71-slot capacity, plus code. There is no heap allocation.

This smooths positions, not rotation, scale, skeletal poses or particle
positions. It adds up to one fixed tick (about 16.7 ms) of visual latency. It is
not an FPS optimization and does not manufacture additional rendered frames.

## Measurements

Owned fixture: `%TEMP%/epok-scene-open-20260912`, ForestClearing, current-scene
Play, disc data, embedded Redux, 640 x 480, all five debug indicators enabled,
retained packets enabled, visibility and geometry streaming disabled.

Controller SHA-256 remained
`951d68787b61a2c5df70abbcf2e5f6cb97705b93d9377294c1e1fc0a8ebe6d76`.
`tools/profile_forest_motion.py` injects the controller's existing test-pad
commands, samples RAM and separately reports simulation and host-wall time.
Each approximately two-second movement phase has 32-33 observed samples;
HTTP polling introduces host overhead and phase endpoints are not exact replays.

| Configuration | Walk frames / wall second | Run frames / wall second |
| --- | ---: | ---: |
| Transform optimization only | 35.69 | 35.84 |
| Initial full-pool interpolation prototype | 33.76 | 34.16 |
| Final selective interpolation enabled | 35.18 | 35.04 |
| Final interpolation disabled | 35.77 | 35.69 |

Final enabled walking was 2.293 units per simulation second, running 4.190
(Q12 rounding of the unchanged 2.3 / 4.2 settings). Observed simulation rates
were approximately 60-61 ticks per host second. No simulation steps or triangles
were dropped. This confirms real-time movement in these emulator captures, not
a guarantee for every host or physical hardware.

The final smoothing cost in this short route was approximately 0.6 FPS compared
with the same optimized runtime switched off. It improves presentation cadence,
not speed or throughput. The earlier ~34 FPS reference was a stationary capture,
so it is not an exact moving-route baseline. No claim of recovering 47 FPS is
made; mesh vertex/polygon preparation remains the major cost.

Raw reports/screenshots are preserved under `%TEMP%/epok-motion-*20260912`,
especially `selected-final` and `off-final`. The `baseline` directory contains a
failed first diagnostic (Redux GET ignored offset/size); it is not a measurement.

## Verification

- Rust: 324 passed, 30 ignored.
- Native host suite: input/time, transform and collision caches, lifecycle,
  streaming, transitions and new motion tests passed.
- Production clock/Fixed multiplication yields identical running distance at
  15, 30, 34, 47, 60 and 120 rendering FPS.
- Interpolation tests cover constant velocity at mismatched cadences, scaled
  parents with arbitrary slot order, authoritative-state preservation, direct
  writes, resets, selection, activation, reparenting, generations and cycles.
- MIPS builds and controlled native movement passed with interpolation on/off.
- Native screenshots inspected: forest, player, geometry, HUD and debug overlay
  remain visible. This is not a claim of a measured subjective smoothness score.

The user's project was untouched. Restore the disposable fixture's startup
scene and remove its temporary interpolation override after diagnostics.
