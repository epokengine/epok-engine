#pragma once
#include "epok.hpp"
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
};
struct Event {uint16_t kind=0;int32_t value=0;DataHandle source;};
template<size_t Capacity=64> class EventQueue {
    static_assert(Capacity>0);Event events[Capacity];size_t first=0,count=0;
public:
    uint32_t dropped=0;
    bool emit(Event event){if(count==Capacity){++dropped;return false;}events[(first+count)%Capacity]=event;++count;return true;}
    bool poll(Event& event){if(!count)return false;event=events[first];first=(first+1)%Capacity;--count;return true;}
    void clear(){first=count=0;}size_t size()const{return count;}
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
}
