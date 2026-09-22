#pragma once
#include <stdint.h>
namespace epok::sprite_detail {
// Full Q12 frustum test without overflow for arbitrary script coordinates.
// Bound Y before multiplying; the extra bound follows from the tighter planes.
inline bool interior(int32_t x,int32_t y,int32_t z){
    return z>=1024&&z<128*4096&&x>=-z&&x<=z&&y>=-z&&y<=z&&4*y>=-3*z&&4*y<=3*z;
}
// Sprite projection historically truncates toward zero (not mesh floor-rounding).
inline int32_t project_ratio(int32_t coordinate,int scale,int32_t depth){
    const int64_t numerator=int64_t(coordinate)*scale;
    if(numerator>=INT32_MIN&&numerator<=INT32_MAX)return int32_t(numerator)/depth;
    return int32_t(numerator/depth);
}
}
