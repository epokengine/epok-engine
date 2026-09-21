#include "ThirdPersonController.hpp"
#include "CharacterClips.hpp"
#include "world2d.hpp"
using namespace epok;
using character_motion::State;

namespace {
constexpr Fixed max_speed=6.0,walk_cycle_speed=2.6,run_cycle_speed=6.0;
constexpr Fixed turn_rate=540.0,orbit_rate=140.0,pitch_rate=70.0;
constexpr Fixed pitch_min=-10.0,pitch_max=55.0,boom_length=5.4,boom_height=1.1;
constexpr Fixed gravity=24.0,jump_speed=9.0,stick_to_ground=-0.05,step_height=0.6;
constexpr Fixed ground_response=12.0,air_response=3.0;

bool same_name(const char* a,const char* b){
    while(*a&&*a==*b){++a;++b;}return *a==*b;
}
Fixed shortest_turn(Fixed from,Fixed to){
    constexpr int32_t turn=360*4096;
    int32_t delta=(to.raw()-from.raw())%turn;
    if(delta>turn/2)delta-=turn;else if(delta< -turn/2)delta+=turn;
    return Fixed(delta,Fixed::RAW);
}
Fixed approach_angle(Fixed from,Fixed to,Fixed step){
    const auto delta=shortest_turn(from,to);
    if(delta>step)return from+step;
    if(delta< -step)return from-step;
    return to;
}
Fixed clamp_pitch(Fixed value){
    if(value<pitch_min)return pitch_min;
    if(value>pitch_max)return pitch_max;
    return value;
}
}

void ThirdPersonController::start(Transform& transform){
    facing=transform.rotation[1];camera_yaw=facing;camera_pitch=18.0;
    for(auto& value:velocity)value=0.0;
    grounded=false;animation={};playing_state=State::Land;playback_phase=0;
    visual_handle={};camera_handle={};for(auto& clip:clips)clip=-1;

    if(auto* visual=find_actor_data("Visual")){
        visual_handle=handle(visual);
        if(const auto* model=visual->animator.model){
            for(unsigned slot=0;slot<6;++slot)
                for(size_t index=0;index<model->clip_count;++index)
                    if(model->clips[index].name&&same_name(model->clips[index].name,character_clips::names[slot]))
                        clips[slot]=int(index);
        }
    }
    if(auto* camera=find_actor_data("Camera")){
        camera_handle=handle(camera);set_active_camera(camera);
    }
    place_camera(transform);
}

void ThirdPersonController::animate(Fixed dt,Fixed speed,bool jumped){
    auto* visual=visual_handle.get();if(!visual)return;
    auto& animator=visual->animator;
    const bool finished=!animator.model||animator.clip<0||size_t(animator.clip)>=animator.model->clip_count||
        animator.ticks>=uint32_t(animator.model->clips[animator.clip].frames-1)*2;
    const State next=animation.choose(grounded,velocity[1].raw(),speed.raw(),finished,jumped);
    if(next!=playing_state&&clips[unsigned(next)]>=0){
        if(animator.play(clips[unsigned(next)],character_motion::AnimationState::looping(next))){
            playing_state=next;playback_phase=0;
        }
    }
    if(next==State::Walk||next==State::Run){
        animator.pause();
        Fixed rate=speed/(next==State::Walk?walk_cycle_speed:run_cycle_speed);
        if(rate<Fixed(0.2))rate=0.2;if(rate>Fixed(1.6))rate=1.6;
        playback_phase+=uint32_t((rate*dt*60).raw());
        if(animator.model&&animator.clip>=0){
            const uint32_t frames=animator.model->clips[animator.clip].frames;
            const uint32_t period=(frames>1?frames-1:1)*2;
            playback_phase%=period*4096;animator.ticks=playback_phase/4096;
        }
    }
}

void ThirdPersonController::place_camera(const Transform& transform){
    auto* camera=camera_handle.get();if(!camera)return;
    const Fixed reach=boom_length*cos_degrees(camera_pitch);
    const Fixed yaw_cos=cos_degrees(camera_yaw),yaw_sin=sin_degrees(camera_yaw);
    Fixed origin[3]={transform.position[0],transform.position[1]+boom_height,transform.position[2]};
    Fixed offset[3]={-yaw_sin*reach,boom_length*sin_degrees(camera_pitch),-yaw_cos*reach};
    const auto hit=raycast(origin,offset,0xffffffffu,get_owner()->data());
    if(hit){
        Fixed fraction=hit.fraction-Fixed(0.04);
        if(fraction<Fixed(0.12))fraction=0.12;
        for(auto& value:offset)value*=fraction;
    }
    for(int i=0;i<3;++i)camera->transform.position[i]=origin[i]+offset[i];
    camera->transform.rotation[0]=camera_pitch;
    camera->transform.rotation[1]=camera_yaw;
    camera->transform.rotation[2]=0.0;
}

void ThirdPersonController::update(Transform& transform,Fixed dt){
    if(dt.raw()<=0)return;
    auto* body=get_owner()->data();

    const auto look=character_motion::intent(input.axis_raw(Axis::RightX),input.axis_raw(Axis::RightY));
    camera_yaw+=Fixed(look.x,Fixed::RAW)*orbit_rate*dt;
    camera_pitch=clamp_pitch(camera_pitch+Fixed(-look.z,Fixed::RAW)*pitch_rate*dt);
    camera_yaw=Fixed((camera_yaw.raw()%(360*4096)+360*4096)%(360*4096),Fixed::RAW);

    int32_t x=input.axis_raw(Axis::LeftX),z=input.axis_raw(Axis::LeftY);
    const int digital_x=int(input.held(Button::Right))-int(input.held(Button::Left));
    const int digital_z=int(input.held(Button::Up))-int(input.held(Button::Down));
    if(digital_x||digital_z){x=digital_x*4096;z=digital_z*4096;}
    const auto intent=character_motion::intent(x,z);
    const Fixed strafe(intent.x,Fixed::RAW),forward(intent.z,Fixed::RAW);
    const Fixed yaw_sin=sin_degrees(camera_yaw),yaw_cos=cos_degrees(camera_yaw);
    const Fixed desired_x=(yaw_sin*forward+yaw_cos*strafe)*max_speed;
    const Fixed desired_z=(yaw_cos*forward-yaw_sin*strafe)*max_speed;
    if(intent.strength){
        const Fixed target=camera_yaw+Fixed(character_motion::heading(intent.x,intent.z),Fixed::RAW);
        facing=approach_angle(facing,target,turn_rate*dt);
    }

    Fixed response=(grounded?ground_response:air_response)*dt;
    if(response>Fixed(1.0))response=1.0;
    velocity[0]+=(desired_x-velocity[0])*response;
    velocity[2]+=(desired_z-velocity[2])*response;

    const bool jumped=grounded&&input.pressed(Button::Cross);
    if(grounded)velocity[1]=jumped?jump_speed:Fixed(0.0);
    else velocity[1]-=gravity*dt;
    const Fixed fall=grounded&&!jumped?stick_to_ground:velocity[1]*dt;
    const Fixed movement[3]={velocity[0]*dt,fall,velocity[2]*dt};
    const Fixed before[3]={transform.position[0],transform.position[1],transform.position[2]};
    const auto result=move_and_slide(*body,movement);
    grounded=result.grounded;
    if(velocity[1]>Fixed(0.0)&&result.displacement[1]+Fixed(0.001)<fall)velocity[1]=0.0;
    if(result.blocked&&grounded&&!jumped){
        const Fixed remaining[3]={movement[0]-result.displacement[0],0.0,movement[2]-result.displacement[2]};
        if(remaining[0].raw()||remaining[2].raw()){
            const Fixed lift[3]={0.0,step_height,0.0};
            const auto lifted=move_and_slide(*body,lift);
            move_and_slide(*body,remaining);
            const Fixed settle[3]={0.0,-lifted.displacement[1]+stick_to_ground,0.0};
            grounded=move_and_slide(*body,settle).grounded;
        }
    }

    transform.rotation[1]=facing;
    const int32_t speed_x=((transform.position[0]-before[0])/dt).raw();
    const int32_t speed_z=((transform.position[2]-before[2])/dt).raw();
    const Fixed speed(int32_t(character_motion::root(uint32_t(int64_t(speed_x)*speed_x+int64_t(speed_z)*speed_z))),Fixed::RAW);
    animate(dt,speed,jumped);
    place_camera(transform);
}
