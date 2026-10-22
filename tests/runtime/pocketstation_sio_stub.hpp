#pragma once
#include <cassert>
#include <cstdint>
#include "psyqo/sio0-bus.hh"
namespace pocket_sio_test {
inline uint16_t control=0,status=0;
inline bool irq=false,masked=true,ack=true,receive=true,pad_collision=false;
inline unsigned writes=0,expected_port=0;
inline unsigned byte_index=0,ack_bytes=~0u;
inline uint8_t input=0;
inline uint32_t other_irqs=0;
struct Counter{uint16_t value=0;};
inline Counter counters[3];
inline void reset(unsigned port){
    control=status=0;irq=false;masked=true;ack=receive=true;pad_collision=false;
    writes=byte_index=0;ack_bytes=~0u;expected_port=port;input=0;counters[1].value=0;other_irqs=0;
}
}
#define COUNTERS pocket_sio_test::counters
namespace psyqo::Hardware {
namespace CPU {
enum class IRQ:uint32_t {Controller=1u<<7};
struct Mask {
    bool isSet(IRQ)const{return pocket_sio_test::masked;}
    void clear(IRQ){pocket_sio_test::masked=false;}
    void set(IRQ){pocket_sio_test::masked=true;}
};
struct Pending {
    bool isSet(IRQ)const{return pocket_sio_test::irq;}
    Pending& access(){return *this;}
    void operator=(uint32_t value){
        // New DMA, music-timer and VBlank edges appear immediately before the
        // acknowledge write. Only the controller bit may be cleared.
        pocket_sio_test::other_irqs|=0x49;
        pocket_sio_test::other_irqs&=value;
        assert(value==~uint32_t(0x80)&&pocket_sio_test::other_irqs==0x49);
        pocket_sio_test::irq=pocket_sio_test::irq&&bool(value&0x80);
    }
};
inline Mask IMask;inline Pending IReg;
inline void flushWriteQueue(){}
}
namespace SIO {
enum Control:uint16_t {CTRL_TXEN=1,CTRL_DTR=2,CTRL_ERRRES=16,CTRL_ACKIRQEN=4096,CTRL_PORTSEL=8192};
enum Status:uint16_t {STAT_RXRDY=2,STAT_ACK=128};
struct ControlRegister {
    void operator=(uint16_t value){pocket_sio_test::control=value;if(!value)pocket_sio_test::byte_index=0;}
    void operator|=(uint16_t value){
        if(value&CTRL_ERRRES)assert(!pocket_sio_test::irq); // I_STAT first, SIO second.
        pocket_sio_test::control|=value;
    }
    operator uint16_t()const{return pocket_sio_test::control;}
};
inline ControlRegister Ctrl;
inline auto& Stat=pocket_sio_test::status;
inline uint16_t Baud=0,Mode=0;
struct DataRegister {
    void operator=(uint8_t byte){
        using namespace pocket_sio_test;
        // Any competing bus user must observe ownership and leave SIO alone.
        if(!psyqo::SIO0Bus::owned()){pad_collision=true;control=0;}
        assert(bool(control&CTRL_PORTSEL)==bool(expected_port));
        assert(!masked);assert((control&(CTRL_TXEN|CTRL_DTR))==(CTRL_TXEN|CTRL_DTR));
        input=byte_index==2?2:0xff;
        if(byte_index==0)assert(byte==0x81);
        ++writes;++byte_index;status=receive?STAT_RXRDY:0;irq=ack&&byte_index<=ack_bytes;
    }
    operator uint8_t(){pocket_sio_test::status&=~STAT_RXRDY;return pocket_sio_test::input;}
    void throwAway(){pocket_sio_test::status&=~STAT_RXRDY;}
};
inline DataRegister Data;
}
}
