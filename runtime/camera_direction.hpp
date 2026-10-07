#pragma once
#include "fixed_trigonometry.hpp"
#include "camera_aim.hpp"
namespace epok {
namespace camera_direction_detail {
constexpr double atan_series(double x) {
    double term=x,sum=x;
    for(int i=1;i<28;++i){term*= -x*x;sum+=term/double(2*i+1);}
    return sum;
}
struct AtanTable {uint16_t values[1025]{};};
consteval AtanTable make_atan_table() {
    AtanTable table;
    for(unsigned i=0;i<=1024;++i){
        const double x=double(i)/1024.;
        const double a=x>.5?0.78539816339744830962+atan_series((x-1)/(x+1)):atan_series(x);
        table.values[i]=uint16_t(a*(4096./6.28318530717958647692)+.5);
    }
    return table;
}
// Mathematical definition, not a table extracted from an imported game.
inline constexpr auto atan_table=make_atan_table();
}
inline uint16_t atan2_turn12(int64_t y,int64_t x) {
    // Unsigned magnitude handles INT64_MIN without signed negation overflow.
    const bool nx=x<0,ny=y<0;
    uint64_t ax=nx?0u-uint64_t(x):uint64_t(x),ay=ny?0u-uint64_t(y):uint64_t(y);
    if(!(ax|ay))return 0;
    while((ax|ay)>0x3fffffu){ax>>=1;ay>>=1;}
    const uint32_t low=uint32_t(ax<ay?ax:ay),high=uint32_t(ax<ay?ay:ax);
    uint32_t angle=camera_direction_detail::atan_table.values[(low<<10)/high];
    if(ax<=ay)angle=1024-angle;
    if(nx)angle=2048-angle;
    if(ny)angle=0u-angle;
    return uint16_t(angle&4095u);
}
// Rig convention: +pitch looks up, yaw zero looks along -Z, +roll about Z.
// The adapter to Epok's local Euler convention is kept outside this function.
inline void camera_target_direction(const int32_t* camera,const int32_t* target,
                                     bool reflect_x,bool front_only,uint16_t reference_yaw,
                                     uint16_t* result) {
    int64_t x=int64_t(target[0])-camera[0],y=int64_t(target[1])-camera[1],z=int64_t(target[2])-camera[2];
    if(reflect_x)x=-x;
    result[1]=atan2_turn12(-x,-z);result[2]=0;
    const int32_t offset=angle_delta_turn12(reference_yaw,result[1]);
    if(front_only&&(offset< -1024||offset>1024)){result[0]=3072;return;}
    const int64_t horizontal=(-z*cosine_turn12(result[1])-x*sine_turn12(result[1]))>>12;
    result[0]=atan2_turn12(y,horizontal);
}
}
