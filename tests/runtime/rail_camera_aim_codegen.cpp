#include "rail_camera_aim.hpp"
extern "C" void rail_camera_aim(const epok::RailAimData* data,epok::RailAimState* state,uint32_t elapsed,
                                const int32_t* camera,const int32_t* target,int32_t* rotation,
                                uint16_t current,uint16_t next,uint32_t fraction,bool cut) {
    epok::advance_rail_aim(*data,*state,elapsed,camera,target,rotation,current,next,fraction,cut);
}
