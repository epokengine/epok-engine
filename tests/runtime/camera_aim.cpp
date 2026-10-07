#include "camera_aim.hpp"
#include <cassert>
#include <cstdio>
using namespace epok;
int main(){
    assert(angle_delta_turn12(0,2048)==-2048);
    assert(angle_delta_turn12(2048,0)==-2048);
    assert(blend_angle_turn12(0,4095,128)==4095);
    assert(blend_angle_turn12(4095,0,256)==0);
    assert(smooth_angle_turn12(0,1000,128,45)==22);
    assert(smooth_angle_turn12(0,3096,128,45)==4073);
    assert(limit_angle_turn12(2048,0,100,20)==100);
    assert(limit_angle_turn12(4095,0,100,20)==4095);
    // Exhaustive modular domain, including positive/negative half-turn ties.
    for(unsigned from=0;from<4096;++from)for(unsigned to=0;to<4096;++to){
        const int32_t raw=int32_t(to)-int32_t(from);
        const int32_t difference=raw>=2048?raw-4096:raw< -2048?raw+4096:raw;
        assert(angle_delta_turn12(from,to)==difference);
        assert(blend_angle_turn12(from,to,0)==from);
        assert(blend_angle_turn12(from,to,256)==to);
        assert(blend_angle_turn12(from+8192,to+4096,128)==blend_angle_turn12(from,to,128));
        assert(smooth_angle_turn12(from,to,256,2048)==to);
        assert(limit_angle_turn12(to,from,2048,2048)==to);
    }
    CameraAimParameters params;
    const uint16_t base[]={3870,120,4095},target[]={3600,138,0},previous[]={3875,128,4095};
    uint16_t result[3];blend_camera_aim(base,target,previous,params,result);
    for(unsigned i=0;i<3;++i)assert(result[i]==base[i]);
    params.base_weight[0]=166;params.base_weight[1]=params.base_weight[2]=128;
    params.target_bias[0]=287;params.smoothing_weight=128;params.smoothing_limit=45;
    params.positive_limit[0]=341;params.negative_limit[0]=11;
    params.positive_limit[1]=params.negative_limit[1]=147;
    blend_camera_aim(base,target,previous,params,result);
    assert(result[0]==3875&&result[1]==128&&result[2]==4095);
    blend_camera_aim(base,target,nullptr,params,result);
    assert(result[0]==3875&&result[1]==129&&result[2]==4095);
    // No hidden mutable state. Output may alias either target or previous.
    uint16_t aliased[]={3600,138,0};blend_camera_aim(base,aliased,previous,params,aliased);
    assert(aliased[0]==3875&&aliased[1]==128&&aliased[2]==4095);
    blend_camera_aim(base,target,aliased,params,aliased);
    assert(aliased[0]==3875&&aliased[1]==128&&aliased[2]==4095);
    std::puts("Camera aim: 16777216 angle pairs, wrap, rounding, smoothing, limits and aliasing passed.");
}
