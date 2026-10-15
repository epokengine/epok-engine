#pragma once
#include "epok.hpp"

// A locked dungeon door. Its collider blocks the doorway until its key has been
// picked up (the key actor is no longer active); then the door's actor deactivates,
// which hides it and removes the collider. The Dungeon Generator sets `lock` and
// `key`. The Blueprint and Lua flavors behave the same way.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="d44f897a-e1f7-4330-9b1e-a5706ec4c032") DungeonDoor : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;
    // The key actor that opens this door.
    EPOK_PROPERTY(EditAnywhere) epok::ObjectId key{};

    void tick(epok::Fixed delta_seconds) override {
        (void)delta_seconds;
        auto* registry = epok::active_object_registry;
        auto* held = registry ? registry->resolve<epok::Actor>(key) : nullptr;
        if (!held || held->active()) return;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
