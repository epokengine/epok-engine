#pragma once
#include "blueprint_spawn.hpp"
#include "input.hpp"
namespace epok::bp { int asset_index(uint64_t asset,uint32_t kind); }

// The closed Blueprint node adapter surface. Invalid/stale references never
// access a reused entity slot. Setters address local, not world, transforms.
namespace epok::bp::api {
timeline::Handle play_sequence_component(EntityHandle);
bool stop_sequence(timeline::Handle);
bool pause_sequence(timeline::Handle);
bool resume_sequence(timeline::Handle);
effects::Handle play_effect_component(EntityHandle);
bool stop_effect(effects::Handle);
bool burst_effect(effects::Handle,uint32_t);
bool pause_effect(effects::Handle);
bool resume_effect(effects::Handle);
timeline::Handle effect_sequence(effects::Handle);
PlaybackSnapshot playback_snapshot(const PlaybackWait&);
inline bool valid(EntityHandle target) { return target.get() != nullptr; }
inline Transform transform(EntityHandle target){if(auto* entity=target.get())return entity->transform;return {};}
inline Transform make_transform(const Vector<3>& position,const Vector<3>& rotation,const Vector<3>& scale){
    Transform value;for(size_t i=0;i<3;++i){value.position[i]=position[i];value.rotation[i]=rotation[i];value.scale[i]=scale[i];}return value;
}
inline Vector<3> position(EntityHandle target) {
    Vector<3> value; if (auto* entity = target.get()) for (size_t i=0;i<3;++i) value[i]=entity->transform.position[i]; return value;
}
inline Vector<3> rotation(EntityHandle target) {
    Vector<3> value; if (auto* entity = target.get()) for (size_t i=0;i<3;++i) value[i]=entity->transform.rotation[i]; return value;
}
inline Vector<3> scale(EntityHandle target) {
    Vector<3> value; if (auto* entity = target.get()) for (size_t i=0;i<3;++i) value[i]=entity->transform.scale[i]; return value;
}
inline void set_position(EntityHandle target,const Vector<3>& value) { if (auto* entity=target.get()) for(size_t i=0;i<3;++i)entity->transform.position[i]=value[i]; }
inline void set_rotation(EntityHandle target,const Vector<3>& value) { if (auto* entity=target.get()) for(size_t i=0;i<3;++i)entity->transform.rotation[i]=value[i]; }
inline void set_scale(EntityHandle target,const Vector<3>& value) {
    // The established transform contract requires strictly positive scale.
    for(size_t i=0;i<3;++i)if(value[i].raw()<=0)return;
    if(auto* entity=target.get())for(size_t i=0;i<3;++i)entity->transform.scale[i]=value[i];
}
inline bool held(uint32_t button,uint32_t port) {return button<16&&port<2&&input.held(static_cast<Button>(button),port);}
inline bool pressed(uint32_t button,uint32_t port) {return button<16&&port<2&&input.pressed(static_cast<Button>(button),port);}
inline bool released(uint32_t button,uint32_t port) {return button<16&&port<2&&input.released(static_cast<Button>(button),port);}
inline bool request_scene(uint32_t index) {return epok::request_scene(size_t(index));}
inline void set_active(EntityHandle target,bool active) {if(auto* entity=target.get())epok::set_active(entity,active);}
inline void destroy(EntityHandle target) {if(auto* entity=target.get())epok::destroy_entity(entity);}
inline void play_audio(EntityHandle target) {if(auto* entity=target.get())if(is_active(entity))if(auto* audio=entity->get<AudioSource>())audio->play();}
inline void stop_audio(EntityHandle target) {if(auto* entity=target.get())if(auto* audio=entity->get<AudioSource>())audio->stop();}
inline EntityHandle spawn(ClassId type,EntityHandle parent) {return bp::spawn(type,"Blueprint",parent.get());}
inline EntityHandle spawn_class(ClassId base,ClassId type,EntityHandle parent) {
    const auto* info=bp::find_class(type);
    if(!info||!info->create||!bp::class_is_a(type,base))return {};
    return spawn(type,parent);
}
inline EntityHandle cast(ClassId type,EntityHandle target) {return bp::is_a(target,type)?target:EntityHandle{};}
inline void set_texture(EntityHandle target,uint64_t texture) {
    auto* entity=target.get();if(!entity)return;
    const int index=texture?bp::asset_index(texture,1):-1;if(texture&&index<0)return;
    entity->material.texture=index;
    if(entity->sprite.enabled)entity->sprite.texture=index;
    if(entity->image.enabled)entity->image.texture=index;
}
inline void set_audio_clip(EntityHandle target,uint64_t clip) {
    auto* entity=target.get();if(!entity)return;
    const int index=clip?bp::asset_index(clip,2):-1;if(clip&&index<0)return;
    if(auto* audio=entity->get<AudioSource>()){audio->stop();audio->clip=index;}
}
}
