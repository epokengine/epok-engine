#pragma once
#include <stdint.h>

namespace epok {
// GPU-domain words prepared offline. The high half of uv34 carries the fourth
// UV, including when a first-half GT3 ignores it. CLUT/TPAGE never move slots.
struct BakedTextureWords { uint32_t uv_clut,uv_tpage,uv34; };
static_assert(sizeof(BakedTextureWords)==12);
inline BakedTextureWords baked_second_half(BakedTextureWords source) {
    return {(source.uv_clut&0xffff0000u)|(source.uv_tpage&0xffffu),
            (source.uv_tpage&0xffff0000u)|(source.uv34&0xffffu),source.uv34>>16};
}
template<bool Textured,bool Quad> struct BakedPacket {
    static constexpr unsigned words=(Textured?3:2)*(Quad?4:3);
    static constexpr unsigned bytes_with_dma=(words+1)*4;
    // Destination excludes its DMA tag. Caller supplies projected corner order,
    // raw RGB24 and the cooked triangle opcode (quad bit is added here).
    // No lighting, sort, clipping, allocation, projection or asset lookup here.
    static inline void write(uint32_t* output,const uint32_t* rgb,const uint32_t* screen,
        BakedTextureWords uv,uint32_t command) {
        output[0]=(Textured?command:0x30000000u)|(Quad?0x08000000u:0)|rgb[0];
        output[1]=screen[0];
        if constexpr(Textured){
            output[2]=uv.uv_clut;output[3]=rgb[1];output[4]=screen[1];output[5]=uv.uv_tpage;
            output[6]=rgb[2];output[7]=screen[2];output[8]=uv.uv34;
            if constexpr(Quad){output[9]=rgb[3];output[10]=screen[3];output[11]=uv.uv34>>16;}
        }else{
            output[2]=rgb[1];output[3]=screen[1];output[4]=rgb[2];output[5]=screen[2];
            if constexpr(Quad){output[6]=rgb[3];output[7]=screen[3];}
        }
    }
};
// Candidate buckets are already in the ascending-OT coordinate system. The
// renderer chooses the depth mapping and resets this cursor once per pass.
// Reverse traversal + prepend with this policy preserves the prepared order.
template<unsigned Buckets> struct MonotoneDrawBucket {
    static_assert(Buckets>0&&Buckets<=65536);
    uint16_t cursor=Buckets-1;
    uint16_t next(uint16_t candidate){if(candidate<cursor)cursor=candidate;return cursor;}
};
}
