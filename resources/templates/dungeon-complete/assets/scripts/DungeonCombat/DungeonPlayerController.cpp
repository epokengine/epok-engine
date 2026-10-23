#include "DungeonPlayerController.hpp"
#include "CharacterClips.hpp"
#include "DungeonTargeting.hpp"
#include "DungeonGame.hpp"
#include "DungeonProfiling.hpp"
#include "world2d.hpp"
using namespace epok;
using character_motion::State;
namespace {
// Movement, camera, animation, and targeting constants are kept together so
// the controller's tuning values can be adjusted without changing its logic.
constexpr Fixed max_speed=6.0,walk_cycle_speed=2.6,run_cycle_speed=6.0;
constexpr Fixed turn_rate=540.0,orbit_rate=140.0,pitch_rate=70.0;
constexpr Fixed lock_turn_rate=420.0,lock_pitch_rate=180.0;
constexpr Fixed pitch_min=-10.0,pitch_max=55.0,boom_length=5.0,boom_height=1.1;
constexpr Fixed aim_boom_height=1.35,aim_shoulder_offset=0.55;
constexpr Fixed aim_camera_blend_rate=4.0;
constexpr Fixed aim_pitch_min=-35.0,aim_pitch_max=45.0,aim_pose_rate=220.0;
constexpr Fixed target_focus_height=0.8,lock_acquire_distance=13.0,lock_keep_distance=16.0;
Fixed focus_height(const ActorData& a){return a.collider.enabled?a.collider.center[1]*a.transform.scale[1]:target_focus_height;}
constexpr Fixed lock_marker_final_size=0.8;
// This Dungeon's floors and corridor joins are coplanar. Its doors, walls,
// terminals and enemies are taller than a step, so failed stair probes would
// only repeat three more collision queries while walking against them.
constexpr Fixed gravity=24.0,jump_speed=9.0,stick_to_ground=-0.05,step_height=0.0;
constexpr Fixed ground_response=12.0,air_response=3.0;

// Compare two null-terminated names without relying on the standard library.
// This is used to find animation clips by their semantic names.
bool same_name(const char* a,const char* b){
    while(*a&&*a==*b){++a;++b;}return *a==*b;
}

// Return the smallest signed rotation from one angle to another.
// Angles are stored as fixed-point degrees, with 4096 raw units per degree.
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
}

// Initialize the controller and resolve the scene objects it depends on.
void DungeonPlayerController::start(Transform& transform){
    // Reset movement, camera, animation, and target state for a fresh run.
    facing=transform.rotation[1];camera_yaw=facing;camera_pitch=18.0;
    goal_yaw=facing;goal_pitch=18.0;manual_hold=0.0;
    for(auto& v:velocity)v=0.0;
    grounded=false;camera_mode=CameraMode::Normal;aim_pose_pitch=0.0;aim_camera_blend=0.0;visibility_phase=0;
    animation={};playing_state=State::Land;playback_phase=0;
    for(auto& clip:attack_clips)clip=-1;attack_phase=0;attacking=shot_sent=false;
    visual_handle={};camera_handle={};target_handle={};lock_marker_handle={};
    lock_marker_intro_frames=0;lock_marker_spin=0;for(auto& clip:clips)clip=-1;

    // Find the character visual and map its animation clips by name.
    if(auto* visual=find_actor_data(use_aqua_visual?"Character Aqua":"Character Visual")){
        visual_handle=handle(visual);
        if(const auto* model=visual->animator.model){
            // Match semantic names once, so reordering clips cannot change actions.
            for(unsigned slot=0;slot<6;++slot)
                for(size_t index=0;index<model->clip_count;++index)
                    if(model->clips[index].name&&same_name(model->clips[index].name,character_clips::names[slot]))
                        clips[slot]=int(index);
            const char* attacks[]={"CharacterRig|Sequence_08_Attack_2","CharacterRig|Sequence_09_Attack_3","CharacterRig|Sequence_10_Attack_4"};
            for(unsigned slot=0;slot<3;++slot)for(size_t index=0;index<model->clip_count;++index)
                if(model->clips[index].name&&same_name(model->clips[index].name,attacks[slot]))attack_clips[slot]=int(index);
            // Denise's segmented rig has the torso at bone 2. Arms and head
            // inherit its pitch; counter-rotation at both leg roots keeps the
            // lower body aligned with locomotion and the collider.
            if(model->bone_count==13){
                visual->aim_bone=2;
                visual->aim_stop_bones[0]=6;
                visual->aim_stop_bones[1]=7;
            }else{
                visual->aim_bone=-1;
                visual->aim_stop_bones[0]=visual->aim_stop_bones[1]=-1;
            }
        }
        visual->aim_pitch=0.0;
    }

    // Register the follow camera and make it the active camera.
    if(auto* camera=find_actor_data("Follow Camera")){
        camera_handle=handle(camera);set_active_camera(camera);
    }

    // The lock-on marker starts hidden and is enabled only when a target exists.
    if(auto* marker=find_actor_data("Lock-on Marker")){
        lock_marker_handle=handle(marker);
        set_active(marker,false);
    }
    place_camera(transform);
}

// Cycle to the next valid target, or acquire the nearest target when none is selected.
void DungeonPlayerController::toggle_target(ActorData& body){
    const auto* queries=DungeonGame::current?&DungeonGame::current->spatial_queries():nullptr;
    auto* current=target_handle.get();
    if(current&&third_person_targeting::candidate(current)&&third_person_targeting::nearby(body.transform,*current,lock_keep_distance))
        target_handle=third_person_targeting::next(body.transform,target_handle,lock_acquire_distance,queries);
    else target_handle=third_person_targeting::nearest(body.transform,lock_acquire_distance,queries);
    if(auto* selected=target_handle.get();selected&&selected!=current){
        // Tron Bonne's effect starts at r=608, immediately advances to r=544,
        // and reaches r=96 over eight rendered frames.
        lock_marker_intro_frames=8;
        lock_marker_spin=0;
        if(auto* game=DungeonGame::current)game->sound(DungeonSound::LockOn);
    }else if(!target_handle.get())lock_marker_intro_frames=0;
}

// Update target validity, camera mode, and camera orientation for this frame.
void DungeonPlayerController::update_camera(Transform& transform,Fixed dt,bool free_aim,bool moving){
    auto* target=target_handle.get();

    // Drop targets that disappeared, moved too far away, or are no longer eligible.
    if(!target||!third_person_targeting::candidate(target)||!third_person_targeting::nearby(transform,*target,lock_keep_distance)){
        target_handle={};target=nullptr;
    }
    if(target&&++visibility_phase>=8){
        visibility_phase=0;
        const auto* queries=DungeonGame::current?&DungeonGame::current->spatial_queries():nullptr;
        if(!third_person_targeting::visible(transform,*target,queries)){target_handle={};target=nullptr;}
    }

    // Lock-on and Free Aim share the over-the-shoulder camera placement.
    aim_camera_blend=approach(aim_camera_blend,(free_aim||target)?Fixed(1.0):Fixed(0.0),aim_camera_blend_rate*dt);
    camera_mode=free_aim?CameraMode::FreeAim:(target?CameraMode::LockOn:CameraMode::Normal);

    // Manual orbit and target tracking use the same yaw as movement. Recenter
    // behind the character only at rest: chasing facing while the left stick
    // steers would feed each camera turn back into the requested direction.
    const auto look=character_motion::intent(input.axis_raw(Axis::RightX),input.axis_raw(Axis::RightY));
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

// Smooth the visual rig toward the pitch requested by the current aim mode.
void DungeonPlayerController::update_aim_pose(Fixed dt,bool free_aim){
    Fixed desired=free_aim?camera_pitch:Fixed(0.0);
    if(!free_aim)if(auto* target=target_handle.get())desired=target_pitch(get_owner()->data()->transform,*target);
    if(desired<aim_pitch_min)desired=aim_pitch_min;
    if(desired>aim_pitch_max)desired=aim_pitch_max;
    aim_pose_pitch=approach(aim_pose_pitch,desired,aim_pose_rate*dt);
    if(auto* visual=visual_handle.get())visual->aim_pitch=aim_pose_pitch;
}

// Keep the lock-on marker attached to the target and animate its sprite effect.
void DungeonPlayerController::update_lock_marker(){
    auto* marker=lock_marker_handle.get();
    auto* target=target_handle.get();
    if(!marker)return;
    if(!target){
        if(is_active(marker))set_active(marker,false);
        return;
    }

    // Follow the target's focus point and make the marker visible.
    if(!is_active(marker))set_active(marker,true);
    marker->transform.position[0]=target->transform.position[0];
    marker->transform.position[1]=target->transform.position[1]+focus_height(*target);
    marker->transform.position[2]=target->transform.position[2];

    // The source effect increments one of 64 angles every frame. The imported
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

// Select the next animation state and advance locomotion playback manually.
void DungeonPlayerController::animate(Fixed dt,Fixed speed,bool jumped){
    auto* visual=visual_handle.get();if(!visual)return;
    auto& animator=visual->animator;

    // Determine whether the current clip has finished before choosing the next state.
    const bool finished=!animator.model||animator.clip<0||size_t(animator.clip)>=animator.model->clip_count||
        animator.ticks>=uint32_t(animator.model->clips[animator.clip].frames-1)*2;
    const State next=animation.choose(grounded,velocity[1].raw(),speed.raw(),finished,jumped);

    // Switch clips only when the state changes, preserving the current animation otherwise.
    if(next!=playing_state){
        if(animator.play(clips[unsigned(next)],character_motion::AnimationState::looping(next))){
            playing_state=next;playback_phase=0;
        }
    }
    if(next==State::Walk||next==State::Run){
        // Own the locomotion clock; the runtime must not advance it again.
        // Cadence follows collision-resolved speed, not just requested input.
        animator.pause();
        Fixed rate=speed/(next==State::Walk?walk_cycle_speed:run_cycle_speed);
        if(rate<Fixed(0.2))rate=0.2;if(rate>Fixed(1.6))rate=1.6;
        playback_phase+=uint32_t((rate*dt*60).raw());
        if(animator.model&&animator.clip>=0){
            const uint32_t frames=animator.model->clips[animator.clip].frames;
            const uint32_t period=(frames>1?frames-1:1)*2;
            playback_phase%=period*4096;
            animator.ticks=playback_phase/4096;
        }
    }
}

// Original bs07a: Attack_2 raises the gun, Attack_3 fires, Attack_4 lowers it.
// Fire once at the transition into recoil. The actor selects its cooked muzzle
// vertex; Current + World includes pose, aim and parent scale.
bool DungeonPlayerController::update_attack(bool jumped,const SpatialQueryBatch* queries){
    DungeonMeasure measure(10);
    auto* visual=visual_handle.get();if(!visual)return false;
    auto& a=visual->animator;
    if(attacking&&(jumped||!grounded)){
        attacking=false;playing_state=State(255);
    }
    if(attacking&&!a.playing){
        if(attack_phase==0){attack_phase=1;a.play(attack_clips[1],false);shot_sent=false;}
        else if(attack_phase==1){attack_phase=input.held(Button::Square)?0:2;a.play(attack_clips[attack_phase],false);}
        else{attacking=false;playing_state=State(255);}
    }
    if(!attacking&&grounded&&!jumped&&input.held(Button::Square)&&attack_clips[0]>=0&&attack_clips[1]>=0&&attack_clips[2]>=0){
        attack_phase=0;attacking=a.play(attack_clips[0],false);shot_sent=false;
    }
    if(!attacking)return false;
    if(!shot_sent&&attack_phase==1){
        shot_sent=true;
        const auto muzzle=skeletal_sample_vertex(visual,muzzle_vertex,PoseKind::Current,CoordinateSpace::World);
        if(!muzzle.success){++combat_telemetry.muzzle_errors;return true;}
        Fixed direction[3];
        if(auto* target=target_handle.get()){
            int64_t length2=0;
            for(int k=0;k<3;++k){direction[k]=target->transform.position[k]-muzzle.position[k];if(k==1)direction[k]+=focus_height(*target);length2+=int64_t(direction[k].raw())*direction[k].raw();}
            const Fixed length(int32_t(character_motion::root(uint32_t(length2>>4)))*4,Fixed::RAW);
            if(length.raw())for(auto& v:direction)v/=length;
        }else{
            const auto pitch_cos=cos_degrees(aim_pose_pitch);
            direction[0]=sin_degrees(facing)*pitch_cos;direction[1]=-sin_degrees(aim_pose_pitch);direction[2]=cos_degrees(facing)*pitch_cos;
            if(camera_mode==CameraMode::FreeAim)if(auto* camera=camera_handle.get()){
                // Converge the muzzle onto the center reticle's world ray.
                const Fixed c=cos_degrees(camera_pitch);
                const Fixed ray[3]={sin_degrees(camera_yaw)*c*Fixed(40.0),-sin_degrees(camera_pitch)*Fixed(40.0),cos_degrees(camera_yaw)*c*Fixed(40.0)};
                const auto hit=queries?queries->raycast(camera->transform.position,ray,3,get_owner()->data()):raycast(camera->transform.position,ray,3,get_owner()->data());
                int64_t n=0;for(int k=0;k<3;++k){direction[k]=(hit?hit.point[k]:camera->transform.position[k]+ray[k])-muzzle.position[k];n+=int64_t(direction[k].raw())*direction[k].raw();}
                const Fixed length(int32_t(character_motion::root(uint32_t(n>>4)))*4,Fixed::RAW);
                if(length.raw())for(auto& value:direction)value/=length;
            }
        }
        if(auto* arena=DungeonGame::current){
            // The gun may poke through a nearby wall. Check body-to-muzzle too.
            const auto* body=get_owner()->data();Fixed start[3]={body->transform.position[0],muzzle.position[1],body->transform.position[2]},delta[3];
            for(int k=0;k<3;++k)delta[k]=muzzle.position[k]-start[k];
            const auto wall=queries?queries->raycast(start,delta,1,body):raycast(start,delta,1,body);
            if(!wall)arena->fire(muzzle.position,direction);
        }
    }
    return true;
}

// Place the camera behind the character, apply shoulder offset, and prevent wall clipping.
void DungeonPlayerController::place_camera(const Transform& transform,const SpatialQueryBatch* queries){
    DungeonMeasure measure(2);
    auto* camera=camera_handle.get();if(!camera)return;

    // Blend boom length, height, and shoulder offset when entering an aim mode.
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

    // Shorten the camera boom when a raycast detects an obstacle behind it.
    const auto* game=DungeonGame::current;const uint32_t revision=game?game->camera_collision_revision():0;
    bool changed=!game||camera_revision!=revision;
    for(int i=0;i<3;++i)changed|=origin[i]!=query_origin[i]||offset[i]!=query_offset[i];
    if(changed){
        const auto hit=queries?queries->raycast(origin,offset,1,get_owner()->data()):raycast(origin,offset,1,get_owner()->data());
        camera_hit=bool(hit);camera_fraction=hit.fraction;camera_revision=revision;
        for(int i=0;i<3;++i){query_origin[i]=origin[i];query_offset[i]=offset[i];}
    }
    if(camera_hit){
        const Fixed clearance=camera_fraction<Fixed(.08)?camera_fraction/Fixed(2.0):Fixed(.04);
        const Fixed fraction=camera_fraction-clearance;
        for(auto& v:offset)v*=fraction;
    }
    for(int i=0;i<3;++i)camera->transform.position[i]=origin[i]+offset[i];
    camera->transform.rotation[0]=camera_pitch;
    camera->transform.rotation[1]=camera_yaw;
    camera->transform.rotation[2]=0.0;
}

// Process input, resolve movement and collisions, then update visual systems.
void DungeonPlayerController::update(Transform& transform,Fixed dt){
    DungeonMeasure measure(1);
    if(dt.raw()<=0)return;
    auto* body=get_owner()->data();
    if(auto* game=DungeonGame::current;game&&(!game->ready()||game->handle_interfaces(dt)||!game->controls_enabled())){
        attacking=false;for(auto& v:velocity)v=0.0;
        if(auto* visual=visual_handle.get()){
            if(visual->animator.clip!=clips[0])visual->animator.play(clips[0],true);
            visual->animator.pause();
        }
        playing_state=State(255);target_handle={};
        if(auto* marker=lock_marker_handle.get())set_active(marker,false);
        place_camera(transform);return;
    }
    if(auto* game=DungeonGame::current;game&&game->ready()&&!game->player_alive()){
        attacking=false;for(auto& v:velocity)v=0.0;
        if(auto* visual=visual_handle.get())visual->animator.pause();
        if(auto* marker=lock_marker_handle.get())set_active(marker,false);
        place_camera(transform);return;
    }
    const bool free_aim=input.held(Button::L1);
    if(auto* game=DungeonGame::current)game->aiming=free_aim;

    // L1 enters free aim and clears lock-on; R1 acquires a target only when aim is released.
    // L1 owns manual aim and cancels lock-on. R1 is ignored until L1 is released,
    // so both inputs cannot reacquire and cancel a target in the same frame.
    if(input.pressed(Button::L1)){
        target_handle={};
        lock_marker_intro_frames=0;
    }else if(!free_aim&&input.pressed(Button::R1))toggle_target(*body);
    // Combine analog and digital movement input, with digital input taking priority.
    int32_t x=input.axis_raw(Axis::LeftX),z=input.axis_raw(Axis::LeftY);
    const int dx=int(input.held(Button::Right))-int(input.held(Button::Left));
    const int dz=int(input.held(Button::Up))-int(input.held(Button::Down));
    if(dx||dz){x=dx*4096;z=dz*4096;}
    auto intent=character_motion::intent(x,z);
    // Attack owns the full-body clip. Locomotion resumes as soon as it ends.
    if(attacking||(grounded&&input.held(Button::Square)))intent={};
    update_camera(transform,dt,free_aim,intent.strength!=0);
    const Fixed strafe(intent.x,Fixed::RAW),forward(intent.z,Fixed::RAW);
    const Fixed input_heading(character_motion::heading(intent.x,intent.z),Fixed::RAW);
    const Fixed movement_yaw=camera_yaw;
    const Fixed sy=sin_degrees(movement_yaw),cy=cos_degrees(movement_yaw);
    const Fixed desired_x=(sy*forward+cy*strafe)*max_speed;
    const Fixed desired_z=(cy*forward-sy*strafe)*max_speed;

    // Rotate the character according to the active camera mode and target state.
    if(camera_mode==CameraMode::Normal&&intent.strength){
        const Fixed target=movement_yaw+input_heading;
        facing=approach_angle(facing,target,turn_rate*dt);
    }else if(camera_mode==CameraMode::LockOn){
        if(auto* target=target_handle.get()){
            const Fixed target_facing=target_yaw(transform,*target);
            // While moving, Denise turns with the requested camera-relative
            // direction. Releasing the stick (or firing) restores target-facing.
            if(intent.strength&&!(attacking||(grounded&&input.held(Button::Square)))){
                const Fixed movement_facing=camera_yaw+Fixed(character_motion::heading(intent.x,intent.z),Fixed::RAW);
                facing=approach_angle(facing,movement_facing,turn_rate*dt);
            }else if(input.pressed(Button::Square))facing=target_facing;
            else facing=approach_angle(facing,target_facing,turn_rate*dt);
        }
    }else if(camera_mode==CameraMode::FreeAim){
        // Free aim owns yaw: left-stick strafing does not independently turn the
        // character, while right-stick yaw rotates camera and body together.
        facing=approach_angle(facing,camera_yaw,turn_rate*dt);
    }

    // Smooth horizontal velocity differently on the ground and in the air.
    Fixed response=(grounded?ground_response:air_response)*dt;
    if(response>Fixed(1.0))response=1.0;
    velocity[0]+=(desired_x-velocity[0])*response;
    velocity[2]+=(desired_z-velocity[2])*response;

    // Apply jump and gravity, keeping a small downward force on grounded actors.
    const bool jumped=grounded&&input.pressed(Button::Cross);
    if(grounded)velocity[1]=jumped?jump_speed:Fixed(0.0);
    else velocity[1]-=gravity*dt;
    const Fixed fall=grounded&&!jumped?stick_to_ground:velocity[1]*dt;
    const Fixed movement[3]={velocity[0]*dt,fall,velocity[2]*dt};
    const Fixed before[3]={transform.position[0],transform.position[1],transform.position[2]};

    // Resolve the requested movement against the world and update grounded state.
    const auto* game=DungeonGame::current;const uint32_t revision=game?game->collision_revision():0;
    bool settled=game&&grounded&&!jumped&&movement[0].raw()==0&&movement[2].raw()==0&&movement_revision==revision;
    for(int k=0;k<3;++k)settled&=transform.position[k]==settled_position[k]&&transform.rotation[k]==settled_rotation[k];
    SpatialQueryBatch local_queries;
    auto& queries=game?DungeonGame::current->spatial_queries():local_queries;
    if(!settled){
    const auto result=queries.move_and_slide(*body,movement);
    grounded=result.grounded;
    if(velocity[1]>Fixed(0.0)&&result.displacement[1]+Fixed(0.001)<fall)velocity[1]=0.0;
    // Ground contact can leave a one-unit Q12 horizontal remainder. Only a
    // horizontal contact is a step obstacle; the floor alone must not trigger
    // the three extra lift/move/settle collision queries on every walking tick.
    if(step_height.raw()>0&&result.blocked&&grounded&&!jumped&&(result.normal[0].raw()||result.normal[2].raw())){
        const Fixed remaining[3]={movement[0]-result.displacement[0],0.0,movement[2]-result.displacement[2]};
        if(remaining[0].raw()||remaining[2].raw()){
            const Fixed lift[3]={0.0,step_height,0.0};
            const auto lifted=move_and_slide(*body,lift);
            move_and_slide(*body,remaining);
            const Fixed settle[3]={0.0,-lifted.displacement[1]+stick_to_ground,0.0};
            grounded=move_and_slide(*body,settle).grounded;
        }
    }
    }
    movement_revision=revision;for(int k=0;k<3;++k){settled_position[k]=transform.position[k];settled_rotation[k]=transform.rotation[k];}

    transform.rotation[1]=facing;
    place_camera(transform,&queries);
    update_aim_pose(dt,free_aim);
    // Attack clips do not consume locomotion speed. Keep the exact displacement
    // calculation for animation, but skip its divisions/root while firing.
    if(!update_attack(jumped,&queries)){
        const int32_t sx=((transform.position[0]-before[0])/dt).raw();
        const int32_t sz=((transform.position[2]-before[2])/dt).raw();
        const Fixed speed(int32_t(character_motion::root(uint32_t(int64_t(sx)*sx+int64_t(sz)*sz))),Fixed::RAW);
        animate(dt,speed,jumped);
    }

    // Camera and marker follow the final, collision-resolved transform.
    update_lock_marker();
}
