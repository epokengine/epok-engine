#define EPOK_TRANSITIONS 1
#include "../../runtime/transition.hpp"
#define main legacy_lifecycle_main
#include "lifecycle.cpp"
#undef main
int main(){
    reset();transition={};pending_transition={};
    Observer observer;observer.owner=&objects[0];observer.teardown_test=true;
    Binding list[]={{&observer,0}};bindings={list,1};
    auto old=handle(&objects[0]);psyqo::GPU gpu;
    TransitionOptions options;options.fade_out_ms=400;options.fade_in_ms=200;
    char message[]="Next level";options.loading.text=message;
    assert(request_scene("Second",options));message[0]='X';
    assert(!scene_tick(gpu)&&old.get()&&objects[0].audio.stops==0);
    assert(std::strcmp(transition.options.loading.text,"Next level")==0);
    gpu.clock=200000;assert(!scene_tick(gpu)&&transition_audio_gain()==2048&&old.get());
    gpu.clock=400000;assert(!scene_tick(gpu)&&transition.loading()&&objects[0].audio.stops==0);
    // A black loading frame must reach the display before resource teardown.
    assert(!scene_tick(gpu)&&observer.destroyed==0);
    transition.presented=true;music_active=&objects[0].audio;music_lookup=true;
    assert(!scene_tick(gpu)&&objects[0].audio.stops==1&&old.get());
    assert(!scene_tick(gpu)&&objects[0].audio.stops==1);
    music_active=nullptr;music_lookup=false;
    assert(scene_tick(gpu)&&!old.get()&&observer.destroyed==1&&current_scene()==1);
    assert(transition_audio_gain()==0&&scene_loading());
    // Requests from teardown remain queued throughout the incoming fade.
    assert(pending_scene==0);transition.loaded(gpu.clock);
    gpu.clock=500000;assert(!scene_tick(gpu)&&transition_audio_gain()==2048&&current_scene()==1);
    gpu.clock=600000;assert(!scene_tick(gpu)&&transition.phase==TransitionPhase::FadeOut&&current_scene()==1);
    assert(scene_stats.transitions==1&&scene_stats.waiting);
    std::puts("Scene fade lifetime, deferred loading frame, XA quarantine and queued transitions passed.");
}
