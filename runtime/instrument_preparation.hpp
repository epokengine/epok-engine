#pragma once
// Immutable initial voice states, prepared before the audio clock starts.
// The resident event stream fixes every note's initial controls, including
// restored loop state. No synthesis, searching of a library, or allocation is
// needed to rebuild those initial parameters inside the note-on interrupt.
#include "instrument_synth.hpp"
#include "sequence_kernel.hpp"
#include "sequence_tables.hpp"

namespace epok::instrument::preparation {
// Exact initial SPU pitch for the common AudioSource pitch of 1.0. Compute it
// before the clock starts; other source pitches retain the live calculation.
inline uint16_t initial_pitch(const BankView& bank,uint16_t zone,const synth::State& state){
    const int64_t cents=state.output().pitch_cents_x100;
    int64_t octave=cents/120000,remainder=cents%120000;
    if(remainder<0){remainder+=120000;--octave;}
    const auto position=unsigned(remainder)/100,part=unsigned(remainder)%100;
    const uint32_t ratio=psx_audio::cent_ratio[position]+(psx_audio::cent_ratio[position+1]-psx_audio::cent_ratio[position])*part/100;
    constexpr uint64_t denominator=uint64_t(44100)<<20,ceiling=uint64_t(0x4000)*denominator;
    const uint64_t base=uint64_t(bank.sample(bank.zone(zone).sample).rate)*ratio*4096;
    uint64_t pitch=0;
    if(octave>=0){
        const auto shift=uint64_t(octave);
        if(shift>=64 || base>((ceiling-1)>>unsigned(shift)))return 0;
        pitch=(base<<unsigned(shift))/denominator;
    }else{
        const auto shift=uint64_t(-(octave+1))+1;
        pitch=shift>=64?0:(base>>unsigned(shift))/denominator;
    }
    return pitch>=1 && pitch<=0x3fff?uint16_t(pitch):0;
}
inline synth::Controls controls(const sequence::Channel& c) {
    synth::Controls result;
    result.cc[1]=c.modulation;result.cc[7]=c.volume;result.cc[10]=c.pan;result.cc[11]=c.expression;
    result.cc[64]=c.sustain?127:0;result.cc[66]=c.sostenuto?127:0;result.cc[91]=c.reverb;
    result.bend=c.bend;result.bend_range_cents=c.bend_range_cents;
    result.fine_tuning=c.fine_tuning;result.coarse_tuning=c.coarse_tuning;
    return result;
}
struct Key {
    uint32_t words[6]{};
    Key()=default;
    Key(uint16_t zone,uint8_t note,uint8_t velocity,const sequence::Channel& c) {
        words[0]=zone|(uint32_t(note)<<16)|(uint32_t(velocity)<<24);
        words[1]=c.program|(uint32_t(c.bank)<<8)|(uint32_t(c.coarse_tuning)<<24);
        words[2]=c.volume|(uint32_t(c.pan)<<8)|(uint32_t(c.expression)<<16)|(uint32_t(c.modulation)<<24);
        words[3]=c.bend|(uint32_t(c.bend_range_cents)<<16);
        words[4]=c.fine_tuning|(uint32_t(c.reverb)<<16)|(uint32_t(bool(c.sustain))<<24)|(uint32_t(bool(c.sostenuto))<<25);
        // reverb_set gates hardware sends outside the synth; it does not alter
        // its initial state. Keep reserved bits deterministic on every host.
    }
    bool operator==(const Key& rhs)const {
        for(unsigned i=0;i<6;++i)if(words[i]!=rhs.words[i])return false;
        return true;
    }
    uint32_t hash()const {
        uint32_t result=2166136261u;
        for(auto word:words){result=(result^word)*16777619u;result^=result>>16;}
        return result;
    }
};
inline constexpr uint16_t MaxStarts=1024;
inline constexpr uint16_t bucket_count(uint16_t capacity) {
    uint16_t result=2;while(result<capacity*2)result*=2;return result;
}
struct EventStart {uint16_t offset=0,count=0;};
struct Cache {
    Key* keys;
    synth::State* states; // Null only during host capacity analysis.
    uint16_t* buckets;
    uint16_t capacity,bucket_size,count=0;
    bool ready=false;
    EventStart* event_map=nullptr;
    uint16_t* references=nullptr;
    uint16_t* pitches=nullptr;
    uint32_t event_capacity=0,reference_capacity=0,reference_count=0;
    Cache(Key* k,synth::State* s,uint16_t* b,uint16_t n,uint16_t bs):keys(k),states(s),buckets(b),capacity(n),bucket_size(bs){}
    uint16_t slot(const Key& key)const {
        uint16_t at=uint16_t(key.hash()&(bucket_size-1));
        for(uint16_t probes=0;probes<bucket_size;++probes){
            if(!buckets[at] || keys[buckets[at]-1]==key)return at;
            at=uint16_t((at+1)&(bucket_size-1));
        }
        return bucket_size;
    }
    const synth::State* find(uint16_t zone,uint8_t note,uint8_t velocity,const sequence::Channel& channel)const {
        if(!ready || !states)return nullptr;
        const auto at=slot(Key(zone,note,velocity,channel));
        return at<bucket_size && buckets[at]?states+buckets[at]-1:nullptr;
    }
    uint16_t insert(const BankView& bank,uint16_t zone,const sequence::Note& note,const sequence::Channel& channel) {
        const Key key(zone,note.key,note.velocity,channel);
        const auto at=slot(key);
        if(at==bucket_size)return UINT16_MAX;
        if(buckets[at])return uint16_t(buckets[at]-1);
        if(count==capacity)return UINT16_MAX;
        if(states && states[count].start_validated(bank,zone,note.key,note.velocity,controls(channel))!=synth::Error::None)return UINT16_MAX;
        if(states && pitches)pitches[count]=initial_pitch(bank,zone,states[count]);
        keys[count]=key;buckets[at]=++count;return uint16_t(count-1);
    }
    bool prepare(const BankView& bank,const sequence::Event* events,uint32_t size,uint16_t ppqn) {
        ready=false;count=0;reference_count=0;
        if(!capacity || capacity>MaxStarts || bucket_size!=bucket_count(capacity) || !keys || !buckets || !bank.valid())return false;
        for(uint16_t i=0;i<bucket_size;++i)buckets[i]=0;
        if(event_map){
            if(event_capacity!=size || !references)return false;
            for(uint32_t i=0;i<event_capacity;++i)event_map[i]={};
        }
        sequence::Kernel kernel;
        if(!kernel.begin(events,size,ppqn,128))return false;
        struct Backend {
            Cache& cache;sequence::Kernel& kernel;const BankView& bank;
            bool start(uint16_t index,const sequence::Note& note,const sequence::Channel& channel) {
                if(kernel.loops){kernel.retire(index);return true;}
                bool matched=false;
                EventStart mapping{uint16_t(cache.reference_count),0};
                for(uint16_t z=0;z<bank.zone_count();++z){
                    const auto& zone=bank.zone(z);
                    if(zone.bank!=channel.bank || zone.program!=channel.program || zone.percussion!=uint8_t(note.channel==9) ||
                        note.key<zone.key_lo || note.key>zone.key_hi || note.velocity<zone.velocity_lo || note.velocity>zone.velocity_hi)continue;
                    matched=true;
                    const auto entry=cache.insert(bank,z,note,channel);
                    if(entry==UINT16_MAX || cache.reference_count==UINT16_MAX)return false;
                    if(cache.references){
                        if(cache.reference_count==cache.reference_capacity)return false;
                        cache.references[cache.reference_count]=entry;
                    }
                    ++cache.reference_count;++mapping.count;
                }
                if(cache.event_map)cache.event_map[kernel.cursor-1]=mapping;
                // No physical voices during loading. Preserve the kernel's
                // tombstones so later Note Off / pedal/reset semantics remain valid.
                kernel.retire(index);return matched;
            }
            void cut(uint16_t){}
            void release(uint16_t index){kernel.retire(index);}
            void update(uint16_t,const sequence::Channel&){}
        } backend{*this,kernel,bank};
        while(kernel.running && !kernel.loops){
            const auto& next=events[kernel.cursor];
            kernel.clock=kernel.event_time+uint64_t(next.tick-kernel.tick)*kernel.tempo;
            kernel.advance(0,backend);
        }
        ready=kernel.error==sequence::Error::None && count!=0;
        return ready;
    }
};
template<uint16_t Capacity,uint32_t Events=0,uint32_t References=0> struct Storage {
    static_assert(Capacity>0 && Capacity<=MaxStarts);
    Key keys[Capacity]{};
    synth::State states[Capacity]{};
    uint16_t buckets[bucket_count(Capacity)]{};
    EventStart event_map[Events?Events:1]{};
    uint16_t references[References?References:1]{};
    uint16_t pitches[Capacity]{};
    Cache cache{keys,states,buckets,Capacity,bucket_count(Capacity)};
    Storage(){cache.pitches=pitches;if(Events){cache.event_map=event_map;cache.references=references;cache.event_capacity=Events;cache.reference_capacity=References;}}
};
}
