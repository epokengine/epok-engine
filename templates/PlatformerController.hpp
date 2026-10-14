#pragma once
#include "epok.hpp"

// Project-owned side-scrolling movement: run with Left/Right, jump with Cross,
// face the way the player moves and switch between the idle and run animations.
// Velocities are pixels per second; the body slides along floors and walls.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Id="ca7559df-c335-44a1-ae8c-5268c5b68ace") PlatformerController : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) epok::Fixed speed = 96.0;
    EPOK_PROPERTY(EditAnywhere) epok::Fixed jump_speed = 264.0;
    EPOK_PROPERTY(EditAnywhere) epok::Fixed gravity = 720.0;
    EPOK_PROPERTY(EditAnywhere) uint32_t idle_animation = 0;
    EPOK_PROPERTY(EditAnywhere) uint32_t run_animation = 1;

    void tick(epok::Fixed delta_seconds) override {
        auto* body = epok::world2d::find<epok::CharacterBody2DComponent>(get_owner());
        auto* sprite = epok::world2d::find<epok::Sprite2DComponent>(get_owner());
        if (!body) return;
        epok::Fixed move = 0.0;
        if (epok::input.held(epok::Button::Right)) move += speed;
        if (epok::input.held(epok::Button::Left)) move -= speed;
        epok::Fixed fall = body->velocity_y() + gravity * delta_seconds;
        if (body->is_on_floor() && epok::input.pressed(epok::Button::Cross)) fall = -jump_speed;
        body->set_velocity(move, fall);
        body->move_and_slide();
        if (!sprite) return;
        if (move.raw() != 0) {
            sprite->set_flip_h(move < 0.0);
            sprite->play(run_animation);
        } else {
            sprite->play(idle_animation);
        }
    }
};
