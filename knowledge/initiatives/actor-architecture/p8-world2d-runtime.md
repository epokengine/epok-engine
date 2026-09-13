# P8 — Real 2D world, runtime half (2026-09-13)

Scope of this task: one new header `runtime/world2d.hpp`, its host tests
`tests/runtime/world2d.cpp` (registered in `tests/runtime/verify_spatial.py`), and a single
line added to `project::runtime_sources()` in `src/project.rs` so the header is staged and
exported with the rest of the runtime. `runtime/object_model.hpp` and `runtime/epok.hpp`
were not modified: consumers include `world2d.hpp` explicitly, so a project without 2D
content pays nothing for it. Nothing was committed.

The editor half of P8 (Camera2D/Sprite2D authoring, the 2D viewport, cooking, Actor2D scene
composition with UI) is deliberately out of scope here; see *Deferred* below.

## 1. Units and ranges

`Transform2D` already lived in `object_model.hpp`; this header fixes what its fields mean.

| Field | Unit | Valid range | Notes |
| --- | --- | --- | --- |
| `position[2]` | world units, Q12 (`epok::Fixed`) | \|p\| ≤ `world2d_position_limit` = 8192 | Same scale as the 3D world. A world unit is **not** a pixel. The limit keeps every intermediate inside int64 without saturation and squared distances below 2^51. |
| `rotation` | degrees, Q12 | any value (wrapped into one turn) | Same convention as the 3D `Transform::rotation` components. Positive = counter-clockwise (+X towards +Y). |
| `scale[2]` | factor | (0, `world2d_scale_limit`] = (0, 64] | Zero and negative are rejected, not silently mirrored: a mirrored sprite is a rendering flag, not a degenerate matrix. |
| `draw_order` | int16 | full range | Combined with a layer and a creation index by `draw_key_2d`. |

`transform2d_valid(const Transform2D&)` reports validity (rotation can never fail);
`transform2d_clamp(Transform2D&)` clamps into the range instead, pinning a non-positive
scale to the smallest representable positive Q12 value (raw 1) rather than to zero.

## 2. Projection convention (`Camera2D`)

```
screen = viewport_center + R(-camera.rotation) * (world - camera.position) * k
k = camera.zoom * pixels_per_unit_2d
```

* `pixels_per_unit_2d = 32`: one world unit is 32 screen pixels at zoom 1, so ten world
  units cross a 320 px display. This is the density the 2D fixture is authored at.
* The world Y axis points **up**, the screen Y axis points **down**; the Y component is
  negated by the projection. `screen_to_world` is its exact inverse up to Q12 rounding, and
  falls back to the camera position when the zoom is zero (no inverse exists).
* Screen coordinates are Q12 **pixels with sub-pixel precision, relative to the display
  origin**, not to the viewport origin. `viewport[4]` is `{x, y, w, h}` in pixels and
  defaults to the whole display (`display_width`/`display_height` from `display.hh`); a
  smaller viewport moves the projection centre with it.
* Coordinates far off screen saturate rather than wrap (\|world\| = 8192 at zoom 64 exceeds
  the Q12 range); a saturated coordinate is still rejected by every clip test.
* **No dependency on the 3D camera**: neither `camera_project` nor `projection_focal` nor
  the GTE is involved, and the 2D path works with no 3D camera in the level at all.

### Trigonometry

`psyqo::Trig<>` is a runtime object owned by `main.cpp` whose angle unit is a turn
fraction, not degrees, so depending on it would drag a service into a header the host tests
compile standalone. Instead `epok::sin_degrees` / `cos_degrees` use a `constexpr` table of
257 Q12 samples of sine over [0°, 90°] (256 intervals of exactly 1440 raw degree units =
0.3515625°), linearly interpolated and mirrored into the other three quadrants. No floating
point survives into the image — only 1028 bytes of rodata. Error against the ideal sine
stays below one Q12 unit (2.4e-4) and the cardinal angles 0/90/180/270 are exact, so a 90°
rotation is bit-exact and round-trips.

## 3. Hierarchical 2D transforms

`Affine2D { Fixed m[2][2]; Fixed t[2]; }` with `identity()`, `point()`, `direction()`,
plus free functions `compose(parent, child)` (parent × child) and
`local_matrix(const Transform2D&)` (translate × rotate × scale, i.e. a point is scaled,
then rotated, then translated).

`world_matrix_2d(leaf, resolve, out)` is a template over a resolver callback
`const SceneComponent2D* (ObjectId)`, so it never touches `Level` internals and host tests
pass a plain array lookup. Rules:

* Depth is bounded by `world2d_depth_limit = 32` (root included); a longer chain returns
  `false` and leaves `out` untouched.
* Cycles are rejected by pointer identity along the visited chain (self-attachment
  included).
* A parent that fails to resolve simply ends the chain, exactly like a detached component.
* No recursion and no allocation: the chain is collected into a 32-entry stack array and
  composed root-down.

## 4. Draw order

`draw_key_2d(int8_t layer, int16_t draw_order, uint16_t creation) -> uint64_t` packs
layer (bits 32-39, biased), draw_order (bits 16-31, biased) and the creation index
(bits 0-15) into one monotonically comparable 40-bit key; a higher key means drawn later,
i.e. on top. The creation index is the tie-breaker that makes the order reproducible across
frames.

`sort_draw_order(uint16_t* indices, size_t count, key)` is a bounded, allocation-free,
**stable** insertion sort over an index array. Insertion sort is the right shape here: the
draw list is a few dozen sprites, it is almost sorted between frames, and it needs no
scratch memory. A null pointer or a zero count is a no-op.

## 5. Collider semantics and limits

```cpp
struct Collider2D {
    enum Shape : uint8_t { Box, Circle };
    Shape shape; Fixed half_extents[2]; Fixed radius; Fixed offset[2];
    uint32_t layer; uint32_t layer_mask; bool trigger; bool enabled;
};
```

* **Boxes are axis-aligned.** The owner's rotation is deliberately **ignored** for
  collision, mirroring the conservative world AABB of the 3D collider: a sprite turned 45°
  keeps upright collision bounds. Oriented narrow-phase boxes are an explicit extension.
* Circles are rotation invariant, therefore exact.
* `offset` is applied in world axes (unrotated) on top of the owner's world position.
* `layer` is the bit (or bits) the collider occupies; `layer_mask` is the set of layers it
  interacts with. A pair interacts only when **both** masks accept each other, the same
  rule as the 3D world. Query functions take a separate `mask` argument, also as in 3D.
* Overlap predicates come in two flavours: the strict one treats touching as **not**
  overlapping (open intervals, matching `epok::aabb_overlap`), the inclusive one
  (`inclusive = true`) treats exact contact as an overlap. Triggers use the strict rule so a
  mover that stops exactly against a wall does not enter it.
* `collider2d_overlap` dispatches box/box, circle/circle and box/circle (both orders) in
  world space and uses the real distance, so a diagonal corner is correctly rejected even
  when the bounding boxes overlap.

### `CollisionWorld2D<Capacity, PairCapacity = 64>`

Mirrors the shape of the 3D `CollisionWorld`: fixed capacity (≤ 1024, static-asserted), no
allocation, conservative world AABBs for the broad phase and for sweeps, exact shape tests
for triggers.

| Member | Behaviour |
| --- | --- |
| `sync(entries, count)` / `set(index, entry)` / `disable(index)` / `clear()` | `sync` replaces the whole live set; slots past `count` become disabled and therefore produce trigger exits on the next update. `ColliderEntry2D` carries the collider, its world position, a generation and an `active` flag (owner activation folded in without touching the authored `enabled`). |
| `overlap(aabb, out, capacity, mask, ignore, triggers)` | Broad phase only (circles through their AABB). Returns the **total** number of matches; only the first `capacity` indices are written. |
| `raycast2d(origin, displacement, mask, ignore, triggers)` | `displacement` is the complete segment, not a direction. Slab test in Q24 (as in 3D, to avoid Q12 time rounding tunnelling thin obstacles); returns the nearest hit with fraction, point, axis normal and `started_inside`. The truncating Q24 fraction can put the contact point one raw unit short of the exact plane. Triggers are transparent unless requested. |
| `move_and_slide_2d(index, displacement, mask)` | Axis-separated resolution: X is applied first and clipped against every blocking box whose Y span overlaps, then Y is applied against the updated box. A mover blocked on one axis keeps sliding on the other. Triggers and filtered layers never block. A pre-existing overlap is reported through `unresolved_overlap` rather than silently resolved. The entry's stored position and bounds are updated with the resolved displacement. |
| `update_triggers(callback)` | `TriggerPhase` Enter/Stay/Exit per pair through the existing `epok::TriggerEvent`, exactly once each. The pair set is frozen before any callback runs, so user code that moves or destroys objects cannot change the events of the frame it is reacting to. |

**Pair table capacity.** `PairCapacity` (default **64**) bounds the remembered overlapping
trigger pairs. A frame producing more pairs increments `dropped_trigger_pairs` and drops
the extra pairs — they receive neither Enter nor Exit — instead of overflowing the table or
growing it. This is the same controlled failure the 3D world uses, with a smaller default
because 2D levels carry fewer simultaneous trigger pairs and the table is a third of the
object's footprint at capacity 32 (12 B/pair).

**Exit guarantees.** A pair whose generation changed (slot reused by another owner) exits
and re-enters in the same frame; a disabled or removed partner always produces its Exit
exactly once, and never a second time.

## 6. Picking

`pick_2d(camera, screen_xy, entries, count, out_index)` converts the screen point to world
space and returns the **topmost** candidate containing it: highest draw key wins, ties go to
the lowest index so the result is stable. `Pick2DEntry { Aabb2D box; uint64_t key; bool
enabled; }` keeps the helper free of any dependency on the collision world, so the editor
and the runtime select the same object for the same click. Circles are picked through their
bounding box; a caller needing exact circle picking filters the result with
`collider2d_overlap`.

## 7. Measured sizes (host, clang++ x86-64)

Printed by `tests/runtime/world2d.cpp::size_report`. The MIPS image uses 32-bit pointers,
but none of these types contains a pointer or a vtable, so the numbers should carry over
except for alignment padding.

| Type | bytes | Type | bytes |
| --- | --- | --- | --- |
| `Camera2D` | 24 | `Collider2D` | 36 |
| `ColliderEntry2D` | 52 | `Aabb2D` | 16 |
| `Affine2D` | 24 | `Transform2D` | 24 |
| `SpatialHit2D` | 32 | `MoveResult2D` | 24 |
| `Pick2DEntry` | 32 | sine table (rodata) | 1028 |
| `CollisionWorld2D<32>` (64 pairs) | 2976 | `CollisionWorld2D<32, 256>` | 5280 |

`CollisionWorld2D<32>` is ~2.9 KB of static state: 32 × 68 B of entries, 64 × 12 B of
remembered pairs and ~40 B of bookkeeping (measured: `<32,1>` 2224 B, `<32,32>` 2592 B,
`<32,64>` 2976 B). Each remembered pair costs 12 B, so halving `PairCapacity` to 32 saves
384 B if a fixture needs it.

## 8. Validation

| Check | Result |
| --- | --- |
| `clang++ -std=c++20 -fsyntax-only -Wall -Wextra -fno-rtti -fno-exceptions -Wno-unused-parameter` on `runtime/world2d.hpp` standalone (stub `display.hh` + `EASTL/functional.h`) | clean |
| `tests/runtime/verify_spatial.py` (now includes `world2d.cpp`) | pass, all 24 suites |
| `cargo build --locked --bins` | pass |
| MIPS build / export / emulator | **not run** — no `mipsel-none-elf-g++` on this machine (see p0-baseline) |

Host test cases in `tests/runtime/world2d.cpp`:

| Case | Covers |
| --- | --- |
| `units_and_validation` | position/scale limits accepted and rejected at the boundary, `transform2d_clamp`, rotation always valid, exact sine/cosine at 0/90/180/270/−90/450 and interpolated values at 30/45/60 |
| `camera_projection` | pixels-per-unit and the screen-Y-down convention, world↔screen round trip at zoom 1/2/0.5 with an offset camera over nine world points, exact 90° camera rotation and its round trip, non-default viewport centre, zoom-zero fallback |
| `hierarchy` | translate+rotate composition, parent scale applied to the child offset and to the child's own scale, `Affine2D::point` through the composed matrix, detached component, cycle and self-attachment rejection with `out` untouched, depth 32 accepted / 33 rejected, unresolvable parent ends the chain |
| `draw_order` | key ordering across layer/draw_order/creation including negative layers and orders, full sort against an expected permutation, stability with four equal keys, null/empty no-ops |
| `overlaps` | box/box, circle/circle and box/circle (both orders) including touching edges under both the strict and the inclusive rule, diagonal corner rejection where the AABBs still overlap, collider offsets, AABB semantics for rotated owners |
| `rays` | nearest hit among three boxes (not the first slot), contact point and normal, misses pointing away / too short / parallel, `ignore`, origin inside a box, triggers transparent unless requested |
| `movement` | stops exactly at a wall on X while sliding the full amount on Y, no further motion when pushed into the wall, unblocked motion away from a touching wall, symmetric negative direction, triggers and filtered layers never block, `unresolved_overlap` reported, invalid index is a no-op, `overlap` counting with and without `ignore` |
| `triggers` | Enter once, Stay on later frames, Exit once when separating, exact shape test overruling the broad phase, re-entry, disabled (destroyed) partner exit exactly once and no repeat, generation change producing Exit+Enter in one frame, two non-triggers never pair, layer masks required in both directions, pair-table exhaustion (6 pairs into a capacity of 2 → 2 enters and `dropped_trigger_pairs == 4`) |
| `picking` | topmost by draw key across layer and draw_order, disabled candidates skipped, empty space and null array, picking that follows camera zoom and pan |
| `size_report` | the size table above |

## 9. Deferred

* **Editor half of P8**: Camera2D and Sprite2D authoring, the 2D viewport and its gizmos,
  drag/drop and the mode selector. This header only provides the runtime contract those
  tools must agree with (units, projection, draw key, picking).
* **Cooking**: no component class is introduced here. The reserved IDs for
  `Sprite2DComponent`, `Camera2DComponent` and `Collider2DComponent` (design.md section 2)
  are still unused; wiring `Collider2D` into a `Collider2DComponent` and synchronizing
  `CollisionWorld2D` from the `Level` belongs with that work.
* **Actor2D scene composition with UI**: `Actor2D` still has only its `SceneComponent2D`
  root; combining a 2D world with a UI canvas in one level is a later step.
* **Rendering**: sprite submission, rotation of the quads and the ordering table are
  untouched; `draw_key_2d` defines the order the renderer must honour, nothing more.
* **Physics extensions**, as the plan states: oriented boxes, rigid-body dynamics,
  tilemaps and navigation stay out. `move_and_slide_2d` is axis-separated AABB resolution,
  not a solver.
* **MIPS numbers**: the sizes above are host measurements; the real budget must be captured
  on a machine with the SDK.
