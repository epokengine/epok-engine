#include <cassert>
#include <cstdint>
#include <cstdio>
#include "polygon.hpp"
#include "sprite_math.hpp"
using namespace epok;
// Reference implementations mirror the previous renderer's arithmetic.
static uint32_t reference_modulate(uint32_t c) { return (c * 128 + 127) / 255; }
static uint32_t reference_fog_amount(int32_t depth, int32_t start, int32_t end) {
    if (depth <= start) return 0;
    if (depth >= end) return 4096;
    return uint32_t(int64_t(depth - start) * 4096 / (end - start));
}
static ProjectedVertex vertex(int32_t x, int32_t y, int32_t z) {
    ProjectedVertex v{};
    v.camera[0] = x; v.camera[1] = y; v.camera[2] = z;
    project_cpu<CameraQ8, 640, 480>(v);
    return v;
}
int main() {
    uint32_t seed=7219;
    for(int i=0;i<20000;++i){
        int32_t p[3];for(auto& v:p){seed=seed*1664525u+1013904223u;v=i%2?int32_t(seed):int32_t(seed%1200000)-600000;}
        const int64_t x=p[0],y=p[1],z=p[2];
        const bool inside=z>=1024&&z<128*4096&&z+x>=0&&z-x>=0&&3*z+4*y>=0&&3*z-4*y>=0;
        assert(sprite_detail::interior(p[0],p[1],p[2])==inside);
        const int32_t depth=1024+int32_t(seed%520000);
        const int scales[]={160,320,240,4096};
        for(int scale:scales)assert(sprite_detail::project_ratio(p[0],scale,depth)==int32_t(int64_t(p[0])*scale/depth));
    }
    assert(sprite_detail::interior(0,768,1024)&&!sprite_detail::interior(0,769,1024));
    assert(!sprite_detail::interior(INT32_MIN,INT32_MAX,INT32_MAX));
    for(int i=0;i<20000;++i){
        seed=seed*1664525u+1013904223u;const uint32_t denominator=1+(seed&0x0fffffffu);
        seed=seed*1664525u+1013904223u;const uint32_t numerator=i%2?seed%denominator:(seed&65535)%denominator;
        assert(clip_fraction16(numerator,denominator)==(uint64_t(numerator)<<16)/denominator);
        assert(clip_fraction16(denominator,denominator)==65536);
    }
    const int64_t boundaries[]={INT32_MIN,int64_t(INT32_MIN)-1,-65537,-1,0,1,65535,INT32_MAX,int64_t(INT32_MAX)+1,int64_t(1)<<38,-(int64_t(1)<<38)};
    for(auto n:boundaries)for(int32_t d=1;d<33000;d+=137){
        const int32_t expected=n>=0?int32_t(n/d):-int32_t((-n+d-1)/d);
        assert(floor_div(n,d)==expected);
    }
    for(int i=0;i<2000;++i){
        ProjectedVertex full[3]{};CompactProjectedVertex compact[3]{};
        for(int k=0;k<3;++k){
            full[k].screen.x=int16_t((i*17+k*73)%320);full[k].screen.y=int16_t((i*11+k*97)%240);
            full[k].camera[2]=64+(i*19+k*101)%32700;
            compact[k].screen=full[k].screen;compact[k].camera.value=uint16_t(full[k].camera[2]);
        }
        assert(screen_area(full[0],full[1],full[2])==screen_area(compact[0],compact[1],compact[2]));
        assert(CameraQ8::bucket(full[0].camera[2],full[1].camera[2],full[2].camera[2])==
            CameraQ8::bucket(compact[0].camera[2],compact[1].camera[2],compact[2].camera[2]));
    }
    for (uint32_t c = 0; c < 256; ++c) {
        assert(modulate_channel(c) == reference_modulate(c));
        for (uint32_t t = 0; t < 256; t += 5) assert(scale_channel(c, t) == c * t / 255);
    }
    assert(modulate_color(pack_color(255, 0, 128)) == pack_color(128, 0, 64));
    // Ordering table buckets match integer division over the whole frustum.
    for (int32_t z = CameraQ8::near; z < CameraQ8::far; z += 7)
        assert(CameraQ8::bucket(z, z + 3, z + 5) == (z + z + 3 + z + 5) / (3 * 64));
    for (int32_t z = CameraQ12::near; z < CameraQ12::far; z += 97)
        assert(CameraQ12::bucket(z, z, z + 1000) == (3 * z + 1000) / (3 * 1024));
    for (int32_t depth = 0; depth < 40000; depth += 13)
        assert(fog_amount(depth, 7424, 16384) == reference_fog_amount(depth, 7424, 16384));
    assert(fog_amount(100, 100, 101) == 0 && fog_amount(101, 100, 101) == 4096);
    const uint8_t fog[3] = {64, 77, 102};
    assert(blend_fog(pack_color(200, 100, 50), 0, fog) == pack_color(200, 100, 50));
    assert(blend_fog(pack_color(200, 100, 50), 4096, fog) == pack_color(64, 77, 102));
    assert(blend_fog(pack_color(200, 100, 50), 2048, fog) == pack_color(132, 89, 76));
    // CPU projection rounds toward negative infinity like the GTE.
    assert(floor_div(7, 2) == 3 && floor_div(-7, 2) == -4 && floor_div(-8, 2) == -4 && floor_div(0, 5) == 0);
    auto p = vertex(256, -128, 3584);
    assert(p.visible && p.outcode == 0 && p.screen.x == 320 + 256 * 320 / 3584 && p.screen.y == 240 + (128 * 320 + 3583) / 3584);
    assert(!vertex(0, 0, 63).visible && (vertex(0, 0, 63).outcode & 1));
    assert(!vertex(0, 0, 32768).visible && (vertex(0, 0, 32768).outcode & 2));
    assert(vertex(-5000, 0, 4000).outcode == 4 && vertex(5000, 0, 4000).outcode == 8);
    assert(vertex(0, -3001, 4000).outcode == 16 && vertex(0, 3001, 4000).outcode == 32);
    const uint8_t left = frustum_outcode32<64, 32768>(-5000, 0, 4000), top = frustum_outcode32<64, 32768>(0, 3001, 4000);
    assert(left == 4 && top == 32);
    // A far-off vertex still inside the frustum but past the guard band is invisible.
    const auto outlier = vertex(-32000, 0, 32000);
    assert(!outlier.visible || outlier.screen.x >= -1023);
    // Clipping a triangle crossing the near plane produces a quad with the
    // intersections pinned to the plane and interpolated attributes.
    ClipVertex buffers[2][12] = {};
    const int32_t positions[3][3] = {{0, 0, 1000}, {2000, 0, 1000}, {0, 0, 20}};
    for (int i = 0; i < 3; ++i) {
        for (int d = 0; d < 3; ++d) buffers[0][i].p[d] = positions[i][d];
        buffers[0][i].color[0] = i == 2 ? 0 : 200; buffers[0][i].uv[0] = (i == 2 ? 100 : 0) << 16;
    }
    int from = 0;
    const int count = clip_polygon<CameraQ8>(buffers, 3, 1, from);
    assert(count == 4);
    int pinned = 0;
    for (int i = 0; i < count; ++i) {
        const auto& v = buffers[from][i];
        assert(v.p[2] >= CameraQ8::near);
        if (v.p[2] == CameraQ8::near) { ++pinned; assert(v.color[0] > 0 && v.color[0] < 200 && (v.uv[0] >> 16) > 0 && (v.uv[0] >> 16) < 100); }
    }
    assert(pinned == 2);
    // A triangle fully behind the near plane disappears.
    for (int i = 0; i < 3; ++i) buffers[0][i].p[2] = 10;
    assert(clip_polygon<CameraQ8>(buffers, 3, 1, from) == 0);
    auto a = vertex(0, 0, 4000), b = vertex(1000, 0, 4000), c = vertex(0, 1000, 4000);
    assert(screen_area(a, b, c) < 0 && screen_area(a, c, b) > 0 && screen_area(a, a, b) == 0);
    std::puts("Polygon color, fog, depth bucket, projection and clipping tests passed.");
}
