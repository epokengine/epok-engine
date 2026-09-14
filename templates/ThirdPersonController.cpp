#include "ThirdPersonController.hpp"
// Only for sin_degrees/cos_degrees. They are pure functions over a shared
// quarter-wave table; --gc-sections drops the rest of the 2D runtime.
#include "world2d.hpp"
using namespace epok;

namespace {
// Template proportions, expressed in Epok units and degrees per second.
// The character is about two units tall, so a walk speed of six units is a brisk
// run across the arena rather than a crawl.
constexpr Fixed walk_speed = 6.0;
constexpr Fixed turn_rate = 540.0;   // character yaw rate
constexpr Fixed orbit_rate = 140.0;  // camera yaw, held on the shoulder buttons
constexpr Fixed pitch_rate = 70.0;
constexpr Fixed pitch_min = -10.0, pitch_max = 55.0;
constexpr Fixed boom_length = 8.0;   // camera boom length
constexpr Fixed boom_height = 2.0;
constexpr Fixed gravity = 24.0;
constexpr Fixed jump_speed = 9.0;
// Air control keeps input steering active, but weakly.
constexpr Fixed air_control = 0.35;
// Downward bias that keeps a standing character in contact with the floor.
constexpr Fixed stick_to_ground = -0.05;
// The camera boom is manually controlled and never turns automatically. A
// digital pad has no analog stick, so the boom drifts behind the character while
// it runs away from the camera. Only then: aligning on a sideways press would
// rotate the frame the input is measured in and send the character circling.
constexpr Fixed follow_rate = 90.0;
// Tallest ledge the character walks onto without jumping. Every authored ramp
// step rises less than this.
constexpr Fixed step_height = 0.6;
// How quickly horizontal velocity chases the input, per second. Braking falls
// out of the same term when the stick is released, without a second constant.
constexpr Fixed ground_response = 12.0;
constexpr Fixed air_response = 3.0;
// Diagonals would otherwise travel sqrt(2) times faster. One constant multiply
// is cheaper than normalising, and with a digital pad it is exact.
constexpr Fixed diagonal = 0.7071;

// Eight headings relative to the camera, indexed by (forward+1)*3+(strafe+1).
// This is what replaces the arc tangent: the pad cannot express any other
// direction, so the answer is always one of these.
// Stored as raw Q12 so the lookup needs no conversion: Fixed's constructor from
// a literal is consteval and cannot take a runtime index.
constexpr int32_t heading[9] = {
    225 * 4096, 180 * 4096, 135 * 4096,   // pulling back: back-left, back, back-right
    270 * 4096,          0,  90 * 4096,   // no forward input: left, unused, right
    315 * 4096,          0,  45 * 4096,   // pushing forward: front-left, forward, front-right
};

// Shortest signed way round from one heading to another, in degrees.
Fixed shortest_turn(Fixed from, Fixed to) {
    constexpr int32_t turn = 360 * 4096;
    int32_t delta = (to.raw() - from.raw()) % turn;
    if (delta > turn / 2) delta -= turn;
    else if (delta < -turn / 2) delta += turn;
    return Fixed(delta, Fixed::RAW);
}
}  // namespace

void ThirdPersonController::start(Transform& transform) {
    facing = transform.rotation[1];
    camera_yaw = facing;
    for (auto& component : velocity) component = 0.0;
    if (auto* camera = find_actor_data("Follow Camera")) set_active_camera(camera);
}

void ThirdPersonController::update(Transform& transform, Fixed dt) {
    // ---- camera boom ------------------------------------------------------
    // The boom owns the yaw; the character never inherits it. Holding a shoulder
    // button orbits, which is the digital stand-in for an analog right stick.
    if (input.held(Button::L1)) camera_yaw -= orbit_rate * dt;
    if (input.held(Button::R1)) camera_yaw += orbit_rate * dt;
    if (input.held(Button::L2)) camera_pitch -= pitch_rate * dt;
    if (input.held(Button::R2)) camera_pitch += pitch_rate * dt;
    if (camera_pitch.raw() < pitch_min.raw()) camera_pitch = pitch_min;
    if (camera_pitch.raw() > pitch_max.raw()) camera_pitch = pitch_max;

    // One pair of trig calls per frame serves both the movement frame and the
    // boom placement. Epok's rotation convention is forward = (sin, 0, cos).
    const Fixed yaw_sin = sin_degrees(camera_yaw), yaw_cos = cos_degrees(camera_yaw);

    // ---- movement input, in the camera's frame ----------------------------
    int32_t forward = 0, strafe = 0;
    if (input.held(Button::Up)) ++forward;
    if (input.held(Button::Down)) --forward;
    if (input.held(Button::Right)) ++strafe;
    if (input.held(Button::Left)) --strafe;
    const bool moving = forward != 0 || strafe != 0;

    Fixed desired_x = 0.0, desired_z = 0.0;
    if (moving) {
        // forward = (sin, 0, cos), right = (cos, 0, -sin).
        Fixed dx = yaw_sin * forward + yaw_cos * strafe;
        Fixed dz = yaw_cos * forward - yaw_sin * strafe;
        const Fixed speed = (forward != 0 && strafe != 0) ? walk_speed * diagonal : walk_speed;
        desired_x = dx * speed;
        desired_z = dz * speed;

        // Turn toward the heading at a limited rate. The character keeps moving
        // in the input direction while the body catches up.
        const Fixed target = camera_yaw + Fixed(heading[(forward + 1) * 3 + (strafe + 1)], Fixed::RAW);
        const Fixed delta = shortest_turn(facing, target);
        const Fixed step = turn_rate * dt;
        if (delta.raw() > step.raw()) facing += step;
        else if (delta.raw() < -step.raw()) facing -= step;
        else facing = target;
    }

    // ---- horizontal velocity ---------------------------------------------
    // An exponential approach: acceleration when there is input, braking when
    // there is not, and air control by using a weaker response off the ground.
    Fixed response = (grounded ? ground_response : air_response) * dt;
    if (response.raw() > 4096) response = 1.0;
    if (!grounded) { desired_x = desired_x * air_control; desired_z = desired_z * air_control; }
    velocity[0] += (desired_x - velocity[0]) * response;
    velocity[2] += (desired_z - velocity[2]) * response;

    // ---- gravity and jump --------------------------------------------------
    // move_and_slide reports contact, so the ground state is last frame's answer.
    // That is one sweep per frame instead of a separate downward probe.
    bool jumping = false;
    if (grounded) {
        velocity[1] = 0.0;
        if (input.pressed(Button::Cross)) { velocity[1] = jump_speed; jumping = true; }
    } else {
        velocity[1] -= gravity * dt;
    }
    // Standing still resolves to no vertical motion, and a sweep that never
    // touches the floor reports no contact. A small bias keeps the character
    // pressed against it without being visible.
    Fixed fall = velocity[1] * dt;
    if (grounded && !jumping) fall = stick_to_ground;

    const Fixed movement[3] = {velocity[0] * dt, fall, velocity[2] * dt};
    const auto result = move_and_slide(*get_owner()->data(), movement);
    grounded = result.grounded;

    // Step up. Collision resolves boxes only, so a ramp is authored as steps and
    // the character lifts over the ones it walked into: raise, finish the move
    // that was refused, then settle back down onto whatever is underneath.
    if (result.blocked && grounded) {
        const Fixed remaining[3] = {movement[0] - result.displacement[0], 0.0,
                                    movement[2] - result.displacement[2]};
        if (remaining[0].raw() != 0 || remaining[2].raw() != 0) {
            const Fixed lift[3] = {0.0, step_height, 0.0};
            move_and_slide(*get_owner()->data(), lift);
            move_and_slide(*get_owner()->data(), remaining);
            const Fixed settle[3] = {0.0, -step_height + stick_to_ground, 0.0};
            grounded = move_and_slide(*get_owner()->data(), settle).grounded;
        }
    }
    transform.rotation[1] = facing;

    // ---- let the boom drift behind the character ---------------------------
    if (moving && forward > 0 && !input.held(Button::L1) && !input.held(Button::R1)) {
        const Fixed delta = shortest_turn(camera_yaw, facing);
        const Fixed step = follow_rate * dt;
        if (delta.raw() > step.raw()) camera_yaw += step;
        else if (delta.raw() < -step.raw()) camera_yaw -= step;
        else camera_yaw = facing;
    }

    // ---- place the boom ----------------------------------------------------
    if (auto* camera = find_actor_data("Follow Camera")) {
        const Fixed reach = boom_length * cos_degrees(camera_pitch);
        const Fixed lift = boom_length * sin_degrees(camera_pitch);
        camera->transform.position[0] = transform.position[0] - yaw_sin * reach;
        camera->transform.position[1] = transform.position[1] + boom_height + lift;
        camera->transform.position[2] = transform.position[2] - yaw_cos * reach;
        camera->transform.rotation[0] = camera_pitch;
        camera->transform.rotation[1] = camera_yaw;
        camera->transform.rotation[2] = 0.0;
    }
}
