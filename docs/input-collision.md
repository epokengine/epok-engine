# Runtime input, time and collision

The Inspector's **Add Box Collider** attaches a serialized collider to any entity.
The selected collider is drawn as a green world box (amber for a trigger).
Center and half extents are local coordinates. The runtime transforms the box
through its complete hierarchy, then uses the enclosing world AABB, including
inherited scale, rotation and shear. Rotated boxes therefore collide against
their conservative bounds rather than an oriented narrow phase.

Local extents must be at least one Q12 unit (1/4096); validation also checks the
transformed size and ±512 world bounds. The same data is written to the exported
C++ scene banks. A collider can be attached, enabled, disabled or removed through
`entity().add<epok::Collider>()`, `get<Collider>()` and `remove<Collider>()`.

## Controller and simulation

`epok::input` reads both ports through PsyQo AdvancedPad. Port arguments are
zero-based and default to the first port. `held`, `pressed`, and `released` take
a `epok::Button` (`Cross`, `Start`, `Up`, `Left`, etc.). A disconnection releases
all held buttons. Edges are buffered until a simulation update runs and delivered
once, including when several fixed updates catch up during a rendered frame.

The runtime measures PsyQo GPU time in microseconds and advances simulation at
60 Hz. Catch-up is bounded to eight steps per rendered frame; excess steps are
dropped and counted by `epok::time.dropped_steps`. Q12 `dt` alternates between
68 and 69 raw units, totaling exactly one second every 60 simulation steps.
Animation and particles advance during simulation, independently of rendering.

`epok::time.set_paused(true)` pauses simulation. `Behaviour::frame_update` still
runs once per rendered frame and receives unscaled elapsed microseconds. Use
`input.frame_pressed()` and `frame_released()` there for pause menus. Paused input
edges are discarded so resuming does not replay gameplay actions.

```cpp
void frame_update(epok::Transform&, uint32_t) override {
    if (epok::input.frame_pressed(epok::Button::Start))
        epok::time.set_paused(!epok::time.paused());
}

void update(epok::Transform&, epok::Fixed dt) override {
    using namespace epok;
    Fixed movement[3] = {0.0, Fixed(-6.0) * dt, 0.0};
    if (input.held(Button::Left)) movement[0] -= Fixed(3.0) * dt;
    if (input.held(Button::Right)) movement[0] += Fixed(3.0) * dt;
    const MoveResult result = move_and_slide(entity(), movement);
    if (result.grounded) { /* project-specific locomotion */ }
}
```

## Queries and triggers

All query coordinates and displacements are in world space. Queries refresh
transforms so one Behaviour sees movement performed by earlier Behaviours in the
same simulation tick. Movement transforms the resulting displacement back into
the mover's local parent space.

| API | Result |
| --- | --- |
| `raycast(origin, displacement, mask, ignore, triggers)` | Nearest finite-segment hit, fraction, point, normal, and inside flag. The displacement is the entire segment, not a unit direction. |
| `collider_aabb(entity, output)` | Current active collider bounds. Returns false for a missing or disabled collider. |
| `overlap(box, outputHandles, capacity, mask, ignore, triggers)` | Total matching count; writes only up to capacity. Strict touching is not overlap. |
| `query_ground(entity, distance, mask)` | Nearest floor within nonnegative distance below the collider's full footprint. Triggers are ignored. |
| `move_and_slide(entity, displacement, mask)` | Applies a swept movement, up to three sliding planes and bounded overlap recovery. Reports displacement, contacts, grounding and unresolved overlap. |
| `hit_entity(hit)` | Generation-checked entity handle for a ray or ground hit. |

Queries default to all layers. The `ignore` entity defaults to null and raycasts
exclude triggers by default. Overlap includes triggers by default. Movement and
ground automatically ignore the querying entity and intersect the query mask
with its collider mask. `layer` and `mask` are 32-bit bitfields, not layer indices.

Triggers never block movement. `Behaviour::on_trigger(EntityHandle other,
TriggerPhase phase)` receives `Enter`, `Stay` and `Exit` at simulation boundaries.
Both participants receive the callback when they have active scripts. For a
destroyed participant the exit handle is invalid; always check `other.get()`.
Trigger pairs require both colliders' layer/mask filters to match. At most 256
overlapping pairs are retained; excess pairs are dropped deterministically.
Triggers detect overlaps at tick boundaries, so a shape that crosses entirely
through a trigger in one tick should use a segment query for that fast interaction.

Collision provides static boxes, queries and kinematic movement. It does not
implement impulses, rigid bodies, capsule geometry, slopes or mesh triangles.
Initial penetration is resolved for at most eight passes; callers can inspect
`unresolved_overlap` when geometry cannot be resolved. No collider means movement
returns that flag without moving the entity.

## Verification

`python tests/runtime/verify_spatial.py` compiles and executes the collision
kernel with the actual PsyQo fixed-point type. It covers thin walls, sliding,
parallel rays, ground footprints, penetration recovery, masks, trigger phases,
slot generations, controller edges, pause, timer wrap and bounded catch-up.
It also executes lifecycle and scene-service tests, including 100 bank switches,
asynchronous XA owner quarantine and teardown callback reentrancy.

`cargo test collision` covers serialization defaults, round trips, malformed
colliders, inherited transforms and generated component initialization.
