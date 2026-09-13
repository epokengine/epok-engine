#pragma once
// Real 2D world runtime (actor-architecture initiative, phase P8).
//
// Everything a World2D level needs at runtime and nothing the editor owns: units and
// validation for Transform2D, a Camera2D with a deterministic world<->screen projection
// that never touches the 3D camera, hierarchical 2D transforms over SceneComponent2D
// attachments, a stable draw order, box/circle colliders with triggers, and a picking
// helper shared by the runtime and the editor.
//
// Consumers include this header explicitly; object_model.hpp does not pull it in, so a
// build that has no 2D content pays nothing. MIPS constraints apply: C++20 without RTTI,
// exceptions, heap or STL containers, fixed capacities, no recursion.
//
// ---------------------------------------------------------------------------------------
// Units and ranges (the contract every 2D document and tool must respect)
// ---------------------------------------------------------------------------------------
//   * Position: world units in Q12 (`epok::Fixed`), exactly the same scale as the 3D world.
//     A 2D world unit is NOT a pixel; see `pixels_per_unit_2d` below. Valid range is
//     |position[k]| <= `world2d_position_limit` (8192 world units). The limit keeps every
//     intermediate of the fixed-point math below inside int64 without saturation and keeps
//     squared distances (used by circle tests) inside 2^51.
//   * Rotation: degrees, stored as `Fixed`, exactly like the 3D `Transform::rotation`
//     components. Positive rotation turns counter-clockwise in world space (+X towards +Y).
//     Any value is accepted; it is wrapped into one turn before use.
//   * Scale: per axis, valid range is (0, `world2d_scale_limit`] = (0, 64]. Zero or
//     negative scale is rejected rather than silently mirrored: a mirrored sprite is a
//     rendering flag, not a degenerate matrix.
//   * Draw order: `Transform2D::draw_order` is an int16 combined with an int8 layer and the
//     creation index into a stable sort key (see `draw_key_2d`).
// ---------------------------------------------------------------------------------------
#include "object_model.hpp"
#include <stddef.h>
#include <stdint.h>

namespace epok {

// ---- units, limits and validation ----------------------------------------------------

// Screen pixels covered by one world unit at zoom 1. 32 px/unit puts 10 world units across
// a 320 px display, which is the density the 2D fixture is authored at.
inline constexpr int pixels_per_unit_2d = 32;
inline constexpr Fixed world2d_position_limit = 8192.0;
inline constexpr Fixed world2d_scale_limit = 64.0;
// Maximum length of an attachment chain resolved in one call (root included).
inline constexpr size_t world2d_depth_limit = 32;

inline bool fixed_in_range(Fixed value, Fixed limit) { return value.abs() <= limit; }

// Documented validity of a Transform2D. Rotation is always valid (it is wrapped), so only
// position and scale can fail.
inline bool transform2d_valid(const Transform2D& transform) {
    for (int k = 0; k < 2; ++k) {
        if (!fixed_in_range(transform.position[k], world2d_position_limit)) return false;
        if (transform.scale[k] <= 0.0 || transform.scale[k] > world2d_scale_limit) return false;
    }
    return true;
}
// Clamps into the documented range instead of rejecting. Scale is clamped to the smallest
// representable positive Q12 value rather than to zero.
inline void transform2d_clamp(Transform2D& transform) {
    const Fixed epsilon(1, Fixed::RAW);
    for (int k = 0; k < 2; ++k) {
        if (transform.position[k] > world2d_position_limit) transform.position[k] = world2d_position_limit;
        if (transform.position[k] < -world2d_position_limit) transform.position[k] = -world2d_position_limit;
        if (transform.scale[k] <= 0.0) transform.scale[k] = epsilon;
        if (transform.scale[k] > world2d_scale_limit) transform.scale[k] = world2d_scale_limit;
    }
}

// ---- deterministic Q12 trigonometry --------------------------------------------------
//
// psyqo::Trig<> is a runtime object owned by main.cpp and its angle unit is a turn
// fraction, not degrees; depending on it here would drag a service into a header that the
// host tests compile standalone. Instead the quarter turn is tabulated at compile time:
// 257 Q12 samples of sin over [0, 90] degrees (256 intervals of exactly 1440 raw degree
// units = 0.3515625 degrees), linearly interpolated and mirrored into the other three
// quadrants. The table is `constexpr`, so no floating point survives into the image; only
// 257 * 4 = 1028 bytes of rodata do. Maximum error against the ideal sine is below
// 1 Q12 unit (2.4e-4), and the cardinal angles (0, 90, 180, 270) are exact.
namespace detail {
constexpr double sine_series_2d(double x) {  // x in [0, pi/2]; alternating Taylor series
    double term = x, sum = x;
    for (int n = 1; n < 12; ++n) {
        term *= -x * x / double((2 * n) * (2 * n + 1));
        sum += term;
    }
    return sum;
}
struct SineTable2D { int32_t values[257] = {}; };
consteval SineTable2D make_sine_table_2d() {
    SineTable2D table;
    for (int i = 0; i <= 256; ++i) {
        const double angle = 3.14159265358979323846 * 0.5 * double(i) / 256.0;
        table.values[i] = int32_t(sine_series_2d(angle) * 4096.0 + 0.5);
    }
    return table;
}
inline constexpr SineTable2D sine_table_2d = make_sine_table_2d();
inline constexpr int32_t degrees_per_turn_raw = 360 * 4096;
inline constexpr int32_t degrees_per_quarter_raw = 90 * 4096;
inline constexpr int32_t sine_step_raw = degrees_per_quarter_raw / 256;  // 1440, exact
// sin of an angle inside the first quadrant, expressed in raw Q12 degrees [0, 90*4096].
inline int32_t sine_quarter_2d(int32_t raw_degrees) {
    if (raw_degrees >= degrees_per_quarter_raw) return sine_table_2d.values[256];
    const int32_t index = raw_degrees / sine_step_raw;
    const int32_t fraction = raw_degrees - index * sine_step_raw;
    const int32_t low = sine_table_2d.values[index], high = sine_table_2d.values[index + 1];
    return low + int32_t((int64_t(high - low) * fraction) / sine_step_raw);
}
// Saturating conversion of a raw Q12 accumulator back into Fixed.
inline Fixed clamp_raw_2d(int64_t value) {
    if (value > INT32_MAX) value = INT32_MAX;
    if (value < INT32_MIN) value = INT32_MIN;
    return Fixed(int32_t(value), Fixed::RAW);
}
inline Fixed mul_2d(Fixed a, Fixed b) { return clamp_raw_2d((int64_t(a.raw()) * b.raw()) / 4096); }
}  // namespace detail

// Sine of an angle in degrees. Deterministic and identical on host and MIPS.
inline Fixed sin_degrees(Fixed degrees) {
    int32_t wrapped = degrees.raw();
    if (wrapped >= detail::degrees_per_turn_raw || wrapped <= -detail::degrees_per_turn_raw)
        wrapped %= detail::degrees_per_turn_raw;
    if (wrapped < 0) wrapped += detail::degrees_per_turn_raw;
    const int32_t quadrant = wrapped / detail::degrees_per_quarter_raw;
    const int32_t rest = wrapped - quadrant * detail::degrees_per_quarter_raw;
    const int32_t magnitude = (quadrant & 1) ? detail::sine_quarter_2d(detail::degrees_per_quarter_raw - rest)
                                             : detail::sine_quarter_2d(rest);
    return Fixed(quadrant >= 2 ? -magnitude : magnitude, Fixed::RAW);
}
inline Fixed cos_degrees(Fixed degrees) { return sin_degrees(degrees + Fixed(90 * 4096, Fixed::RAW)); }

// ---- hierarchical 2D transforms ------------------------------------------------------

// 2x3 affine matrix: linear part `m` (rotation and scale, shear when inherited) plus the
// translation `t`. Mirrors epok::Affine<Number> in two dimensions.
struct Affine2D {
    Fixed m[2][2] = {};
    Fixed t[2] = {};
    static Affine2D identity() {
        Affine2D out;
        out.m[0][0] = 1.0;
        out.m[1][1] = 1.0;
        return out;
    }
    void point(const Fixed* in, Fixed* out) const {
        for (int r = 0; r < 2; ++r)
            out[r] = t[r] + detail::mul_2d(m[r][0], in[0]) + detail::mul_2d(m[r][1], in[1]);
    }
    // Same as point() without the translation: for directions, extents and velocities.
    void direction(const Fixed* in, Fixed* out) const {
        for (int r = 0; r < 2; ++r) out[r] = detail::mul_2d(m[r][0], in[0]) + detail::mul_2d(m[r][1], in[1]);
    }
};
// parent * child: the child's local matrix expressed in the parent's space.
inline Affine2D compose(const Affine2D& parent, const Affine2D& child) {
    Affine2D out;
    for (int r = 0; r < 2; ++r) {
        for (int c = 0; c < 2; ++c)
            out.m[r][c] = detail::mul_2d(parent.m[r][0], child.m[0][c]) + detail::mul_2d(parent.m[r][1], child.m[1][c]);
        out.t[r] = parent.t[r] + detail::mul_2d(parent.m[r][0], child.t[0]) + detail::mul_2d(parent.m[r][1], child.t[1]);
    }
    return out;
}
// Local matrix of a Transform2D: translate * rotate * scale, applied in that order to a
// point (scale first, then rotation, then translation).
inline Affine2D local_matrix(const Transform2D& transform) {
    const Fixed s = sin_degrees(transform.rotation), c = cos_degrees(transform.rotation);
    Affine2D out;
    out.m[0][0] = detail::mul_2d(c, transform.scale[0]);
    out.m[0][1] = detail::mul_2d(-s, transform.scale[1]);
    out.m[1][0] = detail::mul_2d(s, transform.scale[0]);
    out.m[1][1] = detail::mul_2d(c, transform.scale[1]);
    out.t[0] = transform.position[0];
    out.t[1] = transform.position[1];
    return out;
}

// World matrix of a SceneComponent2D by walking its attach_parent chain. `resolve` is any
// callable `const SceneComponent2D* (ObjectId)`, so this never depends on Level internals
// and host tests can supply a plain array lookup. Returns false and leaves `out` untouched
// when the chain exceeds `world2d_depth_limit` or contains a cycle; a parent that fails to
// resolve ends the chain, exactly as a detached component would.
template <class Resolver>
inline bool world_matrix_2d(const SceneComponent2D& leaf, Resolver&& resolve, Affine2D& out) {
    const SceneComponent2D* chain[world2d_depth_limit] = {};
    size_t depth = 0;
    const SceneComponent2D* node = &leaf;
    while (node) {
        if (depth >= world2d_depth_limit) return false;
        for (size_t i = 0; i < depth; ++i)
            if (chain[i] == node) return false;  // cycle
        chain[depth++] = node;
        if (!node->attach_parent.valid()) break;
        node = resolve(node->attach_parent);
    }
    Affine2D result = local_matrix(chain[depth - 1]->transform);
    for (size_t i = depth - 1; i > 0; --i) result = compose(result, local_matrix(chain[i - 1]->transform));
    out = result;
    return true;
}

// ---- Camera2D ------------------------------------------------------------------------
//
// Projection convention (no dependency whatsoever on the 3D camera or on projection_focal):
//   screen = viewport_center + R(-camera.rotation) * (world - camera.position) * k
// with k = zoom * pixels_per_unit_2d, and the screen Y axis pointing DOWN while the world
// Y axis points UP, so the Y component is negated. Screen coordinates are Q12 pixels
// (sub-pixel precision) relative to the display origin, not to the viewport origin.
// Positions far outside the viewport saturate instead of wrapping: |world| <= 8192 at
// zoom 64 exceeds the Q12 range, and a saturated off-screen coordinate is still rejected
// by every clip test.
struct Camera2D {
    Fixed position[2] = {0.0, 0.0};
    Fixed zoom = 1.0;
    Fixed rotation = 0.0;
    // x, y, width, height in pixels. Defaults to the whole display.
    int16_t viewport[4] = {0, 0, int16_t(display_width), int16_t(display_height)};
};

inline Fixed camera2d_pixels_per_unit(const Camera2D& camera) {
    return detail::clamp_raw_2d(int64_t(camera.zoom.raw()) * pixels_per_unit_2d);
}
inline void camera2d_viewport_center(const Camera2D& camera, Fixed* out) {
    out[0] = Fixed(int32_t(camera.viewport[0]) * 4096 + int32_t(camera.viewport[2]) * 2048, Fixed::RAW);
    out[1] = Fixed(int32_t(camera.viewport[1]) * 4096 + int32_t(camera.viewport[3]) * 2048, Fixed::RAW);
}
inline void world_to_screen(const Camera2D& camera, const Fixed* world_xy, Fixed* screen_xy) {
    const Fixed dx = world_xy[0] - camera.position[0], dy = world_xy[1] - camera.position[1];
    const Fixed s = sin_degrees(-camera.rotation), c = cos_degrees(-camera.rotation);
    const Fixed rx = detail::mul_2d(dx, c) - detail::mul_2d(dy, s);
    const Fixed ry = detail::mul_2d(dx, s) + detail::mul_2d(dy, c);
    const Fixed k = camera2d_pixels_per_unit(camera);
    Fixed center[2];
    camera2d_viewport_center(camera, center);
    screen_xy[0] = center[0] + detail::mul_2d(rx, k);
    screen_xy[1] = center[1] - detail::mul_2d(ry, k);
}
// Exact inverse of world_to_screen up to Q12 rounding. A zoom of zero has no inverse and
// returns the camera position.
inline void screen_to_world(const Camera2D& camera, const Fixed* screen_xy, Fixed* world_xy) {
    Fixed center[2];
    camera2d_viewport_center(camera, center);
    const Fixed k = camera2d_pixels_per_unit(camera);
    if (!k.raw()) {
        world_xy[0] = camera.position[0];
        world_xy[1] = camera.position[1];
        return;
    }
    const Fixed px = (screen_xy[0] - center[0]) / k, py = -((screen_xy[1] - center[1]) / k);
    const Fixed s = sin_degrees(camera.rotation), c = cos_degrees(camera.rotation);
    world_xy[0] = camera.position[0] + detail::mul_2d(px, c) - detail::mul_2d(py, s);
    world_xy[1] = camera.position[1] + detail::mul_2d(px, s) + detail::mul_2d(py, c);
}

// ---- draw order ----------------------------------------------------------------------
//
// A single monotonically comparable key so the renderer never needs a multi-field
// comparator: layer (int8, coarse) above draw_order (int16) above the creation index
// (uint16, the tie breaker that makes the order stable and reproducible across frames).
// 40 significant bits; ascending key means "drawn later", i.e. on top.
inline uint64_t draw_key_2d(int8_t layer, int16_t draw_order, uint16_t creation) {
    const uint64_t layer_bits = uint64_t(uint8_t(int16_t(layer) + 128));
    const uint64_t order_bits = uint64_t(uint16_t(int32_t(draw_order) + 32768));
    return (layer_bits << 32) | (order_bits << 16) | uint64_t(creation);
}
// Bounded, allocation-free, stable insertion sort of an index array by draw key. `key` is
// any callable `uint64_t (uint16_t index)`. Insertion sort is the right shape here: the
// draw list is small (a few dozen sprites), already almost sorted between frames, and the
// algorithm needs no scratch memory.
template <class Key>
inline void sort_draw_order(uint16_t* indices, size_t count, Key&& key) {
    if (!indices) return;
    for (size_t i = 1; i < count; ++i) {
        const uint16_t value = indices[i];
        const uint64_t k = key(value);
        size_t j = i;
        while (j > 0 && key(indices[j - 1]) > k) {
            indices[j] = indices[j - 1];
            --j;
        }
        indices[j] = value;
    }
}

// ---- 2D colliders --------------------------------------------------------------------
//
// Semantics and deliberate limits:
//   * A collider is placed by a world position plus `offset`, both in world axes. Box
//     colliders are AXIS-ALIGNED: the owner's rotation is IGNORED for collision, exactly
//     like the 3D collider's conservative world AABB. A rotated sprite therefore keeps an
//     upright collision box; oriented narrow-phase boxes are an explicit extension.
//   * Circles are rotation invariant, so they are exact.
//   * `layer` is the single bit (or bits) this collider occupies, `layer_mask` is the set
//     of layers it interacts with; a pair interacts only when both masks accept each other,
//     the same rule as the 3D world.
//   * Overlap predicates come in two flavours: the strict one treats touching as NOT
//     overlapping (open intervals, matching epok::aabb_overlap), the inclusive one treats
//     exact contact as an overlap. Triggers use the strict rule so a mover that stops
//     exactly at a wall does not enter it.
struct Collider2D {
    enum Shape : uint8_t { Box, Circle };
    Shape shape = Box;
    Fixed half_extents[2] = {0.5, 0.5};
    Fixed radius = 0.5;
    Fixed offset[2] = {0.0, 0.0};
    uint32_t layer = 1;
    uint32_t layer_mask = 0xffffffffu;
    bool trigger = false;
    bool enabled = true;
};
struct Aabb2D { Fixed min[2], max[2]; };

inline bool aabb2d_overlap(const Aabb2D& a, const Aabb2D& b) {
    for (int k = 0; k < 2; ++k)
        if (a.max[k] <= b.min[k] || a.min[k] >= b.max[k]) return false;
    return true;
}
inline bool aabb2d_touch(const Aabb2D& a, const Aabb2D& b) {
    for (int k = 0; k < 2; ++k)
        if (a.max[k] < b.min[k] || a.min[k] > b.max[k]) return false;
    return true;
}
inline bool aabb2d_contains(const Aabb2D& box, const Fixed* point) {
    for (int k = 0; k < 2; ++k)
        if (point[k] < box.min[k] || point[k] > box.max[k]) return false;
    return true;
}
inline Aabb2D aabb2d_translated(Aabb2D box, const Fixed* delta) {
    for (int k = 0; k < 2; ++k) {
        box.min[k] += delta[k];
        box.max[k] += delta[k];
    }
    return box;
}
// World centre of a collider: the owner's world position plus the (unrotated) offset.
inline void collider2d_center(const Collider2D& collider, const Fixed* world_xy, Fixed* out) {
    out[0] = world_xy[0] + collider.offset[0];
    out[1] = world_xy[1] + collider.offset[1];
}
inline Aabb2D collider2d_bounds(const Collider2D& collider, const Fixed* world_xy) {
    Fixed center[2];
    collider2d_center(collider, world_xy, center);
    Aabb2D out;
    for (int k = 0; k < 2; ++k) {
        const Fixed extent = collider.shape == Collider2D::Circle ? collider.radius : collider.half_extents[k];
        out.min[k] = center[k] - extent;
        out.max[k] = center[k] + extent;
    }
    return out;
}
// Squared distance in raw Q24; |position| <= 8192 keeps this inside 2^51.
inline int64_t distance_squared_raw_2d(const Fixed* a, const Fixed* b) {
    const int64_t dx = int64_t(a[0].raw()) - b[0].raw(), dy = int64_t(a[1].raw()) - b[1].raw();
    return dx * dx + dy * dy;
}
inline bool circle_overlap_2d(const Fixed* a, Fixed radius_a, const Fixed* b, Fixed radius_b, bool inclusive = false) {
    const int64_t sum = int64_t(radius_a.raw()) + radius_b.raw();
    const int64_t distance = distance_squared_raw_2d(a, b);
    return inclusive ? distance <= sum * sum : distance < sum * sum;
}
// Closest point on the box to the circle centre; touching counts only when inclusive.
inline bool box_circle_overlap_2d(const Aabb2D& box, const Fixed* center, Fixed radius, bool inclusive = false) {
    Fixed closest[2];
    for (int k = 0; k < 2; ++k) {
        closest[k] = center[k];
        if (closest[k] < box.min[k]) closest[k] = box.min[k];
        if (closest[k] > box.max[k]) closest[k] = box.max[k];
    }
    const int64_t distance = distance_squared_raw_2d(closest, center);
    const int64_t r = radius.raw();
    return inclusive ? distance <= r * r : distance < r * r;
}
// Shape-aware pair test in world space.
inline bool collider2d_overlap(const Collider2D& a, const Fixed* world_a, const Collider2D& b, const Fixed* world_b,
                               bool inclusive = false) {
    Fixed ca[2], cb[2];
    collider2d_center(a, world_a, ca);
    collider2d_center(b, world_b, cb);
    if (a.shape == Collider2D::Circle && b.shape == Collider2D::Circle)
        return circle_overlap_2d(ca, a.radius, cb, b.radius, inclusive);
    if (a.shape == Collider2D::Box && b.shape == Collider2D::Circle)
        return box_circle_overlap_2d(collider2d_bounds(a, world_a), cb, b.radius, inclusive);
    if (a.shape == Collider2D::Circle && b.shape == Collider2D::Box)
        return box_circle_overlap_2d(collider2d_bounds(b, world_b), ca, a.radius, inclusive);
    const Aabb2D box_a = collider2d_bounds(a, world_a), box_b = collider2d_bounds(b, world_b);
    return inclusive ? aabb2d_touch(box_a, box_b) : aabb2d_overlap(box_a, box_b);
}

// One synchronized collider. `active` folds the owner's activation into the world without
// touching the authored `Collider2D::enabled` flag.
struct ColliderEntry2D {
    Collider2D collider;
    Fixed position[2] = {0.0, 0.0};
    uint32_t generation = 0;
    bool active = true;
};
struct SpatialHit2D {
    int index = -1;
    uint32_t generation = 0;
    Fixed fraction = 1.0;
    Fixed point[2] = {}, normal[2] = {};
    bool started_inside = false;
    explicit operator bool() const { return index >= 0; }
};
struct MoveResult2D {
    Fixed displacement[2] = {}, normal[2] = {};
    int index = -1;
    bool blocked = false;
    bool unresolved_overlap = false;
};

// Bounded 2D collision world mirroring the shape of epok::CollisionWorld: fixed capacity,
// no allocation, conservative world AABBs for the broad phase and for sweeps, exact shape
// tests for triggers. PairCapacity bounds the remembered overlapping trigger pairs; a
// frame that produces more increments `dropped_trigger_pairs` and drops the extra pairs
// (they get neither enter nor exit) instead of corrupting the table or growing it.
template <size_t Capacity, size_t PairCapacity = 64>
class CollisionWorld2D {
    static_assert(Capacity > 0 && Capacity <= 1024, "2D collision capacity must stay bounded");
    struct Entry {
        Collider2D collider;
        Fixed position[2] = {0.0, 0.0};
        Aabb2D box = {{0.0, 0.0}, {0.0, 0.0}};
        uint32_t generation = 0;
        bool enabled = false;
    };
    struct Pair { uint16_t a, b; uint32_t ga, gb; };
    Entry entries[Capacity];
    size_t limit = 0, trigger_count = 0;
    Pair previous[PairCapacity];
    size_t previous_count = 0;
    static constexpr int64_t one = int64_t(1) << 24;  // Q24 sweep fraction, as in 3D
    static bool same(const Pair& a, const Pair& b) { return a.a == b.a && a.b == b.b && a.ga == b.ga && a.gb == b.gb; }
    bool match(size_t i, uint32_t mask, int ignore, bool triggers) const {
        return entries[i].enabled && int(i) != ignore && (entries[i].collider.layer & mask) &&
               (triggers || !entries[i].collider.trigger);
    }
    // Segment/AABB slab test in Q24; `t` is the entry fraction.
    static bool segment(const Fixed* origin, const Fixed* delta, const Aabb2D& box, int64_t& t, int& axis, int& sign,
                        bool& inside) {
        int64_t enter = 0, leave = one;
        axis = -1;
        sign = 0;
        inside = true;
        for (int k = 0; k < 2; ++k) {
            const int64_t p = origin[k].raw(), d = delta[k].raw(), lo = box.min[k].raw(), hi = box.max[k].raw();
            if (p <= lo || p >= hi) inside = false;
            if (!d) {
                if (p < lo || p > hi) return false;
                continue;
            }
            int64_t near_t = ((lo - p) * one) / d, far_t = ((hi - p) * one) / d;
            int normal = -1;
            if (near_t > far_t) {
                const int64_t tmp = near_t;
                near_t = far_t;
                far_t = tmp;
                normal = 1;
            }
            if (near_t > enter || (near_t == enter && axis < 0)) {
                enter = near_t;
                axis = k;
                sign = normal;
            }
            if (far_t < leave) leave = far_t;
            if (enter > leave) return false;
        }
        if (leave < 0 || enter > one) return false;
        t = enter;
        return true;
    }
    void note_enabled(size_t index) {
        if (index >= limit) limit = index + 1;
        if (entries[index].collider.trigger) ++trigger_count;
    }

public:
    uint32_t dropped_trigger_pairs = 0;

    void clear() {
        for (auto& entry : entries) entry.enabled = false;
        previous_count = 0;
        dropped_trigger_pairs = 0;
        limit = trigger_count = 0;
    }
    // Replaces the whole live set in one call; slots beyond `count` become disabled and
    // therefore produce trigger exits on the next update_triggers().
    void sync(const ColliderEntry2D* list, size_t count) {
        for (auto& entry : entries) entry.enabled = false;
        limit = trigger_count = 0;
        if (!list) return;
        if (count > Capacity) count = Capacity;
        for (size_t i = 0; i < count; ++i) set(i, list[i]);
    }
    void set(size_t index, const ColliderEntry2D& value) {
        if (index >= Capacity) return;
        auto& entry = entries[index];
        entry.collider = value.collider;
        entry.position[0] = value.position[0];
        entry.position[1] = value.position[1];
        entry.generation = value.generation;
        entry.enabled = value.active && value.collider.enabled;
        entry.box = collider2d_bounds(value.collider, value.position);
        if (entry.enabled) note_enabled(index);
    }
    void disable(size_t index) { if (index < Capacity) entries[index].enabled = false; }
    size_t live() const {
        size_t count = 0;
        for (size_t i = 0; i < limit; ++i)
            if (entries[i].enabled) ++count;
        return count;
    }
    const Aabb2D* bounds(size_t index) const {
        return index < Capacity && entries[index].enabled ? &entries[index].box : nullptr;
    }
    const Fixed* position(size_t index) const {
        return index < Capacity && entries[index].enabled ? entries[index].position : nullptr;
    }
    // Returns the total number of matches; only the first `output_capacity` are written.
    // Broad phase only: circles are tested through their bounding box.
    size_t overlap(const Aabb2D& box, uint16_t* output, size_t output_capacity, uint32_t mask = 0xffffffffu,
                   int ignore = -1, bool triggers = true) const {
        size_t count = 0;
        for (size_t i = 0; i < limit; ++i)
            if (match(i, mask, ignore, triggers) && aabb2d_overlap(box, entries[i].box)) {
                if (output && count < output_capacity) output[count] = uint16_t(i);
                ++count;
            }
        return count;
    }
    // `displacement` is the complete segment, not a unit direction. Nearest hit wins;
    // circles are approximated by their AABB, as in the broad phase.
    SpatialHit2D raycast2d(const Fixed* origin, const Fixed* displacement, uint32_t mask = 0xffffffffu, int ignore = -1,
                           bool triggers = false) const {
        SpatialHit2D hit;
        int64_t best = one + 1;
        for (size_t i = 0; i < limit; ++i) {
            if (!match(i, mask, ignore, triggers)) continue;
            int64_t t = 0;
            int axis = -1, sign = 0;
            bool inside = false;
            if (!segment(origin, displacement, entries[i].box, t, axis, sign, inside) || t >= best) continue;
            best = t;
            hit.index = int(i);
            hit.generation = entries[i].generation;
            hit.started_inside = inside;
            hit.fraction = Fixed(int32_t(t * 4096 / one), Fixed::RAW);
            for (int k = 0; k < 2; ++k) {
                hit.point[k] = origin[k] + detail::clamp_raw_2d(int64_t(displacement[k].raw()) * t / one);
                hit.normal[k] = Fixed(k == axis ? sign * 4096 : 0, Fixed::RAW);
            }
        }
        return hit;
    }
    // Axis-separated resolution: X is applied first and clipped against every blocking
    // box whose Y span overlaps, then Y is applied against the updated position. A mover
    // blocked on one axis therefore keeps sliding on the other. Triggers never block. The
    // entry's stored position and bounds are updated with the resolved displacement.
    MoveResult2D move_and_slide_2d(size_t index, const Fixed* displacement, uint32_t mask = 0xffffffffu) {
        MoveResult2D result;
        if (index >= Capacity || !entries[index].enabled) return result;
        Aabb2D box = entries[index].box;
        if (overlap(box, nullptr, 0, mask, int(index), false)) result.unresolved_overlap = true;
        for (int axis = 0; axis < 2; ++axis) {
            const Fixed wanted = displacement[axis];
            if (!wanted.raw()) continue;
            const int other = axis ^ 1;
            Fixed allowed = wanted;
            int blocker = -1;
            for (size_t i = 0; i < limit; ++i) {
                if (!match(i, mask, int(index), false)) continue;
                const Aabb2D& b = entries[i].box;
                if (box.max[other] <= b.min[other] || box.min[other] >= b.max[other]) continue;
                if (wanted > 0.0 && box.max[axis] <= b.min[axis]) {
                    const Fixed gap = b.min[axis] - box.max[axis];
                    if (gap < allowed) { allowed = gap; blocker = int(i); }
                } else if (wanted < 0.0 && box.min[axis] >= b.max[axis]) {
                    const Fixed gap = b.max[axis] - box.min[axis];
                    if (gap > allowed) { allowed = gap; blocker = int(i); }
                }
            }
            box.min[axis] += allowed;
            box.max[axis] += allowed;
            result.displacement[axis] = allowed;
            if (blocker >= 0 && allowed.raw() != wanted.raw()) {
                result.blocked = true;
                result.index = blocker;
                result.normal[axis] = wanted > 0.0 ? Fixed(-4096, Fixed::RAW) : Fixed(4096, Fixed::RAW);
            }
        }
        auto& entry = entries[index];
        for (int k = 0; k < 2; ++k) entry.position[k] += result.displacement[k];
        entry.box = box;
        return result;
    }
    // Enter/Stay/Exit per pair, exactly once each, using the exact shape tests. A pair
    // whose generation changed (slot reused by another owner) exits and re-enters, so a
    // destroyed partner always produces its Exit.
    template <class Callback>
    void update_triggers(Callback&& callback) {
        Pair next[PairCapacity];
        size_t count = 0;
        dropped_trigger_pairs = 0;
        if (trigger_count)
            for (size_t a = 0; a < limit; ++a) {
                if (!entries[a].enabled) continue;
                for (size_t b = a + 1; b < limit; ++b) {
                    const auto& x = entries[a];
                    const auto& y = entries[b];
                    if (!y.enabled || (!x.collider.trigger && !y.collider.trigger)) continue;
                    if (!(x.collider.layer & y.collider.layer_mask) || !(y.collider.layer & x.collider.layer_mask)) continue;
                    if (!aabb2d_overlap(x.box, y.box)) continue;
                    if (!collider2d_overlap(x.collider, x.position, y.collider, y.position)) continue;
                    if (count >= PairCapacity) { ++dropped_trigger_pairs; continue; }
                    next[count++] = Pair{uint16_t(a), uint16_t(b), x.generation, y.generation};
                }
            }
        // The pair set is frozen before any callback runs, so user code that moves or
        // destroys objects cannot change the events of the frame it is reacting to.
        for (size_t n = 0; n < count; ++n) {
            bool was = false;
            for (size_t i = 0; i < previous_count; ++i)
                if (same(next[n], previous[i])) { was = true; break; }
            callback(TriggerEvent{next[n].a, next[n].b, next[n].ga, next[n].gb,
                                  was ? TriggerPhase::Stay : TriggerPhase::Enter});
        }
        for (size_t i = 0; i < previous_count; ++i) {
            bool stays = false;
            for (size_t j = 0; j < count; ++j)
                if (same(previous[i], next[j])) { stays = true; break; }
            if (!stays)
                callback(TriggerEvent{previous[i].a, previous[i].b, previous[i].ga, previous[i].gb, TriggerPhase::Exit});
        }
        previous_count = count;
        for (size_t i = 0; i < count; ++i) previous[i] = next[i];
    }
};

// ---- picking -------------------------------------------------------------------------

// One candidate for pick_2d: its world AABB and its draw key. Circles are picked through
// their bounding box; the caller filters further when that is not good enough.
struct Pick2DEntry {
    Aabb2D box = {{0.0, 0.0}, {0.0, 0.0}};
    uint64_t key = 0;
    bool enabled = true;
};
// Topmost entry under a screen point (highest draw key wins; ties go to the lowest index,
// which keeps the result stable). Returns false when nothing is hit. Shared by the runtime
// and the editor so both select the same object for the same click.
inline bool pick_2d(const Camera2D& camera, const Fixed* screen_xy, const Pick2DEntry* entries, size_t count,
                    size_t& out_index) {
    if (!entries) return false;
    Fixed world[2];
    screen_to_world(camera, screen_xy, world);
    bool found = false;
    uint64_t best = 0;
    for (size_t i = 0; i < count; ++i) {
        if (!entries[i].enabled || !aabb2d_contains(entries[i].box, world)) continue;
        if (found && entries[i].key <= best) continue;
        found = true;
        best = entries[i].key;
        out_index = i;
    }
    return found;
}
}  // namespace epok
