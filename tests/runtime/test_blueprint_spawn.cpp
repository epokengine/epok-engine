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
ActorData* rollback_observer=nullptr;
bool configure(const DataHandle* data,size_t count){
 assert(count==2&&data[0].get()&&data[1].get());
 auto* actor=data[0].get()->owner;
 assert(actor&&object_registry.resolve<Hero>(actor->id())->speed==Fixed(2.5));
 if(reject_services&&rollback_observer) {
  rollback_observer->canvas.focused=int16_t(data[1].index);
  rollback_observer->focusable.neighbors[0]=int16_t(data[1].index);
 }
 return !reject_services;
}
const ActorPrototype prototype={&prototype_table,prototype_data,2,&configure};
// A cooked prefab keeps no ActorData: each spawn builds its slots in place.
size_t compact_builds=0;
void build_in_place(ActorData* const* slots){
 ++compact_builds;
 PrototypeSlots objects{slots};
 objects[0].transform.position[0]=7.0;
 objects[1].transform.scale[0]=objects[1].transform.scale[1]=objects[1].transform.scale[2]=1.0;
}
const ActorPrototype compact={&prototype_table,nullptr,2,nullptr,&build_in_place};
const ActorPrototype* active_prototype=&prototype;
const ActorPrototype* lookup(uint64_t id){return id==hero_class?active_prototype:nullptr;}
size_t live_data(){size_t count=0;for(size_t i=0;i<object_count;++i)if(objects[i].alive)++count;return count;}
}
void compact_prototype_builds_every_spawn(ObjectId parent,Actor* owner){
 active_prototype=&compact;
 for(int spawn=0;spawn<2;++spawn){
  const auto root=bp::spawn_actor(owner,hero_class,parent);
  auto* hero=object_registry.resolve<Hero>(root);assert(hero&&hero->data());
  assert(hero->data()->transform.position[0]==Fixed(7.0)&&hero->data()->alive);
  assert(level.destroy_actor(root));
 }
 assert(compact_builds==2);
 active_prototype=&prototype;
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
 prototype_data[0].canvas.enabled=true;prototype_data[0].canvas.focused=1;
 prototype_data[1].focusable.enabled=true;prototype_data[1].focusable.neighbors[0]=0;prototype_data[1].focusable.neighbors[1]=1;
 const auto root=bp::spawn_actor(owner,hero_class,parent);
 auto* hero=object_registry.resolve<Hero>(root);assert(hero&&hero->observed_speed==Fixed(2.5)&&hero->observed_component);
 assert(hero->data()->transform.position[0]==Fixed(3.0));
 assert(hero->logical_parent()==parent&&hero->data()->parent==int(handle(owner->data()).index));
 assert(hero->root.attach_parent==owner->root_id());
 auto* child=object_registry.resolve<Actor>(level.actor_at(3));assert(child&&child->logical_parent()==root);
 assert(child->data()->parent==int(handle(hero->data()).index));
 assert(hero->data()->canvas.focused==int(handle(child->data()).index));
 assert(child->data()->focusable.neighbors[0]==int(handle(hero->data()).index));
 assert(child->data()->focusable.neighbors[1]==int(handle(child->data()).index));
 assert(child->data()->focusable.neighbors[2]==-1);
 const auto child_id=child->id();const auto old=handle(hero->data());
 assert(level.destroy_actor(root));assert(!object_registry.get(root)&&!object_registry.get(child_id)&&!old.get());
 // A failed preparation releases every Actor, component and backing slot.
 const auto live=object_registry.live(),data_count=live_data();reject_services=true;rollback_observer=owner->data();
 assert(!bp::spawn_actor(owner,hero_class,{}).valid());
 assert(object_registry.live()==live&&live_data()==data_count);
 assert(rollback_observer->canvas.focused==-1&&rollback_observer->focusable.neighbors[0]==-1);
 reject_services=false;rollback_observer=nullptr;
 const auto retry=bp::spawn_actor(owner,hero_class,{});assert(retry.valid()&&retry!=root);
 auto* retried=object_registry.resolve<Actor>(retry);assert(retried);
 const auto retry_slot=handle(retried->data()).index;
 assert(retried->data()->canvas.focused!=1&&retried->data()->canvas.focused!=int(retry_slot));
 assert(prototype_data[0].canvas.focused==1&&prototype_data[1].focusable.neighbors[0]==0);
 // Partial destruction clears incoming links, including temporarily disabled
 // controls. A replacement in the same data slot must never inherit focus.
 Actor* target=nullptr;
 for(size_t i=0;i<level.actor_count();++i){auto* a=object_registry.resolve<Actor>(level.actor_at(i));if(a&&a->logical_parent()==retry)target=a;}
 assert(target);const auto target_handle=handle(target->data());
 auto* observer=owner->data();observer->canvas.enabled=false;observer->focusable.enabled=false;
 observer->canvas.focused=int16_t(target_handle.index);
 observer->focusable.neighbors[0]=int16_t(target_handle.index);
 observer->focusable.neighbors[1]=int16_t(retry_slot);
 assert(level.destroy_actor(target->id()));assert(!target_handle.get());
 assert(retried->data()->canvas.focused==-1&&observer->canvas.focused==-1);
 assert(observer->focusable.neighbors[0]==-1&&observer->focusable.neighbors[1]==int(retry_slot));
 const auto replacement=bp::spawn_actor(owner,Actor3D::static_class_id,{});
 auto* replaced=object_registry.resolve<Actor>(replacement);assert(replaced);
 const auto replacement_handle=handle(replaced->data());
 assert(replacement_handle.index==target_handle.index&&replacement_handle.generation!=target_handle.generation);
 replaced->data()->focusable.enabled=true;
 assert(retried->data()->canvas.focused==-1&&observer->canvas.focused==-1&&observer->focusable.neighbors[0]==-1);
 assert(level.destroy_actor(replacement));
 assert(level.destroy_actor(retry));
 assert(observer->focusable.neighbors[1]==-1);
 compact_prototype_builds_every_spawn(parent,owner);
 // Callback spawns are queued and allocate nothing until dispatch ends.
 const auto actors=level.actor_count(),slots=object_count;
 {ObjectDispatchScope scope(object_registry);assert(!bp::spawn_actor(owner,Actor3D::static_class_id,{}).valid());assert(level.actor_count()==actors&&object_count==slots);}
 level.tick(0.25);assert(level.actor_count()==actors+1);
 assert(!bp::spawn_actor(owner,tracker_class,{}).valid());
 const auto final_widget=bp::spawn_actor(owner,hero_class,{});assert(final_widget.valid());
 auto* final_data=object_registry.resolve<Actor>(final_widget)->data();const auto final_handle=handle(final_data);
 assert(final_data->canvas.focused>=0);
 unload_actor_bank();assert(level.actor_count()==0&&!final_handle.get()&&final_data->canvas.focused==-1);
 std::puts("Actor spawning: identity, prototypes, rollback, callbacks and focus cleanup on partial destruction, slot reuse and unload passed.");
}
