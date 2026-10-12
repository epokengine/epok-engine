#pragma once
#include <stdint.h>

namespace epok {
// Angles use 4096 steps per turn, weights use 256 = 1. The caller supplies
// angles in one consistent camera convention and owns the update cadence.
// These helpers neither move the camera nor select visibility/streaming pages.
inline int32_t angle_delta_turn12(uint32_t from,uint32_t to) {
    return int32_t((to-from+2048u)&4095u)-2048;
}
inline uint16_t blend_angle_turn12(uint32_t from,uint32_t to,uint16_t weight) {
    // Arithmetic shift intentionally floors negative products, unlike /256.
    return uint16_t((from+uint32_t((angle_delta_turn12(from,to)*int32_t(weight))>>8))&4095u);
}
inline uint16_t smooth_angle_turn12(uint32_t from,uint32_t to,uint16_t weight,uint16_t limit) {
    int32_t difference=angle_delta_turn12(from,to);
    if(difference>int32_t(limit))difference=limit;
    if(difference< -int32_t(limit))difference=-int32_t(limit);
    return uint16_t((from+uint32_t((difference*int32_t(weight))>>8))&4095u);
}
inline uint16_t limit_angle_turn12(uint32_t angle,uint32_t base,uint16_t positive,uint16_t negative) {
    // A positive half-turn stays positive here. Interpolation instead chooses
    // the negative half-turn; making both operations identical changes limits.
    int32_t difference=int32_t(angle&4095u)-int32_t(base&4095u);
    if(difference>2048)difference-=4096;
    if(difference< -2048)difference+=4096;
    if(difference>int32_t(positive))difference=positive;
    if(difference< -int32_t(negative))difference=-int32_t(negative);
    return uint16_t((base+uint32_t(difference))&4095u);
}
struct CameraAimParameters {
    // 0 follows the target; 256 keeps the authored base orientation.
    uint16_t base_weight[3]={256,256,256};
    int16_t target_bias[3]{};
    uint16_t smoothing_weight=256,smoothing_limit=2048;
    uint16_t positive_limit[3]={2048,2048,2048};
    uint16_t negative_limit[3]={2048,2048,2048};
};
inline void blend_camera_aim(const uint16_t* base,const uint16_t* target,
                             const uint16_t* previous,const CameraAimParameters& parameters,
                             uint16_t* result) {
    for(unsigned axis=0;axis<3;++axis){
        const uint32_t biased=uint32_t(target[axis])+uint32_t(int32_t(parameters.target_bias[axis]));
        uint16_t angle=blend_angle_turn12(biased,base[axis],parameters.base_weight[axis]);
        if(previous)angle=smooth_angle_turn12(previous[axis],angle,parameters.smoothing_weight,parameters.smoothing_limit);
        result[axis]=limit_angle_turn12(angle,base[axis],parameters.positive_limit[axis],parameters.negative_limit[axis]);
    }
}
}
