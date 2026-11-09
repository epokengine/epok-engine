#pragma once
#define EPOK_INCLUDE_FROM_INTERACTION 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_INTERACTION
#include "gameplay_control.hpp"
#include "world2d_components.hpp"

namespace epok {
enum class InteractionResult : uint8_t {
    Accepted, Unavailable, OutOfRange, Busy, MissingBinding, InvalidOwner, DomainMismatch
};
class InteractableComponent;
class InteractorComponent;
namespace interaction {
inline bool live(ObjectId id) {
    auto* object=id.get();
    return object&&object->state()!=ObjectState::EndingPlay&&object->state()!=ObjectState::Destroyed&&is_active(object);
}
inline Actor* actor(ObjectId id) {
    if(!active_object_registry)return nullptr;
    if(auto* value=active_object_registry->resolve<Actor>(id))return value;
    auto* component=active_object_registry->resolve<ActorComponent>(id);
    return component?component->get_owner():nullptr;
}
inline ObjectDomain domain(Actor* value) {
    const auto* type=value?find_object_class(value->class_id()):nullptr;
    return type?type->domain:ObjectDomain::None;
}
inline bool position(Actor* value,Fixed* output) {
    if(!value||!active_object_registry)return false;
    const auto kind=domain(value);
    if(kind==ObjectDomain::World2D) {
        auto* root=active_object_registry->resolve<SceneComponent2D>(value->root_id());
        Affine2D matrix;
        if(!root||!world_matrix_2d(*root,[](ObjectId id){return world2d::resolve_scene2d(id);},matrix))return false;
        output[0]=matrix.t[0];output[1]=matrix.t[1];output[2]=0.0;return true;
    }
    if(kind!=ObjectDomain::World3D)return false;
    if(auto* data=value->data()) {
        // An unparented actor's world position is its transform. This runs for
        // every candidate each frame and would otherwise resolve a transform path.
        if(data->parent<0){for(unsigned i=0;i<3;++i)output[i]=data->transform.position[i];return true;}
        const auto sample=gameplay_world_affine(data);
        if(!sample.success)return false;
        for(unsigned i=0;i<3;++i)output[i]=sample.position[i];return true;
    }
    auto* root=active_object_registry->resolve<SceneComponent3D>(value->root_id());
    if(!root||root->attach_parent.valid()||!root->transform)return false;
    for(unsigned i=0;i<3;++i)output[i]=root->transform->position[i];return true;
}
inline uint64_t distance_squared(const Fixed* first,const Fixed* second) {
    uint64_t result=0;
    for(unsigned i=0;i<3;++i) {
        const int64_t delta=int64_t(first[i].raw())-second[i].raw();
        const uint64_t magnitude=uint64_t(delta<0?-delta:delta),square=magnitude*magnitude;
        if(~uint64_t(0)-result<square)return ~uint64_t(0);
        result+=square;
    }
    return result;
}
inline uint64_t radius_squared(Fixed radius) {
    return radius.raw()>0?uint64_t(radius.raw())*uint64_t(radius.raw()):0;
}
inline bool source_live(ObjectId id) {
    if(!id.valid())return true;
    if(!live(id))return false;
    if(auto* shape=active_object_registry->resolve<Collider3DComponent>(id))return shape->enabled()&&shape->trigger();
    if(auto* shape=active_object_registry->resolve<Collider2DComponent>(id))return shape->shape.enabled&&shape->shape.trigger;
    return true;
}
template<class T> struct Link {
    T* previous=nullptr;T* next=nullptr;ObjectRegistry* registry=nullptr;
    Link()=default;Link(const Link&){}
    Link& operator=(const Link&){return *this;}
};
template<class T> struct List {
    T* head=nullptr;
    void add(T* item) {
        auto& link=item->interaction_link;if(link.registry)return;
        link.registry=active_object_registry;link.next=head;
        if(head)head->interaction_link.previous=item;head=item;
    }
    void remove(T* item) {
        auto& link=item->interaction_link;if(!link.registry)return;
        if(link.previous)link.previous->interaction_link.next=link.next;else head=link.next;
        if(link.next)link.next->interaction_link.previous=link.previous;
        link.previous=link.next=nullptr;link.registry=nullptr;
    }
};
inline List<InteractableComponent> targets;
inline List<InteractorComponent> interactors;
void activation_changed();
}

class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Capability=interaction, Id="310e882c-2353-55bc-bba3-f3d54277f671") InteractableComponent : public ActorComponent {
    bool busy_=false,deferred_=false,dispatching_=false;
    mutable bool querying_=false;
    ObjectId initiator_{};uint32_t port_=0;
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("310e882c-2353-55bc-bba3-f3d54277f671");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    interaction::Link<InteractableComponent> interaction_link;
    void level_attached() override{interaction::targets.add(this);}
    ~InteractableComponent() override{interaction::targets.remove(this);}
    EPOK_PROPERTY(EditAnywhere, Id="d1136333-2426-5652-b03b-005a239ac1cb") bool enabled=true;
    EPOK_PROPERTY(EditAnywhere, Id="9d91b029-0855-5081-8d14-bf6d0097dafd") int32_t priority=0;
    EPOK_PROPERTY(EditAnywhere, Id="f655ea07-4844-53a7-b19f-254bf66a56b9") Fixed range_3d=2.0;
    EPOK_PROPERTY(EditAnywhere, Id="92e94aab-4f42-5c2d-8d20-02650befdf3c") Fixed range_2d=64.0;
    EPOK_PROPERTY(EditAnywhere, Id="7c1a1b42-7aee-588b-8901-4ba9c784b601") int32_t prompt_key=-1;
    // The cook assigns the rank of the persistent component UUID unless overridden.
    EPOK_PROPERTY(EditAnywhere, Id="d4c0c3f9-74a6-5e0e-86d1-cb439173efec") uint32_t selection_order=0;
    EPOK_FUNCTION(BlueprintEvent, BlueprintPure, Id="fb9c1c09-be15-5b42-a069-4480e8e389f2") virtual bool can_interact(ObjectId initiator) const {(void)initiator;return true;}
    EPOK_FUNCTION(BlueprintEvent, Id="a7b1cb4c-2980-5724-87e6-b585104c7883") virtual void focus_gained(ObjectId initiator,uint32_t port){(void)initiator;(void)port;}
    EPOK_FUNCTION(BlueprintEvent, Id="4a3aca51-563b-50ca-9926-520cf74b1483") virtual void focus_lost(ObjectId initiator,uint32_t port){(void)initiator;(void)port;}
    EPOK_FUNCTION(BlueprintEvent, Id="e66c2703-4f26-5bfc-b269-2f0c0eefbb8f") virtual void interaction_started(ObjectId initiator,uint32_t port){(void)initiator;(void)port;}
    EPOK_FUNCTION(BlueprintEvent, Id="361ca845-edc0-5814-89c7-af6d8bbb58d3") virtual void interaction_finished(ObjectId initiator,uint32_t port,InteractionResult result){(void)initiator;(void)port;(void)result;}
    EPOK_FUNCTION(BlueprintPure, Id="ae631ec0-6fad-537f-ac69-49a3ee40cc29") bool is_busy() const{return busy_;}
    EPOK_FUNCTION(BlueprintPure, Id="b314ec56-6262-59f6-ba31-ca7464875591") InteractionResult availability(ObjectId initiator) const {
        auto* source=interaction::actor(initiator);auto* owner=const_cast<InteractableComponent*>(this)->get_owner();
        if(!source||!owner||!interaction::live(id())||!interaction::live(source->id()))return InteractionResult::InvalidOwner;
        if(source->level_id()!=owner->level_id()||interaction::domain(source)!=interaction::domain(owner))return InteractionResult::DomainMismatch;
        if(!enabled)return InteractionResult::Unavailable;
        if(busy_||dispatching_||querying_||scene_loading())return InteractionResult::Busy;
        Fixed first[3],second[3];
        if(!interaction::position(source,first)||!interaction::position(owner,second))return InteractionResult::InvalidOwner;
        const Fixed radius=interaction::domain(owner)==ObjectDomain::World2D?range_2d:range_3d;
        if(radius<0.0||interaction::distance_squared(first,second)>interaction::radius_squared(radius))return InteractionResult::OutOfRange;
        ObjectDispatchScope scope(*active_object_registry);
        querying_=true;const bool accepted=can_interact(source->id());querying_=false;
        if(!interaction::live(id())||!interaction::live(initiator))return InteractionResult::InvalidOwner;
        if(!enabled)return InteractionResult::Unavailable;
        if(busy_||dispatching_||scene_loading())return InteractionResult::Busy;
        return accepted?InteractionResult::Accepted:InteractionResult::Unavailable;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="dfc6abc0-52d0-55b4-a531-52bb38943f77") InteractionResult request(ObjectId initiator,uint32_t port) {
        if(!active_object_registry||port>=4)return InteractionResult::InvalidOwner;
        ObjectDispatchScope scope(*active_object_registry);
        const auto result=availability(initiator);if(result!=InteractionResult::Accepted)return result;
        // A condition may call native code that changes owner lifetime.
        if(!interaction::live(id())||!interaction::live(initiator))return InteractionResult::InvalidOwner;
        busy_=true;deferred_=false;dispatching_=true;initiator_=interaction::actor(initiator)->id();port_=port;
        interaction_started(initiator_,port_);
        dispatching_=false;
        if(interaction::live(id())&&busy_&&!deferred_)complete(InteractionResult::Accepted);
        return InteractionResult::Accepted;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="943e95a1-88fa-5f20-89ff-345679998c37") bool hold_open(){if(!busy_)return false;deferred_=true;return true;}
    EPOK_FUNCTION(BlueprintCallable, Id="273f00f0-c9ea-5cb2-9313-41fe7359469c") bool complete(InteractionResult result) {
        if(!busy_||!active_object_registry)return false;
        ObjectDispatchScope scope(*active_object_registry);
        const auto source=initiator_;const auto port=port_;const bool previous=dispatching_;
        busy_=false;deferred_=false;initiator_={};dispatching_=true;
        if(interaction::live(id()))interaction_finished(source,port,result);
        dispatching_=previous;return true;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="7d166263-f599-5f79-8f93-3ba11dc3a70d") void set_enabled(bool value){enabled=value;if(!value)interaction::activation_changed();}
    void prune_lifetime() {
        if(busy_&&(!enabled||!interaction::live(id())||!interaction::live(initiator_)))complete(InteractionResult::InvalidOwner);
    }
};

class EPOK_CLASS(Blueprintable, Domain=None, Owners=World3D|World2D, Capability=interaction, Id="ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a") InteractorComponent : public ActorComponent {
    struct Registration {ObjectId target{},sensor{};};
    struct Candidate {ObjectId target{};uint64_t distance=0;int32_t priority=0;uint32_t order=0;};
    Registration registrations_[16]{};
    Candidate candidates_[8]{};
    ObjectId focused_{};
    uint32_t candidate_count_=0,overflows_=0;
    bool refreshing_=false;
    static InteractableComponent* target(ObjectId value) {
        return active_object_registry?active_object_registry->resolve<InteractableComponent>(value):nullptr;
    }
    static bool before(const Candidate& first,const Candidate& second) {
        if(first.priority!=second.priority)return first.priority>second.priority;
        if(first.distance!=second.distance)return first.distance<second.distance;
        if(first.order!=second.order)return first.order<second.order;
        return first.target.index<second.target.index;
    }
    void change_focus(ObjectId next) {
        if(focused_==next)return;
        const auto previous=focused_;focused_={};
        if(auto* value=target(previous);value&&interaction::live(previous))value->focus_lost(owner_id(),input_port);
        if(!interaction::live(id()))return;
        if(auto* value=target(next);value&&value->availability(owner_id())==InteractionResult::Accepted) {
            focused_=next;value->focus_gained(owner_id(),input_port);
            if(!interaction::live(next))focused_={};
        }
        if(interaction::live(id()))focus_changed(previous,focused_);
    }
    void consider(ObjectId id,const Fixed* position) {
        auto* value=target(id);
        if(!value)return;
        for(unsigned i=0;i<candidate_count_;++i)if(candidates_[i].target==id)return;
        Fixed there[3];if(!interaction::position(value->get_owner(),there))return;
        const auto distance=interaction::distance_squared(position,there);
        const Fixed radius=interaction::domain(get_owner())==ObjectDomain::World2D?range_2d:range_3d;
        if(radius<0.0||distance>interaction::radius_squared(radius))return;
        // Availability is pure, and most targets are simply out of range.
        if(value->availability(owner_id())!=InteractionResult::Accepted||!interaction::live(id))return;
        Candidate candidate{id,distance,value->priority,value->selection_order};
        unsigned index=0;while(index<candidate_count_&&!before(candidate,candidates_[index]))++index;
        if(candidate_count_==8){if(overflows_!=0xffffffffu)++overflows_;if(index==8)return;}
        else ++candidate_count_;
        for(unsigned i=candidate_count_-1;i>index;--i)candidates_[i]=candidates_[i-1];
        candidates_[index]=candidate;
    }
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("ebe82fcd-e9db-5de5-89a7-ed42f3a6d29a");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    interaction::Link<InteractorComponent> interaction_link;
    void level_attached() override{interaction::interactors.add(this);}
    ~InteractorComponent() override{interaction::interactors.remove(this);}
    EPOK_PROPERTY(EditAnywhere, Id="a3414717-11f3-5ec5-9465-7838e3ec7071") bool enabled=true;
    EPOK_PROPERTY(EditAnywhere, Id="23c42819-2651-567a-9f20-1a0eca406b98") bool proximity=true;
    EPOK_PROPERTY(EditAnywhere, Id="340d66bc-cb80-525e-a1d8-13e6aa8f2d6d") bool auto_input=true;
    EPOK_PROPERTY(EditAnywhere, Id="1da2ec70-2899-58dd-af49-71bfe4839c1d") uint32_t input_port=0;
    EPOK_PROPERTY(EditAnywhere, Id="b3b2e706-015b-5c01-8003-c493e41ca875") Fixed range_3d=2.0;
    EPOK_PROPERTY(EditAnywhere, Id="f0351650-67b3-5c34-8ed4-cc9fa7d26082") Fixed range_2d=64.0;
    EPOK_FUNCTION(BlueprintEvent, Id="76ee1381-dacd-51eb-b653-29ecedadfe88") virtual void focus_changed(ObjectId previous,ObjectId current){(void)previous;(void)current;}
    EPOK_FUNCTION(BlueprintPure, Id="10a945cd-00c8-5f25-b8e1-5b875ec37129") ObjectId focused_target() const{
        const auto* value=target(focused_);
        return enabled&&interaction::live(id())&&value&&value->enabled&&!value->is_busy()&&interaction::live(focused_)?focused_:ObjectId{};
    }
    EPOK_FUNCTION(BlueprintPure, Id="503c7d97-3a26-5d7b-8068-f7ab979acd21") uint32_t candidate_count() const{return candidate_count_;}
    EPOK_FUNCTION(BlueprintPure, Id="c071c652-a596-5b0e-a5ae-cc2eb0474b56") uint32_t overflow_count() const{return overflows_;}
    EPOK_FUNCTION(BlueprintCallable, Id="5322fab8-c528-5005-8326-d66202184cfe") bool register_candidate(ObjectId value,ObjectId sensor) {
        auto* item=target(value);auto* owner=get_owner();auto* source=interaction::actor(sensor);
        if(!item||!owner||!interaction::live(value)||!interaction::live(id())||item->owner_id()==owner_id())return false;
        if(sensor.valid()&&(!source||!interaction::source_live(sensor)||source->level_id()!=owner->level_id()))return false;
        if(item->get_owner()->level_id()!=owner->level_id()||interaction::domain(item->get_owner())!=interaction::domain(owner))return false;
        for(auto& entry:registrations_)if(entry.target==value&&entry.sensor==sensor)return true;
        for(auto& entry:registrations_)if(!entry.target.valid()){entry={value,sensor};refresh();return true;}
        if(overflows_!=0xffffffffu)++overflows_;return false;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="b9728a41-f2bf-5bad-bad7-2335fe881e18") bool unregister_candidate(ObjectId value,ObjectId sensor) {
        for(auto& entry:registrations_)if(entry.target==value&&entry.sensor==sensor){entry={};refresh();return true;}return false;
    }
    void forget_sensor(ObjectId sensor) {
        bool changed=false;
        for(auto& entry:registrations_)if(entry.sensor==sensor&&entry.target.valid()){entry={};changed=true;}
        if(changed)refresh();
    }
    EPOK_FUNCTION(BlueprintCallable, Id="ec404c7f-9148-536e-8f77-5a5842f14ae7") void refresh() {
        if(refreshing_||!active_object_registry)return;
        ObjectDispatchScope scope(*active_object_registry);refreshing_=true;candidate_count_=0;
        Fixed here[3];
        if(enabled&&input_port<4&&interaction::live(id())&&interaction::position(get_owner(),here)) {
            if(proximity)for(auto* item=interaction::targets.head;item;item=item->interaction_link.next)
                if(item->interaction_link.registry==active_object_registry&&item->owner_id()!=owner_id())consider(item->id(),here);
            for(auto& entry:registrations_) {
                if(!interaction::live(entry.target)||!interaction::source_live(entry.sensor)){entry={};continue;}
                consider(entry.target,here);
            }
        }
        change_focus(candidate_count_?candidates_[0].target:ObjectId{});refreshing_=false;
    }
    EPOK_FUNCTION(BlueprintCallable, Id="caee042f-9472-5e16-b43c-6b22ba4488a0") InteractionResult interact() {
        if(!enabled||!interaction::live(id())||input_port>=4)return InteractionResult::InvalidOwner;
        ObjectDispatchScope scope(*active_object_registry);
        refresh();if(!interaction::live(id()))return InteractionResult::InvalidOwner;
        auto* value=target(focused_);
        return value?value->request(owner_id(),input_port):InteractionResult::Unavailable;
    }
    // Framework work precedes the extensible on_frame event, including in Lua/BP subclasses.
    void frame_update(uint32_t elapsed) override {
        refresh();
        if(auto_input&&focused_.valid()&&!time.paused()&&!scene_loading()&&
           GameplayInputLibrary::take_frame_pressed(GameplayAction::Interact,input_port))interact();
        if(interaction::live(id()))ActorComponent::frame_update(elapsed);
    }
    void prune_lifetime() {
        for(auto& entry:registrations_)if(entry.target.valid()&&(!interaction::live(entry.target)||!interaction::source_live(entry.sensor)))entry={};
        const auto* value=target(focused_);
        bool registered=false;for(const auto& entry:registrations_)if(entry.target==focused_&&entry.target.valid())registered=true;
        if(!enabled||!interaction::live(id())||!interaction::live(focused_)||!value||!value->enabled||value->is_busy()||(!proximity&&!registered))change_focus({});
    }
};

namespace interaction {
inline void forget_sensor(ObjectId sensor) {
    if(!active_object_registry)return;
    ObjectDispatchScope scope(*active_object_registry);
    for(auto* item=interactors.head;item;item=item->interaction_link.next)
        if(item->interaction_link.registry==active_object_registry)item->forget_sensor(sensor);
}
inline InteractionResult sensor_binding(ActorComponent& sensor,ObjectId target,ObjectId accepted) {
    auto* owner=sensor.get_owner();
    if(!owner||!live(sensor.id()))return InteractionResult::InvalidOwner;
    auto* endpoint=active_object_registry->resolve<InteractableComponent>(target);
    if(!endpoint)return InteractionResult::MissingBinding;
    auto* target_owner=endpoint->get_owner();
    if(!target_owner||!live(target))return InteractionResult::InvalidOwner;
    if(owner->level_id()!=target_owner->level_id()||domain(owner)!=domain(target_owner))return InteractionResult::DomainMismatch;
    if(accepted.valid()) {
        auto* source=actor(accepted);
        if(!source||!live(accepted))return InteractionResult::InvalidOwner;
        if(source->level_id()!=owner->level_id()||domain(source)!=domain(owner))return InteractionResult::DomainMismatch;
    }
    return InteractionResult::Accepted;
}
inline void activation_changed() {
    if(!active_object_registry)return;
    ObjectDispatchScope scope(*active_object_registry);
    for(auto* item=targets.head;item;item=item->interaction_link.next)
        if(item->interaction_link.registry==active_object_registry)item->prune_lifetime();
    for(auto* item=interactors.head;item;item=item->interaction_link.next)
        if(item->interaction_link.registry==active_object_registry)item->prune_lifetime();
}
inline void sensor_event(ActorComponent& sensor,ObjectId target,ObjectId accepted,ObjectId other,TriggerPhase phase) {
    if(!source_live(sensor.id())||sensor_binding(sensor,target,accepted)!=InteractionResult::Accepted)return;
    auto* source=actor(other);auto* owner=sensor.get_owner();
    if(!source||!owner||(accepted.valid()&&source->id()!=accepted)||domain(source)!=domain(owner))return;
    for(size_t i=0;i<source->component_count();++i) {
        auto* interactor=active_object_registry->resolve<InteractorComponent>(source->component_id(i));
        if(!interactor)continue;
        if(phase==TriggerPhase::Exit)interactor->unregister_candidate(target,sensor.id());
        else interactor->register_candidate(target,sensor.id());
    }
}
}

class EPOK_CLASS(Blueprintable, Domain=World3D, Owners=World3D, Capability=interaction, Id="ce7a9a84-40f2-500d-9b51-03de04fbfb8a") InteractionSensor3DComponent : public Collider3DComponent {
    bool configured_=false,last_monitoring_=false;
    ObjectId last_target_{},last_accepted_{};
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("ce7a9a84-40f2-500d-9b51-03de04fbfb8a");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::InteractableComponent", Id="c367ed86-898b-5055-a91d-89aa9c398ffb") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere, ActorClass="epok::Actor3D", Id="5aeff0ba-0311-5d48-ad52-fc5f281309c1") ObjectId accepted_initiator{};
    EPOK_PROPERTY(EditAnywhere, Id="4f613118-9784-5b29-9394-e1cc2367491d") Fixed width=2.0;
    EPOK_PROPERTY(EditAnywhere, Id="5f6c9487-bc5b-55c4-b3ae-fab2464d4028") Fixed height=2.0;
    EPOK_PROPERTY(EditAnywhere, Id="014637ff-6fb0-59b2-bf9d-d2b04169f864") Fixed depth=2.0;
    EPOK_PROPERTY(EditAnywhere, Id="ee1c0a7f-f7c6-5710-9543-278cfe6b24ad") bool monitoring=true;
    EPOK_FUNCTION(BlueprintCallable, Id="405d1e08-809b-538e-b0ab-e2bc5d3f8caf") void set_monitoring(bool value){monitoring=value;configured_=false;configure();}
    EPOK_FUNCTION(BlueprintPure, Id="127335a7-00b2-5469-9078-eff2b05f640d") InteractionResult binding_status() const{return interaction::sensor_binding(*const_cast<InteractionSensor3DComponent*>(this),target,accepted_initiator);}
    void configure() {
        auto* data=entity_slot();if(!data)return;
        const bool changed=last_target_!=target||last_accepted_!=accepted_initiator;
        if(!configured_||last_monitoring_!=monitoring)data->collider.enabled=monitoring;
        configured_=true;last_monitoring_=monitoring;last_target_=target;last_accepted_=accepted_initiator;
        auto& shape=data->collider;shape.trigger=true;shape.slope_rise=0.0;shape.heights=nullptr;shape.boxes=nullptr;shape.box_count=0;
        shape.half_extents[0]=width>0.0?width/2:Fixed(0.0);
        shape.half_extents[1]=height>0.0?height/2:Fixed(0.0);
        shape.half_extents[2]=depth>0.0?depth/2:Fixed(0.0);
        if(!monitoring||width<=0.0||height<=0.0||depth<=0.0)shape.enabled=false;
        if(changed||!shape.enabled)interaction::forget_sensor(id());
    }
    void frame_update(uint32_t elapsed) override {
        configure();
        if(interaction::live(id()))ActorComponent::frame_update(elapsed);
    }
    void trigger_event(ObjectId other,TriggerPhase phase) override{interaction::sensor_event(*this,target,accepted_initiator,other,phase);}
};
class EPOK_CLASS(Blueprintable, Domain=World2D, Owners=World2D, Capability=interaction, Id="eaa0415b-21d7-5659-87b8-d0929c8756fc") InteractionSensor2DComponent : public Collider2DComponent {
    bool configured_=false,last_monitoring_=false;
    ObjectId last_target_{},last_accepted_{};
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("eaa0415b-21d7-5659-87b8-d0929c8756fc");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::InteractableComponent", Id="f30cba9e-17c7-55a4-9dd2-b64e2714a35a") ObjectId target{};
    EPOK_PROPERTY(EditAnywhere, ActorClass="epok::Actor2D", Id="b2219e2a-fad7-5971-b57c-1fc3342804bc") ObjectId accepted_initiator{};
    EPOK_PROPERTY(EditAnywhere, Id="586b3e10-7610-5071-bc4c-399742b62722") Fixed width=64.0;
    EPOK_PROPERTY(EditAnywhere, Id="36993b24-8fa0-54d2-91ba-436d14eee0d4") Fixed height=64.0;
    EPOK_PROPERTY(EditAnywhere, Id="c4a6bea3-030d-5457-b6bf-71b4485d7a23") bool monitoring=true;
    EPOK_FUNCTION(BlueprintCallable, Id="e36716bf-d600-572c-8d70-2a90fb283c5b") void set_monitoring(bool value){monitoring=value;configured_=false;configure();}
    EPOK_FUNCTION(BlueprintPure, Id="6f05bac6-37ef-5f4f-9068-ee433c7491a4") InteractionResult binding_status() const{return interaction::sensor_binding(*const_cast<InteractionSensor2DComponent*>(this),target,accepted_initiator);}
    void level_attached() override{Collider2DComponent::level_attached();configure();}
    void configure(){
        const bool changed=last_target_!=target||last_accepted_!=accepted_initiator;
        if(!configured_||last_monitoring_!=monitoring)shape.enabled=monitoring;
        configured_=true;last_monitoring_=monitoring;last_target_=target;last_accepted_=accepted_initiator;
        shape.trigger=true;set_box_size(width>0.0?width:Fixed(0.0),height>0.0?height:Fixed(0.0));
        if(!monitoring||width<=0.0||height<=0.0)shape.enabled=false;
        if(changed||!shape.enabled)interaction::forget_sensor(id());
    }
    void frame_update(uint32_t elapsed) override{configure();ActorComponent::frame_update(elapsed);}
    void trigger_event(ObjectId other,TriggerPhase phase) override{interaction::sensor_event(*this,target,accepted_initiator,other,phase);}
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World3D, Id="177ad606-c610-5ca6-ba99-a6126b85595d") TriggerBox3D : public Actor3D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("177ad606-c610-5ca6-ba99-a6126b85595d");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Sensor", Id="99a4c53c-7ee9-5cef-a124-5ba4b9e346c0") InteractionSensor3DComponent sensor;
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=World2D, Id="1d128dde-89a7-516e-bc39-2b4c2eada9c5") TriggerArea2D : public Actor2D {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("1d128dde-89a7-516e-bc39-2b4c2eada9c5");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Sensor", Id="b1387e56-fd81-5b43-bd1a-d0648a2f3063") InteractionSensor2DComponent sensor;
};
class EPOK_CLASS(Blueprintable, Domain=UI, Owners=UI, Capability=interaction, Id="b7e56800-3c6f-5e1d-a165-848c122b51c5") InteractionPromptComponent : public UIComponent {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("b7e56800-3c6f-5e1d-a165-848c122b51c5");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_PROPERTY(EditAnywhere, ComponentClass="epok::InteractorComponent", Id="031a5808-57db-59f5-bb79-95862afff739") ObjectId interactor{};
    EPOK_PROPERTY(EditAnywhere, Id="b98bc2b4-993d-500d-8eaa-a115b434b92e") uint32_t input_port=0;
    void frame_update(uint32_t elapsed) override {
        auto* owner=get_owner();auto* data=owner?owner->data():nullptr;
        if(data) {
            auto* source=active_object_registry->resolve<InteractorComponent>(interactor);
            auto* target=source?active_object_registry->resolve<InteractableComponent>(source->focused_target()):nullptr;
            data->text.enabled=target&&source->input_port==input_port&&target->prompt_key>=0&&target->prompt_key<localization.keys&&localization.strings;
            data->text.key=data->text.enabled?int16_t(target->prompt_key):-1;
            if(data->text.enabled)data->text.set_text(localization.get(target->prompt_key));
        }
        UIComponent::frame_update(elapsed);
    }
};
class EPOK_CLASS(Blueprintable, Placeable, Spawnable, Domain=UI, Id="b68bfec7-d1e2-5a99-8afd-14536a1196c3") InteractionPrompt : public UIActor {
public:
    static constexpr uint64_t static_class_id=detail::compact_class_id("b68bfec7-d1e2-5a99-8afd-14536a1196c3");
    uint64_t class_id() const override{return m_runtime_class_id?m_runtime_class_id:static_class_id;}
    EPOK_COMPONENT(Name="Text", Id="63c9f590-6e81-56f9-969f-ef5c55ed383c") TextComponent text;
    EPOK_COMPONENT(Name="Presenter", Id="40731523-da1a-54e8-9c99-799878c080d5") InteractionPromptComponent presenter;
};
}
#if defined(EPOK_DIALOGUE) && !defined(EPOK_INCLUDE_FROM_DIALOGUE)
#include "dialogue.hpp"
#endif
