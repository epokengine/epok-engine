#pragma once
#include "camera_direction.hpp"
namespace epok {
struct RailAimData {
    const CameraAimParameters* parameters;
    const uint16_t* samples;
    const uint8_t* smooth_samples;
    uint16_t update_hz;
    bool reflect_x,front_only;
};
struct RailAimState {
    uint16_t angles[3]{};
    uint32_t accumulated=0,updates=0;
    bool initialized=false;
};
inline RailAimState rail_aim_state;
inline CameraAimParameters interpolate_rail_aim_parameters(const RailAimData& data,uint16_t current,uint16_t next,uint32_t fraction) {
    auto result=data.parameters[data.samples[current]];
    if(!fraction)return result;
    const auto& b=data.parameters[data.samples[next]];
    auto interpolate=[&](int32_t a,int32_t b){return a+((b-a)*int32_t(fraction))/4096;};
    for(unsigned c=0;c<3;++c){
        result.base_weight[c]=uint16_t(interpolate(result.base_weight[c],b.base_weight[c]));
        result.target_bias[c]=int16_t(interpolate(result.target_bias[c],b.target_bias[c]));
        result.positive_limit[c]=uint16_t(interpolate(result.positive_limit[c],b.positive_limit[c]));
        result.negative_limit[c]=uint16_t(interpolate(result.negative_limit[c],b.negative_limit[c]));
    }
    result.smoothing_weight=uint16_t(interpolate(result.smoothing_weight,b.smoothing_weight));
    result.smoothing_limit=uint16_t(interpolate(result.smoothing_limit,b.smoothing_limit));
    return result;
}
// Separate from transport and object storage: both PSX and native runners can
// apply the same controller after publishing a ready rail pose. Elapsed time
// excludes paused/loading time. No target or camera authoring transform moves.
inline void advance_rail_aim(const RailAimData& data,RailAimState& state,uint32_t elapsed,
                              const int32_t* camera,const int32_t* target,int32_t* rotation,
                              uint16_t current,uint16_t next,uint32_t fraction,bool cut) {
    const int32_t sign=data.reflect_x?-1:1;
    uint16_t base[3]={uint16_t(uint32_t(-(rotation[0]/360))&4095u),
                     uint16_t(uint32_t(2048+sign*(rotation[1]/360))&4095u),
                     uint16_t(uint32_t(sign*(rotation[2]/360))&4095u)};
    const auto parameters=interpolate_rail_aim_parameters(data,current,next,fraction);
    if(cut){state.initialized=false;state.accumulated=state.updates=0;}
    const bool first=!state.initialized;
    uint32_t steps=0;
    if(first){steps=1;state.initialized=true;}
    else{
        // Bounded elapsed multiplication fits u32 for the cooked 1..120 Hz.
        state.accumulated+=(elapsed>100000?100000:elapsed)*data.update_hz;
        steps=state.accumulated/1000000;state.accumulated%=1000000;
    }
    if(steps){
        uint16_t look[3];camera_target_direction(camera,target,data.reflect_x,data.front_only,base[1],look);
        for(uint32_t step=0;step<steps;++step){
            const bool smooth=!(first&&step==0)&&(!data.smooth_samples||data.smooth_samples[current]);
            blend_camera_aim(base,look,smooth?state.angles:nullptr,parameters,state.angles);++state.updates;
        }
    }
    // Keep the rendered camera within the current sample's authored angular
    // envelope even between fixed aim updates. Do not advance smoothing here.
    uint16_t shown[3];
    for(unsigned c=0;c<3;++c)shown[c]=limit_angle_turn12(state.angles[c],base[c],parameters.positive_limit[c],parameters.negative_limit[c]);
    rotation[0]=-int32_t(shown[0])*360;
    rotation[1]=sign*(int32_t(shown[1])-2048)*360;
    rotation[2]=sign*int32_t(shown[2])*360;
}
}
