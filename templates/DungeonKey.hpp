#pragma once
#include "epok.hpp"
#include "DungeonKeyRing.hpp"

// A dungeon key. The first time something enters its trigger it is picked up: it joins
// the key ring and its actor deactivates, which hides it and removes its collider.
// Doors with the same lock number then open. The Dungeon Generator sets `lock`.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="9dcbb5be-b1da-4a00-a81c-a46a332f5d89") DungeonKey : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;
    // Degrees per second the key turns around its vertical axis (3D).
    EPOK_PROPERTY(EditAnywhere) epok::Fixed spin = 120.0;

    void begin_play() override { dungeon::key_ring &= ~dungeon::key_bit(lock); }

    void tick(epok::Fixed delta_seconds) override {
        auto* owner = get_owner();
        auto* root = owner && epok::active_object_registry
            ? epok::active_object_registry->resolve<epok::SceneComponent3D>(owner->root_id())
            : nullptr;
        if (root && root->transform) root->transform->rotation[1] += spin * delta_seconds;
    }

    void trigger_event(epok::ObjectId other, epok::TriggerPhase phase) override {
        (void)other;
        if (phase != epok::TriggerPhase::Enter || lock == 0) return;
        dungeon::key_ring |= dungeon::key_bit(lock);
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
