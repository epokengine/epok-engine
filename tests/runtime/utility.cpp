#include <array>
#include <cassert>
#include <cstdio>
#include <climits>
#ifdef _MSC_VER
#include <crtdbg.h>
#endif
#include "../../runtime/utility.hpp"
namespace epok {
static std::array<ActorData,4> entities;
ActorData* DataHandle::get()const{return index<entities.size()&&entities[index].alive&&entities[index].generation==generation?&entities[index]:nullptr;}
bool is_active(const ActorData* entity){return entity&&entity->alive&&entity->active;}
}
using namespace epok;
void tweens_and_easing(){
    assert(lerp(Fixed(INT32_MIN,Fixed::RAW),Fixed(INT32_MAX,Fixed::RAW),1.0).raw()==INT32_MAX);
    assert(lerp(Fixed(INT32_MAX,Fixed::RAW),Fixed(INT32_MIN,Fixed::RAW),1.0).raw()==INT32_MIN);
    assert(lerp(-10.0,10.0,-1.0)==Fixed(-10.0));assert(lerp(-10.0,10.0,2.0)==Fixed(10.0));
    for(auto mode:{Ease::Linear,Ease::SmoothStep,Ease::InQuad,Ease::OutQuad}){
        assert(ease(0.0,mode).raw()==0&&ease(1.0,mode).raw()==4096);
        for(int raw=0;raw<=4096;++raw){auto value=ease(Fixed(raw,Fixed::RAW),mode);assert(value.raw()>=0&&value.raw()<=4096);}
    }
    Tween tween;assert(tween.start(1.0,3.0,1.0));assert(tween.advance(0.5)==Fixed(2.0));assert(tween.playing()&&!tween.take_completion());
    assert(tween.advance(3.0)==Fixed(3.0)&&!tween.playing());assert(tween.take_completion()&&!tween.take_completion());
    assert(tween.start(0.0,9.0,0.0));assert(tween.value()==Fixed(9.0)&&tween.take_completion());
    assert(tween.start(0.0,10.0,1.0));tween.advance(0.5);tween.cancel();assert(!tween.playing()&&!tween.take_completion()&&tween.advance(1.0)==Fixed(5.0));
    assert(tween.start(0.0,10.0,1.0));assert(!tween.start(0.0,10.0,-1.0));assert(tween.playing());assert(tween.advance(-1.0)==Fixed(0.0));
}
void queues_and_sequences(){
    entities={};DataHandle source{0,entities[0].generation};EventQueue<3> queue;
    assert(queue.emit({1,42,source}));++entities[0].generation;QueueEvent event;assert(queue.poll(event)&&event.value==42&&!event.source.get());
    assert(queue.emit({1,0,{}})&&queue.emit({2,0,{}})&&queue.emit({3,0,{}}));assert(!queue.emit({4,0,{}})&&queue.dropped==1);
    assert(queue.poll(event)&&event.kind==1);assert(queue.emit({4,0,{}}));for(unsigned kind=2;kind<=4;++kind)assert(queue.poll(event)&&event.kind==kind);assert(!queue.poll(event));
    Sequence sequence;SequenceStep steps[]={{0.0,10},{0.25,11},{0.25,12},{0.0,13}};assert(sequence.start(steps,4));
    steps[1].event=99; // A started sequence owns its plan, independent of caller lifetime.
    sequence.advance(0.0,queue);assert(queue.poll(event)&&event.kind==10);assert(sequence.playing());
    sequence.advance(0.75,queue);for(unsigned kind=11;kind<=13;++kind)assert(queue.poll(event)&&event.kind==kind);
    assert(!sequence.playing()&&sequence.take_completion()&&!sequence.take_completion());sequence.advance(1.0,queue);assert(!queue.poll(event));
    assert(sequence.start(steps,4));sequence.cancel();sequence.advance(9.0,queue);assert(!sequence.playing()&&!sequence.take_completion()&&!queue.poll(event));
    SequenceStep zeros[256];for(auto& step:zeros)step={0.0,8};assert(sequence.start(zeros,256));sequence.advance(0.0,queue);assert(sequence.take_completion()&&queue.size()==3&&queue.dropped==254);queue.clear();
    assert(!sequence.start(nullptr,4));assert(!sequence.start(zeros,257));zeros[0].duration=-1.0;assert(!sequence.start(zeros,256));
}
void focus_and_layout(){
    entities={};Focus<4> focus;for(unsigned i=0;i<4;++i){entities[i].active=false;assert(focus.add({uint16_t(i),entities[i].generation}));}
    assert(!focus.current());entities[3].active=true;assert(focus.move(-1,false)&&focus.current().index==3);
    entities[1].active=true;assert(focus.move(-1,false)&&focus.current().index==1);++entities[1].generation;assert(!focus.current());assert(focus.move(1)&&focus.current().index==3);
    EventQueue<4> queue;input.sample(0,true,1<<14);focus.navigate(queue);QueueEvent event;assert(queue.poll(event)&&event.kind==1&&event.source.index==3);
    DataHandle children[3]={{0,entities[0].generation},{1,entities[1].generation},{2,entities[2].generation}};
    for(unsigned i=0;i<3;++i)entities[i].rect.enabled=true;
    layout_list(children,3,12.0,2.0);assert(entities[0].rect.position[1]==Fixed(0.0));assert(entities[2].rect.position[1]==Fixed(-28.0));
    assert(entities[1].rect.size[1]==Fixed(12.0)&&entities[1].rect.anchor_min[1]==Fixed(1.0));
}
int main(){
#ifdef _MSC_VER
    _set_error_mode(_OUT_TO_STDERR);_set_abort_behavior(0,_WRITE_ABORT_MSG|_CALL_REPORTFAULT);
#endif
    tweens_and_easing();queues_and_sequences();focus_and_layout();std::puts("Q12 utility tween/easing, event handles, sequencer and focus tests passed.");
}
