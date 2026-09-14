// Dynamic native/Blueprint Actors use the same tables and lifecycle as scene Actors.
#define EPOK_ACTOR_TABLE_FIXTURE_ONLY
#include "test_actor_tables.cpp"
#include "blueprint_spawn.hpp"
namespace {
ActorData prototype_data[2];
constexpr ActorComponentRecord root_components[]={
 {SceneComponent3D::static_class_id,"Root",true,-1,0},
 {tracker_class,"Tracker",false,-1,0},
};
constexpr ActorComponentRecord child_components[]={
 {SceneComponent3D::static_class_id,"Root",true,-1,1},
};
const ActorRecord prototype_rows[]={
 {hero_class,"Parent",true,-1,-1,-1,root_components,2,&actor_apply_0},
 {Actor3D::static_class_id,"Child",true,0,0,0,child_components,1,nullptr},
};
const ActorTable prototype_table={prototype_rows,2};
bool reject_services=false;
bool configure(const DataHandle* data,size_t count){
 assert(count==2&&data[0].get()&&data[1].get());
 auto* actor=data[0].get()->owner;
 assert(actor&&object_registry.resolve<Hero>(actor->id())->speed==Fixed(2.5));
 return !reject_services;
}
const ActorPrototype prototype={&prototype_table,prototype_data,2,&configure};
const ActorPrototype* lookup(uint64_t id){return id==hero_class?&prototype:nullptr;}
size_t live_data(){size_t count=0;for(size_t i=0;i<object_count;++i)if(objects[i].alive)++count;return count;}
}
int main(){
#ifdef _MSC_VER
 _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
 reset();load_scene();actor_template_lookup=&lookup;
 const auto parent=level.actor_at(0);
 auto* owner=object_registry.resolve<Actor>(parent);assert(owner);
 for(auto& data:prototype_data){data=ActorData{};data.transform.scale[0]=data.transform.scale[1]=data.transform.scale[2]=1.0;}
 prototype_data[0].transform.position[0]=3.0;
 const auto root=bp::spawn_actor(owner,hero_class,parent);
 auto* hero=object_registry.resolve<Hero>(root);assert(hero&&hero->observed_speed==Fixed(2.5)&&hero->observed_component);
 assert(hero->data()->transform.position[0]==Fixed(3.0));
 assert(hero->logical_parent()==parent&&hero->data()->parent==int(handle(owner->data()).index));
 assert(hero->root.attach_parent==owner->root_id());
 auto* child=object_registry.resolve<Actor>(level.actor_at(3));assert(child&&child->logical_parent()==root);
 assert(child->data()->parent==int(handle(hero->data()).index));
 const auto child_id=child->id();const auto old=handle(hero->data());
 assert(level.destroy_actor(root));assert(!object_registry.get(root)&&!object_registry.get(child_id)&&!old.get());
 // A failed preparation releases every Actor, component and backing slot.
 const auto live=object_registry.live(),data_count=live_data();reject_services=true;
 assert(!bp::spawn_actor(owner,hero_class,{}).valid());
 assert(object_registry.live()==live&&live_data()==data_count);
 reject_services=false;
 const auto retry=bp::spawn_actor(owner,hero_class,{});assert(retry.valid()&&retry!=root);
 assert(level.destroy_actor(retry));
 // Callback spawns are queued and allocate nothing until dispatch ends.
 const auto actors=level.actor_count(),slots=object_count;
 {ObjectDispatchScope scope(object_registry);assert(!bp::spawn_actor(owner,Actor3D::static_class_id,{}).valid());assert(level.actor_count()==actors&&object_count==slots);}
 level.tick(0.25);assert(level.actor_count()==actors+1);
 assert(!bp::spawn_actor(owner,tracker_class,{}).valid());
 unload_actor_bank();assert(level.actor_count()==0);
 std::puts("Actor spawning: typed identity, complete prototypes, parent transforms, pre-BeginPlay overrides, rollback and deferred callbacks passed.");
}
