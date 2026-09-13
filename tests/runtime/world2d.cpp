// Host contract tests for the real 2D world runtime (actor-architecture P8).
// Compiles runtime/world2d.hpp through object_model.hpp/epok.hpp, exactly as a cooked
// build would; nothing here stubs the 2D code itself.
#include <cassert>
#include <cstdint>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/world2d.hpp"

using namespace epok;

namespace {
Fixed q(double value) {
    const double scaled = value * 4096.0;
    return Fixed(int32_t(scaled < 0 ? scaled - 0.5 : scaled + 0.5), Fixed::RAW);
}
double real(Fixed value) { return value.raw() / 4096.0; }
bool close(double a, double b, double tolerance = 1.0 / 4096.0) {
    const double d = a - b;
    return (d < 0 ? -d : d) <= tolerance;
}
Collider2D box_collider(double hx = .5, double hy = .5, bool trigger = false) {
    Collider2D c;
    c.shape = Collider2D::Box;
    c.half_extents[0] = q(hx);
    c.half_extents[1] = q(hy);
    c.trigger = trigger;
    return c;
}
Collider2D circle_collider(double radius = .5, bool trigger = false) {
    Collider2D c;
    c.shape = Collider2D::Circle;
    c.radius = q(radius);
    c.trigger = trigger;
    return c;
}
ColliderEntry2D entry(const Collider2D& collider, double x, double y, uint32_t generation = 0) {
    ColliderEntry2D e;
    e.collider = collider;
    e.position[0] = q(x);
    e.position[1] = q(y);
    e.generation = generation;
    return e;
}
using World = CollisionWorld2D<16, 8>;

// ---- 1. units and validation ---------------------------------------------------------
void units_and_validation() {
    Transform2D transform;
    assert(transform2d_valid(transform));  // defaults: origin, scale 1
    transform.position[0] = q(8192.);
    assert(transform2d_valid(transform));
    transform.position[0] = q(8193.);
    assert(!transform2d_valid(transform));
    transform.position[0] = q(-9000.);
    assert(!transform2d_valid(transform));
    transform2d_clamp(transform);
    assert(real(transform.position[0]) == -8192.);
    transform.scale[1] = q(0.);
    assert(!transform2d_valid(transform));
    transform.scale[0] = q(65.);
    assert(!transform2d_valid(transform));
    transform2d_clamp(transform);
    assert(real(transform.scale[0]) == 64. && transform.scale[1].raw() == 1);
    assert(transform2d_valid(transform));
    // Rotation is never invalid: it is wrapped, in both directions and beyond a turn.
    transform.rotation = q(-4000.);
    assert(transform2d_valid(transform));
    assert(sin_degrees(q(0.)).raw() == 0 && real(sin_degrees(q(90.))) == 1.);
    assert(sin_degrees(q(180.)).raw() == 0 && real(sin_degrees(q(270.))) == -1.);
    assert(real(cos_degrees(q(0.))) == 1. && cos_degrees(q(90.)).raw() == 0);
    assert(sin_degrees(q(450.)).raw() == sin_degrees(q(90.)).raw());
    assert(sin_degrees(q(-90.)).raw() == -4096);
    assert(close(real(sin_degrees(q(30.))), .5, .001) && close(real(cos_degrees(q(60.))), .5, .001));
    assert(close(real(sin_degrees(q(45.))), .70710678, .001));
}

// ---- 2. camera projection ------------------------------------------------------------
void camera_projection() {
    Camera2D camera;
    assert(camera.viewport[2] == int16_t(display_width) && camera.viewport[3] == int16_t(display_height));
    Fixed world[2] = {q(1.), q(0.)}, screen[2] = {}, back[2] = {};
    world_to_screen(camera, world, screen);
    assert(real(screen[0]) == display_width / 2. + pixels_per_unit_2d && real(screen[1]) == display_height / 2.);
    world[0] = q(0.);
    world[1] = q(1.);
    world_to_screen(camera, world, screen);  // world +Y is screen up
    assert(real(screen[0]) == display_width / 2. && real(screen[1]) == display_height / 2. - pixels_per_unit_2d);
    // Round trip at zoom 1, 2 and 0.5, with an offset camera.
    camera.position[0] = q(3.);
    camera.position[1] = q(-2.);
    for (double zoom : {1., 2., .5}) {
        camera.zoom = q(zoom);
        for (double x : {0., 2.5, -4.25})
            for (double y : {0., 1.75, -3.5}) {
                world[0] = q(x);
                world[1] = q(y);
                world_to_screen(camera, world, screen);
                screen_to_world(camera, screen, back);
                assert(close(real(back[0]), x) && close(real(back[1]), y));
            }
    }
    // 90 degree camera rotation: exact, and still a round trip.
    camera.zoom = q(1.);
    camera.rotation = q(90.);
    world[0] = q(4.);
    world[1] = q(-2.);
    world_to_screen(camera, world, screen);
    // camera at (3,-2): offset (1,0) seen through a camera turned 90 deg CCW is (0,-1)
    // in view space, i.e. one unit down the screen.
    assert(real(screen[0]) == display_width / 2. && real(screen[1]) == display_height / 2. + pixels_per_unit_2d);
    screen_to_world(camera, screen, back);
    assert(real(back[0]) == 4. && real(back[1]) == -2.);
    // A viewport that is not the whole display moves the projection centre with it.
    camera.rotation = q(0.);
    camera.viewport[0] = 40;
    camera.viewport[1] = 20;
    camera.viewport[2] = 80;
    camera.viewport[3] = 60;
    world[0] = camera.position[0];
    world[1] = camera.position[1];
    world_to_screen(camera, world, screen);
    assert(real(screen[0]) == 80. && real(screen[1]) == 50.);
    // Zoom zero has no inverse: screen_to_world falls back to the camera position.
    camera.zoom = q(0.);
    screen_to_world(camera, screen, back);
    assert(back[0].raw() == camera.position[0].raw() && back[1].raw() == camera.position[1].raw());
}

// ---- 3. hierarchical transforms ------------------------------------------------------
void hierarchy() {
    SceneComponent2D nodes[40];
    auto resolve = [&](ObjectId id) -> const SceneComponent2D* {
        return id.index < 40 ? &nodes[id.index] : nullptr;
    };
    nodes[0].transform.position[0] = q(2.);
    nodes[0].transform.rotation = q(90.);
    nodes[1].transform.position[0] = q(1.);
    nodes[1].attach_parent = ObjectId{0, 1};
    Affine2D world = Affine2D::identity();
    assert(world_matrix_2d(nodes[1], resolve, world));
    assert(real(world.t[0]) == 2. && real(world.t[1]) == 1.);
    // Parent scale multiplies the child's offset and its own scale.
    nodes[0].transform.scale[0] = q(2.);
    nodes[0].transform.scale[1] = q(2.);
    nodes[1].transform.scale[0] = q(3.);
    assert(world_matrix_2d(nodes[1], resolve, world));
    assert(real(world.t[0]) == 2. && real(world.t[1]) == 2.);
    Fixed local[2] = {q(1.), q(0.)}, point[2] = {};
    world.point(local, point);  // child +X, scaled by 3*2 and rotated 90 degrees
    assert(real(point[0]) == 2. && real(point[1]) == 8.);
    // A detached component is its own local matrix.
    assert(world_matrix_2d(nodes[0], resolve, world));
    assert(real(world.t[0]) == 2. && real(world.t[1]) == 0.);
    // Cycles are rejected, self-attachment included.
    nodes[0].attach_parent = ObjectId{1, 1};
    Affine2D untouched = world;
    assert(!world_matrix_2d(nodes[1], resolve, world));
    assert(world.t[0].raw() == untouched.t[0].raw());
    nodes[0].attach_parent = ObjectId{0, 1};
    assert(!world_matrix_2d(nodes[0], resolve, world));
    nodes[0].attach_parent = ObjectId{};
    // Depth limit: a chain of exactly 32 resolves, 33 is rejected.
    for (size_t i = 1; i < 40; ++i) {
        nodes[i].attach_parent = ObjectId{uint16_t(i - 1), 1};
        nodes[i].transform.position[0] = q(1.);
        nodes[i].transform.rotation = q(0.);
        nodes[i].transform.scale[0] = q(1.);
        nodes[i].transform.scale[1] = q(1.);
    }
    nodes[0].transform = Transform2D{};
    assert(world_matrix_2d(nodes[world2d_depth_limit - 1], resolve, world));
    assert(real(world.t[0]) == double(world2d_depth_limit - 1));
    assert(!world_matrix_2d(nodes[world2d_depth_limit], resolve, world));
    // A parent that cannot be resolved simply ends the chain.
    nodes[3].attach_parent = ObjectId{100, 1};
    assert(world_matrix_2d(nodes[5], resolve, world));
    assert(real(world.t[0]) == 3.);
}

// ---- 4. draw order -------------------------------------------------------------------
void draw_order() {
    assert(draw_key_2d(0, 0, 0) < draw_key_2d(0, 0, 1));
    assert(draw_key_2d(0, 1, 0) > draw_key_2d(0, 0, 65535));
    assert(draw_key_2d(1, -32768, 0) > draw_key_2d(0, 32767, 65535));
    assert(draw_key_2d(-1, 0, 0) < draw_key_2d(0, -32768, 0));
    struct Item { int8_t layer; int16_t order; uint16_t creation; };
    const Item items[6] = {{0, 5, 0}, {1, 0, 1}, {0, 5, 2}, {-1, 90, 3}, {0, 5, 4}, {0, -3, 5}};
    uint16_t indices[6] = {0, 1, 2, 3, 4, 5};
    auto key = [&](uint16_t index) {
        return draw_key_2d(items[index].layer, items[index].order, items[index].creation);
    };
    sort_draw_order(indices, 6, key);
    const uint16_t expected[6] = {3, 5, 0, 2, 4, 1};
    for (size_t i = 0; i < 6; ++i) assert(indices[i] == expected[i]);
    // Equal keys keep their relative order (stability), whatever the initial permutation.
    const Item tied[4] = {{2, 7, 9}, {2, 7, 9}, {2, 7, 9}, {2, 7, 9}};
    uint16_t order[4] = {3, 1, 2, 0};
    auto tied_key = [&](uint16_t index) { return draw_key_2d(tied[index].layer, tied[index].order, tied[index].creation); };
    sort_draw_order(order, 4, tied_key);
    assert(order[0] == 3 && order[1] == 1 && order[2] == 2 && order[3] == 0);
    sort_draw_order(order, 0, tied_key);  // empty and null are no-ops
    sort_draw_order(static_cast<uint16_t*>(nullptr), 4, tied_key);
}

// ---- 5. overlap primitives -----------------------------------------------------------
void overlaps() {
    Fixed a[2] = {q(0.), q(0.)}, b[2] = {q(0.9), q(0.)};
    const Collider2D box = box_collider(), circle = circle_collider(1.);
    assert(collider2d_overlap(box, a, box, b));
    b[0] = q(1.);  // touching edges: strict says no, inclusive says yes
    assert(!collider2d_overlap(box, a, box, b));
    assert(collider2d_overlap(box, a, box, b, true));
    b[0] = q(1.01);
    assert(!collider2d_overlap(box, a, box, b, true));
    // circle / circle
    b[0] = q(1.5);
    assert(collider2d_overlap(circle, a, circle, b));
    b[0] = q(2.);
    assert(!collider2d_overlap(circle, a, circle, b));
    assert(collider2d_overlap(circle, a, circle, b, true));
    // Diagonal separation must use the real distance, not the bounding box.
    b[0] = q(1.5);
    b[1] = q(1.5);
    assert(!collider2d_overlap(circle, a, circle, b));
    assert(aabb2d_overlap(collider2d_bounds(circle, a), collider2d_bounds(circle, b)));  // boxes still do
    // box / circle, both orders
    const Collider2D wide = box_collider(1., 1.), small = circle_collider(.5);
    b[0] = q(1.25);
    b[1] = q(0.);
    assert(collider2d_overlap(wide, a, small, b) && collider2d_overlap(small, b, wide, a));
    b[0] = q(1.5);
    assert(!collider2d_overlap(wide, a, small, b));
    assert(collider2d_overlap(wide, a, small, b, true));
    // A corner is only reached by the diagonal distance: (1.3,1.3) is 0.42 from the
    // corner and overlaps, (1.4,1.4) is 0.57 away and does not.
    b[0] = q(1.3);
    b[1] = q(1.3);
    assert(collider2d_overlap(wide, a, small, b));
    b[0] = q(1.4);
    b[1] = q(1.4);
    assert(!collider2d_overlap(wide, a, small, b));
    assert(aabb2d_overlap(collider2d_bounds(wide, a), collider2d_bounds(small, b)));
    // Offsets move the collider without moving the owner.
    Collider2D offset = box_collider();
    offset.offset[0] = q(2.);
    Fixed origin[2] = {q(0.), q(0.)}, far_away[2] = {q(2.), q(0.)};
    assert(collider2d_overlap(offset, origin, box, far_away));
    assert(!collider2d_overlap(box, origin, box, far_away));
    const Aabb2D bounds = collider2d_bounds(offset, origin);
    assert(real(bounds.min[0]) == 1.5 && real(bounds.max[0]) == 2.5);
    // Box colliders are AABBs: the owner's rotation is deliberately not applied, so a
    // sprite turned 45 degrees keeps the same upright bounds.
    const Aabb2D upright = collider2d_bounds(box_collider(1., .25), origin);
    assert(real(upright.max[0]) == 1. && real(upright.max[1]) == .25);
}

// ---- 6. rays -------------------------------------------------------------------------
void rays() {
    World world;
    ColliderEntry2D list[3] = {entry(box_collider(), 5., 0., 11), entry(box_collider(), 2., 0., 12),
                               entry(box_collider(), 0., 5., 13)};
    world.sync(list, 3);
    assert(world.live() == 3);
    Fixed origin[2] = {q(-2.), q(0.)}, delta[2] = {q(10.), q(0.)};
    auto hit = world.raycast2d(origin, delta);
    assert(hit && hit.index == 1 && hit.generation == 12);  // nearest, not the first slot
    // The Q24 entry fraction truncates, so the contact point can land one raw unit short.
    assert(close(real(hit.point[0]), 1.5) && real(hit.normal[0]) == -1.);
    assert(close(real(hit.fraction), .35, .001));
    delta[0] = q(-10.);  // pointing away
    assert(!world.raycast2d(origin, delta));
    delta[0] = q(2.);  // too short
    assert(!world.raycast2d(origin, delta));
    origin[1] = q(2.);  // parallel miss
    delta[0] = q(10.);
    assert(!world.raycast2d(origin, delta));
    origin[1] = q(0.);
    assert(world.raycast2d(origin, delta, 0xffffffffu, 1).index == 0);  // ignore the nearest
    Fixed inside[2] = {q(2.), q(0.)}, none[2] = {q(0.), q(0.)};
    auto within = world.raycast2d(inside, none);
    assert(within && within.started_inside && within.index == 1 && within.fraction.raw() == 0);
    // Triggers are transparent to rays unless asked for.
    World triggers;
    ColliderEntry2D trigger_list[1] = {entry(box_collider(.5, .5, true), 2., 0., 5)};
    triggers.sync(trigger_list, 1);
    assert(!triggers.raycast2d(origin, delta));
    assert(triggers.raycast2d(origin, delta, 0xffffffffu, -1, true).index == 0);
}

// ---- 7. movement ---------------------------------------------------------------------
void movement() {
    World world;
    ColliderEntry2D list[2] = {entry(box_collider(), 0., 0., 1), entry(box_collider(.5, 5.), 2., 0., 2)};
    world.sync(list, 2);
    Fixed displacement[2] = {q(5.), q(3.)};
    auto result = world.move_and_slide_2d(0, displacement);
    // Stops exactly at the wall on X and keeps sliding the full amount on Y.
    assert(result.blocked && result.index == 1 && real(result.normal[0]) == -1.);
    assert(real(result.displacement[0]) == 1. && real(result.displacement[1]) == 3.);
    assert(real(*world.position(0)) == 1.);
    assert(real(world.bounds(0)->max[0]) == 1.5);
    // Pushing into the wall again moves nothing on X.
    displacement[1] = q(0.);
    result = world.move_and_slide_2d(0, displacement);
    assert(result.blocked && result.displacement[0].raw() == 0);
    // Moving away from a touching wall is never blocked.
    displacement[0] = q(-2.);
    result = world.move_and_slide_2d(0, displacement);
    assert(!result.blocked && real(result.displacement[0]) == -2.);
    // The negative direction is clipped symmetrically.
    world.sync(list, 2);
    displacement[0] = q(-5.);
    displacement[1] = q(0.);
    World left;
    ColliderEntry2D pair[2] = {entry(box_collider(), 0., 0., 1), entry(box_collider(.5, 5.), -2., 0., 2)};
    left.sync(pair, 2);
    result = left.move_and_slide_2d(0, displacement);
    assert(result.blocked && real(result.displacement[0]) == -1. && real(result.normal[0]) == 1.);
    // Triggers and filtered layers never block a mover.
    World soft;
    ColliderEntry2D mixed[3] = {entry(box_collider(), 0., 0., 1), entry(box_collider(.5, 5., true), 2., 0., 2),
                                entry(box_collider(.5, 5.), 3., 0., 3)};
    mixed[2].collider.layer = 2;
    soft.sync(mixed, 3);
    displacement[0] = q(10.);
    result = soft.move_and_slide_2d(0, displacement, 1);
    assert(!result.blocked && real(result.displacement[0]) == 10.);
    // Starting inside another collider is reported, not silently resolved.
    World stuck;
    ColliderEntry2D overlapping[2] = {entry(box_collider(), 0., 0., 1), entry(box_collider(), .2, 0., 2)};
    stuck.sync(overlapping, 2);
    displacement[0] = q(1.);
    result = stuck.move_and_slide_2d(0, displacement);
    assert(result.unresolved_overlap);
    // An index that is not live moves nothing.
    result = stuck.move_and_slide_2d(9, displacement);
    assert(!result.blocked && result.displacement[0].raw() == 0);
    // overlap() reports every box under a query, ignoring the mover itself.
    uint16_t matches[2] = {99, 99};
    Aabb2D query = *stuck.bounds(0);
    assert(stuck.overlap(query, matches, 2) == 2 && matches[0] == 0 && matches[1] == 1);
    assert(stuck.overlap(query, nullptr, 0, 0xffffffffu, 0) == 1);
}

// ---- 8. triggers ---------------------------------------------------------------------
unsigned enters = 0, stays = 0, exits = 0;
TriggerEvent last = {};
void count(const TriggerEvent& event) {
    last = event;
    if (event.phase == TriggerPhase::Enter) ++enters;
    else if (event.phase == TriggerPhase::Stay) ++stays;
    else ++exits;
}
void reset_counters() { enters = stays = exits = 0; }
void triggers() {
    World world;
    auto callback = [](const TriggerEvent& event) { count(event); };
    ColliderEntry2D list[2] = {entry(circle_collider(1., true), 0., 0., 7), entry(box_collider(), 1.2, 0., 8)};
    world.sync(list, 2);
    reset_counters();
    world.update_triggers(callback);
    assert(enters == 1 && stays == 0 && exits == 0);
    assert(last.first == 0 && last.second == 1 && last.first_generation == 7 && last.second_generation == 8);
    reset_counters();
    world.update_triggers(callback);  // a second frame without motion is Stay, exactly once
    assert(enters == 0 && stays == 1 && exits == 0);
    reset_counters();
    world.update_triggers(callback);
    assert(stays == 1);
    // The exact circle test rules a diagonal corner out even though the boxes overlap.
    reset_counters();
    list[1] = entry(box_collider(), 1.3, 1.3, 8);
    world.sync(list, 2);
    assert(aabb2d_overlap(*world.bounds(0), *world.bounds(1)));  // broad phase still pairs them
    world.update_triggers(callback);
    assert(enters == 0 && exits == 1);
    // Re-entering fires Enter once more, not Stay.
    reset_counters();
    list[1] = entry(box_collider(), 1.2, 0., 8);
    world.sync(list, 2);
    world.update_triggers(callback);
    assert(enters == 1 && exits == 0);
    // A destroyed partner (slot disabled) still exits, exactly once.
    reset_counters();
    world.disable(1);
    world.update_triggers(callback);
    assert(exits == 1 && enters == 0);
    reset_counters();
    world.update_triggers(callback);
    assert(exits == 0 && enters == 0 && stays == 0);
    // A slot reused by another owner (new generation) exits and enters in the same frame.
    reset_counters();
    world.sync(list, 2);
    world.update_triggers(callback);
    assert(enters == 1);
    reset_counters();
    list[1] = entry(box_collider(), 1.2, 0., 99);
    world.sync(list, 2);
    world.update_triggers(callback);
    assert(enters == 1 && exits == 1 && stays == 0);
    // Two non-trigger colliders never produce a pair.
    World solid;
    ColliderEntry2D plain[2] = {entry(box_collider(), 0., 0., 1), entry(box_collider(), .2, 0., 2)};
    solid.sync(plain, 2);
    reset_counters();
    solid.update_triggers(callback);
    assert(enters == 0);
    // Layer masks must accept each other in both directions.
    World masked;
    ColliderEntry2D filtered[2] = {entry(box_collider(.5, .5, true), 0., 0., 1), entry(box_collider(), .2, 0., 2)};
    filtered[0].collider.layer_mask = 2;
    filtered[1].collider.layer = 1;
    masked.sync(filtered, 2);
    reset_counters();
    masked.update_triggers(callback);
    assert(enters == 0);
    // Pair table exhaustion is counted, not silently ignored, and never overflows.
    CollisionWorld2D<8, 2> small;
    ColliderEntry2D crowd[4];
    for (int i = 0; i < 4; ++i) crowd[i] = entry(box_collider(.5, .5, true), 0., 0., uint32_t(i + 1));
    small.sync(crowd, 4);
    reset_counters();
    small.update_triggers([](const TriggerEvent& event) { count(event); });
    assert(enters == 2 && small.dropped_trigger_pairs == 4);  // 6 pairs, capacity 2
}

// ---- 9. picking ----------------------------------------------------------------------
void picking() {
    Camera2D camera;
    Pick2DEntry entries[4];
    Fixed origin[2] = {q(0.), q(0.)}, aside[2] = {q(6.), q(0.)};
    entries[0].box = collider2d_bounds(box_collider(1., 1.), origin);
    entries[0].key = draw_key_2d(0, 0, 0);
    entries[1].box = entries[0].box;  // same place, drawn on top
    entries[1].key = draw_key_2d(0, 5, 1);
    entries[2].box = entries[0].box;  // higher layer than both
    entries[2].key = draw_key_2d(1, -10, 2);
    entries[3].box = collider2d_bounds(box_collider(), aside);
    entries[3].key = draw_key_2d(9, 0, 3);
    Fixed world[2] = {q(.5), q(.5)}, screen[2] = {};
    world_to_screen(camera, world, screen);
    size_t index = 99;
    assert(pick_2d(camera, screen, entries, 4, index) && index == 2);
    entries[2].enabled = false;  // hidden candidates are skipped
    assert(pick_2d(camera, screen, entries, 4, index) && index == 1);
    entries[1].enabled = false;
    assert(pick_2d(camera, screen, entries, 4, index) && index == 0);
    // Empty space picks nothing.
    world[0] = q(3.);
    world_to_screen(camera, world, screen);
    index = 99;
    assert(!pick_2d(camera, screen, entries, 4, index) && index == 99);
    assert(!pick_2d(camera, screen, static_cast<const Pick2DEntry*>(nullptr), 4, index));
    // Picking follows the camera: zoom and pan keep the same world point selected.
    camera.zoom = q(2.);
    camera.position[0] = q(3.);
    world[0] = q(6.);
    world[1] = q(0.);
    world_to_screen(camera, world, screen);
    assert(pick_2d(camera, screen, entries, 4, index) && index == 3);
}

void size_report() {
    std::printf("World2D sizes (host, %zu-bit pointers):\n", sizeof(void*) * 8);
    std::printf("  Camera2D %zu  Collider2D %zu  ColliderEntry2D %zu  Aabb2D %zu  Affine2D %zu\n", sizeof(Camera2D),
                sizeof(Collider2D), sizeof(ColliderEntry2D), sizeof(Aabb2D), sizeof(Affine2D));
    std::printf("  Transform2D %zu  SpatialHit2D %zu  MoveResult2D %zu  Pick2DEntry %zu\n", sizeof(Transform2D),
                sizeof(SpatialHit2D), sizeof(MoveResult2D), sizeof(Pick2DEntry));
    std::printf("  CollisionWorld2D<32> %zu  CollisionWorld2D<32,256> %zu  sine table %zu\n",
                sizeof(CollisionWorld2D<32>), sizeof(CollisionWorld2D<32, 256>), sizeof(detail::sine_table_2d));
}
}  // namespace

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    units_and_validation();
    camera_projection();
    hierarchy();
    draw_order();
    overlaps();
    rays();
    movement();
    triggers();
    picking();
    size_report();
    std::puts("World2D units, camera, hierarchy, draw order, colliders, rays, movement, triggers and picking tests passed.");
    return 0;
}
