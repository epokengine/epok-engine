#pragma once
#ifdef EPOK_NATIVE_SEQUENCES_ONLY
#include "native_music_runtime.hpp"
#else
// PSX musical service. Four bounded start mailboxes, 24 deferred hardware starts.
// The IRQ reads parameter snapshots, never dereferences AudioSource or invokes gameplay.
#include "audio.hpp"
#include "sequence_data.hpp"
#include "sequence_tables.hpp"
#include "instrument_synth.hpp"
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
enum class Phase : uint8_t { Free, Queued, Active, Retiring };
struct Parameters { uint16_t volume=4095,pitch=4096;uint8_t priority=128; };
inline constexpr uint32_t InstrumentControlPeriodUs=4000;
struct Physical {
    const Zone* zone=nullptr;
    instrument::synth::State synthesis;
    uint32_t generation=0,off_us=0,started_us=0,level=0,step=0,remaining_ms=0,due_us=0;
    uint32_t synthesis_us=0,control_due_us=0;
    int32_t last_cents=INT32_MAX;
    uint16_t last_source_pitch=0;
    uint16_t last_source_volume=UINT16_MAX,last_synth_gain=UINT16_MAX;
    int16_t last_pan=INT16_MAX;
    uint16_t off_ticks=0;
    uint16_t left=0,right=0,pitch=0,library_zone=0;
    uint8_t envelope=0,key=0,velocity=0;
    const native_music::Tone* native_tone=nullptr;
    uint16_t native_pitch=0,native_left=0,native_right=0;
    uint8_t native_lane=0;
    bool pending=false,looping=false,library=false,released=false,reverb_send=false;
    void reset_metadata(){
        zone=nullptr;generation=off_us=started_us=level=step=remaining_ms=due_us=0;
        synthesis_us=control_due_us=0;
        last_cents=INT32_MAX;last_source_pitch=off_ticks=left=right=pitch=library_zone=0;
        last_source_volume=last_synth_gain=UINT16_MAX;last_pan=INT16_MAX;
        envelope=key=velocity=0;pending=looping=library=released=reverb_send=false;
        native_tone=nullptr;native_pitch=native_left=native_right=0;native_lane=0;
    }
};
inline Physical physical[24];
inline instrument::reverb::Resource<> reverb_resource;
inline uint32_t now_us=0,fraction_us=0;
inline uint32_t native_keyoffs=0;
inline uint16_t service_starts=0;
inline bool allocation_current=false;
inline uint32_t allocation_free_mask=0;
inline uint8_t allocation_music_count=0,allocation_own_count[4]{},allocation_ceiling=0,allocation_free_count=0;
inline uint16_t counter_ticks(){
#ifndef EPOK_SEQUENCE_HOST_TEST
    return COUNTERS[2].value;
#else
    return 0;
#endif
}
inline constexpr uint32_t FullLevel=32767u << 16;
inline int first_voice(uint32_t mask){
    if(!mask)return -1;
    int bit=0;
    if(!(mask&0xffff)){mask>>=16;bit+=16;}
    if(!(mask&0xff)){mask>>=8;bit+=8;}
    if(!(mask&15)){mask>>=4;bit+=4;}
    if(!(mask&3)){mask>>=2;bit+=2;}
    return bit+(!(mask&1));
}
struct Instance {
    sequence::Kernel kernel;
    uint32_t note_voices[sequence::Kernel::MaxVoices]{};
    const Sequence* asset=nullptr;
    AudioSource* owner=nullptr; // Identity only inside service(); main-thread updates own dereference.
    uint32_t generation=1,retire_us=0;
    int clip=-1;
    uint8_t index=0;
    Phase phase=Phase::Free;
    Parameters parameters;
    uint32_t native_cursor=0,native_clock=0,native_loop_cursor=0,native_loop_time=0;
    int8_t native_lanes[24];
    void native_begin(){native_cursor=native_clock=native_loop_cursor=native_loop_time=0;for(auto& lane:native_lanes)lane=-1;}
    void native_advance(uint32_t elapsed);
    void native_parameters(int voice);
    bool defer_note(const sequence::Event&,uint32_t cursor){
        if(!asset || kernel.events!=asset->events() || !asset->prepared || !asset->prepared->event_map || cursor>=asset->prepared->event_capacity)return false;
        const auto layers=asset->prepared->event_map[cursor].count;
        // At most one physical pool of starts per service. Admission stays
        // atomic; a busy chord retains its original absolute deadline.
        bool exhausted=service_starts && service_starts+layers>24;
        if(exhausted){++sequence_timing_stats.deferred_events;return true;}
        service_starts+=layers;return false;
    }
    bool owns(int voice,uint16_t note)const{
        return audio_voices[voice].sequence==index && audio_voices[voice].note==note && physical[voice].generation==generation;
    }
    int voice(uint16_t note) const {
        for(uint32_t mask=note_voices[note];mask;mask&=mask-1){const int v=first_voice(mask);if(owns(v,note))return v;}
        return -1;
    }
    void cut(uint16_t note) {
        allocation_current=false;
        for(uint32_t mask=note_voices[note];mask;mask&=mask-1){const int i=first_voice(mask);
            if(!owns(i,note))continue;
            reverb_resource.clear_voice(uint8_t(i));
            if(physical[i].native_tone){native_keyoffs|=1u<<i;native_lanes[physical[i].native_lane]=-1;}
            else audio_keyoff(i);
            audio_voices[i]={};physical[i].reset_metadata();
        }
        note_voices[note]=0;
    }
    void finish_layer(int i) {
        allocation_current=false;
        const auto note=audio_voices[i].note;
        note_voices[note]&=~(1u<<i);
        reverb_resource.clear_voice(uint8_t(i));
        if(physical[i].native_tone){native_keyoffs|=1u<<i;native_lanes[physical[i].native_lane]=-1;}
        else audio_keyoff(i);
        audio_voices[i]={};physical[i].reset_metadata();
        if(!asset->is_native() && voice(note)<0)kernel.retire(note);
    }
    bool advance_library(int number);
    void release_library(uint16_t note);
    void update_library(int physical_voice,const sequence::Channel& channel,bool controls_changed);
    void release(uint16_t note) {
        if(asset->bank->is_library()){release_library(note);return;}
        const int i=voice(note);if(i<0)return;
        auto& v=physical[i];if(v.envelope==3)return;
        v.envelope=3;v.remaining_ms=v.zone->release_ms;
        if(!v.remaining_ms || !v.level){cut(note);kernel.retire(note);return;}
        v.step=(v.level+v.remaining_ms-1)/v.remaining_ms;
    }
    void update(uint16_t note,const sequence::Channel& channel) {
        if(asset->bank->is_library()){
            for(uint32_t mask=note_voices[note];mask;mask&=mask-1){const int i=first_voice(mask);if(owns(i,note))update_library(i,channel,true);}
            return;
        }
        const int i=voice(note);if(i<0)return;
        auto& v=physical[i];const auto& z=*v.zone;
        // Hundredths of a cent. Pitch affects samples, never sequence time.
        const int64_t cents=int64_t(int(v.key)-z.root_key)*10000+z.cents+channel.pitch_cents100();
        int64_t octave=cents/120000,remainder=cents%120000;
        if(remainder<0){remainder+=120000;--octave;}
        const unsigned position=unsigned(remainder)/100,part=unsigned(remainder)%100;
        const uint32_t ratio=cent_ratio[position]+(cent_ratio[position+1]-cent_ratio[position])*part/100;
        const uint64_t denominator=uint64_t(44100)<<20;
        const uint64_t base=uint64_t(asset->bank->sample(z.sample).rate)*ratio*parameters.pitch;
        const uint64_t clamp_numerator=uint64_t(0x4000)*denominator;
        uint64_t pitch=0;
        bool high=false;
        if(octave>=0){
            const uint64_t shift=uint64_t(octave);
            high=shift>=64 || base>((clamp_numerator-1)>>unsigned(shift));
            if(!high)pitch=(base<<unsigned(shift))/denominator;
        }else{
            const uint64_t shift=uint64_t(-(octave+1))+1;
            pitch=shift>=64?0:(base>>unsigned(shift))/denominator;
        }
        if(high || pitch<1 || pitch>0x3fff){++music_sequence_stats.pitch_clamps;pitch=high || pitch>0x3fff?0x3fff:1;}
        v.pitch=uint16_t(pitch);
        int pan=int(z.pan)+(int(channel.pan)-64)*256;
        if(pan < -16384)pan=-16384;if(pan>16384)pan=16384;
        const unsigned pan_index=unsigned(pan+16384)/256;
        uint32_t gain=uint32_t(z.gain)*v.velocity/127;
        gain=gain*channel.volume/127;gain=gain*channel.expression/127;
        gain=gain*parameters.volume/4096;
        const uint32_t left=gain*pan_gain[128-pan_index]/4096;
        const uint32_t right=gain*pan_gain[pan_index]/4096;
        v.left=uint16_t(left>16383?16383:left);v.right=uint16_t(right>16383?16383:right);
        if(!v.pending)SPU_VOICES[i].sampleRate=v.pitch;
    }
    bool start(uint16_t note,const sequence::Note& n,const sequence::Channel& channel);
    bool start_library(uint16_t note,const sequence::Note& n,const sequence::Channel& channel);
};
inline Instance instances[4];

inline void retire(Instance& instance) {
    allocation_current=false;
    if(instance.asset && instance.asset->is_native()){for(unsigned n=0;n<sequence::Kernel::MaxVoices;++n)if(instance.note_voices[n])instance.cut(uint16_t(n));}
    else instance.kernel.stop(instance);
    if(reverb_resource.active() && reverb_resource.owner()==instance.index && reverb_resource.generation()==instance.generation)reverb_resource.release(instance.index,instance.generation);
    instance.owner=nullptr;instance.phase=Phase::Retiring;instance.retire_us=now_us;
}
inline void stolen(int i) {
    auto& voice=audio_voices[i];
    if(voice.sequence<0)return;
    auto& instance=instances[voice.sequence];
    const auto note=voice.note;
    instance.cut(note);if(!instance.asset->is_native())instance.kernel.retire(note);++music_sequence_stats.steals;
}
inline void next_envelope(Physical& v) {
    if(v.envelope==0){
        v.level=FullLevel;v.envelope=1;v.remaining_ms=v.zone->decay_ms;
        const uint32_t target=uint32_t(v.zone->sustain)<<16;
        v.step=v.remaining_ms?(FullLevel-target+v.remaining_ms-1)/v.remaining_ms:0;
        if(v.remaining_ms)return;
    }
    if(v.envelope==1){v.level=uint32_t(v.zone->sustain)<<16;v.envelope=2;v.remaining_ms=0;v.step=0;}
}
inline bool Instance::start(uint16_t note,const sequence::Note& n,const sequence::Channel& channel) {
    if(asset->bank->is_library())return start_library(note,n,channel);
    // EPSB v1 has only logical bank 0. Refuse a requested bank before choosing a zone.
    if(channel.bank) return false;
    const Zone* zone=nullptr;
    for(uint16_t i=0;i<asset->bank->zone_count();++i){
        const auto& z=asset->bank->zone(i);
        if(z.program==channel.program && z.drum_key==(n.channel==9?n.key:255) && n.key>=z.key_lo && n.key<=z.key_hi && n.velocity>=z.velocity_lo && n.velocity<=z.velocity_hi){zone=&z;break;}
    }
    if(!zone)return false;
    cut(note);
    instrument::allocation::Slot slots[24]{};uint8_t ceiling=uint8_t(asset->voices());
    for(const auto& active:instances)if((active.phase==Phase::Active || active.phase==Phase::Queued) && active.asset && active.asset->voices()>ceiling)ceiling=uint8_t(active.asset->voices());
    for(int p=0;p<24;++p){const auto& v=audio_voices[p];if(v.owner)slots[p]={true,v.sequence>=0,uint8_t(v.sequence>=0?v.sequence:0),v.note,v.priority,physical[p].generation,v.started};}
    const auto plan=instrument::allocation::reserve(slots,{index,parameters.priority,1,uint8_t(asset->voices()),ceiling,generation});
    if(!plan.fits){++music_sequence_stats.denied_notes;kernel.retire(note);return true;}
    int chosen=-1;
    for(int p=0;p<24;++p){
        if((plan.evict_mask&(1u<<p)) && audio_voices[p].owner){
            if(audio_voices[p].sequence>=0)stolen(p);else {audio_keyoff(p);audio_voices[p]={};physical[p].reset_metadata();++music_sequence_stats.steals;}
        }
        if(plan.start_mask&(1u<<p))chosen=p;
    }
    audio_keyoff(chosen);
    audio_voices[chosen]={owner,audio_frame,clip,int8_t(index),uint8_t(note),parameters.priority};
    note_voices[note]=1u<<chosen;
    auto& v=physical[chosen];v.reset_metadata();v.zone=zone;v.generation=generation;v.off_us=now_us;v.off_ticks=counter_ticks();
    const uint64_t late=kernel.clock-kernel.event_time;
    v.due_us=now_us-uint32_t(late>UINT32_MAX?late/kernel.ppqn:uint32_t(late)/kernel.ppqn);
    v.key=n.key;v.velocity=n.velocity;v.pending=true;v.looping=asset->bank->sample(zone->sample).loop_end!=0;
    v.remaining_ms=zone->attack_ms;v.step=v.remaining_ms?(FullLevel+v.remaining_ms-1)/v.remaining_ms:0;
    if(!v.remaining_ms)next_envelope(v);
    update(note,channel);
    return true;
}
}

#include "sequence_instrument_service.hpp"
#include "native_music_service.hpp"

inline void sequence_voice_stolen(int voice){psx_audio::stolen(voice);}
inline bool sequence_is_playing(const AudioSource* source){
    for(const auto& i:psx_audio::instances)if(i.owner==source && (i.phase==psx_audio::Phase::Queued || i.phase==psx_audio::Phase::Active))return true;
    return false;
}
inline void sequence_stop(AudioSource* source){
    for(auto& i:psx_audio::instances)if(i.owner==source)psx_audio::retire(i);
}
inline bool sequence_retiring(){
    SequenceLock lock;
    for(const auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Retiring)return true;
    return false;
}
inline psx_audio::Parameters sequence_parameters(const AudioSource* source){
    int volume=source->volume.raw();if(volume<0)volume=0;if(volume>4095)volume=4095;
    volume=int(uint32_t(volume)*transition_audio_gain()/4096);
    int pitch=source->pitch.raw();if(pitch<1024)pitch=1024;if(pitch>16384)pitch=16384;
    return {uint16_t(volume),uint16_t(pitch),source->priority};
}
inline void sequence_play(AudioSource* source){
    if(!music_sequence_stats.ready){music_sequence_stats.error=1;return;}
    source->stop();
    for(auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Free){
        i.owner=source;i.clip=source->clip;i.asset=audio_bank[source->clip].sequence;
        i.parameters=sequence_parameters(source);++i.generation;if(!i.generation)++i.generation;
        if(i.asset->bank->is_library() && i.asset->bank->library().reverb_preset() &&
            psx_audio::reverb_resource.acquire(i.index,i.generation)!=instrument::reverb::Error::None){
            i.owner=nullptr;i.asset=nullptr;music_sequence_stats.error=11;++music_sequence_stats.capacity_errors;return;
        }
        // Initialize the unpublished mailbox on the main thread. Clearing the
        // bounded FIFO ledger is preparation work, not note-on IRQ work.
        if(i.asset->is_native())i.native_begin();
        else i.kernel.begin_validated(i.asset->events(),i.asset->count(),i.asset->ppqn(),i.asset->bank->is_library()?sequence::Kernel::MaxVoices:i.asset->voices());
        ++i.asset->bank->pins;i.phase=psx_audio::Phase::Queued;return;
    }
    ++music_sequence_stats.capacity_errors;music_sequence_stats.error=2;
}
inline void sequence_update_sources(){
    for(auto& i:psx_audio::instances){
        if(!i.owner)continue;
        if(!i.owner->enabled || i.owner->clip!=i.clip){psx_audio::retire(i);continue;}
        const auto previous=i.parameters;
        i.parameters=sequence_parameters(i.owner);
        if(previous.volume!=i.parameters.volume || previous.pitch!=i.parameters.pitch){
            if(i.asset->is_native()){
                for(int p=0;p<24;++p)if(audio_voices[p].sequence==i.index && psx_audio::physical[p].generation==i.generation)i.native_parameters(p);
            }else if(i.asset->bank->is_library()){
                for(int p=0;p<24;++p)if(audio_voices[p].sequence==i.index && psx_audio::physical[p].generation==i.generation)
                    i.update_library(p,i.kernel.channels[i.kernel.notes[audio_voices[p].note].channel],false);
            }else for(uint16_t n=0;n<i.kernel.limit;++n)if(i.kernel.notes[n].active)i.update(n,i.kernel.channels[i.kernel.notes[n].channel]);
        }
        if(previous.priority!=i.parameters.priority)
            for(auto& v:audio_voices)if(v.sequence==i.index)v.priority=i.parameters.priority;
    }
    psx_audio::native_flush_keyoffs();
}

inline void sequence_service(uint32_t elapsed_us){
    using namespace psx_audio;
    if(!music_sequence_stats.ready)return;
    service_starts=0;
    allocation_current=false;
    ++music_sequence_stats.services;now_us+=elapsed_us;music_sequence_stats.clock_us=now_us;
    fraction_us+=elapsed_us;const uint32_t milliseconds=fraction_us/1000;fraction_us%=1000;
    const uint32_t started=sequence_flush_starts();
    const uint32_t ended=HW_U16(0x1f801d9c)|(uint32_t(HW_U16(0x1f801d9e))<<16);
    uint32_t occupied=0;
    for(int n=0;n<24;++n){
        auto& av=audio_voices[n];if(av.sequence<0)continue;++occupied;
        auto& i=instances[av.sequence];auto& v=physical[n];
        if(v.generation!=i.generation || i.phase!=Phase::Active){i.cut(av.note);continue;}
        if(v.pending || (started&(1u<<n)))continue;
        if(v.native_tone){
            // Envelopes advance entirely in the SPU. Only a release which
            // preceded deferred key-on needs a one-time hardware command.
            if(v.released && !v.envelope){native_keyoffs|=1u<<n;v.envelope=1;}
            continue;
        }
        if(v.library){
            // ENDX remains set after any loop end; UntilRelease therefore uses
            // the software envelope to retire its tail, never that sticky bit.
            if(!v.looping && uint32_t(now_us-v.started_us)>2000 && (ended&(1u<<n))){i.finish_layer(n);continue;}
            // MIDI scheduling stays on every IRQ. Software modulation is held
            // between 250 Hz control updates; elapsed time is never discarded.
            if(int32_t(now_us-v.control_due_us)>=0 && i.advance_library(n))
                i.update_library(n,i.kernel.channels[i.kernel.notes[av.note].channel],false);
            continue;
        }
        if(!v.looping && uint32_t(now_us-v.started_us)>2000 && (ended&(1u<<n))){const auto note=av.note;i.cut(note);i.kernel.retire(note);continue;}
        uint32_t remaining=milliseconds;
        while(remaining && v.envelope!=2){
            const auto step=remaining<v.remaining_ms?remaining:v.remaining_ms;
            remaining-=step;v.remaining_ms-=step;
            if(v.envelope==0){
                if(step && v.step>(FullLevel-v.level)/step)v.level=FullLevel;else v.level+=v.step*step;
            }else{
                if(step && v.step>v.level/step)v.level=0;else v.level-=v.step*step;
            }
            if(!v.remaining_ms){
                if(v.envelope==3){const auto note=av.note;i.cut(note);i.kernel.retire(note);break;}
                next_envelope(v);
            }
        }
        if(!v.zone)continue;
        SPU_VOICES[n].volumeLeft=uint16_t(uint32_t(v.left)*(v.level>>16)/32767);
        SPU_VOICES[n].volumeRight=uint16_t(uint32_t(v.right)*(v.level>>16)/32767);
    }
    if(occupied>music_sequence_stats.peak_voices)music_sequence_stats.peak_voices=occupied;
    music_sequence_stats.active_sequences=0;
    for(auto& i:instances){
        if(i.phase==Phase::Retiring){
            if(uint32_t(now_us-i.retire_us)>=2000){--i.asset->bank->pins;i.asset=nullptr;i.phase=Phase::Free;}
            continue;
        }
        uint32_t delta=elapsed_us;
        if(i.phase==Phase::Queued){
            i.phase=Phase::Active;++music_sequence_stats.starts;delta=0;
        }
        if(i.phase!=Phase::Active)continue;
        ++music_sequence_stats.active_sequences;
        if(i.asset->is_native()){i.native_advance(delta);continue;}
        const auto loops=i.kernel.loops;
        i.kernel.advance(delta,i);music_sequence_stats.loops+=i.kernel.loops-loops;
        if(i.kernel.error!=sequence::Error::None){music_sequence_stats.error=uint32_t(i.kernel.error)+3;retire(i);}
        else if(!i.kernel.running && !i.kernel.active()){++music_sequence_stats.ends;retire(i);}
    }
    native_flush_keyoffs();sequence_flush_starts();
}
inline void sequence_clock_fault(){
    ++music_sequence_stats.clock_faults;music_sequence_stats.error=7;
    for(auto& i:psx_audio::instances)if(i.phase==psx_audio::Phase::Active || i.phase==psx_audio::Phase::Queued)psx_audio::retire(i);
}
inline bool sequence_prepare(){
    using namespace psx_audio;
    uint32_t reverb_preset=0;uint16_t reverb_depth=0;
    for(size_t n=0;n<audio_count;++n)if(audio_bank[n].sequence && !audio_bank[n].sequence->valid()){music_sequence_stats.error=3;return false;}
    uint32_t total=audio_upload_address;
    for(size_t n=0;n<audio_count;++n){
        const auto* sequence=audio_bank[n].sequence;if(!sequence)continue;
        const auto& bank=*sequence->bank;
        if(bank.is_library() && bank.library().reverb_preset()){
            if(reverb_preset && reverb_depth!=bank.library().reverb_depth()){music_sequence_stats.error=11;return false;}
            reverb_preset=1;reverb_depth=bank.library().reverb_depth();
        }
        bool earlier=false;for(size_t j=0;j<n;++j)if(audio_bank[j].sequence && audio_bank[j].sequence->bank==&bank)earlier=true;
        if(!bank.ready && !earlier)for(uint16_t j=0;j<bank.sample_count();++j){
            if(bank.sample(j).size>512*1024-total){music_sequence_stats.error=8;return false;}
            total+=bank.sample(j).size;
        }
    }
    const auto preset=reverb_preset?instrument::reverb::Preset::Room:instrument::reverb::Preset::Dry;
    if(total>512*1024-(reverb_preset?instrument::reverb::room_bytes:0)){music_sequence_stats.error=8;return false;}
    if(reverb_resource.prepare(total,preset,reverb_depth)!=instrument::reverb::Error::None){music_sequence_stats.error=11;return false;}
    for(size_t n=0;n<audio_count;++n){
        const auto* sequence=audio_bank[n].sequence;if(!sequence || sequence->bank->ready)continue;
        auto& bank=*sequence->bank;
        for(uint16_t j=0;j<bank.sample_count();++j){
            const auto& sample=bank.sample(j);
            if(audio_upload_address+sample.size>512*1024){music_sequence_stats.error=8;return false;}
            bank.addresses[j]=uint16_t(audio_upload_address>>3);
            if(!spu::upload(bank.data+sample.offset,audio_upload_address,sample.size)){music_sequence_stats.error=9;return false;}
            audio_upload_address+=sample.size;
        }
        bank.ready=true;
    }
    SPU_CTRL=0xc000;
    for(uint8_t j=0;j<4;++j)instances[j].index=j;
    for(size_t n=0;n<audio_count;++n){
        const auto* sequence=audio_bank[n].sequence;if(!sequence || sequence->is_native() || !sequence->bank->is_library())continue;
#ifndef EPOK_SEQUENCE_HOST_TEST
        if(!sequence->prepared){music_sequence_stats.error=12;return false;}
#endif
        if(sequence->prepared && !sequence->prepared->prepare(sequence->bank->library(),sequence->events(),sequence->count(),sequence->ppqn())){
            music_sequence_stats.error=12;return false;
        }
    }
    music_sequence_stats.ready=1;return true;
}
}
#endif
