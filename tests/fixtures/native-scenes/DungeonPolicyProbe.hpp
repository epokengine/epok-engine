#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t dungeon_policy_probe[8]={};}
class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D,Id="795a13bd-6614-5b87-a537-a1a658aa2edf") DungeonPolicyProbe:public epok::ActorComponent {
    uint32_t frame=0,failures=0,hidden=0,ownership=0,once=0,reset=0;
public:
    static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor") epok::ObjectId player{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::PickupComponent") epok::ObjectId pickup{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::GameplayFlagComponent") epok::ObjectId flag{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::DoorComponent") epok::ObjectId door{};
    void tick(epok::Fixed)override {
        if(frame>7)return;
        using H=epok::HelperResult;using S=epok::DoorState;
        auto* registry=epok::active_object_registry;
        auto* item=registry?registry->resolve<epok::PickupComponent>(pickup):nullptr;
        auto* owned=registry?registry->resolve<epok::GameplayFlagComponent>(flag):nullptr;
        auto* gate=registry?registry->resolve<epok::DoorComponent>(door):nullptr;
        auto* key_actor=item?item->get_owner():nullptr;auto* door_actor=gate?gate->get_owner():nullptr;
        if(!item||!owned||!gate||!key_actor||!door_actor){failures|=1;++frame;return;}
        auto check=[&](bool value,uint32_t bit){if(!value)failures|=bit;};
        if(frame==0){
            check(item->auto_collect&&item->accepted_initiator==player&&item->collected_flag==flag&&gate->requirement_flag==flag,2);
            check(!owned->value()&&!item->is_collected(),4);
            item->range_3d=100.0;item->range_2d=20000.0;
            key_actor->set_active(false);
        }else if(frame==2){
            hidden=!key_actor->active()&&!owned->value()&&gate->state()==S::Locked&&door_actor->active();check(hidden,8);
            check(owned->set_value(true)==H::Applied,16);
        }else if(frame==3){
            ownership=gate->state()==S::Open&&!door_actor->active()&&!item->is_collected()&&owned->value();check(ownership,32);
            owned->set_value(false);key_actor->set_active(true);item->reset_collection();
            door_actor->set_active(true);gate->reset_door();
        }else if(frame==4){
            check(gate->state()==S::Locked&&door_actor->active()&&!owned->value(),64);
            check(item->collect(door_actor->id(),0)==H::InvalidInitiator&&!item->is_collected()&&!owned->value(),128);
            check(item->collect(player,0)==H::Applied&&item->is_collected()&&owned->value()&&!key_actor->active(),256);
        }else if(frame==5){
            check(gate->state()==S::Open&&!door_actor->active(),512);
            key_actor->set_active(true);
            once=item->collect(player,0)==H::AlreadyCollected&&key_actor->active()&&owned->value();check(once,1024);
            owned->set_value(false);item->reset_collection();door_actor->set_active(true);gate->reset_door();
        }else if(frame==6){
            check(gate->state()==S::Locked&&door_actor->active()&&!owned->value(),2048);
            reset=item->collect(player,0)==H::Applied&&item->is_collected()&&owned->value()&&!key_actor->active();check(reset,4096);
        }else if(frame==7){
            check(gate->state()==S::Open&&!door_actor->active(),8192);
            auto* p=dungeon_policy_probe;p[0]=DUNGEON_POLICY_MAGIC+EPOK_LUA_MODE;p[1]=8;p[2]=failures;p[3]=hidden;p[4]=ownership;p[5]=once;p[6]=reset;
            p[7]=epok::object_storage::hooks.statistics?epok::object_storage::hooks.statistics->rejected:~0u;
            // Native Play exposes actor transforms through its normal snapshot.
            if(auto* observer=registry->resolve<epok::Actor3D>(owner_id()))if(auto* data=observer->data()){
                data->transform.rotation[0]=epok::Fixed(int32_t(failures*4096),epok::Fixed::RAW);
                data->transform.rotation[1]=epok::Fixed(int32_t((hidden+ownership+once+reset)*4096),epok::Fixed::RAW);
            }
        }
        ++frame;
    }
};
