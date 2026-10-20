#pragma once
#include <stdint.h>
namespace psyqo::GTE {
enum class Register {R11R12,R13R21,R22R23,R31R32,R33,TRX,TRY,TRZ,H,OFX,OFY,DQA,DQB,
    VXY0,VZ0,VXY1,VZ1,VXY2,VZ2,MAC1,MAC2,MAC3,SXY0,SXY1,SXY2,SZ1,SZ2,SZ3,FLAG,ZSF3,ZSF4,OTZ,Count};
inline uint32_t registers[unsigned(Register::Count)]={};
enum Safety {Unsafe,Safe};
template<Register R,Safety=Safe> void write(uint32_t value){registers[unsigned(R)]=value;}
template<Register R> void clear(){write<R>(0);}
template<Register R> uint32_t readRaw(){return registers[unsigned(R)];}
namespace Kernels {inline void rt(){} inline void rtps(){} inline void rtpt(){} inline void avsz3(){} inline void avsz4(){}}
}
