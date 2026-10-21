#define EPOK_ACTOR_TABLE_FIXTURE_ONLY
#include "test_actor_tables.cpp"
int main(){
#ifdef _MSC_VER
 _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
 load_and_begin_play();tick_order_runs_actors_then_script();
 auto* actor=object_registry.resolve<Actor>(level.actor_at(0));
 auto* child=object_registry.resolve<Actor>(level.actor_at(1));assert(actor&&child);
 bool activation[objects.size()]={};for(size_t i=0;i<object_count;++i)activation[i]=is_active_slot(i);
 begin_render_activation_snapshot();for(size_t i=0;i<object_count;++i)assert(is_active_slot(i)==activation[i]);end_render_activation_snapshot();
 assert(set_active(actor->data(),false)&&!is_active(child->data()));
 assert(set_active(actor->data(),true)&&is_active(child->data()));
 const auto inactive_id=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Inactive Parent");
 auto* inactive=object_registry.resolve<Actor>(inactive_id);assert(inactive&&set_active(inactive->data(),false));
 assert(level.set_logical_parent(child->id(),inactive_id)&&!is_active(child->data()));
 assert(level.set_logical_parent(child->id(),actor->id())&&is_active(child->data()));
 assert(level.destroy_actor(inactive_id));
 const auto id=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"First");
 auto* first=object_registry.resolve<Actor>(id);assert(first);auto* slot=first->data();auto old=handle(slot);
 assert(destroy_actor_data(slot)&&!old.get());
 music_active=&slot->audio;
 const auto quarantined=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Quarantine");
 assert(object_registry.resolve<Actor>(quarantined)->data()!=slot);
 music_active=nullptr;
 const auto reused=level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Reused");
 assert(object_registry.resolve<Actor>(reused)->data()==slot&&!old.get());
 assert(find_actor_data("Reused")==slot&&!find_actor_data("First"));
 transition_ends_the_script_before_actors();
 psyqo::GPU gpu;
 for(unsigned i=0;i<100;++i){
  const auto before=level.actor_at(0);const auto data=handle(object_registry.resolve<Actor>(before)->data());
  assert(request_scene(size_t(0))&&scene_tick(gpu));
  assert(!object_registry.get(before)&&!data.get());assert(level.actor_count()==2);
 }
 unload_actor_bank();assert(!level.actor_count());
 std::puts("Actor lifecycle: inherited activation, quarantine, data-slot reuse and 100 scene reloads passed.");
}
