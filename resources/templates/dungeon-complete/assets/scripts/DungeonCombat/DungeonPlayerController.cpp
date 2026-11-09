#include "DungeonPlayerController.hpp"
#include "CharacterClips.hpp"
#include "DungeonTargeting.hpp"
#include "DungeonGame.hpp"
#include "DungeonProfiling.hpp"
#include "world2d.hpp"
#include "scene.hh"
EPOK_SPEED_BEGIN
using namespace epok;
using character_motion::State;
namespace {
bool target_candidate(const ActorData* value,DungeonPlayerCombatBase& rules,const DungeonGame* arena){
    return value&&rules.candidate(is_active(value),value->mesh,value->collider.enabled,arena&&arena->is_target(value));
}
DungeonTargetDistance target_distance(const Transform& from,const ActorData& target){
    const auto value=uint64_t(third_person_targeting::horizontal_distance_squared(from,target));
    return {uint32_t(value>>32),uint32_t(value)};
}
bool same_name(const char* a,const char* b){
    while(*a&&*a==*b){++a;++b;}return *a==*b;
}
Fixed focus_height(const ActorData& value,DungeonPlayerCombatBase& rules){
    return rules.control_focus_height(value.collider.enabled,value.collider.center[1],value.transform.scale[1]);
}
DungeonControlAngles target_angles(const Transform& from,const ActorData& target,DungeonPlayerCombatBase& rules){
    return rules.control_target_angles(gameplay_vector(from.position),gameplay_vector(target.transform.position),focus_height(target,rules));
}
}

DungeonGame* DungeonPlayerController::game()const{auto* value=game_binding_.resolve(game_component);return value&&value->running()?value:nullptr;}

// Initialize the controller and resolve the scene objects it depends on.
void DungeonPlayerController::start(Transform& transform){
    // Reset movement, camera, animation, and target state for a fresh run.
    auto* rules=combat_rules();if(!rules)return;facing=transform.rotation[1];assign_camera(rules->control_camera_start(facing));
    motion_={};
    if(auto* player=character()){
        player->motor.automatic_tick=false;
        player->motor.reset_motion();motion_=player->motor.motion();
    }
    camera_mode=CameraMode::Normal;aim_pose_pitch=0.0;aim_camera_blend=0.0;visibility_phase=0;
    animation={};playing_state=State::Land;playback_phase=0;
    for(auto& clip:attack_clips)clip=-1;attack_phase=0;attacking=shot_sent=false;
    visual_handle={};target_handle={};lock_marker_handle={};
    lock_marker_intro_frames=0;lock_marker_spin=0;for(auto& clip:clips)clip=-1;

    // Find the character visual and map its animation clips by name.
    ActorData* visual=nullptr;
    if(auto* player=character())if(auto* registry=epok::active_object_registry)
        if(auto* mesh=registry->resolve<Mesh3DComponent>(player->visual))
            if(auto* owner=mesh->get_owner())visual=owner->data();
    if(visual){
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
            const auto rig=rules->control_rig(uint32_t(model->bone_count));
            visual->aim_bone=int16_t(rig.bone);visual->aim_stop_bones[0]=int16_t(rig.stop_a);visual->aim_stop_bones[1]=int16_t(rig.stop_b);
        }
        visual->aim_pitch=0.0;
    }

    // Register the follow camera and make it the active camera.
    if(auto* view=camera())set_active_camera(view->data());

    // The lock-on marker starts hidden and is enabled only when a target exists.
    if(auto* marker=gameplay_actor_data(lock_marker)){
        lock_marker_handle=handle(marker);
        set_active(marker,false);
    }
    place_camera(transform);
}

// Cycle to the next valid target, or acquire the nearest target when none is selected.
void DungeonPlayerController::toggle_target(ActorData& body){
    auto* rules=combat_rules();if(!rules)return;
    auto* arena=game();
    const auto* queries=arena?&arena->spatial_queries():nullptr;
    auto* current=target_handle.get();
    const auto range=rules->tuning.acquire_distance;
    const bool cycling=target_candidate(current,*rules,arena)
        &&third_person_targeting::nearby(body.transform,*current,rules->tuning.keep_distance)
        &&third_person_targeting::nearby(body.transform,*current,range);
    const auto selected_distance=cycling?target_distance(body.transform,*current):DungeonTargetDistance{};
    ActorData* best=nullptr;DungeonTargetDistance best_distance{};uint32_t best_index=0;
    for(size_t index=0;index<object_count;++index){
        auto& value=objects[index];
        if((cycling&& &value==current)||!target_candidate(&value,*rules,arena)
            ||!third_person_targeting::nearby(body.transform,value,range)
            ||!third_person_targeting::visible(body.transform,value,queries))continue;
        const auto distance=target_distance(body.transform,value);
        if(cycling&&!rules->after_selected(distance,uint32_t(index),selected_distance,target_handle.index))continue;
        if(rules->prefer_candidate(distance,uint32_t(index),best!=nullptr,best_distance,best_index,cycling)){
            best=&value;best_distance=distance;best_index=uint32_t(index);
        }
    }
    target_handle=best?handle(best):DataHandle{};
    if(auto* selected=target_handle.get();selected&&selected!=current){
        // Tron Bonne's effect starts at r=608, immediately advances to r=544,
        // and reaches r=96 over eight rendered frames.
        lock_marker_intro_frames=uint8_t(rules->control_marker_begin());
        lock_marker_spin=0;
        if(auto* arena=game())arena->sound(DungeonSound::LockOn);
    }else if(!target_handle.get())lock_marker_intro_frames=0;
}

// Update target validity, camera mode, and camera orientation for this frame.
DungeonControlAngles DungeonPlayerController::update_camera(Transform& transform,Fixed dt,bool free_aim,bool moving){
    auto* target=target_handle.get();
    auto* rules=combat_rules();if(!rules)return {};

    // Drop targets that disappeared, moved too far away, or are no longer eligible.
    const auto checked=rules?rules->target_check(visibility_phase,target_candidate(target,*rules,game()),
        target&&third_person_targeting::nearby(transform,*target,rules->tuning.keep_distance)):DungeonTargetCheck{};
    visibility_phase=checked.phase;
    if(!checked.keep){
        target_handle={};target=nullptr;
    }
    if(target&&checked.query_sight){
        const auto* queries=game()?&game()->spatial_queries():nullptr;
        if(!third_person_targeting::visible(transform,*target,queries)){target_handle={};target=nullptr;}
    }

    const auto look=intent_for(look_memo_,*rules,Fixed(GameplayInputLibrary::axis_raw(Axis::RightX),Fixed::RAW),
        Fixed(GameplayInputLibrary::axis_raw(Axis::RightY),Fixed::RAW),false,false,false,false,false);
    const auto angles=target?target_angles(transform,*target,*rules):DungeonControlAngles{};
    const DungeonControlCameraInput observed{look,dt,facing,angles.yaw,angles.pitch,free_aim,target!=nullptr,moving};
    assign_camera(rules->control_camera_step(camera_state(),observed));
    return angles;
}

// Smooth the visual rig toward the pitch requested by the current aim mode.
void DungeonPlayerController::update_aim_pose(Fixed dt,bool free_aim){
    auto* rules=combat_rules();if(!rules)return;auto* target=target_handle.get();
    const auto angles=target?target_angles(get_owner()->data()->transform,*target,*rules):DungeonControlAngles{};
    aim_pose_pitch=rules->control_aim_step(aim_pose_pitch,camera_pitch,angles.pitch,dt,free_aim,target!=nullptr);
    if(auto* visual=visual_handle.get())visual->aim_pitch=aim_pose_pitch;
}

// Keep the lock-on marker attached to the target and animate its sprite effect.
void DungeonPlayerController::update_lock_marker(){
    auto* marker=lock_marker_handle.get();auto* target=target_handle.get();auto* rules=combat_rules();if(!marker||!rules)return;
    if(!target){if(is_active(marker))set_active(marker,false);return;}
    if(!is_active(marker))set_active(marker,true);
    marker->transform.position[0]=target->transform.position[0];marker->transform.position[1]=target->transform.position[1]+focus_height(*target,*rules);
    marker->transform.position[2]=target->transform.position[2];
    const auto visual=rules->control_marker_step(Fixed(int32_t(lock_marker_intro_frames)*4096,Fixed::RAW),Fixed(int32_t(lock_marker_spin)*4096,Fixed::RAW));
    lock_marker_intro_frames=uint8_t(visual.intro.raw()/4096);lock_marker_spin=uint8_t(visual.spin.raw()/4096);
    marker->sprite.region[0]=uint16_t(lock_marker_spin&7)*32;marker->sprite.region[1]=uint16_t(lock_marker_spin>>3)*32;
    marker->sprite.region[2]=marker->sprite.region[3]=32;
    marker->sprite.size[0]=marker->sprite.size[1]=visual.size;
    marker->sprite.color[0]=marker->sprite.color[1]=marker->sprite.color[2]=uint8_t(visual.tint);
    marker->sprite.blend=visual.average?BlendMode::Average:BlendMode::Cutout;
}

// Select the next animation state and advance locomotion playback manually.
void DungeonPlayerController::animate(Fixed dt,Fixed speed,bool jumped){
    auto* visual=visual_handle.get();auto* rules=combat_rules();if(!visual||!rules)return;auto& animator=visual->animator;
    const bool finished=!animator.model||animator.clip<0||size_t(animator.clip)>=animator.model->clip_count||
        animator.ticks>=uint32_t(animator.model->clips[animator.clip].frames-1)*2;
    const auto decision=rules->control_animation_select(uint32_t(animation.state),uint32_t(playing_state),motion_.grounded,motion_.velocity.y,speed,finished,jumped);
    const State next=State(decision.state);animation.state=next;
    if(decision.switch_clip)if(animator.play(clips[decision.state],decision.loop)){playing_state=next;playback_phase=0;}
    if(decision.locomotion){
        animator.pause();const bool valid=animator.model&&animator.clip>=0;
        const auto clock=rules->control_animation_clock(playback_phase,valid?animator.model->clips[animator.clip].frames:0,decision.rate,dt,valid);
        playback_phase=clock.phase;if(valid)animator.ticks=clock.ticks;
    }
}

// Original bs07a: Attack_2 raises the gun, Attack_3 fires, Attack_4 lowers it.
// Fire once at the transition into recoil. The actor selects its cooked muzzle
// vertex; Current + World includes pose, aim and parent scale.
bool DungeonPlayerController::update_attack(bool jumped,const SpatialQueryBatch* queries){
    DungeonMeasure measure(10);
    auto* visual=visual_handle.get();if(!visual)return false;
    auto* rules=combat_rules();if(!rules){attacking=false;return false;}
    auto& a=visual->animator;
    const auto step=rules->attack({attack_phase,attacking,shot_sent},motion_.grounded,jumped,
        GameplayInputLibrary::button_held(Button::Square),a.playing,attack_clips[0]>=0&&attack_clips[1]>=0&&attack_clips[2]>=0);
    attack_phase=uint8_t(step.state.phase);attacking=step.state.attacking;shot_sent=step.state.shot_sent;
    if(step.invalidate_locomotion)playing_state=State(255);
    if(step.clip>=0&&step.clip<3){const bool started=a.play(attack_clips[step.clip],false);if(step.starting)attacking=started;}
    if(!attacking)return false;
    if(step.fire){
        const auto muzzle=skeletal_sample_vertex(visual,muzzle_vertex,PoseKind::Current,CoordinateSpace::World);
        if(!muzzle.success){++combat_telemetry.muzzle_errors;return true;}
        Fixed direction[3];auto* selected=target_handle.get();
        const auto aim=rules->control_shot_aim(selected!=nullptr,uint32_t(camera_mode));
        if(aim==1){
            const auto delta=MathLibrary::subtract(gameplay_vector(selected->transform.position),gameplay_vector(muzzle.position));
            GameplayVector3 focused=delta;focused.y+=focus_height(*selected,*rules);const auto aimed=rules->control_direction_toward(focused);
            if(!aimed.valid){++combat_telemetry.muzzle_errors;return true;}gameplay_vector(aimed.value,direction);
        }else{
            gameplay_vector(rules->control_direction(facing,aim_pose_pitch),direction);
            // A catch-up step only carried the camera along; solve it before aiming from it.
            if(aim==2&&!epok::time.drawn_step)place_camera(get_owner()->data()->transform);
            if(aim==2)if(auto* camera=camera_data()){
                Fixed ray[3];gameplay_vector(rules->control_aim_ray(camera_yaw,camera_pitch),ray);
                const auto hit=queries?queries->raycast(camera->transform.position,ray,rules->control_aim_mask(false),get_owner()->data()):
                    raycast(camera->transform.position,ray,rules->control_aim_mask(false),get_owner()->data());
                Fixed delta[3];for(int k=0;k<3;++k)delta[k]=(hit?hit.point[k]:camera->transform.position[k]+ray[k])-muzzle.position[k];
                const auto aimed=rules->control_direction_toward(gameplay_vector(delta));
                if(!aimed.valid){++combat_telemetry.muzzle_errors;return true;}gameplay_vector(aimed.value,direction);
            }
        }
        if(auto* arena=game()){
            // The gun may poke through a nearby wall. Check body-to-muzzle too.
            const auto* body=get_owner()->data();Fixed start[3]={body->transform.position[0],muzzle.position[1],body->transform.position[2]},delta[3];
            for(int k=0;k<3;++k)delta[k]=muzzle.position[k]-start[k];
            const auto wall=queries?queries->raycast(start,delta,rules->control_aim_mask(true),body):raycast(start,delta,rules->control_aim_mask(true),body);
            if(!wall)arena->fire(muzzle.position,direction);
        }
    }
    return true;
}

// Place the camera behind the character, apply shoulder offset, and prevent wall clipping.
void DungeonPlayerController::place_camera(const Transform& transform){
    DungeonMeasure measure(2);auto* view=camera();auto* rules=combat_rules();if(!view||!rules)return;
    const auto proposal=rules->control_view_proposal(gameplay_vector(transform.position),camera_state(),shoulder_camera_distance);
    const auto* arena=game();
    view->place(proposal.origin,proposal.offset,proposal.yaw,proposal.pitch,proposal.distance,arena?arena->camera_collision_revision():0);
}

// Process input, resolve movement and collisions, then update visual systems.
void DungeonPlayerController::update(Transform& transform,Fixed dt){
    DungeonMeasure measure(1);
    if(dt.raw()<=0)return;
    auto* player=character();if(!player)return;
    auto* combat=combat_rules();if(!combat)return;auto& motor=player->motor;motion_=motor.motion();
    auto* body=get_owner()->data();
    auto* gate_game=game();const bool ready=!gate_game||gate_game->ready();
    const bool interfaces=gate_game&&ready?gate_game->handle_interfaces(dt):false;
    const auto gate=combat->control_gate(ready,interfaces,!gate_game||gate_game->controls_enabled(),!gate_game||gate_game->player_alive());
    if(gate==1){
        attacking=false;motor.clear_velocity();motion_=motor.motion();
        if(auto* visual=visual_handle.get()){
            if(visual->animator.clip!=clips[0])visual->animator.play(clips[0],true);
            visual->animator.pause();
        }
        playing_state=State(255);target_handle={};
        if(auto* marker=lock_marker_handle.get())set_active(marker,false);
        place_camera(transform);return;
    }
    if(gate==2){
        attacking=false;motor.clear_velocity();motion_=motor.motion();
        if(auto* visual=visual_handle.get())visual->animator.pause();
        if(auto* marker=lock_marker_handle.get())set_active(marker,false);
        place_camera(transform);return;
    }
    const bool free_aim=GameplayInputLibrary::button_held(Button::L1);
    if(auto* arena=game())arena->aiming=free_aim;

    // L1 enters free aim and clears lock-on; R1 acquires a target only when aim is released.
    // L1 owns manual aim and cancels lock-on. R1 is ignored until L1 is released,
    // so both inputs cannot reacquire and cancel a target in the same frame.
    const auto target_action=combat?combat->target_input(free_aim,GameplayInputLibrary::button_pressed(Button::L1),GameplayInputLibrary::button_pressed(Button::R1)):0;
    if(target_action==1){
        target_handle={};
        lock_marker_intro_frames=0;
    }else if(target_action==2)toggle_target(*body);
    const bool attack_owns_movement=combat->blocks_movement({attack_phase,attacking,shot_sent},motion_.grounded,GameplayInputLibrary::button_held(Button::Square));
    const auto intent=intent_for(move_memo_,*combat,Fixed(GameplayInputLibrary::axis_raw(Axis::LeftX),Fixed::RAW),Fixed(GameplayInputLibrary::axis_raw(Axis::LeftY),Fixed::RAW),
        GameplayInputLibrary::button_held(Button::Right),GameplayInputLibrary::button_held(Button::Left),GameplayInputLibrary::button_held(Button::Up),GameplayInputLibrary::button_held(Button::Down),attack_owns_movement);
    const auto angles=update_camera(transform,dt,free_aim,intent.strength.raw()!=0);
    auto* selected=target_handle.get();
    const auto movement=combat->control_movement_step(intent,camera_state(),facing,angles.yaw,selected!=nullptr,attack_owns_movement,GameplayInputLibrary::button_pressed(Button::Square),dt);
    facing=movement.facing;

    // Game rules produce desired world velocity and facing. The common motor
    // owns integration, gravity, jumping, contact response and stair policy.
    const Fixed before[3]={transform.position[0],transform.position[1],transform.position[2]};
    const auto* arena=game();
    SpatialQueryBatch local_queries;
    auto& queries=arena?game()->spatial_queries():local_queries;
    CharacterCommand3D command;command.velocity={movement.x,movement.z};command.jump=GameplayInputLibrary::button_pressed(Button::Cross);
    motor.set_facing(facing);motor.submit(command);
    motion_=arena?motor.simulate_shared(dt,arena->collision_queries,arena->collision_revision()):motor.simulate(dt);
    const bool jumped=motion_.jumped;

    // A catch-up step is never drawn. Its camera only keeps pace with the body,
    // so interpolation of the drawn step starts from the right place. Gameplay
    // never reads this estimate: free-aim fire solves the camera first.
    if(epok::time.drawn_step)place_camera(transform);
    else if(auto* view=camera_data())for(int k=0;k<3;++k)view->transform.position[k]+=transform.position[k]-before[k];
    update_aim_pose(dt,free_aim);
    // Attack clips do not consume locomotion speed. Keep the exact displacement
    // calculation for animation, but skip its divisions/root while firing.
    if(update_attack(jumped,&queries)){playback_elapsed_=0.0;playback_jumped_=false;}
    else{
        // Playback is drawn once, and its clock advances linearly with the time it receives.
        playback_elapsed_+=dt;playback_jumped_|=jumped;
        if(epok::time.drawn_step){
            const GameplayVector3 displacement{transform.position[0]-before[0],transform.position[1]-before[1],transform.position[2]-before[2]};
            const Fixed speed=combat->control_motion_speed(displacement,dt);
            animate(playback_elapsed_,speed,playback_jumped_);
            playback_elapsed_=0.0;playback_jumped_=false;
        }
    }

    // Camera and marker follow the final, collision-resolved transform.
    if(epok::time.drawn_step)update_lock_marker();
}
EPOK_SPEED_END
