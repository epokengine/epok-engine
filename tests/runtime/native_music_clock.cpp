#define EPOK_HAS_SEQUENCES 1
#define EPOK_SEQUENCE_HOST_TEST 1
#define EPOK_NATIVE_SEQUENCES_ONLY 1
#include <cassert>
#include <cstdio>
#include <cstring>
#include "audio-runtime/sequence_clock.hpp"
namespace epok {
void music_play(AudioSource*){} void music_stop(AudioSource*){} bool music_is_playing(const AudioSource*){return false;}
}
int main(){
    using namespace epok;
    assert(sequence_clock_start());
    assert(audio_test_timer==2 && COUNTERS[0].mode==0);
    assert(COUNTERS[2].mode==(TM_CLK_DIV8|TM_IRQ_TARGET|TM_IRQ_REPEAT));
    assert(!(COUNTERS[2].mode&TM_RESET_TARGET));
    // An idle service sleeps within the wrap guard instead of polling at 529 Hz.
    COUNTERS[2].value=1000;sequence_native_schedule(1000);
    assert(COUNTERS[2].target==30636);
    alignas(4) uint8_t bytes[48]{};
    native_music::Command command{2000,native_music::End,0,0};
    bytes[12]=1;std::memcpy(bytes+40,&command,8);
    psx_audio::Sequence song{bytes,sizeof(bytes),nullptr};
    auto& instance=psx_audio::instances[0];instance.asset=&song;
    instance.phase=psx_audio::Phase::Active;instance.native_clock=1000;
    // A deadline across 0xffff is still in the future; an overdue event is
    // retried promptly, never delayed by a complete counter wrap.
    COUNTERS[2].value=62000;sequence_native_schedule(60000);
    assert(COUNTERS[2].target==uint16_t(62000+4234));
    COUNTERS[2].value=65000;sequence_native_schedule(60000);
    assert(COUNTERS[2].target==uint16_t(65000+4234));
    instance.phase=psx_audio::Phase::Free;instance.asset=nullptr;
    auto& voice=psx_audio::physical[0];voice.pending=true;voice.off_ticks=64960;
    sequence_native_schedule(65000);assert(COUNTERS[2].target==uint16_t(65000+4234));
    voice.pending=false;
    COUNTERS[2].value=35900;sequence_native_schedule(35900);
    assert(COUNTERS[2].target==512); // never arm the overflow boundary or target 1
    COUNTERS[2].value=35400;sequence_native_schedule(35400);
    assert(COUNTERS[2].target==512); // near-end compares also move safely past wrap
    // Every possible idle phase leaves a future compare inside the safe window.
    for(unsigned tick=0;tick<65536;++tick){
        COUNTERS[2].value=uint16_t(tick);sequence_native_schedule(uint16_t(tick));
        const uint16_t target=COUNTERS[2].target,wait=uint16_t(target-tick);
        assert(target>=512&&target<=65535-512&&wait>=4234&&wait<32768);
    }
    // Fractional CPU/8 time survives many 16-bit wraps without tempo drift.
    sequence_previous_ticks=COUNTERS[2].value;sequence_previous_lines=COUNTERS[1].value;
    for(unsigned n=0;n<40;++n){COUNTERS[2].value+=29636;COUNTERS[1].value+=110;sequence_clock_irq();}
    assert(music_sequence_stats.clock_faults==0 && music_sequence_stats.error==0);
    assert(psx_audio::now_us==uint32_t(uint64_t(40)*29636*625/2646));
    const auto before=psx_audio::now_us;
    {
        SequenceStoragePause storage;
        {SequenceStoragePause nested;COUNTERS[2].value+=1000;COUNTERS[1].value+=1000;sequence_clock_irq();}
        assert(sequence_storage_depth==1&&psx_audio::now_us==before);
        COUNTERS[2].value+=30000;COUNTERS[1].value+=1000;sequence_clock_irq();
        assert(music_sequence_stats.clock_faults==0&&music_sequence_stats.error==0);
    }
    assert(sequence_storage_depth==0&&psx_audio::now_us==before);
    // Losing the compare IRQ during transport must not leave playback waiting
    // nearly a full 16-bit wrap after the storage call returns.
    COUNTERS[2].value=20000;
    {
        SequenceStoragePause storage;
        COUNTERS[2].value=30000;COUNTERS[2].target=29999;
        COUNTERS[1].value+=1000;
    }
    assert(sequence_previous_ticks==30000);
    assert(uint16_t(COUNTERS[2].target-COUNTERS[2].value)>=4234);
    assert(uint16_t(COUNTERS[2].target-COUNTERS[2].value)<32768);
    COUNTERS[2].value+=29636;COUNTERS[1].value+=110;sequence_clock_irq();
    assert(music_sequence_stats.clock_faults==0&&psx_audio::now_us>before);
    // A delayed compare must preserve exact CPU time across several wraps.
    // Exercise both ends of the PAL/NTSC range and every Timer 2 phase.
    const unsigned ratios[]={268,272};
    for(unsigned ratio:ratios)for(unsigned lines=225;lines<=1600;++lines){
        const uint32_t exact=lines*ratio;uint32_t elapsed=0;
        assert(sequence_elapsed_ticks(uint16_t(exact),uint16_t(lines),elapsed)&&elapsed==exact);
    }
    for(unsigned phase=0;phase<65536;++phase){
        const uint32_t exact=94500;uint32_t elapsed=0;
        const uint16_t end=uint16_t(phase+exact);
        assert(sequence_elapsed_ticks(uint16_t(end-uint16_t(phase)),350,elapsed)&&elapsed==exact);
    }
    uint32_t elapsed=0;
    assert(!sequence_elapsed_ticks(1000,1000,elapsed)); // inconsistent counters
    assert(!sequence_elapsed_ticks(1000,1601,elapsed)); // beyond bounded recovery
    const auto delayed_before=psx_audio::now_us;
    COUNTERS[2].value+=uint16_t(94500);COUNTERS[1].value+=350;sequence_clock_irq();
    assert(music_sequence_stats.clock_faults==0&&sequence_clock_diagnostics.recoveries==1);
    assert(psx_audio::now_us-delayed_before>=22320&&psx_audio::now_us-delayed_before<=22322);
    // Unexpected masking remains a real fault outside the explicit transport.
    COUNTERS[2].value+=1000;COUNTERS[1].value+=1000;sequence_clock_irq();
    assert(music_sequence_stats.clock_faults==1&&music_sequence_stats.error==7);
    std::puts("native_music_clock: event deadlines, hardware timer selection, recovery, overdue events and wrap-safe clock passed");
}
