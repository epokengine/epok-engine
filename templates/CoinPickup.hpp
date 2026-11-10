#pragma once
#include "epok.hpp"

// The shared pickup validates the player and commits collection once. This
// project callback decides the reward/presentation; edit it to add game rules.
class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Id="6755cf22-df21-4f8f-81b5-21c85f6872fa") CoinPickup : public epok::PickupComponent {
public:
    static constexpr uint64_t static_class_id=epok::detail::compact_class_id("6755cf22-df21-4f8f-81b5-21c85f6872fa");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    void collected(epok::ObjectId initiator, uint32_t port) override {
        (void)initiator; (void)port;
        if (auto* owner = get_owner()) owner->set_active(false);
    }
};
