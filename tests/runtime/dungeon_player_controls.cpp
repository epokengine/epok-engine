#define EPOK_WORLD_GAMEPLAY 1
#include "actor_scene_fixture.hpp"
#ifndef DRAFT_COMBAT_HEADER
#define DRAFT_COMBAT_HEADER "../../resources/templates/dungeon-complete/assets/scripts/DungeonCombat/DungeonPlayerCombat.hpp"
#endif
#include DRAFT_COMBAT_HEADER
#include "CharacterMotion.hpp"
#include "DungeonTargeting.hpp"
#include <cstdio>
using namespace epok;
namespace third_person_targeting {int64_t horizontal_distance_squared(const epok::Transform& from,const epok::ActorData& target){
    const int64_t x=(target.transform.position[0]-from.position[0]).raw();
    const int64_t z=(target.transform.position[2]-from.position[2]).raw();
    return x*x+z*z;
}}
namespace legacy {
constexpr Fixed max_speed=6.0,walk_cycle_speed=2.6,run_cycle_speed=6.0;
constexpr Fixed turn_rate=540.0,orbit_rate=140.0,pitch_rate=70.0;
constexpr Fixed lock_turn_rate=420.0,lock_pitch_rate=180.0;
constexpr Fixed pitch_min=-10.0,pitch_max=55.0,boom_length=5.0,boom_height=1.1;
constexpr Fixed aim_boom_height=1.35,aim_shoulder_offset=0.55;
constexpr Fixed aim_camera_blend_rate=4.0;
constexpr Fixed aim_pitch_min=-35.0,aim_pitch_max=45.0,aim_pose_rate=220.0;
constexpr Fixed target_focus_height=0.8;
Fixed focus_height(const ActorData& a){return a.collider.enabled?a.collider.center[1]*a.transform.scale[1]:target_focus_height;}
constexpr Fixed lock_marker_final_size=0.8;


Fixed shortest_turn(Fixed from,Fixed to){
    constexpr int32_t turn=360*4096;
    int32_t delta=(to.raw()-from.raw())%turn;
    if(delta>turn/2)delta-=turn;else if(delta< -turn/2)delta+=turn;
    return Fixed(delta,Fixed::RAW);
}

// Move toward a target angle by at most the supplied step size.
Fixed approach_angle(Fixed from,Fixed to,Fixed step){
    const auto delta=shortest_turn(from,to);
    if(delta>step)return from+step;
    if(delta< -step)return from-step;
    return to;
}

// Move a scalar value toward a target without overshooting it.
Fixed approach(Fixed from,Fixed to,Fixed step){
    if(from<to-step)return from+step;
    if(from>to+step)return from-step;
    return to;
}

// Keep camera pitch inside the range supported by the controller.
Fixed clamp_pitch(Fixed value){
    if(value<pitch_min)return pitch_min;
    if(value>pitch_max)return pitch_max;
    return value;
}

// Calculate the horizontal heading from the actor to a target.
Fixed target_yaw(const Transform& from,const ActorData& target){
    return Fixed(character_motion::heading((target.transform.position[0]-from.position[0]).raw(),
                                            (target.transform.position[2]-from.position[2]).raw()),Fixed::RAW);
}

// Calculate the camera pitch needed to look at the target's focus point.
// The zero-distance case avoids a square-root and provides a stable fallback.
Fixed target_pitch(const Transform& from,const ActorData& target){
    const int64_t distance=third_person_targeting::horizontal_distance_squared(from,target);
    if(!distance)return target.transform.position[1]>=from.position[1]?pitch_max:pitch_min;
    const Fixed horizontal(int32_t(character_motion::root(uint32_t(distance))),Fixed::RAW);
    // Positive camera pitch looks down, so measure from the orbit origin toward
    // the target instead of using the target-to-origin sign.
    const Fixed vertical=from.position[1]+boom_height-(target.transform.position[1]+focus_height(target));
    return shortest_turn(Fixed(0.0),Fixed(character_motion::heading(vertical.raw(),horizontal.raw()),Fixed::RAW));
}
struct Control {
    enum class CameraMode {Normal,LockOn,FreeAim};
    Fixed camera_yaw=0.0,camera_pitch=18.0,goal_yaw=0.0,goal_pitch=18.0,manual_hold=0.0,aim_camera_blend=0.0,facing=0.0;
    Fixed shoulder_camera_distance=2.15;CameraMode camera_mode=CameraMode::Normal;
    uint8_t lock_marker_intro_frames=8,lock_marker_spin=0;
    void camera(Transform& transform,Fixed dt,bool free_aim,bool moving,ActorData* target,character_motion::Intent observed_look){    // Lock-on and Free Aim share the over-the-shoulder camera placement.
    aim_camera_blend=approach(aim_camera_blend,(free_aim||target)?Fixed(1.0):Fixed(0.0),aim_camera_blend_rate*dt);
    camera_mode=free_aim?CameraMode::FreeAim:(target?CameraMode::LockOn:CameraMode::Normal);

    // Manual orbit and target tracking use the same yaw as movement. Recenter
    // behind the character only at rest: chasing facing while the left stick
    // steers would feed each camera turn back into the requested direction.
    const auto look=observed_look;
    if(look.strength&&camera_mode!=CameraMode::LockOn){
        if(manual_hold.raw()<=0){goal_yaw=camera_yaw;goal_pitch=camera_pitch;}
        manual_hold=1.0;
        goal_yaw+=Fixed(look.x,Fixed::RAW)*orbit_rate*dt;
        goal_pitch=clamp_pitch(goal_pitch-Fixed(look.z,Fixed::RAW)*pitch_rate*dt);
    }else if(manual_hold.raw()>0){manual_hold-=dt;if(manual_hold.raw()<0)manual_hold=0.0;}
    if(camera_mode==CameraMode::LockOn){goal_yaw=target_yaw(transform,*target);goal_pitch=clamp_pitch(target_pitch(transform,*target));}
    else if(camera_mode==CameraMode::Normal&&manual_hold.raw()<=0)goal_yaw=moving?camera_yaw:facing;
    const bool manual=camera_mode!=CameraMode::Normal||manual_hold.raw()>0;
    const Fixed response=(manual?Fixed(12.0):Fixed(4.0))*dt;
    Fixed turn=shortest_turn(camera_yaw,goal_yaw)*response/(Fixed(1.0)+response);
    const Fixed cap=(manual?Fixed(140.0):Fixed(90.0))*dt;
    if(turn>cap)turn=cap;if(turn< -cap)turn=-cap;camera_yaw+=turn;
    if(shortest_turn(camera_yaw,goal_yaw)>Fixed(-.01)&&shortest_turn(camera_yaw,goal_yaw)<Fixed(.01))camera_yaw=goal_yaw;
    const Fixed pitch_response=Fixed(12.0)*dt;
    camera_pitch+=(goal_pitch-camera_pitch)*pitch_response/(Fixed(1.0)+pitch_response);

    // Normalize yaw after orbiting so it remains within one full rotation.
    camera_yaw=Fixed((camera_yaw.raw()%(360*4096)+360*4096)%(360*4096),Fixed::RAW);
    camera_pitch=clamp_pitch(camera_pitch);
}
    DungeonControlView view(const Transform& transform){    // Blend boom length, height, and shoulder offset when entering an aim mode.
    Fixed shoulder_distance=shoulder_camera_distance;
    if(shoulder_distance<Fixed(0.5))shoulder_distance=0.5;
    const Fixed boom=boom_length+(shoulder_distance-boom_length)*aim_camera_blend;
    const Fixed height=boom_height+(aim_boom_height-boom_height)*aim_camera_blend;
    const Fixed shoulder=aim_shoulder_offset*aim_camera_blend;
    const Fixed reach=boom*cos_degrees(camera_pitch);
    const Fixed yaw_cos=cos_degrees(camera_yaw),yaw_sin=sin_degrees(camera_yaw);
    Fixed origin[3]={transform.position[0],transform.position[1]+height,transform.position[2]};
    // Shoulder offset is camera-only; blending it avoids a positional snap when
    // entering or leaving Lock-on and Free Aim.
    Fixed offset[3]={yaw_cos*shoulder-yaw_sin*reach,boom*sin_degrees(camera_pitch),-yaw_sin*shoulder-yaw_cos*reach};


        return {gameplay_vector(origin),gameplay_vector(offset),boom,camera_yaw,camera_pitch};}
    void marker(ActorData* marker){    // The source effect increments one of 64 angles every frame. The imported
    // atlas contains those same 64 rotations in an 8x8 grid.
    lock_marker_spin=(lock_marker_spin+1)&63;
    marker->sprite.region[0]=uint16_t(lock_marker_spin&7)*32;
    marker->sprite.region[1]=uint16_t(lock_marker_spin>>3)*32;
    marker->sprite.region[2]=marker->sprite.region[3]=32;

    // Expand and brighten the marker during its eight-frame introduction.
    if(lock_marker_intro_frames)--lock_marker_intro_frames;
    const int radius=96+64*lock_marker_intro_frames;
    const Fixed scale(int32_t(radius*4096/96),Fixed::RAW);
    marker->sprite.size[0]=marker->sprite.size[1]=lock_marker_final_size*scale;
    const int source_brightness=lock_marker_intro_frames?((9-lock_marker_intro_frames)*16):128;
    const uint8_t tint=uint8_t(source_brightness*2>255?255:source_brightness*2);
    marker->sprite.color[0]=marker->sprite.color[1]=marker->sprite.color[2]=tint;
    marker->sprite.blend=lock_marker_intro_frames?BlendMode::Average:BlendMode::Cutout;
}
    void face(Transform& transform,Fixed dt,character_motion::Intent intent,ActorData* target,bool attack_owns_movement,bool fire_pressed){
        const Fixed movement_yaw=camera_yaw,input_heading(character_motion::heading(intent.x,intent.z),Fixed::RAW);
    // Rotate the character according to the active camera mode and target state.
    if(camera_mode==CameraMode::Normal&&intent.strength){
        const Fixed target=movement_yaw+input_heading;
        facing=approach_angle(facing,target,turn_rate*dt);
    }else if(camera_mode==CameraMode::LockOn){
        if(target){
            const Fixed target_facing=target_yaw(transform,*target);
            // While moving, Denise turns with the requested camera-relative
            // direction. Releasing the stick (or firing) restores target-facing.
            if(intent.strength&&!attack_owns_movement){
                const Fixed movement_facing=camera_yaw+Fixed(character_motion::heading(intent.x,intent.z),Fixed::RAW);
                facing=approach_angle(facing,movement_facing,turn_rate*dt);
            }else if(fire_pressed)facing=target_facing;
            else facing=approach_angle(facing,target_facing,turn_rate*dt);
        }
    }else if(camera_mode==CameraMode::FreeAim){
        // Free aim owns yaw: left-stick strafing does not independently turn the
        // character, while right-stick yaw rotates camera and body together.
        facing=approach_angle(facing,camera_yaw,turn_rate*dt);
    }


    }
};
}
static const char* context="start";static uint32_t context_frame=0;
static void same(Fixed a,Fixed b){if(a!=b){std::fprintf(stderr,"Q12 mismatch %s frame%u %d != %d\n",context,context_frame,a.raw(),b.raw());assert(false);}}
static void same(GameplayVector3 a,GameplayVector3 b){same(a.x,b.x);same(a.y,b.y);same(a.z,b.z);}
static GameplayVector3 normalize_legacy(GameplayVector3 value){
    Fixed direction[]={value.x,value.y,value.z};int64_t n=0;for(auto v:direction)n+=int64_t(v.raw())*v.raw();
    const Fixed length(int32_t(character_motion::root(uint32_t(n>>4)))*4,Fixed::RAW);
    if(length.raw())for(auto& v:direction)v/=length;return gameplay_vector(direction);
}
int main(){
    DungeonPlayerCombat rules;uint32_t checks=0;
    static_assert(sizeof(DungeonControlCamera)==7*4&&sizeof(DungeonControlCameraInput)<=10*4);
    static_assert(sizeof(DungeonControlView)==9*4&&sizeof(DungeonControlMarker)==5*4);
    for(int32_t x=-4096;x<=4096;x+=257)for(int32_t z=-4096;z<=4096;z+=263)for(uint32_t digital=0;digital<16;++digital){
        int32_t old_x=x,old_z=z;const int dx=int(bool(digital&1))-int(bool(digital&2)),dz=int(bool(digital&4))-int(bool(digital&8));
        if(dx||dz){old_x=dx*4096;old_z=dz*4096;}const auto old=character_motion::intent(old_x,old_z);
        const auto next=rules.control_intent(Fixed(x,Fixed::RAW),Fixed(z,Fixed::RAW),digital&1,digital&2,digital&4,digital&8,false);
        assert(next.x.raw()==old.x&&next.y.raw()==old.z&&next.strength.raw()==old.strength);++checks;
    }
    legacy::Control old;auto camera=rules.control_camera_start(0.0);Transform body{};ActorData target{};
    target.collider.enabled=true;target.collider.center[1]=.8;target.transform.position[0]=3.0;target.transform.position[1]=0.0;target.transform.position[2]=8.0;
    for(auto& value:target.transform.scale)value=1.0;
    const Fixed dt(137,Fixed::RAW);Fixed facing=0.0;
    for(uint32_t frame=0;frame<6000;++frame){
        const bool free=frame%173>=90&&frame%173<135,selected=!free&&frame%97<70,moving=frame%83<50;
        ActorData* chosen=selected?&target:nullptr;
        const int32_t rx=frame%29<13?1234:0,ry=frame%31<11?-700:0;const auto look=character_motion::intent(rx,ry);
        const auto typed_look=rules.control_intent(Fixed(rx,Fixed::RAW),Fixed(ry,Fixed::RAW),false,false,false,false,false);
        const auto angles=chosen?rules.control_target_angles(gameplay_vector(body.position),gameplay_vector(target.transform.position),rules.control_focus_height(true,.8,1.0)):DungeonControlAngles{};
        old.facing=facing;old.camera(body,dt,free,moving,chosen,look);
        camera=rules.control_camera_step(camera,{typed_look,dt,facing,angles.yaw,angles.pitch,free,selected,moving});
        context_frame=frame;context="camera yaw";same(camera.yaw,old.camera_yaw);context="camera pitch";same(camera.pitch,old.camera_pitch);context="blend";same(camera.blend,old.aim_camera_blend);context="hold";same(camera.hold,old.manual_hold);assert(camera.mode==uint32_t(old.camera_mode));checks+=5;
        const auto proposal=rules.control_view_proposal(gameplay_vector(body.position),camera,2.15),legacy_view=old.view(body);
        context="view origin";same(proposal.origin,legacy_view.origin);context="view offset";same(proposal.offset,legacy_view.offset);context="view distance";same(proposal.distance,legacy_view.distance);checks+=3;
        const auto intent=character_motion::intent(frame%71<40?2300:0,frame%79<40?3000:0);
        const auto typed=rules.control_intent(Fixed(intent.x,Fixed::RAW),Fixed(intent.z,Fixed::RAW),false,false,false,false,false);
        // intent above is already rescaled; compare facing with the exact raw
        // vector supplied to both, without applying a second dead zone.
        MovementIntent requested{Fixed(intent.x,Fixed::RAW),Fixed(intent.z,Fixed::RAW),Fixed(intent.strength,Fixed::RAW)};
        const bool owns=frame%53==0,pressed=frame%47==0;
        old.face(body,dt,intent,chosen,owns,pressed);
        const auto move=rules.control_movement_step(requested,camera,facing,angles.yaw,selected,owns,pressed,dt);
        context="facing";same(move.facing,old.facing);facing=move.facing;
        const auto sy=sin_degrees(camera.yaw),cy=cos_degrees(camera.yaw);same(move.x,(sy*requested.y+cy*requested.x)*Fixed(6.0));same(move.z,(cy*requested.y-sy*requested.x)*Fixed(6.0));checks+=3;
        if(chosen){context="target yaw";same(angles.yaw,legacy::target_yaw(body,target));context="target pitch";same(angles.pitch,legacy::target_pitch(body,target));checks+=2;}
    }
    ActorData marker{};legacy::Control mark;Fixed intro=Fixed(8.0),spin=0.0;
    for(uint32_t frame=0;frame<200;++frame){
        mark.marker(&marker);const auto next=rules.control_marker_step(intro,spin);intro=next.intro;spin=next.spin;
        assert(intro.raw()/4096==mark.lock_marker_intro_frames&&spin.raw()/4096==mark.lock_marker_spin);same(next.size,marker.sprite.size[0]);
        assert(next.tint==marker.sprite.color[0]&&next.average==(marker.sprite.blend==BlendMode::Average));checks+=3;
    }
    character_motion::AnimationState animation;uint32_t playing=5,phase=0,previous_state=0;
    for(uint32_t tick=0;tick<12000;++tick){
        const bool grounded=tick%109<100,finished=tick%17==0,jumped=tick%97==0;const Fixed vertical(tick%31<15?4096:-4096,Fixed::RAW),speed(int32_t(tick%30000),Fixed::RAW);
        const auto expected=animation.choose(grounded,vertical.raw(),speed.raw(),finished,jumped);
        const auto actual=rules.control_animation_select(tick?uint32_t(previous_state):0,playing,grounded,vertical,speed,finished,jumped);
        assert(actual.state==uint32_t(expected)&&actual.loop==character_motion::AnimationState::looping(expected));
        const bool change=uint32_t(expected)!=playing;assert(actual.switch_clip==change);if(change){playing=uint32_t(expected);phase=0;}
        if(expected==character_motion::State::Walk||expected==character_motion::State::Run){
            Fixed rate=speed/(expected==character_motion::State::Walk?Fixed(2.6):Fixed(6.0));if(rate<Fixed(.2))rate=.2;if(rate>Fixed(1.6))rate=1.6;
            same(actual.rate,rate);uint32_t old_phase=phase+uint32_t((rate*dt*60).raw());old_phase%=uint32_t((23-1)*2*4096);
            const auto clock=rules.control_animation_clock(phase,23,actual.rate,dt,true);assert(clock.phase==old_phase&&clock.ticks==old_phase/4096);phase=clock.phase;
        }
        previous_state=actual.state;++checks;
    }
    for(int32_t x=-40;x<=40;x+=3)for(int32_t y=-3;y<=3;++y)for(int32_t z=-4;z<=4;++z){
        const GameplayVector3 delta{Fixed(x),Fixed(y),Fixed(z)};const auto direction=rules.control_direction_toward(delta);assert(direction.valid);same(direction.value,normalize_legacy(delta));++checks;
    }
    // Tuning edits are bounded before multiplies/divides. Their out-of-domain
    // behavior is a new explicit guard, not claimed as old-controller parity.
    rules.control_movement={9.0,180.0};const auto movement=rules.control_movement_step({0.0,1.0,1.0},rules.control_camera_start(0.0),0.0,0.0,false,false,false,dt);same(movement.z,9.0);++checks;
    rules.control_camera.initial_pitch=25.0;assert(rules.control_camera_start(0.0).pitch==25.0);
    rules.control_marker={.5,4.0,96.0,32.0,8.0,100.0};assert(rules.control_marker_begin()==4);
    const auto edited_marker=rules.control_marker_step(4.0,63.0);assert(edited_marker.intro==3.0&&edited_marker.spin==0.0&&edited_marker.tint==32);checks+=3;
    rules.control_view.boom=7.0;assert(rules.control_view_proposal({},rules.control_camera_start(0.0),2.15).distance==7.0);++checks;
    rules.control_aim.ray_distance=50.0;assert(rules.control_aim_ray(0.0,0.0).z==50.0);++checks;
    rules.control_animation.walk_cycle=1.0;assert(rules.control_animation_select(0,5,true,0.0,1.0,true,false).rate==1.0);++checks;
    for(uint32_t mask=0;mask<16;++mask){const auto gate=rules.control_gate(mask&1,mask&2,mask&4,mask&8);assert(gate==((!(mask&1)||(mask&2)||!(mask&4))?1u:!(mask&8)?2u:0u));++checks;}
    for(uint32_t frames:{0u,1u,2u,65535u,UINT32_MAX})for(uint32_t start:{0u,1u,UINT32_MAX}){
        const auto clock=rules.control_animation_clock(start,frames,Fixed(INT32_MAX,Fixed::RAW),Fixed(1.0),true);assert(clock.phase<2147483648u&&clock.ticks<=524288u);++checks;}
    for(Fixed step:{Fixed(64.0),Fixed(-100.0),Fixed(INT32_MIN,Fixed::RAW),Fixed(INT32_MAX,Fixed::RAW)}){
        const auto safe=rules.control_direction_toward({step,step,step});assert(!safe.valid);++checks;}
    assert(rules.control_rig(13).bone==2&&rules.control_rig(12).bone==-1);checks+=2;
    Fixed pose=0.0,legacy_pose=0.0;
    for(uint32_t n=0;n<1000;++n){const bool free=n%73<32,targeted=n%49<20;const Fixed camera_pitch=Fixed(int32_t(n%75)-20),target_pitch=Fixed(int32_t(n%93)-40);
        Fixed desired=free?camera_pitch:targeted?target_pitch:Fixed(0.0);if(desired<Fixed(-35.0))desired=-35.0;if(desired>Fixed(45.0))desired=45.0;
        legacy_pose=legacy::approach(legacy_pose,desired,Fixed(220.0)*dt);pose=rules.control_aim_step(pose,camera_pitch,target_pitch,dt,free,targeted);same(pose,legacy_pose);++checks;}
    std::printf("PASS %u controller sequence/input/animation/muzzle/edit/limit checkpoints; frozen source bodies and common mechanisms\n",checks);
}
