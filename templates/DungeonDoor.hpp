#pragma once
#include "epok.hpp"

// Project policy requests opening from explicit ownership. The shared door
// commits state and collision before this project's visual callback.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="d44f897a-e1f7-4330-9b1e-a5706ec4c032") DungeonDoor : public epok::DoorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock_id = 0;

    void tick(epok::Fixed delta_seconds) override {
        if(requirement_flag.valid()&&requirement_met()) open(owner_id());
        epok::DoorComponent::tick(delta_seconds);
    }
    void door_changed(epok::DoorState previous,epok::DoorState current) override {
        (void)previous;
        if(current==epok::DoorState::Open)
            if(auto* owner=get_owner()) owner->set_active(false);
    }
};
