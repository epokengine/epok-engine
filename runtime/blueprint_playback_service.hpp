#pragma once
// Service definitions are linked once by main.cpp; generated class headers use
// only blueprint_api.hpp declarations, avoiding cooked asset include cycles.
#include "blueprint_api.hpp"
#ifdef EPOK_TIMELINES
#include "timeline_service.hpp"
#endif
#ifdef EPOK_EFFECTS
#include "particle_effect_service.hpp"
#endif
namespace epok::bp::api {
timeline::Handle play_sequence_component(DataHandle target){
#ifdef EPOK_TIMELINES
    return timeline::play(target);
#else
    (void)target;return {};
#endif
}
bool stop_sequence(timeline::Handle handle){
#ifdef EPOK_TIMELINES
    const bool result=timeline::sequences.stop(handle);timeline::publish_stats();return result;
#else
    (void)handle;return false;
#endif
}
bool pause_sequence(timeline::Handle handle){
#ifdef EPOK_TIMELINES
    return timeline::sequences.pause(handle,true);
#else
    (void)handle;return false;
#endif
}
bool resume_sequence(timeline::Handle handle){
#ifdef EPOK_TIMELINES
    return timeline::sequences.pause(handle,false);
#else
    (void)handle;return false;
#endif
}
effects::Handle play_effect_component(DataHandle target){
#ifdef EPOK_EFFECTS
    return effects::play(target);
#else
    (void)target;return {};
#endif
}
bool stop_effect(effects::Handle handle){
#ifdef EPOK_EFFECTS
    return effects::stop(handle);
#else
    (void)handle;return false;
#endif
}
bool pause_effect(effects::Handle handle){
#ifdef EPOK_EFFECTS
    return effects::pool.pause(handle,true);
#else
    (void)handle;return false;
#endif
}
bool burst_effect(effects::Handle handle,uint32_t count){
#ifdef EPOK_EFFECTS
    const bool result=effects::pool.burst(handle,count);effects::publish_stats();return result;
#else
    (void)handle;(void)count;return false;
#endif
}
bool resume_effect(effects::Handle handle){
#ifdef EPOK_EFFECTS
    return effects::pool.pause(handle,false);
#else
    (void)handle;return false;
#endif
}
timeline::Handle effect_sequence(effects::Handle handle){
#ifdef EPOK_EFFECTS
    return effects::pool.sequence(handle);
#else
    (void)handle;return {};
#endif
}
PlaybackSnapshot playback_snapshot(const PlaybackWait& wait){
    if(wait.effect_completion){
#ifdef EPOK_EFFECTS
        return effects::pool.snapshot(wait.effect);
#else
        return {};
#endif
    }
#ifdef EPOK_TIMELINES
    return timeline::sequences.snapshot(wait.sequence,wait.asset,wait.marker);
#else
    return {};
#endif
}
}
