# World sprites, animation and particles

For a complete layered effect, start with [Using the VFX editor](vfx-editor.md).
This page explains the underlying sprite, animator and emitter components.

Reusable [ParticleEffects](timelines.md) compose Sprite/Emitter layers around one
embedded TimelineAsset. Explicitly reflected Q12 tracks animate layer properties
and events trigger bursts. Existing scene emitter data remains supported.

Add **Sprite** to an Empty object. Select an imported Texture, world size, normalized pivot and pixel atlas rectangle. `[0,0,0,0]` uses the entire texture. Fixed orientation inherits the complete object transform; Upright follows camera yaw while preserving world up; Spherical follows camera yaw and pitch. Sprite transforms include the object's parents and scale. Flip affects UVs only.

Cutout pixels use the PSX zero palette entry. Average, Add, Subtract and Add quarter enable the hardware semitransparency mode; imported alpha determines which texels participate. Sprites and geometry share the 512-entry ordering table. A positive depth bias moves a sprite farther away in quarter-unit steps. Near/far and screen-plane clipping interpolate UVs. A clipped sprite uses one depth bucket to avoid a diagonal seam. The PSX ordering table cannot perfectly sort intersecting polygons, so split occluders and use modest bias where needed.

Lit sprites use the existing ambient and selected directional/point lights through GTE, with the plane's facing normal. Unlit bypasses lighting. Blob Shadow remains an optional separate contact-shadow component.

**Sprite Animator** stores reusable named clips, pixel rectangles, durations and numeric frame events. The inspector edits clips and can slice a horizontal atlas row. Duration must be at least 1/60 second. The runtime advances by simulation time, holds the final frame of a non-looping clip, and exposes completion once:

```cpp
auto& animator = entity().sprite_animator;
animator.play(1);
uint16_t event;
while (animator.poll_event(event)) { /* game-defined event IDs */ }
if (animator.take_completion()) { /* final frame remains visible */ }
```

Events are emitted on entering a frame, including the initial frame when playing. Sixteen events are queued; `dropped_events` records overflow. Pause/resume preserves the current frame. Sprite animation and particle simulation share the runtime's fixed simulation clock and pause state.

**Particle Emitter** supports continuous rate or one burst on start, additional `burst(count)` requests, finite lifetime, velocity spread, gravity, start/end size and color, atlas flipbooks, and local/world simulation space. Local particles follow their emitter's hierarchy. World particles retain their emitted position and direction when the emitter moves. New particles are discarded when a pool is full; existing particles finish naturally. Deleting/reusing their owner invalidates them by generation.

```cpp
auto& emitter = entity().particle_emitter;
emitter.burst(24); // on the next simulation tick
emitter.stop();    // stops new automatic emission
emitter.play();    // restarts automatic emission
```

Limits are 64 authored emitters, 128 particles per emitter, 256 globally, and 2048 sprite triangles per frame. Particle slots are independent of entity slots. `particle_stats` reports alive, spawned, dropped and peak counts; `sprite_stats` reports submitted, culled, clipped and dropped rendering work. Lifetimes and flipbook cells are validated before export. The editor previews an eight-second repeating timeline with a cached fixed-step pool and at most eight catch-up ticks per frame.

These capacities bound storage and work; a full pool can still consume substantial
simulation and drawing time. Profile the intended simultaneous effects together
with scene geometry and HUD at the game's display resolution. Reduce active
layers, emission or concurrent effects when native frame costs or dropped
simulation steps rise. The [native profiler](performance.md) reports effect,
sequence and particle counters alongside frame costs.

ParticleEffect layers share these budgets with scene emitters and sprites. The
runtime admits at most 64 emitter sources per tick, in scene order followed by
effect pool/layer order. `particle_stats.dropped_emitters` counts rejected source
registrations; their due emission requests count as dropped particles. Effects
use generation-checked internal layer handles without allocating scene entities.
See [Timelines](timelines.md) for the current effect lifecycle and implementation
status. The dedicated Particle Effect editor previews the compiled effect using
the actual C++ timeline/effect/particle kernels linked into the editor. Its
eight-second transport supports pause, restart and seeking from the authored
seed. Camera, background and particle/drop counters remain available alongside
the image. Rendering uses the existing host GPU sprite, atlas and blend paths;
neutral lighting replaces scene lighting in this isolated asset view.

The dedicated effect simulation uses 68 raw Q12 ticks per step, matching the
default PSX fixed clock. Seeking replays at most 482 steps. Live playback admits
at most six catch-up steps per UI frame and discards excess wall time. The older
Scene view particle preview still uses its existing Rust simulation; it is not
the effect parity reference. GPU rendering is not pixel-identical to PSX.

The [included example](../examples/rpg-2-5d-demo/README.md) exercises textured geometry, sprite sorting, animation, bursts, collision, input, resource reuse and HUD text. Host runtime tests use the actual PsyQo Q12 type: `python tests/runtime/verify_sprites_particles.py`.
