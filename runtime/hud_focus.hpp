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
// `pressed` holds this frame's press edges in Button bit order.
inline void hud_focus_update(ActorData* entities,size_t count,uint32_t pressed){
    if(!entities)return;
    // Focusable::neighbors is authored left, right, up, down.
    const uint32_t wanted[4]={1u<<unsigned(Button::Left),1u<<unsigned(Button::Right),
                              1u<<unsigned(Button::Up),1u<<unsigned(Button::Down)};
    int direction=-1;
    for(int i=0;i<4&&direction<0;++i)if(pressed&wanted[i])direction=i;
    if(direction<0)return;
    for(size_t i=0;i<count;++i){
        auto& e=entities[i];
        if(!e.canvas.enabled||!e.alive||!e.active)continue;
        const int current=e.canvas.focused;
        if(!hud_focus_reachable(entities,count,current))continue;
        const int target=entities[current].focusable.neighbors[direction];
        if(hud_focus_reachable(entities,count,target))e.canvas.focused=int16_t(target);
    }
}
// This frame's direction edges across every port, for callers that read the
// shared Input rather than a wire message.
inline uint32_t hud_focus_edges(){
    static const Button directions[4]={Button::Up,Button::Right,Button::Down,Button::Left};
    uint32_t mask=0;
    for(unsigned port=0;port<4;++port)
        for(const auto button:directions)
            if(input.frame_pressed(button,port))mask|=1u<<unsigned(button);
    return mask;
}
}
