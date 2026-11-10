#pragma once
#include "epok.hpp"
#include "projectile3d.hpp"
#include "DungeonRulesBinding.hpp"

struct EPOK_VALUE(Configuration,Id="53ca29e5-10ea-5bd6-967a-d7ef5a62fed1") DungeonPlayerCombatTuning {
    epok::Fixed acquire_distance=13.0;
    epok::Fixed keep_distance=16.0;
    epok::Fixed projectile_speed=22.0;
    epok::Fixed projectile_lifetime=1.7;
    uint32_t visibility_period=8;
    uint32_t damage=20;
};
struct EPOK_VALUE(Id="b545a4d3-b9eb-5d1a-9515-a891d281e829") DungeonAttackState {
    uint32_t phase=0;
    bool attacking=false;
    bool shot_sent=false;
};
struct EPOK_VALUE(Id="44d396de-4b21-5df3-b62f-492685c4e52d") DungeonAttackStep {
    DungeonAttackState state{};
    int32_t clip=-1;
    bool fire=false;
    bool invalidate_locomotion=false;
    bool starting=false;
};
struct EPOK_VALUE(Id="be5667ea-d447-5b54-8adf-c5587183d15a") DungeonTargetCheck {
    uint32_t phase=0;
    bool keep=false;
    bool query_sight=false;
};

struct EPOK_VALUE(Id="1e5b0f7e-1258-5205-a48e-ed6b015af768") DungeonTargetDistance {
    uint32_t high=0;
    uint32_t low=0;
};

struct EPOK_VALUE(Configuration,Id="96decaf4-e314-5e18-9bba-bb76e4884bfa") DungeonImpactTuning {
    uint32_t player_trace_mask=3;
    uint32_t enemy_trace_mask=5;
};
struct EPOK_VALUE(Configuration,Id="df18d8ca-fd63-5d76-8705-7136666ea5e5") DungeonPlayerShotVisual {
    epok::Fixed size=.18;
    uint32_t red=255;
    uint32_t green=255;
    uint32_t blue=255;
};
struct EPOK_VALUE(Configuration,Id="f6dce8e8-3f15-57c5-9252-84397bf32e6b") DungeonCombatEffectTuning {
    epok::Fixed muzzle_size=.28;
    epok::Fixed hit_size=.48;
    uint32_t muzzle_first_frame=4;
    uint32_t muzzle_second_frame=5;
    uint32_t muzzle_switch_age=4;
    uint32_t muzzle_end_age=6;
    uint32_t hit_first_frame=6;
    uint32_t hit_ticks_per_frame=2;
    uint32_t hit_end_age=10;
};
struct EPOK_VALUE(Configuration,Id="e473d1f1-4d50-51cb-b370-417140e4fb8a") DungeonDamageFeedbackTuning {
    uint32_t duration_us=220000;
    uint32_t border_pixels=6;
    uint32_t red_base=96;
    uint32_t red_span=128;
    uint32_t green=16;
    uint32_t blue=20;
    uint32_t rumble_large=160;
    uint32_t rumble_duration_ms=120;
    bool rumble_small=true;
};
struct EPOK_VALUE(Id="d98f56cf-d233-523b-a4fa-a9d518a20bab") DungeonImpactTrace {
    uint32_t mask=0;
    bool ignore_player=false;
};
struct EPOK_VALUE(Id="c7493a8a-3fbc-5b80-9036-9edb545a3736") DungeonCombatEffectStep {
    uint32_t age=0;
    uint32_t frame=0;
    epok::Fixed size=0.0;
    bool playing=false;
};
struct EPOK_VALUE(Id="d58f3216-5f7f-5544-a866-b44a58667d6b") DungeonDamageFeedback {
    uint32_t remaining_us=0;
    bool enabled=false;
    uint32_t border_pixels=0;
    uint32_t red=0;
    uint32_t green=0;
    uint32_t blue=0;
};
struct EPOK_VALUE(Id="21f545b5-4697-5423-9bb1-bc269c6c88aa") DungeonDamageRumble {
    uint32_t large=0;
    bool small=false;
    uint32_t duration_ms=0;
};

struct EPOK_VALUE(Id="2cb2a25b-9538-562b-8ad7-553ad6dd50fe") DungeonControlCamera {
    epok::Fixed yaw{};
    epok::Fixed pitch{};
    epok::Fixed goal_yaw{};
    epok::Fixed goal_pitch{};
    epok::Fixed hold{};
    epok::Fixed blend{};
    uint32_t mode{};
};
struct EPOK_VALUE(Id="6da6442e-8811-541f-98be-a32eca472c62") DungeonControlCameraInput {
    epok::MovementIntent look{};
    epok::Fixed dt{};
    epok::Fixed facing{};
    epok::Fixed target_yaw{};
    epok::Fixed target_pitch{};
    bool free_aim{};
    bool target{};
    bool moving{};
};
struct EPOK_VALUE(Id="5f382af6-1619-5da7-8b5e-656647458245") DungeonControlMove {
    epok::Fixed x{};
    epok::Fixed z{};
    epok::Fixed facing{};
};
struct EPOK_VALUE(Id="12286154-5c98-590c-9638-f0dc6c66b1b3") DungeonControlView {
    epok::GameplayVector3 origin{};
    epok::GameplayVector3 offset{};
    epok::Fixed distance{};
    epok::Fixed yaw{};
    epok::Fixed pitch{};
};
struct EPOK_VALUE(Id="213ee7d4-8d3c-5894-be72-c6d304c646d9") DungeonControlAngles {
    epok::Fixed yaw{};
    epok::Fixed pitch{};
};
struct EPOK_VALUE(Id="c848c939-cb91-55b0-b1d2-9419bfb3dff4") DungeonControlMarker {
    epok::Fixed intro{};
    epok::Fixed spin{};
    epok::Fixed size{};
    uint32_t tint{};
    bool average{};
};
struct EPOK_VALUE(Id="21115ec9-1b6d-59ff-aa93-b7560b571fff") DungeonControlRig {
    int32_t bone{};
    int32_t stop_a{};
    int32_t stop_b{};
};
struct EPOK_VALUE(Id="c67ee177-373b-5fc8-a907-be7db94040e6") DungeonControlAnimation {
    uint32_t state{};
    bool loop{};
    bool locomotion{};
    bool switch_clip{};
    epok::Fixed rate{};
};
struct EPOK_VALUE(Id="7000affb-31a0-5a55-bc83-06af46ec8c4a") DungeonControlClock {
    uint32_t phase{};
    uint32_t ticks{};
};
struct EPOK_VALUE(Id="ad32ec66-7281-5f3c-bf5e-bfa245447976") DungeonControlDirection {
    epok::GameplayVector3 value{};
    bool valid{};
};
struct EPOK_VALUE(Configuration,Id="9a07aac4-1379-5c8a-8846-59bfbebd9240") DungeonControlMovementTuning {
    epok::Fixed speed=6.0;
    epok::Fixed turn_rate=540.0;
};
struct EPOK_VALUE(Configuration,Id="b1531533-2c7c-5191-aaab-bb57a0dff912") DungeonControlCameraTuning {
    epok::Fixed initial_pitch=18.0;
    epok::Fixed orbit_rate=140.0;
    epok::Fixed pitch_rate=70.0;
    epok::Fixed pitch_min=-10.0;
    epok::Fixed pitch_max=55.0;
    epok::Fixed blend_rate=4.0;
    epok::Fixed normal_response=4.0;
    epok::Fixed manual_response=12.0;
    epok::Fixed normal_cap=90.0;
    epok::Fixed manual_cap=140.0;
    epok::Fixed pitch_response=12.0;
    epok::Fixed manual_time=1.0;
};
struct EPOK_VALUE(Configuration,Id="0823f799-464a-5129-b75d-0c3d885c3fe7") DungeonControlViewTuning {
    epok::Fixed boom=5.0;
    epok::Fixed height=1.1;
    epok::Fixed aim_height=1.35;
    epok::Fixed shoulder_offset=0.55;
    epok::Fixed shoulder_min=0.5;
};
struct EPOK_VALUE(Configuration,Id="442f4d86-bc38-5812-b318-dcf74d17df71") DungeonControlAimTuning {
    epok::Fixed pitch_min=-35.0;
    epok::Fixed pitch_max=45.0;
    epok::Fixed pose_rate=220.0;
    epok::Fixed focus_height=0.8;
    epok::Fixed ray_distance=40.0;
    uint32_t ray_mask=3;
    uint32_t muzzle_mask=1;
};
struct EPOK_VALUE(Configuration,Id="3b3d02a8-5561-553a-866b-fa58936611b0") DungeonControlMarkerTuning {
    epok::Fixed size=0.8;
    epok::Fixed intro_frames=8.0;
    epok::Fixed base_radius=96.0;
    epok::Fixed step_radius=64.0;
    epok::Fixed brightness_step=16.0;
    epok::Fixed brightness_end=128.0;
};
struct EPOK_VALUE(Configuration,Id="61eb2008-755d-589f-a04b-267c4cb548e1") DungeonControlAnimationTuning {
    epok::Fixed walk_cycle=2.6;
    epok::Fixed run_cycle=6.0;
    epok::Fixed rate_min=0.2;
    epok::Fixed rate_max=1.6;
    epok::Fixed idle_speed=0.119873046875;
    epok::Fixed run_speed=3.60009765625;
    epok::Fixed keep_run_speed=3.0;
};

// Shared data/dispatch contract only. Each authored flavor owns attack phases,
// movement suppression, target eligibility/order and shot tuning decisions.
class EPOK_CLASS(Abstract,Blueprintable,Owners=World3D,Id="3eb1c1c2-571a-565d-943f-d908937b5754") DungeonPlayerCombatBase:public epok::ActorComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("3eb1c1c2-571a-565d-943f-d908937b5754");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,Id="19d74c12-ff4a-52d9-8926-8a7620b2e64a") DungeonPlayerCombatTuning tuning{13.0,16.0,22.0,1.7,8,20};
    EPOK_FUNCTION(BlueprintPure,Id="5f1ecdfc-0d02-504a-958b-c30b21805bf2") DungeonAttackState make_attack_state(uint32_t phase,bool attacking,bool shot_sent)const{return {phase,attacking,shot_sent};}
    EPOK_FUNCTION(BlueprintPure,Id="1af7e3e7-a310-5f27-9850-8497152e9586") DungeonAttackStep make_attack_step(DungeonAttackState state,int32_t clip,bool fire,bool invalidate_locomotion,bool starting)const{return {state,clip,fire,invalidate_locomotion,starting};}
    EPOK_FUNCTION(BlueprintPure,Id="eacab543-bb29-535f-9c0d-b90f3157d7a2") DungeonTargetCheck make_target_check(uint32_t phase,bool keep,bool query_sight)const{return {phase,keep,query_sight};}
    EPOK_FUNCTION(BlueprintPure,Id="39f5093a-4df4-542b-8f3d-7faf6168efe0") DungeonTargetDistance make_target_distance(uint32_t high,uint32_t low)const{return {high,low};}

    EPOK_PROPERTY(EditAnywhere,Id="8c0142c3-4047-57aa-b66c-3bdcd3ce0f46") DungeonImpactTuning impact_tuning{3,5};
    EPOK_PROPERTY(EditAnywhere,Id="df338b81-a73c-540d-b7ad-1ee275c0cb67") DungeonPlayerShotVisual shot_style{.18,255,255,255};
    EPOK_PROPERTY(EditAnywhere,Id="dad7de9b-21f5-5a39-ae8b-9d95860341e4") DungeonCombatEffectTuning effect_tuning{.28,.48,4,5,4,6,6,2,10};
    EPOK_PROPERTY(EditAnywhere,Id="93dad28f-6220-53b1-9057-d96f9a3d9495") DungeonDamageFeedbackTuning feedback_tuning{220000,6,96,128,16,20,160,120,true};
    EPOK_FUNCTION(BlueprintPure,Id="d0557468-4c56-5b5c-b72b-51da157cbdd1") DungeonImpactTrace make_impact_trace(uint32_t mask,bool ignore_player)const{return {mask,ignore_player};}
    EPOK_FUNCTION(BlueprintPure,Id="4fa2eded-cb8a-5648-985a-b5ece368fb2e") DungeonPlayerShotVisual make_shot_visual(epok::Fixed size,uint32_t red,uint32_t green,uint32_t blue)const{return {size,red,green,blue};}
    EPOK_FUNCTION(BlueprintPure,Id="c1109276-a4fb-5469-ab0d-def499aa401c") DungeonCombatEffectStep make_effect_step(uint32_t age,uint32_t frame,epok::Fixed size,bool playing)const{return {age,frame,size,playing};}
    EPOK_FUNCTION(BlueprintPure,Id="2c37d1d3-8638-5376-bb7e-2d3fbd1382ac") DungeonDamageFeedback make_damage_feedback(uint32_t remaining_us,bool enabled,uint32_t border_pixels,uint32_t red,uint32_t green,uint32_t blue)const{return {remaining_us,enabled,border_pixels,red,green,blue};}
    EPOK_FUNCTION(BlueprintPure,Id="7bae22ba-9fc4-584e-8830-7b65aefa7c6f") DungeonDamageRumble make_damage_rumble(uint32_t large,bool small,uint32_t duration_ms)const{return {large,small,duration_ms};}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="b289a719-8839-58d8-9ebc-cb9adf57853b") virtual DungeonImpactTrace trace_settings(bool enemy)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="b1dc77d0-2bf2-5701-bb47-093d95f1769a") virtual uint32_t impact(bool enemy,bool target_exists,bool target_is_player,uint32_t target_layers)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="9bf8c160-63fa-5e38-ae14-6b366fdad33a") virtual DungeonPlayerShotVisual shot_visual()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="56d5a663-5adf-5145-9b7a-3d3c57fbdb52") virtual DungeonCombatEffectStep effect_step(uint32_t kind,uint32_t age,bool advance)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="0830b192-4912-5114-8c95-4c7b17d11030") virtual DungeonDamageFeedback damage_feedback(uint32_t remaining_us,uint32_t elapsed_us,bool restart)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="551d158d-d8b2-5b68-94be-c743a39863ab") virtual DungeonDamageRumble damage_rumble()const=0;
    EPOK_PROPERTY(EditAnywhere,Id="cd76f734-46e2-5d0c-a688-3ec5f575265e") DungeonControlMovementTuning control_movement{6.0,540.0};
    EPOK_PROPERTY(EditAnywhere,Id="20ac89c0-9386-53de-924c-15808c98ea9b") DungeonControlCameraTuning control_camera{18.0,140.0,70.0,-10.0,55.0,4.0,4.0,12.0,90.0,140.0,12.0,1.0};
    EPOK_PROPERTY(EditAnywhere,Id="87b5ccd2-bc74-5a04-a366-e002a799d42e") DungeonControlViewTuning control_view{5.0,1.1,1.35,0.55,0.5};
    EPOK_PROPERTY(EditAnywhere,Id="222e1858-06df-5768-9c16-cf81ea2131d2") DungeonControlAimTuning control_aim{-35.0,45.0,220.0,0.8,40.0,3,1};
    EPOK_PROPERTY(EditAnywhere,Id="9a90b033-d156-5e40-86cf-c2619a2d703a") DungeonControlMarkerTuning control_marker{0.8,8.0,96.0,64.0,16.0,128.0};
    EPOK_PROPERTY(EditAnywhere,Id="e4595e5e-2b56-5ced-a74e-2892e8d3e371") DungeonControlAnimationTuning control_animation{2.6,6.0,0.2,1.6,0.119873046875,3.60009765625,3.0};
    EPOK_FUNCTION(BlueprintPure,Id="49b42812-b508-5a91-8fc3-c1d2f2504e1d") DungeonControlCamera make_controls_DungeonControlCamera(epok::Fixed yaw,epok::Fixed pitch,epok::Fixed goal_yaw,epok::Fixed goal_pitch,epok::Fixed hold,epok::Fixed blend,uint32_t mode)const{return {yaw,pitch,goal_yaw,goal_pitch,hold,blend,mode};}
    EPOK_FUNCTION(BlueprintPure,Id="cf889906-ac69-53cd-b49f-cc9400836cfe") DungeonControlMove make_controls_DungeonControlMove(epok::Fixed x,epok::Fixed z,epok::Fixed facing)const{return {x,z,facing};}
    EPOK_FUNCTION(BlueprintPure,Id="b577d06b-8309-59f1-bf93-46eb511002a0") DungeonControlView make_controls_DungeonControlView(epok::GameplayVector3 origin,epok::GameplayVector3 offset,epok::Fixed distance,epok::Fixed yaw,epok::Fixed pitch)const{return {origin,offset,distance,yaw,pitch};}
    EPOK_FUNCTION(BlueprintPure,Id="91c04af0-27cf-5e60-876b-e10a98209fa3") DungeonControlAngles make_controls_DungeonControlAngles(epok::Fixed yaw,epok::Fixed pitch)const{return {yaw,pitch};}
    EPOK_FUNCTION(BlueprintPure,Id="847bd88f-9a92-55e8-9682-ddd3934f1184") DungeonControlMarker make_controls_DungeonControlMarker(epok::Fixed intro,epok::Fixed spin,epok::Fixed size,uint32_t tint,bool average)const{return {intro,spin,size,tint,average};}
    EPOK_FUNCTION(BlueprintPure,Id="763fc006-ea16-5e57-b7e1-59d0c7eba3b6") DungeonControlRig make_controls_DungeonControlRig(int32_t bone,int32_t stop_a,int32_t stop_b)const{return {bone,stop_a,stop_b};}
    EPOK_FUNCTION(BlueprintPure,Id="435dad09-91ae-5154-9a15-c7193f14b885") DungeonControlAnimation make_controls_DungeonControlAnimation(uint32_t state,bool loop,bool locomotion,bool switch_clip,epok::Fixed rate)const{return {state,loop,locomotion,switch_clip,rate};}
    EPOK_FUNCTION(BlueprintPure,Id="1fc3cfcd-1ecb-5150-bbde-5711eca66875") DungeonControlClock make_controls_DungeonControlClock(uint32_t phase,uint32_t ticks)const{return {phase,ticks};}
    EPOK_FUNCTION(BlueprintPure,Id="685f4a51-97f8-53f3-8600-68127a40d3a7") DungeonControlDirection make_controls_DungeonControlDirection(epok::GameplayVector3 value,bool valid)const{return {value,valid};}
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="229f54ac-0d84-5f4f-a907-5ede3f250aa3") virtual uint32_t control_gate(bool ready,bool interface_block,bool controls,bool alive)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="66482d32-ed31-5e30-8916-fefb015a6803") virtual epok::MovementIntent control_intent(epok::Fixed x,epok::Fixed y,bool right,bool left,bool up,bool down,bool suppress)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="2ed1669f-005a-5f36-b63a-d18150409ff5") virtual DungeonControlCamera control_camera_start(epok::Fixed facing)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4541b6c7-c67a-5d9a-b58a-a5111ecc0ae1") virtual DungeonControlCamera control_camera_step(DungeonControlCamera state,DungeonControlCameraInput input)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="b62ae289-bbf5-5447-86d5-a3f4cb9d2973") virtual DungeonControlMove control_movement_step(epok::MovementIntent intent,DungeonControlCamera state,epok::Fixed facing,epok::Fixed target_yaw,bool target,bool attack_owns,bool fire_pressed,epok::Fixed dt)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="285a6df6-fb73-57ea-88c7-eec0b401e7c0") virtual DungeonControlView control_view_proposal(epok::GameplayVector3 position,DungeonControlCamera state,epok::Fixed shoulder_distance)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="7f3b40ce-2a81-5219-9384-974096012cc5") virtual epok::Fixed control_focus_height(bool enabled,epok::Fixed center,epok::Fixed scale)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="7ed92b16-4877-52ac-bca6-ab29fddb1319") virtual DungeonControlAngles control_target_angles(epok::GameplayVector3 origin,epok::GameplayVector3 target,epok::Fixed focus)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="f69864b9-4aaa-5cc9-ae73-d01d23755e53") virtual epok::Fixed control_aim_step(epok::Fixed current,epok::Fixed camera_pitch,epok::Fixed target_pitch,epok::Fixed dt,bool free_aim,bool target)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="886f9b7b-04e5-510e-a560-6163383615ad") virtual uint32_t control_marker_begin()const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="9c293ef5-0ebf-57a8-881f-a6776355ca9d") virtual DungeonControlMarker control_marker_step(epok::Fixed intro,epok::Fixed spin)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="c497900a-156b-5db8-8755-4dd0f99fc5b8") virtual DungeonControlRig control_rig(uint32_t bones)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="18b86a6d-c9ff-5b7d-ae26-1f3aefc705f0") virtual DungeonControlAnimation control_animation_select(uint32_t state,uint32_t playing,bool grounded,epok::Fixed vertical_speed,epok::Fixed speed,bool finished,bool jumped)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="aee77db2-9520-58f5-9ebe-aaa6230672c0") virtual DungeonControlClock control_animation_clock(uint32_t phase,uint32_t frames,epok::Fixed rate,epok::Fixed dt,bool model_valid)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a3bbb9dd-eed3-5181-b293-c0aec3421007") virtual epok::GameplayVector3 control_direction(epok::Fixed yaw,epok::Fixed pitch)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a98dd22b-f8bf-54a6-b086-6bc4dad62b14") virtual epok::GameplayVector3 control_aim_ray(epok::Fixed yaw,epok::Fixed pitch)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="c81656ed-bc67-5251-b5e8-7f9d76495a28") virtual uint32_t control_aim_mask(bool muzzle)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="2736827a-88db-5ba8-acf8-485f8f44e705") virtual uint32_t control_shot_aim(bool target,uint32_t mode)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="b970517d-7b31-55b9-a443-944757ba444c") virtual DungeonControlDirection control_direction_toward(epok::GameplayVector3 delta)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a6c3cb6b-8a22-5c96-b16f-dee35d8a5691") virtual epok::Fixed control_motion_speed(epok::GameplayVector3 displacement,epok::Fixed dt)const=0;
    EPOK_FUNCTION(BlueprintCallable,BlueprintEvent,Id="e2010996-d5fa-571a-87bd-c33ec4180b28") virtual DungeonAttackStep attack(DungeonAttackState state,bool grounded,bool jumped,bool held,bool clip_playing,bool clips_ready)=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="4e9568fa-1852-5b75-b461-3c7cf24b9c03") virtual bool blocks_movement(DungeonAttackState state,bool grounded,bool held)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="5e7c0bac-25b7-5df7-9edd-09da2e766fc8") virtual uint32_t target_input(bool free_held,bool free_pressed,bool lock_pressed)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="ad1792fc-24f4-526f-8377-05c60a05dbae") virtual DungeonTargetCheck target_check(uint32_t phase,bool candidate,bool within_keep)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="db5c14ec-4419-5cb2-bc0f-ebb2a8812269") virtual bool candidate(bool active,bool has_mesh,bool collider_enabled,bool marked)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="01a07e0b-4e95-5dda-b2b6-234372987704") virtual bool after_selected(DungeonTargetDistance distance,uint32_t index,DungeonTargetDistance selected_distance,uint32_t selected_index)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="a157b49b-7041-5cdb-95bc-e7af5d4981ab") virtual bool prefer_candidate(DungeonTargetDistance distance,uint32_t index,bool best_exists,DungeonTargetDistance best_distance,uint32_t best_index,bool cycling)const=0;
    EPOK_FUNCTION(BlueprintPure,BlueprintEvent,Id="55118aca-7bab-53c6-8af5-e3b5df35d4a9") virtual epok::ProjectileSpawn3D projectile(epok::GameplayVector3 origin,epok::GameplayVector3 direction)const=0;
};
using DungeonPlayerCombatBinding=DungeonRulesBinding<DungeonPlayerCombatBase>;
