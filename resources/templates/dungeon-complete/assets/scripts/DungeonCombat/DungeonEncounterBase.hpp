#pragma once
#include "epok.hpp"
#include "projectile3d.hpp"
#include "DungeonRulesBinding.hpp"

// Project data/dispatch only. Each flavor owns the encounter decisions.
struct EPOK_VALUE(Configuration,Id="03be1d46-b784-5294-81f7-2907d4ddf768") DungeonEncounterHealth {
    uint32_t guard_health=60;
    uint32_t boss_health=280;
    uint32_t boss_phase_health=140;
};
struct EPOK_VALUE(Configuration,Id="4f6c8f55-5c8e-5082-9f49-5ba0d8cb4629") DungeonEncounterMovement {
    epok::Fixed activation_radius=14.0;
    epok::Fixed retention_radius=22.0;
    epok::Fixed sight_radius=11.0;
    epok::Fixed guard_range=5.0;
    epok::Fixed patrol_extent=1.25;
    epok::Fixed patrol_arrival=.2;
    epok::Fixed chase_speed=1.05;
    epok::Fixed patrol_speed=.6;
};
struct EPOK_VALUE(Configuration,Id="94894761-e2d2-5429-8efa-eea905ad5316") DungeonGuardTiming {
    epok::Fixed guard_initial_delay=1.1;
    epok::Fixed guard_cooldown=2.1;
    epok::Fixed guard_windup=.5;
    epok::Fixed guard_flash=.14;
    epok::Fixed guard_death=.85;
    epok::Fixed sight_interval=.1;
    epok::Fixed ai_interval=1.0/30.0;
};
struct EPOK_VALUE(Configuration,Id="bdb32f9c-f3b3-5f7a-8b38-17c3d8666a40") DungeonBossTiming {
    epok::Fixed boss_initial_delay=1.8;
    epok::Fixed boss_cooldown=1.9;
    epok::Fixed boss_fan_cooldown=1.35;
    epok::Fixed boss_windup=.7;
    epok::Fixed boss_fan_windup=.5;
    epok::Fixed boss_flash=.16;
    epok::Fixed boss_range=7.0;
    epok::Fixed boss_speed=.85;
    epok::Fixed boss_fan_speed=1.3;
};
struct EPOK_VALUE(Id="eb5c8d1d-862b-5904-b13f-0613a61d253e") DungeonGuardState {
    epok::Fixed timer=0.0,windup=0.0,flash=0.0,dead_time=0.0,sight_timer=0.0,ai_time=0.0;
    bool patrol=false,can_see=false;
};
// action: 0 none, 1 idle, 2 chase, 3 patrol, 4 charge, 5 fire,
// 6 unload, 7 scheduled update. Values are ordinary project data.
struct EPOK_VALUE(Id="adb2c4ad-316d-5a31-8542-e3bd32d888bd") DungeonGuardStep {
    DungeonGuardState state{};epok::Fixed dt=0.0;uint32_t action=0;
};
struct EPOK_VALUE(Id="428cab91-4530-553a-a49f-95636d6dd43a") DungeonBossState {
    epok::Fixed timer=0.0,windup=0.0,flash=0.0;
};
struct EPOK_VALUE(Id="6ba4dc44-860c-5890-88d7-3d87aa2c4c4f") DungeonBossStep {
    DungeonBossState state{};uint32_t action=0;bool fan=false;epok::Fixed speed=0.0;
};
struct EPOK_VALUE(Configuration,Id="cb42a10e-4be0-50e5-a490-08b175b92905") DungeonEncounterShotTuning {
    epok::Fixed speed=7.0;
    epok::Fixed fan_speed=8.0;
    epok::Fixed lifetime=2.5;
    epok::Fixed fan_degrees=12.0;
    epok::Fixed size=.24;
    epok::Fixed fan_size=.32;
    uint32_t damage=12;
    uint32_t boss_damage=16;
    epok::Fixed target_height=.95;
};
struct EPOK_VALUE(Id="9f1ccb80-a401-58c5-8e96-93e8c0ffdb0a") DungeonWorldColor {
    uint32_t red{};
    uint32_t green{};
    uint32_t blue{};
};
struct EPOK_VALUE(Id="3cd02a0e-ec8d-532c-b746-ce03df08eacd") DungeonWorldHit {
    bool defeated{};
    bool disable_collider{};
    bool death_clip{};
    bool victory{};
    bool reset_projectiles{};
    bool explosion{};
    bool hud_dirty{};
    uint32_t reward{};
    uint32_t sound{};
};
struct EPOK_VALUE(Id="6b0f1ad2-4e04-5a8e-a27b-ea0fed672812") DungeonWorldDeath {
    bool defeat{};
    bool clear_aim{};
    bool reset_projectiles{};
};
struct EPOK_VALUE(Id="e34e1a34-a3fb-523e-90f5-de133e8435e7") DungeonWorldDoor {
    epok::Fixed progress{};
    bool opening{};
    bool moved{};
    bool opened{};
    bool locked_tint{};
    epok::Fixed offset{};
    DungeonWorldColor color{};
};
struct EPOK_VALUE(Id="3183a566-5693-5e77-b53f-eb55ab193390") DungeonWorldClip {
    int32_t clip{};
    bool play{};
    bool loop{};
    bool charge_cue{};
};
struct EPOK_VALUE(Id="a51d5164-c68a-5e50-a6b0-42dadd596fc0") DungeonWorldDirection {
    epok::GameplayVector3 value{};
    bool valid{};
};
struct EPOK_VALUE(Id="35aff715-7998-5115-8ae2-eb736faf0c74") DungeonWorldMotion {
    epok::GameplayVector3 delta{};
    epok::Fixed yaw{};
    epok::Fixed patrol_yaw{};
    bool move{};
    bool turn{};
    bool patrol_turn{};
    bool valid{};
};
struct EPOK_VALUE(Configuration,Id="bed1201c-1065-5c2f-9af5-f51d6ab6a56d") DungeonWorldDamageTuning {
    uint32_t guard_reward=1;
    uint32_t boss_reward=0;
    bool player_defeat=true;
    bool player_reset=true;
    bool boss_victory=true;
    bool boss_reset=true;
    bool hide_victorious_boss=true;
};
struct EPOK_VALUE(Configuration,Id="9c424353-dd04-5a5a-9f4e-f0b2e0df9739") DungeonGuardColors {
    uint32_t normal_red=100;
    uint32_t normal_green=210;
    uint32_t normal_blue=255;
    uint32_t windup_red=255;
    uint32_t windup_green=180;
    uint32_t windup_blue=85;
    uint32_t hit_red=255;
    uint32_t hit_green=110;
    uint32_t hit_blue=110;
};
struct EPOK_VALUE(Configuration,Id="c87674fb-0995-51c8-8237-a646b12b9e2b") DungeonBossColors {
    uint32_t normal_red=255;
    uint32_t normal_green=255;
    uint32_t normal_blue=255;
    uint32_t fan_red=255;
    uint32_t fan_green=190;
    uint32_t fan_blue=150;
    uint32_t windup_red=255;
    uint32_t windup_green=150;
    uint32_t windup_blue=55;
    uint32_t hit_red=255;
    uint32_t hit_green=90;
    uint32_t hit_blue=90;
};
struct EPOK_VALUE(Configuration,Id="780258f8-8b18-5f4a-b8a7-5e54de357ad7") DungeonEnemyShotColor {
    uint32_t red=255;
    uint32_t green=100;
    uint32_t blue=50;
};
struct EPOK_VALUE(Configuration,Id="cb857eaf-e615-515a-9217-0f9a41a6e85e") DungeonWorldDoorTuning {
    epok::Fixed open_radius=5.0;
    epok::Fixed safety_radius=2.0;
    epok::Fixed speed=1.4;
    epok::Fixed closed_offset=0.75;
    epok::Fixed travel=1.7;
    uint32_t locked_red=255;
    uint32_t locked_green=130;
    uint32_t locked_blue=110;
};
struct EPOK_VALUE(Configuration,Id="a9c76d3f-1f5f-580a-9cec-c907330e8ce7") DungeonWorldPickupTuning {
    epok::Fixed radius=1.0;
    epok::Fixed maximum_y=2.2;
    uint32_t heal=35;
    epok::Fixed bob_rate=120.0;
    epok::Fixed bob_extent=0.12;
    epok::Fixed left_x=-8.0;
    epok::Fixed right_x=8.0;
    epok::Fixed minimum_z=32.0;
    epok::Fixed maximum_z=55.0;
};
struct EPOK_VALUE(Configuration,Id="13363957-4b65-53af-be74-16c43b06f2e7") DungeonWorldAnimationTuning {
    int32_t idle_clip=0;
    int32_t move_clip=1;
    int32_t death_clip=2;
    bool loop=true;
};
struct EPOK_VALUE(Configuration,Id="b1d9884a-a055-5ccb-8205-43d99fdfc328") DungeonWorldMotionTuning {
    epok::Fixed ground_step=-0.05;
    epok::Fixed boss_interval=0.03333333333333333;
};

class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="471e00ac-5af4-56ca-ada3-06d007b81676") DungeonEncounterBase:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("471e00ac-5af4-56ca-ada3-06d007b81676");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,Id="6380951b-182b-59f0-a6d5-f695180efb49") DungeonEncounterHealth health_tuning{60,280,140};
    EPOK_PROPERTY(EditAnywhere,Id="b5bc4ff2-109e-59dd-a1df-91100c651197") DungeonEncounterMovement movement{14.0,22.0,11.0,5.0,1.25,.2,1.05,.6};
    EPOK_PROPERTY(EditAnywhere,Id="603d89de-3dce-5d21-aa70-17f451835f5a") DungeonGuardTiming guard_timing{1.1,2.1,.5,.14,.85,.1,.0333333333333333};
    EPOK_PROPERTY(EditAnywhere,Id="06b71955-c901-5484-8e16-aa1d46f586f1") DungeonBossTiming boss_timing{1.8,1.9,1.35,.7,.5,.16,7.0,.85,1.3};
    EPOK_PROPERTY(EditAnywhere,Id="a47e187c-a9b4-5cf8-8309-233770d07d14") DungeonEncounterShotTuning shot_tuning{7.0,8.0,2.5,12.0,.24,.32,12,16,.95};
    EPOK_PROPERTY(EditAnywhere,Id="9eb907b7-1412-54a0-b4aa-5400ca641d13") DungeonWorldDamageTuning world_damage{1,0,true,true,true,true,true};
    EPOK_PROPERTY(EditAnywhere,Id="78a9da17-ac72-5339-9ab6-05961af0f2d0") DungeonGuardColors guard_colors{100,210,255,255,180,85,255,110,110};
    EPOK_PROPERTY(EditAnywhere,Id="1627ac87-c7e3-51a4-872d-aba956c7ef9e") DungeonBossColors boss_colors{255,255,255,255,190,150,255,150,55,255,90,90};
    EPOK_PROPERTY(EditAnywhere,Id="c3e55025-86ed-5312-a8f0-fb3acb1ddc8e") DungeonEnemyShotColor enemy_shot_color{255,100,50};
    EPOK_PROPERTY(EditAnywhere,Id="813787f1-a0d2-5343-ba5c-17e19a091f83") DungeonWorldDoorTuning world_doors{5.0,2.0,1.4,0.75,1.7,255,130,110};
    EPOK_PROPERTY(EditAnywhere,Id="5c1aa1be-b6ea-5458-a2b4-596ac3a3585f") DungeonWorldPickupTuning world_pickups{1.0,2.2,35,120.0,0.12,-8.0,8.0,32.0,55.0};
    EPOK_PROPERTY(EditAnywhere,Id="d5c7ba1d-9c68-56a9-b33a-47e5b5a20dc5") DungeonWorldAnimationTuning world_animation{0,1,2,true};
    EPOK_PROPERTY(EditAnywhere,Id="8a52df21-05a8-5c8e-b632-cb031cecf56c") DungeonWorldMotionTuning world_motion{-0.05,0.03333333333333333};
    EPOK_FUNCTION(BlueprintPure,Id="80353250-17bc-5f6c-8e79-833d24883ab3") DungeonWorldHit make_world_DungeonWorldHit(bool defeated,bool disable_collider,bool death_clip,bool victory,bool reset_projectiles,bool explosion,bool hud_dirty,uint32_t reward,uint32_t sound)const{return {defeated,disable_collider,death_clip,victory,reset_projectiles,explosion,hud_dirty,reward,sound};}
    EPOK_FUNCTION(BlueprintPure,Id="2ea2c3b1-3310-5ff7-b1d9-d07f44715a9c") DungeonWorldDeath make_world_DungeonWorldDeath(bool defeat,bool clear_aim,bool reset_projectiles)const{return {defeat,clear_aim,reset_projectiles};}
    EPOK_FUNCTION(BlueprintPure,Id="1eaae771-9dcc-57ed-b191-a43a69bbc08f") DungeonWorldColor make_world_DungeonWorldColor(uint32_t red,uint32_t green,uint32_t blue)const{return {red,green,blue};}
    EPOK_FUNCTION(BlueprintPure,Id="fdcc692e-f49e-55d4-8f5c-757feb3c92cb") DungeonWorldClip make_world_DungeonWorldClip(int32_t clip,bool play,bool loop,bool charge_cue)const{return {clip,play,loop,charge_cue};}
    EPOK_FUNCTION(BlueprintPure,Id="950f42a4-dfdb-5fae-abd4-56f8896df4b0") DungeonWorldDoor make_world_DungeonWorldDoor(epok::Fixed progress,bool opening,bool moved,bool opened,bool locked_tint,epok::Fixed offset,DungeonWorldColor color)const{return {progress,opening,moved,opened,locked_tint,offset,color};}
    EPOK_FUNCTION(BlueprintPure,Id="1a7325a8-acee-591d-a79f-556c4c0bae4a") DungeonWorldDirection make_world_DungeonWorldDirection(epok::GameplayVector3 value,bool valid)const{return {value,valid};}
    EPOK_FUNCTION(BlueprintPure,Id="645a5ce2-0d56-5503-bc63-b5db6ddeda40") DungeonWorldMotion make_world_DungeonWorldMotion(epok::GameplayVector3 delta,epok::Fixed yaw,epok::Fixed patrol_yaw,bool move,bool turn,bool patrol_turn,bool valid)const{return {delta,yaw,patrol_yaw,move,turn,patrol_turn,valid};}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="2f452823-3f10-5f55-9869-0714d81e5b32") virtual bool world_damage_eligible(bool boss,bool alive,bool boss_fight,uint32_t health,bool registered,epok::Fixed dead_time,bool defeated)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="6944a870-8042-50b8-9807-2e217c7c86db") virtual uint32_t world_damage_total(uint32_t base,uint32_t bonus)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a73d3d6a-33f3-5ee4-9783-88db59b98229") virtual uint32_t world_reward_count(uint32_t count,uint32_t amount)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="edb7d4f5-d09b-596f-a342-19aca8b0614d") virtual DungeonWorldHit world_hit_result(bool boss,bool depleted,uint32_t remaining)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="317dd1e9-e18e-50b5-88e2-196623cf1137") virtual DungeonWorldDeath world_player_death(bool depleted)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="fb5e464d-f83f-5e01-bf86-6e70b0320e07") virtual bool world_hide_boss(uint32_t state,bool playing)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="5b8892da-9419-5dbc-925d-211e57385f65") virtual DungeonWorldColor world_enemy_color(bool boss,epok::Fixed flash,epok::Fixed windup,bool fan)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="ce559c13-a92a-5d4d-b885-7b95e2d22553") virtual DungeonWorldColor world_shot_color()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="200d82a0-152d-5454-a7a0-399bf51122f8") virtual DungeonWorldClip world_death_animation()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="90a81c8b-4109-5dcc-befe-6ed9c764a107") virtual DungeonWorldClip world_enemy_animation(bool boss,uint32_t action,int32_t current)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="220c1487-03e5-5517-bc1e-9ed84f10ff2d") virtual bool world_door_open(uint32_t index,bool locked,bool has_key,uint32_t state,epok::Fixed progress,bool near_open,bool near_safety)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a582bfdb-9fe6-54ae-af41-2e6567929dd9") virtual epok::Fixed world_door_radius(bool safety)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="afcf7a83-e4b2-5cc1-9e98-7be715a68934") virtual DungeonWorldDoor world_door_step(uint32_t index,bool locked,bool has_key,uint32_t state,epok::Fixed progress,bool near_open,bool near_safety,epok::Fixed dt)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a7db67b3-875d-5148-90fa-dac859d4546e") virtual bool world_pickup_visible(uint32_t kind,epok::GameplayVector3 position)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4fa84806-9e3c-55b2-855b-a11ff336546e") virtual bool world_pickup_eligible(bool can_collect,bool nearby,epok::Fixed height)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="b945fda2-c8f0-5e68-b6c5-5f42d26c1ae8") virtual epok::Fixed world_pickup_radius()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="0e758554-30c7-5b56-9df2-1f672f1382f9") virtual uint32_t world_pickup_heal()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="2db11462-d8de-50e5-8031-6c7bda973ea0") virtual epok::Fixed world_pickup_bob(epok::Fixed elapsed)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="e31b2755-3918-563f-b43a-9ec3bba37053") virtual DungeonWorldDirection world_aim_direction(epok::GameplayVector3 delta)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="8efdea50-bfd4-5da8-b14d-490d02b9d960") virtual epok::GameplayVector3 world_patrol_goal(epok::GameplayVector3 home,bool patrol_side)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="d80ce8e0-b601-5d4f-b44e-783a9ce559a7") virtual DungeonWorldMotion world_enemy_motion(bool boss,uint32_t action,bool patrol_side,bool close,bool seen,epok::GameplayVector3 origin,epok::GameplayVector3 player,epok::GameplayVector3 home,epok::Fixed dt,epok::Fixed boss_speed)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="7c487f45-f20a-5f98-b3a7-a3444fbdf5bb") virtual bool world_boss_due(epok::Fixed elapsed)const=0;
    // Copy-only record constructors expose no game decisions.
    EPOK_FUNCTION(BlueprintPure,Id="601cc5a7-5743-5437-bd5c-a5acbc6dae70") DungeonGuardState make_guard(epok::Fixed timer,epok::Fixed windup,epok::Fixed flash,epok::Fixed dead_time,epok::Fixed sight_timer,epok::Fixed ai_time,bool patrol,bool can_see)const{return {timer,windup,flash,dead_time,sight_timer,ai_time,patrol,can_see};}
    EPOK_FUNCTION(BlueprintPure,Id="6f4add62-e2a8-586f-97ee-efe234985daf") DungeonGuardStep make_guard_step(DungeonGuardState state,epok::Fixed dt,uint32_t action)const{return {state,dt,action};}
    EPOK_FUNCTION(BlueprintPure,Id="5b4e496f-b668-5f21-8509-88efac0f6a0e") DungeonBossState make_boss(epok::Fixed timer,epok::Fixed windup,epok::Fixed flash)const{return {timer,windup,flash};}
    EPOK_FUNCTION(BlueprintPure,Id="38813525-320e-543c-b379-20dfcfa92896") DungeonBossStep make_boss_step(DungeonBossState state,uint32_t action,bool fan,epok::Fixed speed)const{return {state,action,fan,speed};}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="da963537-3fa3-5039-aa61-bde0fb30590d") virtual epok::HealthState starting_health(bool boss)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="d3829194-0c80-5eb6-ad29-cfde5a57870d") virtual bool should_activate(bool defeated,bool exploring,bool near_home)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="41047332-7609-56d5-b6f5-7097a4fb2cb3") virtual bool should_unload(DungeonGuardState state,bool boss_fight,bool near_home)const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="a8d664be-2f6e-59e6-8720-1471c5140ff7") virtual DungeonGuardState guard_start(uint32_t encounter)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="db0ab883-fad8-5c66-8cd9-f45062f58a81") virtual DungeonGuardStep guard_schedule(DungeonGuardState state,epok::Fixed dt)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="1e21bb4a-feda-5c4d-97dc-5b73ed3816a8") virtual DungeonGuardStep guard_decide(DungeonGuardState state,epok::Fixed dt,bool close,bool observed,bool within_range,bool at_patrol_goal)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="e29a9eed-eeb6-5e3e-a262-9234ba3399cd") virtual DungeonGuardState guard_hit(DungeonGuardState state,bool depleted)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="3276c755-4d1a-51c7-a953-7562afc6f151") virtual DungeonBossState boss_start()=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="1acf6df0-ecfa-58dc-a103-cb6e4862fb98") virtual bool boss_needs_sight(DungeonBossState state,epok::Fixed dt)const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="735b9a4b-63f7-550e-b563-df7be30d816a") virtual DungeonBossStep boss_advance(DungeonBossState state,uint32_t health,epok::Fixed dt,bool observed,bool within_range)=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="380f7f21-38ac-5b2b-b74a-f86378c172a7") virtual DungeonBossState boss_hit(DungeonBossState state)=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="0583882a-bec6-5b34-beab-790cc26cd104") virtual epok::ProjectileSpawn3D projectile(epok::GameplayVector3 origin,epok::GameplayVector3 direction,int32_t pellet,bool fan)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="abe59915-f222-5532-acd0-f7aaf071d14e") virtual epok::Fixed shot_size(bool fan)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a6c0a656-daaa-530d-8b2c-1de51323fbb4") virtual uint32_t shot_damage(bool boss_fight)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="cd296d9e-0e0d-528a-84c8-ae9d1c98e239") virtual uint32_t pellet_count(bool fan)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="d0d280e4-f819-5cae-9d43-611bf7c4b168") virtual int32_t pellet_index(uint32_t ordinal,bool fan)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4d9c57dc-9853-5f5c-8e09-2d4f5ba83fb0") virtual epok::GameplayVector3 shot_target(epok::GameplayVector3 player_position)const=0;
};
using DungeonEncounterBinding=DungeonRulesBinding<DungeonEncounterBase>;
