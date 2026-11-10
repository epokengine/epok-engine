#pragma once
#include "epok.hpp"

// Exercise the project's actual pickup callback, not a substitute coin rule.
// The owner supplies an independent initiator and carries observable results.
class EPOK_CLASS(Blueprintable,Owners=World2D,Id="061e2973-0830-5447-a720-172e7d7bfe8f") PlatformerPickupPolicyProbe:public epok::ActorComponent {
    bool checked_=false;
public:
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor2D") epok::ObjectId player{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::PickupComponent") epok::ObjectId pickup{};
    void tick(epok::Fixed)override{
        if(checked_)return;checked_=true;
        using H=epok::HelperResult;using P=epok::TriggerPhase;
        auto* registry=epok::active_object_registry;
        auto* item=registry?registry->resolve<epok::PickupComponent>(pickup):nullptr;
        auto* actor=registry?registry->resolve<epok::Actor2D>(player):nullptr;
        auto* marker=static_cast<epok::Actor2D*>(get_owner());
        unsigned passed=0,total=0;
        auto check=[&](bool good){++total;if(good)++passed;};
        check(item&&actor);
        if(item&&actor){
            auto* coin=static_cast<epok::Actor2D*>(item->get_owner());
            actor->root.set_position(coin->root.position_x(),coin->root.position_y());
            check(item->auto_collect&&item->accepted_initiator==player);
            item->trigger_event(marker->id(),P::Enter);
            check(!item->is_collected()&&epok::is_active(coin));
            check(item->collect(marker->id(),0)==H::InvalidInitiator);
            item->trigger_event(player,P::Stay);
            check(!item->is_collected()&&epok::is_active(coin));
            item->trigger_event(player,P::Enter);
            check(item->is_collected()&&!epok::is_active(coin));
            coin->set_active(true);
            check(item->collect(player,0)==H::AlreadyCollected&&epok::is_active(coin));
            check(item->reset_collection()&&!item->is_collected());
            check(item->collect(player,0)==H::Applied&&item->is_collected()&&!epok::is_active(coin));
            coin->set_active(true);
            check(item->reset_collection());
            item->trigger_event(player,P::Enter);
            check(item->is_collected()&&!epok::is_active(coin));
        }
        const auto x=epok::Fixed(int32_t(passed)*4096,epok::Fixed::RAW);
        const auto y=epok::Fixed(int32_t(total)*4096,epok::Fixed::RAW);
        marker->root.set_position(x,y);
        marker->data()->transform.position[0]=x;
        marker->data()->transform.position[1]=y;
    }
};
