#pragma once
#define EPOK_INCLUDE_FROM_UTILITY 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_UTILITY
#include "fixed_math.hpp"
namespace epok {
inline Fixed lerp(Fixed a,Fixed b,Fixed t){if(t.raw()<0)t=0.0;if(t.raw()>4096)t=1.0;return Fixed(int32_t(int64_t(a.raw())+(int64_t(b.raw())-a.raw())*t.raw()/4096),Fixed::RAW);}
// Serialized in authored content as Linear=0, SmoothStep=1, InQuad=2, OutQuad=3.
// Curves are appended only: reordering would reshape every tween already saved.
// Sine, exponential, elastic, back and bounce are absent because no fixed-point
// trigonometric primitive exists on the target and none is being added for them.
enum class Ease {Linear,SmoothStep,InQuad,OutQuad,InOutQuad,InCubic,OutCubic,InOutCubic,InQuart,OutQuart,InOutQuart,InQuint,OutQuint,InOutQuint,InCirc,OutCirc,InOutCirc};
// Replay plan for one tween. Restart repeats the same leg; PingPong swaps the
// endpoints on every other leg. Neither name may be a Lua keyword, which rules
// out the obvious `repeat`/`until` spellings.
enum class TweenLoop {None,Restart,PingPong};
// Q12 easing. Every curve returns exactly 0 at t=0 and exactly 4096 at t=1, which
// is what lets Tween::advance land on `to` the moment elapsed reaches duration.
// The In/Out halves double the argument before taking the power rather than
// scaling a truncated power afterwards: scaling afterwards costs an order of
// magnitude of accuracy by the fifth power. SmoothStep, InQuad and OutQuad keep
// their original expressions so already authored tweens keep their exact shape;
// OutQuad therefore truncates the whole product instead of only the square.
inline Fixed ease(Fixed t,Ease kind){
    if(t.raw()<0)t=0.0;if(t.raw()>4096)t=1.0;
    using fixed_math::powq;using fixed_math::q_sqrt;
    const int32_t u=t.raw(),v=4096-u;const bool rising=u*2<=4096;
    const auto q=[](int32_t raw){return Fixed(raw,Fixed::RAW);};
    switch(kind){
        case Ease::SmoothStep:return t*t*(Fixed(3.0)-t*2);
        case Ease::InQuad:return t*t;
        case Ease::OutQuad:return t*(Fixed(2.0)-t);
        case Ease::InOutQuad:return q(rising?powq(u*2,2)/2:4096-powq(v*2,2)/2);
        case Ease::InCubic:return q(powq(u,3));
        case Ease::OutCubic:return q(4096-powq(v,3));
        case Ease::InOutCubic:return q(rising?powq(u*2,3)/2:4096-powq(v*2,3)/2);
        case Ease::InQuart:return q(powq(u,4));
        case Ease::OutQuart:return q(4096-powq(v,4));
        case Ease::InOutQuart:return q(rising?powq(u*2,4)/2:4096-powq(v*2,4)/2);
        case Ease::InQuint:return q(powq(u,5));
        case Ease::OutQuint:return q(4096-powq(v,5));
        case Ease::InOutQuint:return q(rising?powq(u*2,5)/2:4096-powq(v*2,5)/2);
        case Ease::InCirc:return q(4096-q_sqrt(4096-powq(u,2)));
        case Ease::OutCirc:return q(q_sqrt(4096-powq(v,2)));
        case Ease::InOutCirc:return q(rising?(4096-q_sqrt(4096-powq(u*2,2)))/2:(4096+q_sqrt(4096-powq(v*2,2)))/2);
        default:return t;
    }
}
// Value tween: the project applies its output to any property. It does not retain
// raw references to entity fields that could be invalidated by a scene switch.
class Tween {
    Fixed from=0.0,to=0.0,duration=1.0,elapsed=0.0,delay=0.0;Ease easing=Ease::Linear;
    TweenLoop looping=TweenLoop::None;uint32_t cycles=1;
    bool running=false,finished=false,reversed=false;
public:
    bool start(Fixed a,Fixed b,Fixed seconds,Ease mode=Ease::Linear){return schedule(a,b,seconds,mode,0.0,TweenLoop::None,1);}
    // `wait` holds the first leg back without shortening it. `legs` counts the
    // duration spans to play and 0 asks for an unbounded replay; without a loop
    // mode exactly one leg plays whatever `legs` says. A zero duration is already
    // finished on arrival, so it ignores the wait rather than deferring `to`.
    bool schedule(Fixed a,Fixed b,Fixed seconds,Ease mode,Fixed wait,TweenLoop loop,uint32_t legs){
        if(seconds.raw()<0||wait.raw()<0)return false;
        from=a;to=b;duration=seconds;elapsed=0.0;delay=wait;easing=mode;looping=loop;
        cycles=loop==TweenLoop::None?1:legs;running=seconds.raw()>0;finished=!running;reversed=false;return true;
    }
    Fixed ratio() const{return duration.raw()==0?Fixed(4096,Fixed::RAW):Fixed(int32_t(int64_t(elapsed.raw())*4096/duration.raw()),Fixed::RAW);}
    // The eased interpolation parameter. A reversed leg mirrors the curve about
    // both axes, which is the same shape with the endpoints swapped and keeps both
    // turnarounds exact. Vector tweens read this value and lerp each component
    // with it, so they agree with three scalar tweens raw unit for raw unit; a zero
    // duration answers 1 so both forms report `to` whatever leg a record claims.
    Fixed alpha() const{if(duration.raw()==0)return Fixed(4096,Fixed::RAW);const auto eased=ease(ratio(),easing);return reversed?Fixed(4096-eased.raw(),Fixed::RAW):eased;}
    Fixed value() const{return duration.raw()==0?to:lerp(from,to,alpha());}
    // Whole completed legs are counted by one division, so a long delta cannot
    // spin here and an unbounded ping-pong keeps the correct leg parity.
    Fixed advance(Fixed dt){
        if(running&&dt.raw()>0){
            int64_t step=dt.raw();
            if(delay.raw()>0){const int64_t left=int64_t(delay.raw())-step;delay=Fixed(int32_t(left>0?left:0),Fixed::RAW);step=left>0?0:-left;}
            int64_t next=int64_t(elapsed.raw())+step;
            if(next>=duration.raw()){
                const int64_t legs=next/duration.raw();
                if(cycles&&int64_t(cycles)<=legs){if(looping==TweenLoop::PingPong&&((cycles-1)&1))reversed=!reversed;next=duration.raw();cycles=1;running=false;finished=true;}
                else{next-=legs*duration.raw();if(cycles)cycles-=uint32_t(legs);if(looping==TweenLoop::PingPong&&(legs&1))reversed=!reversed;}
            }
            elapsed=Fixed(int32_t(next),Fixed::RAW);
        }
        return value();
    }
    void cancel(){running=finished=false;}bool playing()const{return running;}
    bool take_completion(){bool result=finished;finished=false;return result;}
    Fixed start_value()const{return from;}Fixed end_value()const{return to;}
    Fixed duration_value()const{return duration;}Fixed elapsed_value()const{return elapsed;}
    Ease ease_kind()const{return easing;}bool completion_pending()const{return finished;}
    Fixed delay_value()const{return delay;}TweenLoop loop_mode()const{return looping;}
    uint32_t cycles_remaining()const{return cycles;}bool reversed_leg()const{return reversed;}
    void restore(Fixed a,Fixed b,Fixed seconds,Fixed progress,Ease mode,bool active,bool completion,Fixed wait=0.0,TweenLoop loop=TweenLoop::None,uint32_t legs=1,bool reverse=false){
        from=a;to=b;duration=seconds;elapsed=progress;easing=mode;running=active;finished=completion;delay=wait;looping=loop;cycles=legs;reversed=reverse;
    }
};
struct QueueEvent {uint16_t kind=0;int32_t value=0;DataHandle source;};
template<size_t Capacity=64> class EventQueue {
    static_assert(Capacity>0);QueueEvent events[Capacity];size_t first=0,count=0;
public:
    uint32_t dropped=0;
    bool emit(QueueEvent event){if(count==Capacity){++dropped;return false;}events[(first+count)%Capacity]=event;++count;return true;}
    bool poll(QueueEvent& event){if(!count)return false;event=events[first];first=(first+1)%Capacity;--count;return true;}
    void clear(){first=count=0;}size_t size()const{return count;}
    size_t snapshot(QueueEvent* output,size_t capacity)const{const size_t written=count<capacity?count:capacity;if(output)for(size_t i=0;i<written;++i)output[i]=events[(first+i)%Capacity];return written;}
    void restore(const QueueEvent* input,size_t length,uint32_t dropped_count=0){clear();dropped=dropped_count;if(!input)return;const size_t written=length<Capacity?length:Capacity;for(size_t i=0;i<written;++i)emit(input[i]);if(length>Capacity)dropped+=uint32_t(length-Capacity);}
};
struct SequenceStep {Fixed duration;uint16_t event=0;};
class Sequence {
    SequenceStep steps[256];size_t count=0,index=0;Fixed elapsed=0.0;bool running=false,completed=false;
public:
    bool start(const SequenceStep* data,size_t length){if(!data||!length||length>256)return false;for(size_t i=0;i<length;++i)if(data[i].duration.raw()<0)return false;for(size_t i=0;i<length;++i)steps[i]=data[i];count=length;index=0;elapsed=0.0;running=true;completed=false;return true;}
    template<size_t N>void advance(Fixed dt,EventQueue<N>& queue,DataHandle source={}){if(!running||dt.raw()<0)return;int64_t total=int64_t(elapsed.raw())+dt.raw();for(size_t budget=0;running&&budget<256;++budget){if(total<steps[index].duration.raw())break;total-=steps[index].duration.raw();if(steps[index].event)queue.emit({steps[index].event,0,source});if(++index==count){running=false;completed=true;}}elapsed=Fixed(int32_t(total>INT32_MAX?INT32_MAX:total),Fixed::RAW);}
    void cancel(){running=completed=false;}bool playing()const{return running;}
    bool take_completion(){bool result=completed;completed=false;return result;}
};
// Generic focus/navigation independent of menu meaning. Disabled or destroyed
// targets are skipped and callers consume activation/cancel events themselves.
template<size_t Capacity=64>class Focus {
    DataHandle targets[Capacity];size_t count=0;int selected=-1;
    bool available(size_t i)const{auto* e=targets[i].get();return e&&is_active(e);}
public:
    bool add(DataHandle entity){if(!entity||count==Capacity)return false;targets[count++]=entity;if(selected<0&&available(count-1))selected=int(count-1);return true;}
    void clear(){count=0;selected=-1;}
    size_t size()const{return count;}
    DataHandle current()const{return selected>=0&&size_t(selected)<count&&available(size_t(selected))?targets[selected]:DataHandle{};}
    bool move(int direction,bool wrap=true){if(!count||!direction)return false;int step=direction>0?1:-1;int next=selected<0?(step>0?-1:int(count)):selected;for(size_t i=0;i<count;++i){next+=step;if(next<0||next>=int(count)){if(!wrap)return false;next=next<0?int(count)-1:0;}if(available(size_t(next))){selected=next;return true;}}selected=-1;return false;}
    template<size_t N>void navigate(EventQueue<N>& queue,unsigned port=0){if(input.frame_pressed(Button::Up,port))move(-1);if(input.frame_pressed(Button::Down,port))move(1);auto target=current();if(target&&input.frame_pressed(Button::Cross,port))queue.emit({1,selected,target});if(input.frame_pressed(Button::Circle,port))queue.emit({2,selected,target});}
};
// List layout in native HUD pixels; anchoring remains the RectTransform system.
inline void layout_list(DataHandle* children,size_t count,Fixed item_extent,Fixed spacing,bool vertical=true){
    if(!children||item_extent.raw()<0)return;Fixed cursor=0.0;
    for(size_t i=0;i<count;++i)if(auto* e=children[i].get())if(auto* r=e->get<RectTransform>()){
        r->anchor_min[0]=r->anchor_max[0]=r->pivot[0]=0.0;r->anchor_min[1]=r->anchor_max[1]=r->pivot[1]=1.0;
        r->position[vertical?1:0]=vertical?-cursor:cursor;r->size[vertical?1:0]=item_extent;cursor+=item_extent+spacing;
    }
}

struct EPOK_VALUE(Id="d8df46a8-ab62-44a9-8279-420a3765b2d1") FocusSnapshot {bool valid=false;ObjectId current{};uint32_t count=0;};
// Layout is append-only: saved Blueprint graphs address split members by dotted
// name, so a member added at the end is simply absent from older graphs and
// takes its pin default there. `cycles_remaining` is 0 in a hand-built record,
// which reads as unbounded, so gameplay_tween forces one leg when loop is None.
struct EPOK_VALUE(Id="5e278107-d8f8-4a76-8c72-568e0d4d3fc2") GameplayTweenState {Fixed from=0.0,to=0.0,duration=0.0,elapsed=0.0;Ease easing=Ease::Linear;bool running=false,completion_pending=false;Fixed delay=0.0;TweenLoop loop=TweenLoop::None;uint32_t cycles_remaining=0;bool reversed=false;};
struct EPOK_VALUE(Id="31e41da8-6334-47f0-99f6-692949bb6f17") TweenAdvanceSample {GameplayTweenState state{};Fixed value=0.0;bool completed=false;};
struct EPOK_VALUE(Id="9a3e9e93-eade-4991-b813-fc2296542a16") GameplayEventSample {uint32_t kind=0;int32_t value=0;ObjectId source{};};
struct EPOK_VALUE(Id="15b9944c-8b33-4e23-9951-f4c91c4cf486") GameplayEventQueue4 {GameplayEventSample item0{},item1{},item2{},item3{};uint32_t count=0,dropped=0;};
struct EPOK_VALUE(Id="52ec1051-39e5-4360-a681-65ff36b9f5e4") EventQueueMutation {GameplayEventQueue4 state{};bool accepted=false;};
struct EPOK_VALUE(Id="c2b92619-e338-42d8-95c1-0ed3862fa361") EventQueuePoll {GameplayEventQueue4 state{};GameplayEventSample event{};bool valid=false;};

inline ActorData* utility_actor_data(ObjectId id){auto* actor=active_object_registry?active_object_registry->resolve<Actor>(id):nullptr;return actor?actor->data():nullptr;}
inline ObjectId utility_actor_id(DataHandle value){auto* data=value.get();return data&&data->owner?data->owner->id():ObjectId{};}
inline Focus<16>& gameplay_focus(){static Focus<16> value;return value;}
inline GameplayTweenState gameplay_tween_state(const Tween& value){return {value.start_value(),value.end_value(),value.duration_value(),value.elapsed_value(),value.ease_kind(),value.playing(),value.completion_pending(),value.delay_value(),value.loop_mode(),value.cycles_remaining(),value.reversed_leg()};}
inline Tween gameplay_tween(GameplayTweenState state){Tween value;value.restore(state.from,state.to,state.duration,state.elapsed,state.easing,state.running,state.completion_pending,state.delay,state.loop,state.loop==TweenLoop::None?1u:state.cycles_remaining,state.reversed);return value;}
inline GameplayEventSample gameplay_event(QueueEvent value){return {value.kind,value.value,utility_actor_id(value.source)};}
inline QueueEvent gameplay_event(GameplayEventSample value){auto* source=utility_actor_data(value.source);return {uint16_t(value.kind>65535?65535:value.kind),value.value,source?handle(source):DataHandle{}};}
inline GameplayEventQueue4 gameplay_event_queue(EventQueue<4>& value){QueueEvent events[4]{};GameplayEventQueue4 result;result.count=uint32_t(value.snapshot(events,4));result.dropped=value.dropped;GameplayEventSample* output[4]={&result.item0,&result.item1,&result.item2,&result.item3};for(uint32_t i=0;i<result.count;++i)*output[i]=gameplay_event(events[i]);return result;}
inline EventQueue<4> gameplay_event_queue(GameplayEventQueue4 state){EventQueue<4> result;QueueEvent events[4]={gameplay_event(state.item0),gameplay_event(state.item1),gameplay_event(state.item2),gameplay_event(state.item3)};result.restore(events,state.count<4?state.count:4,state.dropped+(state.count>4?state.count-4:0));return result;}

struct EPOK_FUNCTION_LIBRARY(Category="Focus", Capability=focus, Id="eea9a8a5-9822-436e-a827-a7822e4ae740") FocusLibrary {
    EPOK_FUNCTION(BlueprintCallable, Id="ddc6063a-8340-4c1b-8d17-b68129520d6b") static void clear(){gameplay_focus().clear();}
    EPOK_FUNCTION(BlueprintCallable, Id="d36fd319-35cc-4417-916f-f77224f89389") static bool add(ObjectId actor){auto* data=utility_actor_data(actor);return data&&gameplay_focus().add(handle(data));}
    EPOK_FUNCTION(BlueprintCallable, Id="76ea98b4-1881-46ca-89ec-539cab868741") static bool move(int32_t direction,bool wrap){return gameplay_focus().move(direction,wrap);}
    EPOK_FUNCTION(BlueprintCallable, Id="12c57f55-21a7-4b42-978c-858244566247") static GameplayEventQueue4 navigate(uint32_t port){EventQueue<4> events;gameplay_focus().navigate(events,port);return gameplay_event_queue(events);}
    EPOK_FUNCTION(BlueprintPure, Id="56dac2da-ea68-4573-b1c2-2a00813fb66b") static FocusSnapshot snapshot(){FocusSnapshot result;const auto current=gameplay_focus().current();result.valid=bool(current);result.current=utility_actor_id(current);result.count=uint32_t(gameplay_focus().size());return result;}
    EPOK_FUNCTION(BlueprintCallable, Id="75e0266a-ac5a-427a-af18-756642656502") static void layout(ObjectBatch8 actors,Fixed item_extent,Fixed spacing,bool vertical){DataHandle items[8]{};const ObjectId ids[8]={actors.item0,actors.item1,actors.item2,actors.item3,actors.item4,actors.item5,actors.item6,actors.item7};const uint32_t count=actors.count<8?actors.count:8;for(uint32_t i=0;i<count;++i){auto* data=utility_actor_data(ids[i]);items[i]=data?handle(data):DataHandle{};}layout_list(items,count,item_extent,spacing,vertical);}
};
struct EPOK_FUNCTION_LIBRARY(Category="Utilities", Id="6499b0ab-c987-4e1b-b3f3-4918236ca9f1") UtilityLibrary {
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="96f61f10-0581-43a7-8b91-f1564e3f5a02") static GameplayTweenState tween_start(Fixed from,Fixed to,Fixed seconds,Ease easing){Tween value;value.start(from,to,seconds,easing);return gameplay_tween_state(value);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="8a083901-1054-40d6-9395-0230935d8ad1") static TweenAdvanceSample tween_advance(GameplayTweenState state,Fixed delta_seconds){auto value=gameplay_tween(state);TweenAdvanceSample result;result.value=value.advance(delta_seconds);result.completed=value.completion_pending();result.state=gameplay_tween_state(value);return result;}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="246c8362-87f8-479d-9b0f-352cc292ac27") static GameplayTweenState tween_cancel(GameplayTweenState state){auto value=gameplay_tween(state);value.cancel();return gameplay_tween_state(value);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="ee9dc203-4641-43aa-a6e4-e270536a1458") static Fixed tween_value(GameplayTweenState state){return gameplay_tween(state).value();}
    // The full plan: `delay_seconds` holds the first leg back, `loop` and `legs`
    // decide the replay, and `legs` 0 loops without end. A rejected plan returns
    // a cancelled state rather than a half-applied one.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="80e1ced0-392c-4534-85bf-d8bef9d016ff") static GameplayTweenState tween_schedule(Fixed from,Fixed to,Fixed seconds,Ease easing,Fixed delay_seconds,TweenLoop loop,uint32_t legs){Tween value;value.schedule(from,to,seconds,easing,delay_seconds,loop,legs);return gameplay_tween_state(value);}
    // The easing catalogue on its own, for curves applied to something that is
    // not a tween. `t` is clamped to 0..1 and both endpoints are exact.
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="15fc8b3a-80e8-4757-8216-38c89d3e3c7c") static Fixed ease(Fixed t,Ease easing){return epok::ease(t,easing);}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="680b43e9-1329-4382-90bd-48a0cf21fca6") static GameplayEventQueue4 event_queue_clear(){return {};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="07803730-b04c-4917-af7a-30abf3695271") static EventQueueMutation event_queue_emit(GameplayEventQueue4 state,uint32_t kind,int32_t value,ObjectId source){auto queue=gameplay_event_queue(state);EventQueueMutation result;result.accepted=queue.emit(gameplay_event(GameplayEventSample{kind,value,source}));result.state=gameplay_event_queue(queue);return result;}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="3d9a1ba0-a7b8-4773-aaac-f841c7e7a0da") static EventQueuePoll event_queue_poll(GameplayEventQueue4 state){auto queue=gameplay_event_queue(state);EventQueuePoll result;QueueEvent value;result.valid=queue.poll(value);if(result.valid)result.event=gameplay_event(value);result.state=gameplay_event_queue(queue);return result;}
};
}
