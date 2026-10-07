#pragma once
#include "affine.hpp"
#include "mesh_vertices.hpp"
#include "psyqo/gte-registers.hh"
#include "psyqo/gte-kernels.hh"

namespace epok {
// Use MAC1..3, not the saturated 16-bit IR registers: the existing renderer's
// Q12 camera coordinates regularly exceed +/-8 world units. Perspective and
// near/far clipping retain the software path and its full coordinate range.
template<class Number> bool load_geometry_matrix(const Affine<Number>& m) {
    using namespace psyqo::GTE;
    for(int r=0;r<3;++r){
        for(int c=0;c<3;++c)if(m.values[r][c].raw()<-32768||m.values[r][c].raw()>32767)return false;
        // Leave room for three worst-case signed 16x16 products in MAC/4096.
        if(m.values[r][3].raw()<-2146697215||m.values[r][3].raw()>2146697215)return false;
    }
    auto pair=[](int32_t a,int32_t b){return uint32_t(uint16_t(a))|(uint32_t(uint16_t(b))<<16);};
    write<Register::R11R12>(pair(m.values[0][0].raw(),m.values[0][1].raw()));
    write<Register::R13R21>(pair(m.values[0][2].raw(),m.values[1][0].raw()));
    write<Register::R22R23>(pair(m.values[1][1].raw(),m.values[1][2].raw()));
    write<Register::R31R32>(pair(m.values[2][0].raw(),m.values[2][1].raw()));
    write<Register::R33>(uint32_t(m.values[2][2].raw()));
    write<Register::TRX>(uint32_t(m.values[0][3].raw()));
    write<Register::TRY>(uint32_t(m.values[1][3].raw()));
    write<Register::TRZ>(uint32_t(m.values[2][3].raw()));
    return true;
}
inline void transform_geometry_vertex(const int16_t* vertex,int32_t* camera) {
    using namespace psyqo::GTE;
    write<Register::VXY0>(uint32_t(uint16_t(vertex[0]))|(uint32_t(uint16_t(vertex[1]))<<16));
    write<Register::VZ0>(uint32_t(int32_t(vertex[2])));
    Kernels::rt();
    camera[0]=int32_t(readRaw<Register::MAC1>());
    camera[1]=int32_t(readRaw<Register::MAC2>());
    camera[2]=int32_t(readRaw<Register::MAC3>());
}

// Perspective path. Vertices drop four bits (Q12 -> Q8, one unit = 256) so the
// camera-space result and screen Z fit the GTE's 16-bit projection stage while
// the rotation/scale coefficients keep their full Q12 precision. Row Y is
// negated because the GPU's Y axis points down. Only the translation is scaled;
// a bounded vertex fraction (at most 15/16 of a Q12 tick per axis) is dropped.
inline constexpr uint32_t gte_projection_flags = (1u << 24) | (1u << 23) | (1u << 18) | (1u << 17) | (1u << 14) | (1u << 13);
// Largest Q8 chunk origin that keeps origin + any int16 Q12 vertex within the
// GTE's 16-bit vector inputs.
inline constexpr int32_t gte_shared_origin_limit = 32767 - 2048;
inline bool gte_vertex_bounds_fit(const int32_t* origin8,const int32_t* center12,const int32_t* extent12) {
    for(int c=0;c<3;++c) {
        const int64_t low=int64_t(origin8[c])+((int64_t(center12[c])-extent12[c])>>4);
        const int64_t high=int64_t(origin8[c])+((int64_t(center12[c])+extent12[c])>>4);
        if(low<-32768||high>32767)return false;
    }
    return true;
}
template<class Number> bool load_projection_matrix(const Affine<Number>& m) {
    using namespace psyqo::GTE;
    for(int r=0;r<3;++r){
        for(int c=0;c<3;++c)if(m.values[r][c].raw()<-32767||m.values[r][c].raw()>32767)return false;
        if(m.values[r][3].raw()<-2146697215||m.values[r][3].raw()>2146697215)return false;
    }
    auto pair=[](int32_t a,int32_t b){return uint32_t(uint16_t(a))|(uint32_t(uint16_t(b))<<16);};
    write<Register::R11R12>(pair(m.values[0][0].raw(),m.values[0][1].raw()));
    write<Register::R13R21>(pair(m.values[0][2].raw(),-m.values[1][0].raw()));
    write<Register::R22R23>(pair(-m.values[1][1].raw(),-m.values[1][2].raw()));
    write<Register::R31R32>(pair(m.values[2][0].raw(),m.values[2][1].raw()));
    write<Register::R33>(uint32_t(m.values[2][2].raw()));
    write<Register::TRX>(uint32_t(m.values[0][3].raw()>>4));
    write<Register::TRY>(uint32_t(-(m.values[1][3].raw()>>4)));
    write<Register::TRZ>(uint32_t(m.values[2][3].raw()>>4));
    return true;
}
// H is the pixel focal length shared by both axes; offsets are the screen centre.
inline void load_projection_screen(int32_t focal,int32_t centre_x,int32_t centre_y) {
    using namespace psyqo::GTE;
    write<Register::H>(uint32_t(focal));
    write<Register::OFX>(uint32_t(centre_x<<16));
    write<Register::OFY>(uint32_t(centre_y<<16));
    clear<Register::DQA>();
    clear<Register::DQB>();
}
// Camera output is Q8 with Y up. `offset` is the chunk origin in Q8 added to
// each vertex so every chunk of an object shares one matrix and identical GTE
// inputs for shared vertices. `screen` holds SXY2; `flags` is the GTE FLAG
// register, valid only when no gte_projection_flags bit is set.
template<class Vertex> inline void project_geometry_vertex(const Vertex& vertex,const int32_t* offset,int32_t* camera,uint32_t& screen,uint32_t& flags) {
    using namespace psyqo::GTE;
    write<Register::VXY0>(uint32_t(uint16_t(offset[0]+(vertex[0]>>4)))|(uint32_t(uint16_t(offset[1]+(vertex[1]>>4)))<<16));
    write<Register::VZ0>(uint32_t(int32_t(offset[2]+(vertex[2]>>4))));
    Kernels::rtps();
    camera[0]=int32_t(readRaw<Register::MAC1>());
    camera[1]=-int32_t(readRaw<Register::MAC2>());
    camera[2]=int32_t(readRaw<Register::MAC3>());
    screen=readRaw<Register::SXY2>();
    flags=readRaw<Register::FLAG>();
}
// Interior retained chunks never need camera X/Y for software clipping. RTPT
// preserves all three screen/depth results in its FIFOs (not MAC1..3).
// The ordered pass checks camera-relative inputs once, then retains exactly
// the packed words consumed by the GTE. Offset is Q8 derived from signed Q12.
struct GteProjectionInput {uint32_t xy,z;};
static_assert(sizeof(GteProjectionInput)==8 && alignof(GteProjectionInput)>=4);
// Caller validates the signed Q8 translation and retains the loaded rotation.
inline void load_projection_translation(const int32_t* translation) {
    using namespace psyqo::GTE;
    write<Register::TRX>(uint32_t(translation[0]));
    write<Register::TRY>(uint32_t(-translation[1]));
    write<Register::TRZ>(uint32_t(translation[2]));
}
inline bool pack_projection_input(const int32_t* world,const int32_t* offset,GteProjectionInput& out) {
    const int32_t x=(world[0]>>4)+offset[0],y=(world[1]>>4)+offset[1],z=(world[2]>>4)+offset[2];
    out.xy=uint32_t(uint16_t(x))|(uint32_t(uint16_t(y))<<16);out.z=uint32_t(z);
    return (uint32_t(x+32768)<=65535u)&(uint32_t(y+32768)<=65535u)&(uint32_t(z+32768)<=65535u);
}
// Feed each prepared corner straight into its vector registers. Loading an
// out-of-range value is harmless: the caller must not issue RTPT unless ALL
// corners fit, and must reload all three after changing the projection anchor.
// Unlike begin_projection_triple, this path needs no temporary RAM array.
template<unsigned Slot>
inline bool load_projection_input(const int32_t* world,const int32_t* offset) {
    static_assert(Slot<3);
    GteProjectionInput input;
    const bool fits=pack_projection_input(world,offset,input);
#if defined(__mips__) && !defined(PS1_PC_PORT)
    asm volatile(".set push\n.set noreorder\nmtc2 %0, $%2\nmtc2 %1, $%3\n.set pop\n"
        : : "r"(input.xy), "r"(input.z), "i"(Slot*2), "i"(Slot*2+1));
#else
    using namespace psyqo::GTE;
    constexpr Register xy=Slot==0?Register::VXY0:Slot==1?Register::VXY1:Register::VXY2;
    constexpr Register z=Slot==0?Register::VZ0:Slot==1?Register::VZ1:Register::VZ2;
    write<xy>(input.xy);write<z>(input.z);
#endif
    return fits;
}
inline void begin_loaded_projection_triple() {
#if defined(__mips__) && !defined(PS1_PC_PORT)
    // Do not depend on the optimizer leaving two independent instructions
    // between the final MTC2 and RTPT (including a branch delay slot).
    asm volatile(".set push\n.set noreorder\nnop\nnop\ncop2 0x0280030\n.set pop\n");
#else
    psyqo::GTE::Kernels::rtpt();
#endif
}
inline void begin_projection_triple(const GteProjectionInput* vertices) {
#if defined(__mips__) && !defined(PS1_PC_PORT)
    // Batch independent vector-register loads. Only the last LWC2 needs the
    // two-instruction separation before RTPT; earlier loads fill that gap for
    // one another. Keep the input memory operand: a pointer alone would not
    // tell the compiler that the just-packed words must already be stored.
    asm volatile(
        ".set push\n.set noreorder\n"
        "lwc2 $0, 0(%0)\nlwc2 $1, 4(%0)\n"
        "lwc2 $2, 8(%0)\nlwc2 $3, 12(%0)\n"
        "lwc2 $4, 16(%0)\nlwc2 $5, 20(%0)\n"
        "nop\nnop\ncop2 0x0280030\n.set pop\n"
        : : "r"(vertices), "m"(*reinterpret_cast<const GteProjectionInput (*)[3]>(vertices)));
#else
    using namespace psyqo::GTE;
    write<Register::VXY0>(vertices[0].xy);write<Register::VZ0>(vertices[0].z);
    write<Register::VXY1>(vertices[1].xy);write<Register::VZ1>(vertices[1].z);
    write<Register::VXY2>(vertices[2].xy);write<Register::VZ2>(vertices[2].z);
    Kernels::rtpt();
#endif
}
inline bool finish_projection_triple(uint32_t* screen,int32_t* depth) {
    using namespace psyqo::GTE;
    if(readRaw<Register::FLAG>() & gte_projection_flags)return false;
#if defined(__mips__) && !defined(PS1_PC_PORT)
    // SWC2 transfers directly to the arrays, without MFC2/load-delay/SW pairs.
    // The safe FLAG read above waits for RTPT and rejects the same flags as the
    // scalar path. Output memory operands prevent stale/reordered array reads.
    asm volatile(
        "swc2 $12, 0(%2)\nswc2 $13, 4(%2)\nswc2 $14, 8(%2)\n"
        "swc2 $17, 0(%3)\nswc2 $18, 4(%3)\nswc2 $19, 8(%3)\n"
        : "=m"(*reinterpret_cast<uint32_t (*)[3]>(screen)),
          "=m"(*reinterpret_cast<int32_t (*)[3]>(depth)) : "r"(screen), "r"(depth));
#else
    screen[0]=readRaw<Register::SXY0>();screen[1]=readRaw<Register::SXY1>();screen[2]=readRaw<Register::SXY2>();
    depth[0]=int32_t(readRaw<Register::SZ1>());depth[1]=int32_t(readRaw<Register::SZ2>());depth[2]=int32_t(readRaw<Register::SZ3>());
#endif
    return true;
}
inline bool project_packed_vertex(const GteProjectionInput& vertex,uint32_t& screen,int32_t& depth) {
#if defined(__mips__) && !defined(PS1_PC_PORT)
    uint32_t flags;
    // Preserve MAC3 (signed, unlike SZ3) and the one-instruction CFC2 CPU-load
    // delay. There are still two NOPs between the final input and RTPS.
    asm volatile(
        ".set push\n.set noreorder\n"
        "lwc2 $0, 0(%3)\nlwc2 $1, 4(%3)\nnop\nnop\ncop2 0x0180001\n"
        "swc2 $27, 0(%4)\nswc2 $14, 0(%5)\ncfc2 %0, $31\nnop\n.set pop\n"
        : "=r"(flags), "=m"(screen), "=m"(depth)
        : "r"(&vertex), "r"(&depth), "r"(&screen), "m"(vertex));
    return !(flags & gte_projection_flags);
#else
    using namespace psyqo::GTE;
    write<Register::VXY0>(vertex.xy);write<Register::VZ0>(vertex.z);
    Kernels::rtps();
    depth=int32_t(readRaw<Register::MAC3>());screen=readRaw<Register::SXY2>();
    return !(readRaw<Register::FLAG>() & gte_projection_flags);
#endif
}
template<class Number> inline bool project_geometry_triple(const Number (*vertices)[3],const int32_t* offset,
                                    uint32_t* screen,int32_t* depth) {
    using namespace psyqo::GTE;
    auto xy=[&](unsigned i){return uint32_t(uint16_t(offset[0]+(vertices[i][0]>>4))) |
        (uint32_t(uint16_t(offset[1]+(vertices[i][1]>>4)))<<16);};
    write<Register::VXY0>(xy(0));write<Register::VZ0>(uint32_t(offset[2]+(vertices[0][2]>>4)));
    write<Register::VXY1>(xy(1));write<Register::VZ1>(uint32_t(offset[2]+(vertices[1][2]>>4)));
    write<Register::VXY2>(xy(2));write<Register::VZ2>(uint32_t(offset[2]+(vertices[2][2]>>4)));
    Kernels::rtpt();
    if(readRaw<Register::FLAG>() & gte_projection_flags)return false;
    screen[0]=readRaw<Register::SXY0>();screen[1]=readRaw<Register::SXY1>();screen[2]=readRaw<Register::SXY2>();
    depth[0]=int32_t(readRaw<Register::SZ1>());depth[1]=int32_t(readRaw<Register::SZ2>());depth[2]=int32_t(readRaw<Register::SZ3>());
    return true;
}
inline bool project_geometry_triple(MeshVertices vertices,const int32_t* offset,
                                    uint32_t* screen,int32_t* depth) {
    return vertices.wide() ? project_geometry_triple(vertices.wide_data(),offset,screen,depth)
                          : project_geometry_triple(vertices.narrow_data(),offset,screen,depth);
}
}
