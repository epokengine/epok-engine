"""Combined combat-provider conformance; C++ PSX trace is the comparison baseline.

Independent legacy arithmetic/state checks remain in the focused host suites.
This fixture compares real authored project rules and their cross-subsystem calls;
it does not claim that a native trace is an independent behavioral oracle.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import uuid

from gameplay_probe import binary_reuse_matches, export_probe, run_modes
from verify_camera_rigs import ROOT, EDITOR, documents, write

NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NAMESPACE, "DungeonCombatConformance/" + name))
DRIVER, MAGIC, STEPS, WORDS = identity("driver"), 0x434F4D41, 240, 56
PREFIX_WORDS, TAIL_WORDS = WORDS, 14
ACCEPTED_WORDS, FEEDBACK_WORDS = PREFIX_WORDS + TAIL_WORDS, 20
CONTROLLER_PREFIX_WORDS, CONTROLLER_WORDS = ACCEPTED_WORDS + FEEDBACK_WORDS, 57
WORLD_PREFIX_WORDS, WORLD_WORDS = CONTROLLER_PREFIX_WORDS + CONTROLLER_WORDS, 74
WORDS = WORLD_PREFIX_WORDS + WORLD_WORDS
NATIVE = ROOT / "resources/templates/dungeon-complete/assets/scripts/DungeonCombat"
FLAVORS = ROOT / "resources/templates/dungeon-complete-flavors"
POLICIES = {
    "DungeonPlayerVitals": "070a2ac6-e00f-5e9c-8bd2-560acb25ad2f",
    "DungeonPlayerCombat": "e8e7c9ff-ff0e-5c38-9eec-5434e6a45354",
    "DungeonEncounter": "825024e2-d9f8-5578-8198-bc52b5c217f2",
    "DungeonRunFlow": "df3f7e3e-b99c-5530-9d86-a44f87e67c5f",
}


def common_files(project):
    files = list((ROOT / "runtime").glob("*.hpp"))
    files += list((ROOT / "runtime").glob("*.cpp"))
    files += [p for p in (ROOT / "src").glob("*.rs")
              if p.name.startswith(("blueprint", "lua_", "reflection", "header_extract", "script_"))]
    files += [project / "assets/scripts" / (name + "Base.hpp") for name in POLICIES]
    files += [NATIVE / (name + ".hpp") for name in POLICIES]
    files += [project / "assets/scripts" / name for name in ("DungeonRulesBinding.hpp", "DungeonCombatProbe.hpp")]
    files += [ROOT / "src" / name for name in ("project.rs", "game_helpers.rs", "scene_bank.rs", "actor_components.rs",
              "class_composition.rs", "object_model.rs", "staging_files.rs")]
    return sorted(set(files))


def common_contract(project):
    """Invalidate trace reuse when common source/compiler dependencies change."""
    digest = hashlib.sha256()
    files = common_files(project)
    for path in sorted(files):
        digest.update(path.name.encode("utf-8"))
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


def baseline_contract_matches(base_out, previous, current):
    if previous == current:
        return True
    if binary_reuse_matches(base_out / "cpp", previous, current):
        return True
    proof_path = base_out / "cpp/scoped-provenance-reuse.json"
    if not proof_path.exists():
        return False
    proof = json.loads(proof_path.read_text(encoding="utf-8"))
    observed = json.loads((base_out / "cpp/native_cpp-observation.json").read_text(encoding="utf-8"))
    changed = set(proof["changed_files"])
    allowed = {"src/blueprint_dependencies.rs", "src/lua_vm.rs", "src/lua_vm_value_tests.rs"}
    return (proof["previous_common_contract_sha256"] == previous and proof["current_common_contract_sha256"] == current
            and "src/blueprint_dependencies.rs" in changed and changed <= allowed
            and proof["regenerated_exe_sha256"] == proof["played_exe_sha256"] == observed["played_exe_sha256"])


def source():
    return r'''#pragma once
#include "DungeonPlayerVitalsBase.hpp"
#include "DungeonPlayerCombatBase.hpp"
#include "DungeonEncounterBase.hpp"
#include "DungeonRunFlowBase.hpp"
#include "projectile3d.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t dungeon_combat_probe[8]={};inline uint32_t dungeon_combat_trace[240][TRACE_WORDS]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") DungeonCombatProbe:public epok::Actor3D {
    uint32_t frame_=0,hash_=2166136261u,fail_=0,shots_=0,guard_fires_=0,boss_fires_=0;
    DungeonAttackState attack_{};DungeonGuardState guard_{};DungeonBossState boss_{};
    epok::HealthState guard_health_{},boss_health_{};
    epok::ProjectileHandle last_{};
    uint32_t target_phase_=0;
    DungeonControlCamera control_camera_{};
    epok::Fixed control_facing_=0.0,control_pose_=0.0,control_intro_=0.0,control_spin_=0.0;
    uint32_t control_animation_=0,control_playing_=5,control_clock_=0;
    epok::HealthState world_guard_{},world_boss_{},world_player_{100,25};
    epok::Fixed world_door_=0.0;
    uint32_t world_kills_=0,world_resets_=0,world_heals_=0;
    bool world_key_=false,world_pack_=false,world_guard_defeated_=false;
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerVitalsBase") epok::ObjectId vitals{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerCombatBase") epok::ObjectId combat{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonEncounterBase") epok::ObjectId encounter{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonRunFlowBase") epok::ObjectId flow{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ProjectilePool3DComponent") epok::ObjectId projectiles{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId wall{};
    void on_frame(uint32_t)override{
        using namespace epok;
        if(frame_>=240)return;
        auto* registry=active_object_registry;
        auto* v=registry?registry->resolve<DungeonPlayerVitalsBase>(vitals):nullptr;
        auto* c=registry?registry->resolve<DungeonPlayerCombatBase>(combat):nullptr;
        auto* e=registry?registry->resolve<DungeonEncounterBase>(encounter):nullptr;
        auto* p=registry?registry->resolve<ProjectilePool3DComponent>(projectiles):nullptr;
        if(!v||!c||!e||!p)return;
        if(!frame_){
            if(c->tuning.projectile_speed!=Fixed(22.0)||c->tuning.damage!=20||
                e->health_tuning.guard_health!=60||e->movement.activation_radius!=Fixed(14.0)||
                e->guard_timing.ai_interval.raw()!=137||e->boss_timing.boss_initial_delay!=Fixed(1.8))fail_|=1;
            v->start();guard_=e->guard_start(0);boss_=e->boss_start();
            guard_health_=e->starting_health(false);boss_health_=e->starting_health(true);
        }
        const Fixed dt(137,Fixed::RAW);
        const bool grounded=frame_%61>1,jumped=frame_%61==2,held=frame_%32<27,playing=frame_%4!=0;
        const auto attack=c->attack(attack_,grounded,jumped,held,playing,true);attack_=attack.state;
        const bool movement=c->blocks_movement(attack_,grounded,held);
        const auto input=c->target_input(frame_%13<3,frame_%13==0,frame_%17==0);
        const auto candidate=c->candidate(frame_%7!=0,true,frame_%11!=0,frame_%19!=0);
        const auto target=c->target_check(target_phase_,candidate,frame_%23!=0);target_phase_=target.phase;
        const DungeonTargetDistance distance{frame_%60?frame_%4:0xffffffffu,frame_%60?frame_%9:0xfffffff0u};
        const DungeonTargetDistance selected{frame_%59?frame_%3:0xffffffffu,frame_%59?frame_%8:0xffffffffu},best{frame_%5,frame_%7};
        const bool after=c->after_selected(distance,frame_%6,selected,frame_%4);
        const bool preferred=c->prefer_candidate(distance,frame_%6,frame_%7!=0,best,frame_%5,frame_%2!=0);
        auto spawn=c->projectile({0.0,0.0,frame_%2?Fixed(0.0):Fixed(4.0)},{1.0,0.0,0.0});
        ProjectileLaunchResult launched;
        if(attack.fire){launched=p->launch(0,2,spawn);if(launched.result==ProjectileResult::Applied){last_=launched.handle;++shots_;}}
        if(frame_%40==5){spawn.origin={0.0,0.0,4.0};spawn.lifetime=dt;launched=p->launch(0,2,spawn);if(launched.result==ProjectileResult::Applied)last_=launched.handle;}
        if(frame_%40==10){p->reset();spawn.origin={0.0,0.0,0.0};spawn.velocity={88.0,0.0,0.0};launched=p->launch(0,2,spawn);if(launched.result==ProjectileResult::Applied)last_=launched.handle;}
        ProjectileResult stopped=ProjectileResult::Unchanged;
        if(frame_%40==20){p->reset();stopped=p->stop(last_);if(stopped!=ProjectileResult::InvalidHandle)fail_|=2;}
        const auto batch=p->step(0,2,dt,{1,{},false});
        const auto first=p->result(batch,0),second=p->result(batch,1);
        bool preserved=true;
        if(first.state==ProjectileStepState::Hit){
            if(first.target!=wall)fail_|=4;
            p->reset();const auto again=p->result(batch,0);
            preserved=again.state==first.state&&again.target==first.target&&again.point.x==first.point.x;
            if(!preserved)fail_|=8;
        }
        const auto snapshot=p->snapshot(last_);
        v->advance(dt);HealthChange damage;
        if(frame_%15==0)damage=v->damage(12,v->health.current>0);
        if(frame_%47==0)v->heal(35);
        const auto scheduled=e->guard_schedule(guard_,dt);guard_=scheduled.state;
        uint32_t guard_action=scheduled.action;
        if(scheduled.action==6){guard_=e->guard_start((frame_/40)%6);guard_health_=e->starting_health(false);}
        else if(scheduled.action==7){
            const bool close=frame_%80<70,observed=frame_%90<80;
            const auto decision=e->guard_decide(guard_,scheduled.dt,close,observed,frame_%40<20,frame_%37==0);
            guard_=decision.state;guard_action=decision.action;guard_fires_+=guard_action==5;
        }
        if(frame_%50==0&&guard_health_.current){const auto hit=HealthLibrary::damage(guard_health_,20);guard_health_=hit.state;guard_=e->guard_hit(guard_,hit.depleted);}
        const bool boss_sight=e->boss_needs_sight(boss_,dt);
        const auto boss=e->boss_advance(boss_,boss_health_.current,dt,boss_sight&&frame_%80<75,frame_%40<20);
        boss_=boss.state;boss_fires_+=boss.action==5;
        if(frame_%12==0&&boss_health_.current){const auto hit=HealthLibrary::damage(boss_health_,20);boss_health_=hit.state;boss_=e->boss_hit(boss_);}
        if(e->should_activate(false,true,true)!=true||e->should_activate(true,true,true)||
            e->should_unload(guard_,true,false)!=(guard_.dead_time<=Fixed(0.0)))fail_|=16;
        const uint32_t words[]={frame_,attack_.phase,attack_.attacking,attack_.shot_sent,uint32_t(attack.clip),attack.fire,
            attack.invalidate_locomotion,attack.starting,movement,input,target.phase,target.keep,target.query_sight,candidate,after,preferred,
            uint32_t(spawn.velocity.x.raw()),uint32_t(spawn.lifetime.raw()),uint32_t(launched.result),uint32_t(first.state),uint32_t(second.state),
            uint32_t(first.point.x.raw()),uint32_t(second.point.x.raw()),first.target==wall,second.target==wall,p->active_count(),snapshot.valid,snapshot.active,
            uint32_t(snapshot.lifetime.raw()),v->health.current,uint32_t(v->immunity.raw()),uint32_t(guard_.timer.raw()),uint32_t(guard_.windup.raw()),
            uint32_t(guard_.flash.raw()),uint32_t(guard_.dead_time.raw()),uint32_t(guard_.sight_timer.raw()),uint32_t(guard_.ai_time.raw()),guard_.patrol,
            guard_.can_see,guard_action,uint32_t(scheduled.dt.raw()),boss_health_.current,uint32_t(boss_.timer.raw()),uint32_t(boss_.windup.raw()),
            uint32_t(boss_.flash.raw()),boss.action,boss.fan,uint32_t(boss.speed.raw()),boss_sight,guard_health_.current,guard_fires_,boss_fires_,damage.applied,
            preserved,uint32_t(stopped),first.tag};
        static_assert(sizeof(words)/sizeof(words[0])==56);
        for(unsigned n=0;n<56;++n){dungeon_combat_trace[frame_][n]=words[n];hash_=(hash_^words[n])*16777619u;}
        // The accepted prefix above is observed before the new, independent
        // shot rules. Editing this tuning must not change guard/boss timing.
        if(frame_==120)e->shot_tuning={9.0,11.0,3.0,30.0,.5,.75,24,32,1.5};
        const bool edited=frame_>=120,fan=frame_%2!=0;
        const uint32_t ordinal=frame_%3,count=e->pellet_count(fan);
        const int32_t lane=e->pellet_index(ordinal,fan);
        const GameplayVector3 enemy_direction{Fixed((int32_t(frame_%9)-4)*512,Fixed::RAW),
            Fixed((int32_t(frame_%5)-2)*512,Fixed::RAW),Fixed((int32_t(frame_%7)-3)*512,Fixed::RAW)};
        const auto enemy=e->projectile({1.0,2.0,3.0},enemy_direction,lane,fan);
        const auto target_point=e->shot_target({1.0,Fixed(8192+int32_t(frame_%5)*256,Fixed::RAW),3.0});
        const uint32_t tail[]={edited,fan,ordinal,count,uint32_t(lane),uint32_t(enemy.velocity.x.raw()),
            uint32_t(enemy.velocity.y.raw()),uint32_t(enemy.velocity.z.raw()),uint32_t(enemy.lifetime.raw()),
            uint32_t(e->shot_size(fan).raw()),e->shot_damage(false),e->shot_damage(true),uint32_t(target_point.y.raw()),enemy.tag};
        static_assert(sizeof(tail)/sizeof(tail[0])==14);
        for(unsigned n=0;n<14;++n){dungeon_combat_trace[frame_][56+n]=tail[n];hash_=(hash_^tail[n])*16777619u;}

        const uint32_t feedback_phase=frame_/80,feedback_slot=frame_%8;
        const bool feedback_enemy=frame_%2!=0,feedback_restart=frame_%5==0,feedback_advance=frame_%3!=0;
        const uint32_t feedback_layers=feedback_slot==7?0xffffffffu:feedback_slot;
        const uint32_t feedback_ages[]={0,1,2,3,4,5,6,7,8,9,10,11,65534,65535,0xffffffffu,0};
        const uint32_t feedback_remaining[]={0,1,220000,219999,10000000,0xffffffffu,0xffffffffu,220000};
        const uint32_t feedback_elapsed[]={0,1,1365,33333,219999,220000,0xffffffffu,0};
        const uint32_t feedback_durations[]={0,1,0xffffffffu,220000,10000000,0xffffffffu,0xffffffffu,0xffffffffu};

        // Pure feedback calls occur after the accepted 70-word observation.
        // These distinct properties cannot alter attack/target/shot timing.
        if(frame_==80){
            c->impact_tuning={9,17};c->shot_style={.4,20,40,80};
            c->effect_tuning={.6,.9,8,9,2,4,12,3,7};
            c->feedback_tuning={1000,9,20,40,30,50,60,70,false};
        }
        if(feedback_phase==2){
            c->impact_tuning={0x80000001u,0xffffffffu};c->shot_style={Fixed(0x7fffffff,Fixed::RAW),0xffffffffu,256,0};
            const uint32_t end=frame_%7==0?0u:0xffffffffu;
            c->effect_tuning={-1.0,100.0,0xffffffffu,0xffffffffu,0xffffffffu,end,0xffffffffu,0,end};
            c->feedback_tuning={feedback_durations[feedback_slot],feedback_slot%2?0xffffffffu:0u,
                feedback_slot%2?0xffffffffu:0u,0xffffffffu,256,0xffffffffu,0xffffffffu,0xffffffffu,feedback_enemy};
        }
        const auto feedback_trace=c->trace_settings(feedback_enemy);
        const auto feedback_impact=c->impact(feedback_enemy,frame_%3!=0,frame_%5==0,feedback_layers);
        const auto feedback_visual=c->shot_visual();
        const auto feedback_effect=c->effect_step(frame_%2,feedback_ages[frame_%16],feedback_advance);
        const auto feedback_border=c->damage_feedback(feedback_remaining[feedback_slot],feedback_elapsed[feedback_slot],feedback_restart);
        const auto feedback_rumble=c->damage_rumble();
        const uint32_t feedback_tail[]={feedback_trace.mask,feedback_trace.ignore_player,feedback_impact,
            uint32_t(feedback_visual.size.raw()),feedback_visual.red,feedback_visual.green,feedback_visual.blue,
            feedback_effect.age,feedback_effect.frame,uint32_t(feedback_effect.size.raw()),feedback_effect.playing,
            feedback_border.remaining_us,feedback_border.enabled,feedback_border.border_pixels,feedback_border.red,feedback_border.green,feedback_border.blue,
            feedback_rumble.large,feedback_rumble.small,feedback_rumble.duration_ms};
        static_assert(sizeof(feedback_tail)/sizeof(feedback_tail[0])==20);
        for(unsigned n=0;n<20;++n){dungeon_combat_trace[frame_][70+n]=feedback_tail[n];hash_=(hash_^feedback_tail[n])*16777619u;}

        // Controller results use distinct tuning/state and have no mechanism
        // side effects. The complete accepted 90-word source precedes them.
        const uint32_t ctrl_phase=frame_/80,ctrl_slot=frame_%8;
        if(frame_==80){
            c->control_movement={9.0,180.0};c->control_camera={25.0,90.0,45.0,-15.0,60.0,3.0,5.0,10.0,70.0,110.0,9.0,.5};
            c->control_view={7.0,1.2,1.5,.75,.8};c->control_aim={-30.0,40.0,180.0,.9,50.0,9,3};
            c->control_marker={.5,4.0,96.0,32.0,8.0,100.0};c->control_animation={3.0,7.0,.3,1.4,.15,4.0,3.2};
        }
        if(frame_==160){
            const Fixed high(0x7fffffff,Fixed::RAW);
            c->control_movement={high,high};c->control_camera={high,high,high,high,high,high,high,high,high,high,high,high};
            c->control_view={high,high,high,high,high};c->control_aim={Fixed(-0x7fffffff,Fixed::RAW),high,high,high,high,0xffffffffu,0xffffffffu};
            c->control_marker={high,high,0.0,high,high,high};c->control_animation={0.0,0.0,high,0.0,high,high,high};
        }
        const Fixed ctrl_dt=ctrl_phase<2?Fixed(137,Fixed::RAW):ctrl_slot==0?Fixed(-1.0):ctrl_slot==1?Fixed(0.0):ctrl_slot==2?Fixed(1.0):Fixed(0x7fffffff,Fixed::RAW);
        const auto ctrl_gate=c->control_gate(frame_%17!=0,frame_%19==0,frame_%31!=0,frame_%13!=0);
        const uint32_t ctrl_buttons=frame_%16;const bool ctrl_suppress=frame_%53==0;
        const Fixed ctrl_x=ctrl_phase<2?Fixed((int32_t(frame_%9)-4)*1024,Fixed::RAW):Fixed(0x7fffffff,Fixed::RAW);
        const Fixed ctrl_y=ctrl_phase<2?Fixed((int32_t(frame_%7)-3)*1200,Fixed::RAW):Fixed(-0x7fffffff,Fixed::RAW);
        const auto ctrl_intent=c->control_intent(ctrl_x,ctrl_y,ctrl_buttons&1,ctrl_buttons&2,ctrl_buttons&4,ctrl_buttons&8,ctrl_suppress);
        const auto ctrl_look=c->control_intent(Fixed((int32_t(frame_%7)-3)*600,Fixed::RAW),Fixed((int32_t(frame_%5)-2)*700,Fixed::RAW),false,false,false,false,false);
        const bool ctrl_free=frame_%23>=12,ctrl_target=!ctrl_free&&frame_%29<20;
        const auto ctrl_focus=c->control_focus_height(frame_%3!=0,ctrl_phase<2?Fixed(.8):Fixed(0x7fffffff,Fixed::RAW),ctrl_phase<2?Fixed(1.0):Fixed(0x7fffffff,Fixed::RAW));
        const GameplayVector3 ctrl_origin{1.0,2.0,3.0},ctrl_target_point{frame_%9?Fixed(3.0):Fixed(1.0),frame_%4?Fixed(0.0):Fixed(3.0),frame_%9?Fixed(8.0):Fixed(3.0)};
        const auto ctrl_angles=c->control_target_angles(ctrl_origin,ctrl_target_point,ctrl_focus);
        if(!frame_)control_camera_=c->control_camera_start(0.0);
        control_camera_=c->control_camera_step(control_camera_,{ctrl_look,ctrl_dt,control_facing_,ctrl_angles.yaw,ctrl_angles.pitch,ctrl_free,ctrl_target,ctrl_intent.strength.raw()!=0});
        const auto ctrl_move=c->control_movement_step(ctrl_intent,control_camera_,control_facing_,ctrl_angles.yaw,ctrl_target,ctrl_suppress,frame_%47==0,ctrl_dt);control_facing_=ctrl_move.facing;
        const auto ctrl_view=c->control_view_proposal(ctrl_origin,control_camera_,ctrl_phase==0?Fixed(2.15):ctrl_phase==1?Fixed(-1.0):Fixed(0x7fffffff,Fixed::RAW));
        control_pose_=c->control_aim_step(control_pose_,control_camera_.pitch,ctrl_angles.pitch,ctrl_dt,ctrl_free,ctrl_target);
        const auto ctrl_begin=c->control_marker_begin();if(frame_%40==0){control_intro_=Fixed(int32_t(ctrl_begin)*4096,Fixed::RAW);control_spin_=0.0;}
        const auto ctrl_marker=c->control_marker_step(control_intro_,control_spin_);control_intro_=ctrl_marker.intro;control_spin_=ctrl_marker.spin;
        const auto ctrl_rig=c->control_rig(frame_%3?13u:12u);
        const Fixed ctrl_speed=c->control_motion_speed({ctrl_move.x*Fixed(137,Fixed::RAW),0.0,ctrl_move.z*Fixed(137,Fixed::RAW)},ctrl_dt);
        const auto ctrl_animation=c->control_animation_select(control_animation_,control_playing_,frame_%61<55,frame_%31<15?Fixed(1.0):Fixed(-1.0),ctrl_speed,frame_%17==0,frame_%61==3);
        control_animation_=ctrl_animation.state;if(ctrl_animation.switch_clip&&frame_%7!=0){control_playing_=ctrl_animation.state;control_clock_=0;}
        const auto ctrl_clock=c->control_animation_clock(ctrl_phase==2&&frame_%17==0?0xffffffffu:control_clock_,ctrl_phase<2?23u:ctrl_slot==0?0u:ctrl_slot==1?1u:0xffffffffu,ctrl_animation.rate,ctrl_dt,frame_%11!=0);control_clock_=ctrl_clock.phase;
        const auto ctrl_direction=c->control_direction(control_facing_,control_pose_),ctrl_ray=c->control_aim_ray(control_camera_.yaw,control_camera_.pitch);
        const auto ctrl_ray_mask=c->control_aim_mask(false),ctrl_muzzle_mask=c->control_aim_mask(true),ctrl_shot_aim=c->control_shot_aim(ctrl_target,control_camera_.mode);
        const GameplayVector3 ctrl_deltas[]={{0.0,0.0,0.0},{1.0,2.0,3.0},{-3.0,.5,8.0},{40.0,0.0,0.0},{63.0,.5,0.0},{64.0,0.0,0.0},{-65.0,0.0,0.0},{Fixed(0x7fffffff,Fixed::RAW),0.0,0.0}};
        const auto ctrl_normalized=c->control_direction_toward(ctrl_deltas[ctrl_slot]);
        const uint32_t controller_tail[]={ctrl_gate,uint32_t(ctrl_intent.x.raw()),uint32_t(ctrl_intent.y.raw()),uint32_t(ctrl_intent.strength.raw()),
            uint32_t(control_camera_.yaw.raw()),uint32_t(control_camera_.pitch.raw()),uint32_t(control_camera_.goal_yaw.raw()),uint32_t(control_camera_.goal_pitch.raw()),uint32_t(control_camera_.hold.raw()),uint32_t(control_camera_.blend.raw()),control_camera_.mode,
            uint32_t(ctrl_move.x.raw()),uint32_t(ctrl_move.z.raw()),uint32_t(ctrl_move.facing.raw()),
            uint32_t(ctrl_view.origin.x.raw()),uint32_t(ctrl_view.origin.y.raw()),uint32_t(ctrl_view.origin.z.raw()),uint32_t(ctrl_view.offset.x.raw()),uint32_t(ctrl_view.offset.y.raw()),uint32_t(ctrl_view.offset.z.raw()),uint32_t(ctrl_view.distance.raw()),uint32_t(ctrl_view.yaw.raw()),uint32_t(ctrl_view.pitch.raw()),
            uint32_t(ctrl_focus.raw()),uint32_t(ctrl_angles.yaw.raw()),uint32_t(ctrl_angles.pitch.raw()),uint32_t(control_pose_.raw()),ctrl_begin,
            uint32_t(ctrl_marker.intro.raw()),uint32_t(ctrl_marker.spin.raw()),uint32_t(ctrl_marker.size.raw()),ctrl_marker.tint,ctrl_marker.average,
            uint32_t(ctrl_rig.bone),uint32_t(ctrl_rig.stop_a),uint32_t(ctrl_rig.stop_b),ctrl_animation.state,ctrl_animation.loop,ctrl_animation.locomotion,ctrl_animation.switch_clip,uint32_t(ctrl_animation.rate.raw()),ctrl_clock.phase,ctrl_clock.ticks,
            uint32_t(ctrl_direction.x.raw()),uint32_t(ctrl_direction.y.raw()),uint32_t(ctrl_direction.z.raw()),uint32_t(ctrl_ray.x.raw()),uint32_t(ctrl_ray.y.raw()),uint32_t(ctrl_ray.z.raw()),ctrl_ray_mask,ctrl_muzzle_mask,ctrl_shot_aim,
            uint32_t(ctrl_normalized.value.x.raw()),uint32_t(ctrl_normalized.value.y.raw()),uint32_t(ctrl_normalized.value.z.raw()),ctrl_normalized.valid,uint32_t(ctrl_speed.raw())};
        static_assert(sizeof(controller_tail)/sizeof(controller_tail[0])==57);
        for(unsigned n=0;n<57;++n){dungeon_combat_trace[frame_][90+n]=controller_tail[n];hash_=(hash_^controller_tail[n])*16777619u;}

        // Same scenario, selected-language encounter/world and RunFlow rules.
        // Nothing above the original147 observations is changed or replayed.
        auto* world_flow=registry?registry->resolve<DungeonRunFlowBase>(flow):nullptr;if(!world_flow)return;
        const uint32_t world_phase=frame_/80,world_sub=frame_%40,world_slot=frame_%8;
        if(frame_==80){
            e->world_damage={3,5,true,false,true,false,false};e->guard_colors={30,60,90,150,170,190,200,180,160};
            e->boss_colors={100,120,140,160,180,200,210,220,230,240,250,255};e->enemy_shot_color={25,50,75};
            e->world_doors={7.0,3.0,2.0,.5,3.0,40,80,120};e->world_pickups={2.0,4.0,80,90.0,.25,-9.0,9.0,30.0,60.0};
            e->world_animation={12,18,7,false};e->world_motion={-.1,.05};
        }
        if(frame_==160){
            const Fixed high(0x7fffffff,Fixed::RAW),low(-0x7fffffff,Fixed::RAW);
            e->world_damage={0xffffffffu,0xffffffffu,true,true,true,true,true};e->guard_colors={0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu};
            e->boss_colors={0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu};
            e->enemy_shot_color={0xffffffffu,0xffffffffu,0xffffffffu};e->world_doors={high,high,high,high,high,0xffffffffu,0xffffffffu,0xffffffffu};
            e->world_pickups={high,high,0xffffffffu,high,high,low,high,0.0,high};e->world_animation={-1,0x7fffffff,0x7fffffff,true};e->world_motion={low,0.0};
        }
        if(!world_sub){
            world_guard_=e->starting_health(false);world_boss_=e->starting_health(true);world_player_={100,25};world_door_=0.0;
            world_key_=world_pack_=world_guard_defeated_=false;world_flow->start();for(uint32_t n=0;n<5;++n)world_flow->advance(Fixed(1.0),true);
        }
        if(world_sub==20)world_flow->try_enter_boss(world_key_,Fixed(54.0));
        const Fixed world_dt=world_phase<2?Fixed(137,Fixed::RAW):world_slot?Fixed(0x7fffffff,Fixed::RAW):Fixed(0.0);
        const uint32_t world_state_before=world_flow->status.state,world_index=world_sub>=20?3u:frame_%4;
        const bool world_locked=frame_%3!=0,world_near_open=world_sub<30,world_near_safety=world_sub>=22&&world_sub<=26;
        const Fixed world_door_before=world_door_;
        const auto world_open=e->world_door_open(world_index,world_locked,world_key_,world_state_before,world_door_,world_near_open,world_near_safety);
        const auto world_radius=e->world_door_radius(false),world_safety=e->world_door_radius(true);
        const auto world_door=e->world_door_step(world_index,world_locked,world_key_,world_state_before,world_door_,world_near_open,world_near_safety,world_dt);world_door_=world_door.progress;
        const bool world_is_boss=world_sub>=20;const auto& world_before=world_is_boss?world_boss_:world_guard_;
        const uint32_t world_bases[]={0,1,20,20,80,0x7fffffffu,0xffffffffu,20},world_bonuses[]={0,8,8,0,8,0,0xffffffffu,0};
        const uint32_t world_base=world_is_boss&&world_sub<24?0u:world_bases[world_slot],world_bonus=world_is_boss&&world_sub<24?0u:world_bonuses[world_slot];
        const auto world_total=e->world_damage_total(world_base,world_bonus);
        const auto world_eligible=e->world_damage_eligible(world_is_boss,world_flow->status.alive,world_flow->status.state==1,world_before.current,true,0.0,world_guard_defeated_);
        const uint32_t world_health_before=world_before.current;HealthChange world_change{};
        if(world_eligible){auto& health=world_is_boss?world_boss_:world_guard_;world_change=HealthLibrary::damage(health,int32_t(world_total));health=world_change.state;}
        const auto world_hit=e->world_hit_result(world_is_boss,world_change.depleted,(world_is_boss?world_boss_:world_guard_).current);
        const auto world_reward=e->world_reward_count(world_kills_,world_eligible?world_hit.reward:0);
        if(world_eligible){world_kills_=world_reward;if(world_hit.defeated)world_guard_defeated_=true;if(world_hit.victory)world_flow->win();if(world_hit.reset_projectiles)++world_resets_;}
        HealthChange world_player_change{};
        if(world_sub==17&&(frame_/40)%2&&world_flow->status.alive){world_player_change=HealthLibrary::damage(world_player_,0x7fffffff);world_player_=world_player_change.state;}
        const auto world_death=e->world_player_death(world_player_change.depleted);if(world_death.defeat)world_flow->die();if(world_death.reset_projectiles)++world_resets_;
        const auto world_hide=e->world_hide_boss(world_flow->status.state,world_sub%5!=0);
        const Fixed world_flash=world_sub%9==0?Fixed(.1):Fixed(0.0),world_windup=world_sub%7==0?Fixed(.1):Fixed(0.0);
        const auto world_color=e->world_enemy_color(world_is_boss,world_flash,world_windup,frame_%2!=0),world_shot=e->world_shot_color();
        const auto world_death_clip=e->world_death_animation(),world_clip=e->world_enemy_animation(world_is_boss,world_sub%7,int32_t(frame_%4)-1);
        const GameplayVector3 world_position{frame_%4==0?Fixed(-10.0):frame_%4==1?Fixed(10.0):Fixed(0.0),frame_%7==3?Fixed(3.0):Fixed(0.0),40.0};
        const uint32_t world_kind=frame_%4;const bool world_can_collect=world_flow->status.alive&&(world_kind==0?!world_key_:!world_pack_&&world_player_.current<world_player_.maximum);
        const bool world_near_pickup=frame_%5!=4;
        const auto world_visible=e->world_pickup_visible(world_kind,world_position),world_collect=e->world_pickup_eligible(world_can_collect,world_near_pickup,world_position.y);
        const auto world_pick_radius=e->world_pickup_radius();const auto world_heal=e->world_pickup_heal();
        const Fixed world_elapsed(int32_t(frame_)*137,Fixed::RAW);const auto world_bob=e->world_pickup_bob(world_phase<2?world_elapsed:Fixed(0x7fffffff,Fixed::RAW));
        if(world_collect){if(!world_kind)world_key_=true;else{const auto healed=HealthLibrary::heal(world_player_,int32_t(world_heal));world_player_=healed.state;if(healed.applied){world_pack_=true;world_heals_+=healed.applied;}}}
        const GameplayVector3 world_deltas[]={{0.0,0.0,0.0},{1.0,2.0,3.0},{-3.0,.5,8.0},{40.0,0.0,0.0},{63.0,.5,0.0},{64.0,0.0,0.0},{-65.0,0.0,0.0},{Fixed(0x7fffffff,Fixed::RAW),0.0,0.0}};
        const auto world_aim=e->world_aim_direction(world_deltas[world_slot]);
        const GameplayVector3 world_origin{1.0,0.0,2.0},world_target{4.0,1.0,9.0},world_home{2.0,0.0,3.0};
        const auto world_patrol=e->world_patrol_goal(world_home,frame_%2!=0);
        const auto world_motion=e->world_enemy_motion(world_is_boss,world_sub%7,frame_%2!=0,frame_%3!=0,frame_%4!=0,world_origin,world_target,world_home,world_dt,Fixed(.85));
        const auto world_due=e->world_boss_due(Fixed(int32_t(world_sub)*68,Fixed::RAW));
        const uint32_t world_tail[]={world_eligible,world_total,world_reward,
            world_hit.defeated,world_hit.disable_collider,world_hit.death_clip,world_hit.victory,world_hit.reset_projectiles,world_hit.explosion,world_hit.hud_dirty,world_hit.reward,world_hit.sound,
            world_death.defeat,world_death.clear_aim,world_death.reset_projectiles,world_hide,
            world_color.red,world_color.green,world_color.blue,world_shot.red,world_shot.green,world_shot.blue,
            uint32_t(world_death_clip.clip),world_death_clip.play,world_death_clip.loop,world_death_clip.charge_cue,
            uint32_t(world_clip.clip),world_clip.play,world_clip.loop,world_clip.charge_cue,world_open,uint32_t(world_radius.raw()),uint32_t(world_safety.raw()),
            uint32_t(world_door.progress.raw()),world_door.opening,world_door.moved,world_door.opened,world_door.locked_tint,uint32_t(world_door.offset.raw()),world_door.color.red,world_door.color.green,world_door.color.blue,
            world_visible,world_collect,uint32_t(world_pick_radius.raw()),world_heal,uint32_t(world_bob.raw()),
            uint32_t(world_aim.value.x.raw()),uint32_t(world_aim.value.y.raw()),uint32_t(world_aim.value.z.raw()),world_aim.valid,
            uint32_t(world_patrol.x.raw()),uint32_t(world_patrol.y.raw()),uint32_t(world_patrol.z.raw()),
            uint32_t(world_motion.delta.x.raw()),uint32_t(world_motion.delta.y.raw()),uint32_t(world_motion.delta.z.raw()),uint32_t(world_motion.yaw.raw()),uint32_t(world_motion.patrol_yaw.raw()),world_motion.move,world_motion.turn,world_motion.patrol_turn,world_motion.valid,world_due,
            world_guard_.current,world_boss_.current,world_player_.current,world_kills_,uint32_t(world_door_.raw()),world_key_,world_pack_,world_flow->status.state,world_resets_,world_heals_};
        static_assert(sizeof(world_tail)/sizeof(world_tail[0])==74);
        for(unsigned n=0;n<74;++n){dungeon_combat_trace[frame_][147+n]=world_tail[n];hash_=(hash_^world_tail[n])*16777619u;}
        if(++frame_==240){
            if(!shots_||!guard_fires_||!boss_fires_)fail_|=32;
            const uint32_t summary[]={0x434F4D41u+EPOK_LUA_MODE,240,fail_,240,hash_,shots_,guard_fires_,boss_fires_};
            for(unsigned n=0;n<8;++n)dungeon_combat_probe[n]=summary[n];
        }
    }
};
'''.replace("DRIVER_ID", DRIVER).replace("TRACE_WORDS", str(WORDS))


def freeze_legacy_prefix(base_out):
    """Preserve accepted observations as historical evidence before trace widening."""
    destination = base_out / "cpp/frozen-56word-prefix.json"
    proof = destination.with_name("frozen-56word-prefix-provenance.json")
    if destination.exists():
        assert len(json.loads(destination.read_text(encoding="utf-8"))) == STEPS * PREFIX_WORDS
        provenance = json.loads(proof.read_text(encoding="utf-8"))
        assert hashlib.sha256(destination.read_bytes()).hexdigest() == provenance["frozen_prefix_sha256"]
        return
    previous = base_out / "cpp/native_cpp-dungeon_combat_trace.json"
    if not previous.exists():
        return
    data = json.loads(previous.read_text(encoding="utf-8"))
    if len(data) != STEPS * PREFIX_WORDS:
        return
    shutil.copyfile(previous, destination)
    inventory = documents.loads((base_out / "cpp/source-inventory.json").read_text(encoding="utf-8"))
    observation = json.loads((base_out / "cpp/native_cpp-observation.json").read_text(encoding="utf-8"))
    proof.write_text(json.dumps({"scope": "Historical accepted C++ prefix; never acceptance of the new tail or current SDK",
        "frozen_prefix_sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
        "previous_common_contract_sha256": inventory["common_contract_sha256"],
        "editor_sha256": inventory["editor_sha256"], "played_exe_sha256": observation["played_exe_sha256"],
        "words": len(data)}, indent=2) + "\n", encoding="utf-8")


def host_tail_source():
    """Legacy shot equations over SDK integer math; no authored policy is included."""
    return r'''#include "actor_scene_fixture.hpp"
#include <cstdio>
using namespace epok;
int main(){
    std::printf("[");
    for(uint32_t frame=0;frame<240;++frame){
        const bool edited=frame>=120,fan=frame%2!=0;
        const uint32_t ordinal=frame%3,count=fan?3:1;
        const int32_t pellet=!fan?0:ordinal==0?-1:ordinal==2?1:0;
        const Fixed angle(pellet*(edited?30:12)*4096,Fixed::RAW),c=cos_degrees(angle),s=sin_degrees(angle);
        const Fixed x((int32_t(frame%9)-4)*512,Fixed::RAW),y((int32_t(frame%5)-2)*512,Fixed::RAW),z((int32_t(frame%7)-3)*512,Fixed::RAW);
        const Fixed speed=edited?(fan?Fixed(11.0):Fixed(9.0)):(fan?Fixed(8.0):Fixed(7.0));
        const Fixed vx=(x*c+z*s)*speed,vy=y*speed,vz=(z*c-x*s)*speed;
        const Fixed lifetime=edited?Fixed(3.0):Fixed(2.5);
        const Fixed size=edited?(fan?Fixed(.75):Fixed(.5)):(fan?Fixed(.32):Fixed(.24));
        const Fixed target_y=Fixed(8192+int32_t(frame%5)*256,Fixed::RAW)+(edited?Fixed(1.5):Fixed(.95));
        const uint32_t row[]={edited,fan,ordinal,count,uint32_t(pellet),uint32_t(vx.raw()),uint32_t(vy.raw()),uint32_t(vz.raw()),
            uint32_t(lifetime.raw()),uint32_t(size.raw()),edited?24u:12u,edited?32u:16u,uint32_t(target_y.raw()),1};
        static_assert(sizeof(row)/sizeof(row[0])==14);
        for(unsigned n=0;n<14;++n)std::printf("%s%u",(frame||n)?",":"",row[n]);
    }
    std::puts("]");
}
'''


def host_tail_oracle(base_out, allow_build=True):
    out = base_out / "enemy-shot-host-oracle"
    source_text = host_tail_source()
    # The oracle executes only Fixed arithmetic and these trigonometry kernels.
    # HUD, gameplay policies and the host runner's unrelated suites cannot
    # invalidate its numerical expectations.
    dependencies = [ROOT / "runtime" / name for name in ("epok.hpp", "world2d.hpp", "fixed_math.hpp")]
    dependencies += [ROOT / "third_party/nugget/psyqo/fixed-point.hh", ROOT / "tests/runtime/actor_scene_fixture.hpp"]
    inputs = {str(path.relative_to(ROOT)).replace("\\", "/"): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(dependencies)}
    signature = hashlib.sha256(json.dumps({"source": source_text, "inputs": inputs}, sort_keys=True).encode()).hexdigest()
    receipt_path, trace_path = out / "receipt.json", out / "tail.json"
    if receipt_path.exists() and trace_path.exists():
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt["signature"] == signature and receipt["tail_sha256"] == hashlib.sha256(trace_path.read_bytes()).hexdigest():
            tail = json.loads(trace_path.read_text(encoding="utf-8"))
            assert len(tail) == STEPS * TAIL_WORDS
            return tail
    assert allow_build, "Prepare the current host tail oracle before checking retained captures"
    out.mkdir(parents=True, exist_ok=True)
    cpp_path = out / "EnemyShotOracle.cpp"
    cpp_path.write_text(source_text, encoding="utf-8")
    runner = ROOT / "tests/runtime/verify_spatial.py"
    code = runner.read_text(encoding="utf-8")
    selection = next(marker for marker in ("arguments = _sys.argv[1:]", "if len(_sys.argv)") if marker in code)
    code = code[:code.index(selection)] + "\nSOURCES=[_Path(" + repr(str(cpp_path)) + ")]\ntest_arguments=[]\n" + code[code.index("env = os.environ.copy()"):]
    code = code.replace("    for source in SOURCES:", "    includes.append(ROOT / 'tests/runtime')\n    for source in SOURCES:")
    code = code.replace("        subprocess.run(command, cwd=temp, env=env, check=True)",
                        "        subprocess.run(command, cwd=temp, env=env, check=True, stdout=log, stderr=subprocess.STDOUT)")
    code, replacements = re.subn(r"        subprocess\.run\(\[str\(target\)(?:, \*test_arguments)?\], cwd=temp, check=True, timeout=30\)",
        "        subprocess.run([str(target)], cwd=temp, check=True, timeout=30, stdout=trace, stderr=log)", code)
    assert replacements == 1, "Host runner execution contract changed"
    namespace = {"__file__": str(runner), "__name__": "__main__"}
    with (out / "compile.log").open("w", encoding="utf-8") as log, trace_path.open("w", encoding="utf-8") as trace:
        namespace.update(log=log, trace=trace)
        exec(code, namespace)
    tail = json.loads(trace_path.read_text(encoding="utf-8"))
    assert len(tail) == STEPS * TAIL_WORDS and all(isinstance(value, int) and 0 <= value <= 0xffffffff for value in tail)
    receipt_path.write_text(json.dumps({"scope": "Independent legacy/configured project shot equations using shared integer SDK math; no policy fallback",
        "signature": signature, "source_sha256": hashlib.sha256(cpp_path.read_bytes()).hexdigest(), "inputs": inputs,
        "tail_sha256": hashlib.sha256(trace_path.read_bytes()).hexdigest(), "words": len(tail),
        "compiler": namespace["compiler"]}, indent=2) + "\n", encoding="utf-8")
    return tail


def host_feedback_source():
    return r'''#include "actor_scene_fixture.hpp"
#include <cstdio>
using namespace epok;
// Literal project functions frozen before feedback extraction. No combat
// policy header or implementation participates in this independent oracle.
struct LegacyRef {ActorData* ptr=nullptr;ActorData* get()const{return ptr;}};
struct LegacyGame {
    static constexpr uint32_t damage_flash_duration_us=220000;uint32_t damage_flash_us=0;LegacyRef damage_border{};
    struct Effect {LegacyRef data{};uint16_t age=0,kind=0;bool playing=false;};Effect effects[4]{};
    void update_damage_feedback(uint32_t);void effect(const Fixed*,uint16_t);void advance_effects();
};
void LegacyGame::update_damage_feedback(uint32_t microseconds){
    // Presentation follows rendered time, including death holds and menus.
    // Keep a visible frame at least one pixel wide until it is disabled: zero
    // borders intentionally mean a filled Image, not an empty one.
    damage_flash_us=microseconds>=damage_flash_us?0:damage_flash_us-microseconds;
    if(auto* d=damage_border.get()){
        d->image.enabled=damage_flash_us!=0;
        if(d->image.enabled){
            const uint16_t width=uint16_t((6*damage_flash_us+damage_flash_duration_us-1)/damage_flash_duration_us);
            for(auto& edge:d->image.borders)edge=width;
            d->image.color[0]=uint8_t(96+128*damage_flash_us/damage_flash_duration_us);
            d->image.color[1]=16;d->image.color[2]=20;
        }
    }
}

void LegacyGame::effect(const Fixed* point,uint16_t kind){
    for(auto& f:effects)if(!f.playing)if(auto* d=f.data.get()){
        f.playing=true;f.age=0;f.kind=kind;for(int k=0;k<3;++k)d->transform.position[k]=point[k];
        d->sprite.size[0]=d->sprite.size[1]=kind?Fixed(.48):Fixed(.28);d->sprite.enabled=true;
        const unsigned frame=kind?6:4;d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;return;
    }
}

void LegacyGame::advance_effects(){    for(auto& f:effects)if(f.playing){auto* d=f.data.get();if(!d){f.playing=false;continue;}++f.age;
        const unsigned frame=f.kind?6+f.age/2:(f.age<4?4:5),end=f.kind?10:6;
        if(f.age>=end){d->sprite.enabled=false;f.playing=false;}else{d->sprite.region[0]=(frame%8)*32;d->sprite.region[1]=(frame/8)*32;}}}
static void legacy_shot(ActorData* d){d->sprite.size[0]=d->sprite.size[1]=.18;d->sprite.color[0]=d->sprite.color[1]=d->sprite.color[2]=255;}
static uint32_t cap(uint32_t value,uint32_t low,uint32_t high){return value<low?low:value>high?high:value;}
int main(){
    std::printf("[");
    for(uint32_t frame_=0;frame_<240;++frame_){

        const uint32_t feedback_phase=frame_/80,feedback_slot=frame_%8;
        const bool feedback_enemy=frame_%2!=0,feedback_restart=frame_%5==0,feedback_advance=frame_%3!=0;
        const uint32_t feedback_layers=feedback_slot==7?0xffffffffu:feedback_slot;
        const uint32_t feedback_ages[]={0,1,2,3,4,5,6,7,8,9,10,11,65534,65535,0xffffffffu,0};
        const uint32_t feedback_remaining[]={0,1,220000,219999,10000000,0xffffffffu,0xffffffffu,220000};
        const uint32_t feedback_elapsed[]={0,1,1365,33333,219999,220000,0xffffffffu,0};
        const uint32_t feedback_durations[]={0,1,0xffffffffu,220000,10000000,0xffffffffu,0xffffffffu,0xffffffffu};

        const uint32_t trace_mask=feedback_phase==0?(feedback_enemy?5u:3u):feedback_phase==1?(feedback_enemy?17u:9u):(feedback_enemy?0xffffffffu:0x80000001u);
        const uint32_t impact=feedback_enemy?(frame_%5==0?2u:3u):(frame_%3!=0&&(feedback_layers&2)?1u:0u);
        Fixed shot_size;uint32_t shot_r,shot_g,shot_b;
        if(!feedback_phase){ActorData old;legacy_shot(&old);shot_size=old.sprite.size[0];shot_r=old.sprite.color[0];shot_g=old.sprite.color[1];shot_b=old.sprite.color[2];}
        else if(feedback_phase==1){shot_size=.4;shot_r=20;shot_g=40;shot_b=80;}
        else{shot_size=32.0;shot_r=255;shot_g=255;shot_b=0;}
        const uint32_t kind=frame_%2;uint32_t age=cap(feedback_ages[frame_%16],0,65535);if(feedback_advance&&age<65535)++age;
        Fixed fx_size;uint32_t fx_frame=0;bool fx_playing=false;
        if(!feedback_phase){
            ActorData sprite{};LegacyGame old;old.effects[0].data.ptr=&sprite;const Fixed point[3]{};old.effect(point,uint16_t(kind));
            fx_size=sprite.sprite.size[0];const auto end=kind?10u:6u;fx_playing=age<end;
            if(fx_playing){for(uint32_t n=0;n<age;++n)old.advance_effects();
                fx_frame=sprite.sprite.region[0]/32+(sprite.sprite.region[1]/32)*8;}
        }else if(feedback_phase==1){
            fx_size=kind?Fixed(.9):Fixed(.6);fx_playing=age<(kind?7u:4u);
            if(fx_playing)fx_frame=kind?12+age/3:(age<2?8:9);
        }else{
            fx_size=kind?Fixed(32.0):Fixed(0.0);fx_playing=frame_%7!=0&&age<65535;
            if(fx_playing)fx_frame=2047;
        }
        uint32_t left=0,pixels=0,red=0,green=0,blue=0;bool enabled=false;
        if(!feedback_phase){
            ActorData image{};LegacyGame old;old.damage_border.ptr=&image;
            old.damage_flash_us=feedback_restart?220000:cap(feedback_remaining[feedback_slot],0,220000);
            old.update_damage_feedback(feedback_elapsed[feedback_slot]);left=old.damage_flash_us;enabled=image.image.enabled;
            if(enabled){pixels=image.image.borders[0];red=image.image.color[0];green=image.image.color[1];blue=image.image.color[2];}
        }else{
            const uint64_t duration=feedback_phase==1?1000u:cap(feedback_durations[feedback_slot],0,10000000);
            const uint64_t before=feedback_restart?duration:cap(feedback_remaining[feedback_slot],0,uint32_t(duration));
            if(duration&&feedback_elapsed[feedback_slot]<before){
                enabled=true;left=uint32_t(before-feedback_elapsed[feedback_slot]);
                const uint64_t width=feedback_phase==1?9u:(feedback_slot%2?64u:1u);
                const uint64_t base=feedback_phase==1?20u:(feedback_slot%2?255u:0u),span=feedback_phase==1?40u:255u-base;
                pixels=uint32_t((width*left+duration-1)/duration);red=uint32_t(base+span*left/duration);
                green=feedback_phase==1?30u:255u;blue=feedback_phase==1?50u:255u;
            }
        }
        const uint32_t row[]={trace_mask,!feedback_enemy,impact,uint32_t(shot_size.raw()),shot_r,shot_g,shot_b,
            age,fx_frame,uint32_t(fx_size.raw()),fx_playing,left,enabled,pixels,red,green,blue,
            feedback_phase==0?160u:feedback_phase==1?60u:255u,feedback_phase==0?true:feedback_phase==1?false:feedback_enemy,
            feedback_phase==0?120u:feedback_phase==1?70u:60000u};
        static_assert(sizeof(row)/sizeof(row[0])==20);
        for(unsigned n=0;n<20;++n)std::printf("%s%u",(frame_||n)?",":"",row[n]);
    }
    std::puts("]");
}
'''



def freeze_accepted_prefix(base_out):
    """Keep the accepted 70-word observations historical when widening again."""
    destination = base_out / "cpp/frozen-70word-prefix.json"
    proof_path = destination.with_name("frozen-70word-prefix-provenance.json")
    if destination.exists():
        proof = json.loads(proof_path.read_text(encoding="utf-8"))
        assert hashlib.sha256(destination.read_bytes()).hexdigest() == proof["prefix_sha256"]
        assert len(json.loads(destination.read_text(encoding="utf-8"))) == STEPS * ACCEPTED_WORDS
        return
    current = base_out / "cpp/native_cpp-dungeon_combat_trace.json"
    if not current.exists(): return
    trace = json.loads(current.read_text(encoding="utf-8"))
    if len(trace) != STEPS * ACCEPTED_WORDS: return
    shutil.copyfile(current, destination)
    observation = json.loads((base_out / "cpp/native_cpp-observation.json").read_text(encoding="utf-8"))
    inventory = documents.loads((base_out / "cpp/source-inventory.json").read_text(encoding="utf-8"))
    proof_path.write_text(json.dumps({"scope":"Historical accepted 70-word prefix; not validation of the new feedback/SDK",
        "prefix_sha256":hashlib.sha256(destination.read_bytes()).hexdigest(),"words":len(trace),
        "played_exe_sha256":observation["played_exe_sha256"],"editor_sha256":inventory["editor_sha256"],
        "previous_common_contract_sha256":inventory["common_contract_sha256"]},indent=2)+"\n",encoding="utf-8")


def host_feedback_oracle(base_out, allow_build=True):
    out = base_out / "feedback-host-oracle"
    source_text = host_feedback_source()
    assert "DungeonPlayerCombat" not in source_text, "Feedback reference must not execute current policy"
    dependencies = [ROOT / "runtime" / name for name in ("epok.hpp", "world2d.hpp", "fixed_math.hpp")]
    dependencies += [ROOT / "third_party/nugget/psyqo/fixed-point.hh", ROOT / "tests/runtime/actor_scene_fixture.hpp"]
    inputs = {str(p.relative_to(ROOT)).replace("\\","/"):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(dependencies)}
    signature = hashlib.sha256(json.dumps({"source":source_text,"inputs":inputs},sort_keys=True).encode()).hexdigest()
    receipt_path, trace_path = out / "receipt.json", out / "tail.json"
    if receipt_path.exists() and trace_path.exists():
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt["signature"]==signature and receipt["tail_sha256"]==hashlib.sha256(trace_path.read_bytes()).hexdigest():
            tail = json.loads(trace_path.read_text(encoding="utf-8")); assert_feedback_coverage(tail); return tail
    assert allow_build, "Prepare current independent feedback oracle before checking retained captures"
    out.mkdir(parents=True,exist_ok=True);cpp_path = out / "FeedbackOracle.cpp"
    cpp_path.write_text(source_text,encoding="utf-8")
    runner = ROOT / "tests/runtime/verify_spatial.py";code = runner.read_text(encoding="utf-8")
    selection = next(marker for marker in ("arguments = _sys.argv[1:]", "if len(_sys.argv)") if marker in code)
    code = code[:code.index(selection)] + "\nSOURCES=[_Path(" + repr(str(cpp_path)) + ")]\ntest_arguments=[]\n" + code[code.index("env = os.environ.copy()") :]
    code = code.replace("    for source in SOURCES:","    includes.append(ROOT / 'tests/runtime')\n    for source in SOURCES:")
    code = code.replace("        subprocess.run(command, cwd=temp, env=env, check=True)",
        "        subprocess.run(command, cwd=temp, env=env, check=True, stdout=log, stderr=subprocess.STDOUT)")
    code, count = re.subn(r"        subprocess\.run\(\[str\(target\)(?:, \*test_arguments)?\], cwd=temp, check=True, timeout=30\)",
        "        subprocess.run([str(target)], cwd=temp, check=True, timeout=30, stdout=trace, stderr=log)", code)
    assert count==1
    namespace={"__file__":str(runner),"__name__":"__main__"}
    with (out / "compile.log").open("w",encoding="utf-8") as log, trace_path.open("w",encoding="utf-8") as trace:
        namespace.update(log=log,trace=trace);exec(code,namespace)
    tail = json.loads(trace_path.read_text(encoding="utf-8"));assert_feedback_coverage(tail)
    receipt_path.write_text(json.dumps({"scope":"Independent frozen legacy functions for defaults, wide bounded equations for edited/extreme cases; no combat policy header",
        "signature":signature,"inputs":inputs,"source_sha256":hashlib.sha256(cpp_path.read_bytes()).hexdigest(),
        "tail_sha256":hashlib.sha256(trace_path.read_bytes()).hexdigest(),"words":len(tail),"compiler":namespace["compiler"]},indent=2)+"\n",encoding="utf-8")
    return tail


def assert_feedback_coverage(tail):
    assert len(tail)==STEPS*FEEDBACK_WORDS
    assert all(isinstance(n,int) and 0<=n<=0xffffffff for n in tail)
    for phase in range(3):
        rows=[tail[n*FEEDBACK_WORDS:(n+1)*FEEDBACK_WORDS] for n in range(phase*80,(phase+1)*80)]
        assert {row[10] for row in rows}=={0,1}, (phase,"FX on/off coverage")
        assert {row[12] for row in rows}=={0,1}, (phase,"flash on/off coverage")
        assert {row[2] for row in rows}=={0,1,2,3}, (phase,"world/enemy/player/ignored impacts")
        assert all(row[13]<=64 and max(row[14:17])<=255 and row[17]<=255 and row[19]<=60000 for row in rows)
        assert all(row[13]>=1 for row in rows if row[12])
    assert any(tail[n*FEEDBACK_WORDS] > 0x7fffffff for n in range(160,240))


def trace_accepted(trace):
    return [value for row in range(STEPS) for value in trace[row*WORDS:row*WORDS+ACCEPTED_WORDS]]


def trace_feedback(trace):
    return [value for row in range(STEPS) for value in trace[row*WORDS+ACCEPTED_WORDS:row*WORDS+CONTROLLER_PREFIX_WORDS]]


def host_controller_source():
    return r'''#include "actor_scene_fixture.hpp"
#include "DungeonPlayerCombat.hpp"
#include <cstdio>
using namespace epok;
inline uint32_t dungeon_combat_trace[240][147]{};
int main(){
    DungeonPlayerCombat policy;auto* c=&policy;uint32_t hash_=2166136261u;
    DungeonControlCamera control_camera_{};
    Fixed control_facing_=0.0,control_pose_=0.0,control_intro_=0.0,control_spin_=0.0;
    uint32_t control_animation_=0,control_playing_=5,control_clock_=0;

    std::printf("[");for(uint32_t frame_=0;frame_<240;++frame_){

        // Controller results use distinct tuning/state and have no mechanism
        // side effects. The complete accepted 90-word source precedes them.
        const uint32_t ctrl_phase=frame_/80,ctrl_slot=frame_%8;
        if(frame_==80){
            c->control_movement={9.0,180.0};c->control_camera={25.0,90.0,45.0,-15.0,60.0,3.0,5.0,10.0,70.0,110.0,9.0,.5};
            c->control_view={7.0,1.2,1.5,.75,.8};c->control_aim={-30.0,40.0,180.0,.9,50.0,9,3};
            c->control_marker={.5,4.0,96.0,32.0,8.0,100.0};c->control_animation={3.0,7.0,.3,1.4,.15,4.0,3.2};
        }
        if(frame_==160){
            const Fixed high(0x7fffffff,Fixed::RAW);
            c->control_movement={high,high};c->control_camera={high,high,high,high,high,high,high,high,high,high,high,high};
            c->control_view={high,high,high,high,high};c->control_aim={Fixed(-0x7fffffff,Fixed::RAW),high,high,high,high,0xffffffffu,0xffffffffu};
            c->control_marker={high,high,0.0,high,high,high};c->control_animation={0.0,0.0,high,0.0,high,high,high};
        }
        const Fixed ctrl_dt=ctrl_phase<2?Fixed(137,Fixed::RAW):ctrl_slot==0?Fixed(-1.0):ctrl_slot==1?Fixed(0.0):ctrl_slot==2?Fixed(1.0):Fixed(0x7fffffff,Fixed::RAW);
        const auto ctrl_gate=c->control_gate(frame_%17!=0,frame_%19==0,frame_%31!=0,frame_%13!=0);
        const uint32_t ctrl_buttons=frame_%16;const bool ctrl_suppress=frame_%53==0;
        const Fixed ctrl_x=ctrl_phase<2?Fixed((int32_t(frame_%9)-4)*1024,Fixed::RAW):Fixed(0x7fffffff,Fixed::RAW);
        const Fixed ctrl_y=ctrl_phase<2?Fixed((int32_t(frame_%7)-3)*1200,Fixed::RAW):Fixed(-0x7fffffff,Fixed::RAW);
        const auto ctrl_intent=c->control_intent(ctrl_x,ctrl_y,ctrl_buttons&1,ctrl_buttons&2,ctrl_buttons&4,ctrl_buttons&8,ctrl_suppress);
        const auto ctrl_look=c->control_intent(Fixed((int32_t(frame_%7)-3)*600,Fixed::RAW),Fixed((int32_t(frame_%5)-2)*700,Fixed::RAW),false,false,false,false,false);
        const bool ctrl_free=frame_%23>=12,ctrl_target=!ctrl_free&&frame_%29<20;
        const auto ctrl_focus=c->control_focus_height(frame_%3!=0,ctrl_phase<2?Fixed(.8):Fixed(0x7fffffff,Fixed::RAW),ctrl_phase<2?Fixed(1.0):Fixed(0x7fffffff,Fixed::RAW));
        const GameplayVector3 ctrl_origin{1.0,2.0,3.0},ctrl_target_point{frame_%9?Fixed(3.0):Fixed(1.0),frame_%4?Fixed(0.0):Fixed(3.0),frame_%9?Fixed(8.0):Fixed(3.0)};
        const auto ctrl_angles=c->control_target_angles(ctrl_origin,ctrl_target_point,ctrl_focus);
        if(!frame_)control_camera_=c->control_camera_start(0.0);
        control_camera_=c->control_camera_step(control_camera_,{ctrl_look,ctrl_dt,control_facing_,ctrl_angles.yaw,ctrl_angles.pitch,ctrl_free,ctrl_target,ctrl_intent.strength.raw()!=0});
        const auto ctrl_move=c->control_movement_step(ctrl_intent,control_camera_,control_facing_,ctrl_angles.yaw,ctrl_target,ctrl_suppress,frame_%47==0,ctrl_dt);control_facing_=ctrl_move.facing;
        const auto ctrl_view=c->control_view_proposal(ctrl_origin,control_camera_,ctrl_phase==0?Fixed(2.15):ctrl_phase==1?Fixed(-1.0):Fixed(0x7fffffff,Fixed::RAW));
        control_pose_=c->control_aim_step(control_pose_,control_camera_.pitch,ctrl_angles.pitch,ctrl_dt,ctrl_free,ctrl_target);
        const auto ctrl_begin=c->control_marker_begin();if(frame_%40==0){control_intro_=Fixed(int32_t(ctrl_begin)*4096,Fixed::RAW);control_spin_=0.0;}
        const auto ctrl_marker=c->control_marker_step(control_intro_,control_spin_);control_intro_=ctrl_marker.intro;control_spin_=ctrl_marker.spin;
        const auto ctrl_rig=c->control_rig(frame_%3?13u:12u);
        const Fixed ctrl_speed=c->control_motion_speed({ctrl_move.x*Fixed(137,Fixed::RAW),0.0,ctrl_move.z*Fixed(137,Fixed::RAW)},ctrl_dt);
        const auto ctrl_animation=c->control_animation_select(control_animation_,control_playing_,frame_%61<55,frame_%31<15?Fixed(1.0):Fixed(-1.0),ctrl_speed,frame_%17==0,frame_%61==3);
        control_animation_=ctrl_animation.state;if(ctrl_animation.switch_clip&&frame_%7!=0){control_playing_=ctrl_animation.state;control_clock_=0;}
        const auto ctrl_clock=c->control_animation_clock(ctrl_phase==2&&frame_%17==0?0xffffffffu:control_clock_,ctrl_phase<2?23u:ctrl_slot==0?0u:ctrl_slot==1?1u:0xffffffffu,ctrl_animation.rate,ctrl_dt,frame_%11!=0);control_clock_=ctrl_clock.phase;
        const auto ctrl_direction=c->control_direction(control_facing_,control_pose_),ctrl_ray=c->control_aim_ray(control_camera_.yaw,control_camera_.pitch);
        const auto ctrl_ray_mask=c->control_aim_mask(false),ctrl_muzzle_mask=c->control_aim_mask(true),ctrl_shot_aim=c->control_shot_aim(ctrl_target,control_camera_.mode);
        const GameplayVector3 ctrl_deltas[]={{0.0,0.0,0.0},{1.0,2.0,3.0},{-3.0,.5,8.0},{40.0,0.0,0.0},{63.0,.5,0.0},{64.0,0.0,0.0},{-65.0,0.0,0.0},{Fixed(0x7fffffff,Fixed::RAW),0.0,0.0}};
        const auto ctrl_normalized=c->control_direction_toward(ctrl_deltas[ctrl_slot]);
        const uint32_t controller_tail[]={ctrl_gate,uint32_t(ctrl_intent.x.raw()),uint32_t(ctrl_intent.y.raw()),uint32_t(ctrl_intent.strength.raw()),
            uint32_t(control_camera_.yaw.raw()),uint32_t(control_camera_.pitch.raw()),uint32_t(control_camera_.goal_yaw.raw()),uint32_t(control_camera_.goal_pitch.raw()),uint32_t(control_camera_.hold.raw()),uint32_t(control_camera_.blend.raw()),control_camera_.mode,
            uint32_t(ctrl_move.x.raw()),uint32_t(ctrl_move.z.raw()),uint32_t(ctrl_move.facing.raw()),
            uint32_t(ctrl_view.origin.x.raw()),uint32_t(ctrl_view.origin.y.raw()),uint32_t(ctrl_view.origin.z.raw()),uint32_t(ctrl_view.offset.x.raw()),uint32_t(ctrl_view.offset.y.raw()),uint32_t(ctrl_view.offset.z.raw()),uint32_t(ctrl_view.distance.raw()),uint32_t(ctrl_view.yaw.raw()),uint32_t(ctrl_view.pitch.raw()),
            uint32_t(ctrl_focus.raw()),uint32_t(ctrl_angles.yaw.raw()),uint32_t(ctrl_angles.pitch.raw()),uint32_t(control_pose_.raw()),ctrl_begin,
            uint32_t(ctrl_marker.intro.raw()),uint32_t(ctrl_marker.spin.raw()),uint32_t(ctrl_marker.size.raw()),ctrl_marker.tint,ctrl_marker.average,
            uint32_t(ctrl_rig.bone),uint32_t(ctrl_rig.stop_a),uint32_t(ctrl_rig.stop_b),ctrl_animation.state,ctrl_animation.loop,ctrl_animation.locomotion,ctrl_animation.switch_clip,uint32_t(ctrl_animation.rate.raw()),ctrl_clock.phase,ctrl_clock.ticks,
            uint32_t(ctrl_direction.x.raw()),uint32_t(ctrl_direction.y.raw()),uint32_t(ctrl_direction.z.raw()),uint32_t(ctrl_ray.x.raw()),uint32_t(ctrl_ray.y.raw()),uint32_t(ctrl_ray.z.raw()),ctrl_ray_mask,ctrl_muzzle_mask,ctrl_shot_aim,
            uint32_t(ctrl_normalized.value.x.raw()),uint32_t(ctrl_normalized.value.y.raw()),uint32_t(ctrl_normalized.value.z.raw()),ctrl_normalized.valid,uint32_t(ctrl_speed.raw())};
        static_assert(sizeof(controller_tail)/sizeof(controller_tail[0])==57);
        for(unsigned n=0;n<57;++n){dungeon_combat_trace[frame_][90+n]=controller_tail[n];hash_=(hash_^controller_tail[n])*16777619u;}

        for(uint32_t n=0;n<57;++n)std::printf("%s%u",(frame_||n)?",":"",controller_tail[n]);
    }std::puts("]");
}
'''



def host_controller_oracle(base_out, allow_build=True):
    """Current C++ sequence reference backed by prior independent rule suites."""
    out=base_out / "controller-host-oracle";source_text=host_controller_source()
    dependencies=[ROOT/"runtime"/name for name in ("epok.hpp","gameplay_api.hpp","world2d.hpp","fixed_math.hpp","blueprint_runtime.hpp")]
    dependencies += [NATIVE/name for name in ("DungeonPlayerCombat.hpp","DungeonPlayerCombatBase.hpp","DungeonRulesBinding.hpp")]
    dependencies += [ROOT/"third_party/nugget/psyqo/fixed-point.hh",ROOT/"tests/runtime/actor_scene_fixture.hpp"]
    inputs={str(p.relative_to(ROOT)).replace("\\","/"):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(dependencies)}
    signature=hashlib.sha256(json.dumps({"source":source_text,"inputs":inputs},sort_keys=True).encode()).hexdigest()
    receipt_path,trace_path=out/"receipt.json",out/"tail.json"
    if receipt_path.exists() and trace_path.exists():
        receipt=json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt["signature"]==signature and receipt["tail_sha256"]==hashlib.sha256(trace_path.read_bytes()).hexdigest():
            tail=json.loads(trace_path.read_text(encoding="utf-8"));assert_controller_coverage(tail);return tail
    assert allow_build,"Prepare current controller host sequence reference"
    out.mkdir(parents=True,exist_ok=True);cpp_path=out/"ControllerOracle.cpp";cpp_path.write_text(source_text,encoding="utf-8")
    runner=ROOT/"tests/runtime/verify_spatial.py";code=runner.read_text(encoding="utf-8")
    selection=next(marker for marker in ("arguments = _sys.argv[1:]","if len(_sys.argv)") if marker in code)
    code=code[:code.index(selection)]+"\nSOURCES=[_Path("+repr(str(cpp_path))+")]\ntest_arguments=[]\n"+code[code.index("env = os.environ.copy()") :]
    code=code.replace("    for source in SOURCES:","    includes.extend([ROOT/'tests/runtime',NATIVE])\n    for source in SOURCES:")
    code=code.replace("        subprocess.run(command, cwd=temp, env=env, check=True)","        subprocess.run(command,cwd=temp,env=env,check=True,stdout=log,stderr=subprocess.STDOUT)")
    code,count=re.subn(r"        subprocess\.run\(\[str\(target\)(?:, \*test_arguments)?\], cwd=temp, check=True, timeout=30\)",
        "        subprocess.run([str(target)],cwd=temp,check=True,timeout=30,stdout=trace,stderr=log)",code);assert count==1
    namespace={"__file__":str(runner),"__name__":"__main__","NATIVE":NATIVE}
    with (out/"compile.log").open("w",encoding="utf-8") as log,trace_path.open("w",encoding="utf-8") as trace:
        namespace.update(log=log,trace=trace);exec(code,namespace)
    tail=json.loads(trace_path.read_text(encoding="utf-8"));assert_controller_coverage(tail)
    receipt_path.write_text(json.dumps({"scope":"Current selected C++ controller scenario reference; prior 104173 independent host +104173 graph rule checks reused by source identity, not claimed as an independent new scenario oracle",
        "signature":signature,"inputs":inputs,"source_sha256":hashlib.sha256(cpp_path.read_bytes()).hexdigest(),"tail_sha256":hashlib.sha256(trace_path.read_bytes()).hexdigest(),"words":len(tail),"compiler":namespace["compiler"]},indent=2)+"\n",encoding="utf-8")
    return tail


def assert_controller_coverage(tail):
    assert len(tail)==STEPS*CONTROLLER_WORDS
    assert all(isinstance(n,int) and 0<=n<=0xffffffff for n in tail)
    for phase in range(3):
        rows=[tail[n*CONTROLLER_WORDS:(n+1)*CONTROLLER_WORDS] for n in range(phase*80,(phase+1)*80)]
        assert {row[10] for row in rows}=={0,1,2},(phase,"normal/lock/free camera")
        assert {row[55] for row in rows}=={0,1},(phase,"normalization valid/rejected")
        assert {row[51] for row in rows}=={0,1,2},(phase,"facing/target/reticle aim")
        assert all(row[31]<=255 for row in rows)
    assert any(tail[n*CONTROLLER_WORDS+41]==0xffffffff for n in range(160,240)),"No saturating invalid-model clock"


def trace_controller(trace):
    return [value for row in range(STEPS) for value in trace[row*WORDS+CONTROLLER_PREFIX_WORDS:row*WORDS+WORLD_PREFIX_WORDS]]


def host_world_source():
    return r'''#include "actor_scene_fixture.hpp"
#include "DungeonEncounter.hpp"
#include "DungeonRunFlow.hpp"
#include <cstdio>
using namespace epok;
inline uint32_t dungeon_combat_trace[240][221]{};
static Fixed legacy_clamp(Fixed value){return value<Fixed(0.0)?Fixed(0.0):value>Fixed(1.0)?Fixed(1.0):value;}
int main(){DungeonEncounter encounter;auto* e=&encounter;DungeonRunFlow flow_policy;uint32_t hash_=2166136261u,checks=0;
    HealthState world_guard_{},world_boss_{},world_player_{100,25};
    Fixed world_door_=0.0;
    uint32_t world_kills_=0,world_resets_=0,world_heals_=0;
    bool world_key_=false,world_pack_=false,world_guard_defeated_=false;

    std::printf("[");for(uint32_t frame_=0;frame_<240;++frame_){

        // Same scenario, selected-language encounter/world and RunFlow rules.
        // Nothing above the original147 observations is changed or replayed.
        auto* world_flow=&flow_policy;
        const uint32_t world_phase=frame_/80,world_sub=frame_%40,world_slot=frame_%8;
        if(frame_==80){
            e->world_damage={3,5,true,false,true,false,false};e->guard_colors={30,60,90,150,170,190,200,180,160};
            e->boss_colors={100,120,140,160,180,200,210,220,230,240,250,255};e->enemy_shot_color={25,50,75};
            e->world_doors={7.0,3.0,2.0,.5,3.0,40,80,120};e->world_pickups={2.0,4.0,80,90.0,.25,-9.0,9.0,30.0,60.0};
            e->world_animation={12,18,7,false};e->world_motion={-.1,.05};
        }
        if(frame_==160){
            const Fixed high(0x7fffffff,Fixed::RAW),low(-0x7fffffff,Fixed::RAW);
            e->world_damage={0xffffffffu,0xffffffffu,true,true,true,true,true};e->guard_colors={0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu};
            e->boss_colors={0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu};
            e->enemy_shot_color={0xffffffffu,0xffffffffu,0xffffffffu};e->world_doors={high,high,high,high,high,0xffffffffu,0xffffffffu,0xffffffffu};
            e->world_pickups={high,high,0xffffffffu,high,high,low,high,0.0,high};e->world_animation={-1,0x7fffffff,0x7fffffff,true};e->world_motion={low,0.0};
        }
        if(!world_sub){
            world_guard_=e->starting_health(false);world_boss_=e->starting_health(true);world_player_={100,25};world_door_=0.0;
            world_key_=world_pack_=world_guard_defeated_=false;world_flow->start();for(uint32_t n=0;n<5;++n)world_flow->advance(Fixed(1.0),true);
        }
        if(world_sub==20)world_flow->try_enter_boss(world_key_,Fixed(54.0));
        const Fixed world_dt=world_phase<2?Fixed(137,Fixed::RAW):world_slot?Fixed(0x7fffffff,Fixed::RAW):Fixed(0.0);
        const uint32_t world_state_before=world_flow->status.state,world_index=world_sub>=20?3u:frame_%4;
        const bool world_locked=frame_%3!=0,world_near_open=world_sub<30,world_near_safety=world_sub>=22&&world_sub<=26;
        const bool key_before=world_key_,guard_defeated_before=world_guard_defeated_;
        const Fixed world_door_before=world_door_;
        const auto world_open=e->world_door_open(world_index,world_locked,world_key_,world_state_before,world_door_,world_near_open,world_near_safety);
        const auto world_radius=e->world_door_radius(false),world_safety=e->world_door_radius(true);
        const auto world_door=e->world_door_step(world_index,world_locked,world_key_,world_state_before,world_door_,world_near_open,world_near_safety,world_dt);world_door_=world_door.progress;
        const bool world_is_boss=world_sub>=20;const auto& world_before=world_is_boss?world_boss_:world_guard_;
        const uint32_t world_bases[]={0,1,20,20,80,0x7fffffffu,0xffffffffu,20},world_bonuses[]={0,8,8,0,8,0,0xffffffffu,0};
        const uint32_t world_base=world_is_boss&&world_sub<24?0u:world_bases[world_slot],world_bonus=world_is_boss&&world_sub<24?0u:world_bonuses[world_slot];
        const auto world_total=e->world_damage_total(world_base,world_bonus);
        const auto world_eligible=e->world_damage_eligible(world_is_boss,world_flow->status.alive,world_flow->status.state==1,world_before.current,true,0.0,world_guard_defeated_);
        const uint32_t world_health_before=world_before.current;HealthChange world_change{};
        if(world_eligible){auto& health=world_is_boss?world_boss_:world_guard_;world_change=HealthLibrary::damage(health,int32_t(world_total));health=world_change.state;}
        const auto world_hit=e->world_hit_result(world_is_boss,world_change.depleted,(world_is_boss?world_boss_:world_guard_).current);
        const auto world_reward=e->world_reward_count(world_kills_,world_eligible?world_hit.reward:0);
        if(world_eligible){world_kills_=world_reward;if(world_hit.defeated)world_guard_defeated_=true;if(world_hit.victory)world_flow->win();if(world_hit.reset_projectiles)++world_resets_;}
        HealthChange world_player_change{};
        if(world_sub==17&&(frame_/40)%2&&world_flow->status.alive){world_player_change=HealthLibrary::damage(world_player_,0x7fffffff);world_player_=world_player_change.state;}
        const auto world_death=e->world_player_death(world_player_change.depleted);if(world_death.defeat)world_flow->die();if(world_death.reset_projectiles)++world_resets_;
        const auto world_hide=e->world_hide_boss(world_flow->status.state,world_sub%5!=0);
        const Fixed world_flash=world_sub%9==0?Fixed(.1):Fixed(0.0),world_windup=world_sub%7==0?Fixed(.1):Fixed(0.0);
        const auto world_color=e->world_enemy_color(world_is_boss,world_flash,world_windup,frame_%2!=0),world_shot=e->world_shot_color();
        const auto world_death_clip=e->world_death_animation(),world_clip=e->world_enemy_animation(world_is_boss,world_sub%7,int32_t(frame_%4)-1);
        const GameplayVector3 world_position{frame_%4==0?Fixed(-10.0):frame_%4==1?Fixed(10.0):Fixed(0.0),frame_%7==3?Fixed(3.0):Fixed(0.0),40.0};
        const uint32_t world_kind=frame_%4;const bool world_can_collect=world_flow->status.alive&&(world_kind==0?!world_key_:!world_pack_&&world_player_.current<world_player_.maximum);
        const bool world_near_pickup=frame_%5!=4;
        const auto world_visible=e->world_pickup_visible(world_kind,world_position),world_collect=e->world_pickup_eligible(world_can_collect,world_near_pickup,world_position.y);
        const auto world_pick_radius=e->world_pickup_radius();const auto world_heal=e->world_pickup_heal();
        const Fixed world_elapsed(int32_t(frame_)*137,Fixed::RAW);const auto world_bob=e->world_pickup_bob(world_phase<2?world_elapsed:Fixed(0x7fffffff,Fixed::RAW));
        if(world_collect){if(!world_kind)world_key_=true;else{const auto healed=HealthLibrary::heal(world_player_,int32_t(world_heal));world_player_=healed.state;if(healed.applied){world_pack_=true;world_heals_+=healed.applied;}}}
        const GameplayVector3 world_deltas[]={{0.0,0.0,0.0},{1.0,2.0,3.0},{-3.0,.5,8.0},{40.0,0.0,0.0},{63.0,.5,0.0},{64.0,0.0,0.0},{-65.0,0.0,0.0},{Fixed(0x7fffffff,Fixed::RAW),0.0,0.0}};
        const auto world_aim=e->world_aim_direction(world_deltas[world_slot]);
        const GameplayVector3 world_origin{1.0,0.0,2.0},world_target{4.0,1.0,9.0},world_home{2.0,0.0,3.0};
        const auto world_patrol=e->world_patrol_goal(world_home,frame_%2!=0);
        const auto world_motion=e->world_enemy_motion(world_is_boss,world_sub%7,frame_%2!=0,frame_%3!=0,frame_%4!=0,world_origin,world_target,world_home,world_dt,Fixed(.85));
        const auto world_due=e->world_boss_due(Fixed(int32_t(world_sub)*68,Fixed::RAW));
        const uint32_t world_tail[]={world_eligible,world_total,world_reward,
            world_hit.defeated,world_hit.disable_collider,world_hit.death_clip,world_hit.victory,world_hit.reset_projectiles,world_hit.explosion,world_hit.hud_dirty,world_hit.reward,world_hit.sound,
            world_death.defeat,world_death.clear_aim,world_death.reset_projectiles,world_hide,
            world_color.red,world_color.green,world_color.blue,world_shot.red,world_shot.green,world_shot.blue,
            uint32_t(world_death_clip.clip),world_death_clip.play,world_death_clip.loop,world_death_clip.charge_cue,
            uint32_t(world_clip.clip),world_clip.play,world_clip.loop,world_clip.charge_cue,world_open,uint32_t(world_radius.raw()),uint32_t(world_safety.raw()),
            uint32_t(world_door.progress.raw()),world_door.opening,world_door.moved,world_door.opened,world_door.locked_tint,uint32_t(world_door.offset.raw()),world_door.color.red,world_door.color.green,world_door.color.blue,
            world_visible,world_collect,uint32_t(world_pick_radius.raw()),world_heal,uint32_t(world_bob.raw()),
            uint32_t(world_aim.value.x.raw()),uint32_t(world_aim.value.y.raw()),uint32_t(world_aim.value.z.raw()),world_aim.valid,
            uint32_t(world_patrol.x.raw()),uint32_t(world_patrol.y.raw()),uint32_t(world_patrol.z.raw()),
            uint32_t(world_motion.delta.x.raw()),uint32_t(world_motion.delta.y.raw()),uint32_t(world_motion.delta.z.raw()),uint32_t(world_motion.yaw.raw()),uint32_t(world_motion.patrol_yaw.raw()),world_motion.move,world_motion.turn,world_motion.patrol_turn,world_motion.valid,world_due,
            world_guard_.current,world_boss_.current,world_player_.current,world_kills_,uint32_t(world_door_.raw()),world_key_,world_pack_,world_flow->status.state,world_resets_,world_heals_};
        static_assert(sizeof(world_tail)/sizeof(world_tail[0])==74);
        for(unsigned n=0;n<74;++n){dungeon_combat_trace[frame_][147+n]=world_tail[n];hash_=(hash_^world_tail[n])*16777619u;}


        if(world_phase==0){
            const auto base=world_base>0x7fffffffu?0x7fffffffu:world_base;const uint64_t total=uint64_t(base)+world_bonus;
            assert(world_total==(total>0x7fffffffu?0x7fffffffu:uint32_t(total)));++checks;
            const bool eligible=world_state_before<2&&(world_is_boss?(world_state_before==1&&world_health_before>0):!guard_defeated_before);
            assert(world_eligible==eligible);++checks;
            bool open=(!world_locked||key_before||world_state_before==2)&&world_near_open;
            if(world_index==3&&world_state_before==1)open=false;if(!open&&world_door_before>Fixed(0.0)&&world_near_safety)open=true;
            const auto progress=legacy_clamp(world_door_before+(open?world_dt:-world_dt)*Fixed(1.4));
            assert(world_open==open&&world_door.progress==progress&&world_door.opening==open);++checks;
            const auto offset=Fixed(.75)+progress*progress*(Fixed(3.0)-Fixed(2.0)*progress)*Fixed(1.7);assert(world_door.offset==offset);++checks;
            const bool visible=world_kind==0||world_kind==2?world_position.x<Fixed(-8.0):world_kind==1?world_position.x>Fixed(8.0):(world_position.z>Fixed(32.0)&&world_position.z<Fixed(55.0));
            assert(world_visible==visible&&world_collect==(world_can_collect&&world_near_pickup&&world_position.y<Fixed(2.2)));checks+=2;
            assert(world_heal==35&&world_pick_radius==Fixed(1.0)&&world_bob==sin_degrees(world_elapsed*Fixed(120.0))*Fixed(.12));++checks;
            assert(world_death.defeat==world_player_change.depleted&&world_death.reset_projectiles==world_player_change.depleted);++checks;
            assert(world_hit.victory==(world_is_boss&&world_change.depleted)&&world_hit.reward==(!world_is_boss&&world_change.depleted?1u:0u));++checks;
        }

        for(uint32_t n=0;n<74;++n)std::printf("%s%u",(frame_||n)?",":"",world_tail[n]);
    }std::puts("]");std::fprintf(stderr,"PASS %u independent default-case legacy checks; current C++ scenario reference17760 words\n",checks);
}
'''



def host_world_oracle(base_out,allow_build=True):
    out=base_out / "world-host-oracle";source_text=host_world_source()
    dependencies=[ROOT/"runtime"/name for name in ("epok.hpp","gameplay_api.hpp","world2d.hpp","fixed_math.hpp","blueprint_runtime.hpp")]
    dependencies += [NATIVE/name for name in ("DungeonEncounter.hpp","DungeonEncounterBase.hpp","DungeonRunFlow.hpp","DungeonRunFlowBase.hpp","DungeonRulesBinding.hpp")]
    dependencies += [ROOT/"third_party/nugget/psyqo/fixed-point.hh",ROOT/"tests/runtime/actor_scene_fixture.hpp"]
    inputs={str(p.relative_to(ROOT)).replace("\\","/"):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(dependencies)}
    signature=hashlib.sha256(json.dumps({"source":source_text,"inputs":inputs},sort_keys=True).encode()).hexdigest()
    receipt_path,trace_path=out/"receipt.json",out/"tail.json"
    if receipt_path.exists() and trace_path.exists():
        receipt=json.loads(receipt_path.read_text(encoding="utf-8"))
        if receipt["signature"]==signature and receipt["tail_sha256"]==hashlib.sha256(trace_path.read_bytes()).hexdigest():
            tail=json.loads(trace_path.read_text(encoding="utf-8"));assert_world_coverage(tail);return tail
    assert allow_build,"Prepare current A/B world scenario reference"
    out.mkdir(parents=True,exist_ok=True);cpp_path=out/"WorldOracle.cpp";cpp_path.write_text(source_text,encoding="utf-8")
    runner=ROOT/"tests/runtime/verify_spatial.py";code=runner.read_text(encoding="utf-8")
    selection=next(marker for marker in ("arguments = _sys.argv[1:]","if len(_sys.argv)") if marker in code)
    code=code[:code.index(selection)]+"\nSOURCES=[_Path("+repr(str(cpp_path))+")]\ntest_arguments=[]\n"+code[code.index("env = os.environ.copy()") :]
    code=code.replace("    for source in SOURCES:","    includes.extend([ROOT/'tests/runtime',NATIVE])\n    for source in SOURCES:")
    code=code.replace("        subprocess.run(command, cwd=temp, env=env, check=True)","        subprocess.run(command,cwd=temp,env=env,check=True,stdout=log,stderr=subprocess.STDOUT)")
    code,count=re.subn(r"        subprocess\.run\(\[str\(target\)(?:, \*test_arguments)?\], cwd=temp, check=True, timeout=30\)","        subprocess.run([str(target)],cwd=temp,check=True,timeout=30,stdout=trace,stderr=log)",code);assert count==1
    namespace={"__file__":str(runner),"__name__":"__main__","NATIVE":NATIVE}
    with (out/"compile.log").open("w",encoding="utf-8") as log,trace_path.open("w",encoding="utf-8") as trace:
        namespace.update(log=log,trace=trace);exec(code,namespace)
    tail=json.loads(trace_path.read_text(encoding="utf-8"));assert_world_coverage(tail)
    log_text=(out/"compile.log").read_text(encoding="utf-8");checks=int(re.search(r"PASS (\d+) independent default-case legacy checks",log_text)[1])
    receipt_path.write_text(json.dumps({"scope":"Current C++ scenario reference plus independent legacy conditions/equations on80 default frames; broad105257 host+105257 graph suites retained separately by identity",
        "signature":signature,"inputs":inputs,"source_sha256":hashlib.sha256(cpp_path.read_bytes()).hexdigest(),"tail_sha256":hashlib.sha256(trace_path.read_bytes()).hexdigest(),"words":len(tail),"independent_legacy_checks":checks,"compiler":namespace["compiler"]},indent=2)+"\n",encoding="utf-8")
    return tail


def assert_world_coverage(tail):
    assert len(tail)==STEPS*WORLD_WORDS
    assert all(isinstance(n,int) and 0<=n<=0xffffffff for n in tail)
    for phase in range(3):
        rows=[tail[n*WORLD_WORDS:(n+1)*WORLD_WORDS] for n in range(phase*80,(phase+1)*80)]
        assert {row[50] for row in rows}=={0,1},(phase,"direction valid/rejected")
        assert {row[71] for row in rows}>={0,1,2,3},(phase,"RunFlow explore/boss/victory/defeat")
        assert any(row[4] for row in rows) and any(row[6] for row in rows),(phase,"death and victory consequences")
        assert any(row[69] for row in rows) and any(row[70] for row in rows),(phase,"key and heal pickup")
        assert all(max(row[16:22])<=255 for row in rows)
    assert tail[239*WORLD_WORDS+67]==0x7fffffff,"Reward count did not saturate"


def trace_world(trace):
    return [value for row in range(STEPS) for value in trace[row*WORDS+WORLD_PREFIX_WORDS:(row+1)*WORDS]]


def trace_prefix(trace):
    return [value for row in range(STEPS) for value in trace[row * WORDS:row * WORDS + PREFIX_WORDS]]


def trace_tail(trace):
    return [value for row in range(STEPS) for value in trace[row * WORDS + PREFIX_WORDS:row * WORDS + ACCEPTED_WORDS]]


def validate_captures(base_out, out, flavor, selected, fingerprint):
    reference_path = base_out / "cpp/native_cpp-dungeon_combat_trace.json"
    baseline_inventory = documents.loads((base_out / "cpp/source-inventory.json").read_text(encoding="utf-8"))
    assert baseline_contract_matches(base_out, baseline_inventory["common_contract_sha256"], fingerprint), "C++ baseline has stale common source/compiler dependencies"
    expected_tail = host_tail_oracle(base_out, allow_build=False)
    expected_feedback = host_feedback_oracle(base_out, allow_build=False)
    expected_controller=host_controller_oracle(base_out,allow_build=False)
    expected_world=host_world_oracle(base_out,allow_build=False)
    frozen70_path=base_out / 'cpp/frozen-70word-prefix.json'
    frozen70=json.loads(frozen70_path.read_text(encoding='utf-8')) if frozen70_path.exists() else None
    frozen_path = base_out / "cpp/frozen-56word-prefix.json"
    frozen = json.loads(frozen_path.read_text(encoding="utf-8")) if frozen_path.exists() else None
    for mode in selected:
        actual_path = out / (mode + "-dungeon_combat_trace.json")
        actual = json.loads(actual_path.read_text(encoding="utf-8"))
        assert len(actual) == STEPS * WORDS
        observation = json.loads((out / (mode + "-observation.json")).read_text(encoding="utf-8"))
        expected = MAGIC + {"native_cpp": 0, "vm_bytecode": 1, "vm_source": 2}[mode]
        values = observation["values"]
        assert values[:4] == [expected, STEPS, 0, STEPS] and all(values[n] > 0 for n in (5, 6, 7)), values
        digest = 2166136261
        for value in actual:
            digest = ((digest ^ value) * 16777619) & 0xffffffff
        assert values[4] == digest, (flavor, mode, "summary/trace checksum mismatch", values[4], digest)
        actual_tail = trace_tail(actual)
        if actual_tail != expected_tail:
            index = next(n for n, (a, b) in enumerate(zip(actual_tail, expected_tail)) if a != b)
            raise AssertionError((flavor, mode, "enemy shot oracle", index // TAIL_WORDS, index % TAIL_WORDS,
                                  actual_tail[index], expected_tail[index]))
        if frozen is not None:
            prefix = trace_prefix(actual)
            if prefix != frozen:
                index = next(n for n, (a, b) in enumerate(zip(prefix, frozen)) if a != b)
                raise AssertionError((flavor, mode, "historical prefix regression", index // PREFIX_WORDS,
                                      index % PREFIX_WORDS, prefix[index], frozen[index]))
        feedback = trace_feedback(actual)
        assert_feedback_coverage(feedback)
        if feedback != expected_feedback:
            index=next(n for n,(a,b) in enumerate(zip(feedback,expected_feedback)) if a!=b)
            raise AssertionError((flavor,mode,"independent feedback oracle",index//FEEDBACK_WORDS,index%FEEDBACK_WORDS,feedback[index],expected_feedback[index]))
        if frozen70 is not None:
            accepted=trace_accepted(actual)
            if accepted!=frozen70:
                index=next(n for n,(a,b) in enumerate(zip(accepted,frozen70)) if a!=b)
                raise AssertionError((flavor,mode,"accepted 70-word prefix regression",index//ACCEPTED_WORDS,index%ACCEPTED_WORDS,accepted[index],frozen70[index]))
        controller=trace_controller(actual);assert_controller_coverage(controller)
        if controller!=expected_controller:
            index=next(n for n,(a,b) in enumerate(zip(controller,expected_controller)) if a!=b)
            raise AssertionError((flavor,mode,"controller host reference",index//CONTROLLER_WORDS,index%CONTROLLER_WORDS,controller[index],expected_controller[index]))
        world=trace_world(actual);assert_world_coverage(world)
        if world!=expected_world:
            index=next(n for n,(a,b) in enumerate(zip(world,expected_world)) if a!=b)
            raise AssertionError((flavor,mode,"A/B world reference",index//WORLD_WORDS,index%WORLD_WORDS,world[index],expected_world[index]))
        states = {actual[row * WORDS + column] for row in range(STEPS) for column in (19, 20)}
        assert {0, 1, 2, 3} <= states, (flavor, mode, "missing empty/miss/hit/expiry coverage", states)
        assert any(actual[row * WORDS + 23] or actual[row * WORDS + 24] for row in range(STEPS)), "No collision target observed"
        assert all(actual[row * WORDS + 54] == 7 for row in range(20, STEPS, 40)), "Reset did not invalidate the old handle"
        assert any(actual[row * WORDS + 52] for row in range(STEPS)), "No accepted cross-subsystem damage"
        assert reference_path.exists(), "Run the C++ baseline first"
        reference = json.loads(reference_path.read_text(encoding="utf-8"))
        assert len(reference) == STEPS * WORDS, "Prepare a current C++ capture with the enemy-shot tail first"
        if actual != reference:
            index = next(n for n, (a, b) in enumerate(zip(actual, reference)) if a != b)
            raise AssertionError((flavor, mode, index // WORDS, index % WORDS, actual[index], reference[index]))
    (out / "conformance.json").write_text(json.dumps({"provider": flavor, "modes": selected, "words_per_case": STEPS * WORDS,
        "comparison": "Actual current C++ PSX trace, independent host enemy-shot tail, and historical prefix where available",
        "independent_enemy_shot_words": len(expected_tail), "historical_prefix_words_compared": len(frozen) if frozen is not None else 0,
        "historical_70word_prefix_words":len(frozen70) if frozen70 is not None else 0,"independent_feedback_words":len(expected_feedback),
        "controller_reference_words":len(expected_controller),"controller_reference_independent":False,"world_reference_words":len(expected_world),
        "host_feedback_sha256":hashlib.sha256((base_out / "feedback-host-oracle/tail.json").read_bytes()).hexdigest(),
        "host_tail_sha256": hashlib.sha256((base_out / "enemy-shot-host-oracle/tail.json").read_bytes()).hexdigest(),
        "baseline_sha256": hashlib.sha256(reference_path.read_bytes()).hexdigest(), "native_policy_fallback": False}, indent=2) + "\n", encoding="utf-8")
    print("PASS combined combat", flavor, STEPS * WORDS, "conformance words", flush=True)


def verify(out, flavor, modes, prepare_only, standalone_only, check_only=False):
    base_out, out = out, out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    selected = modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",))
    if check_only:
        inventory = documents.loads((out / "source-inventory.json").read_text(encoding="utf-8"))
        fingerprint = common_contract(project)
        assert (inventory["common_contract_sha256"] == fingerprint
                or binary_reuse_matches(out, inventory["common_contract_sha256"], fingerprint)
                or (flavor == "cpp" and baseline_contract_matches(base_out, inventory["common_contract_sha256"], fingerprint))), "Captured common contracts have changed"
        for relative, expected in inventory["files"].items():
            assert hashlib.sha256((project / relative).read_bytes()).hexdigest() == expected, relative
        validate_captures(base_out, out, flavor, selected, fingerprint)
        return

    def run(*args, label=None):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or "command.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        descriptor = next(project.glob("*.epokproject"))
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = "native_cpp"
        write(descriptor, manifest)
        run("--project", project, "--build-psx", label="native-export-build.log")
        export_probe(out, project, run, ["health.hpp", "projectile3d.hpp"])
        return
    freeze_legacy_prefix(base_out)
    freeze_accepted_prefix(base_out)
    host_tail_oracle(base_out)
    host_feedback_oracle(base_out)
    host_controller_oracle(base_out)
    host_world_oracle(base_out)
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Combined Combat " + flavor, label="create.log")
    scripts = project / "assets/scripts"
    shutil.copyfile(NATIVE / "DungeonRulesBinding.hpp", scripts / "DungeonRulesBinding.hpp")
    for name in POLICIES:
        shutil.copyfile(NATIVE / (name + "Base.hpp"), scripts / (name + "Base.hpp"))
        if flavor == "cpp":
            shutil.copyfile(NATIVE / (name + ".hpp"), scripts / (name + ".hpp"))
        elif flavor == "blueprint":
            write(project / "assets/Blueprints" / (name + ".epokbp"),
                  (FLAVORS / "blueprint" / (name + ".epokbp")).read_text(encoding="utf-8"))
        else:
            write(scripts / (name + ".lua"), (FLAVORS / "lua" / (name + ".lua")).read_text(encoding="utf-8"))
        assert flavor == "cpp" or not (scripts / (name + ".hpp")).exists(), (flavor, name, "native policy leak")
    if flavor == "lua":
        write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/" + name + ".lua": class_id for name, class_id in POLICIES.items()}})
    write(scripts / "DungeonCombatProbe.hpp", source())
    # Reflect the current SDK and authored contracts before composing the scene.
    metadata = json.loads(run("--project", project, "--reflect", label="reflection.json"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))

    def ref(name):
        return {"name": name, "class_id": classes[name]["id"]}

    def component(name, key, properties=None, root=False):
        props = properties or {}
        value = {"id": identity(key), "name": key, "class": ref(name), "properties": props, "overrides": list(props)}
        if root:
            value["root"] = True
        return value

    def actor(name, key, x=0, z=0, wall=False):
        result = {"id": identity(key), "name": name, "class": ref("epok::Actor3D"), "properties": {}, "overrides": [],
                  "components": [component("epok::SceneComponent3D", key + "/root", {"position": [x, 0, z]}, True)]}
        if wall:
            result["components"].append(component("epok::Collider3DComponent", key + "/collider", {"collider": {"enabled": True, "layer": 1}}))
        return result

    shot0, shot1, wall = actor("Shot zero", "shot0"), actor("Shot one", "shot1"), actor("Collision target", "wall", 2, wall=True)
    driver = actor("Combined combat probe", "actor")
    driver["class"] = {"name": "DungeonCombatProbe", "class_id": DRIVER}
    driver["properties"] = {"vitals": identity("vitals"), "combat": identity("combat"), "encounter": identity("encounter"), "flow":identity("flow"), "projectiles": identity("pool"), "wall": wall["id"]}
    driver["overrides"] = list(driver["properties"])
    for key, name in (("vitals", "DungeonPlayerVitals"), ("combat", "DungeonPlayerCombat"), ("encounter", "DungeonEncounter")):
        driver["components"].append({"id": identity(key), "name": key, "class": {"name": name, "class_id": POLICIES[name]}, "properties": {}, "overrides": []})
    driver["components"].append(component("epok::SpatialQueryContextComponent", "context"))
    driver["components"].append(component("epok::ProjectilePool3DComponent", "pool", {"visual0": shot0["id"], "visual1": shot1["id"],
        "query_context": identity("context"), "manage_sprite_visibility": False}))
    driver["components"].append({"id":identity("flow"),"name":"Scenario RunFlow","class":{"name":"DungeonRunFlow","class_id":POLICIES["DungeonRunFlow"]},"properties":{},"overrides":[]})
    scene["actors"] = [driver, shot0, shot1, wall]
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    inventory = {str(p.relative_to(project)): hashlib.sha256(p.read_bytes()).hexdigest()
                 for folder in (scripts, project / "assets/Blueprints") if folder.exists() for p in folder.iterdir() if p.is_file()}
    fingerprint = common_contract(project)
    write(out / "source-inventory.json", {"provider": flavor, "native_policy_fallback": False, "files": inventory,
        "common_contract_sha256": fingerprint, "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
        "trace_words": WORDS, "trace_prefix_words": PREFIX_WORDS, "enemy_shot_tail_words": TAIL_WORDS,
        "accepted_prefix_words":ACCEPTED_WORDS,"feedback_tail_words":FEEDBACK_WORDS,"controller_tail_words":CONTROLLER_WORDS,"world_tail_words":WORLD_WORDS})
    if prepare_only:
        print("Prepared combined combat", flavor, flush=True)
        return

    def check(values, expected):
        assert values[:4] == [expected, STEPS, 0, STEPS] and all(values[n] > 0 for n in (5, 6, 7)), values

    run_modes(out, project, flavor, selected, "dungeon_combat_probe", MAGIC, check,
              provider_count=1, trace=("dungeon_combat_trace", STEPS * WORDS))
    validate_captures(base_out, out, flavor, selected, fingerprint)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/combat/providers")
    parser.add_argument("--flavor", action="append", choices=("cpp", "blueprint", "lua"))
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    parser.add_argument("--check-only", action="store_true", help="Validate retained captures without rebuilding or replaying")
    parser.add_argument("--host-oracle-only", action="store_true", help="Freeze the previous prefix and prepare the shared host shot oracle; no editor or console work")
    args = parser.parse_args()
    if args.host_oracle_only:
        freeze_legacy_prefix(args.output.resolve())
        freeze_accepted_prefix(args.output.resolve())
        values = host_tail_oracle(args.output.resolve())
        feedback = host_feedback_oracle(args.output.resolve())
        controller=host_controller_oracle(args.output.resolve())
        world=host_world_oracle(args.output.resolve())
        print("PASS independent host oracles",len(values),"shot words +",len(feedback),"feedback words +",len(controller),"controller reference words +",len(world),"world reference words; no editor or console work",flush=True)
        return
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.output.resolve(), flavor, args.mode, args.prepare_only, args.standalone_only, args.check_only)


if __name__ == "__main__":
    main()
