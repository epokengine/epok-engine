#pragma once
#include <stdint.h>

// The one piece of this starter that is a game decision rather than engine
// math: which locomotion clip a character should be playing. Everything else
// the controller needs -- dead zones, headings, roots, angle approach -- is an
// engine operation, so the C++, Blueprint and Lua flavors all call the same
// implementation and reach the same bits.
namespace character_motion {
enum class State:uint8_t{Idle,Walk,Run,JumpUp,JumpDown,Land};
struct AnimationState{
    State state=State::Idle;
    // Speeds are raw Q12: 491 is 0.12 m/s, 14746 is 3.6 m/s and 12288 is 3 m/s.
    // The run threshold is lower while already running, so a character holding
    // a steady jog does not flicker between Walk and Run.
    State choose(bool grounded,int32_t vertical_speed,int32_t horizontal_speed,bool finished,bool jumped){
        if(jumped)state=State::JumpUp;
        else if(!grounded)state=vertical_speed>0?State::JumpUp:State::JumpDown;
        else if(state==State::JumpUp||state==State::JumpDown)state=State::Land;
        else if(state!=State::Land||finished){
            if(horizontal_speed<491)state=State::Idle;
            else if(horizontal_speed>14746||(state==State::Run&&horizontal_speed>12288))state=State::Run;
            else state=State::Walk;
        }
        return state;
    }
    static bool looping(State value){return value==State::Idle||value==State::Walk||value==State::Run;}
};
}
