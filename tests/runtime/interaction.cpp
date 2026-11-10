// Exercise real actor/component identities, activation and trigger fan-out.
#define EPOK_INTERACTION 1
#define EPOK_GAMEPLAY_CONTROL 1
#include <cassert>
#include <cstdio>
#include "../../runtime/interaction.hpp"

using namespace epok;
struct TestTarget : InteractableComponent {
    static constexpr uint64_t static_class_id=0x987654;
    uint64_t class_id() const override{return static_class_id;}
    int gained=0,lost=0,started=0,finished=0;
    bool allowed=true,defer=false,destroy_self=false,reenter=false;
    mutable bool query_reentry=false,query_destroy=false;
    mutable InteractionResult query_result=InteractionResult::Accepted;
    InteractionResult nested=InteractionResult::Accepted;
    bool can_interact(ObjectId source) const override{
        auto* self=const_cast<TestTarget*>(this);
        if(query_reentry)query_result=self->request(source,0);
        if(query_destroy)self->get_owner()->destroy();
        return allowed;
    }
    void focus_gained(ObjectId,uint32_t) override{++gained;}
    void focus_lost(ObjectId,uint32_t) override{++lost;}
    void interaction_started(ObjectId source,uint32_t port) override{
        ++started;if(reenter)nested=request(source,port);if(defer)hold_open();if(destroy_self)get_owner()->destroy();
    }
    void interaction_finished(ObjectId,uint32_t,InteractionResult) override{++finished;}
};
namespace epok {
// Sensor-only data has no compact HUD/data-slot index in this host fixture.
int entity_index(const ActorData*){return -1;}
void AudioSource::play(){} void AudioSource::stop(){} bool AudioSource::is_playing() const{return false;}
bool scene_loading(){return false;}
WorldAffineSample gameplay_world_affine(const ActorData* data){WorldAffineSample out;if(data){out.success=true;for(int i=0;i<3;++i)out.position[i]=data->transform.position[i];}return out;}
template<class T> ClassDescriptor component(uint64_t parent,ObjectDomain domain,uint8_t owners){
    return {T::static_class_id,parent,ObjectFamily::Component,domain,owners,0,&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,32>::acquire,&ObjectPool<T,32>::release};
}
template<class T> ClassDescriptor actor(ObjectDomain domain){
    return {T::static_class_id,Actor::static_class_id,ObjectFamily::Actor,domain,0,uint8_t(ObjectClassPlaceable|ObjectClassSpawnable),&object_construct<T>,&object_destruct,sizeof(T),alignof(T),&ObjectPool<T,24>::acquire,&ObjectPool<T,24>::release};
}
const ClassDescriptor object_classes[]={
    {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
    {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
    {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
    {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot,&object_construct<SceneComponent3D>,&object_destruct,sizeof(SceneComponent3D),alignof(SceneComponent3D)},
    {SceneComponent2D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World2D,2,ObjectClassRoot,&object_construct<SceneComponent2D>,&object_destruct,sizeof(SceneComponent2D),alignof(SceneComponent2D)},
    actor<Actor3D>(ObjectDomain::World3D),actor<Actor2D>(ObjectDomain::World2D),
    component<InteractorComponent>(ActorComponent::static_class_id,ObjectDomain::None,3),
    component<InteractableComponent>(ActorComponent::static_class_id,ObjectDomain::None,3),
    component<TestTarget>(InteractableComponent::static_class_id,ObjectDomain::None,3),
    component<Collider3DComponent>(ActorComponent::static_class_id,ObjectDomain::World3D,1),
    component<Collider2DComponent>(ActorComponent::static_class_id,ObjectDomain::World2D,2),
    component<InteractionSensor3DComponent>(Collider3DComponent::static_class_id,ObjectDomain::World3D,1),
    component<InteractionSensor2DComponent>(Collider2DComponent::static_class_id,ObjectDomain::World2D,2),
    {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract}
};
const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
}

int main(){
    ObjectRegistryStorage<128> registry;Level level;assert(level.bind(registry));
    auto spawn=[&](uint64_t type=Actor3D::static_class_id){auto id=level.spawn_actor(*find_object_class(type),"Instance");assert(id.valid());return registry.resolve<Actor>(id);};
    auto* player=spawn();auto* scanner=level.add_component<InteractorComponent>(*player);
    scanner->auto_input=false;
    auto* first=spawn();auto* a=level.add_component<TestTarget>(*first);a->selection_order=2;
    auto* second=spawn();auto* b=level.add_component<TestTarget>(*second);b->selection_order=1;
    first->set_name("Zulu");second->set_name("Alpha");
    scanner->refresh();assert(scanner->focused_target()==b->id()&&scanner->candidate_count()==2);
    a->priority=5;scanner->refresh();assert(scanner->focused_target()==a->id()&&b->lost==1);
    a->allowed=false;scanner->refresh();assert(scanner->focused_target()==b->id());
    a->allowed=true;a->priority=0;
    auto* root=registry.resolve<SceneComponent3D>(second->root_id());root->set_local_position(1.0,0.0,0.0);
    scanner->refresh();assert(scanner->focused_target()==a->id());

    // Synchronous overrides, busy/reentry, explicit completion and automatic completion.
    a->reenter=true;assert(scanner->interact()==InteractionResult::Accepted);
    assert(a->started==1&&a->finished==1&&a->nested==InteractionResult::Busy&&!a->is_busy());
    a->defer=true;assert(scanner->interact()==InteractionResult::Accepted&&a->is_busy());
    assert(a->request(player->id(),0)==InteractionResult::Busy);
    assert(a->complete(InteractionResult::Accepted)&&!a->complete(InteractionResult::Accepted));
    assert(a->finished==2);
    scanner->refresh();assert(scanner->focused_target()==a->id());
    a->set_enabled(false);assert(!scanner->focused_target().valid());a->set_enabled(true);
    scanner->refresh();assert(scanner->focused_target()==a->id());
    first->set_active(false);first->set_active(true);assert(!scanner->focused_target().valid());

    // Best eight are independent of registration/list order; overflow is observable.
    Actor* extras[9];for(unsigned i=0;i<9;++i){extras[i]=spawn();auto* value=level.add_component<TestTarget>(*extras[i]);value->selection_order=100+i;value->priority=int32_t(i+1);}
    scanner->refresh();assert(scanner->candidate_count()==8&&scanner->overflow_count()>0);
    assert(scanner->focused_target()==extras[8]->component_id(1));
    for(auto* actor:extras)assert(level.destroy_actor(actor->id()));
    a->defer=false;a->reenter=false;
    a->query_reentry=true;scanner->refresh();assert(a->query_result==InteractionResult::Busy);a->query_reentry=false;

    // Sensor identity distinguishes overlapping sensors; unrelated actors are ignored.
    scanner->proximity=false;scanner->refresh();assert(!scanner->focused_target().valid());
    auto* sensor_actor=spawn();auto* sensor=level.add_component<InteractionSensor3DComponent>(*sensor_actor);
    auto* sensor_actor2=spawn();auto* sensor2=level.add_component<InteractionSensor3DComponent>(*sensor_actor2);
    ActorData sensor_data{},sensor_data2{};sensor_actor->bind_data(sensor_data);sensor_actor2->bind_data(sensor_data2);
    assert(sensor->binding_status()==InteractionResult::MissingBinding);
    sensor->target=a->id();sensor2->target=a->id();sensor->frame_update(0);sensor2->frame_update(0);
    assert(sensor->binding_status()==InteractionResult::Accepted);
    sensor->trigger_event(player->id(),TriggerPhase::Enter);assert(scanner->focused_target()==a->id());
    sensor->trigger_event(player->id(),TriggerPhase::Stay);sensor2->trigger_event(player->id(),TriggerPhase::Enter);
    sensor->trigger_event(player->id(),TriggerPhase::Exit);assert(scanner->focused_target()==a->id());
    sensor2->trigger_event(player->id(),TriggerPhase::Exit);assert(!scanner->focused_target().valid());
    sensor->accepted_initiator=first->id();sensor->trigger_event(player->id(),TriggerPhase::Enter);assert(!scanner->focused_target().valid());
    sensor->accepted_initiator={};sensor->trigger_event(first->id(),TriggerPhase::Enter);assert(!scanner->focused_target().valid());
    sensor->trigger_event(player->id(),TriggerPhase::Enter);sensor_actor->set_active(false);sensor_actor->set_active(true);
    assert(!scanner->focused_target().valid());
    scanner->refresh();assert(!scanner->focused_target().valid());
    sensor->trigger_event(player->id(),TriggerPhase::Enter);assert(scanner->focused_target()==a->id());
    sensor->set_monitoring(false);assert(!scanner->focused_target().valid());
    sensor->trigger_event(player->id(),TriggerPhase::Stay);assert(!scanner->focused_target().valid());
    sensor->set_monitoring(true);sensor->trigger_event(player->id(),TriggerPhase::Enter);assert(scanner->focused_target()==a->id());
    sensor->target=b->id();sensor->frame_update(0);assert(!scanner->focused_target().valid());
    sensor->trigger_event(player->id(),TriggerPhase::Stay);assert(scanner->focused_target()==b->id());
    sensor->set_enabled(false);sensor->frame_update(0);assert(!sensor->enabled()&&!scanner->focused_target().valid());
    sensor->width=-1.0;sensor->set_monitoring(true);assert(!sensor->enabled()&&sensor_data.collider.half_extents[0]==0.0);

    // The identical API uses 2D pixel transforms and refuses cross-domain bindings.
    auto* player2=spawn(Actor2D::static_class_id);auto* scanner2=level.add_component<InteractorComponent>(*player2);scanner2->auto_input=false;
    auto* npc2=spawn(Actor2D::static_class_id);auto* c=level.add_component<TestTarget>(*npc2);
    registry.resolve<SceneComponent2D>(npc2->root_id())->set_position(48.0,0.0);
    scanner2->refresh();assert(scanner2->focused_target()==c->id()&&scanner2->interact()==InteractionResult::Accepted);
    assert(a->availability(player2->id())==InteractionResult::DomainMismatch&&!scanner2->register_candidate(a->id(),{}));
    assert(!level.add_component<InteractionSensor3DComponent>(*npc2));
    registry.resolve<SceneComponent2D>(npc2->root_id())->set_position(65.0,0.0);
    scanner2->refresh();assert(!scanner2->focused_target().valid());

    // A routed frame press interacts once and cannot also reach a later Jump tick.
    scanner->proximity=true;scanner->auto_input=true;a->selection_order=0;
    input.reset();gameplay_controls().reset();input.sample(0,true,0);gameplay_controls().begin_frame();
    input.sample(0,true,1u<<unsigned(Button::Cross));gameplay_controls().begin_frame();
    const auto started=a->started;scanner->frame_update(0);assert(a->started==started+1);
    input.begin_tick();gameplay_controls().begin_tick();assert(!GameplayInputLibrary::take_pressed(GameplayAction::Jump,0));
    scanner->frame_update(0);assert(a->started==started+1);
    const auto removed=a->id();a->destroy_self=true;assert(scanner->interact()==InteractionResult::Accepted);
    scanner->refresh();assert(scanner->focused_target()!=removed);
    auto* condition_owner=spawn();auto* condition_target=level.add_component<TestTarget>(*condition_owner);
    condition_target->query_destroy=true;
    assert(condition_target->availability(player->id())==InteractionResult::InvalidOwner);

    level.end_play_all(EndPlayReason::LevelUnloaded);
    assert(!interaction::targets.head&&!interaction::interactors.head);
    registry.release(level.id());active_object_registry=nullptr;
    std::printf("Interaction: 2D/3D selection, bounds, scoped sensors, lifecycle, reentry and input consumption pass. Interactor %zu / target %zu bytes (host).\n",sizeof(InteractorComponent),sizeof(InteractableComponent));
}
