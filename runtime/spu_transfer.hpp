#pragma once
#include "common/hardware/dma.h"
#include "common/hardware/hwregs.h"
#include <cstdint>

namespace epok::spu {
struct Hardware {
    static uint16_t read(uintptr_t address){return HW_U16(address);}
    static void write(uintptr_t address,uint16_t value){HW_U16(address)=value;}
    static bool dma_busy(){return (DMA_CTRL[DMA_SPU].CHCR&0x01000000)!=0;}
    static void start(const void* data,uint32_t bytes){
        DMA_CTRL[DMA_SPU].MADR=uint32_t(uintptr_t(data));
        DMA_CTRL[DMA_SPU].BCR=((bytes/64)<<16)|16;
        DMA_CTRL[DMA_SPU].CHCR=0x01000201;
    }
    static void cancel(){DMA_CTRL[DMA_SPU].CHCR=0;}
};

// Main-thread DMA only. A CPU DMA completion does not certify that the SPU
// applied a new transfer mode or drained its FIFO. Acknowledge Stop before
// changing the address, then DMA Write before submitting another block.
template<class H=Hardware>
bool upload(const void* data,uint32_t address,uint32_t bytes,uint32_t budget=10000000){
    if(!data || (uintptr_t(data)&3) || (address&63) || !bytes || (bytes&63) ||
       address>512*1024 || bytes>512*1024-address || !budget)return false;
    constexpr uintptr_t Control=0x1f801daa,Status=0x1f801dae,Address=0x1f801da6;
    const uint16_t stop=H::read(Control)&~0x30u;
    const auto fail=[&](){H::cancel();H::write(Control,stop);return false;};
    const auto wait=[&](auto ready){while(!ready()){if(!--budget)return false;}return true;};
    if(!wait([](){return !H::dma_busy();}))return fail();
    H::write(Control,stop);
    if(!wait([&](){return (H::read(Status)&0x3f)==(stop&0x3f);}) ||
       !wait([](){return !(H::read(Status)&0x400);}))return fail();
    H::write(Address,uint16_t(address>>3));
    const uint16_t mode=stop|0x20;
    H::write(Control,mode);
    if(!wait([&](){return (H::read(Status)&0x3f)==(mode&0x3f);}))return fail();
    H::start(data,bytes);
    if(!wait([](){return !H::dma_busy();}) || !wait([](){return !(H::read(Status)&0x400);}))return fail();
    H::write(Control,stop);
    if(!wait([&](){return (H::read(Status)&0x3f)==(stop&0x3f);}))return fail();
    return true;
}
}
