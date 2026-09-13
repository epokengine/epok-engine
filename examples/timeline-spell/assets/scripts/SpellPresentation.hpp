#pragma once
#include "epok.hpp"

// Presentation state only. The combat Blueprint applies damage at the Impact marker.
class EPOK_CLASS(Blueprintable, Id="2c740650-fce4-4f2e-9f04-d72fcf58a751") SpellPresentation : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere, TimelineAnimatable, Id="82b3be44-fc06-43f7-ae08-6e73ec1ec8e4")
    epok::Fixed charge = 0.0;
    EPOK_FUNCTION(TimelineAction, Id="ac795ac1-d358-4d10-970f-0be97d6f1f10")
    void set_charge(epok::Fixed value) { charge = value; }
    void update(epok::Transform&, epok::Fixed) override {}
};
