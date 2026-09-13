#include "Behaviour001.hpp"

void Behaviour001::start(epok::Transform&) {}

void Behaviour001::update(epok::Transform& transform, epok::Fixed dt) {
    transform.rotation[1] += speed * dt;
    if (transform.rotation[1] >= 360.0) transform.rotation[1] -= 360.0;
}
