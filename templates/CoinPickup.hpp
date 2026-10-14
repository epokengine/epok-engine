#pragma once
#include "epok.hpp"

// A collectible: the first time anything enters its trigger shape, the coin's
// actor deactivates, which hides it and removes its collider.
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Id="6755cf22-df21-4f8f-81b5-21c85f6872fa") CoinPickup : public epok::ActorComponent {
public:
    void trigger_event(epok::ObjectId other, epok::TriggerPhase phase) override {
        (void)other;
        if (phase != epok::TriggerPhase::Enter) return;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
