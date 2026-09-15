#pragma once
#include <stdint.h>

namespace epok {
// Button values match PsyQo's digital pad protocol. Both controller ports work.
enum class Button : uint8_t {
    Select, L3, R3, Start, Up, Right, Down, Left,
    L2, R2, L1, R1, Triangle, Circle, Cross, Square
};
// Q12 axes: positive X is right; positive Y is up. No dead zone is imposed here.
enum class Axis : uint8_t { LeftX, LeftY, RightX, RightY };
class Input {
    struct State { uint16_t held=0, pending_pressed=0, pending_released=0, pressed=0, released=0,frame_pressed=0,frame_released=0; bool connected=false,analog=false; int16_t axes[4]={}; };
    State ports[2];
    static uint16_t mask(Button b) { return uint8_t(b)<16?uint16_t(1u << uint8_t(b)):0; }
public:
    bool connected(unsigned port=0) const { return port<2 && ports[port].connected; }
    bool analog(unsigned port=0) const { return connected(port) && ports[port].analog; }
    int16_t axis_raw(Axis axis,unsigned port=0) const {
        return analog(port) && unsigned(axis)<4 ? ports[port].axes[unsigned(axis)] : 0;
    }
    bool held(Button b,unsigned port=0) const { return port<2 && (ports[port].held & mask(b)); }
    bool pressed(Button b,unsigned port=0) const { return port<2 && (ports[port].pressed & mask(b)); }
    bool released(Button b,unsigned port=0) const { return port<2 && (ports[port].released & mask(b)); }
    bool frame_pressed(Button b,unsigned port=0) const { return port<2 && (ports[port].frame_pressed & mask(b)); }
    bool frame_released(Button b,unsigned port=0) const { return port<2 && (ports[port].frame_released & mask(b)); }
    // Sampling accumulates edges until a simulation tick consumes them. A render
    // with no tick cannot lose a quick press/release; catch-up ticks don't repeat it.
    void sample(unsigned port,bool connected,uint16_t held,bool analog=false,
                uint8_t left_x=128,uint8_t left_y=128,uint8_t right_x=128,uint8_t right_y=128) {
        if(port>=2)return;
        auto& s=ports[port];if(!connected)held=0;
        s.frame_pressed=uint16_t(held & ~s.held);s.frame_released=uint16_t(s.held & ~held);
        s.pending_pressed |= s.frame_pressed;
        s.pending_released |= s.frame_released;
        s.held=held;s.connected=connected;
        s.analog=connected&&analog;
        if(!s.analog){for(auto& axis:s.axes)axis=0;return;}
        const uint8_t adc[4]={left_x,left_y,right_x,right_y};
        for(unsigned i=0;i<4;++i){
            const int delta=int(adc[i])-128;
            const int raw=delta*4096/(delta<0?128:127);
            s.axes[i]=s.analog?int16_t(i&1?-raw:raw):0;
        }
    }
    template<class PadReader> void poll(const PadReader& pad,unsigned second_port=1) {
        for(unsigned p=0;p<2;++p) {
            auto port=static_cast<typename PadReader::Pad>(p?second_port:0);
            bool connected=pad.isPadConnected(port);uint16_t bits=0;
            if(connected)for(unsigned b=0;b<16;++b)
                if(pad.isButtonPressed(port,static_cast<typename PadReader::Button>(b)))bits|=uint16_t(1u<<b);
            if constexpr(requires { pad.getPadType(port);pad.getAdc(port,0); }) {
                const auto type=pad.getPadType(port);
                const bool analog=connected&&(type==0x53||type==0x73);
                sample(p,connected,bits,analog,pad.getAdc(port,2),pad.getAdc(port,3),
                       pad.getAdc(port,0),pad.getAdc(port,1));
            } else {
                sample(p,connected,bits);
            }
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
