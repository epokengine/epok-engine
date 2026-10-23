#pragma once
#include <stdint.h>

// Simulation-time presentation: no allocations, blocking waits or input polls.
// Durations use Q12 seconds, the same clock as the gameplay components.
struct DungeonFlow {
    enum Phase : uint8_t {AwaitMusic,MusicLeadIn,FadeIn,ControlDelay,Playing,DeathHold,FadeOut,RestartHold};
    Phase phase=AwaitMusic;
    uint32_t elapsed=0;
    void begin(){phase=AwaitMusic;elapsed=0;}
    void die(){phase=DeathHold;elapsed=0;}
    bool playing() const{return phase==Playing;}
    bool step(uint32_t dt,bool music_playing){
        if(phase==AwaitMusic){if(music_playing){phase=MusicLeadIn;elapsed=0;}return false;}
        if(phase==Playing)return false;
        const uint32_t duration=(phase==ControlDelay||phase==DeathHold||phase==RestartHold)?8192:4096;
        elapsed+=dt;
        if(elapsed<duration)return false;
        elapsed-=duration;
        switch(phase){
            case MusicLeadIn:phase=FadeIn;break;
            case FadeIn:phase=ControlDelay;break;
            case ControlDelay:phase=Playing;elapsed=0;break;
            case DeathHold:phase=FadeOut;break;
            case FadeOut:phase=RestartHold;break;
            case RestartHold:return true;
            default:break;
        }
        return false;
    }
    uint8_t fade() const{
        if(phase==AwaitMusic||phase==MusicLeadIn||phase==RestartHold)return 255;
        if(phase==FadeIn)return uint8_t(255-(elapsed>4096?4096:elapsed)*255/4096);
        if(phase==FadeOut)return uint8_t((elapsed>4096?4096:elapsed)*255/4096);
        return 0;
    }
    uint16_t music_gain() const{
        if(phase==RestartHold)return 0;
        if(phase==FadeOut)return uint16_t(4096-(elapsed>4096?4096:elapsed));
        return 4096;
    }
};
