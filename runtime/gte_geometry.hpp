#pragma once
#include "affine.hpp"
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
inline void project_geometry_vertex(const int16_t* vertex,const int32_t* offset,int32_t* camera,uint32_t& screen,uint32_t& flags) {
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
}
