#pragma once
#include "scene_lifecycle.hpp"
namespace epok {
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
bool scene_loading(){return pending_scene>=0||scene_file_waiting()
#ifdef EPOK_TRANSITIONS
    ||transition.busy()
#endif
    ;}
// The outgoing scene is gone and the next bank's file is loading behind the
// loading screen. Nothing of the bank runs until its file is verified.
inline bool scene_file_tick(psyqo::GPU& gpu){
#ifdef EPOK_TRANSITIONS
    if(transition.phase!=TransitionPhase::Loading||!transition.presented)return false;
#endif
    scene_file_service(gpu.now());
    if(scene_file_stats.state==SceneFileState::Failed){
#ifdef EPOK_TRANSITIONS
        transition.fail();
#endif
        return false;
    }
    const size_t next=size_t(scene_file_bank);
    if(!scene_bank_ready(next))return false;
    scene_file_bank=-1;
    scene_start_bank(next);
    return true;
}
inline bool scene_tick(psyqo::GPU& gpu){
#ifdef EPOK_ACTOR_TABLES
    install_actor_service_hooks();
#endif
#ifdef EPOK_TRANSITIONS
    transition.advance(gpu.now());
    scene_stats.waiting=scene_loading();
#endif
    if constexpr(scene_file_count>0)if(scene_file_waiting())return scene_file_tick(gpu);
#ifdef EPOK_TRANSITIONS
    if(transition.phase==TransitionPhase::FadeIn||transition.phase==TransitionPhase::Failed||transition.boot)return false;
#endif
    if(pending_scene<0||scene_transitioning)return false;
#ifdef EPOK_TRANSITIONS
    if(transition.phase==TransitionPhase::Idle)transition.begin(pending_transition.options,gpu.now());
    if(transition.phase!=TransitionPhase::Loading||!transition.presented)return false;
#endif
    scene_stop_audio();
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
    const size_t next=scene_retire_bank();
    if constexpr(scene_file_count>0)if(!scene_bank_ready(next)){
        scene_file_bank=int(next);
        scene_file_begin(next,gpu.now());
        return scene_file_tick(gpu);
    }
    scene_start_bank(next);
    return true;
}
}
