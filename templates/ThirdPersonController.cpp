#include "ThirdPersonController.hpp"
#include "CharacterClips.hpp"
using namespace epok;
using character_motion::State;

namespace {
// Tuning. The Blueprint and Lua flavors of this template use the same numbers;
// each one is exactly representable in Q12, so all three reach the same raws.
constexpr Fixed max_speed=6.0,walk_cycle_speed=2.6,run_cycle_speed=6.0;
constexpr Fixed turn_rate=540.0,orbit_rate=140.0,pitch_rate=70.0;
constexpr Fixed pitch_min=-10.0,pitch_max=55.0,boom_length=5.4,boom_height=1.1;
constexpr Fixed gravity=24.0,jump_speed=9.0,stick_to_ground=-0.05,step_height=0.6;
constexpr Fixed ground_response=12.0,air_response=3.0;
constexpr Fixed camera_pull_in=0.04,camera_minimum_boom=0.12;

bool same_name(const char* a,const char* b){
    while(*a&&*a==*b){++a;++b;}return *a==*b;
}
Fixed axis(Axis which){return Fixed(int32_t(input.axis_raw(which)),Fixed::RAW);}
uint32_t clip_frames(const Animator& animator){
    return animator.model&&animator.clip>=0&&size_t(animator.clip)<animator.model->clip_count
        ?uint32_t(animator.model->clips[animator.clip].frames):0;
}
}

void ThirdPersonController::start(Transform& transform){
    facing=transform.rotation[1];camera_yaw=facing;camera_pitch=18.0;
    for(auto& value:velocity)value=0.0;
    grounded=false;animation={};playing_state=State::Land;playback_phase=0.0;
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
    const uint32_t frames=clip_frames(animator);
    const bool finished=frames==0||animator.ticks>=(frames-1)*2;
    const State next=animation.choose(grounded,velocity[1].raw(),speed.raw(),finished,jumped);
    if(next!=playing_state&&clips[unsigned(next)]>=0){
        if(animator.play(clips[unsigned(next)],character_motion::AnimationState::looping(next))){
            playing_state=next;playback_phase=0.0;
        }
    }
    if(next==State::Walk||next==State::Run){
        // The cycle is driven by distance travelled, not by the clock, so the
        // animator is paused and its tick cursor written every frame.
        animator.pause();
        const Fixed rate=MathLibrary::clamp(speed/(next==State::Walk?walk_cycle_speed:run_cycle_speed),Fixed(0.2),Fixed(1.6));
        playback_phase+=rate*dt*Fixed(60.0);
        const uint32_t playing=clip_frames(animator);
        // Bounded on purpose, so the Blueprint and Lua flavors express exactly
        // this: the phase advances by at most 1.6 ticks in a fixed step and the
        // shortest clip loops over two, so four passes always suffice.
        const Fixed period(int32_t((playing>1?playing-1:1)*2),0);
        for(int pass=0;pass<4;++pass)if(playback_phase>=period)playback_phase-=period;
        animator.ticks=uint32_t(playback_phase.raw()/4096);
    }
}

void ThirdPersonController::place_camera(const Transform& transform){
    auto* camera=camera_handle.get();if(!camera)return;
    const Fixed reach=boom_length*MathLibrary::cosine_degrees(camera_pitch);
    const Fixed yaw_cos=MathLibrary::cosine_degrees(camera_yaw),yaw_sin=MathLibrary::sine_degrees(camera_yaw);
    Fixed origin[3]={transform.position[0],transform.position[1]+boom_height,transform.position[2]};
    Fixed offset[3]={-yaw_sin*reach,boom_length*MathLibrary::sine_degrees(camera_pitch),-yaw_cos*reach};
    const auto hit=raycast(origin,offset,0xffffffffu,get_owner()->data());
    if(hit){
        // Pull the boom in short of the surface so the near plane never clips it.
        Fixed fraction=hit.fraction-camera_pull_in;
        if(fraction<camera_minimum_boom)fraction=camera_minimum_boom;
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

    const auto look=MathLibrary::stick_intent(axis(Axis::RightX),axis(Axis::RightY));
    camera_yaw=MathLibrary::wrap_degrees(camera_yaw+look.x*orbit_rate*dt);
    camera_pitch=MathLibrary::clamp(camera_pitch-look.y*pitch_rate*dt,pitch_min,pitch_max);

    // A held direction is full deflection, so the keyboard and the stick agree.
    Fixed move_x=axis(Axis::LeftX),move_z=axis(Axis::LeftY);
    const int digital_x=int(input.held(Button::Right))-int(input.held(Button::Left));
    const int digital_z=int(input.held(Button::Up))-int(input.held(Button::Down));
    if(digital_x||digital_z){move_x=Fixed(digital_x,0);move_z=Fixed(digital_z,0);}
    const auto intent=MathLibrary::stick_intent(move_x,move_z);
    const Fixed strafe=intent.x,forward=intent.y;
    const Fixed yaw_sin=MathLibrary::sine_degrees(camera_yaw),yaw_cos=MathLibrary::cosine_degrees(camera_yaw);
    const Fixed desired_x=(yaw_sin*forward+yaw_cos*strafe)*max_speed;
    const Fixed desired_z=(yaw_cos*forward-yaw_sin*strafe)*max_speed;
    if(intent.strength.raw()){
        const Fixed target=camera_yaw+MathLibrary::heading_degrees(intent.x,intent.y);
        facing=MathLibrary::move_toward_degrees(facing,target,turn_rate*dt);
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
        // Lift, retry the blocked part of the step, then settle back down. A
        // curb or a ramp lip is walked over; a wall still stops the character.
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
    const Fixed speed=MathLibrary::length2(
        (transform.position[0]-before[0])/dt,
        (transform.position[2]-before[2])/dt);
    animate(dt,speed,jumped);
    place_camera(transform);
}
