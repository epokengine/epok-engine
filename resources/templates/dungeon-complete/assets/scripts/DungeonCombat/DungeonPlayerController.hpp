#pragma once
#include "epok.hpp"
#include "character3d.hpp"
#include "DungeonCamera.hpp"
#include "CharacterMotion.hpp"
#include "DungeonPlayerCombatBase.hpp"

class DungeonGame;

// Project-owned movement. The child mesh has a separate facing offset;
// the collider and camera use the engine's positive-Z movement convention.
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="78439d2a-85ff-5a98-8f38-23293bcf9a59") DungeonPlayerController : public epok::ActorComponent {
    // Camera, target selection, movement facing and skeletal aim are deliberately
    // separate states. L1 explicitly clears target selection before switching
    // the camera to manual aim.
    enum class CameraMode { Normal, LockOn, FreeAim };
    epok::Fixed camera_yaw=0.0,camera_pitch=18.0,facing=0.0;
    epok::Fixed aim_pose_pitch=0.0,aim_camera_blend=0.0;
    epok::Fixed goal_yaw=0.0,goal_pitch=18.0,manual_hold=0.0;
    uint32_t visibility_phase=0;
    epok::CharacterMotionSample3D motion_{};
    CameraMode camera_mode=CameraMode::Normal;
    character_motion::AnimationState animation;
    epok::DataHandle visual_handle{};
    epok::DataHandle target_handle{};
    epok::DataHandle lock_marker_handle{};
    uint8_t lock_marker_intro_frames=0;
    uint8_t lock_marker_spin=0;
    int clips[6]={-1,-1,-1,-1,-1,-1};
    int attack_clips[3]={-1,-1,-1};
    uint8_t attack_phase=0;
    bool attacking=false,shot_sent=false;
    DungeonPlayerCombatBinding combat_binding_;
    mutable DungeonRulesBinding<DungeonGame> game_binding_;
    DungeonGame* game()const;
    uint32_t playback_phase=0;
    character_motion::State playing_state=character_motion::State::Land;
    epok::Character3D* character(){
        auto* registry=epok::active_object_registry;
        return registry?registry->resolve<epok::Character3D>(owner_id()):nullptr;
    }
    DungeonCamera* camera(){
        auto* registry=epok::active_object_registry;
        auto* player=character();
        return registry?registry->resolve<DungeonCamera>(camera_rig.valid()?camera_rig:(player?player->camera:epok::ObjectId{})):nullptr;
    }
    epok::ActorData* camera_data(){auto* view=camera();return view?view->data():nullptr;}
    DungeonControlCamera camera_state()const{return {camera_yaw,camera_pitch,goal_yaw,goal_pitch,manual_hold,aim_camera_blend,uint32_t(camera_mode)};}
    void assign_camera(const DungeonControlCamera& state){camera_yaw=state.yaw;camera_pitch=state.pitch;goal_yaw=state.goal_yaw;
        goal_pitch=state.goal_pitch;manual_hold=state.hold;aim_camera_blend=state.blend;camera_mode=CameraMode(state.mode);}
    void start(epok::Transform& transform);
    void update(epok::Transform& transform,epok::Fixed dt);
    void animate(epok::Fixed dt,epok::Fixed speed,bool jumped);
    void place_camera(const epok::Transform& transform);
    void toggle_target(epok::ActorData& body);
    DungeonControlAngles update_camera(epok::Transform& transform,epok::Fixed dt,bool free_aim,bool moving);
    // Stick shaping is pure and takes a square root. A held stick repeats it.
    epok::Fixed playback_elapsed_=0.0;
    bool playback_jumped_=false;
    struct IntentMemo {epok::Fixed x=0.0,y=0.0;uint8_t flags=255;epok::MovementIntent value{};} look_memo_{},move_memo_{};
    epok::MovementIntent intent_for(IntentMemo& memo,DungeonPlayerCombatBase& rules,epok::Fixed x,epok::Fixed y,bool right,bool left,bool up,bool down,bool suppress){
        const uint8_t flags=uint8_t(right)|uint8_t(left)<<1|uint8_t(up)<<2|uint8_t(down)<<3|uint8_t(suppress)<<4;
        if(memo.flags!=flags||memo.x!=x||memo.y!=y){memo.x=x;memo.y=y;memo.flags=flags;memo.value=rules.control_intent(x,y,right,left,up,down,suppress);}
        return memo.value;
    }
    void update_aim_pose(epok::Fixed dt,bool free_aim);
    void update_lock_marker();
    bool update_attack(bool jumped,const epok::SpatialQueryBatch* queries=nullptr);
public:
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonGame",Id="1359b91e-8fc0-5807-9b5b-035122cc7a35") epok::ObjectId game_component{};
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D",Id="09c9c06e-1cc8-5189-a8d1-25021fc025da") epok::ObjectId lock_marker{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="DungeonPlayerCombatBase",Id="d0a5fd80-f263-5ff5-9307-008fafb4e372") epok::ObjectId combat_policy{};
    DungeonPlayerCombatBase* combat_rules(){return combat_binding_.resolve(combat_policy);}
    EPOK_PROPERTY(EditAnywhere,ActorClass="DungeonCamera",Id="ae45b0c0-764c-56bd-b06f-af8c3f5205df") epok::ObjectId camera_rig{};
    EPOK_PROPERTY(EditAnywhere, AssetKind=DataTable, Id="19a34c22-5e48-42df-825f-f8535619e834")
    uint64_t sound_effects=0;
    void reset_player(){if(auto* a=get_owner())if(auto* d=a->data())start(d->transform);}
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("78439d2a-85ff-5a98-8f38-23293bcf9a59");
    uint64_t class_id() const override{return static_class_id;}
    EPOK_PROPERTY(EditAnywhere, Id="6d24c0f8-8b57-4bc8-a2b3-4af3ac847214")
    epok::Fixed shoulder_camera_distance=2.15;
    EPOK_PROPERTY(EditAnywhere, Id="ca18323c-971e-4dc6-9292-e3caf1fb0220")
    bool use_aqua_visual=false;
    EPOK_PROPERTY(EditAnywhere, Id="e3df2c09-538c-408e-bc42-c4635f6d18e9")
    uint32_t muzzle_vertex=162;
    void begin_play() override{if(auto* actor=get_owner())if(auto* data=actor->data())start(data->transform);}
    void tick(epok::Fixed dt) override{if(auto* actor=get_owner())if(auto* data=actor->data())update(data->transform,dt);}
};
