#define EPOK_INTERACTION 1
#define EPOK_GAMEPLAY_CONTROL 1
#include <cassert>
#include <cstdio>
#include "../../runtime/world_gameplay.hpp"
using namespace epok;

struct PickupProbe:PickupComponent {
    static constexpr uint64_t static_class_id=0x6a01;
    uint64_t class_id()const override{return static_class_id;}
    unsigned rewards=0;bool allowed=true,destroy=false;
    HelperResult nested=HelperResult::Applied;
    bool can_collect(ObjectId)const override{return allowed;}
    void collected(ObjectId source,uint32_t port)override{++rewards;nested=collect(source,port);assert(!reset_collection());if(destroy)get_owner()->destroy();}
};
struct FlagProbe:GameplayFlagComponent {
    static constexpr uint64_t static_class_id=0x6a02;
    uint64_t class_id()const override{return static_class_id;}
    unsigned changes=0;HelperResult nested=HelperResult::Applied;
    void value_changed(bool previous,bool current)override{assert(previous!=current);++changes;nested=set_value(!current);}
};
struct SwitchProbe:SwitchComponent {
    static constexpr uint64_t static_class_id=0x6a03;
    uint64_t class_id()const override{return static_class_id;}
    unsigned changes=0;HelperResult nested=HelperResult::Applied;
    void switched(ObjectId,bool)override{++changes;nested=reset_switch();}
};
struct DoorProbe:DoorComponent {
    static constexpr uint64_t static_class_id=0x6a04;
    uint64_t class_id()const override{return static_class_id;}
    unsigned changes=0;bool allowed=true,clear_flag=false,destroy_shape=false;
    HelperResult nested=HelperResult::Applied;
    bool can_open(ObjectId)const override{
        auto* self=const_cast<DoorProbe*>(this);self->nested=self->open({});
        if(clear_flag)if(auto* flag=world_gameplay::flag(*this,requirement_flag))flag->set_value(false);
        if(destroy_shape&&collider_2d.valid())if(auto* shape=active_object_registry->resolve<Collider2DComponent>(collider_2d))if(auto* owner=shape->get_owner())active_object_registry->resolve<Level>(owner->level_id())->remove_component(*owner,shape->id());
        return allowed;
    }
    void door_changed(DoorState previous,DoorState current)override{assert(previous!=current);++changes;nested=close();}
};
struct PhysicsProbe:ActorComponent {
    static constexpr uint64_t static_class_id=0x6a05;
    static constexpr TickGroup tick_group=TickGroup::Physics;
    uint64_t class_id()const override{return static_class_id;}
    ObjectId shape{};bool expected=true;unsigned observations=0;
    void tick(Fixed)override{auto* value=active_object_registry->resolve<Collider2DComponent>(shape);assert(value&&value->is_enabled()==expected);++observations;}
};
namespace epok {
int entity_index(const ActorData*){return -1;}
void AudioSource::play(){}void AudioSource::stop(){}bool AudioSource::is_playing()const{return false;}
bool scene_loading(){return false;}
WorldAffineSample gameplay_world_affine(const ActorData* data){WorldAffineSample out;if(data){out.success=true;for(int i=0;i<3;++i)out.position[i]=data->transform.position[i];}return out;}
template<class T,class Base> ClassDescriptor component(ObjectDomain domain=ObjectDomain::None,uint8_t owners=3){
    auto value=ClassDescriptor{T::static_class_id,Base::static_class_id,ObjectFamily::Component,domain,owners,0,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,12>::acquire,&ObjectPool<T,12>::release};
    value.component_tick=ComponentCallbacks<T>::tick;value.component_frame=ComponentCallbacks<T>::frame;value.tick_group=TickGroupOf<T>::value;return value;
}
template<class T> ClassDescriptor actor(ObjectDomain domain){return {T::static_class_id,Actor::static_class_id,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,24>::acquire,&ObjectPool<T,24>::release};}
const ClassDescriptor object_classes[]={
    {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
    {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
    {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,3,ObjectClassAbstract},
    {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot,&object_construct<SceneComponent3D>,&object_destruct,sizeof(SceneComponent3D),alignof(SceneComponent3D)},
    {SceneComponent2D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World2D,2,ObjectClassRoot,&object_construct<SceneComponent2D>,&object_destruct,sizeof(SceneComponent2D),alignof(SceneComponent2D)},
    actor<Actor3D>(ObjectDomain::World3D),actor<Actor2D>(ObjectDomain::World2D),
    component<InteractableComponent,ActorComponent>(),component<GameplayFlagComponent,ActorComponent>(),component<FlagProbe,GameplayFlagComponent>(),
    component<PickupComponent,InteractableComponent>(),component<PickupProbe,PickupComponent>(),
    component<DoorComponent,InteractableComponent>(),component<DoorProbe,DoorComponent>(),
    component<SwitchComponent,InteractableComponent>(),component<SwitchProbe,SwitchComponent>(),component<PhysicsProbe,ActorComponent>(),
    component<Collider3DComponent,ActorComponent>(ObjectDomain::World3D,1),component<Collider2DComponent,ActorComponent>(ObjectDomain::World2D,2),
    {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}
int main(){
    ObjectRegistryStorage<160> registry;Level level;assert(level.bind(registry));
    auto spawn=[&](bool two){auto id=level.spawn_actor(*find_object_class(two?Actor2D::static_class_id:Actor3D::static_class_id),"Renamable");assert(id.valid());return registry.resolve<Actor>(id);};
    ActorData data[24];unsigned slots=0;
    for(bool two:{false,true}){
        auto* player=spawn(two);auto* foreign=spawn(two);auto* pickup_actor=spawn(two);auto* state_actor=spawn(two);auto* door_actor=spawn(two);auto* switch_actor=spawn(two);
        auto* flag=level.add_component<FlagProbe>(*state_actor);auto* pickup=level.add_component<PickupProbe>(*pickup_actor);
        auto* door=level.add_component<DoorProbe>(*door_actor);auto* button=level.add_component<SwitchProbe>(*switch_actor);
        Collider3DComponent* shape3=nullptr;Collider2DComponent* shape2=nullptr;
        if(two){shape2=level.add_component<Collider2DComponent>(*door_actor);door->collider_2d=shape2->id();}
        else{data[slots].collider.enabled=true;door_actor->bind_data(data[slots++]);shape3=level.add_component<Collider3DComponent>(*door_actor);door->collider_3d=shape3->id();}
        auto blocked=[&](){return two?shape2->is_enabled():shape3->enabled();};
        pickup->accepted_initiator=player->id();pickup->collected_flag=flag->id();door->requirement_flag=flag->id();button->target_flag=flag->id();
        flag->begin_play();pickup->begin_play();door->begin_play();button->begin_play();
        assert(blocked()&&door->state()==DoorState::Closed&&!flag->value());
        assert(door->open(player->id())==HelperResult::RequirementUnmet);
        assert(pickup->collect(foreign->id(),0)==HelperResult::InvalidInitiator&&!pickup->is_collected());
        pickup->allowed=false;assert(pickup->collect(player->id(),0)==HelperResult::Unavailable);pickup->allowed=true;
        pickup->auto_collect=true;pickup->trigger_event(foreign->id(),TriggerPhase::Enter);assert(!pickup->is_collected());
        pickup->trigger_event(player->id(),TriggerPhase::Stay);assert(!pickup->is_collected());
        pickup->trigger_event(player->id(),TriggerPhase::Enter);
        assert(pickup->is_collected()&&pickup->rewards==1&&pickup->nested==HelperResult::Busy&&flag->value());
        assert(flag->changes==1&&flag->nested==HelperResult::Busy);
        assert(pickup->collect(player->id(),0)==HelperResult::AlreadyCollected&&pickup->rewards==1);
        pickup_actor->set_active(false);assert(flag->value()&&door->requirement_met());
        assert(door->open(player->id())==HelperResult::Queued&&blocked()&&door->has_pending_transition());
        door->tick(0.0);assert(!blocked()&&door->state()==DoorState::Open&&door->changes==1&&door->nested==HelperResult::Busy);
        assert(door->close()==HelperResult::Queued&&!blocked());door->tick(0.0);assert(blocked());
        assert(button->activate(player->id(),0)==HelperResult::Applied&&!flag->value()&&button->changes==1&&button->nested==HelperResult::Busy);
        assert(door->open(player->id())==HelperResult::RequirementUnmet);
        assert(button->activate(player->id(),0)==HelperResult::Applied&&flag->value());
        assert(door->open(player->id())==HelperResult::Queued);flag->set_value(false);door->tick(0.0);
        assert(blocked()&&door->last_result()==HelperResult::RequirementUnmet&&!door->has_pending_transition());
        flag->set_value(true);door->clear_flag=true;assert(door->open(player->id())==HelperResult::RequirementUnmet&&blocked());door->clear_flag=false;
        assert(door->lock()==HelperResult::Queued);door->tick(0.0);assert(door->state()==DoorState::Locked&&blocked());
        door->requirement_flag={};assert(door->open(player->id())==HelperResult::Locked);
        assert(door->unlock()==HelperResult::Queued);door->tick(0.0);assert(door->state()==DoorState::Closed);
        assert(door->open(player->id())==HelperResult::Queued);door->on_disable();door->tick(0.0);assert(blocked());
        const auto reset=button->reset_switch();assert((reset==HelperResult::Unchanged||reset==HelperResult::Applied)&&!button->value());
        button->toggle=false;const auto changes=button->changes;
        assert(button->activate(player->id(),0)==HelperResult::Applied&&button->value());
        assert(button->activate(player->id(),0)==HelperResult::Unchanged&&button->changes==changes+1);
        assert(button->reset_switch()==HelperResult::Applied&&!button->value());button->toggle=true;
        pickup_actor->set_active(true);assert(pickup->reset_collection()&&!pickup->is_collected());
        pickup->deactivate_on_collect=true;assert(pickup->collect(player->id(),0)==HelperResult::Applied&&!pickup_actor->active()&&pickup->is_collected()&&pickup->rewards==2);
        assert(door->open(player->id())==HelperResult::Queued);door->tick(0.0);assert(!blocked());
        assert(door->unlock()==HelperResult::Unchanged&&!door->has_pending_transition());
        door->initial_state=DoorState::Open;door->close();door->tick(0.0);assert(blocked());
        door->requirement_flag=flag->id();flag->set_value(false);
        assert(door->reset_door()==HelperResult::Queued);door->tick(0.0);assert(!blocked()&&door->state()==DoorState::Open);
        door->requirement_flag={};
        door->end_play(EndPlayReason::Destroyed);assert(blocked());
        if(two){
            // The global Input stage commits a queued opening before any Physics callback,
            // even when the observing actor was placed first.
            auto* observer=level.add_component<PhysicsProbe>(*player);observer->shape=shape2->id();observer->expected=false;
            assert(door->open(player->id())==HelperResult::Unchanged); // committed open; rebind on next input
            level.tick(0.0);assert(observer->observations==1&&!blocked());
            observer->expected=true;assert(door->close()==HelperResult::Queued);level.tick(0.0);assert(observer->observations==2&&blocked());
            auto* replacement_actor=spawn(true);auto* replacement=level.add_component<Collider2DComponent>(*replacement_actor);
            replacement->set_enabled(false);door->collider_2d=replacement->id();door->tick(0.0);assert(shape2->is_enabled()&&replacement->is_enabled());
            assert(door->open(player->id())==HelperResult::Queued);door->tick(0.0);assert(!replacement->is_enabled());
            door->end_play(EndPlayReason::Destroyed);assert(!replacement->is_enabled());
        }
    }
    auto* source=spawn(true);auto* victim=spawn(true);auto* pickup=level.add_component<PickupProbe>(*victim);pickup->accepted_initiator=source->id();pickup->destroy=true;
    auto id=pickup->id();const auto destroyed=pickup->collect(source->id(),0);
    assert(destroyed==HelperResult::Applied&&!interaction::live(id));
    level.frame_update(0);assert(!registry.resolve<PickupComponent>(id));
    auto* doomed=spawn(true);auto* flag=level.add_component<GameplayFlagComponent>(*doomed);
    {ObjectDispatchScope scope(registry);doomed->destroy();assert(flag->set_value(true)==HelperResult::InvalidOwner);}
    level.frame_update(0);
    level.end_play_all(EndPlayReason::LevelUnloaded);assert(!interaction::targets.head);
    registry.release(level.id());active_object_registry=nullptr;
    std::puts("PASS shared pickup/flag/door/switch: 2D/3D, once-only collection, explicit state, queued collision, requirements, reset, replacement and reentry/lifetime");
}
