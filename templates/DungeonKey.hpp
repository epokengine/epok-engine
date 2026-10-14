#pragma once
#include "epok.hpp"

// A dungeon key. The first time something enters its trigger it is picked up: its
// actor deactivates, which hides it and removes its collider, and every door whose
// `key` is this actor opens. The Dungeon Generator sets `lock` and points the doors
// at their key. The Blueprint and Lua flavors behave the same way.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="9dcbb5be-b1da-4a00-a81c-a46a332f5d89") DungeonKey : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;

    void trigger_event(epok::ObjectId other, epok::TriggerPhase phase) override {
        (void)other;
        if (phase != epok::TriggerPhase::Enter) return;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
