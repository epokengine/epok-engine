#include <array>
#include <cassert>
#include <cmath>
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
// The float curve each Q12 kind approximates. Host-only: the runtime never
// evaluates this, it is only the reference the integer kernel is measured
// against. Radicands are clamped so rounding at the midpoint cannot go negative.
static double reference(double u,Ease kind){
    const double v=1.0-u;const double w=u*u>1.0?0.0:1.0-u*u,z=v*v>1.0?0.0:1.0-v*v;
    const double a=4.0*u*u>1.0?0.0:1.0-4.0*u*u,b=4.0*v*v>1.0?0.0:1.0-4.0*v*v;
    switch(kind){
        case Ease::SmoothStep:return u*u*(3.0-2.0*u);
        case Ease::InQuad:return u*u;
        case Ease::OutQuad:return 1.0-v*v;
        case Ease::InOutQuad:return u<=0.5?2.0*u*u:1.0-2.0*v*v;
        case Ease::InCubic:return u*u*u;
        case Ease::OutCubic:return 1.0-v*v*v;
        case Ease::InOutCubic:return u<=0.5?4.0*u*u*u:1.0-4.0*v*v*v;
        case Ease::InQuart:return u*u*u*u;
        case Ease::OutQuart:return 1.0-v*v*v*v;
        case Ease::InOutQuart:return u<=0.5?8.0*u*u*u*u:1.0-8.0*v*v*v*v;
        case Ease::InQuint:return u*u*u*u*u;
        case Ease::OutQuint:return 1.0-v*v*v*v*v;
        case Ease::InOutQuint:return u<=0.5?16.0*u*u*u*u*u:1.0-16.0*v*v*v*v*v;
        case Ease::InCirc:return 1.0-std::sqrt(w);
        case Ease::OutCirc:return std::sqrt(z);
        case Ease::InOutCirc:return u<=0.5?(1.0-std::sqrt(a))/2.0:(1.0+std::sqrt(b))/2.0;
        default:return u;
    }
}
// The whole catalogue, swept over every Q12 input. Exact endpoints are what lets
// Tween::advance land on `to`; the other three properties are what makes a curve
// usable for animation at all.
void easing_catalogue(){
    const Ease kinds[]={Ease::Linear,Ease::SmoothStep,Ease::InQuad,Ease::OutQuad,Ease::InOutQuad,
        Ease::InCubic,Ease::OutCubic,Ease::InOutCubic,Ease::InQuart,Ease::OutQuart,Ease::InOutQuart,
        Ease::InQuint,Ease::OutQuint,Ease::InOutQuint,Ease::InCirc,Ease::OutCirc,Ease::InOutCirc};
    // Authored content stores the enumerator, so the order is the wire format:
    // Linear=0, SmoothStep=1, InQuad=2, OutQuad=3, then the appended curves.
    for(unsigned index=0;index<sizeof(kinds)/sizeof(kinds[0]);++index)assert(unsigned(kinds[index])==index);
    assert(unsigned(Ease::InOutCirc)==16);
    for(auto kind:kinds){
        assert(ease(0.0,kind).raw()==0&&ease(1.0,kind).raw()==4096);
        assert(ease(Fixed(-1,Fixed::RAW),kind).raw()==0&&ease(Fixed(4097,Fixed::RAW),kind).raw()==4096);
        // SmoothStep truncates twice, which lets it dip one raw unit at 108 of the
        // 4097 inputs. That is its shipped behaviour and stays; every appended
        // curve truncates once and is non-decreasing without any tolerance.
        const int32_t tolerance=kind==Ease::SmoothStep?1:0;
        int32_t previous=0;double worst=0.0;
        for(int32_t raw=0;raw<=4096;++raw){
            const int32_t value=ease(Fixed(raw,Fixed::RAW),kind).raw();
            assert(value>=0&&value<=4096);      // neither end overshoots
            assert(value+tolerance>=previous);  // monotonic non-decreasing
            previous=value;
            const double error=std::fabs(double(value)-reference(double(raw)/4096.0,kind)*4096.0);
            if(error>worst)worst=error;
        }
        assert(previous==4096);
        assert(worst<=4.0);                 // the measured worst case is 3.85, on SmoothStep
    }
    // The three curves that predate the catalogue keep their original integer
    // expressions bit for bit, so every tween already authored plays unchanged.
    for(int32_t raw=0;raw<=4096;++raw){
        const int64_t u=raw;const Fixed t(raw,Fixed::RAW);
        assert(ease(t,Ease::Linear).raw()==raw);
        assert(ease(t,Ease::SmoothStep).raw()==int32_t(u*u/4096*(12288-2*u)/4096));
        assert(ease(t,Ease::InQuad).raw()==int32_t(u*u/4096));
        assert(ease(t,Ease::OutQuad).raw()==int32_t(u*(8192-u)/4096));
    }
}
// Delay, the three loop modes, the cycle count and the unbounded option.
void tween_delay_and_loops(){
    Tween tween;
    // The wait holds the value at `from` and does not shorten the leg.
    assert(tween.schedule(0.0,10.0,1.0,Ease::Linear,0.5,TweenLoop::None,1));
    assert(tween.advance(0.25)==Fixed(0.0)&&tween.playing()&&tween.delay_value()==Fixed(0.25));
    assert(tween.advance(0.5)==Fixed(2.5)&&tween.delay_value().raw()==0);
    assert(tween.advance(0.75)==Fixed(10.0)&&!tween.playing()&&tween.take_completion());
    // A negative duration or wait is rejected whole, and a zero duration is
    // already finished on arrival so it never defers `to` behind a wait.
    assert(!tween.schedule(0.0,1.0,-1.0,Ease::Linear,0.0,TweenLoop::None,1));
    assert(!tween.schedule(0.0,1.0,1.0,Ease::Linear,-1.0,TweenLoop::None,1));
    assert(tween.schedule(0.0,9.0,0.0,Ease::Linear,4.0,TweenLoop::Restart,0));
    assert(!tween.playing()&&tween.value()==Fixed(9.0)&&tween.take_completion());
    // Without a loop mode the leg count is ignored: exactly one leg plays.
    assert(tween.schedule(0.0,4.0,1.0,Ease::Linear,0.0,TweenLoop::None,7));
    assert(tween.cycles_remaining()==1&&tween.advance(1.0)==Fixed(4.0)&&!tween.playing());
    // Restart replays the same leg and completes only on the last one.
    assert(tween.schedule(0.0,4.0,1.0,Ease::Linear,0.0,TweenLoop::Restart,3));
    assert(tween.advance(1.5)==Fixed(2.0)&&tween.playing()&&!tween.completion_pending()&&tween.cycles_remaining()==2);
    assert(tween.advance(1.0)==Fixed(2.0)&&tween.playing()&&tween.cycles_remaining()==1&&!tween.reversed_leg());
    assert(tween.advance(1.0)==Fixed(4.0)&&!tween.playing()&&tween.take_completion());
    // Ping-pong mirrors the curve on the reverse leg: the same shape with the
    // endpoints swapped, which keeps both turnarounds exact.
    const Fixed half=ease(0.5,Ease::InQuad);
    assert(tween.schedule(0.0,8.0,1.0,Ease::InQuad,0.0,TweenLoop::PingPong,2));
    assert(tween.advance(0.5)==lerp(0.0,8.0,half)&&!tween.reversed_leg());
    assert(tween.advance(1.0)==lerp(0.0,8.0,Fixed(4096-half.raw(),Fixed::RAW))&&tween.reversed_leg());
    assert(tween.advance(1.0)==Fixed(0.0)&&!tween.playing()&&tween.take_completion());
    // Three legs end at `to` again, and the parity survives one long delta.
    assert(tween.schedule(0.0,8.0,1.0,Ease::InQuad,0.0,TweenLoop::PingPong,3));
    assert(tween.advance(3.0)==Fixed(8.0)&&!tween.playing()&&tween.take_completion());
    // An unbounded plan never reports final completion, leg by leg or in one jump.
    assert(tween.schedule(0.0,4.0,1.0,Ease::Linear,0.0,TweenLoop::PingPong,0));
    for(int leg=0;leg<9;++leg){tween.advance(1.0);assert(tween.playing()&&!tween.completion_pending()&&!tween.take_completion());}
    assert(tween.cycles_remaining()==0&&tween.reversed_leg()&&tween.elapsed_value().raw()==0);
    assert(tween.schedule(0.0,4.0,1.0,Ease::Linear,0.0,TweenLoop::PingPong,0));
    assert(tween.advance(1000.5)==Fixed(2.0)&&tween.playing()&&!tween.reversed_leg());
    assert(tween.elapsed_value()==Fixed(0.5)&&!tween.take_completion());
    // The state record round-trips the whole plan, including the mirrored leg.
    const auto state=gameplay_tween_state(tween);
    assert(state.delay.raw()==0&&state.loop==TweenLoop::PingPong&&state.cycles_remaining==0&&!state.reversed);
    assert(gameplay_tween(state).value()==tween.value());
    // A hand-built record with no loop mode plays one leg even though an
    // untouched authoring pin leaves `cycles_remaining` at zero.
    GameplayTweenState built;built.to=6.0;built.duration=1.0;built.running=true;
    assert(built.cycles_remaining==0&&built.loop==TweenLoop::None);
    auto restored=gameplay_tween(built);assert(restored.cycles_remaining()==1);
    assert(restored.advance(5.0)==Fixed(6.0)&&!restored.playing()&&restored.take_completion());
}
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
    tweens_and_easing();easing_catalogue();tween_delay_and_loops();queues_and_sequences();focus_and_layout();std::puts("Q12 utility easing catalogue, tween delay/loops, event handles, sequencer and focus tests passed.");
}
