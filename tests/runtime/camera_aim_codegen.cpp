#include "camera_aim.hpp"
extern "C" void camera_aim(const uint16_t* base,const uint16_t* target,
                            const uint16_t* previous,const epok::CameraAimParameters* parameters,
                            uint16_t* result) {
    epok::blend_camera_aim(base,target,previous,*parameters,result);
}
