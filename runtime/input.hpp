#pragma once
#include <stdint.h>

namespace epok {
// Button values match PsyQo's digital pad protocol. Both controller ports work.
enum class Button : uint8_t {
    Select, L3, R3, Start, Up, Right, Down, Left,
    L2, R2, L1, R1, Triangle, Circle, Cross, Square
};
class Input {
    struct State { uint16_t held=0, pending_pressed=0, pending_released=0, pressed=0, released=0,frame_pressed=0,frame_released=0; bool connected=false; };
    State ports[2];
    static uint16_t mask(Button b) { return uint8_t(b)<16?uint16_t(1u << uint8_t(b)):0; }
public:
    bool connected(unsigned port=0) const { return port<2 && ports[port].connected; }
    bool held(Button b,unsigned port=0) const { return port<2 && (ports[port].held & mask(b)); }
    bool pressed(Button b,unsigned port=0) const { return port<2 && (ports[port].pressed & mask(b)); }
    bool released(Button b,unsigned port=0) const { return port<2 && (ports[port].released & mask(b)); }
    bool frame_pressed(Button b,unsigned port=0) const { return port<2 && (ports[port].frame_pressed & mask(b)); }
    bool frame_released(Button b,unsigned port=0) const { return port<2 && (ports[port].frame_released & mask(b)); }
    // Sampling accumulates edges until a simulation tick consumes them. A render
    // with no tick cannot lose a quick press/release; catch-up ticks don't repeat it.
    void sample(unsigned port,bool connected,uint16_t held) {
        if(port>=2)return;
        auto& s=ports[port];if(!connected)held=0;
        s.frame_pressed=uint16_t(held & ~s.held);s.frame_released=uint16_t(s.held & ~held);
        s.pending_pressed |= s.frame_pressed;
        s.pending_released |= s.frame_released;
        s.held=held;s.connected=connected;
    }
    template<class PadReader> void poll(const PadReader& pad,unsigned second_port=1) {
        for(unsigned p=0;p<2;++p) {
            auto port=static_cast<typename PadReader::Pad>(p?second_port:0);
            bool connected=pad.isPadConnected(port);uint16_t bits=0;
            if(connected)for(unsigned b=0;b<16;++b)
                if(pad.isButtonPressed(port,static_cast<typename PadReader::Button>(b)))bits|=uint16_t(1u<<b);
            sample(p,connected,bits);
        }
    }
    void begin_tick() {
        for(auto& s:ports) { s.pressed=s.pending_pressed;s.released=s.pending_released;s.pending_pressed=s.pending_released=0; }
    }
    void end_tick() { for(auto& s:ports)s.pressed=s.released=0; }
    void discard_edges() { for(auto& s:ports)s.pending_pressed=s.pending_released=s.pressed=s.released=0; }
    void reset() { for(auto& s:ports)s=State{}; }
};
inline Input input;
}
