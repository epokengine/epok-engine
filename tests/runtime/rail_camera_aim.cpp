#include "track_camera_pose.hpp"
#include <cassert>
#include <cmath>
#include <climits>
#include <cstdio>
#include <initializer_list>
using namespace epok;
int main(){
    for(unsigned i=0;i<=1024;++i)
        assert(camera_direction_detail::atan_table.values[i]==unsigned(std::round(std::atan(double(i)/1024)*4096/(2*std::acos(-1.)))));
    unsigned vectors=0;
    for(int x=-100;x<=100;++x)for(int y=-100;y<=100;++y){
        const int expected=int(std::round(std::atan2(double(y),double(x))*4096/(2*std::acos(-1.))));
        const auto angle=atan2_turn12(y,x);
        assert(std::abs(angle_delta_turn12(uint32_t(expected),angle))<=1);++vectors;
        assert(std::abs(angle_delta_turn12(angle,atan2_turn12(int64_t(y)*10000000000ll,int64_t(x)*10000000000ll)))<=1);
    }
    assert(atan2_turn12(INT64_MIN,0)==3072&&atan2_turn12(0,INT64_MIN)==2048);
    int32_t camera[3]{},target[]={0,4096,-4096},rotation[]={0,-180*4096,0};
    CameraAimParameters parameters[2];
    for(auto& p:parameters)for(auto& w:p.base_weight)w=0;
    const uint16_t ids[]={0,1};const uint8_t smoothing[]={1,0};
    RailAimData data{parameters,ids,smoothing,30,false,false};RailAimState state;
    advance_rail_aim(data,state,0,camera,target,rotation,0,0,0,false);
    assert(rotation[0]==-45*4096&&rotation[1]==-180*4096&&state.updates==1);
    // A pause must not keep smoothing or accumulate elapsed time.
    target[1]=-4096;
    for(unsigned i=0;i<60;++i){rotation[0]=0;rotation[1]=-180*4096;advance_rail_aim(data,state,0,camera,target,rotation,0,0,0,false);}
    assert(rotation[0]==-45*4096&&state.updates==1&&state.accumulated==0);
    rotation[0]=0;advance_rail_aim(data,state,0,camera,target,rotation,0,0,0,true);
    assert(rotation[0]==-315*4096&&state.updates==1); // Cut starts without old smoothing.
    parameters[0].smoothing_weight=128;parameters[0].smoothing_limit=45;
    uint16_t final_angles[3]{};
    for(unsigned frame_us:{10000u,20000u,40000u}){
        RailAimState run;target[1]=0;
        rotation[0]=rotation[2]=0;rotation[1]=-180*4096;
        advance_rail_aim(data,run,0,camera,target,rotation,0,0,0,false);
        target[1]=4096;
        for(unsigned us=0;us<1000000;us+=frame_us){
            rotation[0]=rotation[2]=0;rotation[1]=-180*4096;
            advance_rail_aim(data,run,frame_us,camera,target,rotation,0,0,0,false);
        }
        assert(run.updates==31&&run.accumulated==0);
        if(frame_us==10000)for(unsigned c=0;c<3;++c)final_angles[c]=run.angles[c];
        else for(unsigned c=0;c<3;++c)assert(final_angles[c]==run.angles[c]);
    }
    // Reflected rigs keep their world target on the same visual side.
    target[0]=4096;target[1]=0;
    for(bool reflection:{false,true}){
        data.reflect_x=reflection;state={};rotation[0]=rotation[2]=0;rotation[1]=-180*4096;
        advance_rail_aim(data,state,0,camera,target,rotation,0,0,0,false);
        assert(angle_delta_turn12(uint32_t(rotation[1]/360),1536)==0);
    }
    data.reflect_x=false;parameters[0].positive_limit[0]=100;parameters[1].positive_limit[0]=200;
    const auto p=interpolate_rail_aim_parameters(data,0,1,2048);assert(p.positive_limit[0]==150);
    state={};state.initialized=true;state.angles[0]=512;target[0]=0;target[1]=4096;
    rotation[0]=rotation[2]=0;rotation[1]=-180*4096;
    advance_rail_aim(data,state,0,camera,target,rotation,0,1,2048,false);
    assert(rotation[0]==-150*360&&state.angles[0]==512&&state.updates==0);
    uint16_t behind[3];target[1]=0;target[2]=4096;
    camera_target_direction(camera,target,false,true,0,behind);assert(behind[0]==3072&&behind[1]==2048);
    VisibilityTrackSample a{},b{};a.position[0]=INT32_MIN;b.position[0]=INT32_MAX;
    assert(interpolate_track_camera(a,b,2048).position[0]==-1);
    a.rotation[1]=350*4096;b.rotation[1]=10*4096;
    assert(interpolate_track_camera(a,b,2048).rotation[1]==360*4096);
    rail_aim_state.initialized=true;set_visibility_track(nullptr);assert(!rail_aim_state.initialized);
    std::printf("Rail aim: %u direction vectors, fixed cadence, pause, cuts, reflection, envelope and reset passed.\n",vectors);
}
