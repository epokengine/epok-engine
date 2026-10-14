#pragma once
#include "epok.hpp"
#include "DungeonKeyRing.hpp"

// A locked dungeon door. Its collider blocks the doorway until the key with the same
// lock number is picked up; then its actor deactivates, which hides it and removes the
// collider. The Dungeon Generator sets `lock`.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="d44f897a-e1f7-4330-9b1e-a5706ec4c032") DungeonDoor : public epok::ActorComponent {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t lock = 0;

    void tick(epok::Fixed delta_seconds) override {
        (void)delta_seconds;
        if (lock == 0 || !(dungeon::key_ring & dungeon::key_bit(lock))) return;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
