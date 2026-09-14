#pragma once
#include "particle_effect_runtime.hpp"
#include "timeline_service.hpp"
namespace epok {
Affine<Fixed> effect_world(DataHandle);
Affine<Fixed> effect_matrix(const Transform&);
void remove_effect_particles(EffectLayerHandle);
inline effects::Stats effect_stats;
namespace effects {
inline Pool pool(timeline::sequences,&remove_effect_particles);
struct Component {
    const Asset* asset=nullptr;
    DataHandle owner;
    timeline::BoundTarget bindings[timeline::slot_limit];
    Handle playback;
    uint32_t seed=0;
    LayerInitializer initialize=nullptr;
    bool enabled=true,automatic=true,started=false;
};
inline Component components[capacity];
inline uint16_t component_count=0;
inline void publish_stats(){effect_stats=pool.stats;timeline::publish_stats();}
inline Component* component(DataHandle owner){
    if(!owner.get())return nullptr;
    for(uint16_t i=0;i<component_count;++i)if(bp::same_owner(owner,components[i].owner))return &components[i];
    return nullptr;
}
inline Component* configure(DataHandle owner,const Asset& asset,bool enabled,bool automatic,uint32_t seed){
    if(!owner.get())return nullptr;
    for(uint16_t i=0;i<capacity;++i){auto& value=components[i];
        if(i<component_count&&value.owner.get()&&!bp::same_owner(value.owner,owner))continue;
        if(i<component_count)pool.stop(value.playback);
        value={};value.owner=owner;value.asset=&asset;value.enabled=enabled;value.automatic=automatic;value.seed=seed;
        if(i>=component_count)component_count=i+1;publish_stats();return &value;
    }
    if(pool.stats.dropped<UINT32_MAX)++pool.stats.dropped;publish_stats();return nullptr;
}
inline Handle play(DataHandle owner){
    auto* value=component(owner);if(!value||!value->enabled||!value->asset)return {};
    pool.stop(value->playback);value->started=true;
    value->playback=pool.spawn(*value->asset,effect_world(owner),blueprint_scene_generation,value->bindings,owner,value->seed,value->initialize);
    publish_stats();return value->playback;
}
inline Handle spawn(const Asset& asset,const Transform& transform,uint32_t seed=0,DataHandle owner={},const timeline::BoundTarget* bindings=nullptr){
    const auto result=pool.spawn(asset,effect_matrix(transform),blueprint_scene_generation,bindings,owner,seed);publish_stats();return result;
}
inline bool stop(Handle handle){const bool result=pool.stop(handle);publish_stats();return result;}
inline void prepare(){
    for(uint16_t i=0;i<component_count;++i){auto& value=components[i];
        if(!value.owner.get())continue;
        if(!value.enabled){pool.stop(value.playback);continue;}
        if(value.automatic&&!value.started&&is_active(value.owner.get()))play(value.owner);
        pool.move(value.playback,effect_world(value.owner));
    }
    pool.prepare(blueprint_scene_generation);publish_stats();
}
inline void advance(Fixed dt){
    for(uint16_t i=0;i<component_count;++i)if(components[i].enabled&&components[i].owner.get())pool.move(components[i].playback,effect_world(components[i].owner));
    pool.advance(dt,time.paused());publish_stats();
}
inline void after_timeline(){pool.observe();publish_stats();}
inline void remove_owner(DataHandle owner){pool.cancel_owner(owner);publish_stats();}
inline void reset_scene(){pool.reset();component_count=0;publish_stats();}
}
}
