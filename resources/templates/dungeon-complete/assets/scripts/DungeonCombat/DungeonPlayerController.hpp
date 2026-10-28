#pragma once
#include "epok.hpp"
#include "CharacterMotion.hpp"

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
    uint8_t visibility_phase=0;
    epok::Fixed velocity[3]={0.0,0.0,0.0};
    bool grounded=false;
    epok::Fixed query_origin[3]{},query_offset[3]{},settled_position[3]{},settled_rotation[3]{},camera_fraction=1.0;
    uint32_t camera_revision=0,movement_revision=0;
    bool camera_hit=false;
    CameraMode camera_mode=CameraMode::Normal;
    character_motion::AnimationState animation;
    epok::DataHandle visual_handle{};
    epok::DataHandle camera_handle{};
    epok::DataHandle target_handle{};
    epok::DataHandle lock_marker_handle{};
    uint8_t lock_marker_intro_frames=0;
    uint8_t lock_marker_spin=0;
    int clips[6]={-1,-1,-1,-1,-1,-1};
    int attack_clips[3]={-1,-1,-1};
    uint8_t attack_phase=0;
    bool attacking=false,shot_sent=false;
    uint32_t playback_phase=0;
    character_motion::State playing_state=character_motion::State::Land;
    void start(epok::Transform& transform);
    void update(epok::Transform& transform,epok::Fixed dt);
    void animate(epok::Fixed dt,epok::Fixed speed,bool jumped);
    void place_camera(const epok::Transform& transform,const epok::SpatialQueryBatch* queries=nullptr);
    void toggle_target(epok::ActorData& body);
    void update_camera(epok::Transform& transform,epok::Fixed dt,bool free_aim,bool moving);
    void update_aim_pose(epok::Fixed dt,bool free_aim);
    void update_lock_marker();
    bool update_attack(bool jumped,const epok::SpatialQueryBatch* queries=nullptr);
public:
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
