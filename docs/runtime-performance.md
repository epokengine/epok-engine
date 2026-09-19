# PSX runtime hot paths

The runtime keeps authoritative gameplay at its original fixed-step cadence.
These paths remove repeated work; they do not skip collision checks, reduce the
number of actors, simplify meshes, or slow down animation.

## Collision and transform queries

Collision synchronization examines enabled colliders and their complete parent
chains, rather than all visual, camera, audio, and reserved slots. Unparented
colliders use a root fast path. Direct transform writes remain supported: the
nine transform scalars are checked at every synchronization boundary. Changes to
an ancestor leave pending invalidations for children outside the queried subset.
Rendering later performs a full synchronization. A skeletal world-point query
only synchronizes the requested actor's ancestor chain.
Translation-only edits reuse the existing TRS basis instead of recalculating
rotation and scale. Both the full and root-only synchronization paths retain
direct-write detection and revision invalidation.

Local and inverse transforms resolve sine/cosine once per nonzero axis, sharing
them across matrix columns without changing rotation order or fixed-point
rounding. Inverse unit scale bypasses software division; nonunit and zero scale
keep their original behavior. Host tests compare 40,000 rotated matrices with
the previous column-by-column arithmetic.

Use `raycast_batch(queries, results, count, mask, ignore, triggers)` for independent
segments against one world snapshot. `RaycastQuery` holds world-space `origin[3]`
and finite `displacement[3]`; each result corresponds to the same query index.
The runtime synchronizes once, then performs every raycast with the same nearest
hit, generation, layer, trigger, and started-inside rules as individual queries.
Storage belongs to the caller; there is no allocation. Null arrays or zero count
are no-ops. Apply gameplay changes after the batch. Do not replace interdependent
movement/collision operations with a batch: an earlier move can change a later
query's answer.

Trigger synchronization can be skipped only when there are neither active
triggers nor previous overlap pairs awaiting exit events.

## Rendering

Interior retained geometry without fog or animated UVs needs only screen XY
and depth. Its compact eight-byte vertex representation fits 128 vertices in
the 1 KiB scratchpad (versus 51 full clipping vertices). Larger meshes use the
existing RAM workspace. Rigid projection batches stay within each bone range.
Projection overflow, frustum boundaries, fog and scrolling retain their checked
full-coordinate paths. Packet order, per-triangle depth, topology and UVs are
unchanged; no native-quad conversion is used.

The interior and no-fog boundary packet loops, compact projection, software
clipper, collision synchronization, skeletal pose/decode and blob-shadow drawing
are isolated speed-optimized functions.
Boundary submission preserves clipping and triangle order; it only resolves
UV/material state when a triangle actually needs clipping.
The surrounding runtime and gameplay scripts default
to `-O2`; the SDK remains size-optimized. `EPOK_RUNTIME_OPT` and `EPOK_SCRIPT_OPT`
permit build comparisons. The normal executable RAM-budget gate still applies.
Clipping uses native 32-bit division when the numerator fits, with exactly the
same floor rounding and a wide fallback. Its interpolation fraction retains
16-bit precision; tests compare it against the wide reference calculation.

Sprites wholly inside the frustum bypass six no-op clipping passes. Boundary
sprites still run the full clipper. Sprite projection uses native division when
its numerator fits, preserving its original truncation toward zero (distinct
from mesh projection's floor rounding). Randomized tests include extreme script
coordinates and compare both the containment test and projection to wide math.

Immutable skeletal face-material classification is cached by geometry pointer
and cleared on scene entry. Actor tint, texture, lighting and packet invalidation
remain live inputs. Unlit editable faces do not select lights or build unused
lighting matrices.

## Gameplay dispatch and pools

Cooked classes detect inherited no-op actor and component callbacks. Component
masks are refreshed on attachment, removal and reordering. Actor callbacks,
inherited overrides, private/overloaded hooks and legacy descriptor tables remain
conservative when detection is ambiguous. An active component still ticks when
its owner has no actor tick implementation.

Render-only projectile/VFX pools can keep their actors resident and toggle the
sprite's `enabled` field. This avoids repeated whole-level activation scans;
it is not appropriate when actor enable/disable callbacks are part of gameplay.
Cache generation-checked component handles rather than scanning components on
each AI tick.

## Measuring

CPU HSync cost is not the same as delivered FPS. Check the elapsed time between
frames (`time.frame_microseconds`) and record every frame on the target, not only
host-polled samples. Keep GTE reference validation separate from performance
measurement: its additional computations deliberately cost time. Averages alone
do not certify a frame-time budget.

GTE screen validation compares numerical coordinates within the existing
two-pixel reciprocal tolerance for flag-free, depth-valid results. It does not
equate the CPU fallback's conservative screen guard-band boolean with GTE
projection accuracy: the native GTE can output X=-1024 without saturation,
where the CPU fallback marks the point outside its [-1023,1023] guard. Normal
frustum classification and triangle clipping remain unchanged. Diagnostic
builds accumulate mismatch categories so host polling cannot miss a bad frame.
