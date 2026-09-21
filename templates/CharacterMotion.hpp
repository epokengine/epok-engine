#pragma once
#include <stdint.h>

namespace character_motion {
inline uint32_t root(uint32_t n){
    uint32_t answer=0,bit=1u<<30;
    while(bit>n)bit>>=2;
    while(bit){if(n>=answer+bit){n-=answer+bit;answer=(answer>>1)+bit;}else answer>>=1;bit>>=2;}
    return answer;
}
struct Intent{int32_t x=0,z=0,strength=0;};
inline Intent intent(int32_t x,int32_t z,int32_t dead_zone=491){
    if(x< -4096)x=-4096;if(x>4096)x=4096;
    if(z< -4096)z=-4096;if(z>4096)z=4096;
    const int32_t length=int32_t(root(uint32_t(x*x+z*z)));
    if(length<=dead_zone)return {};
    const int32_t capped=length>4096?4096:length;
    const int32_t strength=(capped-dead_zone)*4096/(4096-dead_zone);
    return {x*strength/length,z*strength/length,strength};
}
inline int32_t heading(int32_t x,int32_t z){
    const int32_t ax=x<0?-x:x,az=z<0?-z:z;
    const int32_t large=ax>az?ax:az,small=ax>az?az:ax;
    if(!large)return 0;
    const int32_t ratio=small*4096/large;
    int32_t angle=ratio*(45*4096+16*(4096-ratio))/4096;
    if(ax>az)angle=90*4096-angle;
    if(z<0)angle=180*4096-angle;
    if(x<0)angle=360*4096-angle;
    return angle;
}
enum class State:uint8_t{Idle,Walk,Run,JumpUp,JumpDown,Land};
struct AnimationState{
    State state=State::Idle;
    State choose(bool grounded,int32_t vertical_speed,int32_t horizontal_speed,bool finished,bool jumped){
        if(jumped)state=State::JumpUp;
        else if(!grounded)state=vertical_speed>0?State::JumpUp:State::JumpDown;
        else if(state==State::JumpUp||state==State::JumpDown)state=State::Land;
        else if(state!=State::Land||finished){
            if(horizontal_speed<491)state=State::Idle;
            else if(horizontal_speed>14746||(state==State::Run&&horizontal_speed>12288))state=State::Run;
            else state=State::Walk;
        }
        return state;
    }
    static bool looping(State value){return value==State::Idle||value==State::Walk||value==State::Run;}
};
}
