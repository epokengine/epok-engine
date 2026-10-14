#pragma once
#include "epok.hpp"

// A dungeon key. The first time its `player` enters its trigger it is picked up: its
// actor deactivates, which hides it and removes its collider, and every door whose
// `key` is this actor opens. While `player` names no live actor, anything entering
// picks it up. The Dungeon Generator sets `lock`, makes its Player the `player` and
// points the doors at their key. The Blueprint and Lua flavors behave the same way.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="9dcbb5be-b1da-4a00-a81c-a46a332f5d89") DungeonKey : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;
    EPOK_PROPERTY(EditAnywhere) epok::ObjectId player{};

    void trigger_event(epok::ObjectId other, epok::TriggerPhase phase) override {
        if (phase != epok::TriggerPhase::Enter) return;
        auto* registry = epok::active_object_registry;
        const bool named = registry && registry->resolve<epok::Actor>(player);
        if (named && other != player) return;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
