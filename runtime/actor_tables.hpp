#pragma once
#include "object_model.hpp"
#ifndef EPOK_OBJECT_REGISTRY_CAPACITY
#define EPOK_OBJECT_REGISTRY_CAPACITY 64
#endif
#define EPOK_ACTOR_TABLES 1
namespace epok {
struct ActorComponentRecord {uint64_t class_id=0;const char* name=nullptr;bool root=false;int16_t attach_parent=-1,data_slot=-1,default_index=-1;};
struct ActorRecord {
 uint64_t class_id=0;const char* name=nullptr;bool active=true;
 int16_t logical_parent=-1,attach_actor=-1,attach_component=-1;
 const ActorComponentRecord* components=nullptr;size_t component_count=0;
 void (*apply)(ObjectRegistry&,Actor&,const ObjectId* components,const ObjectId* actors)=nullptr;
};
enum class SceneRefKind:uint8_t {Actor,Component};
struct SceneReferenceRecord {uint64_t class_id=0,member=0;SceneRefKind kind=SceneRefKind::Actor;int16_t actor=-1,component=-1;};
struct ActorTable {
 const ActorRecord* actors=nullptr;size_t count=0;uint64_t scene_script_class=0;
 const SceneReferenceRecord* references=nullptr;size_t reference_count=0;
 void (*bind_reference)(ObjectRegistry&,Actor&,uint64_t,ObjectId)=nullptr;
};
struct ActorPrototype {const ActorTable* table=nullptr;const ActorData* data=nullptr;size_t count=0;bool (*configure_services)(const DataHandle*,size_t)=nullptr;};
inline const ActorPrototype* (*actor_template_lookup)(uint64_t)=nullptr;
struct ActorStats {uint32_t alive=0,peak=0,rejected=0,spawned=0,deferred=0,actors=0,components=0,scene_scripts=0,banks_loaded=0;};
inline ActorStats actor_stats;

// Every identity and component exists before references, overrides and begin_play.
class SceneLevel:public Level {
 struct Context {const ActorTable* table;const ObjectId* actors;ActorData* const* data;size_t data_count;ObjectId parent;const ActorPrototype* prototype;};
 Context* context=nullptr;
public:
 bool ensure_bound(ObjectRegistry& registry) {if(m_registry==&registry){active_object_registry=&registry;return true;}return bind(registry);}
 ObjectId add_component_by_class(Actor& actor,const ClassDescriptor& type,const char* name) {
  const auto* owner_type=find_object_class(actor.class_id());
  if(!m_registry||!owner_type||!accepts_component(actor,*owner_type,type)){if(m_registry)++m_registry->stats.rejected;return {};}
  const auto id=m_registry->acquire(type);auto* component=m_registry->resolve<ActorComponent>(id);
  if(!component){if(id.valid())m_registry->release(id);return {};}
  attach_component_record(actor,*component,name);set_state(id,ObjectState::Initialized);return id;
 }
 size_t load_bank(const ActorTable& table,ActorData* slots,size_t count) {
  if(!m_registry||table.count>level_actor_capacity||count>level_actor_capacity)return 0;
  ActorData* data[level_actor_capacity]={};for(size_t i=0;i<count;++i)data[i]=&slots[i];
  ObjectId ids[level_actor_capacity]={};Context current{&table,ids,data,count,{},nullptr};
  auto* previous=context;context=&current;const auto loaded=instantiate(current,ids,nullptr);
  if(loaded==table.count)create_bank_scene_script(table);
  context=previous;++actor_stats.banks_loaded;refresh_stats();return loaded;
 }
 ObjectId spawn_actor(const ClassDescriptor& type,const char* name,ObjectId parent={}) override {
  if(!m_registry||type.family!=ObjectFamily::Actor||!(type.flags&ObjectClassSpawnable)||(type.flags&ObjectClassSceneManaged))return {};
  if(m_registry->dispatch){ActorSpawnRequest request;request.type=&type;request.name=name;request.logical_parent=parent;defer_spawn(request);return {};}
  const auto* prototype=actor_template_lookup?actor_template_lookup(type.id):nullptr;
  ActorComponentRecord defaults[actor_component_capacity];ActorRecord record;ActorTable table;
  if(!prototype){record.class_id=type.id;record.name=name;
   if(type.default_component_count){if(type.default_component_count>actor_component_capacity)return {};record.components=defaults;record.component_count=type.default_component_count;
    for(size_t i=0;i<record.component_count;++i){defaults[i].data_slot=0;defaults[i].default_index=int16_t(i);}}
   else if(type.domain!=ObjectDomain::None){defaults[0].root=true;defaults[0].data_slot=0;record.components=defaults;record.component_count=1;}
   table.actors=&record;table.count=1;}
  const auto* source=prototype?prototype->table:&table;const size_t count=prototype?prototype->count:1;
  if(!source||!count||count>level_actor_capacity||source->count!=count)return {};
  ActorData* data[level_actor_capacity]={};
  auto rollback=[&](){for(size_t i=0;i<count;++i)if(data[i]&&!data[i]->owner){data[i]->alive=false;++data[i]->generation;}};
  for(size_t i=0;i<count;++i){data[i]=allocate_actor_data(nullptr,nullptr);if(!data[i]){rollback();return {};}
   if(prototype){const auto generation=data[i]->generation;*data[i]=prototype->data[i];data[i]->generation=generation;data[i]->owner=nullptr;data[i]->alive=true;}}
  ObjectId ids[level_actor_capacity]={};Context current{source,ids,data,count,parent,prototype};
  auto* previous=context;context=&current;const auto loaded=instantiate(current,ids,name);context=previous;
  if(loaded!=count){rollback();return {};}refresh_stats();return ids[0];
 }
 ObjectId create_bank_scene_script(const ActorTable& table) {
  if(!m_registry||scene_script().valid())return {};
  const auto* type=find_object_class(table.scene_script_class?table.scene_script_class:SceneScriptActor::static_class_id);
  if(!type||type->family!=ObjectFamily::Actor||(type->flags&ObjectClassAbstract))return {};
  const auto id=create_scene_script(*type,"SceneScript");
  if(id.valid()){++actor_stats.scene_scripts;bind_scene_references(table,id);begin_play_scene_script();}return id;
 }
 size_t bind_scene_references(const ActorTable& table,ObjectId owner) {
  auto* actor=m_registry?m_registry->resolve<Actor>(owner):nullptr;
  if(!actor||!context||!table.bind_reference)return 0;size_t count=0;
  for(size_t i=0;i<table.reference_count;++i){const auto& row=table.references[i];
   if(row.class_id&&!object_class_is_a(actor->class_id(),row.class_id))continue;
   if(row.actor<0||size_t(row.actor)>=context->table->count)continue;
   const auto id=context->actors[size_t(row.actor)];auto* target=m_registry->resolve<Actor>(id);if(!target)continue;
   table.bind_reference(*m_registry,*actor,row.member,row.kind==SceneRefKind::Actor?id:(row.component>=0?target->component_id(size_t(row.component)):target->root_id()));++count;}
  return count;
 }
 void refresh_stats() {
  if(!m_registry)return;const auto& s=m_registry->stats;
  actor_stats.alive=s.alive;actor_stats.peak=s.peak;actor_stats.rejected=s.rejected;actor_stats.spawned=s.spawned;actor_stats.deferred=s.deferred;
  actor_stats.actors=uint32_t(actor_count());actor_stats.components=0;
  for(size_t i=0;i<actor_count();++i)if(auto* a=m_registry->resolve<Actor>(actor_at(i)))actor_stats.components+=uint32_t(a->component_count());
 }
private:
 size_t instantiate(Context& current,ObjectId* ids,const char* name) {
  ActorSpawnRequest requests[level_actor_capacity]={};
  for(size_t i=0;i<current.table->count;++i){const auto& row=current.table->actors[i];requests[i].type=find_object_class(row.class_id);requests[i].name=i==0&&name?name:row.name;requests[i].active=row.active;}
  return current.table->count?spawn_batch(requests,current.table->count,ids,&prepare_all):0;
 }
 static bool prepare_all(Level& base,Actor&,size_t index) {return index!=0||static_cast<SceneLevel&>(base).configure_all();}
 bool configure_all() {
  if(!context)return false;const auto& table=*context->table;
  for(size_t i=0;i<table.count;++i){
   const auto& row=table.actors[i];auto* actor=m_registry->resolve<Actor>(context->actors[i]);
   if(!actor||row.component_count>actor_component_capacity)return false;
   if(row.logical_parent>=0&&size_t(row.logical_parent)>=table.count)return false;
   if(!set_logical_parent(actor->id(),row.logical_parent>=0?context->actors[size_t(row.logical_parent)]:context->parent))return false;
   ObjectId ordered[actor_component_capacity]={};bool first_audio=true;
   for(size_t c=0;c<row.component_count;++c){
    const auto& entry=row.components[c];ObjectId id;
    if(entry.root)id=actor->root_id();else if(entry.default_index>=0)id=actor->component_id(size_t(entry.default_index));else if(const auto* type=find_object_class(entry.class_id))id=add_component_by_class(*actor,*type,entry.name);
    auto* component=m_registry->resolve<ActorComponent>(id);if(!component)return false;if(entry.name)component->set_name(entry.name);
    if(entry.class_id&&!object_class_is_a(component->class_id(),entry.class_id))return false;
    ordered[c]=id;
    if(entry.data_slot>=0){
     if(size_t(entry.data_slot)>=context->data_count)return false;
     auto& data=*context->data[size_t(entry.data_slot)];actor->bind_data(data);
     if(id==actor->root_id()) {
      if(auto* root=m_registry->resolve<SceneComponent3D>(id))root->bind_slot(data);
      else if(auto* root=m_registry->resolve<RectTransformComponent>(id))root->bind_slot(data);
     }
     if(auto* audio=m_registry->resolve<AudioComponent>(id)) {if(first_audio){audio->bind_slot(data);first_audio=false;}else{audio->bind_local();}}
    }
   }
   if(!order_components(*actor,ordered,row.component_count))return false;
   if(actor->data())actor->data()->set_name(actor->name());
  }
  for(size_t i=0;i<table.count;++i){
   const auto& row=table.actors[i];auto* actor=m_registry->resolve<Actor>(context->actors[i]);
   ObjectId components[actor_component_capacity]={};
   for(size_t c=0;c<row.component_count;++c){components[c]=actor->component_id(c);const auto parent=row.components[c].attach_parent;
    if(parent>=0&&(size_t(parent)>=row.component_count||!attach_component(components[c],actor->component_id(size_t(parent)))))return false;}
   if(row.attach_actor>=0){
    if(size_t(row.attach_actor)>=table.count)return false;
    auto* parent=m_registry->resolve<Actor>(context->actors[size_t(row.attach_actor)]);
    const auto target=row.attach_component>=0?parent->component_id(size_t(row.attach_component)):parent->root_id();
    if(!attach_component(actor->root_id(),target))return false;}
   auto* spatial_parent=row.attach_actor>=0?m_registry->resolve<Actor>(context->actors[size_t(row.attach_actor)]):nullptr;
   if(row.attach_actor<0 && row.logical_parent<0 && context->parent.valid()) {
    auto* parent=m_registry->resolve<Actor>(context->parent);
    const auto* own_type=find_object_class(actor->class_id());const auto* parent_type=parent?find_object_class(parent->class_id()):nullptr;
    if(parent_type&&own_type&&own_type->domain!=ObjectDomain::None&&own_type->domain==parent_type->domain) {
     if(!attach_component(actor->root_id(),parent->root_id()))return false;spatial_parent=parent;
    }
   }
   if(actor->data()) {
    auto* parent=spatial_parent;
    actor->data()->parent=parent&&parent->data()?int(handle(parent->data()).index):-1;
   }
   if(row.apply)row.apply(*m_registry,*actor,components,context->actors);
  }
  if(context->prototype&&context->prototype->configure_services){DataHandle handles[level_actor_capacity]={};for(size_t i=0;i<context->data_count;++i)handles[i]=handle(context->data[i]);if(!context->prototype->configure_services(handles,context->data_count))return false;}
  return true;
 }
};
inline ObjectRegistryStorage<EPOK_OBJECT_REGISTRY_CAPACITY> object_registry;
inline SceneLevel level;
inline constexpr size_t object_registry_capacity=EPOK_OBJECT_REGISTRY_CAPACITY;
inline size_t load_actor_bank(const ActorTable& table,ActorData* slots,size_t count){return level.ensure_bound(object_registry)?level.load_bank(table,slots,count):0;}
inline void unload_actor_bank(){level.end_play_all(EndPlayReason::LevelUnloaded);level.refresh_stats();}
inline void collect_object_quarantine(){object_registry.collect_quarantined();}
inline ObjectId actor_for_slot(const ActorData* data){return data&&data->owner?data->owner->id():ObjectId{};}
inline size_t dispatch_slot_trigger(DataHandle self,DataHandle other,TriggerPhase phase){const auto actor=actor_for_slot(self.get());return actor.valid()?dispatch_trigger(level,actor,other,phase):0;}
}
