# Gameplay API

Epok exposes one reflected gameplay catalog to C++, Blueprint and Epok Lua. The
catalog contains instance operations, function-library services, properties,
events, enums and bounded value records. Blueprint's action menu and the generated
`.epok/lua/epok.d.lua` file are generated from the same declarations.

## Service namespaces

Function libraries use lowercase category names with punctuation replaced by an
underscore. Representative calls are:

```lua
local clock = epok.time.snapshot()
local paused = clock.paused
epok.time.set_paused(not paused)

epok.memory_card.clear_staged_payload()
epok.memory_card.set_staged_word(0, 0x00010002)
epok.memory_card.write_staged(0, 4, 0)
```

Available groups cover input, time, math, World 2D conversion, collision, scene
state/projection, resources, sequence/effect playback, Memory Card and UI focus.
The exact project surface is shown by Lua completion and Blueprint search.

`scene.request_with_transition(index, options)` accepts a bounded transition
record: fade durations are clamped to 65,535 ms and loading color channels to
8-bit values. `scene.transition_snapshot()` exposes phase, opacity and audio
gain without returning the native loading-text or image pointers. Builds without
the optional transition renderer still perform the scene request and ignore the
visual options deterministically.

`scene.fog()` and `scene.set_fog(value)` read and write the scene's distance fog
as a bounded `FogSettings` record. Tint channels are clamped to 8-bit values; a
range outside `0 ≤ start_distance < end_distance ≤ 128`, or narrower than one Q12
step, returns `false` and leaves the scene untouched, which is the same range the
editor validator accepts. `end` is a Lua keyword, so the distances are named
`start_distance` and `end_distance`.

`scene.screen_fade()` and `scene.set_screen_fade(value)` read and write the
post-HUD fade to black as a plain amount from 0 (clear) to 255 (opaque). Every
input has one nearest valid amount, so the setter clamps above 255 instead of
rejecting and reports nothing. The getter returns the authored amount, not the
drawn one: what is drawn is the larger of it and a running transition's opacity,
which `scene.transition_snapshot()` already reports, so a read-back can never
disagree with the write that preceded it. The amount survives scene activation,
which makes a fade out, a `scene.request(index)` and a fade back in one sequence.

The `utilities` group adapts the native tween and event-queue kernels as owned
values. `tween_advance` returns the updated tween plus its current value and
completion edge. The four-entry event queue returns a new queue from `emit` or
`poll`, including its deterministic overflow count. Store those returned records
in the owning Blueprint/Lua class; no global “last result” is shared by scripts.

`tween_schedule(from, to, seconds, easing, delay_seconds, loop, legs)` is the full
plan: `delay_seconds` holds the first leg back without shortening it, `loop` selects
`None`, `Restart` or `PingPong`, and `legs` counts the duration spans to play, with 0
asking for no bound. An unbounded plan never reports completion. A ping-pong reverse
leg is the forward curve with the endpoints swapped, so a two-leg plan ends exactly
on `from`. Without a loop mode exactly one leg plays whatever `legs` says, which is
also what a hand-built record with `cycles_remaining` left at 0 does.

`ease(t, easing)` exposes the easing catalogue on its own. It holds seventeen Q12
curves: `Linear`, `SmoothStep`, the In/Out/InOut families of `Quad`, `Cubic`, `Quart`
and `Quint`, and `InCirc`/`OutCirc`/`InOutCirc`. `t` is clamped to `0..1`, both
endpoints are exact, and no curve deviates from its real-valued reference by more
than four raw Q12 units. There are no sine, exponential, elastic, back or bounce
curves: the target has no fixed-point trigonometric primitive. Enumerators are
appended, never reordered, because authored content stores the number.

`vector_tween_start`, `vector_tween_schedule`, `vector_tween_advance`,
`vector_tween_cancel` and `vector_tween_value` tween a `GameplayVector3` with the
same timing, easing and loop semantics, so a position or a tint is one call instead
of three. The record is two endpoints plus a `timing` member that is an ordinary
scalar tween over `0..1`: its value is the interpolation parameter the three
components share, and each component is exactly what a scalar tween over that
component would report.

## Typed receivers and records

Blueprint operations on an actor/component have a typed `Target` pin. Lua Gameplay
profile 2 accepts `value:method(...)` when `value` is a statically typed actor or
component reference. Null, stale and wrong-class references fail without mutation.

Records are snapshots. Their fields can be split/recombined in Blueprint and read
from Lua (`hit.point.x`, `clock.paused`, `state.result`). Reading multiple fields
does not repeat the underlying operation. Full-width playback handles and object
generations remain intact in all execution modes.

## Skeletal queries

`Mesh3DComponent` publishes vertex/bone/clip counts, animation control, playback
state, single/batch vertex sampling and bone sampling. Indices refer to imported
portable order in rigid and baked storage. Choose bind/current pose and model/world
space explicitly. Failure is returned in `SkeletalError`; zero is a valid position.

Query-only remaps, compressed-frame seek tables and baked bone tracks are cooked
from the reachability manifest. A project that never queries skeletal data retains
none of these sidecars.

## Static mesh geometry and streaming

`Mesh3DComponent.geometry_state()` reports `Unavailable`, `Pending`, `Ready` or
`Failed` for its authored static geometry. `request_geometry()` queues a bounded
asynchronous page request; it never waits for a CD seek in the script call.
`sample_geometry_vertex(index, space)` copies one resident model/world position
into an owned `MeshVertexSample`. A pending streamed page returns `Pending`
instead of blocking, invalid indices and transform failures have distinct errors,
and the page remains pinned only while the three coordinates are copied.

Skeletal meshes use `sample_vertex` instead. Static geometry requests share the
existing streaming cache and CD arbitration, with music retaining priority and an
eight-page gameplay request queue.

## Bounded data

- Memory Card payloads: 4096 bytes, addressed as 1024 little-endian words.
- Text chunk access: 128 four-byte chunks in the built-in atlas buffer.
- Collision overlaps and generic object batches: eight returned identities plus a
  total count.
- Skeletal batch sampling: four input indices and four owned results.
- Utility event queues: four events, with explicit count and dropped total.
- Function-library values: at most 32 VM words per value.

See [Blueprint authoring](blueprints.md), [Lua scripting](lua-scripting.md) and
[Memory Card](memory-card.md) for subsystem workflows.
