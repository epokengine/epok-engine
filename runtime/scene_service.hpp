#pragma once
namespace epok {
struct SceneStats {uint32_t transitions=0,rejected=0;size_t active=0;bool waiting=false;};
inline SceneStats scene_stats;
inline int pending_scene=-1;
inline bool scene_stopping=false;
inline bool scene_transitioning=false;
#ifdef EPOK_TRANSITIONS
inline TransitionState pending_transition;
#endif
#ifdef EPOK_ACTOR_TABLES
// ---- object-model service hooks ----------------------------------------------------
// Both predicates are installed rather than hard-wired: runtime/object_model.hpp is
// compiled before runtime/music.hpp defines `music_active` as an inline variable, and
// before the generated scene bank declares the legacy `bindings` table.

// Component-owned AudioSource storage stays quarantined while the XA consumer still
// points at it -- exactly the rule allocate_actor_data (runtime/lifecycle.hpp) applies to a
// legacy slot's `audio`. `music_lookup` means a CD lookup for `music_active` is still in
// flight, so that source is retained even between requests.
inline bool music_retains_audio_source(const AudioSource* source){
    if(!source)return false;
    return music_active==source||music_requested==source||(music_lookup&&music_active==source);
}
inline void install_actor_service_hooks(){
    audio_source_retained=&music_retains_audio_source;
}
#endif
size_t current_scene(){return scene_stats.active;}
bool scene_loading(){return pending_scene>=0
#ifdef EPOK_TRANSITIONS
    ||transition.busy()
#endif
    ;}
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
inline bool scene_tick(psyqo::GPU& gpu){
#ifdef EPOK_ACTOR_TABLES
    install_actor_service_hooks();
#endif
#ifdef EPOK_TRANSITIONS
    transition.advance(gpu.now());
    scene_stats.waiting=scene_loading();
    if(transition.phase==TransitionPhase::FadeIn||transition.phase==TransitionPhase::Failed||transition.boot)return false;
#endif
    if(pending_scene<0||scene_transitioning)return false;
#ifdef EPOK_TRANSITIONS
    if(transition.phase==TransitionPhase::Idle)transition.begin(pending_transition.options,gpu.now());
    if(transition.phase!=TransitionPhase::Loading||!transition.presented)return false;
#endif
    if(!scene_stopping){
        for(size_t i=0;i<object_count;++i)if(objects[i].alive)objects[i].audio.stop();
        scene_stopping=true;
    }
    // Async XA callbacks retain AudioSource pointers until seek/stop completes.
    // Geometry callbacks retain page slots only; source lifetime is held only
    // by XA lookup/action callbacks. Global page IDs survive scene changes.
    if(music_active||music_lookup)return false;
#ifdef EPOK_HAS_SEQUENCES
    // Resident bank pins outlive key-off consumption. Do not reuse scene owners
    // or start the next bank's mailboxes before those physical commands retire.
    if(sequence_retiring())return false;
#endif
    gpu.waitChainIdle();
    // Consume this request before callbacks, so requests from teardown/start
    // remain queued for the following safe frame boundary.
    size_t next=size_t(pending_scene);pending_scene=-1;scene_transitioning=true;
    bool alive[objects.size()]={},active[objects.size()]={};
    for(size_t i=0;i<object_count;++i){alive[i]=objects[i].alive;active[i]=is_active(&objects[i]);}
    lifecycle_tearing_down=true;
#ifdef EPOK_ACTOR_TABLES
    // The scene script ends play first, while the actors and their legacy slots are
    // still alive; then the level tears the actors down. Both precede the legacy
    // binding teardown and reset_runtime_services below.
    unload_actor_bank();
#endif
    for(size_t i=0;i<object_count;++i)if(alive[i]){
        objects[i].alive=objects[i].active=false;
        objects[i].generation=next_generation(objects[i].generation);
    }
    reset_runtime_services();
    lifecycle_tearing_down=false;scene_stopping=false;
    scene_stats.active=next;++scene_stats.transitions;
    scene_banks[next].load();
    for(size_t i=0;i<object_count;++i)if(is_active(&objects[i])&&objects[i].audio.enabled&&objects[i].audio.play_on_start&&!objects[i].audio.is_playing())objects[i].audio.play();
    scene_transitioning=false;scene_stats.waiting=scene_loading();
    return true;
}
}
