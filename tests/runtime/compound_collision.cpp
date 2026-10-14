// Compound static collider: one entry carries many solid boxes. Generated
// levels use it so a whole floor plan of floors and walls costs one actor.
// The enclosing box is only the broad phase; every query tests the parts.
#include <cassert>
#include <cmath>
#include <cstdint>
#include <cstdio>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/collision.hpp"
#include "psyqo/fixed-point.hh"

using Q12 = psyqo::FixedPoint<12>;
static Q12 q(double value) { return Q12(int32_t(std::round(value * 4096)), Q12::RAW); }
static double real(Q12 value) { return value.raw() / 4096.0; }
using World = epok::CollisionWorld<Q12, 8, 8>;
using Box = epok::AabbT<Q12>;
using Collider = epok::ColliderT<Q12>;

static epok::Affine<Q12> matrix(double x = 0, double y = 0, double z = 0) {
    auto m = epok::Affine<Q12>::identity();
    m.values[0][3] = q(x);
    m.values[1][3] = q(y);
    m.values[2][3] = q(z);
    return m;
}

static int16_t q8(double value) { return int16_t(std::lround(value * 256)); }

// Two floor slabs meeting at x = 0, and a wall along z = 4 with a doorway
// between x = -1 and x = 1. Everything sits below y = 3, so the enclosing
// box reaches the wall top while the floor top is y = 0.
static int16_t parts[] = {
    q8(-6), q8(-0.5), q8(-6), q8(0), q8(0), q8(6),
    q8(0), q8(-0.5), q8(-6), q8(6), q8(0), q8(6),
    q8(-6), q8(0), q8(3.8), q8(-1), q8(3), q8(4.2),
    q8(1), q8(0), q8(3.8), q8(6), q8(3), q8(4.2),
};

static Collider level() {
    Collider c;
    c.enabled = true;
    c.center[1] = q(1.25);
    c.half_extents[0] = q(6);
    c.half_extents[1] = q(1.75);
    c.half_extents[2] = q(6);
    c.boxes = parts;
    c.box_count = 4;
    return c;
}

static Box character(double x, double y, double z) {
    Box b;
    b.min[0] = q(x - 0.3);
    b.max[0] = q(x + 0.3);
    b.min[1] = q(y);
    b.max[1] = q(y + 1.6);
    b.min[2] = q(z - 0.3);
    b.max[2] = q(z + 0.3);
    return b;
}

static void ground_reads_the_part_below_not_the_enclosing_box() {
    World world;
    world.set(0, level(), matrix());
    for (double x : {-3., 3., -0.01, 0.01}) {
        auto hit = world.ground(character(x, 2., 0.), q(8.));
        assert(hit && hit.entity == 0);
        assert(std::fabs(real(hit.point[1])) < 0.001);
    }
    // Over the wall the wall top is the support.
    auto wall = world.ground(character(-4., 5., 4.), q(8.));
    assert(wall && std::fabs(real(wall.point[1]) - 3.) < 0.001);
}

static void walls_block_and_the_floor_seam_does_not() {
    World world;
    world.set(0, level(), matrix());
    // Walk along X across the seam between the two slabs.
    Q12 across[3] = {q(4.), 0.0, 0.0};
    auto slide = world.move_and_slide(character(-2., 0., 0.), across);
    assert(!slide.blocked && !slide.unresolved_overlap);
    assert(std::fabs(real(slide.displacement[0]) - 4.) < 0.001);
    // Walk into the wall: stopped at its face.
    Q12 into[3] = {0.0, 0.0, q(5.)};
    auto blocked = world.move_and_slide(character(-4., 0.01, 0.), into);
    assert(blocked.blocked && blocked.entity == 0);
    assert(real(blocked.normal[2]) < 0.);
    assert(std::fabs(real(blocked.displacement[2]) - 3.5) < 0.01);
    // Walk through the doorway: nothing in the way.
    auto doorway = world.move_and_slide(character(0., 0.01, 0.), into);
    assert(!doorway.blocked);
    assert(std::fabs(real(doorway.displacement[2]) - 5.) < 0.001);
}

static void rays_and_overlaps_ignore_empty_space_inside_the_bounds() {
    World world;
    world.set(0, level(), matrix());
    Q12 origin[3] = {q(0.), q(1.), q(0.)};
    Q12 through_door[3] = {0.0, 0.0, q(8.)};
    assert(!world.raycast(origin, through_door));
    Q12 left[3] = {q(-3.), q(1.), q(0.)};
    auto wall = world.raycast(left, through_door);
    assert(wall && std::fabs(real(wall.point[2]) - 3.8) < 0.01);
    assert(real(wall.normal[2]) < 0.);
    uint16_t found[4];
    assert(world.overlap(character(0., 0.5, 0.), found, 4) == 0);
    assert(world.overlap(character(-3., 0.5, 4.), found, 4) == 1 && found[0] == 0);
}

static void parts_follow_the_owner_translation() {
    World world;
    world.set(0, level(), matrix(10., 2., -5.));
    auto hit = world.ground(character(12., 6., -5.), q(8.));
    assert(hit && std::fabs(real(hit.point[1]) - 2.) < 0.001);
    // Cached synchronisation applies the same origin.
    World cached;
    cached.set_cached(0, level(), matrix(10., 2., -5.), 1);
    auto again = cached.ground(character(12., 6., -5.), q(8.));
    assert(again && std::fabs(real(again.point[1]) - 2.) < 0.001);
}

static void triggers_inside_the_bounds_touch_only_parts() {
    World world;
    world.set(0, level(), matrix());
    Collider trigger;
    trigger.enabled = true;
    trigger.trigger = true;
    trigger.half_extents[0] = q(0.2);
    trigger.half_extents[1] = q(0.2);
    trigger.half_extents[2] = q(0.2);
    // Floating in the room, inside the enclosing box but touching no part.
    world.set(1, trigger, matrix(-3., 1.5, 0.));
    int events = 0;
    world.update_triggers([&](const epok::TriggerEvent&) { ++events; });
    assert(events == 0);
    // Moved into the wall it touches a part.
    world.set(1, trigger, matrix(-3., 1.5, 4.));
    world.update_triggers([&](const epok::TriggerEvent&) { ++events; });
    assert(events == 1);
}

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    ground_reads_the_part_below_not_the_enclosing_box();
    walls_block_and_the_floor_seam_does_not();
    rays_and_overlaps_ignore_empty_space_inside_the_bounds();
    parts_follow_the_owner_translation();
    triggers_inside_the_bounds_touch_only_parts();
    std::puts("Compound collision tests passed.");
    return 0;
}
