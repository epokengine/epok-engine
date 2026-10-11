#pragma once
#include <stdint.h>
#include <stddef.h>

namespace epok {
// Version 3, little-endian archive ABI. The texture words are resolved to
// VRAM at cook time and stored exactly as GT3/GT4 packets take them, as the
// reference renderer reads its texture words from a table: no texture lookup
// or word assembly per face. Palette CONTENTS are still not baked.
// No pointers, normal, mutable material or float UVs in the draw loop.
struct BakedDrawFace {
    uint16_t indices[4];
    uint32_t color;     // uniform: final RGB + command; indexed: u16 corner indices 0,1
    uint32_t shaded;    // uniform: unmodulated RGB; indexed: u16 corner indices 2,3
    uint32_t uv_clut;   // UV0 | CLUT << 16
    uint32_t uv_tpage;  // UV1 | TPAGE << 16; TPAGE bits 12..15 hold the face flags below
    uint32_t uv34;      // UV2 | UV3 << 16
    int16_t depth_bias;
    uint16_t topology;  // 0: general quad, 1: valid triangle, 2: degenerate triangle
};
static_assert(sizeof(BakedDrawFace)==32);
static_assert(offsetof(BakedDrawFace, color)==8 && offsetof(BakedDrawFace, uv_clut)==16);
// Face flags in uv_tpage. The GPU ignores these texpage bits; packets never
// carry them (baked_texture_words clears them).
inline constexpr uint32_t baked_face_double_sided=1u<<28,baked_face_semi_transparent=1u<<29,
    baked_face_textured=1u<<30,baked_face_indexed_color=1u<<31,baked_face_flags=0xf0000000u;
inline bool baked_indexed_color(const BakedDrawFace& face) { return (face.uv_tpage & baked_face_indexed_color) != 0; }
inline bool baked_double_sided(const BakedDrawFace& face) { return (face.uv_tpage & baked_face_double_sided) != 0; }
// Without a resident texture bank, faces draw untextured, as before the
// words were resolved at cook time: callers pass textures_ready=false.
inline bool baked_textured(const BakedDrawFace& face,bool textures_ready=true) {
    return textures_ready && (face.uv_tpage & baked_face_textured) != 0;
}
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
// GPU opcode: 0x30, plus 0x04 textured and 0x02 semi-transparent (only ever
// set on textured faces), from the flag bits 30 and 29.
inline uint32_t baked_command(const BakedDrawFace& face,bool textures_ready=true) {
    const uint32_t flags=textures_ready?face.uv_tpage:face.uv_tpage&~(baked_face_textured|baked_face_semi_transparent);
    return 0x30000000u|((flags>>4)&0x06000000u);
}
// A corner's packed UV (0..3).
inline uint16_t baked_uv(const BakedDrawFace& face,unsigned corner) {
    return corner==0?uint16_t(face.uv_clut):corner==1?uint16_t(face.uv_tpage):
        corner==2?uint16_t(face.uv34):uint16_t(face.uv34>>16);
}
// mesh_static_world: a baked chunk cooked in world space (visibility track
// static_world), drawn with the camera's matrix instead of its actor's.
inline constexpr uint8_t mesh_editable=1, mesh_baked_draw=2, mesh_static_world=4;
}
