#define EPOK_TRANSITIONS 1
#include "../../runtime/transition.hpp"
#define EPOK_ACTOR_TABLE_FIXTURE_ONLY
#include "test_actor_tables.cpp"
#include <cstring>
int main(){
#ifdef _MSC_VER
 _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
 reset();load_scene();transition={};pending_transition={};scene_stats={};
 auto* actor=object_registry.resolve<Actor>(level.actor_at(0));auto* data=actor->data();
 data->audio.enabled=true;const auto old=handle(data);const auto stops=audio_stops;
 psyqo::GPU gpu;TransitionOptions options;options.fade_out_ms=400;options.fade_in_ms=200;
 char message[]="Next level";options.loading.text=message;
 assert(request_scene("Bank",options));message[0]='X';
 assert(!scene_tick(gpu)&&old.get()&&audio_stops==stops);
 assert(std::strcmp(transition.options.loading.text,"Next level")==0);
 gpu.clock=200000;assert(!scene_tick(gpu)&&transition_audio_gain()==2048&&old.get());
 gpu.clock=400000;assert(!scene_tick(gpu)&&transition.loading()&&audio_stops==stops);
 assert(!scene_tick(gpu)&&old.get());
 transition.presented=true;music_active=&data->audio;music_lookup=true;
 assert(!scene_tick(gpu)&&audio_stops==stops+object_count&&old.get());
 assert(!scene_tick(gpu)&&audio_stops==stops+object_count);
 music_active=nullptr;music_lookup=false;request_on_end=true;
 assert(scene_tick(gpu)&&!old.get());assert(transition_audio_gain()==0&&scene_loading());
 assert(pending_scene==0);transition.loaded(gpu.clock);
 gpu.clock=500000;assert(!scene_tick(gpu)&&transition_audio_gain()==2048);
 gpu.clock=600000;assert(!scene_tick(gpu)&&transition.phase==TransitionPhase::FadeOut);
 assert(scene_stats.transitions==1&&scene_stats.waiting);
 std::puts("Actor scene fades: presented loading frame, audio quarantine, and requests queued during end_play passed.");
}
