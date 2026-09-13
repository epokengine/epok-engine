#pragma once
#include "epok.hpp"

class Spinner final : public epok::Behaviour {
public:
    epok::Fixed speed = 90.0;
    void start(epok::Transform& transform) override;
    void update(epok::Transform& transform, epok::Fixed dt) override;
};
