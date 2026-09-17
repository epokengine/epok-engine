#pragma once
// Native-only builds do not reserve the MIDI FIFO or software synth states.
#include "audio.hpp"
#include "sequence_data.hpp"
#include "instrument_allocator.hpp"
#include "instrument_reverb.hpp"
#ifndef EPOK_SEQUENCE_HOST_TEST
#include "common/hardware/counters.h"
#endif
namespace epok {
struct SequenceStats {
    uint32_t ready=0,services=0,starts=0,ends=0,loops=0,steals=0,denied_notes=0,
        capacity_errors=0,pitch_clamps=0,peak_voices=0,active_sequences=0,error=0,
        clock_faults=0,max_gap_us=0,max_service_ticks=0,clock_us=0;
};
inline SequenceStats music_sequence_stats;
struct SequenceTimingStats {uint32_t deferred_events=0,max_key_on_delay_us=0,key_ons=0;uint64_t service_ticks=0;};
inline SequenceTimingStats sequence_timing_stats;
inline uint16_t sequence_irq_begin_ticks=0;
namespace psx_audio {
enum class Phase:uint8_t{Free,Queued,Active,Retiring};
struct Parameters{uint16_t volume=4095,pitch=4096;uint8_t priority=128;};
struct Physical {
    const native_music::Tone* native_tone=nullptr;
    uint32_t generation=0,off_us=0,due_us=0,started_us=0;
    uint16_t native_pitch=0,native_left=0,native_right=0,pitch=0,left=0,right=0,off_ticks=0;
    uint8_t native_lane=0,envelope=0;
    bool pending=false,released=false,looping=false,reverb_send=false;
    void reset_metadata(){*this=Physical{};}
};
inline Physical physical[24];
inline instrument::reverb::Resource<> reverb_resource;
inline uint32_t now_us=0,native_keyoffs=0;
inline bool allocation_current=false;
inline uint16_t counter_ticks(){
#ifndef EPOK_SEQUENCE_HOST_TEST
    return COUNTERS[2].value;
#else
    return 0;
#endif
}
inline int first_voice(uint32_t mask){
    if(!mask)return -1;int bit=0;
    if(!(mask&0xffff)){mask>>=16;bit+=16;}if(!(mask&0xff)){mask>>=8;bit+=8;}
    if(!(mask&15)){mask>>=4;bit+=4;}if(!(mask&3)){mask>>=2;bit+=2;}
    return bit+(!(mask&1));
}
struct Instance {
    uint32_t note_voices[128]{};
    const Sequence* asset=nullptr;AudioSource* owner=nullptr;
    uint32_t generation=1,retire_us=0;
    int clip=-1;uint8_t index=0;Phase phase=Phase::Free;Parameters parameters;
    uint32_t native_cursor=0,native_clock=0,native_loop_cursor=0,native_loop_time=0;
    int8_t native_lanes[24];
    void native_begin(){native_cursor=native_clock=native_loop_cursor=native_loop_time=0;for(auto& lane:native_lanes)lane=-1;}
    void native_advance(uint32_t elapsed);
    void native_parameters(int voice);
    bool owns(int voice,uint16_t note)const{return audio_voices[voice].sequence==index && audio_voices[voice].note==note && physical[voice].generation==generation;}
    void cut(uint16_t note){
        for(uint32_t mask=note_voices[note];mask;mask&=mask-1){const int p=first_voice(mask);if(!owns(p,note))continue;
            native_keyoffs|=1u<<p;native_lanes[physical[p].native_lane]=-1;reverb_resource.clear_voice(uint8_t(p));
            audio_voices[p]={};physical[p].reset_metadata();}
        note_voices[note]=0;
    }
    void finish_layer(int p){
        note_voices[audio_voices[p].note]&=~(1u<<p);native_keyoffs|=1u<<p;
        native_lanes[physical[p].native_lane]=-1;reverb_resource.clear_voice(uint8_t(p));
        audio_voices[p]={};physical[p].reset_metadata();
    }
};
inline Instance instances[4];
inline void retire(Instance& i){
    for(unsigned n=0;n<128;++n)if(i.note_voices[n])i.cut(uint16_t(n));
    if(reverb_resource.active() && reverb_resource.owner()==i.index && reverb_resource.generation()==i.generation)reverb_resource.release(i.index,i.generation);
    i.owner=nullptr;i.phase=Phase::Retiring;i.retire_us=now_us;
}
inline void stolen(int p){if(audio_voices[p].sequence<0)return;auto& i=instances[audio_voices[p].sequence];i.cut(audio_voices[p].note);++music_sequence_stats.steals;}
}
#include "native_music_service.hpp"
inline void sequence_voice_stolen(int p){psx_audio::stolen(p);psx_audio::native_flush_keyoffs();}
inline bool sequence_is_playing(const AudioSource* s){for(const auto& i:psx_audio::instances)if(i.owner==s && (i.phase==psx_audio::Phase::Active || i.phase==psx_audio::Phase::Queued))return true;return false;}
inline void sequence_stop(AudioSource* s){for(auto& i:psx_audio::instances)if(i.owner==s)psx_audio::retire(i);psx_audio::native_flush_keyoffs();}
inline bool sequence_retiring(){SequenceLock lock;for(const auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Retiring)return true;return false;}
inline psx_audio::Parameters sequence_parameters(const AudioSource* s){
    int volume=s->volume.raw();if(volume<0)volume=0;if(volume>4095)volume=4095;
    volume=int(uint32_t(volume)*transition_audio_gain()/4096);
    int pitch=s->pitch.raw();if(pitch<1024)pitch=1024;if(pitch>16384)pitch=16384;
    return {uint16_t(volume),uint16_t(pitch),s->priority};
}
inline void sequence_play(AudioSource* s){
    if(!music_sequence_stats.ready){music_sequence_stats.error=1;return;}s->stop();
    for(auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Free){
        i.owner=s;i.clip=s->clip;i.asset=audio_bank[s->clip].sequence;i.parameters=sequence_parameters(s);
        ++i.generation;if(!i.generation)++i.generation;
        if(i.asset->bank->library().reverb_preset() && psx_audio::reverb_resource.acquire(i.index,i.generation)!=instrument::reverb::Error::None){
            i.owner=nullptr;i.asset=nullptr;music_sequence_stats.error=11;++music_sequence_stats.capacity_errors;return;}
        i.native_begin();++i.asset->bank->pins;i.phase=psx_audio::Phase::Queued;return;
    }
    ++music_sequence_stats.capacity_errors;music_sequence_stats.error=2;
}
inline void sequence_update_sources(){
    for(auto& i:psx_audio::instances){if(!i.owner)continue;
        if(!i.owner->enabled || i.owner->clip!=i.clip){psx_audio::retire(i);continue;}
        const auto previous=i.parameters;i.parameters=sequence_parameters(i.owner);
        if(previous.volume!=i.parameters.volume || previous.pitch!=i.parameters.pitch)
            for(int p=0;p<24;++p)if(audio_voices[p].sequence==i.index && psx_audio::physical[p].generation==i.generation)i.native_parameters(p);
        if(previous.priority!=i.parameters.priority)for(auto& v:audio_voices)if(v.sequence==i.index)v.priority=i.parameters.priority;
    }
    psx_audio::native_flush_keyoffs();
}
inline void sequence_service(uint32_t elapsed_us){
    using namespace psx_audio;if(!music_sequence_stats.ready)return;
    ++music_sequence_stats.services;now_us+=elapsed_us;music_sequence_stats.clock_us=now_us;
    uint32_t started=0,occupied=0;
    for(int p=0;p<24;++p){const auto& av=audio_voices[p];if(av.sequence<0)continue;++occupied;
        auto& v=physical[p];auto& i=instances[av.sequence];
        if(v.generation!=i.generation || i.phase!=Phase::Active){i.cut(av.note);continue;}
        if(!v.pending){if(v.released && !v.envelope){native_keyoffs|=1u<<p;v.envelope=1;}continue;}
        if(uint32_t(now_us-v.off_us)<30)continue;
        const auto& patch=*v.native_tone;const auto& sample=i.asset->bank->sample(patch.sample);auto& hw=SPU_VOICES[p];
        hw.sampleStartAddr=i.asset->bank->addresses[patch.sample];
        hw.sampleRepeatAddr=hw.sampleStartAddr+uint16_t((v.released && patch.loop==3?sample.loop_end:sample.loop_start)/28*2);
        hw.adsrLo=patch.adsr1;hw.adsrHi=patch.adsr2;hw.sampleRate=v.pitch;hw.volumeLeft=v.left;hw.volumeRight=v.right;
        v.pending=false;v.started_us=now_us;started|=1u<<p;reverb_resource.send(uint8_t(p),v.reverb_send);++sequence_timing_stats.key_ons;
    }
    native_flush_keyoffs();
    if(started&0xffff)SPU_KEY_ON_LOW=uint16_t(started);if(started>>16)SPU_KEY_ON_HIGH=uint16_t(started>>16);
    uint32_t write_delay=0;
#ifndef EPOK_SEQUENCE_HOST_TEST
    write_delay=(uint32_t(uint16_t(counter_ticks()-sequence_irq_begin_ticks))*625+2645)/2646;
#endif
    for(uint32_t mask=started;mask;mask&=mask-1){const auto delay=uint32_t(now_us-physical[first_voice(mask)].due_us)+write_delay;
        if(delay>sequence_timing_stats.max_key_on_delay_us)sequence_timing_stats.max_key_on_delay_us=delay;}
    if(occupied>music_sequence_stats.peak_voices)music_sequence_stats.peak_voices=occupied;
    music_sequence_stats.active_sequences=0;
    for(auto& i:instances){
        if(i.phase==Phase::Retiring){if(uint32_t(now_us-i.retire_us)>=2000){--i.asset->bank->pins;i.asset=nullptr;i.phase=Phase::Free;}continue;}
        uint32_t delta=elapsed_us;if(i.phase==Phase::Queued){i.phase=Phase::Active;++music_sequence_stats.starts;delta=0;}
        if(i.phase!=Phase::Active)continue;++music_sequence_stats.active_sequences;i.native_advance(delta);
    }
    native_flush_keyoffs();
}
inline void sequence_clock_fault(){++music_sequence_stats.clock_faults;music_sequence_stats.error=7;
    for(auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Active || i.phase==psx_audio::Phase::Queued)psx_audio::retire(i);
    psx_audio::native_flush_keyoffs();}
inline bool sequence_prepare(){
    using namespace psx_audio;uint32_t total=audio_upload_address,preset=0;uint16_t depth=0;
    for(size_t n=0;n<audio_count;++n){const auto* s=audio_bank[n].sequence;if(!s)continue;
        if(!s->is_native() || !s->bank->is_library() || !s->valid()){music_sequence_stats.error=3;return false;}
        const auto& bank=*s->bank;
        if(bank.library().reverb_preset()){if(preset && depth!=bank.library().reverb_depth()){music_sequence_stats.error=11;return false;}preset=1;depth=bank.library().reverb_depth();}
        bool earlier=false;for(size_t j=0;j<n;++j)if(audio_bank[j].sequence && audio_bank[j].sequence->bank==&bank)earlier=true;
        if(!bank.ready && !earlier)for(uint16_t j=0;j<bank.sample_count();++j){if(total>512*1024 || bank.sample(j).size>512*1024-total){music_sequence_stats.error=8;return false;}total+=bank.sample(j).size;}
    }
    if(total>512*1024-(preset?instrument::reverb::room_bytes:0)){music_sequence_stats.error=8;return false;}
    if(reverb_resource.prepare(total,preset?instrument::reverb::Preset::Room:instrument::reverb::Preset::Dry,depth)!=instrument::reverb::Error::None){music_sequence_stats.error=11;return false;}
    for(size_t n=0;n<audio_count;++n){const auto* s=audio_bank[n].sequence;if(!s || s->bank->ready)continue;auto& bank=*s->bank;
        for(uint16_t j=0;j<bank.sample_count();++j){const auto& sample=bank.sample(j);bank.addresses[j]=uint16_t(audio_upload_address>>3);
            if(!spu::upload(bank.data+sample.offset,audio_upload_address,sample.size)){music_sequence_stats.error=9;return false;}audio_upload_address+=sample.size;}
        bank.ready=true;
    }
    SPU_CTRL=0xc000;for(uint8_t j=0;j<4;++j)instances[j].index=j;music_sequence_stats.ready=1;return true;
}
}
