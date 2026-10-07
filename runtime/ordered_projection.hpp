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
    const BakedDrawFace& face,const uint8_t* page,const unsigned* corners,unsigned count,
    const CameraSpace<Number>& view,const Number* row_scale,
    ProjectedVertex* projected,uint32_t* screen,int32_t* depth) {
    bool direct=true;
    int min_x=32767,max_x=-32768,min_y=32767,max_y=-32768;
    for(unsigned v=0;v<count;++v){
        const auto* point=reinterpret_cast<const int32_t*>(page+face.indices[corners[v]]);
        Number world[3],camera[3];for(unsigned c=0;c<3;++c)world[c]=Number(point[c],Number::RAW);
        view.point(world,camera);camera[0]*=row_scale[0];camera[1]*=row_scale[1];
        auto& p=projected[v];for(unsigned c=0;c<3;++c)p.camera[c]=camera[c].raw()>>4;
        depth[v]=p.camera[2];p.fog=0;
        project_cpu<Units,Width,Height>(p);direct&=p.visible;
        screen[v]=p.visible?p.screen.packed:0;
        if(p.visible){
            const int x=p.screen.x,y=p.screen.y;
            if(x<min_x)min_x=x;if(x>max_x)max_x=x;if(y<min_y)min_y=y;if(y>max_y)max_y=y;
        }
    }
    return direct&&max_x-min_x<=1023&&max_y-min_y<=511;
}
}
