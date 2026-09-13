#pragma once
#include <stdint.h>

namespace epok {
// Elapsed microseconds come from GPU::now(). Unsigned subtraction handles wrap.
// Simulation is exactly 60 Hz, with at most 8 catch-up steps per render frame.
class Time {
    uint32_t previous=0, accumulated=0,delta_remainder=0;
    bool initialized=false, paused_=false;
public:
    static constexpr unsigned rate=60,max_steps=8;
    uint32_t ticks=0,dropped_steps=0,frame_microseconds=0;
    int32_t delta_raw=68; // Q12 seconds, 68/69 alternating without cumulative drift.
    bool paused() const { return paused_; }
    void set_paused(bool value) { if(value!=paused_)accumulated=0;paused_=value; }
    void reset(uint32_t now) { previous=now;accumulated=delta_remainder=0;initialized=true;ticks=dropped_steps=frame_microseconds=0;delta_raw=68; }
    // Explicit loading time is excluded without erasing gameplay counters or pause state.
    void synchronize(uint32_t now) { previous=now;accumulated=0;initialized=true;frame_microseconds=0; }
    unsigned advance(uint32_t now) {
        if(!initialized) { reset(now);return 0; }
        frame_microseconds=now-previous;previous=now;
        if(paused_) { accumulated=0;return 0; }
        // Multiplication is wide so even a long stall can be counted safely.
        uint64_t scaled=uint64_t(frame_microseconds)*rate+accumulated;
        uint32_t steps=uint32_t(scaled/1000000);accumulated=uint32_t(scaled%1000000);
        if(steps>max_steps) { dropped_steps+=steps-max_steps;steps=max_steps; }
        return steps;
    }
    void begin_tick() { ++ticks;delta_raw=68;delta_remainder+=16;if(delta_remainder>=60) { ++delta_raw;delta_remainder-=60; } }
    uint32_t interpolation_thousandths() const { return accumulated/1000; }
    uint32_t interpolation_raw() const { return uint32_t(uint64_t(accumulated)*4096/1000000); }
};
inline Time time;
}
