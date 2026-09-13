#pragma once
#include "epok.hpp"

// Presentation harness. The combat Blueprint owns marker-driven damage.
class EPOK_CLASS(Blueprintable,Id="07991ca7-4517-52b6-9dc0-7df8df729a23") FireballShowcase:public epok::Behaviour {
    epok::Fixed delay=0.0;
public:
    void update(epok::Transform&,epok::Fixed)override;
};
