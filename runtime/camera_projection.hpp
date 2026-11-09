#pragma once
#include <stdint.h>
#include "fixed_trigonometry.hpp"

namespace epok::camera_projection {
// Shared Q12 lens and clip rules for console and Native Play screen queries.
inline int32_t focal(int32_t degrees) {
    if(degrees<25*4096)degrees=25*4096;
    if(degrees>120*4096)degrees=120*4096;
    if(degrees==90*4096)return 4096;
    const uint32_t half=uint32_t(degrees)/720u;
    return int32_t((int64_t(cosine_turn12(half))*4096)/sine_turn12(half));
}
// Camera coordinates and focal lengths are Q12; screen output is top-left +Y
// down. Failure leaves output untouched. Near is inclusive, far/right/bottom
// are exclusive, matching the console's existing visible-point contract.
inline bool point(const int32_t* camera,int32_t focal_x,int32_t focal_y,int width,int height,int32_t* screen) {
    if(!camera||!screen||width<=0||height<=0||width>32767||height>32767
        ||focal_x< -32768||focal_x>32768||focal_y<=0||focal_y>32768)return false;
    const int64_t z=camera[2];
    if(z<1024||z>=128*4096)return false;
    // Keep widened coordinates until clipping; a far-off-screen point must
    // never wrap back into the viewport while applying the lens.
    const int64_t x=(int64_t(camera[0])*focal_x)>>12,y=(int64_t(camera[1])*focal_y)>>12;
    const int64_t sx=int64_t(width/2)*4096+x*(width/2)*4096/z;
    const int64_t sy=int64_t(height/2)*4096-y*(height*2/3)*4096/z;
    if(sx<0||sy<0||sx>=int64_t(width)*4096||sy>=int64_t(height)*4096)return false;
    screen[0]=int32_t(sx);screen[1]=int32_t(sy);return true;
}
}
