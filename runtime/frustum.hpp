#pragma once
#include <stdint.h>
namespace epok {
// Detect once per object. Pitch-only cameras and axis-aligned object transforms
// leave local X independent of the other two axes (including reflection/scale).
inline bool chunk_bounds_isolated_x(const int32_t rows[3][3]) {
    return rows[0][1]==0 && rows[0][2]==0 && rows[1][0]==0 && rows[2][0]==0;
}
// Call only when every coefficient has magnitude <16384 and each input vector
// component has magnitude <131072, as checked by the renderer's narrow path.
// Keep each product's signed Q12 shift separate: shifting a sum would change
// rounding at negative/fractional inputs and could change edge visibility.
#if defined(__GNUC__)
__attribute__((always_inline))
#endif
inline void narrow_chunk_bounds(const int32_t rows[3][3],const int32_t absolute[3][3],
                                const int32_t translation[3],const int32_t vector[3],
                                const int32_t extent[3],int32_t center[3],int32_t span[3],
                                bool isolated_x) {
    if(isolated_x) {
        center[0]=translation[0]+((rows[0][0]*vector[0])>>12);
        span[0]=((absolute[0][0]*extent[0])>>12)+3;
        for(int r=1;r<3;++r) {
            center[r]=translation[r]+(((rows[r][1]*vector[1])>>12)+((rows[r][2]*vector[2])>>12));
            span[r]=((absolute[r][1]*extent[1])>>12)+((absolute[r][2]*extent[2])>>12)+3;
        }
        return;
    }
    for(int r=0;r<3;++r) {
        int32_t sum=0,radius=0;
        for(int c=0;c<3;++c) {
            sum+=(rows[r][c]*vector[c])>>12;
            radius+=(absolute[r][c]*extent[c])>>12;
        }
        center[r]=translation[r]+sum;span[r]=radius+3;
    }
}
// Same six half-spaces as the polygon clipper, before perspective division.
// Wide intermediates also make classification safe for far off-screen points.
template<int32_t Near,int32_t FarExclusive> inline uint8_t frustum_outcode_units(int32_t x,int32_t y,int32_t z) {
    uint8_t code=0;
    if(z<Near)code|=1;
    if(z>=FarExclusive)code|=2;
    if(int64_t(z)+x<0)code|=4;
    if(int64_t(z)-x<0)code|=8;
    if(3*int64_t(z)+4*int64_t(y)<0)code|=16;
    if(3*int64_t(z)-4*int64_t(y)<0)code|=32;
    return code;
}
inline uint8_t frustum_outcode(int32_t x,int32_t y,int32_t z) { return frustum_outcode_units<1024,128*4096>(x,y,z); }
// Plain 32-bit variant for inputs bounded by 2^27 in magnitude (the GTE
// projection path guarantees this); 3z+4y then cannot overflow.
template<int32_t Near,int32_t FarExclusive> inline uint8_t frustum_outcode32(int32_t x,int32_t y,int32_t z) {
    uint8_t code=0;
    if(z<Near)code|=1;
    if(z>=FarExclusive)code|=2;
    if(z+x<0)code|=4;
    if(z-x<0)code|=8;
    if(3*z+4*y<0)code|=16;
    if(3*z-4*y<0)code|=32;
    return code;
}
// The GPU clips pixels to its drawing area, but rejects primitives with spans
// >1023 horizontally or >511 vertically. Keep software clipping for those and
// for vertices crossing near/far or the signed screen coordinate guard band.
template<class Point> bool gpu_clip_safe(const Point& a,const Point& b,const Point& c) {
    if(!a.visible||!b.visible||!c.visible)return false;
    int min_x=a.screen.x,max_x=min_x,min_y=a.screen.y,max_y=min_y;
    const Point* rest[2]={&b,&c};
    for(auto p:rest){
        if(p->screen.x<min_x)min_x=p->screen.x;if(p->screen.x>max_x)max_x=p->screen.x;
        if(p->screen.y<min_y)min_y=p->screen.y;if(p->screen.y>max_y)max_y=p->screen.y;
    }
    return max_x-min_x<=1023&&max_y-min_y<=511;
}
}
