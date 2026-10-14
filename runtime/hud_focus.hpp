#pragma once
#include "epok.hpp"

namespace epok {
// D-pad focus movement over the authored neighbour table. Focus is one index on
// the canvas, not a flag per element, and a move is a static table lookup with
// no search. The editor's Interact mode sends the same Button bit numbering, so
// the console, the preview child and the viewport all move focus identically.
inline bool hud_focus_reachable(const ActorData* entities,size_t count,int index){
    if(!entities||index<0||size_t(index)>=count)return false;
    const auto& e=entities[index];
    return e.alive&&e.active&&e.focusable.enabled;
}
// `pressed` holds this frame's press edges in Button bit order. Cross presses the
// focused element (a Toggle flips), Circle cancels on it, and Left/Right adjust a
// focused slider; the D-pad otherwise moves the focus.
inline void hud_focus_update(ActorData* entities,size_t count,uint32_t pressed){
    if(!entities)return;
    // Focusable::neighbors is authored left, right, up, down.
    const uint32_t wanted[4]={1u<<unsigned(Button::Left),1u<<unsigned(Button::Right),
                              1u<<unsigned(Button::Up),1u<<unsigned(Button::Down)};
    const bool cross=pressed&(1u<<unsigned(Button::Cross)),circle=pressed&(1u<<unsigned(Button::Circle));
    int direction=-1;
    for(int i=0;i<4&&direction<0;++i)if(pressed&wanted[i])direction=i;
    if(direction<0&&!cross&&!circle)return;
    for(size_t i=0;i<count;++i){
        auto& e=entities[i];
        if(!e.canvas.enabled||!e.alive||!e.active)continue;
        const int current=e.canvas.focused;
        if(!hud_focus_reachable(entities,count,current))continue;
        auto& focused=entities[current];
        if(cross){
            focused.focusable.events|=focus_pressed;
            if(focused.focusable.kind==FocusKind::Toggle){focused.focusable.checked=!focused.focusable.checked;focused.focusable.events|=focus_changed;}
        }
        if(circle)focused.focusable.events|=focus_cancelled;
        if(direction<0)continue;
        // A slider keeps the focus on Left/Right and moves its value instead.
        if(direction<2&&focused.progress.enabled&&focused.progress.step.raw()>0){
            int32_t value=focused.progress.value.raw()+(direction?1:-1)*focused.progress.step.raw();
            if(value<0)value=0;
            if(value>4096)value=4096;
            if(value!=focused.progress.value.raw()){focused.progress.value=Fixed(value,Fixed::RAW);focused.focusable.events|=focus_changed;}
            continue;
        }
        const int target=focused.focusable.neighbors[direction];
        if(hud_focus_reachable(entities,count,target))e.canvas.focused=int16_t(target);
    }
}
// This frame's direction, Cross and Circle edges across every port, for callers
// that read the shared Input rather than a wire message.
inline uint32_t hud_focus_edges(){
    static const Button buttons[6]={Button::Up,Button::Right,Button::Down,Button::Left,Button::Cross,Button::Circle};
    uint32_t mask=0;
    for(unsigned port=0;port<4;++port)
        for(const auto button:buttons)
            if(input.frame_pressed(button,port))mask|=1u<<unsigned(button);
    return mask;
}
}
