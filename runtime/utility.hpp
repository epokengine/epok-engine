#pragma once
#define EPOK_INCLUDE_FROM_UTILITY 1
#include "epok.hpp"
#undef EPOK_INCLUDE_FROM_UTILITY
namespace epok {
inline Fixed lerp(Fixed a,Fixed b,Fixed t){if(t.raw()<0)t=0.0;if(t.raw()>4096)t=1.0;return Fixed(int32_t(int64_t(a.raw())+(int64_t(b.raw())-a.raw())*t.raw()/4096),Fixed::RAW);}
enum class Ease {Linear,SmoothStep,InQuad,OutQuad};
inline Fixed ease(Fixed t,Ease kind){if(t.raw()<0)t=0.0;if(t.raw()>4096)t=1.0;switch(kind){case Ease::SmoothStep:return t*t*(Fixed(3.0)-t*2);case Ease::InQuad:return t*t;case Ease::OutQuad:return t*(Fixed(2.0)-t);default:return t;}}
// Value tween: the project applies its output to any property. It does not retain
// raw references to entity fields that could be invalidated by a scene switch.
class Tween {
    Fixed from=0.0,to=0.0,duration=1.0,elapsed=0.0;Ease easing=Ease::Linear;
    bool running=false,finished=false;
public:
    bool start(Fixed a,Fixed b,Fixed seconds,Ease mode=Ease::Linear){if(seconds.raw()<0)return false;from=a;to=b;duration=seconds;elapsed=0.0;easing=mode;running=seconds.raw()>0;finished=!running;return true;}
    Fixed value() const{return duration.raw()==0?to:lerp(from,to,ease(Fixed(int32_t(int64_t(elapsed.raw())*4096/duration.raw()),Fixed::RAW),easing));}
    Fixed advance(Fixed dt){if(running&&dt.raw()>0){int64_t next=int64_t(elapsed.raw())+dt.raw();elapsed=Fixed(int32_t(next>duration.raw()?duration.raw():next),Fixed::RAW);if(elapsed>=duration){running=false;finished=true;}}return value();}
    void cancel(){running=finished=false;}bool playing()const{return running;}
    bool take_completion(){bool result=finished;finished=false;return result;}
    Fixed start_value()const{return from;}Fixed end_value()const{return to;}
    Fixed duration_value()const{return duration;}Fixed elapsed_value()const{return elapsed;}
    Ease ease_kind()const{return easing;}bool completion_pending()const{return finished;}
    void restore(Fixed a,Fixed b,Fixed seconds,Fixed progress,Ease mode,bool active,bool completion){from=a;to=b;duration=seconds;elapsed=progress;easing=mode;running=active;finished=completion;}
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
struct EPOK_VALUE(Id="5e278107-d8f8-4a76-8c72-568e0d4d3fc2") GameplayTweenState {Fixed from=0.0,to=0.0,duration=0.0,elapsed=0.0;Ease easing=Ease::Linear;bool running=false,completion_pending=false;};
struct EPOK_VALUE(Id="31e41da8-6334-47f0-99f6-692949bb6f17") TweenAdvanceSample {GameplayTweenState state{};Fixed value=0.0;bool completed=false;};
struct EPOK_VALUE(Id="9a3e9e93-eade-4991-b813-fc2296542a16") GameplayEventSample {uint32_t kind=0;int32_t value=0;ObjectId source{};};
struct EPOK_VALUE(Id="15b9944c-8b33-4e23-9951-f4c91c4cf486") GameplayEventQueue4 {GameplayEventSample item0{},item1{},item2{},item3{};uint32_t count=0,dropped=0;};
struct EPOK_VALUE(Id="52ec1051-39e5-4360-a681-65ff36b9f5e4") EventQueueMutation {GameplayEventQueue4 state{};bool accepted=false;};
struct EPOK_VALUE(Id="c2b92619-e338-42d8-95c1-0ed3862fa361") EventQueuePoll {GameplayEventQueue4 state{};GameplayEventSample event{};bool valid=false;};

inline ActorData* utility_actor_data(ObjectId id){auto* actor=active_object_registry?active_object_registry->resolve<Actor>(id):nullptr;return actor?actor->data():nullptr;}
inline ObjectId utility_actor_id(DataHandle value){auto* data=value.get();return data&&data->owner?data->owner->id():ObjectId{};}
inline Focus<16>& gameplay_focus(){static Focus<16> value;return value;}
inline GameplayTweenState gameplay_tween_state(const Tween& value){return {value.start_value(),value.end_value(),value.duration_value(),value.elapsed_value(),value.ease_kind(),value.playing(),value.completion_pending()};}
inline Tween gameplay_tween(GameplayTweenState state){Tween value;value.restore(state.from,state.to,state.duration,state.elapsed,state.easing,state.running,state.completion_pending);return value;}
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
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="680b43e9-1329-4382-90bd-48a0cf21fca6") static GameplayEventQueue4 event_queue_clear(){return {};}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="07803730-b04c-4917-af7a-30abf3695271") static EventQueueMutation event_queue_emit(GameplayEventQueue4 state,uint32_t kind,int32_t value,ObjectId source){auto queue=gameplay_event_queue(state);EventQueueMutation result;result.accepted=queue.emit(gameplay_event(GameplayEventSample{kind,value,source}));result.state=gameplay_event_queue(queue);return result;}
    EPOK_FUNCTION(BlueprintPure, PureValue, Id="3d9a1ba0-a7b8-4773-aaac-f841c7e7a0da") static EventQueuePoll event_queue_poll(GameplayEventQueue4 state){auto queue=gameplay_event_queue(state);EventQueuePoll result;QueueEvent value;result.valid=queue.poll(value);if(result.valid)result.event=gameplay_event(value);result.state=gameplay_event_queue(queue);return result;}
};
}
