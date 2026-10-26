#pragma once
// Included after generated banks and lifecycle.hpp on both PSX and Native PC.
// Storage readiness, GPU waits and presentation belong to the platform driver.
namespace epok {
struct SceneStats {uint32_t transitions=0,rejected=0;size_t active=0;bool waiting=false;};
inline SceneStats scene_stats;
inline int pending_scene=-1;
inline bool scene_stopping=false;
inline bool scene_transitioning=false;
#ifdef EPOK_TRANSITIONS
inline TransitionState pending_transition;
#endif
size_t current_scene(){return scene_stats.active;}
uint32_t scene_transition_count(){return scene_stats.transitions;}
uint32_t scene_rejected_count(){return scene_stats.rejected;}
bool scene_waiting(){return scene_stats.waiting;}
bool request_scene(size_t index){
    if(index>=scene_bank_count){++scene_stats.rejected;return false;}
#ifdef EPOK_TRANSITIONS
    pending_transition.begin(TransitionOptions{},0);
#endif
    pending_scene=int(index);scene_stats.waiting=true;return true;
}
bool request_scene(const char* name){
    if(name)for(size_t i=0;i<scene_bank_count;++i){size_t n=0;while(name[n]&&name[n]==scene_banks[i].name[n])++n;if(!name[n]&&!scene_banks[i].name[n])return request_scene(i);}
    ++scene_stats.rejected;return false;
}
#ifdef EPOK_TRANSITIONS
bool request_scene(size_t index,const TransitionOptions& options){
    if(!request_scene(index))return false;
    pending_transition.begin(options,0);return true;
}
bool request_scene(const char* name,const TransitionOptions& options){
    if(!request_scene(name))return false;
    pending_transition.begin(options,0);return true;
}
#endif
inline void scene_stop_audio(){
    if(scene_stopping)return;
    for(size_t i=0;i<object_count;++i)if(objects[i].alive)objects[i].audio.stop();
    scene_stopping=true;
}
// Consume before callbacks: end-play and begin-play may queue the next change.
inline size_t scene_retire_bank(){
    rumble.suspend();
    const size_t next=size_t(pending_scene);pending_scene=-1;scene_transitioning=true;
    bool alive[objects.size()]={};
    for(size_t i=0;i<object_count;++i)alive[i]=objects[i].alive;
    lifecycle_tearing_down=true;
#ifdef EPOK_ACTOR_TABLES
    unload_actor_bank();
#endif
    for(size_t i=0;i<object_count;++i)if(alive[i]){
        objects[i].alive=objects[i].active=false;
        objects[i].generation=next_generation(objects[i].generation);
    }
    reset_runtime_services();
    lifecycle_tearing_down=false;scene_stopping=false;
    scene_stats.active=next;++scene_stats.transitions;
    return next;
}
inline void scene_start_bank(size_t next){
    scene_banks[next].load();
    for(size_t i=0;i<object_count;++i)if(is_active(&objects[i])&&objects[i].audio.enabled&&objects[i].audio.play_on_start&&!objects[i].audio.is_playing())objects[i].audio.play();
    scene_transitioning=false;scene_stats.waiting=scene_loading();
}
}
