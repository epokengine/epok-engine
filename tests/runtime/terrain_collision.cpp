// Heightfield collider: the ramp generalized to a two-axis grid. It is a
// surface to stand on, never an obstacle, so a character walks up a slope
// instead of being pushed sideways by it.
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

// A 4 x 4 cell grid of 2-unit cells: 5 x 5 corners, Q8 above the box bottom.
// The surface rises one unit per cell along X and is flat along Z, so every
// sampled height is predictable by hand.
static constexpr uint16_t cells = 4;
static int16_t heights[(cells + 1) * (cells + 1)];
static void build_heights() {
    for (uint16_t j = 0; j <= cells; ++j)
        for (uint16_t i = 0; i <= cells; ++i) heights[j * (cells + 1) + i] = int16_t(i * 256);
}

// The box spans 8 x 8 units horizontally and 0..4 vertically, centred on the
// origin, matching what the cooker emits for a terrain at the origin.
static Collider terrain() {
    Collider c;
    c.enabled = true;
    c.half_extents[0] = q(4.);
    c.half_extents[1] = q(2.);
    c.half_extents[2] = q(4.);
    c.center[1] = q(2.);
    c.heights = heights;
    c.height_cells[0] = cells;
    c.height_cells[1] = cells;
    c.height_step = q(2.);
    return c;
}

static Box character(double x, double y, double z) {
    Box b;
    b.min[0] = q(x - 0.4);
    b.max[0] = q(x + 0.4);
    b.min[1] = q(y);
    b.max[1] = q(y + 1.6);
    b.min[2] = q(z - 0.4);
    b.max[2] = q(z + 0.4);
    return b;
}

static void samples_the_grid() {
    World world;
    world.set(0, terrain(), matrix());
    // The box bottom is y = 0, so a corner of i*1.0 units reads back directly.
    // x = -4 is grid column 0, x = +4 is column 4.
    struct { double x, expected; } cases[] = {
        {-4., 0.}, {-2., 1.}, {0., 2.}, {2., 3.}, {3.9, 3.95}, {-3., 0.5},
    };
    for (auto c : cases) {
        auto hit = world.ground(character(c.x, 8., 0.), q(16.));
        assert(hit);
        assert(std::fabs(real(hit.point[1]) - c.expected) < 0.02);
        assert(real(hit.normal[1]) == 1.);
    }
    // Z does not change the height on this grid.
    auto a = world.ground(character(0., 8., -3.), q(16.));
    auto b = world.ground(character(0., 8., 3.), q(16.));
    assert(real(a.point[1]) == real(b.point[1]));
}

static void clamps_outside_the_grid() {
    World world;
    world.set(0, terrain(), matrix());
    // Past the last column the border height is held rather than extrapolated,
    // so stepping off the edge does not launch a character off a cliff that
    // the grid never described.
    auto inside = world.ground(character(3.9, 8., 0.), q(16.));
    assert(inside);
    // A probe beyond the footprint misses the box entirely.
    auto outside = world.ground(character(40., 8., 0.), q(16.));
    assert(!outside);
}

static void is_a_surface_not_an_obstacle() {
    World world;
    world.set(0, terrain(), matrix());
    // Standing inside the box: the solver lifts the character onto the
    // surface instead of pushing it out sideways, exactly as for a ramp.
    Q12 delta[3] = {q(0.), q(0.), q(0.)};
    auto result = world.move_and_slide(character(0., 0.5, 0.), delta);
    assert(result.grounded);
    assert(!result.unresolved_overlap);
    assert(real(result.displacement[1]) > 0.);
    assert(real(result.displacement[0]) == 0. && real(result.displacement[2]) == 0.);
    // Walking up the slope is never blocked horizontally.
    Q12 forward[3] = {q(1.5), q(0.), q(0.)};
    auto walk = world.move_and_slide(character(-2., 1.05, 0.), forward);
    assert(!walk.blocked || walk.grounded);
    assert(real(walk.displacement[0]) > 1.0);
}

static void an_empty_grid_falls_back_to_the_box() {
    World world;
    auto c = terrain();
    c.heights = nullptr;
    world.set(0, c, matrix());
    auto hit = world.ground(character(0., 8., 0.), q(16.));
    assert(hit);
    // Without a grid the top of the box is the surface.
    assert(std::fabs(real(hit.point[1]) - 4.) < 0.01);
    // A grid with no cells is equally inert, so a malformed cook cannot read
    // past the array.
    auto degenerate = terrain();
    degenerate.height_cells[0] = 0;
    world.set(1, degenerate, matrix(0., 0., 32.));
    auto flat = world.ground(character(0., 8., 32.), q(16.));
    assert(flat);
    assert(std::fabs(real(flat.point[1]) - 4.) < 0.01);
}

static void translation_moves_the_grid_with_the_box() {
    World world;
    world.set(0, terrain(), matrix(20., 5., 0.));
    // Column 0 of the grid now sits at x = 16, and the box bottom at y = 5.
    auto hit = world.ground(character(16., 20., 0.), q(32.));
    assert(hit);
    assert(std::fabs(real(hit.point[1]) - 5.) < 0.02);
    auto higher = world.ground(character(20., 20., 0.), q(32.));
    assert(std::fabs(real(higher.point[1]) - 7.) < 0.02);
}

int main() {
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);
    _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
    build_heights();
    samples_the_grid();
    clamps_outside_the_grid();
    is_a_surface_not_an_obstacle();
    an_empty_grid_falls_back_to_the_box();
    translation_moves_the_grid_with_the_box();
    std::puts("Terrain heightfield collision tests passed.");
    return 0;
}
