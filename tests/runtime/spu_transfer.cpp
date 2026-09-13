#include "audio_transport_stub.hpp"
#include "audio-runtime/spu_transfer.hpp"
#include <cassert>
#include <cstdio>

struct DelayedSpu {
    inline static uint16_t control,status;
    inline static unsigned ack,drain,dma,starts,addresses,writes;
    inline static unsigned stuck;
    inline static bool cancelled;
    static void reset(unsigned failure=0){
        control=status=0xc020;ack=drain=dma=starts=addresses=writes=0;stuck=failure;cancelled=false;
    }
    static uint16_t read(uintptr_t address){
        if(address==0x1f801daa)return control;
        assert(address==0x1f801dae);
        if(ack && --ack==0 && !(stuck==1 || (stuck==2 && control&0x20)))status=control;
        if(drain){if(stuck!=4 && --drain==0)status&=~0x400;else status|=0x400;}
        return status;
    }
    static void write(uintptr_t address,uint16_t value){
        if(address==0x1f801daa){control=value;ack=3;++writes;}
        else {assert(address==0x1f801da6 && (status&0x430)==0);++addresses;}
    }
    static bool dma_busy(){if(!dma)return false;if(stuck!=3)--dma;return dma!=0;}
    static void start(const void*,uint32_t bytes){
        assert((status&0x30)==0x20 && addresses==starts+1 && bytes==64);
        ++starts;dma=3;drain=4;status|=0x400;
    }
    static void cancel(){cancelled=true;dma=0;}
};
int main(){
    alignas(64) uint8_t sample[64]{};
    DelayedSpu::reset();
    for(unsigned n=0;n<3;++n)assert(epok::spu::upload<DelayedSpu>(sample,4096+64*n,64,100));
    assert(DelayedSpu::starts==3 && DelayedSpu::addresses==3 && !DelayedSpu::cancelled);
    assert((DelayedSpu::status&0x430)==0);
    for(unsigned failure=1;failure<=4;++failure){
        DelayedSpu::reset(failure);
        assert(!epok::spu::upload<DelayedSpu>(sample,4096,64,30));
        assert(DelayedSpu::cancelled && (DelayedSpu::control&0x30)==0);
    }
    DelayedSpu::reset();
    assert(!epok::spu::upload<DelayedSpu>(sample,4097,64));
    assert(!epok::spu::upload<DelayedSpu>(sample,512*1024,64));
    assert(!DelayedSpu::writes && !DelayedSpu::starts);
    std::puts("SPU DMA: delayed Stop/Write acknowledgements, FIFO drain, consecutive transfers and bounded failures passed");
}
