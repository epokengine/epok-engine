#pragma once
#include "frustum.hpp"
#include "psyqo/primitives/common.hh"
#include <stdint.h>

// Polygon preparation shared by the mesh renderer. Everything here is written
// for the R3000: no 64-bit arithmetic or integer division on the paths that run
// once per vertex or per triangle, and the software clipper stays out of line.
namespace epok {
struct ProjectedVertex {
    int32_t camera[3];      // camera space, CameraUnits scale
    psyqo::Vertex screen;   // valid when visible
    uint16_t fog = 0;       // Q12 fog amount, 0 when nearer than the fog start
    uint8_t outcode = 0;
    bool visible = false;
};
// Interior/no-fog submission needs only screen XY and depth. 128 entries fit
// in the PSX scratchpad, versus 51 full clipping vertices. Never pass this
// representation to a clipper or a camera-space lighting calculation.
struct CompactProjectedVertex {
    psyqo::Vertex screen;
    struct Depth {
        uint16_t value;
        int32_t operator[](size_t axis) const{return axis==2?value:0;}
    } camera;
};
static_assert(sizeof(CompactProjectedVertex)==8);
// Per-quad attributes prepared once, then copied per emitted triangle.
struct QuadPacket {
    uint32_t final_color[4];   // packed 0x00BBGGRR; modulated (128 = 1.0) when textured
    uint32_t shaded[4];        // packed fogged 0..255 colors kept for the clipper
    uint32_t uv[4];            // u | v << 8
    uint32_t clut16 = 0, tpage16 = 0;   // already shifted into the high halfword
    uint32_t command = 0x30000000;      // GPU command byte with transparency bit
    int depth_bias = 0;
    bool textured = false;
};
// Camera-space scale used by the renderer. Near/far match the frustum planes.
template<int32_t NearPlane, int32_t FarExclusive, int DepthShift> struct CameraUnits {
    static constexpr int32_t near = NearPlane;
    static constexpr int32_t far = FarExclusive;
    // Ordering table bucket from three depths: (za+zb+zc) / (3 << DepthShift),
    // exact for the frustum range without an integer division.
    static int bucket(int32_t za, int32_t zb, int32_t zc) {
        const uint32_t n = uint32_t(za + zb + zc) >> DepthShift;   // <= 1536 for 512 buckets
        return int((n * 0x5556u) >> 16);
    }
};
// Q12 camera coordinates (4096 = one unit), as used by scripts and colliders.
using CameraQ12 = CameraUnits<1024, 128 * 4096, 10>;
// Q8 camera coordinates (256 = one unit) produced by the GTE projection path.
using CameraQ8 = CameraUnits<64, 128 * 256, 6>;
inline uint32_t pack_color(uint32_t r, uint32_t g, uint32_t b) { return r | (g << 8) | (b << 16); }
// (c * 128 + 127) / 255 for c in 0..255, exact without a division.
inline uint32_t modulate_channel(uint32_t c) { const uint32_t v = c * 128 + 127; return (v + 1 + (v >> 8)) >> 8; }
inline uint32_t modulate_color(uint32_t packed) {
    return pack_color(modulate_channel(packed & 255), modulate_channel((packed >> 8) & 255), modulate_channel((packed >> 16) & 255));
}
// a * b / 255 for 0..255 inputs, exact.
inline uint32_t scale_channel(uint32_t a, uint32_t b) { const uint32_t v = a * b; return (v + 1 + (v >> 8)) >> 8; }
// Linear fog: 0 before start, 4096 at or beyond end. Depth and range share
// one unit; the range must be positive.
inline uint16_t fog_amount(int32_t depth, int32_t start, int32_t end) {
    if (depth <= start) return 0;
    if (depth >= end) return 4096;
    return uint16_t((uint32_t(depth - start) * 4096u) / uint32_t(end - start));
}
inline uint32_t blend_fog(uint32_t packed, uint32_t amount, const uint8_t* fog) {
    if (!amount) return packed;
    const uint32_t keep = 4096 - amount;
    const uint32_t r = ((packed & 255) * keep + fog[0] * amount + 2048) >> 12;
    const uint32_t g = (((packed >> 8) & 255) * keep + fog[1] * amount + 2048) >> 12;
    const uint32_t b = (((packed >> 16) & 255) * keep + fog[2] * amount + 2048) >> 12;
    return pack_color(r, g, b);
}
// Screen area sign in the GPU's coordinate system; zero for degenerate triangles.
template<class Vertex> inline int32_t screen_area(const Vertex& a, const Vertex& b, const Vertex& c) {
    return (int32_t(b.screen.x) - a.screen.x) * (int32_t(c.screen.y) - a.screen.y) -
           (int32_t(b.screen.y) - a.screen.y) * (int32_t(c.screen.x) - a.screen.x);
}
// Floor division for a positive divisor, matching the GTE's arithmetic shift.
inline int32_t floor_div(int64_t numerator, int32_t divisor) {
    // Clipped Q8 coordinates times the pixel focal length fit a native word.
    // Use the R3000 DIV quotient/remainder instead of libgcc's 64-bit divide.
    if(numerator>=INT32_MIN&&numerator<=INT32_MAX){
        const auto n=int32_t(numerator);
        return n/divisor-(n%divisor<0?1:0);
    }
    return numerator >= 0 ? int32_t(numerator / divisor) : -int32_t((-numerator + divisor - 1) / divisor);
}
// CPU perspective for clipped or fallback vertices, rounding like the GTE so
// mixed triangles share edges. Inputs inside the frustum after clipping never
// overflow; the guard band keeps far outliers software-clipped.
template<class Units, int Width, int Height> inline void project_cpu(ProjectedVertex& point) {
    const int32_t z = point.camera[2];
    point.outcode = frustum_outcode_units<Units::near, Units::far>(point.camera[0], point.camera[1], z);
    point.visible = z >= Units::near && z < Units::far;
    if (!point.visible) return;
    constexpr int shift = Units::near == 1024 ? 4 : 0;
    const int32_t zs = z >> shift;
    const int32_t sx = Width / 2 + floor_div(int64_t(point.camera[0] >> shift) * (Width / 2), zs);
    const int32_t sy = Height / 2 - floor_div(int64_t(point.camera[1] >> shift) * (Height * 2 / 3), zs);
    point.visible = sx >= -1023 && sx <= 1023 && sy >= -1023 && sy <= 1023;
    point.screen = {{.x = int16_t(sx), .y = int16_t(sy)}};
}
struct ClipVertex {
    int32_t p[3];
    int32_t color[3];
    int32_t uv[2];   // 16.16 texture pixels
};
inline uint32_t clip_fraction16(uint32_t numerator,uint32_t denominator) {
    if(numerator==denominator)return 65536;
    if(numerator<=65535)return (numerator<<16)/denominator;
    uint32_t fraction=0;
    for(int bit=0;bit<16;++bit){numerator<<=1;fraction<<=1;
        if(numerator>=denominator){numerator-=denominator;fraction|=1;}}
    return fraction;
}
// Sutherland-Hodgman against the frustum planes named by `planes` (bit i =
// plane i of frustum_outcode). Returns the vertex count in buffers[from].
template<class Units> inline int clip_polygon(ClipVertex (&buffers)[2][12], int count, uint8_t planes, int& from) {
    from = 0;
    for (int plane = 0; plane < 6 && count > 0; ++plane) {
        if (!(planes & (1 << plane))) continue;
        auto distance = [plane](const ClipVertex& v) -> int32_t {
            switch (plane) {
            case 0: return v.p[2] - Units::near;
            case 1: return Units::far - 1 - v.p[2];
            case 2: return v.p[2] + v.p[0];
            case 3: return v.p[2] - v.p[0];
            case 4: return 3 * v.p[2] + 4 * v.p[1];
            default: return 3 * v.p[2] - 4 * v.p[1];
            }
        };
        int next = 0;
        auto* input = buffers[from];
        auto* output = buffers[1 - from];
        const ClipVertex* previous = &input[count - 1];
        int32_t pd = distance(*previous);
        for (int i = 0; i < count; ++i) {
            const ClipVertex& current = input[i];
            const int32_t cd = distance(current);
            if ((pd >= 0) != (cd >= 0)) {
                const uint32_t numerator = pd < 0 ? -pd : pd, denominator = pd - cd < 0 ? cd - pd : pd - cd;
                const uint32_t fraction=clip_fraction16(numerator,denominator);
                ClipVertex intersection;
                for (int d = 0; d < 3; ++d) {
                    intersection.p[d] = previous->p[d] + int32_t((int64_t(current.p[d] - previous->p[d]) * fraction) >> 16);
                    intersection.color[d] = previous->color[d] + int32_t((int64_t(current.color[d] - previous->color[d]) * fraction) >> 16);
                }
                for (int d = 0; d < 2; ++d) intersection.uv[d] = previous->uv[d] + int32_t((int64_t(current.uv[d] - previous->uv[d]) * fraction) >> 16);
                if (plane == 0) intersection.p[2] = Units::near;
                else if (plane == 1) intersection.p[2] = Units::far - 1;
                if (next < 12) output[next++] = intersection;
            }
            if (cd >= 0 && next < 12) output[next++] = current;
            previous = &current; pd = cd;
        }
        count = next;
        from = 1 - from;
    }
    return count;
}
}
