#pragma once
#include "DungeonPlayerCombatBase.hpp"
EPOK_SPEED_BEGIN

class EPOK_CLASS(Blueprintable,Id="e8e7c9ff-ff0e-5c38-9eec-5434e6a45354") DungeonPlayerCombat:public DungeonPlayerCombatBase {
public:
    DungeonAttackStep attack(DungeonAttackState state,bool grounded,bool jumped,bool held,bool clip_playing,bool clips_ready)override{
        DungeonAttackStep result;result.state=state;
        if(state.attacking&&(jumped||!grounded)){
            result.state.attacking=false;result.invalidate_locomotion=true;return result;
        }
        if(state.attacking&&!clip_playing){
            if(state.phase==0){result.state.phase=1;result.clip=1;result.state.shot_sent=false;}
            else if(state.phase==1){result.state.phase=held?0:2;result.clip=int32_t(result.state.phase);}
            else{result.state.attacking=false;result.invalidate_locomotion=true;}
        }
        if(!result.state.attacking&&grounded&&!jumped&&held&&clips_ready){
            result.state={0,true,false};result.clip=0;result.starting=true;
        }
        if(result.state.attacking&&!result.state.shot_sent&&result.state.phase==1){
            result.state.shot_sent=true;result.fire=true;
        }
        return result;
    }
    bool blocks_movement(DungeonAttackState state,bool grounded,bool held)const override{return state.attacking||(grounded&&held);}
    uint32_t target_input(bool free_held,bool free_pressed,bool lock_pressed)const override{
        if(free_pressed)return 1;if(!free_held&&lock_pressed)return 2;return 0;
    }
    DungeonTargetCheck target_check(uint32_t phase,bool eligible,bool within_keep)const override{
        if(!eligible||!within_keep)return {phase,false,false};
        ++phase;if(phase>=tuning.visibility_period)return {0,true,true};return {phase,true,false};
    }
    bool candidate(bool active,bool has_mesh,bool collider_enabled,bool marked)const override{return active&&has_mesh&&collider_enabled&&marked;}
    bool after_selected(DungeonTargetDistance distance,uint32_t index,DungeonTargetDistance selected_distance,uint32_t selected_index)const override{
        return distance.high>selected_distance.high||(distance.high==selected_distance.high
            &&(distance.low>selected_distance.low||(distance.low==selected_distance.low&&index>selected_index)));
    }
    bool prefer_candidate(DungeonTargetDistance distance,uint32_t index,bool best_exists,DungeonTargetDistance best_distance,uint32_t best_index,bool cycling)const override{
        return !best_exists||distance.high<best_distance.high||(distance.high==best_distance.high
            &&(distance.low<best_distance.low||(cycling&&distance.low==best_distance.low&&index<best_index)));
    }
    epok::ProjectileSpawn3D projectile(epok::GameplayVector3 origin,epok::GameplayVector3 direction)const override{
        return {origin,{direction.x*tuning.projectile_speed,direction.y*tuning.projectile_speed,direction.z*tuning.projectile_speed},tuning.projectile_lifetime,0};
    }

    DungeonImpactTrace trace_settings(bool enemy)const override{
        return {enemy?impact_tuning.enemy_trace_mask:impact_tuning.player_trace_mask,!enemy};
    }
    // 0 world, 1 enemy, 2 player, 3 ignored. Native code supplies observations;
    // ownership and damage/sound decisions are chosen-language project policy.
    uint32_t impact(bool enemy,bool target_exists,bool target_is_player,uint32_t target_layers)const override{
        if(enemy)return target_is_player?2u:3u;
        return target_exists&&((target_layers/2u)%2u)!=0?1u:0u;
    }
    DungeonPlayerShotVisual shot_visual()const override{
        return {bounded_size(shot_style.size),bounded(shot_style.red,0,255),bounded(shot_style.green,0,255),bounded(shot_style.blue,0,255)};
    }
    DungeonCombatEffectStep effect_step(uint32_t kind,uint32_t age,bool advance)const override{
        age=bounded(age,0,65535);if(advance&&age<65535)++age;
        const auto end=bounded(kind?effect_tuning.hit_end_age:effect_tuning.muzzle_end_age,0,65535);
        const auto size=bounded_size(kind?effect_tuning.hit_size:effect_tuning.muzzle_size);
        if(age>=end)return {age,0,size,false};
        uint32_t frame;
        if(kind){const auto first=bounded(effect_tuning.hit_first_frame,0,2047),stride=bounded(effect_tuning.hit_ticks_per_frame,1,65535);
            frame=bounded(first+age/stride,0,2047);}
        else frame=bounded(age<bounded(effect_tuning.muzzle_switch_age,0,65535)?effect_tuning.muzzle_first_frame:effect_tuning.muzzle_second_frame,0,2047);
        return {age,frame,size,true};
    }
    DungeonDamageFeedback damage_feedback(uint32_t remaining_us,uint32_t elapsed_us,bool restart)const override{
        const auto duration=bounded(feedback_tuning.duration_us,0,10000000);
        if(!duration)return {};
        auto remaining=restart?duration:bounded(remaining_us,0,duration);
        if(elapsed_us>=remaining)return {};
        remaining-=elapsed_us;
        const auto width=bounded(feedback_tuning.border_pixels,1,64),red=bounded(feedback_tuning.red_base,0,255);
        const auto span=bounded(feedback_tuning.red_span,0,255-red);
        // Operands are bounded before multiply/add: max 650,000,000 for the
        // ceil numerator, and max 2,550,000,000 for a color numerator.
        return {remaining,true,(width*remaining+duration-1)/duration,red+span*remaining/duration,
            bounded(feedback_tuning.green,0,255),bounded(feedback_tuning.blue,0,255)};
    }
    DungeonDamageRumble damage_rumble()const override{
        return {bounded(feedback_tuning.rumble_large,0,255),feedback_tuning.rumble_small,bounded(feedback_tuning.rumble_duration_ms,0,60000)};
    }
    uint32_t control_gate(bool ready,bool interface_block,bool controls,bool alive)const override{
        if(((!(ready) || interface_block) || !(controls))){
            return 1u;
        }
        if(!(alive)){
            return 2u;
        }
        return 0u;
    }
    epok::Fixed control_button_axis(bool positive,bool negative)const{
        if((positive && !(negative))){
            return epok::Fixed(1.0);
        }
        if((negative && !(positive))){
            return epok::Fixed(-1.0);
        }
        return epok::Fixed(0.0);
    }
    epok::MovementIntent control_intent(epok::Fixed x,epok::Fixed y,bool right,bool left,bool up,bool down,bool suppress)const override{
        if(suppress){
            return epok::MathLibrary::stick_intent(epok::Fixed(0.0), epok::Fixed(0.0));
        }
        const auto button_axis_1=control_button_axis(right, left);
        if(((button_axis_1 != epok::Fixed(0.0)) || (control_button_axis(up, down) != epok::Fixed(0.0)))){
            return epok::MathLibrary::stick_intent(button_axis_1, control_button_axis(up, down));
        }
        return epok::MathLibrary::stick_intent(x, y);
    }
    epok::Fixed control_pitch(epok::Fixed value)const{
        const auto clamp_1=epok::MathLibrary::clamp(control_camera.pitch_min, epok::Fixed(-89.0), epok::Fixed(89.0));
        return epok::MathLibrary::clamp(value, clamp_1, epok::MathLibrary::clamp(control_camera.pitch_max, clamp_1, epok::Fixed(89.0)));
    }
    DungeonControlCamera control_camera_start(epok::Fixed facing)const override{
        const auto wrap_degrees_2=epok::MathLibrary::wrap_degrees(facing);
        const auto pitch_1=control_pitch(control_camera.initial_pitch);
        return make_controls_DungeonControlCamera(wrap_degrees_2, pitch_1, wrap_degrees_2, pitch_1, epok::Fixed(0.0), epok::Fixed(0.0), 0u);
    }
    uint32_t control_camera_mode(bool free_aim,bool target)const{
        if(free_aim){
            return 2u;
        }
        if(target){
            return 1u;
        }
        return 0u;
    }
    epok::Fixed control_blend_target(bool active)const{
        if(active){
            return epok::Fixed(1.0);
        }
        return epok::Fixed(0.0);
    }
    DungeonControlCamera control_camera_prepare(DungeonControlCamera state,DungeonControlCameraInput input)const{
        return make_controls_DungeonControlCamera(epok::MathLibrary::wrap_degrees((state).yaw), control_pitch((state).pitch), epok::MathLibrary::wrap_degrees((state).goal_yaw), control_pitch((state).goal_pitch), epok::MathLibrary::clamp((state).hold, epok::Fixed(0.0), epok::Fixed(10.0)), epok::MathLibrary::move_toward(epok::MathLibrary::clamp((state).blend, epok::Fixed(0.0), epok::Fixed(1.0)), control_blend_target(((input).free_aim || (input).target)), (epok::MathLibrary::clamp(control_camera.blend_rate, epok::Fixed(0.0), epok::Fixed(32.0)) * epok::MathLibrary::clamp((input).dt, epok::Fixed(0.0), epok::Fixed(1.0)))), control_camera_mode((input).free_aim, (input).target));
    }
    DungeonControlCamera control_camera_manual(DungeonControlCamera state,DungeonControlCameraInput input)const{
        const auto clamp_1=epok::MathLibrary::clamp((input).dt, epok::Fixed(0.0), epok::Fixed(1.0));
        if(((((input).look).strength > epok::Fixed(0.0)) && ((state).mode != 1u))){
            if(((state).hold <= epok::Fixed(0.0))){
                return make_controls_DungeonControlCamera((state).yaw, (state).pitch, epok::MathLibrary::wrap_degrees(((state).yaw + ((((input).look).x * epok::MathLibrary::clamp(control_camera.orbit_rate, epok::Fixed(0.0), epok::Fixed(720.0))) * clamp_1))), control_pitch(((state).pitch - ((((input).look).y * epok::MathLibrary::clamp(control_camera.pitch_rate, epok::Fixed(0.0), epok::Fixed(720.0))) * clamp_1))), epok::MathLibrary::clamp(control_camera.manual_time, epok::Fixed(0.0), epok::Fixed(10.0)), (state).blend, (state).mode);
            }
            return make_controls_DungeonControlCamera((state).yaw, (state).pitch, epok::MathLibrary::wrap_degrees(((state).goal_yaw + ((((input).look).x * epok::MathLibrary::clamp(control_camera.orbit_rate, epok::Fixed(0.0), epok::Fixed(720.0))) * clamp_1))), control_pitch(((state).goal_pitch - ((((input).look).y * epok::MathLibrary::clamp(control_camera.pitch_rate, epok::Fixed(0.0), epok::Fixed(720.0))) * clamp_1))), epok::MathLibrary::clamp(control_camera.manual_time, epok::Fixed(0.0), epok::Fixed(10.0)), (state).blend, (state).mode);
        }
        return make_controls_DungeonControlCamera((state).yaw, (state).pitch, (state).goal_yaw, (state).goal_pitch, epok::MathLibrary::move_toward(epok::MathLibrary::clamp((state).hold, epok::Fixed(0.0), epok::Fixed(10.0)), epok::Fixed(0.0), clamp_1), (state).blend, (state).mode);
    }
    DungeonControlCamera control_camera_goals(DungeonControlCamera state,DungeonControlCameraInput input)const{
        if(((state).mode == 1u)){
            return make_controls_DungeonControlCamera((state).yaw, (state).pitch, (input).target_yaw, control_pitch((input).target_pitch), (state).hold, (state).blend, (state).mode);
        }
        if((((state).mode == 0u) && ((state).hold <= epok::Fixed(0.0)))){
            if((input).moving){
                return make_controls_DungeonControlCamera((state).yaw, (state).pitch, (state).yaw, (state).goal_pitch, (state).hold, (state).blend, (state).mode);
            }
            return make_controls_DungeonControlCamera((state).yaw, (state).pitch, (input).facing, (state).goal_pitch, (state).hold, (state).blend, (state).mode);
        }
        return state;
    }
    epok::Fixed control_response(bool manual)const{
        if(manual){
            return epok::MathLibrary::clamp(control_camera.manual_response, epok::Fixed(0.0), epok::Fixed(32.0));
        }
        return epok::MathLibrary::clamp(control_camera.normal_response, epok::Fixed(0.0), epok::Fixed(32.0));
    }
    epok::Fixed control_yaw_cap(bool manual)const{
        if(manual){
            return epok::MathLibrary::clamp(control_camera.manual_cap, epok::Fixed(0.0), epok::Fixed(720.0));
        }
        return epok::MathLibrary::clamp(control_camera.normal_cap, epok::Fixed(0.0), epok::Fixed(720.0));
    }
    epok::Fixed control_yaw_snap(epok::Fixed from,epok::Fixed to)const{
        const auto delta_degrees_1=epok::MathLibrary::delta_degrees(from, to);
        if(((delta_degrees_1 > epok::Fixed(-0.01)) && (delta_degrees_1 < epok::Fixed(0.01)))){
            return to;
        }
        return from;
    }
    DungeonControlCamera control_camera_smooth(DungeonControlCamera state,epok::Fixed dt)const{
        const auto clamp_4=epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0));
        const auto product_3=(control_yaw_cap((((state).mode != 0u) || ((state).hold > epok::Fixed(0.0)))) * clamp_4);
        const auto product_2=(control_response((((state).mode != 0u) || ((state).hold > epok::Fixed(0.0)))) * clamp_4);
        const auto product_1=(epok::MathLibrary::clamp(control_camera.pitch_response, epok::Fixed(0.0), epok::Fixed(32.0)) * clamp_4);
        return make_controls_DungeonControlCamera(epok::MathLibrary::wrap_degrees(control_yaw_snap(((state).yaw + epok::MathLibrary::clamp(((epok::MathLibrary::delta_degrees(epok::MathLibrary::wrap_degrees((state).yaw), epok::MathLibrary::wrap_degrees((state).goal_yaw)) * product_2) / (epok::Fixed(1.0) + product_2)), (epok::Fixed(0.0) - product_3), product_3)), (state).goal_yaw)), control_pitch(((state).pitch + ((((state).goal_pitch - (state).pitch) * product_1) / (epok::Fixed(1.0) + product_1)))), (state).goal_yaw, (state).goal_pitch, (state).hold, (state).blend, (state).mode);
    }
    DungeonControlCamera control_camera_step(DungeonControlCamera state,DungeonControlCameraInput input)const override{
        return control_camera_smooth(control_camera_goals(control_camera_manual(control_camera_prepare(state, input), input), input), (input).dt);
    }
    epok::Fixed control_facing(epok::MovementIntent intent,DungeonControlCamera state,epok::Fixed facing,epok::Fixed target_yaw,bool target,bool attack_owns,bool fire_pressed,epok::Fixed dt)const{
        if((((state).mode == 0u) && ((intent).strength > epok::Fixed(0.0)))){
            return epok::MathLibrary::move_toward_degrees(epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0)), ((state).yaw + epok::MathLibrary::heading_degrees((intent).x, (intent).y)), (epok::MathLibrary::clamp(control_movement.turn_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
        }
        if((((state).mode == 1u) && target)){
            if((((intent).strength > epok::Fixed(0.0)) && !(attack_owns))){
                return epok::MathLibrary::move_toward_degrees(epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0)), ((state).yaw + epok::MathLibrary::heading_degrees((intent).x, (intent).y)), (epok::MathLibrary::clamp(control_movement.turn_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
            }
            if(fire_pressed){
                return epok::MathLibrary::wrap_degrees(target_yaw);
            }
            return epok::MathLibrary::move_toward_degrees(epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0)), epok::MathLibrary::wrap_degrees(target_yaw), (epok::MathLibrary::clamp(control_movement.turn_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
        }
        if(((state).mode == 2u)){
            return epok::MathLibrary::move_toward_degrees(epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0)), (state).yaw, (epok::MathLibrary::clamp(control_movement.turn_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
        }
        return epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0));
    }
    DungeonControlMove control_movement_step(epok::MovementIntent intent,DungeonControlCamera state,epok::Fixed facing,epok::Fixed target_yaw,bool target,bool attack_owns,bool fire_pressed,epok::Fixed dt)const override{
        const auto sine_degrees_3=epok::MathLibrary::sine_degrees((state).yaw);
        const auto cosine_degrees_2=epok::MathLibrary::cosine_degrees((state).yaw);
        const auto clamp_1=epok::MathLibrary::clamp(control_movement.speed, epok::Fixed(0.0), epok::Fixed(64.0));
        return make_controls_DungeonControlMove((((sine_degrees_3 * (intent).y) + (cosine_degrees_2 * (intent).x)) * clamp_1), (((cosine_degrees_2 * (intent).y) - (sine_degrees_3 * (intent).x)) * clamp_1), control_facing(intent, state, epok::MathLibrary::clamp(facing, epok::Fixed(-1440.0), epok::Fixed(1440.0)), epok::MathLibrary::wrap_degrees(target_yaw), target, attack_owns, fire_pressed, dt));
    }
    DungeonControlView control_view_proposal(epok::GameplayVector3 position,DungeonControlCamera state,epok::Fixed shoulder_distance)const override{
        const auto sine_degrees_8=epok::MathLibrary::sine_degrees((state).yaw);
        const auto cosine_degrees_7=epok::MathLibrary::cosine_degrees((state).yaw);
        const auto clamp_6=epok::MathLibrary::clamp((state).blend, epok::Fixed(0.0), epok::Fixed(1.0));
        const auto clamp_5=epok::MathLibrary::clamp(control_view.boom, epok::Fixed(0.01), epok::Fixed(64.0));
        const auto clamp_4=epok::MathLibrary::clamp(control_view.height, epok::Fixed(-32.0), epok::Fixed(32.0));
        const auto product_3=(epok::MathLibrary::clamp(control_view.shoulder_offset, epok::Fixed(-32.0), epok::Fixed(32.0)) * clamp_6);
        const auto product_2=((epok::MathLibrary::clamp(shoulder_distance, epok::MathLibrary::clamp(control_view.shoulder_min, epok::Fixed(0.01), epok::Fixed(64.0)), epok::Fixed(64.0)) - clamp_5) * clamp_6);
        const auto product_1=((clamp_5 + product_2) * epok::MathLibrary::cosine_degrees((state).pitch));
        return make_controls_DungeonControlView(epok::MathLibrary::vector3((position).x, ((position).y + (clamp_4 + ((epok::MathLibrary::clamp(control_view.aim_height, epok::Fixed(-32.0), epok::Fixed(32.0)) - clamp_4) * clamp_6))), (position).z), epok::MathLibrary::vector3(((cosine_degrees_7 * product_3) - (sine_degrees_8 * product_1)), ((clamp_5 + product_2) * epok::MathLibrary::sine_degrees((state).pitch)), ((epok::Fixed(0.0) - (sine_degrees_8 * product_3)) - (cosine_degrees_7 * product_1))), (clamp_5 + product_2), (state).yaw, (state).pitch);
    }
    epok::Fixed control_focus_height(bool enabled,epok::Fixed center,epok::Fixed scale)const override{
        if(enabled){
            return (epok::MathLibrary::clamp(center, epok::Fixed(-32.0), epok::Fixed(32.0)) * epok::MathLibrary::clamp(scale, epok::Fixed(-32.0), epok::Fixed(32.0)));
        }
        return epok::MathLibrary::clamp(control_aim.focus_height, epok::Fixed(-32.0), epok::Fixed(32.0));
    }
    DungeonControlAngles control_target_angles(epok::GameplayVector3 origin,epok::GameplayVector3 target,epok::Fixed focus)const override{
        const auto clamp_3=epok::MathLibrary::clamp(((target).z - (origin).z), epok::Fixed(-32.0), epok::Fixed(32.0));
        const auto clamp_2=epok::MathLibrary::clamp(((target).x - (origin).x), epok::Fixed(-32.0), epok::Fixed(32.0));
        const auto length2_1=epok::MathLibrary::length2(clamp_2, clamp_3);
        if((length2_1 == epok::Fixed(0.0))){
            if(((target).y >= (origin).y)){
                return make_controls_DungeonControlAngles(epok::MathLibrary::heading_degrees(clamp_2, clamp_3), control_pitch(epok::Fixed(89.0)));
            }
            return make_controls_DungeonControlAngles(epok::MathLibrary::heading_degrees(clamp_2, clamp_3), control_pitch(epok::Fixed(-89.0)));
        }
        return make_controls_DungeonControlAngles(epok::MathLibrary::heading_degrees(clamp_2, clamp_3), epok::MathLibrary::delta_degrees(epok::Fixed(0.0), epok::MathLibrary::heading_degrees(epok::MathLibrary::clamp((((origin).y + epok::MathLibrary::clamp(control_view.height, epok::Fixed(-32.0), epok::Fixed(32.0))) - ((target).y + focus)), epok::Fixed(-128.0), epok::Fixed(128.0)), length2_1)));
    }
    epok::Fixed control_aim_step(epok::Fixed current,epok::Fixed camera_pitch,epok::Fixed target_pitch,epok::Fixed dt,bool free_aim,bool target)const override{
        const auto clamp_1=epok::MathLibrary::clamp(control_aim.pitch_min, epok::Fixed(-89.0), epok::Fixed(89.0));
        if(free_aim){
            return epok::MathLibrary::move_toward(current, epok::MathLibrary::clamp(camera_pitch, clamp_1, epok::MathLibrary::clamp(control_aim.pitch_max, clamp_1, epok::Fixed(89.0))), (epok::MathLibrary::clamp(control_aim.pose_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
        }
        if(target){
            return epok::MathLibrary::move_toward(current, epok::MathLibrary::clamp(target_pitch, clamp_1, epok::MathLibrary::clamp(control_aim.pitch_max, clamp_1, epok::Fixed(89.0))), (epok::MathLibrary::clamp(control_aim.pose_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
        }
        return epok::MathLibrary::move_toward(current, epok::MathLibrary::clamp(epok::Fixed(0.0), clamp_1, epok::MathLibrary::clamp(control_aim.pitch_max, clamp_1, epok::Fixed(89.0))), (epok::MathLibrary::clamp(control_aim.pose_rate, epok::Fixed(0.0), epok::Fixed(1440.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))));
    }
    epok::Fixed control_marker_brightness(epok::Fixed intro)const{
        if((intro > epok::Fixed(0.0))){
            return epok::MathLibrary::clamp(((((epok::MathLibrary::floor(epok::MathLibrary::clamp(control_marker.intro_frames, epok::Fixed(0.0), epok::Fixed(64.0))) + epok::Fixed(1.0)) - intro) * epok::MathLibrary::clamp(control_marker.brightness_step, epok::Fixed(0.0), epok::Fixed(255.0))) * epok::Fixed(2.0)), epok::Fixed(0.0), epok::Fixed(255.0));
        }
        return epok::MathLibrary::clamp((epok::MathLibrary::clamp(control_marker.brightness_end, epok::Fixed(0.0), epok::Fixed(255.0)) * epok::Fixed(2.0)), epok::Fixed(0.0), epok::Fixed(255.0));
    }
    uint32_t control_marker_begin()const override{
        return epok::MathLibrary::truncate_unsigned(epok::MathLibrary::floor(epok::MathLibrary::clamp(control_marker.intro_frames, epok::Fixed(0.0), epok::Fixed(64.0))));
    }
    DungeonControlMarker control_marker_step(epok::Fixed intro,epok::Fixed spin)const override{
        const auto clamp_3=epok::MathLibrary::clamp(control_marker.base_radius, epok::Fixed(1.0), epok::Fixed(4096.0));
        const auto floor_2=epok::MathLibrary::floor(epok::MathLibrary::clamp(spin, epok::Fixed(0.0), epok::Fixed(63.0)));
        const auto move_toward_1=epok::MathLibrary::move_toward(epok::MathLibrary::floor(epok::MathLibrary::clamp(intro, epok::Fixed(0.0), epok::MathLibrary::floor(epok::MathLibrary::clamp(control_marker.intro_frames, epok::Fixed(0.0), epok::Fixed(64.0))))), epok::Fixed(0.0), epok::Fixed(1.0));
        return make_controls_DungeonControlMarker(move_toward_1, ((floor_2 + epok::Fixed(1.0)) - (epok::MathLibrary::floor(((floor_2 + epok::Fixed(1.0)) / epok::Fixed(64.0))) * epok::Fixed(64.0))), (epok::MathLibrary::clamp(control_marker.size, epok::Fixed(0.0), epok::Fixed(32.0)) * epok::MathLibrary::clamp(((clamp_3 + (epok::MathLibrary::clamp(control_marker.step_radius, epok::Fixed(0.0), epok::Fixed(4096.0)) * move_toward_1)) / clamp_3), epok::Fixed(0.0), epok::Fixed(64.0))), epok::MathLibrary::truncate_unsigned(control_marker_brightness(move_toward_1)), (move_toward_1 > epok::Fixed(0.0)));
    }
    DungeonControlRig control_rig(uint32_t bones)const override{
        if((bones == 13u)){
            return make_controls_DungeonControlRig(2, 6, 7);
        }
        return make_controls_DungeonControlRig(-1, -1, -1);
    }
    epok::Fixed control_animation_rate(uint32_t state,epok::Fixed speed)const{
        if((state == 1u)){
            const auto clamp_1=epok::MathLibrary::clamp(control_animation.rate_min, epok::Fixed(0.0), epok::Fixed(2.0));
            return epok::MathLibrary::clamp((epok::MathLibrary::clamp(speed, epok::Fixed(0.0), epok::Fixed(64.0)) / epok::MathLibrary::clamp(control_animation.walk_cycle, epok::Fixed(0.001), epok::Fixed(64.0))), clamp_1, epok::MathLibrary::clamp(control_animation.rate_max, clamp_1, epok::Fixed(2.0)));
        }
        if((state == 2u)){
            const auto clamp_2=epok::MathLibrary::clamp(control_animation.rate_min, epok::Fixed(0.0), epok::Fixed(2.0));
            return epok::MathLibrary::clamp((epok::MathLibrary::clamp(speed, epok::Fixed(0.0), epok::Fixed(64.0)) / epok::MathLibrary::clamp(control_animation.run_cycle, epok::Fixed(0.001), epok::Fixed(64.0))), clamp_2, epok::MathLibrary::clamp(control_animation.rate_max, clamp_2, epok::Fixed(2.0)));
        }
        return epok::Fixed(0.0);
    }
    DungeonControlAnimation control_animation_select(uint32_t state,uint32_t playing,bool grounded,epok::Fixed vertical_speed,epok::Fixed speed,bool finished,bool jumped)const override{
        if(jumped){
            return make_controls_DungeonControlAnimation(3u, (3u <= 2u), ((3u == 1u) || (3u == 2u)), (3u != playing), control_animation_rate(3u, speed));
        }
        if(!(grounded)){
            if((vertical_speed > epok::Fixed(0.0))){
                return make_controls_DungeonControlAnimation(3u, (3u <= 2u), ((3u == 1u) || (3u == 2u)), (3u != playing), control_animation_rate(3u, speed));
            }
            return make_controls_DungeonControlAnimation(4u, (4u <= 2u), ((4u == 1u) || (4u == 2u)), (4u != playing), control_animation_rate(4u, speed));
        }
        if(((state == 3u) || (state == 4u))){
            return make_controls_DungeonControlAnimation(5u, (5u <= 2u), ((5u == 1u) || (5u == 2u)), (5u != playing), control_animation_rate(5u, speed));
        }
        if(((state != 5u) || finished)){
            if((speed < epok::MathLibrary::clamp(control_animation.idle_speed, epok::Fixed(0.0), epok::Fixed(64.0)))){
                return make_controls_DungeonControlAnimation(0u, (0u <= 2u), ((0u == 1u) || (0u == 2u)), (0u != playing), control_animation_rate(0u, speed));
            }
            if(((speed > epok::MathLibrary::clamp(control_animation.run_speed, epok::Fixed(0.0), epok::Fixed(64.0))) || ((state == 2u) && (speed > epok::MathLibrary::clamp(control_animation.keep_run_speed, epok::Fixed(0.0), epok::Fixed(64.0)))))){
                return make_controls_DungeonControlAnimation(2u, (2u <= 2u), ((2u == 1u) || (2u == 2u)), (2u != playing), control_animation_rate(2u, speed));
            }
            return make_controls_DungeonControlAnimation(1u, (1u <= 2u), ((1u == 1u) || (1u == 2u)), (1u != playing), control_animation_rate(1u, speed));
        }
        return make_controls_DungeonControlAnimation(state, (state <= 2u), ((state == 1u) || (state == 2u)), (state != playing), control_animation_rate(state, speed));
    }
    uint32_t control_phase_add(uint32_t phase,uint32_t increment)const{
        if((phase > (4294967295u - increment))){
            return 4294967295u;
        }
        return (phase + increment);
    }
    uint32_t control_frame_count(uint32_t frames)const{
        if((frames > 262144u)){
            return 262144u;
        }
        if((frames <= 1u)){
            return 2u;
        }
        return frames;
    }
    DungeonControlClock control_animation_clock(uint32_t phase,uint32_t frames,epok::Fixed rate,epok::Fixed dt,bool model_valid)const override{
        const auto phase_add_1=control_phase_add(phase, epok::MathLibrary::truncate_unsigned((((epok::MathLibrary::clamp(rate, epok::Fixed(0.0), epok::Fixed(2.0)) * epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0))) * epok::Fixed(60.0)) * epok::Fixed(4096.0))));
        if(model_valid){
            const auto product_3=((control_frame_count(frames) - 1u) * 8192u);
            const auto product_2=((phase_add_1 / product_3) * product_3);
            return make_controls_DungeonControlClock((phase_add_1 - product_2), ((phase_add_1 - product_2) / 4096u));
        }
        return make_controls_DungeonControlClock(phase_add_1, 0u);
    }
    epok::GameplayVector3 control_direction(epok::Fixed yaw,epok::Fixed pitch)const override{
        const auto cosine_degrees_1=epok::MathLibrary::cosine_degrees(pitch);
        return epok::MathLibrary::vector3((epok::MathLibrary::sine_degrees(yaw) * cosine_degrees_1), (epok::Fixed(0.0) - epok::MathLibrary::sine_degrees(pitch)), (epok::MathLibrary::cosine_degrees(yaw) * cosine_degrees_1));
    }
    epok::GameplayVector3 control_aim_ray(epok::Fixed yaw,epok::Fixed pitch)const override{
        const auto sine_degrees_5=epok::MathLibrary::sine_degrees(pitch);
        const auto cosine_degrees_4=epok::MathLibrary::cosine_degrees(pitch);
        const auto product_3=(epok::MathLibrary::sine_degrees(yaw) * cosine_degrees_4);
        const auto product_2=(epok::MathLibrary::cosine_degrees(yaw) * cosine_degrees_4);
        const auto clamp_1=epok::MathLibrary::clamp(control_aim.ray_distance, epok::Fixed(0.0), epok::Fixed(128.0));
        return epok::MathLibrary::vector3(((epok::MathLibrary::vector3(product_3, (epok::Fixed(0.0) - sine_degrees_5), product_2)).x * clamp_1), ((epok::MathLibrary::vector3(product_3, (epok::Fixed(0.0) - sine_degrees_5), product_2)).y * clamp_1), ((epok::MathLibrary::vector3(product_3, (epok::Fixed(0.0) - sine_degrees_5), product_2)).z * clamp_1));
    }
    uint32_t control_aim_mask(bool muzzle)const override{
        if(muzzle){
            return control_aim.muzzle_mask;
        }
        return control_aim.ray_mask;
    }
    uint32_t control_shot_aim(bool target,uint32_t mode)const override{
        if(target){
            return 1u;
        }
        if((mode == 2u)){
            return 2u;
        }
        return 0u;
    }
    DungeonControlDirection control_direction_toward(epok::GameplayVector3 delta)const override{
        if((((((delta).x < epok::Fixed(-64.0)) || ((delta).x > epok::Fixed(64.0))) || (((delta).y < epok::Fixed(-64.0)) || ((delta).y > epok::Fixed(64.0)))) || (((delta).z < epok::Fixed(-64.0)) || ((delta).z > epok::Fixed(64.0))))){
            return make_controls_DungeonControlDirection(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), false);
        }
        const auto length_2=epok::MathLibrary::length(delta);
        if((length_2 >= epok::Fixed(64.0))){
            return make_controls_DungeonControlDirection(epok::MathLibrary::vector3(epok::Fixed(0.0), epok::Fixed(0.0), epok::Fixed(0.0)), false);
        }
        const auto quotient_1=(epok::MathLibrary::floor((length_2 * epok::Fixed(1024.0))) / epok::Fixed(1024.0));
        if((quotient_1 > epok::Fixed(0.0))){
            return make_controls_DungeonControlDirection(epok::MathLibrary::vector3(((delta).x / quotient_1), ((delta).y / quotient_1), ((delta).z / quotient_1)), true);
        }
        return make_controls_DungeonControlDirection(delta, true);
    }
    epok::Fixed control_motion_speed(epok::GameplayVector3 displacement,epok::Fixed dt)const override{
        if((dt <= epok::Fixed(0.0))){
            return epok::Fixed(0.0);
        }
        const auto clamp_1=epok::MathLibrary::clamp(dt, epok::Fixed(0.0), epok::Fixed(1.0));
        return epok::MathLibrary::length2((epok::MathLibrary::clamp((displacement).x, epok::Fixed(-64.0), epok::Fixed(64.0)) / clamp_1), (epok::MathLibrary::clamp((displacement).z, epok::Fixed(-64.0), epok::Fixed(64.0)) / clamp_1));
    }
private:
    static uint32_t bounded(uint32_t value,uint32_t minimum,uint32_t maximum){return value<minimum?minimum:value>maximum?maximum:value;}
    static epok::Fixed bounded_size(epok::Fixed value){return value<epok::Fixed(0.0)?epok::Fixed(0.0):value>epok::Fixed(32.0)?epok::Fixed(32.0):value;}
};
EPOK_SPEED_END
