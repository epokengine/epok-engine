#pragma once
#include <stdint.h>
#include <limits.h>
namespace epok {
// Default cube face numbers: -Z,+Z,-X,+X,+Y,-Y. For an axis-aligned
// closed box, a camera on the inner side of a face cannot see that exterior
// face. Keep a world-space rounding margin; unsupported bases keep all faces.
inline uint8_t closed_cube_faces(const int32_t basis[3][3],const int32_t* position,const int32_t* camera){
    constexpr unsigned negative[3]={2,5,0},positive[3]={3,4,1};
    uint8_t result=63,rows=0;
    for(unsigned c=0;c<3;++c){
        int axis=-1;
        for(unsigned r=0;r<3;++r)if(basis[r][c]){if(axis>=0)return 63;axis=int(r);}
        if(axis<0||(rows&(1u<<axis)))return 63;
        rows|=uint8_t(1u<<axis);
        const int64_t coefficient=basis[axis][c];
        const int64_t radius=(coefficient<0?-coefficient:coefficient)/2;
        const int64_t delta=coefficient<0?int64_t(position[axis])-camera[axis]:int64_t(camera[axis])-position[axis];
        if(delta>-radius+64)result&=uint8_t(~(1u<<negative[c]));
        if(delta<radius-64)result&=uint8_t(~(1u<<positive[c]));
    }
    return result;
}
constexpr int frustum_gcd(int a,int b){return b?frustum_gcd(b,a%b):a;}
// GTE-space X/Y already contain the aspect/FOV row scale. H is common to
// both axes; the clipping planes must match the actual drawing-area extents.
template<int Width,int Height> struct ScreenFrustum {
    static constexpr int focal=Height*2/3;
    static constexpr int gx=frustum_gcd(Width/2,focal),gy=frustum_gcd(Height/2,focal);
    static constexpr int xz=(Width/2)/gx,xx=focal/gx,yz=(Height/2)/gy,yy=focal/gy;
    template<class Number> static Number distance(int plane,Number x,Number y,Number z){
        switch(plane){case 2:return z*xz+x*xx;case 3:return z*xz-x*xx;
          case 4:return z*yz+y*yy;default:return z*yz-y*yy;}
    }
    template<int32_t Near,int32_t Far,bool Bounded=false> static uint8_t outcode(int32_t x,int32_t y,int32_t z){
        uint8_t result=uint8_t((z<Near?1:0)|(z>=Far?2:0));
        const auto sides=[](auto x,auto y,auto z){return uint8_t(
            (z*xz+x*xx<0?4:0)|(z*xz-x*xx<0?8:0)|
            (z*yz+y*yy<0?16:0)|(z*yz-y*yy<0?32:0));};
        // Projection MAC inputs are bounded by 2^27. Ordinary display
        // ratios stay in 32 bits; awkward custom modes retain a wide path.
        if constexpr(Bounded&&xz+xx<=15&&yz+yy<=15)return result|sides(x,y,z);
        else return result|sides(int64_t(x),int64_t(y),int64_t(z));
    }
};
// Detect once per object. Pitch-only cameras and axis-aligned object transforms
// leave local X independent of the other two axes (including reflection/scale).
inline bool chunk_bounds_isolated_x(const int32_t rows[3][3]) {
    return rows[0][1]==0 && rows[0][2]==0 && rows[1][0]==0 && rows[2][0]==0;
}
// Q12 bounds. A margin also covers Q12->Q8 GTE input and translation rounding.
// The near guard avoids the GTE H/2 divide-overflow interval at 320x240.
template<class Frustum=ScreenFrustum<320,240>,class Number=int64_t> inline bool chunk_fully_inside(const int32_t* center,const int32_t* extent,int32_t focal) {
    constexpr int32_t margin=256;
    const Number near_z=Number(center[2])-extent[2],far_z=Number(center[2])+extent[2];
    if(near_z<(focal/2+1)*16+margin || near_z<1024+margin || far_z>=128*4096-margin)return false;
    const Number x=Number(center[0]<0?-Number(center[0]):center[0])+extent[0];
    const Number y=Number(center[1]<0?-Number(center[1]):center[1])+extent[1];
    return Frustum::xz*near_z-Frustum::xx*x>margin*(Frustum::xz+Frustum::xx) &&
        Frustum::yz*near_z-Frustum::yy*y>margin*(Frustum::yz+Frustum::yy);
}
// Every individual product must fit int32; the renderer checks this once per
// chunk using the largest coefficient magnitude. Post-shift sums then fit too.
inline int32_t chunk_bounds_input_limit(int32_t peak) {
    return peak ? INT32_MAX / peak : INT32_MAX;
}
// With the narrow products, each post-shift center/span sum is <=3*524288.
// This translation bound leaves ample room for their signed plane products.
template<class Frustum=ScreenFrustum<320,240>> inline bool chunk_bounds_planes_fit(const int32_t* translation) {
    if constexpr(Frustum::xz+Frustum::xx>15 || Frustum::yz+Frustum::yy>15)return false;
    for(int r=0;r<3;++r)if(translation[r]<=-(1<<26)||translation[r]>=(1<<26))return false;
    return true;
}
template<class Frustum=ScreenFrustum<320,240>,class Number=int64_t>
inline bool chunk_outside(const int32_t* center,const int32_t* extent,int32_t far) {
    const Number far_z=Number(center[2])+extent[2];
    return far_z<1024 || Number(center[2])-extent[2]>=far ||
        (Number(center[0])-extent[0])*Frustum::xx>far_z*Frustum::xz ||
        (Number(center[0])+extent[0])*Frustum::xx<-far_z*Frustum::xz ||
        (Number(center[1])-extent[1])*Frustum::yy>far_z*Frustum::yz ||
        (Number(center[1])+extent[1])*Frustum::yy<-far_z*Frustum::yz;
}
// Call only when coefficients have magnitude <16384 and vector/extent
// components have magnitude below chunk_bounds_input_limit(peak).
// Keep each product's signed Q12 shift separate: shifting a sum would change
// rounding at negative/fractional inputs. A five-tick extent guard contains
// the former sum-then-shift box too (two center and two extent rounding ticks).
#if defined(__GNUC__)
__attribute__((always_inline))
#endif
inline void narrow_chunk_bounds(const int32_t rows[3][3],const int32_t absolute[3][3],
                                const int32_t translation[3],const int32_t vector[3],
                                const int32_t extent[3],int32_t center[3],int32_t span[3],
                                bool isolated_x) {
    if(isolated_x) {
        center[0]=translation[0]+((rows[0][0]*vector[0])>>12);
        span[0]=((absolute[0][0]*extent[0])>>12)+5;
        for(int r=1;r<3;++r) {
            center[r]=translation[r]+(((rows[r][1]*vector[1])>>12)+((rows[r][2]*vector[2])>>12));
            span[r]=((absolute[r][1]*extent[1])>>12)+((absolute[r][2]*extent[2])>>12)+5;
        }
        return;
    }
    for(int r=0;r<3;++r) {
        int32_t sum=0,radius=0;
        for(int c=0;c<3;++c) {
            sum+=(rows[r][c]*vector[c])>>12;
            radius+=(absolute[r][c]*extent[c])>>12;
        }
        center[r]=translation[r]+sum;span[r]=radius+5;
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
