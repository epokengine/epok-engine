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
class ThirdPersonController final : public epok::Behaviour {
    epok::Fixed camera_yaw = 0.0;    // boom orbit around the character, degrees
    epok::Fixed camera_pitch = 20.0; // boom elevation, degrees
    epok::Fixed facing = 0.0;        // character heading, chases the input
    epok::Fixed velocity[3] = {0.0, 0.0, 0.0};
    bool grounded = false;

public:
    void start(epok::Transform& transform) override;
    void update(epok::Transform& transform, epok::Fixed dt) override;
};
