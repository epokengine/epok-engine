#pragma once
#include "timeline_runtime.hpp"
namespace epok {
inline timeline::Stats sequence_stats;
namespace timeline {
struct Component {
    const Asset* asset=nullptr;
    DataHandle owner;
    BoundTarget targets[slot_limit];
    Handle playback;
    bool enabled=true,automatic=true,started=false;
};
inline Director<8> sequences;
inline Component components[8];
inline uint16_t component_count=0;
inline void publish_stats(){sequence_stats=sequences.stats;}
inline Component* component(DataHandle owner){
    if(!owner.get())return nullptr;
    for(uint16_t i=0;i<component_count;++i)if(bp::same_owner(owner,components[i].owner))return &components[i];
    return nullptr;
}
inline Component* configure(DataHandle owner,const Asset& asset,bool enabled,bool automatic){
    if(!owner.get())return nullptr;
    for(uint16_t i=0;i<8;++i){auto& value=components[i];
        if(i<component_count&&value.owner.get()&&!bp::same_owner(value.owner,owner))continue;
        if(i<component_count)sequences.stop(value.playback);
        value={};value.owner=owner;value.asset=&asset;value.enabled=enabled;value.automatic=automatic;
        if(i>=component_count)component_count=i+1;
        publish_stats();return &value;
    }
    sequences.capacity_drop(asset);publish_stats();return nullptr;
}
inline Handle play(DataHandle owner){
    auto* value=component(owner);if(!value||!value->enabled||!value->asset)return {};
    if(sequences.state(value->playback)==State::Playing)sequences.stop(value->playback);
    value->started=true;
    value->playback=sequences.play(*value->asset,owner,value->targets,blueprint_scene_generation);
    publish_stats();return value->playback;
}
inline bool stop(DataHandle owner){auto* value=component(owner);if(!value)return false;const bool stopped=sequences.stop(value->playback);publish_stats();return stopped;}
inline void start_components(){
    for(uint16_t i=0;i<component_count;++i){auto& value=components[i];
        if(value.enabled&&value.automatic&&!value.started&&value.owner.get()&&is_active(value.owner.get()))play(value.owner);
    }
}
inline void advance(Fixed dt){
    for(uint16_t i=0;i<component_count;++i)if(!components[i].enabled)sequences.stop(components[i].playback);
    start_components();sequences.advance(dt,blueprint_scene_generation,time.paused());publish_stats();
}
inline void remove_owner(DataHandle owner){sequences.cancel_owner(owner);publish_stats();}
inline void reset_scene(){sequences.cancel_all();component_count=0;publish_stats();}
}
}
