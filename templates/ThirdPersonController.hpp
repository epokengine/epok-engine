#pragma once
#include "epok.hpp"
#include "CharacterMotion.hpp"

// Project-owned third-person movement, animation and an obstacle-aware orbit camera.
// Gameplay remains intentionally small: no targeting, attacks, NPCs or audio.
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="9233f481-d27e-4765-a2d3-4dcfcb0cc910") ThirdPersonController : public epok::ActorComponent {
    epok::Fixed camera_yaw=0.0,camera_pitch=18.0,facing=0.0;
    epok::Fixed velocity[3]={0.0,0.0,0.0};
    bool grounded=false;
    character_motion::AnimationState animation;
    epok::DataHandle visual_handle{},camera_handle{};
    int clips[6]={-1,-1,-1,-1,-1,-1};
    uint32_t playback_phase=0;
    character_motion::State playing_state=character_motion::State::Land;
    void start(epok::Transform& transform);
    void update(epok::Transform& transform,epok::Fixed dt);
    void animate(epok::Fixed dt,epok::Fixed speed,bool jumped);
    void place_camera(const epok::Transform& transform);
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("9233f481-d27e-4765-a2d3-4dcfcb0cc910");
    uint64_t class_id() const override{return static_class_id;}
    void begin_play() override{if(auto* actor=get_owner())if(auto* data=actor->data())start(data->transform);}
    void tick(epok::Fixed dt) override{if(auto* actor=get_owner())if(auto* data=actor->data())update(data->transform,dt);}
};
