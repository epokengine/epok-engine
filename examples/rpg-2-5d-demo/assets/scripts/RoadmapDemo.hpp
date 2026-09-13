#pragma once
#include "epok.hpp"
class RoadmapDemo final:public epok::Behaviour {
    epok::EntityHandle temporary;
public:
    void start(epok::Transform&) override;
    void update(epok::Transform&,epok::Fixed) override;
    void frame_update(epok::Transform&,uint32_t) override;
    void on_trigger(epok::EntityHandle,epok::TriggerPhase) override;
};
