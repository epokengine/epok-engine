#pragma once
// Included inside epok. Native EPSQ execution is register dispatch only: no
// region lookup, note matching, envelope fitting, LFO or SoundFont arithmetic.
namespace psx_audio {
inline void native_flush_keyoffs(){
    if(native_keyoffs&0xffff)SPU_KEY_OFF_LOW=uint16_t(native_keyoffs);
    if(native_keyoffs>>16)SPU_KEY_OFF_HIGH=uint16_t(native_keyoffs>>16);
    native_keyoffs=0;
}
inline void Instance::native_parameters(int n){
    auto& v=physical[n];
    v.pitch=uint16_t((uint32_t(v.native_pitch)*parameters.pitch)>>12);if(v.pitch>0x3fff)v.pitch=0x3fff;if(!v.pitch)v.pitch=1;
    v.left=uint16_t((uint32_t(v.native_left)*parameters.volume)>>12);
    v.right=uint16_t((uint32_t(v.native_right)*parameters.volume)>>12);
    if(!v.pending){auto& hw=SPU_VOICES[n];hw.sampleRate=v.pitch;hw.volumeLeft=v.left;hw.volumeRight=v.right;}
}
inline void Instance::native_advance(uint32_t elapsed){
    using namespace native_music;
    native_clock+=elapsed;const auto stream=asset->native();const auto* commands=stream.commands();
    unsigned processed=0;
    while(native_cursor<stream.count() && commands[native_cursor].time<=native_clock){
        if(++processed>2048){music_sequence_stats.error=13;retire(*this);return;}
        const auto e=commands[native_cursor++];
        if(e.op==Mark){native_loop_cursor=native_cursor;native_loop_time=e.time;continue;}
        if(e.op==Loop){
            for(unsigned n=0;n<sequence::Kernel::MaxVoices;++n)if(note_voices[n])cut(uint16_t(n));
            native_clock-=e.time-native_loop_time;native_cursor=native_loop_cursor;++music_sequence_stats.loops;continue;
        }
        if(e.op==End){++music_sequence_stats.ends;retire(*this);return;}
        if(e.op==Group){
            instrument::allocation::Slot slots[24]{};uint8_t ceiling=uint8_t(asset->voices());
            for(const auto& i:instances)if((i.phase==Phase::Active || i.phase==Phase::Queued) && i.asset && i.asset->voices()>ceiling)ceiling=uint8_t(i.asset->voices());
            for(int p=0;p<24;++p){const auto& av=audio_voices[p];if(av.owner)slots[p]={true,av.sequence>=0,uint8_t(av.sequence>=0?av.sequence:0),av.note,av.priority,physical[p].generation,av.started};}
            const auto plan=instrument::allocation::reserve(slots,{index,parameters.priority,uint8_t(e.value),uint8_t(asset->voices()),ceiling,generation});
            if(!plan.fits){++music_sequence_stats.denied_notes;for(unsigned z=0;z<e.value;++z)native_lanes[commands[native_cursor++].lane]=-1;continue;}
            for(int p=0;p<24;++p)if((plan.evict_mask&(1u<<p)) && audio_voices[p].owner){
                if(audio_voices[p].sequence>=0)stolen(p);
                else{native_keyoffs|=1u<<p;audio_voices[p]={};physical[p].reset_metadata();++music_sequence_stats.steals;}
            }
            uint32_t mask=plan.start_mask;
            for(unsigned z=0;z<e.value;++z){const auto start=commands[native_cursor++];const int p=first_voice(mask);mask&=mask-1;
                const auto& patch=stream.patches()[start.value];auto& v=physical[p];v.reset_metadata();
                native_keyoffs|=1u<<p;v.native_tone=&patch;v.native_lane=start.lane;v.native_pitch=patch.pitch;v.native_left=patch.left;v.native_right=patch.right;
                v.generation=generation;v.off_us=now_us;v.off_ticks=counter_ticks();v.pending=true;v.looping=patch.loop!=0;v.reverb_send=patch.send;
                v.due_us=now_us-(native_clock-start.time);
                audio_voices[p]={owner,audio_frame,clip,int8_t(index),e.lane,parameters.priority};
                note_voices[e.lane]|=1u<<p;native_lanes[start.lane]=int8_t(p);native_parameters(p);
            }
            allocation_current=false;continue;
        }
        const int p=native_lanes[e.lane];if(p<0)continue;
        auto& v=physical[p];
        if(audio_voices[p].sequence!=index || v.generation!=generation || !v.native_tone || v.native_lane!=e.lane){native_lanes[e.lane]=-1;continue;}
        switch(e.op){
            case Cut:finish_layer(p);break;
            case Release:
                v.released=true;
                if(!v.pending){native_keyoffs|=1u<<p;v.envelope=1;
                    if(v.native_tone->loop==3){const auto s=v.native_tone->sample;SPU_VOICES[p].sampleRepeatAddr=asset->bank->addresses[s]+uint16_t(asset->bank->sample(s).loop_end/28*2);}}
                break;
            case Pitch:v.native_pitch=e.value;native_parameters(p);break;
            case Left:v.native_left=e.value;native_parameters(p);break;
            case Right:v.native_right=e.value;native_parameters(p);break;
            case Send:v.reverb_send=e.value!=0;if(!v.pending)reverb_resource.send(uint8_t(p),v.reverb_send);break;
            default:music_sequence_stats.error=13;retire(*this);return;
        }
    }
}
}
