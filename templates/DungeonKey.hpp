#pragma once
#include "epok.hpp"

// The shared pickup commits ownership and collection once. This editable
// project callback owns the visual result.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="9dcbb5be-b1da-4a00-a81c-a46a332f5d89") DungeonKey : public epok::PickupComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock_id = 0;

    void collected(epok::ObjectId initiator,uint32_t port) override {
        (void)initiator;(void)port;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
