#pragma once
#include "psyqo/fixed-point.hh"
#include "texture_types.hpp"
#include <stdint.h>
namespace epok {
using Fixed = psyqo::FixedPoint<12>;
enum class SpriteOrientation { Fixed, Upright, Spherical };
struct Sprite {
    bool enabled=false;int texture=-1;uint16_t region[4]={};
    Fixed size[2]={1.0,1.0},pivot[2]={0.5,0.0};bool flip_x=false,flip_y=false;
    SpriteOrientation orientation=SpriteOrientation::Upright;
    uint8_t color[3]={255,255,255};bool unlit=false;BlendMode blend=BlendMode::Cutout;int16_t depth_bias=0;
};
struct SpriteFrame {uint16_t region[4];Fixed duration;uint16_t event=0;};
struct SpriteClip {const SpriteFrame* frames;uint16_t frame_count;bool looping;const char* name;};
struct SpriteAnimator {
    bool enabled=false;const SpriteClip* clips=nullptr;uint16_t clip_count=0,clip=0,frame=0;
    bool playing=true,completed=false;Fixed elapsed=0.0;
    // Bounded FIFO preserves each crossed-frame event; drop counter is observable.
    uint16_t events[16]={};uint8_t event_read=0,event_count=0;uint32_t dropped_events=0;
    void emit(uint16_t event){if(!event)return;if(event_count==16){++dropped_events;return;}events[(event_read+event_count)%16]=event;++event_count;}
    bool poll_event(uint16_t& event){if(!event_count)return false;event=events[event_read];event_read=(event_read+1)%16;--event_count;return true;}
    bool play(uint16_t index){if(!clips||index>=clip_count||!clips[index].frame_count)return false;clip=index;frame=0;elapsed=0.0;playing=true;completed=false;event_count=event_read=0;emit(clips[index].frames[0].event);return true;}
    bool take_completion(){bool result=completed;completed=false;return result;}
    void pause(){playing=false;}void resume(){if(clips&&clip<clip_count)playing=true;}
    void apply(Sprite& sprite) const {if(!enabled||!clips||clip>=clip_count||frame>=clips[clip].frame_count)return;for(int i=0;i<4;++i)sprite.region[i]=clips[clip].frames[frame].region[i];}
    void advance(Fixed dt,Sprite& sprite){
        if(!enabled||!playing||!clips||clip>=clip_count||dt.raw()<=0)return;
        const auto& c=clips[clip];if(!c.frame_count)return;
        // Simulation caller supplies bounded steps; cap malicious script input too.
        if(dt.raw()>4096)dt=Fixed(4096,Fixed::RAW);elapsed+=dt;
        for(int transitions=0;transitions<256;++transitions){
            const auto duration=c.frames[frame].duration.raw()>0?c.frames[frame].duration:Fixed(1,Fixed::RAW);
            if(elapsed<duration)break;elapsed-=duration;
            if(frame+1<c.frame_count)++frame;
            else if(c.looping)frame=0;
            else {playing=false;completed=true;elapsed=0.0;break;}
            emit(c.frames[frame].event);
        }
        apply(sprite);
    }
};
struct SpriteStats {uint32_t submitted=0,triangles=0,clipped=0,culled=0,dropped=0,estimated_pixels=0;};
inline SpriteStats sprite_stats;
}
