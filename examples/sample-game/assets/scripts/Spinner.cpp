#include "Spinner.hpp"

void Spinner::start(epok::Transform&) {}

void Spinner::update(epok::Transform& transform, epok::Fixed dt) {
    transform.rotation[1] += speed * dt;
    if (transform.rotation[1] >= 360.0) transform.rotation[1] -= 360.0;
    if (transform.rotation[1] < 0.0) transform.rotation[1] += 360.0;
}
