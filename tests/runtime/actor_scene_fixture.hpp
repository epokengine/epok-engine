#pragma once
#include "epok.hpp"
#include <array>
#include <cassert>
namespace epok {
#ifndef EPOK_TEST_AUDIO_IMPLEMENTED
inline void AudioSource::play() {}
inline void AudioSource::stop() {}
inline bool AudioSource::is_playing() const {return false;}
#endif
inline const ClassDescriptor object_classes[]={
 {Object::static_class_id,0,ObjectFamily::Object,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor::static_class_id,Object::static_class_id,ObjectFamily::Actor,ObjectDomain::None,0,ObjectClassAbstract},
 {Actor3D::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::World3D,0,6,&object_construct<Actor3D>,&object_destruct,sizeof(Actor3D),alignof(Actor3D),&ObjectPool<Actor3D,8>::acquire,&ObjectPool<Actor3D,8>::release},
 {ActorComponent::static_class_id,Object::static_class_id,ObjectFamily::Component,ObjectDomain::None,7,ObjectClassAbstract},
 {SceneComponent3D::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::World3D,1,ObjectClassRoot},
 {Level::static_class_id,Object::static_class_id,ObjectFamily::Level,ObjectDomain::None,0,ObjectClassAbstract},
 {UIActor::static_class_id,Actor::static_class_id,ObjectFamily::Actor,ObjectDomain::UI,0,6,&object_construct<UIActor>,&object_destruct,sizeof(UIActor),alignof(UIActor),&ObjectPool<UIActor,2>::acquire,&ObjectPool<UIActor,2>::release},
 {UIComponent::static_class_id,ActorComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassAbstract},
 {RectTransformComponent::static_class_id,UIComponent::static_class_id,ObjectFamily::Component,ObjectDomain::UI,4,ObjectClassRoot},
};
inline const size_t object_class_count=sizeof(object_classes)/sizeof(object_classes[0]);
inline std::array<ActorData,4> entities;
inline ObjectRegistryStorage<32> test_registry;
inline Level test_level;
inline ObjectId test_actors[4];
inline ObjectId test_owner(size_t index) {return test_actors[index];}
inline DataHandle handle(const ActorData* value) {
 for(size_t i=0;i<entities.size();++i)if(value==&entities[i]&&value->alive)return {uint16_t(i),value->generation};
 return {};
}
inline ActorData* DataHandle::get() const {return index<entities.size()&&entities[index].alive&&entities[index].generation==generation?&entities[index]:nullptr;}
inline bool is_active(const ActorData* value) {return value&&value->alive&&value->owner&&is_active(value->owner);}
inline void test_restore(size_t index) {
 entities[index]=ActorData{};
 test_actors[index]=test_level.spawn_actor(*find_object_class(Actor3D::static_class_id),"Test");
 auto* actor=test_registry.resolve<Actor3D>(test_actors[index]);assert(actor);
 actor->bind_data(entities[index]);actor->root.bind_slot(entities[index]);
}
inline void test_reset_scene() {
 if(test_level.registry()) {test_level.end_play_all(EndPlayReason::LevelUnloaded);test_registry.release(test_level.id());}
 test_level=Level{};assert(test_level.bind(test_registry));
 for(size_t i=0;i<entities.size();++i)test_restore(i);
}
inline void test_set_active(size_t index,bool value) {assert(test_level.set_active(test_actors[index],value));entities[index].active=value;}
inline void test_invalidate(size_t index) {assert(test_level.destroy_actor(test_actors[index]));}
inline UIActor* test_ui_actor(size_t index) {
 test_invalidate(index);entities[index]=ActorData{};
 test_actors[index]=test_level.spawn_actor(*find_object_class(UIActor::static_class_id),"UI");
 auto* actor=test_registry.resolve<UIActor>(test_actors[index]);assert(actor);
 actor->bind_data(entities[index]);actor->root.bind_slot(entities[index]);return actor;
}
}
