#pragma once
#include "affine.hpp"
#include "polygon.hpp"
#include "baked_draw_types.hpp"
#include "ordered_draw_types.hpp"

namespace epok {
// Q8 operands derived from signed Q12 each fit 28 bits. Subtract AFTER the
// shift: the difference is safe in i32 even across the full world range.
inline bool ordered_gte_input_fits(int32_t world_q12,int32_t camera_q8) {
    return uint32_t((world_q12>>4)-camera_q8+32768)<=65535u;
}
// Both signed SXY halves can be range-checked together. Original bits reject
// negative / >= power-of-two coordinates; biased bits reject the unused tail
// for non-power-of-two dimensions. Any carry from X means X was already out.
// Fully inside this viewport also proves the GPU's 1023x511 span constraints.
// This is acceptance only: crossing/offscreen polygons retain the exact bounds
// and clipping path, including their depth/near-plane checks.
template<unsigned Width,unsigned Height>
inline bool ordered_screen_inside(const uint32_t* screen,bool quad) {
    static_assert(Width>0&&Width<=1024&&Height>0&&Height<=512);
    constexpr uint32_t width_power=[](){uint32_t n=1;while(n<Width)n<<=1;return n;}();
    constexpr uint32_t height_power=[](){uint32_t n=1;while(n<Height)n<<=1;return n;}();
    constexpr uint32_t bias=((height_power-Height)<<16)|(width_power-Width);
    constexpr uint32_t mask=((65536u-height_power)<<16)|(65536u-width_power);
    uint32_t outside=screen[0]|screen[1]|screen[2];
    outside|=(screen[0]+bias)|(screen[1]+bias)|(screen[2]+bias);
    if(quad)outside|=screen[3]|(screen[3]+bias);
    return !(outside&mask);
}
// Exact signed division (truncate toward zero), without MIPS DIV. Keeping the
// two denominators in a conditional expression made GCC merge them into one
// variable divide. Shift the magnitude first, then divide by 3 via reciprocal.
inline int32_t ordered_depth_average(int32_t sum,bool quad) {
    const uint32_t magnitude=sum<0?0u-uint32_t(sum):uint32_t(sum);
    const uint32_t quotient=quad?magnitude>>8:
        uint32_t((uint64_t(magnitude>>6)*0xaaaaaaabu)>>33);
    return sum<0?-int32_t(quotient):int32_t(quotient);
}
// The GTE's AVSZ3/AVSZ4 form ZSF * sum >> 12 (arithmetic) saturated to
// 0..0xffff. ZSF4=16 equals sum/256 exactly for the non-negative sums of
// accepted faces; ZSF3=21 approximates sum/192 (at most 1.6% nearer). The CPU
// form below reproduces the GTE result for faces projected by the CPU.
inline constexpr int32_t ordered_zsf3=21,ordered_zsf4=16;
inline int32_t ordered_gte_depth_average(int32_t sum,bool quad) {
    const int64_t value=(int64_t(quad?ordered_zsf4:ordered_zsf3)*sum)>>12;
    return value<0?0:value>0xffff?0xffff:int32_t(value);
}
// Rebase Q8 operands on a 4096-Q8 (16-world-unit) lattice RELATIVE to the
// camera. R is Q12: R*(v-d*4096)/4096 + (TR+R*d) preserves the complete MAC
// sum, including fractional bits and negative-Y rounding. No precision drops.
// Offsets and centers originate in signed Q12. Publish outputs only on success.
template<class Number>
#if defined(__GNUC__) && !defined(__clang__)
__attribute__((noinline,optimize("O3")))
#elif defined(__GNUC__)
__attribute__((noinline))
#endif
bool ordered_projection_anchor(const Affine<Number>& matrix,const int32_t* base_offset,
    const int32_t* center,int32_t* offset,int32_t* translation) {
    int32_t shift[3],next_offset[3],next_translation[3];
    for(unsigned c=0;c<3;++c){
        shift[c]=(center[c]+base_offset[c]+2048)>>12;
        next_offset[c]=base_offset[c]-shift[c]*4096;
    }
    for(unsigned r=0;r<3;++r){
        int64_t value=matrix.values[r][3].raw()>>4;
        for(unsigned c=0;c<3;++c)value+=int64_t(matrix.values[r][c].raw())*shift[c];
        // Reserve room in the signed 44-bit accumulator and 32-bit MAC for
        // three worst-case signed 16x16 products. Negating Y must also be safe.
        if(value<-2146697215LL||value>2146697215LL)return false;
        next_translation[r]=int32_t(value);
    }
    for(unsigned c=0;c<3;++c){offset[c]=next_offset[c];translation[c]=next_translation[c];}
    return true;
}
// Independent integer oracle for the MAC results of the loaded projection
// matrix. Model the GTE's sum-then-floor rounding, including the inverted Y
// row and quantized translation; do not reuse the CPU fallback's different
// per-product truncation. Intended for validation, not the render hot path.
template<class Number>
inline void ordered_gte_camera_reference(const Affine<Number>& matrix,
    const int32_t* world,const int32_t* offset,int32_t* camera) {
    for(unsigned r=0;r<3;++r){
        const int sign=r==1?-1:1;
        int64_t sum=int64_t(sign)*(matrix.values[r][3].raw()>>4)*4096;
        for(unsigned c=0;c<3;++c)
            sum+=int64_t(sign)*matrix.values[r][c].raw()*((world[c]>>4)+offset[c]);
        camera[r]=sign*int32_t(sum>>12);
    }
}
// The GTE's perspective division (RTPS/RTPT): the documented unsigned
// Newton-Raphson reciprocal with its 257-entry seed table. Returns false and
// 0x1FFFF where the GTE flags a divide overflow (H >= 2*SZ3).
struct GteUnrTable {uint8_t value[257];};
inline constexpr GteUnrTable gte_unr_table=[]{
    GteUnrTable table{};
    for(int i=0;i<257;++i){const int value=(0x40000/(i+0x100)+1)/2-0x101;table.value[i]=uint8_t(value<0?0:value);}
    return table;
}();
inline bool gte_divide(uint32_t h,uint32_t sz,uint32_t& quotient) {
    if(h>=sz*2){quotient=0x1ffff;return false;}
    int shift=0;while(!(sz&(0x8000u>>shift)))++shift;
    const uint32_t numerator=h<<shift;uint32_t d=sz<<shift;
    const uint32_t u=gte_unr_table.value[(d-0x7fc0)>>7]+0x101;
    d=(0x2000080u-d*u)>>8;
    d=(0x0000080u+d*u)>>8;
    const uint64_t n=(uint64_t(numerator)*d+0x8000)>>16;
    quotient=n>0x1ffff?0x1ffff:uint32_t(n);
    return true;
}
// Screen position exactly as RTPS/RTPT produce it from GTE MAC results
// (camera Y up, as ordered_gte_camera_reference returns it), or false where
// the GTE would flag the vertex: IR or SZ3 saturation, divide overflow, or
// SX/SY outside -0x400..0x3FF.
inline bool gte_project_point(const int32_t* camera,int32_t h,int32_t ofx,int32_t ofy,uint32_t& screen) {
    const int32_t ir1=camera[0],ir2=-camera[1],sz=camera[2];
    if(ir1<-0x8000||ir1>0x7fff||ir2<-0x8000||ir2>0x7fff||sz<0||sz>0xffff)return false;
    uint32_t n;
    if(!gte_divide(uint32_t(h),uint32_t(sz),n))return false;
    const int64_t sx=(int64_t(n)*ir1+(int64_t(ofx)<<16))>>16,sy=(int64_t(n)*ir2+(int64_t(ofy)<<16))>>16;
    if(sx<-0x400||sx>0x3ff||sy<-0x400||sy>0x3ff)return false;
    screen=uint32_t(uint16_t(int16_t(sx)))|(uint32_t(uint16_t(int16_t(sy)))<<16);
    return true;
}
#ifdef EPOK_VALIDATE_GTE
// Cumulative counters: a sampler cannot lose a transient validation failure.
// world_camera_delta is informational (full Q12 camera vs quantized GTE), not
// the bit-exact MAC oracle. No validation code/counters exist in normal builds.
struct OrderedValidationStats {
    uint32_t vertices=0,triples=0,range_fallback_vertices=0,flagged_vertices=0;
    uint32_t camera_errors=0,batch_errors=0,screen_errors=0;
    uint32_t max_camera_delta=0,max_screen_delta=0,max_world_camera_delta=0;
    uint32_t first_error_kind=0,first_error_sample=0,first_error_token=0,first_error_vertex=0;
};
inline OrderedValidationStats ordered_validation_stats;
#endif
// Full-range projection is not synonymous with clipping. Keep native packets
// when all projected corners fit the GPU guard band and span limits.
template<class Units,int Width,int Height,class Number>
#if defined(__GNUC__) && !defined(__clang__)
__attribute__((noinline,optimize("O3")))
#elif defined(__GNUC__)
__attribute__((noinline))
#endif
bool project_ordered_cpu(
    const int32_t (*points)[3],unsigned count,
    const CameraSpace<Number>& view,const Number* row_scale,
    ProjectedVertex* projected,uint32_t* screen,int32_t* depth,
    const Affine<Number>* gte_matrix=nullptr,const int32_t* gte_offset=nullptr) {
    // With the loaded GTE matrix and lattice offset, corners take the GTE's own
    // integer camera transform, and every corner the GTE itself could project
    // gets the GTE's exact screen position. An edge shared with a neighbour
    // drawn by RTPT therefore matches it to the pixel, leaving no crack. Only
    // corners the GTE would flag (closer than H/2, saturated) use the exact
    // CPU division, and every face containing one comes through here.
    constexpr int32_t focal=Height*2/3;
    bool direct=true;
    int min_x=32767,max_x=-32768,min_y=32767,max_y=-32768;
    for(unsigned v=0;v<count;++v){
        const auto* point=points[v];
        auto& p=projected[v];
        if(gte_matrix){
            int32_t camera[3];ordered_gte_camera_reference(*gte_matrix,point,gte_offset,camera);
            for(unsigned c=0;c<3;++c)p.camera[c]=camera[c];
        }else{
            Number world[3],camera[3];for(unsigned c=0;c<3;++c)world[c]=Number(point[c],Number::RAW);
            view.point(world,camera);camera[0]*=row_scale[0];camera[1]*=row_scale[1];
            for(unsigned c=0;c<3;++c)p.camera[c]=camera[c].raw()>>4;
        }
        depth[v]=p.camera[2];p.fog=0;
        uint32_t packed;
        if(gte_matrix&&gte_project_point(p.camera,focal,Width/2,Height/2,packed)){
            p.outcode=Units::outcode(p.camera[0],p.camera[1],p.camera[2]);
            p.visible=p.camera[2]>=Units::near&&p.camera[2]<Units::far;
            p.screen.packed=packed;
        }else project_cpu<Units,Width,Height>(p);
        direct&=p.visible;
        screen[v]=p.visible?p.screen.packed:0;
        if(p.visible){
            const int x=p.screen.x,y=p.screen.y;
            if(x<min_x)min_x=x;if(x>max_x)max_x=x;if(y<min_y)min_y=y;if(y>max_y)max_y=y;
        }
    }
    return direct&&max_x-min_x<=1023&&max_y-min_y<=511;
}
}
