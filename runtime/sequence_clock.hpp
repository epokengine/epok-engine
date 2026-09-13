#pragma once
#include "sequence_service.hpp"
#include "common/hardware/counters.h"
#include "common/syscalls/syscalls.h"
#include "common/kernel/events.h"
#include "psyqo/kernel.hh"

namespace epok {
// Timer 0 requests approximately 1 kHz. Timer 2 measures actual elapsed CPU/8
// ticks (4.2336 MHz), retaining the fractional microsecond remainder. Timer 1
// remains the GPU's existing HSync counter and is read only as a wrap guard.
inline uint16_t sequence_previous_ticks=0,sequence_previous_lines=0;
inline uint32_t sequence_fraction=0;
inline uint16_t sequence_hsync(){
    uint16_t before=COUNTERS[1].value;
    // HSync crosses the GPU/CPU clock domain. Retry a transient torn read;
    // ordinary adjacent reads are equal or advance by one scanline.
    for(unsigned i=0;i<8;++i){
        const uint16_t after=COUNTERS[1].value;
        if(uint16_t(after-before)<=1)return after;
        before=after;
    }
    return before;
}
inline void sequence_clock_irq(){
    const uint16_t begin=COUNTERS[2].value;
    sequence_irq_begin_ticks=begin;
    const uint16_t lines=sequence_hsync();
    const uint16_t gap_lines=uint16_t(lines-sequence_previous_lines);
    const uint16_t ticks=uint16_t(begin-sequence_previous_ticks);
    sequence_previous_ticks=begin;sequence_previous_lines=lines;
    // A 16-bit Timer 2 wraps in 15.48 ms. Do not invent elapsed time when
    // interrupts were masked long enough to make this delta ambiguous.
    if(gap_lines>=225){sequence_clock_fault();sequence_fraction=0;return;}
    const uint32_t numerator=uint32_t(ticks)*625+sequence_fraction;
    const uint32_t micros=numerator/2646;sequence_fraction=numerator%2646;
    if(micros>music_sequence_stats.max_gap_us)music_sequence_stats.max_gap_us=micros;
    sequence_service(micros);
    const uint16_t cost=uint16_t(COUNTERS[2].value-begin);
    sequence_timing_stats.service_ticks+=cost;
    if(cost>music_sequence_stats.max_service_ticks)music_sequence_stats.max_service_ticks=cost;
}
inline bool sequence_clock_start(){
    if(!sequence_prepare())return false;
    SequenceLock lock;
    COUNTERS[0].mode=0;
    COUNTERS[2].mode=TM_CLK_DIV8;
    COUNTERS[2].value=0;
    sequence_previous_ticks=COUNTERS[2].value;sequence_previous_lines=sequence_hsync();
    if(psyqo::Kernel::isKernelTakenOver()){
        psyqo::Kernel::queueIRQHandler(psyqo::Kernel::IRQ::Timer0,[]{sequence_clock_irq();});
        HW_U16(0x1f801074)=HW_U16(0x1f801074)|(1u<<4);
    }else{
        const auto event=psyqo::Kernel::openEvent(0xf2000000,2,EVENT_MODE_CALLBACK,[]{sequence_clock_irq();});
        if(event==0xffffffffu){music_sequence_stats.error=10;music_sequence_stats.ready=0;return false;}
        syscall_enableEvent(event);syscall_enableTimerIRQ(0);syscall_setTimerAutoAck(0,1);
    }
    COUNTERS[0].target=33866;
    COUNTERS[0].mode=TM_RESET_TARGET|TM_IRQ_TARGET|TM_IRQ_REPEAT;
    return true;
}
}
