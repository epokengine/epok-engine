#pragma once
#include <stdint.h>
#include <stddef.h>

namespace epok {
// Version 2, little-endian archive ABI. Texture identity is deliberately kept
// separate from VRAM placement so palette animation / scene banks remain valid.
// No pointers, normal, mutable material or float UVs in the draw loop.
struct BakedDrawFace {
    uint16_t indices[4];
    uint32_t color;     // uniform: final RGB + command; indexed: u16 corner indices 0,1
    uint32_t shaded;    // uniform: unmodulated RGB; indexed: u16 corner indices 2,3
    uint16_t uv[4];
    uint16_t texture;   // 0xffff = untextured
    int16_t depth_bias;
    uint16_t flags;     // bit 0: double sided; bits 1..3: BlendMode; bit 4: indexed RGB8; high byte: GPU opcode
    uint16_t topology;  // 0: general quad, 1: valid triangle, 2: degenerate triangle
};
static_assert(sizeof(BakedDrawFace)==32);
static_assert(offsetof(BakedDrawFace, color)==8 && offsetof(BakedDrawFace, uv)==16);
inline bool baked_indexed_color(const BakedDrawFace& face) { return (face.flags & 0x10) != 0; }
inline uint32_t baked_corner_color(const BakedDrawFace& face, unsigned corner, const uint32_t* palette) {
    if (!baked_indexed_color(face)) return face.color & 0xffffffu;
    const uint32_t pair = corner < 2 ? face.color : face.shaded;
    return palette[(pair >> ((corner & 1) * 16)) & 0xffffu];
}
// Ordered native packets use only two corner layouts: 0,1,2,3 and 1,2,3,3.
// Decode packed pairs once, without per-corner tests or variable shifts. Keep
// texture/CLUT identity separate; this does not bake mutable palette contents.
inline void baked_ordered_colors(const BakedDrawFace& face,bool second_half,
    const uint32_t* palette,uint32_t* output) {
    const uint32_t first=face.color;
    if(!baked_indexed_color(face)){
        output[0]=output[1]=output[2]=output[3]=first&0xffffffu;
        return;
    }
    const uint32_t last=face.shaded;
    if(second_half){
        output[0]=palette[first>>16];output[1]=palette[last&0xffffu];
        output[2]=output[3]=palette[last>>16];
    }else{
        output[0]=palette[first&0xffffu];output[1]=palette[first>>16];
        output[2]=palette[last&0xffffu];output[3]=palette[last>>16];
    }
}
inline uint32_t baked_command(const BakedDrawFace& face) {
    return uint32_t(face.flags & 0xff00u) << 16;
}
inline constexpr uint8_t mesh_editable=1, mesh_baked_draw=2;
}
