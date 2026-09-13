#pragma once
// Included inside namespace epok after the legacy service's Instance definition.
// Only EPSB v2 uses this backend; v1 keeps its original gain/envelope contract.
namespace psx_audio {
inline void prepare_allocation(){
    if(allocation_current)return;
    allocation_free_mask=0;allocation_music_count=allocation_ceiling=allocation_free_count=0;
    for(auto& count:allocation_own_count)count=0;
    for(const auto& i:instances)if((i.phase==Phase::Active || i.phase==Phase::Queued) && i.asset && i.asset->voices()>allocation_ceiling)
        allocation_ceiling=uint8_t(i.asset->voices());
    for(int p=0;p<24;++p){
        const auto& voice=audio_voices[p];
        if(!voice.owner){allocation_free_mask|=1u<<p;++allocation_free_count;}
        else if(voice.sequence>=0){++allocation_music_count;
            if(physical[p].generation==instances[voice.sequence].generation)++allocation_own_count[voice.sequence];}
    }
    allocation_current=true;
}
inline instrument::synth::Controls instrument_controls(const sequence::Channel& c) {
    return instrument::preparation::controls(c);
}
inline bool Instance::advance_library(int number) {
    auto& v=physical[number];
    if(v.pending)return true;
    const uint32_t elapsed=now_us-v.synthesis_us;
    v.synthesis_us=now_us;v.control_due_us=now_us+InstrumentControlPeriodUs;
    if(v.synthesis.advance(elapsed).finished){finish_layer(number);return false;}
    return true;
}
inline void Instance::update_library(int number,const sequence::Channel& channel,bool changed) {
    auto& v=physical[number];
    // Apply elapsed time with the OLD controls before a new MIDI operation.
    if(changed && !advance_library(number))return;
    if(changed && v.synthesis.update_controls(instrument_controls(channel))!=instrument::synth::Error::None){
        music_sequence_stats.error=10;kernel.error=sequence::Error::InvalidInput;finish_layer(number);return;
    }
    const auto& out=v.synthesis.output();
    const bool sends=asset->bank->library().reverb_preset() && out.reverb_send_permille>0 && (!channel.reverb_set || channel.reverb!=0);
    v.reverb_send=sends;
    reverb_resource.send(uint8_t(number),sends && !v.pending);
    if(out.pitch_cents_x100==v.last_cents && parameters.pitch==v.last_source_pitch &&
        out.gain_q15==v.last_synth_gain && out.pan_permille==v.last_pan && parameters.volume==v.last_source_volume)return;
    v.last_synth_gain=out.gain_q15;v.last_pan=out.pan_permille;v.last_source_volume=parameters.volume;
    const uint16_t previous_pitch=v.pitch,previous_left=v.left,previous_right=v.right;
    if(out.pitch_cents_x100!=v.last_cents || parameters.pitch!=v.last_source_pitch){
        v.last_cents=out.pitch_cents_x100;v.last_source_pitch=parameters.pitch;
        const int64_t cents=out.pitch_cents_x100;
        int64_t octave=cents/120000,remainder=cents%120000;
        if(remainder<0){remainder+=120000;--octave;}
        const unsigned position=unsigned(remainder)/100,part=unsigned(remainder)%100;
        const uint32_t ratio=cent_ratio[position]+(cent_ratio[position+1]-cent_ratio[position])*part/100;
        const auto& zone=asset->bank->library().zone(v.library_zone);
        const uint64_t denominator=uint64_t(44100)<<20;
        const uint64_t base=uint64_t(asset->bank->sample(zone.sample).rate)*ratio*parameters.pitch;
        const uint64_t ceiling=uint64_t(0x4000)*denominator;
        uint64_t pitch=0;bool high=false;
        if(octave>=0){
            const auto shift=uint64_t(octave);
            high=shift>=64 || base>((ceiling-1)>>unsigned(shift));
            if(!high)pitch=(base<<unsigned(shift))/denominator;
        }else{
            const auto shift=uint64_t(-(octave+1))+1;
            pitch=shift>=64?0:(base>>unsigned(shift))/denominator;
        }
        if(high || pitch<1 || pitch>0x3fff){++music_sequence_stats.pitch_clamps;pitch=high || pitch>0x3fff?0x3fff:1;}
        v.pitch=uint16_t(pitch);
    }
    // Synth already includes velocity, channel volume/expression and tuning.
    // Apply only AudioSource gain and the same linear pan law as Target Preview.
    const uint32_t gain=uint32_t(out.gain_q15)*parameters.volume/8192;
    const uint32_t pan=uint32_t(out.pan_permille+500);
    v.left=uint16_t(gain*(1000-pan)/1000);v.right=uint16_t(gain*pan/1000);
    v.level=FullLevel;
    if(!v.pending){
        if(v.pitch!=previous_pitch)SPU_VOICES[number].sampleRate=v.pitch;
        if(v.left!=previous_left)SPU_VOICES[number].volumeLeft=v.left;
        if(v.right!=previous_right)SPU_VOICES[number].volumeRight=v.right;
    }
}
inline void Instance::release_library(uint16_t note) {
    for(uint32_t mask=note_voices[note];mask;mask&=mask-1){
        const int n=first_voice(mask);
        if(!owns(n,note))continue;
        auto& v=physical[n];if(v.released)continue;
        if(!advance_library(n))continue;
        v.released=true;v.synthesis.release();
        const auto& z=asset->bank->library().zone(v.library_zone);
        if(z.loop_mode==3 && !v.pending){
            const auto& sample=asset->bank->sample(z.sample);
            // Cook removes ADPCM loop-start flags for UntilRelease, so an early
            // release cannot be overwritten when hardware later enters the loop.
            SPU_VOICES[n].sampleRepeatAddr=asset->bank->addresses[z.sample]+uint16_t(sample.loop_end/28*2);
        }
        if(v.synthesis.output().finished)finish_layer(n);
    }
}
#if defined(__mips__) && defined(__GNUC__)
// Prepared-note admission is the other measured IRQ hot path. Restrict the
// compiler's stronger optimization to the library backend.
__attribute__((optimize("O3")))
#endif
inline bool Instance::start_library(uint16_t note,const sequence::Note& n,const sequence::Channel& channel) {
    const auto bank=asset->bank->library();
    uint16_t matching[128];uint16_t count=0;
    const instrument::synth::State* prepared[24];
    uint16_t prepared_pitch[24];
    const bool mapped=asset->prepared && asset->prepared->ready && asset->prepared->event_map && kernel.events==asset->events();
    if(mapped){
        if(!kernel.cursor || kernel.cursor>asset->prepared->event_capacity){music_sequence_stats.error=12;return false;}
        const auto& event=asset->prepared->event_map[kernel.cursor-1];count=event.count;
        if(count<=24)for(uint16_t z=0;z<count;++z){
            const auto entry=asset->prepared->references[event.offset+z];
            matching[z]=uint16_t(asset->prepared->keys[entry].words[0]);
            prepared[z]=asset->prepared->states+entry;
            prepared_pitch[z]=asset->prepared->pitches?asset->prepared->pitches[entry]:0;
        }
    }else for(uint16_t z=0;z<bank.zone_count();++z){
        const auto& zone=bank.zone(z);
        if(zone.bank==channel.bank && zone.program==channel.program && zone.percussion==uint8_t(n.channel==9) &&
            n.key>=zone.key_lo && n.key<=zone.key_hi && n.velocity>=zone.velocity_lo && n.velocity<=zone.velocity_hi)matching[count++]=z;
    }
    if(!count)return false;
    if(count>24){++music_sequence_stats.denied_notes;kernel.retire(note);return true;}
    if(asset->prepared && !mapped)for(uint16_t z=0;z<count;++z){
        prepared[z]=asset->prepared->find(matching[z],n.key,n.velocity,channel);
        if(!prepared[z]){music_sequence_stats.error=12;return false;}
        prepared_pitch[z]=asset->prepared->pitches?asset->prepared->pitches[prepared[z]-asset->prepared->states]:0;
    }
    if(!mapped)allocation_current=false; // Host injection may change owners between calls.
    prepare_allocation();
    uint32_t exclusive=0,free_mask=allocation_free_mask;unsigned free_count=allocation_free_count;
    unsigned own_count=allocation_own_count[index],music_count=allocation_music_count;
    bool has_exclusive=false;
    for(uint16_t z=0;z<count;++z)has_exclusive|=bank.zone(matching[z]).exclusive_class!=0;
    const uint8_t ceiling=allocation_ceiling;
    if(has_exclusive)for(int p=0;p<24;++p){
        const auto& av=audio_voices[p];if(!av.owner)continue;
        const bool musical=av.sequence>=0;
        if(!musical || av.sequence!=index || physical[p].generation!=generation || !physical[p].library || kernel.notes[av.note].channel!=n.channel)continue;
        const auto& old=bank.zone(physical[p].library_zone);
        for(uint16_t z=0;z<count;++z){
            const auto& incoming=bank.zone(matching[z]);
            if(incoming.exclusive_class && incoming.exclusive_class==old.exclusive_class && incoming.bank==old.bank && incoming.program==old.program && incoming.percussion==old.percussion){exclusive|=1u<<p;break;}
        }
    }
    // Exclusion removes complete old notes. Include every layer in the staged
    // reservation without mutating playback until the complete new group fits.
    if(exclusive){
        for(uint32_t mask=exclusive;mask;mask&=mask-1)exclusive|=note_voices[audio_voices[first_voice(mask)].note];
        for(uint32_t mask=exclusive;mask;mask&=mask-1){++free_count;--own_count;--music_count;}
        free_mask|=exclusive;
    }
    instrument::allocation::Plan plan;
    if(free_count>=count && own_count+count<=asset->voices() && music_count+count<=ceiling){
        for(uint16_t added=0;added<count;++added){const uint32_t bit=free_mask&(~free_mask+1);plan.start_mask|=bit;free_mask^=bit;}
        plan.fits=true;
    }else{
        instrument::allocation::Slot slots[24]{};
        for(int p=0;p<24;++p){
            const auto& av=audio_voices[p];if(!av.owner)continue;
            const bool musical=av.sequence>=0;
            slots[p]={true,musical,uint8_t(musical?av.sequence:0),av.note,av.priority,physical[p].generation,av.started};
        }
        if(exclusive){
            for(int p=0;p<24;++p)if(exclusive&(1u<<p))for(int q=0;q<24;++q)if(instrument::allocation::same_group(slots[p],slots[q]))exclusive|=1u<<q;
            for(int p=0;p<24;++p)if(exclusive&(1u<<p))slots[p]={};
        }
        plan=instrument::allocation::reserve(slots,{index,parameters.priority,uint8_t(count),uint8_t(asset->voices()),ceiling,generation});
    }
    if(!plan.fits){++music_sequence_stats.denied_notes;kernel.retire(note);return true;}
    bool empty_starts=true;
    if(!asset->prepared)for(int p=0;p<24;++p)if((plan.start_mask&(1u<<p)) && audio_voices[p].owner)empty_starts=false;
    if(!asset->prepared && empty_starts){
        const auto controls=instrument_controls(channel);
        // Preflight directly into unowned slots. They remain unpublished until
        // every layer succeeds, avoiding a second parameter build per voice.
        uint16_t z=0;
        for(int p=0;p<24;++p)if(plan.start_mask&(1u<<p)){
            if(physical[p].synthesis.start_validated(bank,matching[z++],n.key,n.velocity,controls)!=instrument::synth::Error::None){
                for(int q=0;q<24;++q)if(plan.start_mask&(1u<<q))physical[q].synthesis.reset();
                music_sequence_stats.error=10;return false;
            }
        }
    }else if(!asset->prepared)for(uint16_t z=0;z<count;++z){
        instrument::synth::State probe;
        if(probe.start_validated(bank,matching[z],n.key,n.velocity,instrument_controls(channel))!=instrument::synth::Error::None){music_sequence_stats.error=10;return false;}
    }
    if(exclusive)for(int p=0;p<24;++p)if((exclusive&(1u<<p)) && audio_voices[p].sequence==index){
        const auto old=audio_voices[p].note;cut(old);kernel.retire(old);
    }
    if(plan.evict_mask)for(int p=0;p<24;++p)if((plan.evict_mask&(1u<<p)) && audio_voices[p].owner){
        if(audio_voices[p].sequence>=0)stolen(p);
        else {audio_keyoff(p);audio_voices[p]={};physical[p].reset_metadata();++music_sequence_stats.steals;}
    }
    uint16_t zone=0;
    for(uint32_t mask=plan.start_mask;mask;mask&=mask-1){
        const int p=first_voice(mask);
        audio_keyoff(p);const auto off_ticks=counter_ticks();
        audio_voices[p]={owner,audio_frame,clip,int8_t(index),uint8_t(note),parameters.priority};
        note_voices[note]|=1u<<p;
        auto& v=physical[p];
        if(asset->prepared){v.reset_metadata();v.synthesis.copy_initial(*prepared[zone]);}
        else if(empty_starts){v.reset_metadata();}
        else v={};
        if(asset->prepared && parameters.pitch==4096 && prepared_pitch[zone]){
            v.pitch=prepared_pitch[zone];v.last_cents=v.synthesis.output().pitch_cents_x100;v.last_source_pitch=4096;
        }
        v.library=true;v.library_zone=matching[zone++];
        v.generation=generation;v.off_us=now_us;v.off_ticks=off_ticks;v.pending=true;v.key=n.key;v.velocity=n.velocity;
        const uint64_t late=kernel.clock-kernel.event_time;
        v.due_us=now_us-uint32_t(late>UINT32_MAX?late/kernel.ppqn:uint32_t(late)/kernel.ppqn);
        v.looping=bank.zone(v.library_zone).loop_mode!=0;
        if(!asset->prepared && !empty_starts)v.synthesis.start_validated(bank,v.library_zone,n.key,n.velocity,instrument_controls(channel));
        update_library(p,channel,false);
    }
    if(allocation_current){allocation_free_mask&=~plan.start_mask;allocation_free_count-=uint8_t(count);allocation_music_count+=uint8_t(count);allocation_own_count[index]+=uint8_t(count);}
    return true;
}
}

inline uint32_t sequence_flush_starts(){
    using namespace psx_audio;
    uint32_t started=0;
    for(int n=0;n<24;++n){
        auto& v=physical[n];const auto& av=audio_voices[n];
        if(!v.pending || av.sequence<0)continue;
        auto& i=instances[av.sequence];
        if(v.generation!=i.generation || i.phase!=Phase::Active)continue;
        // A library start can be sent in this IRQ once the actual hardware
        // key-off is at least 30 us old. Legacy keeps its service-boundary path.
        if(uint32_t(now_us-v.off_us)<30){
#ifdef EPOK_SEQUENCE_HOST_TEST
            continue;
#else
            if(!v.library || uint16_t(counter_ticks()-v.off_ticks)<128)continue;
#endif
        }
        const auto* zone=v.library?&i.asset->bank->library().zone(v.library_zone):nullptr;
        const auto sample_index=v.library?zone->sample:v.zone->sample;
        const auto& sample=i.asset->bank->sample(sample_index);
        auto& hw=SPU_VOICES[n];hw.sampleStartAddr=i.asset->bank->addresses[sample_index];
        const bool tail=v.library && v.released && zone->loop_mode==3;
        hw.sampleRepeatAddr=hw.sampleStartAddr+uint16_t((tail?sample.loop_end:sample.loop_start)/28*2);
        hw.adsrLo=0x000f;hw.adsrHi=0;hw.sampleRate=v.pitch;
        // Library gain already includes its software envelope; only the v1
        // backend still has a separate legacy level to multiply here.
        hw.volumeLeft=v.library?v.left:uint16_t(uint32_t(v.left)*(v.level>>16)/32767);
        hw.volumeRight=v.library?v.right:uint16_t(uint32_t(v.right)*(v.level>>16)/32767);
        ++sequence_timing_stats.key_ons;
        v.pending=false;v.started_us=now_us;started|=1u<<n;
        // Give a new note its first envelope step at the next 1 kHz service,
        // then reduce steady modulation traffic independently of note timing.
        v.synthesis_us=now_us;v.control_due_us=now_us+1000;
        if(v.library)reverb_resource.send(uint8_t(n),v.reverb_send);
    }
    // KON is a shared write-only latch, not a per-voice command queue. Submit
    // all prepared voices together so a later write cannot replace an earlier
    // chord/layer before the real SPU samples the register.
    if(started&0xffff)SPU_KEY_ON_LOW=uint16_t(started);
    if(started>>16)SPU_KEY_ON_HIGH=uint16_t(started>>16);
    uint32_t write_delay=0;
#ifndef EPOK_SEQUENCE_HOST_TEST
    write_delay=(uint32_t(uint16_t(counter_ticks()-sequence_irq_begin_ticks))*625+2645)/2646;
#endif
    for(uint32_t mask=started;mask;mask&=mask-1){
        const uint32_t delay=uint32_t(now_us-physical[first_voice(mask)].due_us)+write_delay;
        if(delay>sequence_timing_stats.max_key_on_delay_us)sequence_timing_stats.max_key_on_delay_us=delay;
    }
    return started;
}
