#pragma once
#include "fixed_math.hpp"
#include "frame_clear.hpp"
#include "psyqo/fragments.hh"
#include "psyqo/primitives/control.hh"
#include "psyqo/primitives/triangles.hh"
#include <stdint.h>

// Optional camera background that replaces the flat clear: the sky colour
// overhead, the horizon colour at the horizon and the ground colour below it.
// The screen is split along the true horizon (any pitch or roll) and each part
// is drawn as Gouraud triangles whose corner colours follow the elevation of
// the view ray through that corner. Gaps in the world below the horizon then
// show the ground instead of the sky.
//
// Once per frame and 32-bit only: the view rays are pre-shifted so dot
// products fit, and edge intersections divide by values reduced to 15 bits.
namespace epok {
struct SkyGradientColors {
    uint8_t sky[3];
    uint8_t horizon[3];
    uint8_t ground[3];
};
// The sky colour is reached about 45 degrees above the horizon (sin 0.7) and
// the ground colour about 15 degrees below it (sin 0.25).
inline constexpr int32_t sky_gradient_up_band = 2867;
inline constexpr int32_t sky_gradient_down_band = 1024;

struct SkyGradientView {
    // World up in camera space, normalised to Q12 (camera +Z forward, +Y up).
    int32_t up[3];
    // Ray through screen offset (a,b) from the centre, scaled down by 256:
    // (a*ray_x, -b*ray_y, ray_z) where ray_x carries the mirror sign.
    int32_t ray_x, ray_y, ray_z;
    int32_t centre_x, centre_y;
    int32_t level(int32_t x, int32_t y) const {
        const int32_t d[3] = {((x - centre_x) * ray_x) >> 8, (-(y - centre_y) * ray_y) >> 8, ray_z};
        return d[0] * up[0] + d[1] * up[1] + d[2] * up[2];
    }
    // Sine of the ray's elevation, Q12.
    int32_t elevation(int32_t x, int32_t y) const {
        const int32_t d[3] = {((x - centre_x) * ray_x) >> 8, (-(y - centre_y) * ray_y) >> 8, ray_z};
        const uint32_t length = fixed_math::sqrt64(uint64_t(int64_t(d[0]) * d[0] + int64_t(d[1]) * d[1] + int64_t(d[2]) * d[2]));
        if (!length) return 0;
        return (d[0] * up[0] + d[1] * up[1] + d[2] * up[2]) / int32_t(length);
    }
};

// basis: camera-from-world rotation (Q12 rows). row_scale: the projection's
// per-row scale (Q12, row 0 negative when mirrored). focal: GTE H in pixels.
template <class Basis>
inline bool sky_gradient_view(const Basis& basis, int32_t row_scale_x, int32_t row_scale_y, int32_t focal,
                              int32_t centre_x, int32_t centre_y, SkyGradientView& out) {
    int32_t up[3];
    for (int r = 0; r < 3; ++r) up[r] = basis.values[r][1].raw();
    const uint32_t length = fixed_math::sqrt64(uint64_t(int64_t(up[0]) * up[0] + int64_t(up[1]) * up[1] + int64_t(up[2]) * up[2]));
    if (!length || row_scale_y <= 0 || !row_scale_x) return false;
    for (int r = 0; r < 3; ++r) out.up[r] = (up[r] * 4096) / int32_t(length);
    const int32_t sx = row_scale_x < 0 ? -row_scale_x : row_scale_x;
    // Screen x = c + H*sx*X/Z, y = c - H*sy*Y/Z, so the ray through (a,b)
    // is proportional to (a*sy, -b*sx, H*sx*sy), here divided by 256.
    out.ray_x = row_scale_x < 0 ? -row_scale_y : row_scale_y;
    out.ray_y = sx;
    out.ray_z = int32_t((int64_t(focal) * sx * row_scale_y) >> 20);
    out.centre_x = centre_x;
    out.centre_y = centre_y;
    return true;
}

inline uint8_t sky_gradient_mix(uint8_t from, uint8_t to, int32_t t) {
    return uint8_t(int32_t(from) + ((int32_t(to) - int32_t(from)) * t >> 12));
}
inline psyqo::Color sky_gradient_color(const SkyGradientColors& colors, int32_t elevation) {
    const uint8_t* far;
    int32_t t;
    if (elevation >= 0) {
        far = colors.sky;
        t = elevation >= sky_gradient_up_band ? 4096 : (elevation << 12) / sky_gradient_up_band;
    } else {
        far = colors.ground;
        t = -elevation >= sky_gradient_down_band ? 4096 : ((-elevation) << 12) / sky_gradient_down_band;
    }
    return psyqo::Color{{.r = sky_gradient_mix(colors.horizon[0], far[0], t),
                         .g = sky_gradient_mix(colors.horizon[1], far[1], t),
                         .b = sky_gradient_mix(colors.horizon[2], far[2], t)}};
}

template <int Width, int Height>
class SkyGradient {
    static constexpr int max_triangles = 6;
    psyqo::Fragments::SimpleFragment<psyqo::Prim::TPage> pages[2];
    psyqo::Fragments::SimpleFragment<psyqo::Prim::GouraudTriangle> triangles[2][max_triangles];

    struct Point {
        int16_t x, y;
        int32_t level;
        bool horizon;
    };
    // Keeps the part of a convex polygon on one side of the horizon.
    static int clip(const Point* in, int count, int side, Point* out) {
        int n = 0;
        for (int i = 0; i < count; ++i) {
            const Point& p = in[i];
            const Point& q = in[(i + 1) % count];
            const int32_t lp = p.level * side, lq = q.level * side;
            if (lp >= 0) out[n++] = p;
            if ((lp > 0 && lq < 0) || (lp < 0 && lq > 0)) {
                // Reduce both levels to 15 bits so the interpolation stays in 32 bits.
                int32_t a = lp, b = lq;
                while (a > 32767 || a < -32767 || b > 32767 || b < -32767) {
                    a >>= 1;
                    b >>= 1;
                }
                const int32_t span = a - b;
                Point h;
                h.x = span ? int16_t(p.x + (q.x - p.x) * a / span) : p.x;
                h.y = span ? int16_t(p.y + (q.y - p.y) * a / span) : p.y;
                h.level = 0;
                h.horizon = true;
                out[n++] = h;
            }
        }
        return n;
    }

public:
    // Returns false when the camera cannot be resolved; the caller then clears
    // with the flat sky colour instead.
    template <class Gpu>
    bool draw(Gpu& gpu, const SkyGradientView& view, const SkyGradientColors& colors) {
        const unsigned parity = gpu.getParity();
        auto& page = pages[parity];
        configure_display_field<Height == 480>(page.primitive.attr);
        page.primitive.attr.setDithering(true);
        Point screen[4] = {{0, 0, 0, false}, {Width, 0, 0, false}, {Width, Height, 0, false}, {0, Height, 0, false}};
        for (auto& p : screen) p.level = view.level(p.x, p.y);
        gpu.chain(page);
        int used = 0;
        for (int side = 1; side >= -1; side -= 2) {
            Point part[8];
            const int count = clip(screen, 4, side, part);
            if (count < 3) continue;
            psyqo::Color color[8];
            for (int i = 0; i < count; ++i)
                color[i] = part[i].horizon ? psyqo::Color{{.r = colors.horizon[0], .g = colors.horizon[1], .b = colors.horizon[2]}}
                                           : sky_gradient_color(colors, view.elevation(part[i].x, part[i].y));
            for (int i = 1; i + 1 < count && used < max_triangles; ++i) {
                auto& t = triangles[parity][used++];
                t.primitive.setPointA({{.x = part[0].x, .y = part[0].y}})
                    .setPointB({{.x = part[i].x, .y = part[i].y}})
                    .setPointC({{.x = part[i + 1].x, .y = part[i + 1].y}});
                t.primitive.setColorA(color[0]).setColorB(color[i]).setColorC(color[i + 1]).setOpaque();
                gpu.chain(t);
            }
        }
        return used > 0;
    }
};
}  // namespace epok
