#pragma once
#include "sequence_service.hpp"
#include "common/hardware/counters.h"
#include "common/syscalls/syscalls.h"
#include "common/kernel/events.h"
#include "psyqo/kernel.hh"

namespace epok {
// Timer 0 runs at system clock: target 64000 gives ~529 Hz. Stay below
// 0xffff: target/overflow coincidence is mishandled by some emulator cores.
// Only Timer 2 supports system clock / 8. Setting TM_CLK_DIV8 on Timer 0
// silently selects system clock and previously produced ~2000 IRQ/s, not 250.
// Timer 2 measures actual elapsed CPU/8 ticks (4.2336 MHz), retaining the
// fractional microsecond remainder. Timer 1 remains the GPU's existing HSync
// counter and is read only as a wrap guard.
inline uint16_t sequence_previous_ticks=0,sequence_previous_lines=0;
inline uint32_t sequence_fraction=0;
inline unsigned sequence_storage_depth=0;
struct SequenceClockDiagnostics {
    uint32_t recoveries=0,max_elapsed_ticks=0,first_lines=0,first_ticks=0,
        first_target=0,first_pc=0,first_clock_us=0,rejected=0;
};
inline SequenceClockDiagnostics sequence_clock_diagnostics;
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
// Timer 2 keeps running freely at CPU/8. Move only its compare target: the
// elapsed-time counter never resets. A 7 ms idle ceiling stays well inside
// its 15.48 ms wrap and the HSync ambiguity guard.
inline void sequence_native_schedule(uint16_t begin){
    using namespace psx_audio;
    const uint16_t current=COUNTERS[2].value;
    uint32_t delay=29636;
    const auto consider=[&](uint16_t target){
        const int32_t remaining=int16_t(target-current);
        const uint32_t safe=remaining<128?128:uint32_t(remaining);
        if(safe<delay)delay=safe;
    };
    for(const auto& i:instances){
        if(i.phase==Phase::Queued){delay=128;break;}
        if(i.phase==Phase::Retiring){
            const uint32_t age=now_us-i.retire_us;
            consider(uint16_t(begin+(age>=2000?0:(2000-age)*2646/625)));
        }else if(i.phase==Phase::Active){
            const auto stream=i.asset->native();
            if(i.native_cursor<stream.count()){
                const uint32_t event=stream.commands()[i.native_cursor].time;
                const uint32_t wait=event>i.native_clock?event-i.native_clock:0;
                if(wait<7000)consider(uint16_t(begin+wait*2646/625));
            }
        }
    }
    for(int p=0;p<24;++p)if(physical[p].pending)consider(uint16_t(physical[p].off_ticks+128));
    uint16_t target=uint16_t(current+delay);
    // Selecting a deadline takes CPU time. Never publish a compare value
    // already behind the live counter (that would wait an entire 16-bit wrap).
    // Leave 1 ms for BIOS acknowledgement/return and competing pad/card IRQs.
    const uint16_t armed=COUNTERS[2].value;
    if(int16_t(target-armed)<4234)target=uint16_t(armed+4234);
    // Keep compares away from overflow on both sides. Redux can step across a
    // tiny post-wrap target while handling overflow and defer it a full wrap.
    // A target of 1 reproduced a 22.5 ms gap despite a 7 ms scheduled interval.
    if(target<512||target>0xffff-512)target=512;
    COUNTERS[2].target=target;
}
#endif
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
// PCDrv's resident transport can hold IRQs for entire serial chunks. Freeze
// musical time across that known storage boundary; never reinterpret multiple
// counter wraps as a short elapsed interval or a gameplay clock failure.
struct SequenceStoragePause {
    SequenceStoragePause(){SequenceLock lock;++sequence_storage_depth;}
    ~SequenceStoragePause(){
        SequenceLock lock;
        if(!--sequence_storage_depth){
            sequence_previous_ticks=COUNTERS[2].value;
            sequence_previous_lines=sequence_hsync();sequence_fraction=0;
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
            // The old compare may have passed while the resident handler owned
            // the CPU. Arm a new deadline instead of waiting another full wrap.
            sequence_native_schedule(sequence_previous_ticks);
#endif
        }
    }
};
// Timer 2 wraps every 15.48 ms. For a short delayed IRQ, the independent
// HSync counter identifies the wrap count without estimating musical time.
// 267..273 CPU/8 ticks per line covers NTSC/PAL clocks and either output mode;
// two extra lines cover the non-atomic counter reads. Even at the 100 ms bound
// the admissible interval is narrower than one Timer 2 wrap, so it is unique.
inline bool sequence_elapsed_ticks(uint16_t ticks,uint16_t lines,uint32_t& elapsed){
    elapsed=ticks;
    if(lines<225)return true;
    if(lines>1600)return false;
    const uint32_t lower=uint32_t(lines-2)*267,upper=uint32_t(lines+2)*273;
    if(elapsed<lower)elapsed+=((lower-elapsed+65535)>>16)<<16;
    return elapsed<=upper;
}
inline void sequence_clock_irq(){
    const uint16_t begin=COUNTERS[2].value;
    sequence_irq_begin_ticks=begin;
    const uint16_t lines=sequence_hsync();
    const uint16_t gap_lines=uint16_t(lines-sequence_previous_lines);
    const uint16_t ticks=uint16_t(begin-sequence_previous_ticks);
    sequence_previous_ticks=begin;sequence_previous_lines=lines;
    if(sequence_storage_depth){sequence_fraction=0;
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
        sequence_native_schedule(begin);
#endif
        return;}
    uint32_t elapsed_ticks=0;
    const bool resolved=sequence_elapsed_ticks(ticks,gap_lines,elapsed_ticks);
    if(gap_lines>=225){
        auto& d=sequence_clock_diagnostics;
        if(!d.recoveries&&!d.rejected){
            d.first_lines=gap_lines;d.first_ticks=ticks;d.first_target=COUNTERS[2].target;
            d.first_clock_us=music_sequence_stats.clock_us;
#ifdef __mips__
            asm volatile("mfc0 %0,$14\n\tnop":"=r"(d.first_pc));
#endif
        }
        if(resolved){++d.recoveries;if(elapsed_ticks>d.max_elapsed_ticks)d.max_elapsed_ticks=elapsed_ticks;}
        else ++d.rejected;
    }
    // Inconsistent counter pairs and pauses beyond the recovery bound remain
    // faults. Known serial/storage pauses are explicitly handled above.
    if(!resolved){sequence_clock_fault();sequence_fraction=0;
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
        sequence_native_schedule(begin);
#endif
        return;}
    const uint32_t numerator=elapsed_ticks*625+sequence_fraction;
    const uint32_t micros=numerator/2646;sequence_fraction=numerator%2646;
    if(micros>music_sequence_stats.max_gap_us)music_sequence_stats.max_gap_us=micros;
    sequence_service(micros);
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
    sequence_native_schedule(begin);
#endif
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
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
    constexpr unsigned timer=2;
    constexpr auto irq=psyqo::Kernel::IRQ::Timer2;
#else
    constexpr unsigned timer=0;
    constexpr auto irq=psyqo::Kernel::IRQ::Timer0;
#endif
    if(psyqo::Kernel::isKernelTakenOver()){
        psyqo::Kernel::queueIRQHandler(irq,[]{sequence_clock_irq();});
        HW_U16(0x1f801074)=HW_U16(0x1f801074)|(1u<<(4+timer));
    }else{
        const auto event=psyqo::Kernel::openEvent(0xf2000000+timer,2,EVENT_MODE_CALLBACK,[]{sequence_clock_irq();});
        if(event==0xffffffffu){music_sequence_stats.error=10;music_sequence_stats.ready=0;return false;}
        syscall_enableEvent(event);syscall_enableTimerIRQ(timer);syscall_setTimerAutoAck(timer,1);
    }
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
    COUNTERS[2].target=128;
    COUNTERS[2].mode=TM_CLK_DIV8|TM_IRQ_TARGET|TM_IRQ_REPEAT;
    sequence_previous_ticks=COUNTERS[2].value;sequence_previous_lines=sequence_hsync();
#else
    COUNTERS[0].target=64000;
    COUNTERS[0].mode=TM_RESET_TARGET|TM_IRQ_TARGET|TM_IRQ_REPEAT;
#endif
    return true;
}
}
