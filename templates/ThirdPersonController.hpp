#pragma once
#include "epok.hpp"

// Third-person controller with a camera boom that orbits the character, movement
// expressed in the camera's frame, and a character that turns toward where it is
// travelling rather than toward the camera. This makes the character read as
// "running around" a scene instead of strafing on rails.
//
// Two things differ because the target is a PlayStation. The pad is digital, so
// the eight movement directions are eight constant headings and the controller
// never needs an arc tangent. And every value below is Q12 fixed point: the
// R3000 has no floating point unit, so a float here would be a software call
// per operation in the per-frame path.
class EPOK_CLASS(Blueprintable, Owners=World3D, Id="9233f481-d27e-4765-a2d3-4dcfcb0cc910") ThirdPersonController : public epok::ActorComponent {
    epok::Fixed camera_yaw = 0.0;    // boom orbit around the character, degrees
    epok::Fixed camera_pitch = 20.0; // boom elevation, degrees
    epok::Fixed facing = 0.0;        // character heading, chases the input
    epok::Fixed velocity[3] = {0.0, 0.0, 0.0};
    bool grounded = false;

    void start(epok::Transform& transform);
    void update(epok::Transform& transform, epok::Fixed dt);
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("9233f481-d27e-4765-a2d3-4dcfcb0cc910");
    uint64_t class_id() const override {return static_class_id;}
    void begin_play() override {if(auto* actor=get_owner()) if(auto* data=actor->data()) start(data->transform);}
    void tick(epok::Fixed dt) override {if(auto* actor=get_owner()) if(auto* data=actor->data()) update(data->transform,dt);}
};
