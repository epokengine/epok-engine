#pragma once
#define EPOK_INCLUDE_FROM_BLUEPRINT_API 1
#include "blueprint_spawn.hpp"
#undef EPOK_INCLUDE_FROM_BLUEPRINT_API
#include "input.hpp"
namespace epok::bp { int asset_index(uint64_t asset,uint32_t kind); }

// The closed Blueprint node adapter surface. Invalid/stale references never
// access a reused entity slot. Setters address local, not world, transforms.
namespace epok::bp::api {
timeline::Handle play_sequence_component(DataHandle);
bool stop_sequence(timeline::Handle);
bool pause_sequence(timeline::Handle);
bool resume_sequence(timeline::Handle);
effects::Handle play_effect_component(DataHandle);
bool stop_effect(effects::Handle);
bool burst_effect(effects::Handle,uint32_t);
bool pause_effect(effects::Handle);
bool resume_effect(effects::Handle);
timeline::Handle effect_sequence(effects::Handle);
PlaybackSnapshot playback_snapshot(const PlaybackWait&);
inline bool valid(ObjectId target) { return target.get() != nullptr; }
inline Transform transform(ObjectId target){auto* root=root_component<SceneComponent3D>(target);return root&&root->transform ? *root->transform : Transform{};}
inline Transform make_transform(const Vector<3>& position,const Vector<3>& rotation,const Vector<3>& scale){
    Transform value;for(size_t i=0;i<3;++i){value.position[i]=position[i];value.rotation[i]=rotation[i];value.scale[i]=scale[i];}return value;
}
inline Vector<3> position(ObjectId target) {
    Vector<3> value; auto* root=root_component<SceneComponent3D>(target);
    if(root&&root->transform)for(size_t i=0;i<3;++i)value[i]=root->transform->position[i];return value;
}
inline void set_position(ObjectId target,const Vector<3>& value) {
    auto* root=root_component<SceneComponent3D>(target);if(!root||!root->transform)return;
    for(size_t i=0;i<3;++i)root->transform->position[i]=value[i];
}
inline Vector<3> rotation(ObjectId target) {
    Vector<3> value; auto* root=root_component<SceneComponent3D>(target);
    if(root&&root->transform)for(size_t i=0;i<3;++i)value[i]=root->transform->rotation[i];return value;
}
inline void set_rotation(ObjectId target,const Vector<3>& value) {
    auto* root=root_component<SceneComponent3D>(target);if(!root||!root->transform)return;
    for(size_t i=0;i<3;++i)root->transform->rotation[i]=value[i];
}
inline Vector<3> scale(ObjectId target) {
    Vector<3> value; auto* root=root_component<SceneComponent3D>(target);
    if(root&&root->transform)for(size_t i=0;i<3;++i)value[i]=root->transform->scale[i];return value;
}
inline void set_scale(ObjectId target,const Vector<3>& value) {
    auto* root=root_component<SceneComponent3D>(target);if(!root||!root->transform)return;
    for(size_t i=0;i<3;++i)if(value[i].raw()<=0)return;
    for(size_t i=0;i<3;++i)root->transform->scale[i]=value[i];
}
inline Vector<2> position_2d(ObjectId target) {Vector<2> result;auto* root=root_component<SceneComponent2D>(target);if(root)for(size_t i=0;i<2;++i)result[i]=root->transform.position[i];return result;}
inline void set_position_2d(ObjectId target,const Vector<2>& value) {auto* root=root_component<SceneComponent2D>(target);if(!root)return;
for(size_t i=0;i<2;++i)root->transform.position[i]=value[i];}
inline Vector<2> scale_2d(ObjectId target) {Vector<2> result;auto* root=root_component<SceneComponent2D>(target);if(root)for(size_t i=0;i<2;++i)result[i]=root->transform.scale[i];return result;}
inline void set_scale_2d(ObjectId target,const Vector<2>& value) {auto* root=root_component<SceneComponent2D>(target);if(!root)return;
for(size_t i=0;i<2;++i)if(value[i].raw()<=0)return;
for(size_t i=0;i<2;++i)root->transform.scale[i]=value[i];}
inline Fixed rotation_2d(ObjectId target){auto* root=root_component<SceneComponent2D>(target);return root ? root->transform.rotation : Fixed(0.0);}
inline void set_rotation_2d(ObjectId target,Fixed value){if(auto* root=root_component<SceneComponent2D>(target))root->transform.rotation=value;}
inline Vector<2> rect_position(ObjectId target) {Vector<2> result;auto* root=root_component<RectTransformComponent>(target);if(root&&root->rect)for(size_t i=0;i<2;++i)result[i]=root->rect->position[i];return result;}
inline void set_rect_position(ObjectId target,const Vector<2>& value) {auto* root=root_component<RectTransformComponent>(target);if(root&&root->rect)for(size_t i=0;i<2;++i)root->rect->position[i]=value[i];}
inline Vector<2> rect_size(ObjectId target) {Vector<2> result;auto* root=root_component<RectTransformComponent>(target);if(root&&root->rect)for(size_t i=0;i<2;++i)result[i]=root->rect->size[i];return result;}
inline void set_rect_size(ObjectId target,const Vector<2>& value) {auto* root=root_component<RectTransformComponent>(target);if(root&&root->rect)for(size_t i=0;i<2;++i)root->rect->size[i]=value[i];}
inline bool held(uint32_t button,uint32_t port) {return button<16&&port<2&&input.held(static_cast<Button>(button),port);}
inline bool pressed(uint32_t button,uint32_t port) {return button<16&&port<2&&input.pressed(static_cast<Button>(button),port);}
inline bool released(uint32_t button,uint32_t port) {return button<16&&port<2&&input.released(static_cast<Button>(button),port);}
inline bool request_scene(uint32_t index) {return epok::request_scene(size_t(index));}
inline void set_active(ObjectId target,bool active) {auto* actor=owner_actor(target);auto* level=actor&&active_object_registry?active_object_registry->resolve<Level>(actor->level_id()):nullptr;if(level)level->set_active(actor->id(),active);}
inline void destroy(ObjectId target) {auto* actor=owner_actor(target);auto* level=actor&&active_object_registry?active_object_registry->resolve<Level>(actor->level_id()):nullptr;if(level)level->destroy_actor(actor->id());}
inline void play_audio(ObjectId target) {if(auto* data=object_data(target))if(is_active(data))if(auto* audio=data->get<AudioSource>())audio->play();}
inline void stop_audio(ObjectId target) {if(auto* data=object_data(target))if(auto* audio=data->get<AudioSource>())audio->stop();}
inline ObjectId spawn(ClassId type,ObjectId parent) {return bp::spawn_actor(owner_actor(parent),type,parent);}
inline ObjectId spawn_class(ClassId base,ClassId type,ObjectId parent) {return bp::class_is_a(type,base)?spawn(type,parent):ObjectId{};}
inline ObjectId cast(ClassId type,ObjectId target) {return bp::is_a(target,type)?target:ObjectId{};}
inline timeline::Handle play_sequence_component(ObjectId target) {return play_sequence_component(data_handle(target));}
inline effects::Handle play_effect_component(ObjectId target) {return play_effect_component(data_handle(target));}
inline void set_texture(ObjectId target,uint64_t texture) {
    auto* entity=object_data(target);if(!entity)return;
    const int index=texture?bp::asset_index(texture,1):-1;if(texture&&index<0)return;
    entity->material.texture=index;
    if(entity->sprite.enabled)entity->sprite.texture=index;
    if(entity->image.enabled)entity->image.texture=index;
}
inline void set_audio_clip(ObjectId target,uint64_t clip) {
    auto* entity=object_data(target);if(!entity)return;
    const int index=clip?bp::asset_index(clip,2):-1;if(clip&&index<0)return;
    if(auto* audio=entity->get<AudioSource>()){audio->stop();audio->clip=index;}
}
}
