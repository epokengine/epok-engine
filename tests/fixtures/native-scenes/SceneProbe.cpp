#include "SceneProbe.hpp"
#include <cassert>
using namespace epok;
static unsigned starts=0,ends=0;
static DataHandle previous;
static ObjectId previous_component;
static bool queue_on_end=false,queue_on_begin=false;
static TransitionOptions options(){
    TransitionOptions o;o.fade_out_ms=40;o.fade_in_ms=40;
    o.loading.text="Scene probe";o.loading.color[0]=30;
    static const uint16_t pixels[]={31,992};
    static const LoadingImage image{pixels,2,1};o.loading.image=&image;
    return o;
}
void SceneProbe::begin_play(){
    assert(starts==ends);
    if(starts){assert(!previous.get());assert(!previous_component.get());}
    ++starts;
    static const uint8_t save[]={11,22,33,44};
    if(starts==1)assert(memory_card.write("TRANSITION","Native scene test",save,sizeof(save)));
    else {assert(!memory_card.busy());assert(memory_card.status().state==CardState::Succeeded);assert(memory_card.read("TRANSITION"));}
    assert(memory_card.status().request==starts);
    auto* data=get_owner()->data();previous=handle(data);previous_component=id();
    data->transform.position[0]=Fixed(int32_t(current_scene()*100+starts)*4096,Fixed::RAW);
    data->transform.position[1]=Fixed(int32_t(ends)*4096,Fixed::RAW);
    // A new source at the same pool address must not reuse its completion token.
    data->audio.enabled=true;data->audio.clip=int16_t(current_scene());data->audio.play();
    if(auto* second=find_actor_data("Only B")){
        second->camera=true;assert(set_active_camera(second));
        second->transform.position[0]=10.0;second->collider.enabled=true;
    }else assert(!active_camera().get());
    Fixed origin[3]={0.0,0.0,0.0},delta[3]={20.0,0.0,0.0};
    const auto hit=raycast(origin,delta,0xffffffffu,data);
    assert((hit.entity>=0)==(current_scene()==1));
    const auto rejected=scene_rejected_count();
    assert(!request_scene("Excluded"));assert(!request_scene(size_t(999)));assert(!request_scene(static_cast<const char*>(nullptr)));
    assert(scene_rejected_count()==rejected+3);
    if(queue_on_begin&&current_scene()==1){queue_on_begin=false;assert(request_scene(size_t(0),options()));}
}
void SceneProbe::tick(Fixed){
    assert(time.frame_microseconds<=16667); // loading must never accumulate ticks
    if(!memory_card.busy()){
        assert(memory_card.status().state==CardState::Succeeded&&memory_card.size()==4);
        assert(memory_card.data()[0]==11&&memory_card.data()[3]==44);
    }
    auto* data=get_owner()->data();
    data->transform.position[2]=data->audio.is_playing()?Fixed(4096,Fixed::RAW):Fixed(0,Fixed::RAW);
    if(input.pressed(Button::Cross)){assert(request_scene("B",options()));}
    if(input.pressed(Button::Circle)){auto o=options();o.fade_out_ms=o.fade_in_ms=0;assert(request_scene(size_t(0),o));}
    if(input.pressed(Button::Square)){queue_on_end=true;assert(request_scene("B",options()));}
    if(input.pressed(Button::Triangle)){queue_on_begin=true;assert(request_scene("B",options()));}
}
void SceneProbe::end_play(EndPlayReason reason){
    assert(reason==EndPlayReason::LevelUnloaded);assert(starts==ends+1);++ends;
    if(queue_on_end){queue_on_end=false;assert(request_scene(size_t(0),options()));}
}
