#pragma once
#define EPOK_INCLUDE_FROM_WORLD_GAMEPLAY 1
#include "interaction.hpp"
#undef EPOK_INCLUDE_FROM_WORLD_GAMEPLAY
#include "projectile3d.hpp"
#include "device_io.hpp"

namespace epok {
enum class HelperResult : uint8_t {
    Applied, Queued, Unchanged, AlreadyCollected, Unavailable, InvalidInitiator,
    MissingBinding, InvalidOwner, OutOfRange, Busy, RequirementUnmet, Locked, DomainMismatch
};
enum class DoorState : uint8_t { Closed, Open, Locked };
namespace world_gameplay {
inline bool alive(ObjectId id) {
    auto* value=id.get();
    if(!value||value->state()==ObjectState::EndingPlay||value->state()==ObjectState::Destroyed)return false;
    auto* actor=interaction::actor(id);return !actor||!actor->pending_destruction();
}
inline HelperResult interaction_result(InteractionResult value) {
    switch(value){
    case InteractionResult::Accepted:return HelperResult::Applied;
    case InteractionResult::OutOfRange:return HelperResult::OutOfRange;
    case InteractionResult::Busy:return HelperResult::Busy;
    case InteractionResult::MissingBinding:return HelperResult::MissingBinding;
    case InteractionResult::InvalidOwner:return HelperResult::InvalidOwner;
    case InteractionResult::DomainMismatch:return HelperResult::DomainMismatch;
    default:return HelperResult::Unavailable;
    }
}
inline bool accepted(ObjectId source,ObjectId required,bool mandatory) {
    auto* actor=interaction::actor(source);
    return actor&&interaction::live(actor->id())&&(!mandatory||required.valid())&&(!required.valid()||actor->id()==required);
}
}

// Explicit state can be read while its owner is inactive. Destruction still
// invalidates the typed reference; availability never depends on actor visibility.
class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D,Capability=world-gameplay,Id="397c4a8e-a044-59b5-ab51-a54d214f1789") GameplayFlagComponent:public ActorComponent {
    bool value_=false,changing_=false;
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("397c4a8e-a044-59b5-ab51-a54d214f1789");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) bool initial_value=false;
    EPOK_FUNCTION(BlueprintPure) bool value()const{return value_;}
    EPOK_FUNCTION(BlueprintPure) bool changing()const{return changing_;}
    EPOK_FUNCTION(BlueprintEvent) virtual void value_changed(bool previous,bool current){(void)previous;(void)current;}
    EPOK_FUNCTION(BlueprintCallable) HelperResult set_value(bool value){
        if(!active_object_registry||!world_gameplay::alive(id()))return HelperResult::InvalidOwner;
        if(changing_)return HelperResult::Busy;
        if(value_==value)return HelperResult::Unchanged;
        ObjectDispatchScope scope(*active_object_registry);
        const bool previous=value_;value_=value;changing_=true;value_changed(previous,value);changing_=false;
        return HelperResult::Applied;
    }
    void begin_play()override{value_=initial_value;changing_=false;}
};
namespace world_gameplay {
inline GameplayFlagComponent* flag(const ActorComponent& owner,ObjectId id){
    auto* result=active_object_registry?active_object_registry->resolve<GameplayFlagComponent>(id):nullptr;
    auto* receiver=const_cast<ActorComponent&>(owner).get_owner();auto* source=result?result->get_owner():nullptr;
    return source&&receiver&&alive(id)&&source->level_id()==receiver->level_id()?result:nullptr;
}
inline bool flag_ready(const ActorComponent& owner,ObjectId id){
    auto* value=flag(owner,id);return !id.valid()||(value&&!value->changing());
}
}

class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D,Capability=world-gameplay,Id="117a8e4b-4df8-53da-b42e-58be2f3cae4f") PickupComponent:public InteractableComponent {
    bool collected_=false,committing_=false;
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("117a8e4b-4df8-53da-b42e-58be2f3cae4f");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor") ObjectId accepted_initiator{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::GameplayFlagComponent") ObjectId collected_flag{};
    EPOK_PROPERTY(EditAnywhere) bool auto_collect=false;
    EPOK_PROPERTY(EditAnywhere) bool deactivate_on_collect=false;
    EPOK_PROPERTY(EditAnywhere) uint32_t input_port=0;
    EPOK_FUNCTION(BlueprintPure) bool is_collected()const{return collected_;}
    EPOK_FUNCTION(BlueprintEvent,BlueprintPure) virtual bool can_collect(ObjectId initiator)const{(void)initiator;return true;}
    EPOK_FUNCTION(BlueprintEvent) virtual void collected(ObjectId initiator,uint32_t port){(void)initiator;(void)port;}
    bool can_interact(ObjectId initiator)const final{
        return !collected_&&!committing_&&world_gameplay::accepted(initiator,accepted_initiator,true)
            &&world_gameplay::flag_ready(*this,collected_flag)&&can_collect(initiator);
    }
    // Keep the once-only state transition native. Projects extend collected().
    void interaction_started(ObjectId initiator,uint32_t port)final{
        if(collected_||committing_||!world_gameplay::accepted(initiator,accepted_initiator,true)||!world_gameplay::flag_ready(*this,collected_flag))return;
        committing_=true;collected_=true;
        if(auto* flag=world_gameplay::flag(*this,collected_flag))flag->set_value(true);
        if(interaction::live(id()))collected(initiator,port);
        if(interaction::live(id())&&deactivate_on_collect)if(auto* owner=get_owner())owner->set_active(false);
        committing_=false;
    }
    EPOK_FUNCTION(BlueprintCallable) HelperResult collect(ObjectId initiator,uint32_t port){
        if(!active_object_registry||!world_gameplay::alive(id()))return HelperResult::InvalidOwner;
        ObjectDispatchScope scope(*active_object_registry);
        if(committing_)return HelperResult::Busy;
        if(collected_)return HelperResult::AlreadyCollected;
        if(!world_gameplay::accepted(initiator,accepted_initiator,true))return HelperResult::InvalidInitiator;
        if(!world_gameplay::flag_ready(*this,collected_flag))return HelperResult::MissingBinding;
        const auto result=request(initiator,port);
        return result==InteractionResult::Accepted&&!collected_?HelperResult::Unavailable:world_gameplay::interaction_result(result);
    }
    EPOK_FUNCTION(BlueprintCallable) bool reset_collection(){if(committing_||is_busy())return false;collected_=false;return true;}
    void trigger_event(ObjectId other,TriggerPhase phase)override{if(auto_collect&&phase==TriggerPhase::Enter)collect(other,input_port);}
    void begin_play()override{collected_=false;committing_=false;}
};

class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D,Capability=world-gameplay,Id="94fdf04c-4d40-5962-a6cc-2626e2bf1b1e") SwitchComponent:public InteractableComponent {
    bool value_=false,changing_=false;
    HelperResult last_change_=HelperResult::Unchanged;
    HelperResult change(bool next,ObjectId source){
        if(changing_)return HelperResult::Busy;
        if(!interaction::live(id()))return HelperResult::InvalidOwner;
        if(!world_gameplay::flag_ready(*this,target_flag))return HelperResult::MissingBinding;
        if(value()==next)return HelperResult::Unchanged;
        ObjectDispatchScope scope(*active_object_registry);
        changing_=true;value_=next;
        if(auto* flag=world_gameplay::flag(*this,target_flag))flag->set_value(next);
        if(interaction::live(id()))switched(source,next);
        changing_=false;return HelperResult::Applied;
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("94fdf04c-4d40-5962-a6cc-2626e2bf1b1e");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor") ObjectId accepted_initiator{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::GameplayFlagComponent") ObjectId target_flag{};
    EPOK_PROPERTY(EditAnywhere) bool initial_value=false;
    EPOK_PROPERTY(EditAnywhere) bool toggle=true;
    EPOK_FUNCTION(BlueprintPure) bool value()const{auto* flag=world_gameplay::flag(*this,target_flag);return flag?flag->value():value_;}
    EPOK_FUNCTION(BlueprintEvent,BlueprintPure) virtual bool can_switch(ObjectId initiator)const{(void)initiator;return true;}
    EPOK_FUNCTION(BlueprintEvent) virtual void switched(ObjectId initiator,bool value){(void)initiator;(void)value;}
    bool can_interact(ObjectId initiator)const final{return !changing_&&world_gameplay::accepted(initiator,accepted_initiator,false)&&world_gameplay::flag_ready(*this,target_flag)&&can_switch(initiator);}
    void interaction_started(ObjectId initiator,uint32_t)final{last_change_=change(toggle?!value():true,initiator);}
    EPOK_FUNCTION(BlueprintCallable) HelperResult activate(ObjectId initiator,uint32_t port){
        if(!active_object_registry||!world_gameplay::alive(id()))return HelperResult::InvalidOwner;
        ObjectDispatchScope scope(*active_object_registry);
        const auto result=request(initiator,port);return result==InteractionResult::Accepted?last_change_:world_gameplay::interaction_result(result);
    }
    EPOK_FUNCTION(BlueprintCallable) HelperResult reset_switch(){return change(initial_value,{});}
    void begin_play()override{value_=initial_value;changing_=false;}
};

class EPOK_CLASS(Blueprintable,Domain=None,Owners=World3D|World2D,Capability=world-gameplay,Id="fd68c79f-403a-55f2-8f8f-2ef861166923") DoorComponent:public InteractableComponent {
    DoorState state_=DoorState::Closed,desired_=DoorState::Closed;
    HelperResult result_=HelperResult::Unchanged;
    ObjectId applied_{},initiator_{};
    bool pending_=false,original_enabled_=true,changing_=false,reset_pending_=false;
    ActorComponent* blocker()const{
        auto* self=const_cast<DoorComponent*>(this);auto* owner=self->get_owner();
        auto* level=owner&&active_object_registry?active_object_registry->resolve<Level>(owner->level_id()):nullptr;
        if(!owner||!level)return nullptr;
        ActorComponent* value=nullptr;
        if(interaction::domain(owner)==ObjectDomain::World3D)value=collider_3d.valid()?active_object_registry->resolve<Collider3DComponent>(collider_3d):level->get_component<Collider3DComponent>(*owner);
        else if(interaction::domain(owner)==ObjectDomain::World2D)value=collider_2d.valid()?active_object_registry->resolve<Collider2DComponent>(collider_2d):level->get_component<Collider2DComponent>(*owner);
        auto* receiver=value?value->get_owner():nullptr;
        return receiver&&world_gameplay::alive(value->id())&&receiver->level_id()==owner->level_id()?value:nullptr;
    }
    static void collision(ObjectId id,bool enabled){
        if(!active_object_registry)return;
        if(auto* shape=active_object_registry->resolve<Collider3DComponent>(id))shape->set_enabled(enabled);
        if(auto* shape=active_object_registry->resolve<Collider2DComponent>(id))shape->set_enabled(enabled);
    }
    void restore(){collision(applied_,original_enabled_);applied_={};}
    bool bind_collision(){
        auto* shape=blocker();if(!shape)return false;
        if(shape->is_a(Collider3DComponent::static_class_id)&&!static_cast<Collider3DComponent*>(shape)->entity_slot())return false;
        if(shape->id()!=applied_){restore();applied_=shape->id();
            original_enabled_=shape->is_a(Collider3DComponent::static_class_id)?static_cast<Collider3DComponent*>(shape)->enabled():static_cast<Collider2DComponent*>(shape)->is_enabled();}
        collision(applied_,state_!=DoorState::Open);return true;
    }
    HelperResult requirement_status(ObjectId initiator)const{
        if(initiator.valid()&&!world_gameplay::accepted(initiator,{},false))return HelperResult::InvalidInitiator;
        auto* flag=world_gameplay::flag(*this,requirement_flag);
        if(requirement_flag.valid()&&!flag)return HelperResult::MissingBinding;
        if(flag&&!flag->value())return HelperResult::RequirementUnmet;
        if(state_==DoorState::Locked&&!flag)return HelperResult::Locked;
        return HelperResult::Applied;
    }
    HelperResult open_requirement(ObjectId initiator)const{
        const auto before=requirement_status(initiator);if(before!=HelperResult::Applied)return before;
        const bool allowed=can_open(initiator);
        const auto after=requirement_status(initiator);if(after!=HelperResult::Applied)return after;
        return allowed?HelperResult::Applied:HelperResult::Unavailable;
    }
    HelperResult queue(DoorState state,ObjectId initiator,bool reset=false){
        if(changing_)return HelperResult::Busy;
        if(!interaction::live(id()))return result_=HelperResult::InvalidOwner;
        if(!enabled)return result_=HelperResult::Unavailable;
        if(!blocker())return result_=HelperResult::MissingBinding;
        ObjectDispatchScope scope(*active_object_registry);
        if(state==DoorState::Open&&!reset){changing_=true;const auto result=open_requirement(initiator);changing_=false;
            if(!interaction::live(id()))return result_=HelperResult::InvalidOwner;
            if(result!=HelperResult::Applied)return result_=result;}
        if((!pending_&&state_==state)||(pending_&&desired_==state&&reset_pending_==reset))return result_=HelperResult::Unchanged;
        desired_=state;initiator_=initiator;pending_=true;reset_pending_=reset;result_=HelperResult::Queued;return result_;
    }
public:
    static constexpr TickGroup tick_group=TickGroup::Input;
    static constexpr uint64_t static_class_id=detail::compact_class_id("fd68c79f-403a-55f2-8f8f-2ef861166923");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere) DoorState initial_state=DoorState::Closed;
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::GameplayFlagComponent") ObjectId requirement_flag{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Collider3DComponent") ObjectId collider_3d{};
    EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Collider2DComponent") ObjectId collider_2d{};
    EPOK_FUNCTION(BlueprintPure) DoorState state()const{return state_;}
    EPOK_FUNCTION(BlueprintPure) HelperResult last_result()const{return result_;}
    EPOK_FUNCTION(BlueprintPure) bool has_pending_transition()const{return pending_;}
    EPOK_FUNCTION(BlueprintPure) bool requirement_met()const{auto* flag=world_gameplay::flag(*this,requirement_flag);return !requirement_flag.valid()||(flag&&flag->value());}
    EPOK_FUNCTION(BlueprintEvent,BlueprintPure) virtual bool can_open(ObjectId initiator)const{(void)initiator;return true;}
    EPOK_FUNCTION(BlueprintEvent) virtual void door_changed(DoorState previous,DoorState current){(void)previous;(void)current;}
    EPOK_FUNCTION(BlueprintEvent) virtual void door_requested(ObjectId initiator,HelperResult result){(void)initiator;(void)result;}
    EPOK_FUNCTION(BlueprintCallable) HelperResult open(ObjectId initiator){return queue(DoorState::Open,initiator);}
    EPOK_FUNCTION(BlueprintCallable) HelperResult close(){return queue(state_==DoorState::Locked?DoorState::Locked:DoorState::Closed,{});}
    EPOK_FUNCTION(BlueprintCallable) HelperResult lock(){return queue(DoorState::Locked,{});}
    EPOK_FUNCTION(BlueprintCallable) HelperResult unlock(){if(changing_)return HelperResult::Busy;if((pending_?desired_:state_)!=DoorState::Locked)return result_=HelperResult::Unchanged;return queue(DoorState::Closed,{});}
    EPOK_FUNCTION(BlueprintCallable) HelperResult reset_door(){return queue(initial_state,{},true);}
    void interaction_started(ObjectId initiator,uint32_t)override{const auto result=open(initiator);changing_=true;door_requested(initiator,result);changing_=false;}
    void begin_play()override{state_=initial_state;desired_=state_;pending_=false;changing_=false;result_=bind_collision()?HelperResult::Unchanged:HelperResult::MissingBinding;}
    void tick(Fixed)override{
        if(!bind_collision()){pending_=false;result_=HelperResult::MissingBinding;return;}
        if(!pending_||changing_)return;
        ObjectDispatchScope scope(*active_object_registry);
        if(desired_==DoorState::Open&&!reset_pending_){changing_=true;const auto result=open_requirement(initiator_);changing_=false;
            if(!interaction::live(id()))return;
            if(result!=HelperResult::Applied){pending_=false;result_=result;return;}}
        if(!bind_collision()){pending_=false;result_=HelperResult::MissingBinding;return;}
        const auto previous=state_;state_=desired_;pending_=false;collision(applied_,state_!=DoorState::Open);result_=HelperResult::Applied;
        if(previous!=state_){changing_=true;door_changed(previous,state_);changing_=false;}
    }
    void on_disable()override{if(pending_)result_=HelperResult::Unavailable;pending_=false;}
    void end_play(EndPlayReason)override{pending_=false;restore();}
};

class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="be73a9b8-baa1-5790-be17-f3dd20ac2d50") ExaminableObject3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("be73a9b8-baa1-5790-be17-f3dd20ac2d50");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Interaction") InteractableComponent interaction;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="21c4d386-6ea9-548f-920e-3a5696820cd5") ExaminableObject2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("21c4d386-6ea9-548f-920e-3a5696820cd5");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Interaction") InteractableComponent interaction;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="2fa741a6-d51d-5b6b-ba1f-4e7b6df58a9e") Pickup3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("2fa741a6-d51d-5b6b-ba1f-4e7b6df58a9e");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Pickup") PickupComponent pickup;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="f118743d-70a1-5644-bd82-e5b816c80257") Pickup2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("f118743d-70a1-5644-bd82-e5b816c80257");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Pickup") PickupComponent pickup;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="ee1d25d5-a166-5827-9a6d-30221e91f6c6") Door3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("ee1d25d5-a166-5827-9a6d-30221e91f6c6");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Collider") Collider3DComponent collider;
    EPOK_COMPONENT(Name="Door") DoorComponent door;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="f7eb1ccc-0ac9-5c08-a210-7cea6912279b") Door2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("f7eb1ccc-0ac9-5c08-a210-7cea6912279b");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Collider") Collider2DComponent collider;
    EPOK_COMPONENT(Name="Door") DoorComponent door;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World3D,Id="ce9fc943-b001-5eff-aad4-3dbf93721250") Switch3D:public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("ce9fc943-b001-5eff-aad4-3dbf93721250");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Switch") SwitchComponent interaction;
};
class EPOK_CLASS(Blueprintable,Placeable,Spawnable,Domain=World2D,Id="1c070d4c-13a4-52af-9428-548f026c3e9c") Switch2D:public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("1c070d4c-13a4-52af-9428-548f026c3e9c");
    uint64_t class_id()const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Switch") SwitchComponent interaction;
};
}
